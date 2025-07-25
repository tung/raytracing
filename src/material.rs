use crate::color::*;
use crate::hit_record::*;
use crate::random::*;
use crate::ray::*;
use crate::vec3::*;

pub struct ScatterRecord {
    pub attenuation: Color,
    pub scattered: Ray,
}

fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
    // Use Schlick's approximation for reflectance.
    let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
    r0 *= r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
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
                let cos_theta = f64::min((-unit_direction).dot(rec.normal), 1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

                let cannot_refract = ri * sin_theta > 1.0;
                let direction = if cannot_refract || reflectance(cos_theta, ri) > rng.random_f64() {
                    unit_direction.reflect(rec.normal)
                } else {
                    unit_direction.refract(rec.normal, ri)
                };

                Some(ScatterRecord {
                    attenuation: Color::new(1.0, 1.0, 1.0),
                    scattered: Ray {
                        pos: rec.p,
                        dir: direction,
                    },
                })
            }
        }
    }
}
