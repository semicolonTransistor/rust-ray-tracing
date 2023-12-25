use crate::{geometry::{Vec3, Point3}, color::Color};
use crate::materials::Material;

use std::{sync::Arc, fmt::Debug};
use rand::prelude::*;

#[derive(Debug)]
#[derive(Clone)]
pub struct Camera {
    image_width: usize,
    image_height: usize,
    camera_center: Point3,
    viewport_upper_left_corner: Point3,
    viewport_u: Vec3,
    viewport_v: Vec3,
    focal_length: f64,
}

impl Camera {
    pub fn new(image_width: usize, image_height: usize, focal_length: f64, view_angle: f64, camera_center: Point3) -> Camera {
        let aspect_ratio = (image_width as f64) / (image_height as f64);
        let diagonal_length = (view_angle.to_radians() / 2.0).tan() * focal_length * 2.0;
        let upper_left_diagonal_angle = aspect_ratio.atan();
        let viewport_height = upper_left_diagonal_angle.cos() * diagonal_length;
        let viewport_width = viewport_height * aspect_ratio;
        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0 , -viewport_height, 0.0);
        let viewport_upper_left_corner = camera_center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        println!("Parameters:");
        println!("\tCamera Center:            {}", camera_center);
        println!("\tViewport Height:          {}", viewport_height);
        println!("\tViewport Width:           {}", viewport_width);
        println!("\tViewport Top Left Corner: {}", viewport_upper_left_corner);

        Camera {
            image_width: image_width,
            image_height: image_height,
            camera_center: camera_center,
            viewport_upper_left_corner: viewport_upper_left_corner,
            viewport_u: viewport_u,
            viewport_v: viewport_v,
            focal_length: focal_length,
        }
    }

    pub fn get_ray(&self, col: usize,  row: usize) -> Ray {
        let x_offset = thread_rng().gen_range(0.0..1.0);
        let y_offset = thread_rng().gen_range(0.0..1.0);
        let pixel_offset = self.viewport_u * ((col as f64 + x_offset) / (self.image_width as f64)) + self.viewport_v * ((row as f64 + y_offset) / (self.image_height as f64));
        let pixel_center = self.viewport_upper_left_corner + pixel_offset;
        let ray_direction = (pixel_center - self.camera_center).unit();
        
        // println!("{:?}", Ray::new(pixel_center, ray_direction));

        Ray::new(self.camera_center, ray_direction)
    }

    pub fn image_width(&self) -> usize {
        self.image_width
    }

    pub fn image_height(&self) -> usize {
        self.image_height
    }
}

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
#[derive(Clone)]
pub struct HitRecord {
    location: Point3,
    normal: Vec3,
    t: f64,
    front_face: bool,
    material: Arc<dyn Material>,
}

impl HitRecord {
    pub fn new(ray: &Ray, location: Point3, outward_normal: Vec3, t: f64, material: &Arc<dyn Material>) -> HitRecord {
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
            material: material.clone()
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
    
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_range: &std::ops::Range<f64>) -> Option<HitRecord> {
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
}
#[derive(Debug)]
#[derive(Clone)]
pub struct Scene {
    objects: Vec<Arc<dyn Hittable>>
}

impl Scene {
    pub fn new() -> Scene {
        Scene { objects: Vec::new() }
    }

    pub fn from_list(list: &[Arc<dyn Hittable>]) -> Scene {
        Scene { objects: list.to_vec()}
    }

    pub fn add(&mut self, object: &Arc<dyn Hittable>) {
        self.objects.push(object.clone());
    }

    pub fn hit(&self, ray: &Ray, t_range: std::ops::Range<f64>) -> Option<HitRecord> {
        let mut hit_record: Option<HitRecord> = None;
        let mut search_range = t_range;

        for object in &self.objects {
            hit_record = match object.hit(ray, &search_range) {
                Some(new_hit_record) => {
                    search_range = search_range.start..new_hit_record.t;
                    Some(new_hit_record)
                },
                None => hit_record,
            }
        }

        hit_record
    }

    pub fn trace(&self, ray: &Ray, depth_limit: usize) -> Color {
        if depth_limit == 0 {
            Color::black()
        } else {
            match self.hit(ray, 0.001..f64::INFINITY) {
                Some(hit_record) => {
                    let hit_result = hit_record.hit_result(ray);
                    match hit_result.scattered_ray() {
                        Some(scattered_ray) => {
                            self.trace(scattered_ray, depth_limit - 1) * hit_result.attenuation()
                        },
                        None => hit_result.attenuation,
                    }
                },
                None => {
                    let direction = ray.direction();
                    let a = 0.5 * (direction.y() + 1.0);
                    Color::new(
                        (1.0 - a) + a * 0.5, 
                        (1.0 - a) + a * 0.7, 
                        (1.0 - a) + a * 1.0
                    )
                },
            }
        }
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
 