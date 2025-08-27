mod gol;
mod render;

use std::path::Path;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::render::{FPoint, FRect};
use sdl3::{ttf, Error};
use std::time::{Duration, Instant};
use log::{debug, info, warn};
use sdl3::mouse::MouseButton;
use sdl3::ttf::{Font, Sdl3TtfContext};
use crate::gol::*;
use crate::render::main_draw;

const SCALE: u32 = 8;
const GRID_SIZE: usize = 128;
static mut MOUSE_POS: (f32, f32) = (0.0, 0.0);

fn handle_font_error<'font>(e: Error, font_context: Sdl3TtfContext) -> Font<'font>  {
    warn!("Couldn't load font: {}", e);
    if Path::exists("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf".as_ref()) {
        return font_context.load_font("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 32f32).unwrap()
    }
    panic!("Couldn't load font");
}

pub fn main() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    info!("initialized SDL3");

    let window = video_subsystem
        .window("rust-sdl3 demo", GRID_SIZE as u32*SCALE, GRID_SIZE as u32*SCALE)
        .position_centered()
        .build()
        .unwrap();
    info!("initialized window");

    let mut canvas = window.into_canvas();
    info!("initialized canvas");

    let ttf_context = ttf::init().unwrap();
    let font = ttf_context.load_font(
        "/usr/share/fonts/TTF/JetBrainsMono-Regular.ttf",
        32.0).unwrap_or_else(
        |e|
            handle_font_error(e, ttf_context)
    );
    info!("initialized font");

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    info!("initialized event pump");

    let grid = Grid::new();
    let mut gol = GOL::new(grid);
    info!("initialized gol");

    let mut frame_time: Duration = Duration::from_millis(0);
    let mut mouse1_state = false;
    let mut mouse2_state = false;

    'running: loop {
        let start_time = Instant::now();

        main_draw(&mut canvas, gol, frame_time, &font);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,

                Event::MouseButtonDown { mouse_btn , .. } => {
                    if mouse_btn == MouseButton::Left {
                        mouse1_state = true;
                    } else if mouse_btn == MouseButton::Right {
                        mouse2_state = true;
                    }
                }

                Event::MouseButtonUp { mouse_btn , ..} => {
                    if mouse_btn == MouseButton::Left {
                        mouse1_state = false;
                    } else if mouse_btn == MouseButton::Right {
                        mouse2_state = false;
                    }
                }

                Event::KeyDown { keycode: Some(Keycode::R), ..} => {
                    gol.grid.clear_all()
                }
                Event::KeyDown { keycode: Some(Keycode::Space), ..} => {
                    gol.pause();
                }
                Event::KeyDown { keycode: Some(Keycode::W), ..} => {
                    gol.paused = false;
                    gol.update();
                    gol.paused = true;
                }
                Event::MouseMotion { x, y, .. } => unsafe {
                    MOUSE_POS = (x, y);
                }
                _ => {}
            }
        }

        gol.update();

        if mouse1_state {
            unsafe {
                gol.grid.set_cell(
                    MOUSE_POS.1 as usize / SCALE as usize,
                    MOUSE_POS.0 as usize / SCALE as usize,
                    true,
                );
            }
        } else if mouse2_state {
            unsafe {
                gol.grid.set_cell(
                    MOUSE_POS.1 as usize / SCALE as usize,
                    MOUSE_POS.0 as usize / SCALE as usize,
                    false,
                );
            }
        }

        let last_time = Instant::now();
        frame_time = last_time - start_time;
        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 144));
    }
}
