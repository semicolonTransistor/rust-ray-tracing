use rand::prelude::*;
use crate::toml_utils::to_float;
use array_macro::array;
use std::{simd::{LaneCount, SupportedLaneCount, StdFloat, Simd, Mask, SimdElement, MaskElement}, ops::Mul};
use crate::simd_util::masked_assign;

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

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn z(&self) -> f64 {
        self.z
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;
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
    pub fn length_squared(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn abs(&self) -> f64 {
        self.length()
    }

    pub fn unit(self) -> Self {
        self / self.length()
    }

    pub fn dot(&self, rhs: &Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x
        }
    }

    pub fn near_zero(&self) -> bool {
        let epsilon = 1E-8;
        self.x.abs() < epsilon && self.y.abs() < epsilon && self.z.abs() < epsilon
    }

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

    pub fn random_in_unit_disk() -> Vec3 {
        loop {
            let x: f64 = thread_rng().gen_range(-1.0..=1.0);
            let y: f64 = thread_rng().gen_range(-1.0..=1.0);

            if x.powi(2) + y.powi(2) <= 1.0 {
                return Vec3 {
                    x,
                    y,
                    z: 0.0,
                };
            }
        }
        
    }

    pub fn random_on_unit_hemisphere(normal: &Vec3) -> Vec3{
        let random_unit_vector = Self::random_unit_vector();
        if random_unit_vector.dot(normal) > 0.0 {
            random_unit_vector
        } else {
            -random_unit_vector
        }
    }

    pub fn reflect(&self, normal: &Vec3) -> Vec3 {
        (*self) - 2.0 * self.dot(normal) * (*normal)
    }

    pub fn refract(self, normal: &Vec3, refraction_ratio: f64) -> Vec3{
        let cos_theta = (-self).dot(normal).min(1.0);
        let r_out_perpendicular = refraction_ratio * (self + cos_theta * (*normal));
        let r_out_parallel = -((1.0 - r_out_perpendicular.length_squared()).abs().sqrt()) * (*normal);
        r_out_perpendicular + r_out_parallel
    }

    pub fn from_toml(value: &toml::Value) -> Option<Vec3> {
        match value.as_table() {
            Some(table) => {
                let x = to_float(&table["x"]).unwrap();
                let y = to_float(&table["y"]).unwrap();
                let z = to_float(&table["z"]).unwrap();
                Some(Vec3::new(x, y, z))
            },
            None => match value.as_array() {
                Some(array) => {
                    assert!(array.len() >= 3);
                    let x = to_float(&array[0]).unwrap();
                    let y = to_float(&array[1]).unwrap();
                    let z = to_float(&array[2]).unwrap();

                    Some(Vec3::new(x, y, z))
                },
                None => None,
            },
        }
        
    }

}

pub type Point3 = Vec3;

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(Default)]
pub struct PackedVec3<const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    x: Simd<f64, N>,
    y: Simd<f64, N>,
    z: Simd<f64, N>
}

pub type PackedPoint3<const N:usize> = PackedVec3<N>;

impl <M, const N: usize> PackedVec3<N> 
where
    f64: SimdElement<Mask = M>,
    M: MaskElement,
    LaneCount<N>: SupportedLaneCount,
{
    #[inline]
    pub fn zeros() -> PackedVec3<N> {
        PackedVec3 {
            x: Simd::splat(0.0),
            y: Simd::splat(0.0),
            z: Simd::splat(0.0),
        }
    }

    #[inline]
    pub fn from_simd(x: Simd<f64, N>, y: Simd<f64, N>, z: Simd<f64, N>) -> PackedVec3<N>{
        PackedVec3 {
            x,
            y,
            z
        }
    }

    #[inline]
    pub fn from_vec3s(vec3s: &[Vec3]) -> PackedVec3<N> {
        assert!(vec3s.len() == N);

        PackedVec3 {
            x: Simd::from_array(array![i => vec3s[i].x(); N]),
            y: Simd::from_array(array![i => vec3s[i].y(); N]),
            z: Simd::from_array(array![i => vec3s[i].z(); N]),
        }
    }

    #[inline]
    pub fn from_broadcast_vec3(vec3: &Vec3) -> PackedVec3<N> {
        Self::splat(vec3)
    }

    #[inline]
    pub fn splat(vec3: &Vec3) -> PackedVec3<N> {
        PackedVec3 {
            x: Simd::splat(vec3.x()),
            y: Simd::splat(vec3.y()),
            z: Simd::splat(vec3.z()),
        }
    }

    #[inline]
    pub fn assign_masked(&mut self, values: &PackedVec3<N>, mask: Mask<M, N>) {
        masked_assign(&mut self.x, values.x, mask);
        masked_assign(&mut self.y, values.y, mask);
        masked_assign(&mut self.z, values.z, mask);
    }
}

