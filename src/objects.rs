use array_macro::array;

use crate::geometry::{Vec3, Point3};
use crate::materials::Material;
use crate::ray_tracing::{Ray, PackedRays};
use crate::color::Color;
use crate::toml_utils::to_float;
use crate::geometry::{PackedVec3, PackedPoint3};
use crate::packed::{PackedScalerPartialOrd, PackedPartialOrd, self, PackedF64, PackedBool, Packed};

use std::collections::HashMap;
use std::mem::Discriminant;
use std::panic::Location;
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

    pub fn hit_packed<'a, const N: usize>(&'a self, rays: &PackedRays<N>, t_range: &std::ops::Range<f64>, hit_records: &mut PackedHitRecords<'a, N>) {
        match self {
            Object::Sphere(s) => s.hit_packed(rays, t_range, hit_records),
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


#[derive(Debug)]
#[derive(Clone)]
pub struct PackedHitRecords<'a, const N: usize> {
    locations: PackedPoint3<N>,
    normals: PackedVec3<N>,
    t: PackedF64<N>,
    front_face: PackedBool<N>,
    hits: PackedBool<N>,
    materials: [Option<&'a Arc<dyn Material>>; N]
}

impl <const N: usize> Default for PackedHitRecords<'_, N> {
    fn default() -> Self {
        PackedHitRecords {
            locations: PackedPoint3::default(),
            normals: PackedVec3::default(),
            t: PackedF64::broadcast_scaler(f64::INFINITY),
            front_face: PackedBool::default(),
            hits: PackedBool::default(),
            materials: array![None; N]
        }
    }
}

impl <'a, const N: usize> PackedHitRecords<'a, N> {
    pub fn update(&mut self, rays: &PackedRays<N>, locations: &PackedPoint3<N>, outward_normals: &PackedVec3<N>, t: &PackedF64<N>, valid_mask: &PackedBool<N>, material: &'a Arc<dyn Material>) {
        let mut normals = *outward_normals;

        let front_face = PackedScalerPartialOrd::lt(&rays.directions().dot(&outward_normals), &0.0);

        normals.assign_masked(&-*outward_normals, !front_face);

        let update_mask = *valid_mask & PackedPartialOrd::le(t, &self.t);

        self.locations.assign_masked(&locations, update_mask);
        self.normals.assign_masked(&normals, update_mask);
        self.t.assign_masked(*t, update_mask);
        self.front_face.assign_masked(front_face, update_mask);
        self.hits = self.hits | update_mask;
        
        for i in 0..N {
            if update_mask[i] {
                self.materials[i] = Some(material);
            }
        }

    }

    pub fn at(&self, index: usize) -> Option<HitRecord<'a>> {
        if self.hits[index] {
            Some(HitRecord {
                location: self.locations.at(index),
                normal: self.normals.at(index),
                t: self.t[index],
                front_face: self.front_face[index],
                material: self.materials[index].unwrap(),
            })
        } else {
            None
        }
    }

    pub fn locations(&self) -> PackedPoint3<N> {
        self.locations
    }

    pub fn normals(&self) -> PackedVec3<N> {
        self.normals
    }

    pub fn t(&self) -> PackedF64<N> {
        self.t
    }

    pub fn hits(&self) -> PackedBool<N> {
        self.hits
    }

    pub fn front_face(&self) -> PackedBool<N> {
        self.front_face
    }

    pub fn material(&self) -> &[Option<&'a Arc<dyn Material>>; N] {
        &self.materials
    }
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
    
    pub fn hit(&self, ray: &Ray, t_range: &std::ops::Range<f64>) -> Option<HitRecord> {
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


        let location = ray.at(root);

        Some(HitRecord::new(
            ray,
            location,
            (location - self.center) / self.radius,
            root,
            &self.material,
        ))

    }

    pub fn hit_packed<'a, const N: usize>(&'a self, rays: &PackedRays<N>, t_range: &std::ops::Range<f64>, hit_records: &mut PackedHitRecords<'a, N>) {
        let center_offset = rays.origins() - self.center;
        let a = rays.directions().length_squared();
        let half_b = center_offset.dot(&rays.directions());
        let c = center_offset.length_squared() - self.radius.powi(2);
        let discriminant = half_b.powi(2) - a * c;

        let discriminant_positive = PackedScalerPartialOrd::ge(&discriminant, &0.0);
        let sqrt_discriminant = discriminant.sqrt();
        let mut root = (-half_b - sqrt_discriminant) / a;
        let root_out_of_range = (!root.inside(t_range)) & discriminant_positive;
        root.assign_masked((-half_b + sqrt_discriminant) / a, root_out_of_range);
        let root_valid = root.inside(t_range) & discriminant_positive;
        let valid = root_valid & rays.enabled();

        let locations = rays.at_t(root);
        let normal = (locations - self.center) / self.radius;

        // let normal_not_unit = PackedScalerPartialOrd::gt(&(normal.length() - 1.0).abs(), &0.01) & valid;
        // if normal_not_unit.any() {
        //     println!("Normal not Unit: {:?}", normal_not_unit);
        //     println!("Valid: {:?}", valid);
        //     println!("Discriminate_positive: {:?}", discriminant_positive);
        //     println!("Sphere: {:?}", self);
        //     // println!("Rays: {:?}", rays);
        //     println!("Roots: {:?}", root);
        //     println!("Normals: {:?}", normal);
        //     println!("Normal Lengths: {:?}", normal.length());
        //     panic!()
        // }
        
        hit_records.update(
            rays, 
            &locations, 
            &normal,
            &root, 
            &root_valid, 
            &self.material
        )
    }

    pub fn from_table(table: &toml::Table, material_table: &HashMap<String, Arc<dyn Material>>) -> Self where Self: Sized {
        let center = Point3::from_toml(&table["center"]).unwrap();
        let radius = to_float(&table["radius"]).unwrap();
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