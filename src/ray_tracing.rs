use crate::color::PackedColor;
use crate::packed::{PackedF64Mask, Scaler, Mask};
use crate::{color::Color, objects::Object};
use crate::objects::{HitRecord, PackedHitRecords, HitResult};
use crate::geometry::{Vec3, Point3, PackedVec3, PackedPoint3};
use crate::ray::{Ray, PackedRays};

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
#[derive(Clone)]
pub struct Scene {
    objects: Vec<Object>
}

struct RayState {
    color: Color,
    ray: Option<Ray>
}

#[derive(Debug)]
#[derive(Clone, Copy)]
struct CombinedIndex<const N: usize> {
    chuck_index: usize,
    slot_index: usize
}

impl <const N: usize> CombinedIndex<N> {
    fn before_first(rays: &[PackedRays<N>]) -> CombinedIndex<N>{
        return CombinedIndex { chuck_index: 0, slot_index: N }
    }

    fn after_last(rays: &[PackedRays<N>]) -> CombinedIndex<N> {
        return CombinedIndex { chuck_index: rays.len(), slot_index: 0 }
    }

    fn increment(&self, rays: &[PackedRays<N>]) -> Option<CombinedIndex<N>> {
        // println!("IS {:?}", self);
        if self.slot_index >= N {
            // special before first
            if rays.len() > 0 {
                // println!("SPECIAL");
                return Some(CombinedIndex {
                    chuck_index: 0,
                    slot_index: 0,
                });
            } else {
                return None;
            }
        }else if self.slot_index < N - 1 {
            Some(CombinedIndex {
                chuck_index: self.chuck_index,
                slot_index: self.slot_index + 1
            })
        } else if self.chuck_index < rays.len() - 1 {
            Some(CombinedIndex {
                chuck_index: self.chuck_index + 1,
                slot_index: 0
            })
        } else {
            None
        }
    }

    fn decrement(&self, _: &[PackedRays<N>]) -> Option<CombinedIndex<N>> {
        if self.slot_index > 0 {
            Some(CombinedIndex {
                chuck_index: self.chuck_index,
                slot_index: self.slot_index - 1
            })
        } else if self.chuck_index > 0 {
            Some(CombinedIndex {
                chuck_index: self.chuck_index - 1,
                slot_index: N - 1
            })
        } else {
            None
        }
    }


    fn next_disabled(&self, rays: &[PackedRays<N>]) -> Option<CombinedIndex<N>>{
        let mut index = self.increment(rays);
        // println!("NDI {:?}", index);
        loop {
            match index {
                Some(safe_index) => {
                    // println!("ND {:?}", index);
                    if !rays[safe_index.chuck_index].is_enabled(safe_index.slot_index) {
                        return Some(safe_index);
                    }

                    index = safe_index.increment(rays)
                },
                None => return None,
            }

        }
    }

    fn previous_enabled(&self, rays: &[PackedRays<N>]) -> Option<CombinedIndex<N>>{
        let mut index = self.decrement(rays);
        loop {
            match index {
                Some(safe_index) => {
                    // println!("PE {:?}", self);
                    if rays[safe_index.chuck_index].is_enabled(safe_index.slot_index) {
                        return Some(safe_index);
                    }

                    index = safe_index.decrement(rays)
                },
                None => return None,
            }

        }
    }

    fn valid(&self, rays: &[PackedRays<N>]) -> bool {
        self.chuck_index < rays.len() && self.slot_index < N
    }
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
        let mut hit_sky = PackedF64Mask::<N>::broadcast_bool(false);

