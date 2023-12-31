use std::simd::{LaneCount, SupportedLaneCount, Mask, Simd, SimdElement};

use crate::geometry::{Vec3, Point3, PackedVec3, PackedPoint3};

#[derive(Debug)]
#[derive(Clone, Copy)]
pub struct Ray {
    origin: Point3,
    direction: Vec3
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3) -> Ray{
        Ray {
            origin: origin,
            direction: direction,
        }
    }
    
    pub fn origin(&self) -> Point3 {
        self.origin
    }

    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    pub fn at(&self, t: f64) -> Point3 {
        self.origin + self.direction * t
    }

}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct PackedRays<const N: usize> 
where
    LaneCount<N>: SupportedLaneCount,
{
    origins: PackedPoint3<N>,
    directions: PackedVec3<N>,
    enabled: Mask<<f64 as SimdElement>::Mask, N>
}

impl <const N: usize> PackedRays<N> 
where LaneCount<N>: SupportedLaneCount
{
    #[inline]
    pub fn new(origins: PackedPoint3<N>, directions: PackedVec3<N>) -> PackedRays<N> {
        PackedRays {
            origins,
            directions,
            enabled: Mask::splat(true)
        }
    }

    #[inline]
    pub fn new_with_enable(origins: PackedPoint3<N>, directions: PackedVec3<N>, enabled: Mask<<f64 as SimdElement>::Mask, N>) -> PackedRays<N> {
        PackedRays { origins, directions, enabled }
    }

    #[inline]
    pub fn origins(&self) -> PackedPoint3<N> {
        self.origins
    }

    #[inline]
    pub fn directions(&self) -> PackedPoint3<N> {
        self.directions
    }

    #[inline]
    pub fn enabled(&self) -> Mask<<f64 as SimdElement>::Mask, N> {
        self.enabled
    }

    #[inline]
    pub fn is_enabled(&self, index: usize) -> bool {
        self.enabled.test(index)
    }

    #[inline]
    pub fn count() -> usize {
        N
    }

    #[inline]
    pub fn at(&self, index: usize) -> Option<Ray> {
        if self.enabled.test(index) {
            Some(Ray::new(self.origins.at(index), self.directions.at(index)))
        } else {
            None
        }
    }

    #[inline]
    pub fn at_including_disabled(&self, index: usize) -> Ray {
        Ray::new(self.origins.at(index), self.directions.at(index))
    }

    #[inline]
    pub fn at_t(&self, t: Simd<f64, N>) -> PackedPoint3<N> {
        self.origins + self.directions * t
    }

    #[inline]
    pub fn update(&mut self, index: usize, value: Ray) {
        self.origins.update(index, value.origin());
        self.directions.update(index, value.direction());
        self.enabled.set(index, true);
    }

    #[inline]
    pub fn update_with_enable(&mut self, index: usize, value: Ray, enable: bool) {
        self.origins.update(index, value.origin());
        self.directions.update(index, value.direction());
        self.enabled.set(index, enable);
    }

    #[inline]
    pub fn any_enabled(&self) -> bool {
        self.enabled().any()
    }

    #[inline]
    pub fn enable(&mut self, index: usize) {
        self.enabled.set(index, true)
    }

    #[inline]
    pub fn disable(&mut self, index: usize) {
        self.enabled.set(index, false);
    }
}

impl <const N: usize> FromIterator<Ray> for PackedRays<N> 
where LaneCount<N>: SupportedLaneCount
{
    fn from_iter<T: IntoIterator<Item = Ray>>(iter: T) -> Self {
        let mut packed_rays = PackedRays {
            directions: PackedVec3::default(),
            origins: PackedPoint3::default(),
            enabled: Mask::splat(false)
        };

        for (index, value) in iter.into_iter().enumerate() {
            assert!(index < N, "too may elements given in iterator!");
            packed_rays.update(index, value)
        }

        packed_rays
    }
}