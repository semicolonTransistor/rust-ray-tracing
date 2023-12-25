use crate::ray_tracing::{Scene, Camera};
use crate::color::Color;
use image::{Rgb, RgbImage};
use std::num::NonZeroUsize;
use std::sync::Arc;
use console::Term;
use std::io::Write;

pub trait Renderer {
    fn render(&self, max_bounces: usize, samples_per_pixel: usize, scene: &Arc<Scene>, camera: &Arc<Camera>) -> RgbImage;
}

pub struct TileRenderer {
    num_threads: NonZeroUsize,
    block_size: NonZeroUsize,
}

#[derive(Debug)]
#[derive(Clone, Default)]
struct TileRenderTask {
    block_index_x: usize,
    block_index_y: usize,
    size_x: usize,
    size_y: usize,
    block_size: usize,
}

#[derive(Debug)]
#[derive(Clone, Default)]
struct TileRenderResult {
    block_index_x: usize,
    block_index_y: usize,
    thread_id: usize,
    average_pixel_throughput: f64,
    output: Vec<Rgb<u8>>   
}

impl TileRenderTask {
    fn render(&self, camera: &Arc<Camera>, scene: &Arc<Scene>, max_bounces: usize, samples_per_pixel: usize, thread_id: usize) -> TileRenderResult {
        let mut result = vec![Rgb::<u8>([0, 0, 0]); self.block_size.pow(2)];

        let col_offset = self.block_index_x * self.block_size;
        let row_offset = self.block_index_y * self.block_size;

        let start = std::time::Instant::now();
        for j in 0..self.size_y {
            for i in 0..self.size_x {

                let col = i + col_offset;
                let row = j + row_offset;

                let pixel = Color::average(
                    (0..samples_per_pixel).map(|_| camera.get_ray(col, row)).map(|ray| scene.trace(&ray, max_bounces))
                );

                result[j * self.block_size + i] = Rgb(pixel.to_u8_array());
            }
        }
        let duration = std::time::Instant::now().duration_since(start);
        let pixels_per_second = ((self.size_x * self.size_y) as f64) / duration.as_secs_f64();

        TileRenderResult {
            block_index_x: self.block_index_x,
            block_index_y: self.block_index_y,
            thread_id: thread_id,
            average_pixel_throughput: pixels_per_second,
            output: result
        }
    }

    
}

#[derive(Debug)]
#[derive(Clone)]
struct TileRenderStartMessage {
    block_index_x: usize,
    block_index_y: usize,
}

#[derive(Debug)]
#[derive(Clone)]
enum TileRenderUpdates {
    Start(TileRenderStartMessage),
    End(TileRenderResult),
}

impl TileRenderer {
    pub fn new(num_threads: Option<NonZeroUsize>, block_size: NonZeroUsize) -> Box<dyn Renderer> {
        Box::new(TileRenderer {
            num_threads: match num_threads {
                Some(n) => n,
                None => std::thread::available_parallelism().unwrap(),
            },
            block_size
        })
    }
}

