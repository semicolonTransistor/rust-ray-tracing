use crate::geometry::{Vec3, Point3};
use crate::materials::Material;
use crate::ray::{Ray, PackedRays};
use crate::color::Color;
use crate::toml_utils::to_float;
use crate::geometry::{PackedVec3, PackedPoint3};
use crate::simd_util::{masked_assign, simd_inside, PackedOptionalReference, negate_simd_float, masked_select};

use std::collections::HashMap;
use std::simd::cmp::SimdPartialOrd;
use std::simd::{LaneCount, SupportedLaneCount, Simd, Mask, SimdElement, StdFloat};
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

    // #[inline(never)]
    pub fn hit_packed<'a, const N: usize>(&'a self, rays: &PackedRays<N>, t_range: &std::ops::Range<f64>, hit_records: &mut PackedHitRecords<'a, N>) 
    where LaneCount<N>: SupportedLaneCount
    {
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
pub struct PackedHitRecords<'a, const N: usize> 
where LaneCount<N>: SupportedLaneCount
{
    locations: PackedPoint3<N>,
    normals: PackedVec3<N>,
    t: Simd<f64, N>,
    front_face: Mask<<f64 as SimdElement>::Mask, N>,
    hits:  Mask<<f64 as SimdElement>::Mask, N>,
    materials: PackedOptionalReference<'a, Arc<dyn Material>, N>
    // materials: [Option<&'a Arc<dyn Material>>; N]
}

impl <const N: usize> Default for PackedHitRecords<'_, N> 
where LaneCount<N>: SupportedLaneCount
{
    fn default() -> Self {
        PackedHitRecords {
            locations: PackedPoint3::default(),
            normals: PackedVec3::default(),
            t: Simd::splat(f64::INFINITY),
            front_face: Mask::splat(false),
            hits: Mask::splat(false),
            materials: PackedOptionalReference::nones(),
        }
    }
}

impl <'a, const N: usize> PackedHitRecords<'a, N> 
where 
    LaneCount<N>: SupportedLaneCount
{
    pub fn update(&mut self, rays: &PackedRays<N>, outward_normals: &PackedVec3<N>, t: &Simd<f64, N>, valid_mask: &Mask<<f64 as SimdElement>::Mask, N>, material: &'a Arc<dyn Material>) {
        let update_mask = *valid_mask & (t.simd_le(self.t));

        self.normals.assign_masked(outward_normals, update_mask);
        masked_assign(&mut self.t, *t, update_mask);
        self.hits = self.hits | update_mask;

        self.materials.assign_masked(&PackedOptionalReference::splat(Some(material)), update_mask.cast())
        
        // for i in 0..N {
        //     if update_mask.test(i) {
        //         self.materials[i] = Some(material);
        //     }
        // }

    }

    pub fn finalize(&mut self, rays: &PackedRays<N>) {
        self.normals = self.normals.unit_vector();
        self.locations = rays.at_t(self.t);
        self.front_face = rays.directions().dot(&self.normals).simd_lt(Simd::splat(0.0));
        self.normals.assign_masked(&-self.normals, !self.front_face);
    }

    pub fn at(&self, index: usize) -> Option<HitRecord<'a>> {
        if self.hits.test(index) {
            Some(HitRecord {
                location: self.locations.at(index),
                normal: self.normals.at(index),
                t: self.t[index],
                front_face: self.front_face.test(index),
                material: self.materials.at(index).unwrap(),
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

    pub fn t(&self) -> Simd<f64, N> {
        self.t
    }

    pub fn hits(&self) -> Mask<<f64 as SimdElement>::Mask, N> {
        self.hits
    }

    pub fn front_face(&self) -> Mask<<f64 as SimdElement>::Mask, N> {
        self.front_face
    }

    // pub fn material(&self) -> &[Option<&'a Arc<dyn Material>>; N] {
    //     &self.materials
    // }
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
    
    pub fn hit_packed<'a, const N: usize>(&'a self, rays: &PackedRays<N>, t_range: &std::ops::Range<f64>, hit_records: &mut PackedHitRecords<'a, N>) 
    where LaneCount<N>: SupportedLaneCount
    {
        let center_offset = rays.origins() - self.center;
        let a = rays.directions().length_squared();
        let inverse_a = Simd::splat(1.0) / a;
        let half_b = center_offset.dot(&rays.directions());
        let c = center_offset.length_squared() - Simd::splat(self.radius.powi(2));
        let discriminant = half_b.mul_add(half_b, -a * c);

        let discriminant_positive = discriminant.simd_ge(Simd::splat(0.0)) & rays.enabled();

        if discriminant_positive.any() {
            let neg_half_b = negate_simd_float(half_b);
            let sqrt_discriminant = discriminant.sqrt();
            // let mut root = (neg_half_b - sqrt_discriminant) * inverse_a;
            // let mut root_valid = (simd_inside(&root, t_range)) & discriminant_positive;

            // masked_assign(&mut root, (neg_half_b + sqrt_discriminant) * inverse_a, !root_valid);
            // root_valid = simd_inside(&root, t_range) & discriminant_positive;

            let root1 = (neg_half_b - sqrt_discriminant) * inverse_a;
            let root2 = (neg_half_b + sqrt_discriminant) * inverse_a;
            let root1_valid = simd_inside(&root1, t_range);
            let root2_valid = simd_inside(&root1, t_range);

            let root = masked_select(root2, root1, root1_valid);

            let valid = (root1_valid | root2_valid)& rays.enabled();

            let locations = rays.at_t(root);
            let normal = locations - self.center;
            
            hit_records.update(
                rays, 
                &normal,
                &root, 
                &valid, 
                &self.material
            )
        }
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