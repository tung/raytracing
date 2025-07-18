use crate::random::*;

#[derive(Copy, Clone, Debug)]
pub struct Vec3([f64; 3]);

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self([x, y, z])
    }

    pub fn x(&self) -> f64 {
        self.0[0]
    }

    pub fn y(&self) -> f64 {
        self.0[1]
    }

    pub fn z(&self) -> f64 {
        self.0[2]
    }

    pub fn dot(self, other: Self) -> f64 {
        self.0[0] * other.0[0] + self.0[1] * other.0[1] + self.0[2] * other.0[2]
    }

    pub fn cross(self, other: Self) -> Self {
        Self([
            self.0[1] * other.0[2] - self.0[2] * other.0[1],
            self.0[2] * other.0[0] - self.0[0] * other.0[2],
            self.0[0] * other.0[1] - self.0[1] * other.0[0],
        ])
    }

    pub fn length_squared(&self) -> f64 {
        self.0[0] * self.0[0] + self.0[1] * self.0[1] + self.0[2] * self.0[2]
    }

    pub fn near_zero(&self) -> bool {
        // Returns true if the vector is close to zero in all directions.
        let s = 1.0e-8;
        self.0[0].abs() < s && self.0[1].abs() < s && self.0[2].abs() < s
    }

    pub fn random(rng: &mut Rng) -> Self {
        Self([rng.random_f64(), rng.random_f64(), rng.random_f64()])
    }

    pub fn random_range(rng: &mut Rng, min: f64, max: f64) -> Self {
        Self([
            rng.random_f64_range(min, max),
            rng.random_f64_range(min, max),
            rng.random_f64_range(min, max),
        ])
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn unit(&self) -> Self {
        *self / self.length()
    }

    pub fn random_in_unit_disk(rng: &mut Rng) -> Self {
        loop {
            let p = Self([
                rng.random_f64_range(-1.0, 1.0),
                rng.random_f64_range(-1.0, 1.0),
                0.0,
            ]);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn random_unit_vector(rng: &mut Rng) -> Self {
        loop {
            let p = Self::random_range(rng, -1.0, 1.0);
            let lensq = p.length_squared();
            if lensq > 1.0e-160 {
                return p / lensq.sqrt();
            }
        }
    }

    pub fn reflect(self, n: Self) -> Self {
        self - 2.0 * self.dot(n) * n
    }

    pub fn refract(self, n: Self, etai_over_etat: f64) -> Self {
        let cos_theta = f64::min((-self).dot(n), 1.0);
        let r_out_perp = etai_over_etat * (self + cos_theta * n);
        let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
        r_out_perp + r_out_parallel
    }
}

impl std::ops::Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
        ])
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3([
            self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
        ])
    }
}

impl std::ops::Mul for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3([
            self.0[0] * rhs.0[0],
            self.0[1] * rhs.0[1],
            self.0[2] * rhs.0[2],
        ])
    }
}

impl std::ops::Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3([self * rhs.0[0], self * rhs.0[1], self * rhs.0[2]])
    }
}

impl std::ops::Div for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: Vec3) -> Vec3 {
        Vec3([
            self.0[0] / rhs.0[0],
            self.0[1] / rhs.0[1],
            self.0[2] / rhs.0[2],
        ])
    }
}

impl std::ops::Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f64) -> Vec3 {
        Vec3([self.0[0] / rhs, self.0[1] / rhs, self.0[2] / rhs])
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3([-self.0[0], -self.0[1], -self.0[2]])
    }
}

impl std::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        self.0[0] += rhs.0[0];
        self.0[1] += rhs.0[1];
        self.0[2] += rhs.0[2];
    }
}
