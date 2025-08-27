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

static mut SCALE: f32 = 4.0;
const GRID_SIZE: usize = 1024;
static mut MOUSE_POS: (f32, f32) = (0.0, 0.0);

static mut CAMERA_POS: (f32, f32) = (0.0, 0.0);

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
        .window("rust-sdl3 demo", 800, 600)
        .position_centered()
        .resizable()
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
    let mut mouse3_state = false;

    let mut drag_start: (f32, f32) = (0.0, 0.0);
    let mut camera_start: (f32, f32) = (0.0, 0.0);

    let mut wasd_state = (false, false, false, false);

    'running: loop {
        let start_time = Instant::now();

        main_draw(&mut canvas, &mut gol, frame_time, &font);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,

                Event::MouseButtonDown { mouse_btn , .. } => {
                    if mouse_btn == MouseButton::Left {
                        mouse1_state = true;
                    } else if mouse_btn == MouseButton::Right {
                        mouse2_state = true;
                    } else if mouse_btn == MouseButton::Middle {
                        mouse3_state = true;
                        drag_start = unsafe {MOUSE_POS};
                        camera_start = unsafe {CAMERA_POS};
                    }
                }

                Event::MouseButtonUp { mouse_btn , ..} => {
                    if mouse_btn == MouseButton::Left {
                        mouse1_state = false;
                    } else if mouse_btn == MouseButton::Right {
                        mouse2_state = false;
                    } else if mouse_btn == MouseButton::Middle {
                        mouse3_state = false;
                    }
                }

                Event::KeyDown { keycode: Some(Keycode::R), ..} => {
                    gol.grid.clear_all()
                }
                Event::KeyDown { keycode: Some(Keycode::Space), ..} => {
                    gol.pause();
                }
                Event::KeyDown { keycode: Some(Keycode::P), ..} => {
                    gol.paused = false;
                    gol.update();
                    gol.paused = true;
                }

                // wasd controls ////////////////////////////////////
                Event::KeyDown { keycode: Some(Keycode::W), ..} => {
                    wasd_state.0 = true;
                }
                Event::KeyUp { keycode: Some(Keycode::W), ..} => {
                    wasd_state.0 = false;
                }
                Event::KeyDown { keycode: Some(Keycode::S), ..} => {
                    wasd_state.1 = true;
                }
                Event::KeyUp { keycode: Some(Keycode::S), ..} => {
                    wasd_state.1 = false;
                }
                Event::KeyDown { keycode: Some(Keycode::A), ..} => {
                    wasd_state.2 = true;
                }
                Event::KeyUp { keycode: Some(Keycode::A), ..} => {
                    wasd_state.2 = false;
                }
                Event::KeyDown { keycode: Some(Keycode::D), ..} => {
                    wasd_state.3 = true;
                }
                Event::KeyUp { keycode: Some(Keycode::D), ..} => {
                    wasd_state.3 = false;
                }
                /////////////////////////////////////////////////////

                Event::MouseMotion { x, y, .. } => unsafe {
                    MOUSE_POS = (x, y);
                }
                Event::MouseWheel { y, .. } => unsafe {
                    let old_scale = SCALE;
                    let mut new_scale = old_scale + y;
                    if new_scale < 1.0 { new_scale = 1.0; }
                    if (new_scale - old_scale).abs() < f32::EPSILON { continue }

                    let k = new_scale / old_scale;
                    CAMERA_POS.1 = k * CAMERA_POS.1 + (1.0 - k) * MOUSE_POS.0;
                    CAMERA_POS.0 = k * CAMERA_POS.0 + (1.0 - k) * MOUSE_POS.1;

                    SCALE = new_scale;
                }
                _ => {}
            }
        }

        gol.update();

        if mouse1_state {
            unsafe {
                let rel_x = MOUSE_POS.0 - CAMERA_POS.1;
                let rel_y = MOUSE_POS.1 - CAMERA_POS.0;
                if rel_x >= 0.0 && rel_y >= 0.0 {
                    let col = (rel_x / SCALE as f32).floor() as isize;
                    let row = (rel_y / SCALE as f32).floor() as isize;
                    if row >= 0 && col >= 0 {
                        gol.grid.set_cell(row as usize, col as usize, true);
                    }
                }
            }
        } else if mouse2_state {
            unsafe {
                let rel_x = MOUSE_POS.0 - CAMERA_POS.1;
                let rel_y = MOUSE_POS.1 - CAMERA_POS.0;
                if rel_x >= 0.0 && rel_y >= 0.0 {
                    let col = (rel_x / SCALE as f32).floor() as isize;
                    let row = (rel_y / SCALE as f32).floor() as isize;
                    if row >= 0 && col >= 0 {
                        gol.grid.set_cell(row as usize, col as usize, false);
                    }
                }
            }
        }

        if mouse3_state {
            unsafe {
                let mouse_delta = (MOUSE_POS.0 - drag_start.0, drag_start.1 - MOUSE_POS.1);

                CAMERA_POS = (camera_start.0 - mouse_delta.1, camera_start.1 - -mouse_delta.0);
            }
        }

        if wasd_state.0 {
            unsafe {CAMERA_POS.0 += frame_time.as_millis().max(4) as f32;};
        }
        if wasd_state.1 {
            unsafe {CAMERA_POS.0 -= frame_time.as_millis().max(4) as f32;};
        }
        if wasd_state.2 {
            unsafe { CAMERA_POS.1 += frame_time.as_millis().max(4) as f32; };
        }
        if wasd_state.3 {
            unsafe { CAMERA_POS.1 -= frame_time.as_millis().max(4) as f32; };
        }

        let last_time = Instant::now();
        frame_time = last_time - start_time;
        canvas.present();

        if gol.paused {
            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 144));
        }
    }
}
