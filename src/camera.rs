use crate::color::*;
use crate::ray::*;
use crate::vec3::*;

fn hit_sphere(center: &Vec3, radius: f64, r: &Ray) -> bool {
    let oc = *center - r.pos;
    let a = r.dir.dot(r.dir);
    let b = -2.0 * r.dir.dot(oc);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    discriminant >= 0.0
}

fn ray_color(r: &Ray) -> Color {
    if hit_sphere(&Vec3::new(0.0, 0.0, -1.0), 0.5, r) {
        return Color::new(1.0, 0.0, 0.0);
    }

    let unit_direction = r.dir.unit();
    let a = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}

pub struct Camera {
    pixels: Vec<u8>,
    image_width: usize,
    image_height: usize,
    camera_center: Vec3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    pixel00_loc: Vec3,
}

impl Camera {
    pub fn new(image_width: f64, image_height: f64) -> Self {
        let pixels = vec![0_u8; 4 * image_width as usize * image_height as usize];

        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * image_width / image_height;
        let camera_center = Vec3::new(0.0, 0.0, 0.0);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, viewport_height, 0.0);

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        let pixel_delta_u = viewport_u / image_width;
        let pixel_delta_v = viewport_v / image_height;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left =
            camera_center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Self {
            pixels,
            image_width: image_width as _,
            image_height: image_height as _,
            camera_center,
            pixel_delta_u,
            pixel_delta_v,
            pixel00_loc,
        }
    }

    pub fn get_pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn render(&mut self) {
        let coords =
            (0..self.image_height).flat_map(|y| std::iter::repeat(y).zip(0..self.image_width));

        for ((y, x), p) in coords.zip(self.pixels.chunks_exact_mut(4)) {
            let x = x as f64;
            let y = y as f64;
            let pixel_center = self.pixel00_loc + x * self.pixel_delta_u + y * self.pixel_delta_v;
            let ray_direction = pixel_center - self.camera_center;
            let ray = Ray {
                pos: self.camera_center,
                dir: ray_direction,
            };
            let pixel_color = ray_color(&ray);

            p[0] = (pixel_color.r() * 255.999).floor() as u8;
            p[1] = (pixel_color.g() * 255.999).floor() as u8;
            p[2] = (pixel_color.b() * 255.999).floor() as u8;
            p[3] = 255;
        }
    }
}
