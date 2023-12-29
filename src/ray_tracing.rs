use crate::color::PackedColor;
use crate::{color::Color, objects::Object};
use crate::objects::{HitRecord, PackedHitRecords, HitResult};
use crate::geometry::{Vec3, Point3, PackedVec3, PackedPoint3};
use crate::packed::{PackedBool, PackedF64, PackedScalerPartialEq, PackedScalerPartialOrd};

use std::fmt::Debug;
use rand::prelude::*;
use array_macro::array;

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
        let center = Vec3::from_toml(&table["center"]).unwrap();
        let look_at =  Vec3::from_toml(&table["look_at"]).unwrap();
        let up = Vec3::from_toml(&table["up"]).unwrap();
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
#[derive(Copy, Clone)]
pub struct PackedRays<const N: usize> {
    origins: PackedPoint3<N>,
    directions: PackedVec3<N>,
    enabled: PackedBool<N>
}

impl <const N: usize> PackedRays<N> {
    #[inline]
    pub fn new(origins: PackedPoint3<N>, directions: PackedVec3<N>) -> PackedRays<N> {
        PackedRays {
            origins,
            directions,
            enabled: PackedBool::<N>::broadcast_scaler(true)
        }
    }

    #[inline]
    pub fn new_with_enable(origins: PackedPoint3<N>, directions: PackedVec3<N>, enabled: PackedBool<N>) -> PackedRays<N> {
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
    pub fn enabled(&self) -> PackedBool<N> {
        self.enabled
    }

    #[inline]
    pub fn is_enabled(&self, index: usize) -> bool {
        self.enabled[index]
    }

    #[inline]
    pub fn count() -> usize {
        N
    }

    #[inline]
    pub fn at(&self, index: usize) -> Option<Ray> {
        if self.enabled[index] {
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
    pub fn at_t(&self, t: PackedF64<N>) -> PackedPoint3<N> {
        self.origins + self.directions * t
    }

    #[inline]
    pub fn update(&mut self, index: usize, value: Ray) {
        self.origins.update(index, value.origin());
        self.directions.update(index, value.direction());
        self.enabled[index] = true;
    }

    #[inline]
    pub fn update_with_enable(&mut self, index: usize, value: Ray, enable: bool) {
        self.origins.update(index, value.origin());
        self.directions.update(index, value.direction());
        self.enabled[index] = enable;
    }

    #[inline]
    pub fn any_enabled(&self) -> bool {
        self.enabled().any()
    }

    #[inline]
    pub fn enable(&mut self, index: usize) {
        self.enabled[index] = true;
    }

    #[inline]
    pub fn disable(&mut self, index: usize) {
        self.enabled[index] = false;
    }
}

impl <const N: usize> FromIterator<Ray> for PackedRays<N> {
    fn from_iter<T: IntoIterator<Item = Ray>>(iter: T) -> Self {
        let mut packed_rays = PackedRays {
            directions: PackedVec3::default(),
            origins: PackedPoint3::default(),
            enabled: PackedBool::broadcast_scaler(false)
        };

        for (index, value) in iter.into_iter().enumerate() {
            assert!(index < N, "too may elements given in iterator!");
            packed_rays.update(index, value)
        }

        packed_rays
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

    pub fn trace_vectorized<const N: usize>(
        &self,
        mut rays: PackedRays<N>,
        depth_limit: usize,
    ) -> PackedColor<N> 
    {
        let mut color = PackedColor::<N>::broadcast_scaler(Color::black());
        color.assign_masked(PackedColor::broadcast_scaler(Color::white()), rays.enabled());
        let mut hit_sky = PackedBool::<N>::broadcast_scaler(false);

        for _ in 0..depth_limit {
            if !rays.any_enabled() {
                // all rays have ended
                break;
            }

            let mut hit_records = PackedHitRecords::<N>::default();

            for object in &self.objects {
                object.hit_packed(&rays, &(0.001..f64::INFINITY), &mut hit_records)
            }


            // let mut attenuations = PackedColor::<N>::broadcast_scaler(Color::white());

            for i in 0..N {
                match hit_records.at(i) {
                    Some(hit_record) => {
                        let hit_result = hit_record.hit_result(&(rays.at(i).unwrap()));
                        color.update(color.at(i) * hit_result.attenuation(), i);
                        match hit_result.scattered_ray() {
                            Some(ray) => {
                                rays.update(i, *ray);
                            },
                            None => {
                                rays.disable(i);
                            },
                        }
                    },
                    None => {
                        rays.disable(i);
                        hit_sky[i] = true;
                    },
                }
            }

            // color = color * attenuations;
        }

        // apply sky color to those rays that didn't hit the sky
        let a = (rays.directions().y() + 1.0) * 0.5;
        let sky_color_part_1 = PackedColor::<N>::broadcast_scaler(Color::white()) * (-a + 1.0);
        let sky_color_part_2 = PackedColor::<N>::broadcast_scaler(Color::new(0.5, 0.7, 1.0)) * a;
        let sky_color = sky_color_part_1 + sky_color_part_2;
        color.assign_masked(color * sky_color, hit_sky);
        color.assign_masked(PackedColor::<N>::broadcast_scaler(Color::black()), rays.enabled());

        color
    }

    pub fn trace_vectorized2<const N: usize> (
        &self,
        rays: &[PackedRays<N>],
        depth_limit: usize,
    ) -> Color {
        let mut ray_buffers: [Vec<PackedRays<N>>; 2] = [rays.to_vec(), vec![PackedRays::<N>::new(PackedVec3::default(), PackedVec3::default()); rays.len()]];
        let mut color:[Vec<PackedColor<N>>; 2] = array![vec![PackedColor::<N>::broadcast_scaler(Color::white()); rays.len()]; 2];
        let mut hit_sky: [Vec<PackedBool<N>>; 2] = array![vec![PackedBool::<N>::broadcast_scaler(false); rays.len()]; 2];

        let mut last_active_chunk = rays.len(); // set to one above the last active chunk

        for k in 0..depth_limit {
            if last_active_chunk == 0 {
                // all rays have ended
                break;
            }

            let selector = k % 2;

            for j in 0..last_active_chunk {
                let mut hit_records = PackedHitRecords::<N>::default();

                for object in &self.objects {
                    object.hit_packed(&ray_buffers[selector][j], &(0.001..f64::INFINITY), &mut hit_records)
                }


                // let mut attenuations = PackedColor::<N>::broadcast_scaler(Color::white());

                for i in 0..N {
                    match hit_records.at(i) {
                        Some(hit_record) => {
                            let hit_result = hit_record.hit_result(&(ray_buffers[selector][j].at(i).unwrap()));
                            let new_color = color[selector][j].at(i) * hit_result.attenuation();
                            color[selector][j].update(new_color, i);
                            match hit_result.scattered_ray() {
                                Some(ray) => {
                                    ray_buffers[selector][j].update(i, *ray);
                                },
                                None => {
                                    ray_buffers[selector][j].disable(i);
                                },
                            }
                        },
                        None => {
                            ray_buffers[selector][j].disable(i);
                            hit_sky[selector][j][i] = true;
                        },
                    }
                }
            }
            
            // shuffle;
            let mut output_chunk = 0;
            let mut output_slot = 0;

            let next_selector = 1 - selector;

            for i in 0..last_active_chunk {
                for j in 0..N {
                    match ray_buffers[selector][i].at(j) {
                        Some(ray) => {
                            // println!("RB_LEN {:?}", ray_buffers.len());
                            // println!("NS {}", next_selector);
                            // println!("RB_NS_LEN {}", ray_buffers[next_selector].len());
                            ray_buffers[next_selector][output_chunk].update(output_slot, ray);
                            let tmp_color = color[selector][i].at(j);
                            color[next_selector][output_chunk].update(tmp_color, output_slot);
                            let tmp_hit_sky = hit_sky[selector][i][j];
                            hit_sky[next_selector][output_chunk][output_slot] = tmp_hit_sky;

                            output_slot += 1;

                            if output_slot >= N {
                                output_slot = 0;
                                output_chunk += 1;
                            }
                        },
                        None => (),
                    }
                }
            }

            
            let new_last_active_chunk = if output_slot == 0 {output_chunk} else {output_chunk + 1};

            for i in 0..last_active_chunk {
                for j in 0..N {
                    if !ray_buffers[selector][i].is_enabled(j) {
                        let ray = ray_buffers[selector][i].at_including_disabled(j);
                        ray_buffers[next_selector][output_chunk].update_with_enable(output_slot, ray, false);
                        let tmp_color = color[selector][i].at(j);
                        color[next_selector][output_chunk].update(tmp_color, output_slot);
                        let tmp_hit_sky = hit_sky[selector][i][j];
                        hit_sky[next_selector][output_chunk][output_slot] = tmp_hit_sky;

                        output_slot += 1;

                        if output_slot >= N {
                            output_slot = 0;
                            output_chunk += 1;
                        }
                    }
                }
            }

            last_active_chunk = new_last_active_chunk;
        }

        let selector = (rays.len() - 1) % 2;

        for j in 0..rays.len() {
            // apply sky color to those rays that didn't hit the sky
            let a = (rays[j].directions().y() + 1.0) * 0.5;
            let sky_color_part_1 = PackedColor::<N>::broadcast_scaler(Color::white()) * (-a + 1.0);
            let sky_color_part_2 = PackedColor::<N>::broadcast_scaler(Color::new(0.5, 0.7, 1.0)) * a;
            let sky_color = sky_color_part_1 + sky_color_part_2;
            let sky_color_result = color[selector][j] * sky_color;
            color[selector][j].assign_masked(sky_color_result, hit_sky[selector][j]);
            color[selector][j].assign_masked(PackedColor::<N>::broadcast_scaler(Color::black()), ray_buffers[selector][j].enabled());
        }

        let mut sum_color = PackedColor::<N>::broadcast_scaler(Color::black());
        for color_chunk in &color[selector] {
            sum_color = sum_color +  *color_chunk;
        }

        sum_color.sum()
    }   
}
 