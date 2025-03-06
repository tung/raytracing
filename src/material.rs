use crate::color::*;
use crate::hit_record::*;
use crate::random::*;
use crate::ray::*;
use crate::vec3::*;

pub struct ScatterRecord {
    pub attenuation: Color,
    pub scattered: Ray,
}

pub enum Material {
    Lambertian { albedo: Color },
    Metal { albedo: Color, fuzz: f64 },
    Dieletric { refraction_index: f64 },
}

impl Material {
    pub fn lambertian(albedo: Color) -> Self {
        Self::Lambertian { albedo }
    }

    pub fn metal(albedo: Color, fuzz: f64) -> Self {
        Self::Metal { albedo, fuzz }
    }

    pub fn dielectric(refraction_index: f64) -> Self {
        Self::Dieletric { refraction_index }
    }

    pub fn scatter(&self, rng: &mut Rng, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        match self {
            Self::Lambertian { albedo } => {
                let mut scatter_direction = rec.normal + Vec3::random_unit_vector(rng);

                // Catch degenerate scatter direction.
                if scatter_direction.near_zero() {
                    scatter_direction = rec.normal;
                }

                Some(ScatterRecord {
                    attenuation: *albedo,
                    scattered: Ray {
                        pos: rec.p,
                        dir: scatter_direction,
                    },
                })
            }
            Self::Metal { albedo, fuzz } => {
                let mut reflected = r_in.dir.reflect(rec.normal);
                reflected = reflected.unit() + *fuzz * Vec3::random_unit_vector(rng);
                if reflected.dot(rec.normal) > 0.0 {
                    Some(ScatterRecord {
                        attenuation: *albedo,
                        scattered: Ray {
                            pos: rec.p,
                            dir: reflected,
                        },
                    })
                } else {
                    None
                }
            }
            Self::Dieletric { refraction_index } => {
                let ri = if rec.front_face {
                    1.0 / *refraction_index
                } else {
                    *refraction_index
                };

                let unit_direction = r_in.dir.unit();
                let refracted = unit_direction.refract(rec.normal, ri);

                Some(ScatterRecord {
                    attenuation: Color::new(1.0, 1.0, 1.0),
                    scattered: Ray {
                        pos: rec.p,
                        dir: refracted,
                    },
                })
            }
        }
    }
}
