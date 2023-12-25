mod geometry;
mod color;
mod ray_tracing;
mod materials;
mod renderer;

use color::Color;
use geometry::{Vec3, Point3};
use ray_tracing::{Camera, Scene, Sphere, Hittable};
use materials::{Lambertian, Material, Metal, Dielectric};
use std::{sync::Arc, num::NonZeroUsize};
use renderer::{Renderer, TileRenderer};

fn main() -> image::ImageResult<()>{
    // let image_width = 400;
    // let image_height = 225;

    let image_width = 1920;
    let image_height = 1080;
    // let max_pixel_value = 256;

    let camera = Arc::new(Camera::new(image_width, image_height, 1.0, 127.76, Vec3::zero()));

    let material_ground: Arc<dyn Material> = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.1)));
    let material_center: Arc<dyn Material> = Arc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    // let material_left: Arc<dyn Material> = Arc::new(Metal::new(Color::new(0.8, 0.8, 0.8), 0.3));
    // let material_center: Arc<dyn Material> = Arc::new(Dielectric::new(1.5));
    let material_left: Arc<dyn Material> = Arc::new(Dielectric::new(1.5, false));
    let material_right: Arc<dyn Material> = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 1.0));

    // World
    let world_objects: Vec<Arc<dyn Hittable>> = vec![
        Arc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, &material_ground)),
        Arc::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, &material_center)),
        Arc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, &material_left)),
        Arc::new(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, &material_right)),
    ];

    let world = Arc::new(Scene::from_list(&world_objects));

    let renderer = TileRenderer::new(None, NonZeroUsize::new(128).unwrap());

    let (render_result, render_stat) = renderer.render(50, 100,  &world, &camera);

    render_result.save("output.png")?;

    println!("Image Size: {} x {}", camera.image_width(), camera.image_height());
    println!("Total Pixels: {}", render_stat.pixels_rendered());
    println!("Time Taken: {:.3} seconds", render_stat.duration().as_secs_f64());
    println!("Average Pixel Rate: {:.2} px/s", render_stat.pixels_per_second());

    Ok(())
}
