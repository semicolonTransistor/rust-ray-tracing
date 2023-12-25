mod geometry;
mod color;
mod ray_tracing;
mod materials;
mod renderer;

use image::{RgbImage, Rgb};

use color::Color;
use geometry::{Vec3, Point3};
use ray_tracing::{Camera, Scene, Sphere, Hittable};
use materials::{Lambertian, Material, Metal, Dielectric};
use std::{sync::Arc, num::NonZeroUsize};
use renderer::{Renderer, TileRenderer};

fn main() -> image::ImageResult<()>{
    // let image_width = 400;
    // let image_height = 225;

    let image_width = 3840;
    let image_height = 2160;
    // let max_pixel_value = 256;

    let camera = Arc::new(Camera::new(image_width, image_height, 1.0, 127.76, Vec3::zero()));

    let material_ground: Arc<dyn Material> = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.1)));
    let material_center: Arc<dyn Material> = Arc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    // let material_left: Arc<dyn Material> = Arc::new(Metal::new(Color::new(0.8, 0.8, 0.8), 0.3));
    // let material_center: Arc<dyn Material> = Arc::new(Dielectric::new(1.5));
    let material_left: Arc<dyn Material> = Arc::new(Dielectric::new(1.5));
    let material_right: Arc<dyn Material> = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 1.0));

    // World
    let world_objects: Vec<Arc<dyn Hittable>> = vec![
        Arc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, &material_ground)),
        Arc::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, &material_center)),
        Arc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), -0.5, &material_left)),
        Arc::new(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, &material_right)),
    ];

    let world = Arc::new(Scene::from_list(&world_objects));

    let renderer = TileRenderer::new(None, NonZeroUsize::new(256).unwrap());

    let render_result = renderer.render(50, 100,  &world, &camera);

    render_result.save("output.png")?;

    // let mut image = RgbImage::new(image_width, image_height);

    // let start = std::time::Instant::now();
    // let mut start_current_line = start.clone();
    // for row in 0..image_height {
    //     for col in 0..image_width {

    //         let pixel = Color::average(
    //             (0..100).map(|_| camera.get_ray(col, row)).map(|ray| world.trace(&ray, 50))
    //         );

    //         //println!("{:?}", pixel);

    //         image.put_pixel(col, row, Rgb(pixel.to_u8_array()));
    //     }
    //     let end_current_line = std::time::Instant::now();
    //     let duration_of_line = end_current_line.duration_since(start_current_line);
    //     let duration_so_far = end_current_line.duration_since(start);
    //     let per_pixel_this_line = duration_of_line / image_width;
    //     let per_pixel_so_far = duration_so_far / (image_width * (row + 1));

    //     start_current_line = end_current_line;


    //     eprintln!("Completed Line {} out of {}, \t {:.2} px/s, \t {:.2} px/s", row + 1, image_height, 1.0 / per_pixel_this_line.as_secs_f64(), 1.0 / per_pixel_so_far.as_secs_f64());

    //     image.save("output.png")?;
    // }

    Ok(())
}
