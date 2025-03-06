use crate::color::*;
use crate::random::*;
use crate::ray::*;
use crate::scene::*;
use crate::vec3::*;

pub struct CameraOptions {
    pub aspect_ratio: f64,  // Ratio of image width over height
    pub image_width: u16,   // Rendered image width in pixel count
    pub max_depth: u16,     // Maximum number of ray bounces into scene
    pub vfov: f64,          // Vertical view angle (field of view)
    pub lookfrom: Vec3,     // Point camera is looking from
    pub lookat: Vec3,       // Point camera is looking at
    pub vup: Vec3,          // Camera-relative "up" direction
    pub defocus_angle: f64, // Variation angle of rays through each pixel.
    pub focus_dist: f64,    // Distance from camera lookfrom point to plane of perfect focus.
}

impl Default for CameraOptions {
    fn default() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            max_depth: 10,
            vfov: 90.0,
            lookfrom: Vec3::new(0.0, 0.0, 0.0),
            lookat: Vec3::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,
        }
    }
}

fn sample_square(rng: &mut Rng) -> Vec3 {
    // Returns the vector to a random point in the [-0.5,-0.5] to [+0.5,+0.5] unit square.
    Vec3::new(rng.random_f64() - 0.5, rng.random_f64() - 0.5, 0.0)
}

fn ray_color(rng: &mut Rng, depth: u16, r: &Ray, scene: &Scene) -> Color {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(rec) = scene.hit(r, 0.001, f64::INFINITY) {
        return if let Some(sc_rec) = rec.mat.scatter(rng, r, &rec) {
            sc_rec.attenuation * ray_color(rng, depth - 1, &sc_rec.scattered, scene)
        } else {
            Color::new(0.0, 0.0, 0.0)
        };
    }

    let unit_direction = r.dir.unit();
    let a = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}

pub struct Camera {
    rng: Rng,
    start_row: usize,
    render_passes: f64,
    colors: Vec<Color>,
    pixels: Vec<u8>,
    image_width: usize,
    image_height: usize,
    max_depth: u16,
    center: Vec3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    pixel00_loc: Vec3,
    defocus_angle: f64,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    pub fn new(rng: Rng, options: CameraOptions) -> Self {
        let i_width_usize = options.image_width as usize;
        let i_height_usize = usize::max(
            1,
            (options.image_width as f64 / options.aspect_ratio) as usize,
        );

        let colors = vec![Color::new(0.0, 0.0, 0.0); i_width_usize * i_height_usize];
        let pixels = vec![0_u8; 4 * i_width_usize * i_height_usize];

        let image_width = i_width_usize as f64;
        let image_height = i_height_usize as f64;

        let center = options.lookfrom;

        // Determine viewport dimensions.
        let theta = options.vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * options.focus_dist;
        let viewport_width = viewport_height * image_width / image_height;

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        let w = (options.lookfrom - options.lookat).unit();
        let u = options.vup.cross(w).unit();
        let v = w.cross(u);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = viewport_width * u; // Vector across viewport horizontal edge
        let viewport_v = viewport_height * -v; // Vector across viewport vertical edge

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        let pixel_delta_u = viewport_u / image_width;
        let pixel_delta_v = viewport_v / image_height;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left =
            center - options.focus_dist * w - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        // Calculate the defocus disk basis vectors.
        let defocus_radius = options.focus_dist * (options.defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u = defocus_radius * u;
        let defocus_disk_v = defocus_radius * v;

        Self {
            rng,
            start_row: 0,
            render_passes: 1.0,
            colors,
            pixels,
            image_width: i_width_usize,
            image_height: i_height_usize,
            max_depth: options.max_depth,
            center,
            pixel_delta_u,
            pixel_delta_v,
            pixel00_loc,
            defocus_angle: options.defocus_angle,
            defocus_disk_u,
            defocus_disk_v,
        }
    }

    pub fn get_image_height(&self) -> usize {
        self.image_height
    }

    pub fn get_pixels(&self) -> &[u8] {
        &self.pixels
    }

    fn defocus_disk_sample(&mut self) -> Vec3 {
        // Returns a random point in the camera defocus disk.
        let p = Vec3::random_in_unit_disk(&mut self.rng);
        self.center + p.x() * self.defocus_disk_u + p.y() * self.defocus_disk_v
    }

    fn get_ray(&mut self, i: f64, j: f64) -> Ray {
        // Construct a camera ray originating from the defocus disk and directed at a
        // randomly-sampled point around the pixel location i, j.

        let offset = sample_square(&mut self.rng);
        let pixel_sample = self.pixel00_loc
            + ((i + offset.x()) * self.pixel_delta_u)
            + ((j + offset.y()) * self.pixel_delta_v);

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_sample - ray_origin;

        Ray {
            pos: ray_origin,
            dir: ray_direction,
        }
    }

    pub fn render(&mut self, scene: &Scene, until: f64) {
        let mut colors = Vec::new();
        let mut pixels = Vec::new();

        std::mem::swap(&mut colors, &mut self.colors);
        std::mem::swap(&mut pixels, &mut self.pixels);

        let color_rows = colors.chunks_exact_mut(self.image_width);
        let pixel_rows = pixels.chunks_exact_mut(self.image_width * 4);

        for (y, (color_row, pixel_row)) in
            color_rows.zip(pixel_rows).enumerate().skip(self.start_row)
        {
            let cs = color_row.iter_mut();
            let ps = pixel_row.chunks_exact_mut(4);

            for (x, (c, p)) in cs.zip(ps).enumerate() {
                let ray = self.get_ray(x as f64, y as f64);
                *c += ray_color(&mut self.rng, self.max_depth, &ray, scene);
                p[0] = ((c.r() / self.render_passes).sqrt() * 255.999) as u8;
                p[1] = ((c.g() / self.render_passes).sqrt() * 255.999) as u8;
                p[2] = ((c.b() / self.render_passes).sqrt() * 255.999) as u8;
                p[3] = 255;
            }

            self.start_row += 1;
            if self.start_row >= self.image_height {
                self.render_passes += 1.0;
                self.start_row = 0;
            }
            if miniquad::date::now() >= until {
                break;
            }
        }

        std::mem::swap(&mut colors, &mut self.colors);
        std::mem::swap(&mut pixels, &mut self.pixels);
    }
}
