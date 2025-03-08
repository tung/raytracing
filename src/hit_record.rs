use crate::material::*;
use crate::ray::*;
use crate::vec3::*;

use std::sync::Arc;

pub struct HitRecord<'m> {
    pub p: Vec3,
    pub normal: Vec3,
    pub mat: &'m Arc<Material>,
    pub t: f64,
    pub front_face: bool,
}

impl<'m> HitRecord<'m> {
    pub fn new(r: &Ray, t: f64, outward_normal: Vec3, mat: &'m Arc<Material>) -> Self {
        // NOTE: The parameter `outward_normal` is assumed to have unit length.

        let front_face = r.dir.dot(outward_normal) < 0.0;

        Self {
            p: r.at(t),
            normal: if front_face {
                outward_normal
            } else {
                -outward_normal
            },
            mat,
            t,
            front_face,
        }
    }
}
