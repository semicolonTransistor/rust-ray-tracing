use crate::{geometry::{Vec3, Point3}, color::Color, objects::Object};
use crate::objects::HitRecord;

use std::fmt::Debug;
use rand::prelude::*;

#[derive(Debug)]
#[derive(Clone)]
pub struct Camera {
    image_width: usize,
    image_height: usize,
    center: Point3,
    viewport_upper_left_corner: Point3,
    viewport_u: Vec3,
    viewport_v: Vec3,
    defocus_u: Vec3,
    defocus_v: Vec3,
}

impl Camera {
    pub fn new(image_width: usize, image_height: usize, focal_length: f64, view_angle: f64, center: Point3, look_at: Vec3, up: Vec3, defocus_angle: f64) -> Camera {
        let aspect_ratio = (image_width as f64) / (image_height as f64);
        let diagonal_length = (view_angle.to_radians() / 2.0).tan() * focal_length * 2.0;
        let upper_left_diagonal_angle = aspect_ratio.atan();
        let viewport_height = upper_left_diagonal_angle.cos() * diagonal_length;
        let viewport_width = viewport_height * aspect_ratio;

        let direction = (look_at - center).unit();
        let w = -direction;
        let u = up.cross(&w).unit();
        let v = w.cross(&u);

        let viewport_u = u * viewport_width;
        let viewport_v = -v * viewport_height;
        let viewport_upper_left_corner = center - w * focal_length - viewport_u / 2.0 - viewport_v / 2.0;

        let defocus_radius = focal_length * (defocus_angle / 2.0).to_radians().tan();
        let defocus_u = defocus_radius * u;
        let defocus_v = defocus_radius * v;
        println!("Parameters:");
        println!("\tCamera Center:            {}", center);
        println!("\tViewport Height:          {}", viewport_height);
        println!("\tViewport Width:           {}", viewport_width);
        println!("\tViewport Top Left Corner: {}", viewport_upper_left_corner);

        Camera {
            image_width,
            image_height,
            center,
            viewport_upper_left_corner,
            viewport_u,
            viewport_v,
            defocus_u,
            defocus_v,
        }
    }

    pub fn from_toml(table: &toml::Table) -> Camera {
        let image_width: usize = table["image_width"].as_integer().unwrap().try_into().unwrap();
        let image_height: usize = table["image_height"].as_integer().unwrap().try_into().unwrap();
        let focal_length: f64 = table["focal_length"].as_float().unwrap();
        let view_angle: f64 = table["view_angle"].as_float().unwrap();
        let center = Vec3::from_toml(table["center"].as_table().unwrap());
        let look_at =  Vec3::from_toml(table["look_at"].as_table().unwrap());
        let up = Vec3::from_toml(table["up"].as_table().unwrap());
        let defocus_angle = table["view_angle"].as_float().unwrap();

        Camera::new(image_width, image_height, focal_length, view_angle, center, look_at, up, defocus_angle)
    }

    pub fn get_ray(&self, col: usize,  row: usize) -> Ray {
        let x_offset = thread_rng().gen_range(0.0..1.0);
        let y_offset = thread_rng().gen_range(0.0..1.0);
        let pixel_offset = self.viewport_u * ((col as f64 + x_offset) / (self.image_width as f64)) + self.viewport_v * ((row as f64 + y_offset) / (self.image_height as f64));
        let pixel_center = self.viewport_upper_left_corner + pixel_offset;
        let disk_offset = Vec3::random_in_unit_disk();
        let ray_origin = self.defocus_u * disk_offset.x() + self.defocus_v * disk_offset.y() + self.center;
        let ray_direction = (pixel_center - ray_origin).unit();
        
        // println!("{:?}", Ray::new(pixel_center, ray_direction));

        Ray::new(ray_origin, ray_direction)
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
pub struct Scene {
    objects: Vec<Object>
}

struct RayState {
    color: Color,
    ray: Option<Ray>
}

impl Scene {
    pub fn new() -> Scene {
        Scene { objects: Vec::new() }
    }

    pub fn from_list(list: &[Object]) -> Scene {
        Scene { objects: list.to_vec()}
    }

    pub fn add(&mut self, object: Object) {
        self.objects.push(object);
    }

    
    pub fn hit(&self, ray: &Ray, t_range: std::ops::Range<f64>) -> Option<HitRecord> {
        // let mut hit_record: Option<HitRecord> = None;
        // let mut search_range = t_range;

        // for object in &self.objects {
        //     hit_record = match object.hit(ray, &search_range) {
        //         Some(new_hit_record) => {
        //             search_range = search_range.start..new_hit_record.t();
        //             Some(new_hit_record)
        //         },
        //         None => hit_record,
        //     }
        // }

        self.objects.iter().map(|obj| {
            obj.hit(ray, &t_range)
        }).filter_map(|e| e).min_by_key(|h| {ordered_float::OrderedFloat::from(h.t())})
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
                        None => hit_result.attenuation(),
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

    pub fn trace_rays(&self, rays: &[Ray], depth_limit: usize) -> Vec<Color> {
        let mut ray_stats = rays.iter().map(|r| {
            RayState {
                color: Color::white(),
                ray: Some(*r)
            }
        }).collect::<Vec<_>>();

        for _ in 0..depth_limit {
            for ray_state in &mut ray_stats {
                
                match ray_state.ray {
                    Some(ray) => {
                        match self.hit(&ray, 0.001..f64::INFINITY) {
                            Some(hit_record) => {
                                let hit_result = hit_record.hit_result(&ray);
                                ray_state.color = ray_state.color * hit_result.attenuation();
                                ray_state.ray = hit_result.scattered_ray().copied();
                            },
                            None => {
                                let direction = ray.direction();
                                let a = 0.5 * (direction.y() + 1.0);
                                ray_state.color = ray_state.color * Color::new(
                                    (1.0 - a) + a * 0.5, 
                                    (1.0 - a) + a * 0.7, 
                                    (1.0 - a) + a * 1.0
                                );
                                ray_state.ray = None;
                            },
                        }
                    },
                    None => (),
                }
            }
        }

        ray_stats.iter().map(|rs| {
            match rs.ray {
                Some(_) => Color::black(),
                None => rs.color,
            }
        }).collect()
    }
}
 