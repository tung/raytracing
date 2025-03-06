use crate::color::*;
use crate::random::*;
use crate::ray::*;
use crate::scene::*;
use crate::vec3::*;

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
    render_passes: f64,
    colors: Vec<Color>,
    pixels: Vec<u8>,
    image_width: usize,
    max_depth: u16,
    center: Vec3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    pixel00_loc: Vec3,
}

impl Camera {
    pub fn new(image_width: f64, image_height: f64, max_depth: u16, rng_seed: u64) -> Self {
        let colors = vec![Color::new(0.0, 0.0, 0.0); image_width as usize * image_height as usize];
        let pixels = vec![0_u8; 4 * image_width as usize * image_height as usize];

        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * image_width / image_height;
        let center = Vec3::new(0.0, 0.0, 0.0);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, viewport_height, 0.0);

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        let pixel_delta_u = viewport_u / image_width;
        let pixel_delta_v = viewport_v / image_height;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left =
            center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Self {
            rng: Rng::new(rng_seed),
            render_passes: 1.0,
            colors,
            pixels,
            image_width: image_width as _,
            max_depth,
            center,
            pixel_delta_u,
            pixel_delta_v,
            pixel00_loc,
        }
    }

    pub fn get_pixels(&self) -> &[u8] {
        &self.pixels
    }

    fn get_ray(&mut self, i: f64, j: f64) -> Ray {
        // Construct a camera ray originating from the origin and directed at a randomly-sampled
        // point around the pixel location i, j.

        let offset = sample_square(&mut self.rng);
        let pixel_sample = self.pixel00_loc
            + ((i + offset.x()) * self.pixel_delta_u)
            + ((j + offset.y()) * self.pixel_delta_v);

        Ray {
            pos: self.center,
            dir: pixel_sample - self.center,
        }
    }

    pub fn render(&mut self, scene: &Scene) {
        let mut colors = Vec::new();
        let mut pixels = Vec::new();

        std::mem::swap(&mut colors, &mut self.colors);
        std::mem::swap(&mut pixels, &mut self.pixels);

        let color_rows = colors.chunks_exact_mut(self.image_width);
        let pixel_rows = pixels.chunks_exact_mut(self.image_width * 4);

        for (y, (color_row, pixel_row)) in color_rows.zip(pixel_rows).enumerate() {
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
        }

        std::mem::swap(&mut colors, &mut self.colors);
        std::mem::swap(&mut pixels, &mut self.pixels);

        self.render_passes += 1.0;
    }
}