impl <const N: usize> std::ops::Add for PackedVec3<N> 
where LaneCount<N>: SupportedLaneCount
{
    type Output = PackedVec3<N>;
    
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        PackedVec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}


impl <const N: usize> std::ops::Sub for PackedVec3<N> 
where LaneCount<N>: SupportedLaneCount
{
    type Output = PackedVec3<N>;
    
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        PackedVec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl <const N: usize> std::ops::Sub<Vec3> for PackedVec3<N> 
where LaneCount<N>: SupportedLaneCount
{
    type Output = PackedVec3<N>;
    
    #[inline]
    fn sub(self, rhs: Vec3) -> Self::Output {
        self - Self::splat(&rhs)
    }
}

impl <const N: usize> std::ops::Sub<PackedVec3<N>> for Vec3 
where LaneCount<N>: SupportedLaneCount
{
    type Output = PackedVec3<N>;
    
    #[inline]
    fn sub(self, rhs: PackedVec3<N>) -> Self::Output {
        PackedVec3::splat(&self) - rhs
    }
}

impl <const N: usize> std::ops::Neg for PackedVec3<N> 
where LaneCount<N>: SupportedLaneCount
{
    type Output = PackedVec3<N>;

    #[inline]
    fn neg(self) -> Self::Output {
        PackedVec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl <const N: usize> std::ops::Mul<f64> for PackedVec3<N>
where LaneCount<N>: SupportedLaneCount
{
    type Output = PackedVec3<N>;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        let rhs_as_simd = Simd::splat(rhs);
        PackedVec3 {
            x: self.x * rhs_as_simd,
            y: self.y * rhs_as_simd,
            z: self.z * rhs_as_simd,
        }
    }
}

impl <const N: usize> std::ops::Mul<Simd<f64, N>> for PackedVec3<N> 
where LaneCount<N>: SupportedLaneCount
{
    type Output = PackedVec3<N>;

    #[inline]
    fn mul(self, rhs: Simd<f64, N>) -> Self::Output {
        PackedVec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl <const N: usize> std::ops::Mul<PackedVec3<N>> for f64 
where LaneCount<N>: SupportedLaneCount
{
    type Output = PackedVec3<N>;

    #[inline]
    fn mul(self, rhs: PackedVec3<N>) -> Self::Output {
        rhs * self
    }
}

impl <const N: usize> std::ops::Div<Simd<f64, N>> for PackedVec3<N> 
where LaneCount<N>: SupportedLaneCount
{
    type Output = PackedVec3<N>;

    #[inline]
    fn div(self, rhs: Simd<f64, N>) -> Self::Output {
        PackedVec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl <const N: usize> std::ops::Div<f64> for PackedVec3<N> 
where LaneCount<N>: SupportedLaneCount
{
    type Output = PackedVec3<N>;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        let rhs_as_simd = Simd::splat(rhs);
        PackedVec3 {
            x: self.x / rhs_as_simd,
            y: self.y / rhs_as_simd,
            z: self.z / rhs_as_simd,
        }
    }
}

impl <const N: usize> PackedVec3<N> 
where LaneCount<N>: SupportedLaneCount
{
    
    #[inline]
    pub fn length_squared(&self) -> Simd<f64, N> {
        self.z.mul_add(self.z , self.y.mul_add(self.y, self.x * self.x))
    }

    #[inline]
    pub fn length(&self) -> Simd<f64, N> {
        self.length_squared().sqrt()
    }

    #[inline]
    pub fn unit_vector(&self) -> PackedVec3<N> {
        *self / self.length()
    }

    #[inline]
    pub fn count() -> usize {
        N
    }

    #[inline]
    pub fn at(&self, index: usize) -> Vec3 {
        Vec3 { x: self.x[index], y: self.y[index], z: self.z[index] }
    }

    #[inline]
    pub fn update(&mut self, index: usize, value: Vec3) {
        self.x[index] = value.x();
        self.y[index] = value.y();
        self.z[index] = value.z();
    }

    #[inline]
    pub fn dot(&self, rhs: &Self) -> Simd<f64, N> {
        self.z.mul_add(rhs.z , self.y.mul_add(rhs.y, self.x * rhs.x))
    }

    #[inline]
    pub fn x(&self) -> Simd<f64, N> {
        self.x
    }

    #[inline]
    pub fn y(&self) -> Simd<f64, N> {
        self.y
    }

    #[inline]
    pub fn z(&self) -> Simd<f64, N> {
        self.z
    }

}
