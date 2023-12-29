mod geometry;
mod color;
mod ray_tracing;
mod materials;
mod renderer;
mod objects;
mod toml_utils;
mod packed;

use geometry::Vec3;
use ray_tracing::{Camera, Scene};
use std::{sync::Arc, num::NonZeroUsize, path::Path, fs::File, io::Read};
use renderer::TileRenderer;

use crate::{materials::get_materials, objects::get_object_list};

fn read_file_into_string (path: &Path) -> std::io::Result<String> {
    let mut file_content = String::new();
    File::open(path)?.read_to_string(&mut file_content)?;
    Ok(file_content)
}

fn main() -> image::ImageResult<()> {

    let path = Path::new("scene.toml");

    let scene_config_content = match read_file_into_string(path) {
        Ok(c) => c,
        Err(why) => panic!("Can't read scene from file {}: {}", path.display(), why),
    };

    let scene_data = match toml::from_str::<toml::Table>(&scene_config_content) {
        Ok(t) => t,
        Err(why) => panic!("Failed parsing scene from file {}: {}", path.display(), why),
    };

    println!(
        "{}", scene_data
    );

    let materials = get_materials(scene_data["materials"].as_table().unwrap());

    let scene = Arc::new(Scene::from_list(&get_object_list(scene_data["objects"].as_array().unwrap(), &materials)));

    // println!("Materials {:?}", materials);
    // println!("Scene {:?}", scene);
    // let image_width = 400;
    // let image_height = 225;

    let image_width = 1280;
    let image_height = 720;
    // let max_pixel_value = 256;

    let camera = Arc::new(Camera::new(
        image_width, image_height,
        10.0, 30.0, 
        Vec3::new(13.0, 2.0, 3.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        0.6
    ));

    let renderer = TileRenderer::new(None, NonZeroUsize::new(128).unwrap());

    let (render_result, render_stat) = renderer.render(50, 128,  &scene, &camera);

    render_result.save("output.png")?;

    println!("Image Size: {} x {}", camera.image_width(), camera.image_height());
    println!("Total Pixels: {}", render_stat.pixels_rendered());
    println!("Time Taken: {:.3} seconds", render_stat.duration().as_secs_f64());
    println!("Average Pixel Rate: {:.2} px/s", render_stat.pixels_per_second());

    Ok(())
}
