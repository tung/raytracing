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
}

impl Material {
    pub fn lambertian(albedo: Color) -> Self {
        Self::Lambertian { albedo }
    }

    pub fn scatter(&self, rng: &mut Rng, _r_in: &Ray, rec: &HitRecord) -> ScatterRecord {
        match self {
            Self::Lambertian { albedo } => {
                let scatter_direction = Vec3::random_on_hemisphere(rng, &rec.normal);

                ScatterRecord {
                    attenuation: *albedo,
                    scattered: Ray {
                        pos: rec.p,
                        dir: scatter_direction,
                    },
                }
            }
        }
    }
}