        for _ in 0..depth_limit {
            if !rays.any_enabled() {
                // all rays have ended
                break;
            }

            let mut hit_records = PackedHitRecords::<N>::default();

            for object in &self.objects {
                object.hit_packed(&rays, &(0.001..f64::INFINITY), &mut hit_records)
            }

            hit_records.finalize(&rays);

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
                        hit_sky[i] = <f64 as Scaler>::MaskType::mask_from_bool(true);
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
        let mut hit_sky: [Vec<PackedF64Mask<N>>; 2] = array![vec![PackedF64Mask::<N>::broadcast_bool(false); rays.len()]; 2];

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

                hit_records.finalize(&ray_buffers[selector][j]);
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
                            hit_sky[selector][j][i] = u64::mask_from_bool(true);
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

    pub fn trace_vectorized3<const N: usize> (
        &self,
        rays: &mut [PackedRays<N>],
        depth_limit: usize,
    ) -> Color {
        let mut color = vec![PackedColor::<N>::broadcast_scaler(Color::white()); rays.len()];
        let mut hit_sky = vec![PackedF64Mask::<N>::broadcast_bool(false); rays.len()];

        let mut last_active_chunk = rays.len(); // set to one above the last active chunk

        for _ in 0..depth_limit {
            if last_active_chunk == 0 {
                // all rays have ended
                break;
            }

            for j in 0..last_active_chunk {
                let mut hit_records = PackedHitRecords::<N>::default();

                for object in &self.objects {
                    object.hit_packed(&rays[j], &(0.001..f64::INFINITY), &mut hit_records)
                }

                hit_records.finalize(&rays[j]);


                // let mut attenuations = PackedColor::<N>::broadcast_scaler(Color::white());

                for i in 0..N {
                    match hit_records.at(i) {
                        Some(hit_record) => {
                            let hit_result = hit_record.hit_result(&(rays[j].at(i).unwrap()));
                            let new_color = color[j].at(i) * hit_result.attenuation();
                            color[j].update(new_color, i);
                            match hit_result.scattered_ray() {
                                Some(ray) => {
                                    rays[j].update(i, *ray);
                                },
                                None => {
                                    rays[j].disable(i);
                                },
                            }
                        },
                        None => {
                            rays[j].disable(i);
                            hit_sky[j][i] = u64::mask_from_bool(true);
                        },
                    }
                }
            }
            
            // shuffle;

            let mut front_index = CombinedIndex::before_first(&rays);
            let mut back_index = CombinedIndex::after_last(&rays);

            loop {
                front_index = match front_index.next_disabled(&rays) {
                    Some(ci) => ci,
                    None => {
                        last_active_chunk = rays.len();
                        break;
                    },
                };

                back_index = match back_index.previous_enabled(&rays) {
                    Some(ci) => ci,
                    None => {
                        last_active_chunk = 0;
                        break;
                    },
                };

                if front_index.chuck_index >= back_index.chuck_index {
                    last_active_chunk = back_index.chuck_index + 1;
                    break;
                }

                // swap the items at front and back index
                let front_ray = rays[front_index.chuck_index].at_including_disabled(front_index.slot_index);
                let back_ray = rays[back_index.chuck_index].at_including_disabled(back_index.slot_index);
                rays[front_index.chuck_index].update_with_enable(front_index.slot_index, back_ray, true);
                rays[back_index.chuck_index].update_with_enable(back_index.slot_index, front_ray, false);

                let front_color = color[front_index.chuck_index].at(front_index.slot_index);
                let back_color = color[back_index.chuck_index].at(back_index.slot_index);
                color[front_index.chuck_index].update(back_color, front_index.slot_index);
                color[back_index.chuck_index].update(front_color, back_index.slot_index);

                let front_hit_sky = hit_sky[front_index.chuck_index][front_index.slot_index];
                let back_hit_sky = hit_sky[back_index.chuck_index][back_index.slot_index];
                hit_sky[front_index.chuck_index][front_index.slot_index] = back_hit_sky;
                hit_sky[back_index.chuck_index][back_index.slot_index] = front_hit_sky;

            }
        }


        for j in 0..rays.len() {
            // apply sky color to those rays that didn't hit the sky
            let a = (rays[j].directions().y() + 1.0) * 0.5;
            let sky_color_part_1 = PackedColor::<N>::broadcast_scaler(Color::white()) * (-a + 1.0);
            let sky_color_part_2 = PackedColor::<N>::broadcast_scaler(Color::new(0.5, 0.7, 1.0)) * a;
            let sky_color = sky_color_part_1 + sky_color_part_2;
            let sky_color_result = color[j] * sky_color;
            color[j].assign_masked(sky_color_result, hit_sky[j]);
            color[j].assign_masked(PackedColor::<N>::broadcast_scaler(Color::black()), rays[j].enabled());
        }

        let mut sum_color = PackedColor::<N>::broadcast_scaler(Color::black());
        for color_chunk in &color {
            sum_color = sum_color +  *color_chunk;
        }

        sum_color.sum()
    }   
}
 