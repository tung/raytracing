// splitmix64 adapted from https://prng.di.unimi.it/splitmix64.c
fn splitmix64_next(x: &mut u64) -> u64 {
    *x = x.overflowing_add(0x9e3779b97f4a7c15).0;
    let z = *x;
    let z = (z ^ (z >> 30)).overflowing_mul(0xbf58476d1ce4e5b9).0;
    let z = (z ^ (z >> 27)).overflowing_mul(0x94d049bb133111eb).0;
    z ^ (z >> 31)
}

pub struct Rng {
    state: [u64; 4],
}

impl Rng {
    pub fn new(mut seed: u64) -> Self {
        Self {
            state: [
                splitmix64_next(&mut seed),
                splitmix64_next(&mut seed),
                splitmix64_next(&mut seed),
                splitmix64_next(&mut seed),
            ],
        }
    }

    // xoshiro256+ adapted from https://prng.di.unimi.it/xoshiro256plus.c
    fn xoshiro256p_next(&mut self) -> u64 {
        let result = self.state[0].overflowing_add(self.state[3]).0;

        let t = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;

        self.state[3] = self.state[3].rotate_left(45);

        result
    }

    // Daniel Lemire's algorithm to get a random number from zero to s.
    pub fn random_u64(&mut self, s: u64) -> u64 {
        let mut x = self.xoshiro256p_next();
        let mut m = x as u128 * s as u128;
        let mut l = (m & 0xffff_ffff_ffff_ffff) as u64;

        if l < s {
            let t = s.wrapping_neg() % s;

            while l < t {
                x = self.xoshiro256p_next();
                m = x as u128 * s as u128;
                l = (m & 0xffff_ffff_ffff_ffff) as u64;
            }
        }

        (m >> 64) as u64
    }

    pub fn random_f64(&mut self) -> f64 {
        let limit = (1 << 53) - 1;
        self.random_u64(limit - 1) as f64 / limit as f64
    }

    pub fn random_f64_range(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.random_f64()
    }
}
