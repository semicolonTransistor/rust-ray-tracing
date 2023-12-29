use rand::{thread_rng, Rng};

use crate::geometry::Vec3;
use crate::color::Color;
use crate::ray::Ray;
use crate::objects::{HitRecord, HitResult};
use crate::toml_utils::to_float;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

pub fn get_materials(table: &toml::Table) -> HashMap<String, Arc<dyn Material>>{
    table.iter().map(|(key, value)| {
        (
            key.clone(),
            load_material_from_toml(value.as_table().unwrap())
        )
    }).collect()
}

pub fn load_material_from_toml(table: &toml::Table) -> Arc<dyn Material> {
    let material_type = table["type"].as_str().unwrap().to_ascii_lowercase();
    
    if material_type == "lambertian" {
        Lambertian::from_table(table)
    } else if material_type == "metal" {
        Metal::from_table(table)
    } else if material_type == "dielectric" {
        Dielectric::from_table(table)
    } else {
        panic!("Unknown material type {}!", material_type)
    }
}

pub trait Material : Debug + Sync + Send {
    fn get_hit_result(&self, ray: &Ray, hit_record: &HitRecord) -> HitResult;

    fn from_table(table: &toml::Table) -> Arc<dyn Material> where Self: Sized;
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

    fn from_table(table: &toml::Table) -> Arc<dyn Material> where Self: Sized {
        let albedo = Color::from_toml(&table["albedo"]).unwrap();
        Arc::new(Lambertian::new(albedo))
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

    fn from_table(table: &toml::Table) -> Arc<dyn Material> where Self: Sized {
        println!("=====================");
        println!("{}", table);
        let albedo = Color::from_toml(&table["albedo"]).unwrap();
        let fuzzy_factor = to_float(&table["fuzzy_factor"]).unwrap();

        Arc::new(Metal::new(albedo, fuzzy_factor))
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

    fn from_table(table: &toml::Table) -> Arc<dyn Material> where Self: Sized {
        let index_of_refraction = to_float(&table["index_of_refraction"]).unwrap();
        let hollow = table["hollow"].as_bool().unwrap();

        Arc::new(Dielectric::new(index_of_refraction, hollow))
    }
}
    