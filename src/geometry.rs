use rand::prelude::*;

#[derive(Debug)]
#[derive(Clone, Copy)]
pub struct Vec3 {
    x: f64,
    y: f64,
    z: f64
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3{
        Vec3 { x: x, y: y, z: z}
    }

    pub fn zero() -> Vec3 {
        Vec3 {x: 0.0, y: 0.0, z: 0.0}
    }

    #[inline]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> f64 {
        self.y
    }

    #[inline]
    pub fn z(&self) -> f64 {
        self.z
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x, 
            y: self.y + rhs.y, 
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x, 
            y: self.y - rhs.y, 
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Mul<f64> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f64) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl std::ops::Mul<Vec3> for f64{
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        rhs * self
    }
}

impl std::ops::Div<f64> for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs
        }
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

impl Vec3 {
    #[inline]
    pub fn length_squared(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
    }

    #[inline]
    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    #[inline]
    pub fn abs(&self) -> f64 {
        self.length()
    }

    #[inline]
    pub fn unit(self) -> Self {
        self / self.length()
    }

    #[inline]
    pub fn dot(&self, rhs: &Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    #[inline]
    pub fn cross(&self, rhs: &Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x
        }
    }

    #[inline]
    pub fn near_zero(&self) -> bool {
        let epsilon = 1E-8;
        self.x.abs() < epsilon && self.y.abs() < epsilon && self.z.abs() < epsilon
    }

    #[inline]
    pub fn random_unit_vector() -> Vec3 {
        let normal = rand_distr::Normal::new(0.0, 1.0).unwrap();
        let vector = Vec3 {
            x: normal.sample(&mut thread_rng()),
            y: normal.sample(&mut thread_rng()),
            z: normal.sample(&mut thread_rng()),
        };

        if vector.near_zero() {
            Vec3{x: 0.0, y: 0.0, z: 1.0}
        } else {
            vector.unit()
        }
    }

    #[inline]
    pub fn random_on_unit_hemisphere(normal: &Vec3) -> Vec3{
        let random_unit_vector = Self::random_unit_vector();
        if random_unit_vector.dot(normal) > 0.0 {
            random_unit_vector
        } else {
            -random_unit_vector
        }
    }

    #[inline]
    pub fn reflect(&self, normal: &Vec3) -> Vec3 {
        (*self) - 2.0 * self.dot(normal) * (*normal)
    }

    #[inline]
    pub fn refract(self, normal: &Vec3, refraction_ratio: f64) -> Vec3{
        let cos_theta = (-self).dot(normal).min(1.0);
        let r_out_perpendicular = refraction_ratio * (self + cos_theta * (*normal));
        let r_out_parallel = -((1.0 - r_out_perpendicular.length_squared()).abs().sqrt()) * (*normal);
        r_out_perpendicular + r_out_parallel
    }

    pub fn from_toml(table: &toml::Table) -> Vec3 {
        let x = table["x"].as_float().unwrap();
        let y = table["y"].as_float().unwrap();
        let z = table["z"].as_float().unwrap();

        Vec3::new(x, y, z)
    }

}

pub type Point3 = Vec3;