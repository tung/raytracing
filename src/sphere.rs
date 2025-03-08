use crate::hit_record::*;
use crate::material::*;
use crate::ray::*;
use crate::vec3::*;

use std::sync::Arc;

pub struct Sphere {
    center: Vec3,
    radius: f64,
    mat: Arc<Material>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f64, mat: Arc<Material>) -> Self {
        Self {
            center,
            radius,
            mat,
        }
    }

    pub fn hit<'s>(&'s self, r: &Ray, ray_tmin: f64, ray_tmax: f64) -> Option<HitRecord<'s>> {
        let oc = self.center - r.pos;
        let a = r.dir.length_squared();
        let h = r.dir.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        let mut root = (h - sqrtd) / a;
        if root <= ray_tmin || root >= ray_tmax {
            root = (h + sqrtd) / a;
            if root <= ray_tmin || root >= ray_tmax {
                return None;
            }
        }

        Some(HitRecord::new(
            r,
            root,
            (r.at(root) - self.center) / self.radius,
            &self.mat,
        ))
    }
}
