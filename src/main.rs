#![feature(portable_simd)]
#![feature(stdsimd)]

mod geometry;
mod color;
mod real;
mod ray_tracing;
mod materials;
mod renderer;
mod objects;
mod toml_utils;
// mod packed;
mod ray;
mod simd_util;

use clap::{Parser, ValueEnum};
use geometry::Vec3;
use ray_tracing::{Camera, Scene};
use std::{sync::Arc, num::NonZeroUsize, path::{Path, PathBuf}, fs::File, io::Read};
use renderer::TileRenderer;
use crate::toml_utils::to_float;
use crate::real::{Real, duration_as_secs_real};

use crate::{materials::get_materials, objects::get_object_list, renderer::TileRenderMode};

fn read_file_into_string (path: &Path) -> std::io::Result<String> {
    let mut file_content = String::new();
    File::open(path)?.read_to_string(&mut file_content)?;
    Ok(file_content)
}

#[derive(Parser)]
#[command(name = "Ray Tracer")]
#[command(author = "Jinyu Liu <liu.jinyu@psu.edu>")]
#[command(about = "Multithreaded ray tracing implementation based on \"Ray Tracing in One Weekend\"")]
struct Cli{
    
    /// Scene file location
    #[arg(short, long, value_name = "FILE")]
    scene: PathBuf,

    /// Camera parameters, if not specified the parameters will be loaded from scene file instead.
    #[arg(short, long, value_name = "FILE")]
    camera: Option<PathBuf>,

    /// Report file, specify to write render stats to a file
    #[arg(short, long, value_name = "FILE")]
    report: Option<PathBuf>,

    /// Image Width
    #[arg(short, long, value_name = "PIXELS", default_value="3840", value_parser= clap::value_parser!(u64).range(1..))]
    width: u64,

    /// Image Height
    #[arg(short, long, value_name = "PIXELS", default_value="2160", value_parser= clap::value_parser!(u64).range(1..)) ]
    height: u64,

    /// Output image location
    #[arg(short, long, value_name = "FILE", default_value="output.png")]
    output_image: PathBuf,


    /// Renderer mode
    #[arg(short, long, value_enum, value_name = "Mode", default_value="vectorized")]
    render_mode: RendererMode,

    /// The size of tiles, sets the size tasks assign to threads when rendering.
    #[arg(short, long, default_value="128", value_parser= clap::value_parser!(u64).range(1..1024))]
    tile_size: u64,

    /// Number of samples to take for each pixel. Higher value increase anti-aliasing quality but decreases performance.
    #[arg(short, long, default_value="100", value_parser= clap::value_parser!(u64).range(1..1024))]
    samples_per_pixel: u64,

    /// Max number of bounces to calculate per ray. Higher value increase anti-aliasing quality but decreases performance.
    #[arg(short, long, default_value="50", value_parser= clap::value_parser!(u64).range(1..))]
    bounces: u64,

    /// Number of threads to use, default to the number of logical processors if not specified
    #[arg(short, long, value_parser= clap::value_parser!(u64).range(1..))]
    thread_count: Option<u64>,
}

#[derive(Clone, Copy)]
#[derive(Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(ValueEnum)]
enum RendererMode {
    // scaler version
    Scaler,

    // SIMD version
    Vectorized,
}

