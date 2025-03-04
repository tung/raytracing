use crate::hit_record::*;
use crate::ray::*;
use crate::sphere::*;

pub struct Scene {
    spheres: Vec<Sphere>,
}

impl Scene {
    pub fn new() -> Self {
        Scene { spheres: vec![] }
    }

    pub fn add(&mut self, sphere: Sphere) {
        self.spheres.push(sphere);
    }

    pub fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64) -> Option<HitRecord> {
        let mut hit_rec: Option<HitRecord> = None;
        let mut closest_so_far = ray_tmax;

        for sphere in &self.spheres {
            if let Some(rec) = sphere.hit(r, ray_tmin, closest_so_far) {
                closest_so_far = rec.t;
                hit_rec = Some(rec);
            }
        }

        hit_rec
    }
}