impl Renderer for TileRenderer {
    fn render(&self, max_bounces: usize, samples_per_pixel: usize, scene: &Arc<Scene>, camera: &Arc<Camera>) -> RgbImage {
        // divide image into blocks
        let width_in_blocks = (camera.image_width() + self.block_size.get() - 1) / self.block_size.get();
        let height_in_blocks = (camera.image_height() as usize+ self.block_size.get() - 1) / self.block_size.get();

        let (task_tx, task_rx) = crossbeam_channel::unbounded::<TileRenderTask>();
        let (update_tx, update_rx) = crossbeam_channel::unbounded::<TileRenderUpdates>();

        for block_index_y in 0..height_in_blocks {
            for block_index_x in 0..width_in_blocks {
                let size_x = self.block_size.get().min(camera.image_width() - block_index_x * self.block_size.get());
                let size_y = self.block_size.get().min(camera.image_height() - block_index_y * self.block_size.get());
                task_tx.send(TileRenderTask { 
                    block_index_x, 
                    block_index_y, 
                    size_x, 
                    size_y,
                    block_size: self.block_size.get()
                }).unwrap();
            }
        }

        drop(task_tx);

        // spawn threads

        let thread_handles: Vec<_> = (0..self.num_threads.get()).map(|thread_id| {
            let thread_task_rx = task_rx.clone();
            let thread_update_tx = update_tx.clone();
            let thread_camera = camera.clone();
            let thread_scene = scene.clone();

            std::thread::spawn(move || {
                loop {
                    let task = match thread_task_rx.recv() {
                        Ok(task) => task,
                        Err(_) => break,
                    };

                    thread_update_tx.send(TileRenderUpdates::Start(TileRenderStartMessage {
                        block_index_x: task.block_index_x,
                        block_index_y: task.block_index_y,
                    })).unwrap();

                    let result = task.render(&thread_camera, &thread_scene, max_bounces, samples_per_pixel, thread_id);

                    thread_update_tx.send(TileRenderUpdates::End(result)).unwrap();

                }
            })
        }).collect();

        drop(update_tx);

        // polling update phase

        let waiting_char = " ▪ ";
        let in_progress_char = "▒▒▒";
        let complete_char = "███";
        let indicator_size = waiting_char.chars().count();

        let mut results = vec![TileRenderResult::default(); width_in_blocks * height_in_blocks];
        
        let mut term = Term::stderr();

        writeln!(term, "Rendering {} by {} image in {} by {} blocks...", camera.image_width(), camera.image_height(), self.block_size, self.block_size).unwrap_or_default();
        writeln!(term, "Using {} threads", self.num_threads).unwrap_or_default();

        for _ in 0..height_in_blocks {
            term.write_line(&str::repeat(waiting_char, width_in_blocks)).unwrap_or_default();
        }

        writeln!(term, "").unwrap_or_default();
        term.move_cursor_left(100000).unwrap_or_default();
        term.flush().unwrap_or_default();

        loop {
            let update = match update_rx.recv() {
                Ok(update) => update,
                Err(_) => break,
            };

            let (update_x, update_y, update_char, message) =  match update {
                TileRenderUpdates::Start(start) => {
                    (start.block_index_x, start.block_index_y, in_progress_char, None)
                },
                TileRenderUpdates::End(end) => {
                    let ret_value = (end.block_index_x, end.block_index_y, complete_char, Some(format!("Thread {} complete block ({}, {}) at {:.2} px/s", end.thread_id, end.block_index_x, end.block_index_y, end.average_pixel_throughput)));
                    let result_index = end.block_index_x + end.block_index_y * width_in_blocks;
                    results[result_index] = end;
                    ret_value
                },
            };

            term.move_cursor_up(height_in_blocks - update_y + 1).unwrap_or_default();
            term.move_cursor_right(update_x * indicator_size).unwrap_or_default();

            write!(term, "{}", update_char).unwrap_or_default();

            term.move_cursor_left(10000000).unwrap_or_default();
            term.move_cursor_down(height_in_blocks - update_y).unwrap_or_default();

            match message {
                Some(s) => {
                    term.clear_line().unwrap_or_default();
                    writeln!(term, "{}", s).unwrap_or_default();
                },
                None => {
                    term.move_cursor_down(1).unwrap_or_default();
                },
            }

            term.flush().unwrap_or_default();
        }

        for handle in thread_handles {
            handle.join().unwrap();
        }

        writeln!(term, "Rendering Completed!").unwrap_or_default();
        term.flush().unwrap_or_default();

        RgbImage::from_fn(camera.image_width().try_into().unwrap(), camera.image_height().try_into().unwrap(), |col, row| {
            let block_index_x = (col as usize) / self.block_size;
            let block_index_y = (row as usize) / self.block_size;
            let intra_block_x = (col as usize) % self.block_size;
            let intra_block_y = (row as usize) % self.block_size;

            results[block_index_x + block_index_y * width_in_blocks].output[intra_block_x + intra_block_y * self.block_size.get()]
        })
    }
}