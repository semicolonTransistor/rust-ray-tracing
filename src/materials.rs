use rand::{thread_rng, Rng};

use crate::geometry::Vec3;
use crate::color::Color;
use crate::ray_tracing::{Ray, HitRecord, HitResult};
use std::fmt::Debug;

pub trait Material : Debug + Sync + Send {
    fn get_hit_result(&self, ray: &Ray, hit_record: &HitRecord) -> HitResult;
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Lambertian {
    albedo: Color
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn get_hit_result(&self, _ray: &Ray, hit_record: &HitRecord) -> HitResult {
        let mut scatter_direction = Vec3::random_unit_vector() + hit_record.normal();

        if scatter_direction.near_zero() {
            scatter_direction = hit_record.normal();
        }

        HitResult::new_scattered(self.albedo, Ray::new(hit_record.location(), scatter_direction))
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Metal {
    albedo: Color,
    fuzzy_factor: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzzy_factor: f64) -> Self {
        Self { 
            albedo, 
            fuzzy_factor: if fuzzy_factor < 1.0 {
                fuzzy_factor
            } else {
                1.0
            }
        }
    }
}

impl Material for Metal {
    fn get_hit_result(&self, ray: &Ray, hit_record: &HitRecord) -> HitResult {
        let reflected = ray.direction().reflect(&hit_record.normal()) + self.fuzzy_factor * Vec3::random_unit_vector();
        let scattered = Ray::new(hit_record.location(), reflected);

        HitResult::new_scattered(self.albedo, scattered)
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Dielectric {
    index_of_refraction: f64,
    hollow: bool,
}

impl Dielectric {
    pub fn new(index_of_refraction: f64, hollow: bool) -> Dielectric {
        Dielectric { index_of_refraction, hollow}
    }

    fn reflectance(cosine: f64, index_of_refraction: f64) -> f64 {
        let r0 = ((1.0 - index_of_refraction) / (1.0 + index_of_refraction)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn get_hit_result(&self, ray: &Ray, hit_record: &HitRecord) -> HitResult {
        let refraction_ratio = if hit_record.front_face() {1.0 / self.index_of_refraction} else {self.index_of_refraction};

        let normal = if self.hollow {-hit_record.normal()} else {hit_record.normal()};
        let cos_theta = (-ray.direction()).dot(&normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let refracted_direction = if cannot_refract || Dielectric::reflectance(cos_theta, refraction_ratio) > thread_rng().gen_range(0.0..1.0){
            ray.direction().reflect(&normal)
        } else {
            ray.direction().refract(&normal, refraction_ratio)
        };
        
        HitResult::new_scattered(
            Color::white(), 
            Ray::new(hit_record.location(), refracted_direction),
        )
    }
}
    