fn main() -> image::ImageResult<()> {

    let cli_arguments = Cli::parse();

    let path = Path::new("scene.toml");

    let scene_config_content = match read_file_into_string(&cli_arguments.scene) {
        Ok(c) => c,
        Err(why) => panic!("Can't read scene from file {}: {}", path.display(), why),
    };

    let scene_data = match toml::from_str::<toml::Table>(&scene_config_content) {
        Ok(t) => t,
        Err(why) => panic!("Failed parsing scene from file {}: {}", path.display(), why),
    };

    let scene = load_scene(&scene_data);

    let image_width = cli_arguments.width.try_into().unwrap();
    let image_height = cli_arguments.height.try_into().unwrap();

    // get camera
    let (focal_length, fov, center, look_at, up, defocus_angle) = match cli_arguments.camera {
        Some(camera_file_path) => {
            let camera_config_content = match read_file_into_string(&camera_file_path) {
                Ok(c) => c,
                Err(why) => panic!("Can't read camera from file {}: {}", path.display(), why),
            };

            let camera_data = match toml::from_str::<toml::Table>(&camera_config_content) {
                Ok(t) => t,
                Err(why) => panic!("Failed parsing camera from file {}: {}", path.display(), why),
            };

            match load_camera(&camera_data) {
                Some(v) => v,
                None => panic!("Camera file don't contain a valid camera!"),
            }
        
        },
        None => match load_camera(&scene_data) {
            Some(v) => v,
            None => panic!("No camera file provided and scene file didn't contain a valid camera"),
        },
    };

    let camera = Arc::new(Camera::new(
        image_width, image_height,
        focal_length, fov, 
        center, look_at, up, defocus_angle
    ));

    println!("\t Number of objects: \t {}", scene.len());

    let render_mode = match cli_arguments.render_mode {
        RendererMode::Scaler => TileRenderMode::Scaler,
        RendererMode::Vectorized => TileRenderMode::Vectorized,
    };
    let renderer = TileRenderer::new(
        match cli_arguments.thread_count {
            Some(tc) => Some(NonZeroUsize::new(tc.try_into().unwrap()).unwrap()),
            None => None,
        }, 
        NonZeroUsize::new(cli_arguments.tile_size.try_into().unwrap()).unwrap(), 
        render_mode
    );

    let (render_result, render_stat) = renderer.render(
        cli_arguments.bounces.try_into().unwrap(), 
        cli_arguments.samples_per_pixel.try_into().unwrap(),  
        &scene, 
        &camera
    );

    render_result.save(cli_arguments.output_image)?;

    println!("Image Size: {} x {}", camera.image_width(), camera.image_height());
    println!("Total Pixels: {}", render_stat.pixels_rendered());
    println!("Time Taken: {:.3} seconds", duration_as_secs_real(&render_stat.duration()));
    println!("Average Pixel Rate: {:.2} px/s", render_stat.pixels_per_second());

    match cli_arguments.report {
        Some(report_path) => {
            let mut report = toml::value::Table::new();
            report.insert("image_size".to_owned(), toml::Value::try_from([camera.image_width(), camera.image_height()]).unwrap());
            report.insert("total_pixels".to_owned(), toml::Value::Integer(render_stat.pixels_rendered() as i64));
            report.insert("time_taken".to_owned(), toml::Value::Float(render_stat.duration().as_secs_f64()));
            report.insert("average_pixel_rate".to_owned(), toml::Value::Float(render_stat.pixels_per_second() as f64));

            match render_stat.detailed_stat() {
                Some(stat) => {
                    report.insert("renderer_detailed_stat".to_owned(), toml::Value::Table(stat.clone()));
                },
                None => todo!(),
            }

            let serialized_report = toml::to_string(&report).unwrap();

            std::fs::write(report_path, serialized_report)?;
        },
        None => (),
    };

    Ok(())
}

fn load_scene(table: &toml::value::Table) -> Arc<Scene>{
    let material_toml_table = match table["materials"].as_table() {
        Some(t) => t,
        None => panic!("Can't find materials table!"),
    };

    let materials_table = get_materials(material_toml_table);

    let objects_toml_array = match table.get("objects") {
        Some(a) => a.as_array().unwrap(),
        None => match table.get("hitables") {
            Some(a) => a.as_array().unwrap(),
            None => panic!("Can't find the objects array")
        },
    };

    let objects = get_object_list(objects_toml_array, &materials_table);

    Arc::new(
        Scene::from_list(&objects)
    )
}

fn load_camera(table: &toml::value::Table) -> Option<(Real, Real, Vec3, Vec3, Vec3, Real)> {
    let camera_toml_table = match table.get("camera") {
        Some(t) => t.as_table().unwrap(),
        None => return None,
    };

    let focal_length = match camera_toml_table.get("focal_length") {
        Some(f) => to_float(f).unwrap(),
        None => 10.0,
    };

    let fov = match camera_toml_table.get("fov") {
        Some(f) => to_float(f).unwrap(),
        None => 30.0,
    };

    let center = match camera_toml_table.get("center") {
        Some(v) => Vec3::from_toml(v).unwrap(),
        None => match camera_toml_table.get("look_from") {
            Some(v) => Vec3::from_toml(v).unwrap(),
            None => match camera_toml_table.get("lookFrom") {
                Some(v) => Vec3::from_toml(v).unwrap(),
                None => panic!("Can't find camera center"),
            },
        },
    };

    let look_at = match camera_toml_table.get("look_at") {
        Some(v) => Vec3::from_toml(v).unwrap(),
        None => match camera_toml_table.get("lookAt") {
            Some(v) => Vec3::from_toml(v).unwrap(),
            None => panic!("Can't find look_at"),
        },
    };

    let up = match camera_toml_table.get("up") {
        Some(v) => Vec3::from_toml(v).unwrap(),
        None => match camera_toml_table.get("lookUp") {
            Some(v) => Vec3::from_toml(v).unwrap(),
            None => panic!("Can't find up"),
        },
    };


    let defocus_angle = match camera_toml_table.get("defocus_angle") {
        Some(f) => to_float(f).unwrap(),
        None => 0.0,
    };

    Some((focal_length, fov, center, look_at, up, defocus_angle))
}