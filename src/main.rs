mod gol;
mod render;

use std::default::Default;
use std::path::Path;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::{ttf, Error};
use std::time::{Duration, Instant};
use log::{info, warn};
use sdl3::mouse::MouseButton;
use sdl3::ttf::{Font, Sdl3TtfContext};
use crate::gol::*;
use crate::render::main_draw;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
struct Vector2 {
    x: f32,
    y: f32,
}

impl Vector2 {
    fn new(x: f32, y: f32) -> Self {
        Vector2 { x, y }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct ViewState {
    camera_pos: Vector2,
    mouse_pos: Vector2,
    zoom: f32,
}

impl Default for ViewState {
    fn default() -> Self {
        ViewState {
            camera_pos: Default::default(),
            mouse_pos: Default::default(),
            zoom: 4.0,
        }
    }
}

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

    let mut texture_creator = canvas.texture_creator();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    info!("initialized event pump");

    let grid = Grid::new();
    let mut gol = GOL::new(grid);
    info!("initialized gol");

    let mut viewstate: ViewState = Default::default();

    let mut frame_time: Duration = Duration::from_millis(0);
    let mut mouse1_state = false;
    let mut mouse2_state = false;
    let mut mouse3_state = false;

    let mut drag_start: Vector2 = Vector2::new(0.0, 0.0);
    let mut camera_start: Vector2 = Vector2::new(0.0, 0.0);

    let mut wasd_state = (false, false, false, false);

    'running: loop {
        let start_time = Instant::now();

        main_draw(&mut canvas, &mut gol, frame_time, &font, &mut viewstate, &mut texture_creator);

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
                        drag_start = viewstate.mouse_pos;
                        camera_start = viewstate.camera_pos;
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
                Event::KeyDown { keycode: Some(Keycode::Tab), ..} => {
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

                Event::MouseMotion { x, y, .. } => {
                    viewstate.mouse_pos = Vector2::new(x, y);
                }
                Event::MouseWheel { y, .. } => {
                    let old_scale = viewstate.zoom;
                    let mut new_scale = old_scale + y;
                    if new_scale < 1.0 { new_scale = 1.0; }
                    if (new_scale - old_scale).abs() < f32::EPSILON { continue }

                    let k = new_scale / old_scale;
                    viewstate.camera_pos.x = k * viewstate.camera_pos.x + (1.0 - k) * viewstate.mouse_pos.x;
                    viewstate.camera_pos.y = k * viewstate.camera_pos.y + (1.0 - k) * viewstate.mouse_pos.y;

                    viewstate.zoom = new_scale;
                }
                _ => {}
            }
        }

        gol.update();

        if mouse1_state {

            let rel_x = viewstate.mouse_pos.x - viewstate.camera_pos.x;
            let rel_y = viewstate.mouse_pos.y - viewstate.camera_pos.y;
            let col = (rel_x / viewstate.zoom).floor() as isize;
            let row = (rel_y / viewstate.zoom).floor() as isize;
            gol.grid.set_cell(row, col, true);

        } else if mouse2_state {
            let rel_x = viewstate.mouse_pos.x - viewstate.camera_pos.x;
            let rel_y = viewstate.mouse_pos.y - viewstate.camera_pos.y;
            let col = (rel_x / viewstate.zoom).floor() as isize;
            let row = (rel_y / viewstate.zoom).floor() as isize;
            gol.grid.set_cell(row, col, false);
        }

        if mouse3_state {
            let mouse_delta = Vector2::new(
                viewstate.mouse_pos.x - drag_start.x,
                drag_start.y - viewstate.mouse_pos.y
            );

            viewstate.camera_pos = Vector2::new(camera_start.x - -mouse_delta.x, camera_start.y - mouse_delta.y);
        }

        if wasd_state.0 {
            viewstate.camera_pos.y += frame_time.as_millis().max(4) as f32;
        }
        if wasd_state.1 {
            viewstate.camera_pos.y -= frame_time.as_millis().max(4) as f32;
        }
        if wasd_state.2 {
            viewstate.camera_pos.x += frame_time.as_millis().max(4) as f32;
        }
        if wasd_state.3 {
            viewstate.camera_pos.x -= frame_time.as_millis().max(4) as f32;
        }

        let last_time = Instant::now();
        frame_time = last_time - start_time;
        canvas.present();


        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 144));

    }
}
