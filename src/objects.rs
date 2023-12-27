use crate::geometry::{Vec3, Point3};
use crate::materials::Material;
use crate::ray_tracing::Ray;
use crate::color::Color;

use std::collections::HashMap;
use std::sync::Arc;
use std::fmt::Debug;

#[derive(Debug)]
#[derive(Clone)]
pub enum Object {
    Sphere(Sphere),
}

impl Object {
    pub fn hit(&self, ray: &Ray, t_range: &std::ops::Range<f64>) -> Option<HitRecord> {
        match self {
            Object::Sphere(s) => s.hit(ray, t_range),
        }
    }

    pub fn hit_rays<'a>(&'a self, rays: &[Ray], t_range: &std::ops::Range<f64>, ray_enable_flags: &mut [bool], hit_records: &mut[Option<HitRecord<'a>>]) {
        match self {
            Object::Sphere(s) => s.hit_rays(rays, t_range, ray_enable_flags, hit_records),
        }
    }
}

pub fn get_object_list(toml_object_list: &toml::value::Array, material_table: &HashMap<String, Arc<dyn Material>>) -> Vec<Object> {
    toml_object_list.iter().map(|value| {
        load_object_from_toml(value.as_table().unwrap(), material_table)
    }).collect()
}

pub fn load_object_from_toml(table: &toml::Table, material_table: &HashMap<String, Arc<dyn Material>>) -> Object {
    let object_type = table["type"].as_str().unwrap().to_ascii_lowercase();
    
    if object_type == "sphere" {
        Object::Sphere(Sphere::from_table(table, material_table))
    } else {
        panic!("Unknown object type {}", object_type)
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct HitRecord<'a> {
    location: Point3,
    normal: Vec3,
    t: f64,
    front_face: bool,
    material: &'a Arc<dyn Material>,
}

impl HitRecord<'_> {
    pub fn new <'a> (ray: &Ray, location: Point3, outward_normal: Vec3, t: f64, material: &'a Arc<dyn Material>) -> HitRecord<'a> {
        debug_assert!((outward_normal.length() - 1.0).abs() < 1E-9, "expecting 1.0, got {}", outward_normal.length());
        
        
        let (front_face, normal) = if ray.direction().dot(&outward_normal) < 0.0 {
            (true, outward_normal)
        } else {
            (false, -outward_normal)
        };
        

        HitRecord {
            location: location,
            normal: normal,
            t: t,
            front_face: front_face,
            material: material
        }
    }

    pub fn location(&self) -> Point3 {
        self.location
    }

    pub fn normal(&self) -> Vec3 {
        self.normal
    }

    pub fn t(&self) -> f64 {
        self.t
    }

    pub fn front_face(&self) -> bool {
        self.front_face
    }

    pub fn hit_result(&self, ray: &Ray) -> HitResult {
        self.material.get_hit_result(ray, self)
    }
}

pub trait Hittable : Debug + Sync + Send {
    fn hit(&self, ray: &Ray, t_range: &std::ops::Range<f64>) -> Option<HitRecord>;

    fn hit_rays<'a>(&'a self, rays: &[Ray], t_range: &std::ops::Range<f64>, ray_enable_flags: &mut [bool], hit_records: &mut[Option<HitRecord<'a>>]);

    fn from_table(table: &toml::Table, material_table: &HashMap<String, Arc<dyn Material>>) -> Self where Self: Sized;
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    material: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, material: &Arc<dyn Material>) -> Sphere{
        Sphere { center: center, radius: radius, material: material.clone()}
    }

    fn find_closest_root(&self, ray: &Ray, t_range: &std::ops::Range<f64>) -> Option<f64> {
        let center_offset = ray.origin() - self.center;

        let a = ray.direction().length_squared();
        let half_b = center_offset.dot(&ray.direction());
        let c = center_offset.length_squared() - self.radius.powi(2);
        let discriminant = half_b.powi(2) - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrt_discriminant = discriminant.sqrt();
        let mut root = (-half_b - sqrt_discriminant) / a;
        if !t_range.contains(&root) {
            root = (-half_b + sqrt_discriminant) / a;
            if !t_range.contains(&root) {
                return None;
            }
        }

        Some(root)
    }
    
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_range: &std::ops::Range<f64>) -> Option<HitRecord> {
        match self.find_closest_root(ray, t_range) {
            Some(root) => {
                let location = ray.at(root);
                Some(HitRecord::new(
                    ray,
                    location,
                    (location - self.center) / self.radius,
                    root,
                    &self.material,
                ))
            },
            None => None
        }

        

    }

    fn hit_rays<'a>(&'a self, rays: &[Ray], t_range: &std::ops::Range<f64>, ray_enable_flags: &mut [bool], hit_records: &mut[Option<HitRecord<'a>>]) {
        for i in 0..rays.len() {
            if ray_enable_flags[i] {
                match self.find_closest_root(&rays[i], t_range) {
                    Some(root) => {
                        if hit_records[i].is_none() || hit_records[i].as_ref().unwrap().t() > root {
                            let location = rays[i].at(root);
                            hit_records[i] = Some(HitRecord::new(
                                &rays[i],
                                location,
                                (location - self.center) / self.radius,
                                root,
                                &self.material,
                            ))
                        }
                    },
                    None => (),
                }
            }
        }
    }

    fn from_table(table: &toml::Table, material_table: &HashMap<String, Arc<dyn Material>>) -> Self where Self: Sized {
        let center = Point3::from_toml(table["center"].as_table().unwrap());
        let radius = table["radius"].as_float().unwrap();
        let material_name = table["material"].as_str().unwrap();
        let material = material_table.get(material_name).unwrap();

        Sphere::new(center, radius, material)
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct HitResult {
    attenuation: Color,
    scattered_ray: Option<Ray>
}

impl HitResult {
    pub fn new_absorbed(attenuation: Color) -> HitResult {
        HitResult { attenuation: attenuation, scattered_ray: None }
    }

    pub fn new_scattered(attenuation: Color, scattered_ray: Ray) -> HitResult {
        HitResult { attenuation: attenuation, scattered_ray: Some(scattered_ray) }
    }

    pub fn attenuation(&self) -> Color {
        self.attenuation
    }

    pub fn scattered_ray(&self) -> Option<&Ray> {
        self.scattered_ray.as_ref()
    }
}