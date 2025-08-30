mod gol;
mod render;
mod types;

use crate::gol::*;
use crate::render::draw_frame;
use log::{info, warn};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;
use sdl3::pixels::Color;
use sdl3::ttf::{Font, Sdl3TtfContext};
use sdl3::{ttf, Error};
use std::default::Default;
use std::path::Path;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use types::{Vector2, ViewState};
use crate::types::{RenderCtx, UpdateResult};

fn handle_font_error<'font>(e: Error, font_context: Sdl3TtfContext) -> Font<'font> {
    warn!("Couldn't load font: {}", e);
    if Path::exists("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf".as_ref()) {
        return font_context
            .load_font("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 32f32)
            .unwrap();
    }
    panic!("Couldn't load font");
}

fn main() {
    info!("rayon using: {} threads", rayon::current_num_threads());

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
    let font = ttf_context
        .load_font("/usr/share/fonts/TTF/JetBrainsMono-Regular.ttf", 32.0)
        .unwrap_or_else(|e| handle_font_error(e, ttf_context));
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

    let mut speed = 14usize;

    let (next_grid_request_tx, next_grid_request_rx) = mpsc::channel::<Grid>();
    let (next_grid_result_tx, next_grid_result_rx) = mpsc::channel::<UpdateResult>();

    thread::spawn(move || {
        while let Ok(current_grid_snapshot) = next_grid_request_rx.recv() {
            let start = Instant::now();
            let next = GOL::update_from(&current_grid_snapshot);
            let compute_time = Instant::now() - start;
            let result = UpdateResult { next_grid: next, compute_time };
            if next_grid_result_tx.send(result).is_err() {
                break;
            }
        }
    });
    
    let mut update_in_progress = false;

    'running: loop {
        let start_time = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::MouseButtonDown { mouse_btn, .. } => {
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

                Event::MouseButtonUp { mouse_btn, .. } => {
                    if mouse_btn == MouseButton::Left {
                        mouse1_state = false;
                    } else if mouse_btn == MouseButton::Right {
                        mouse2_state = false;
                    } else if mouse_btn == MouseButton::Middle {
                        mouse3_state = false;
                    }
                }

                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => gol.grid.clear_all(),
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    gol.pause();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Tab),
                    ..
                } => {
                    gol.paused = false;
                    gol.grid = GOL::update_from(&gol.grid.clone());
                    gol.paused = true;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    wasd_state.0 = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    wasd_state.0 = false;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    wasd_state.1 = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    wasd_state.1 = false;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    wasd_state.2 = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    wasd_state.2 = false;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    wasd_state.3 = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    wasd_state.3 = false;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    speed += 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    let newspeed = speed as isize - 1;

                    if newspeed < 0 {
                        continue;
                    }

                    speed = newspeed as usize;
                }

                Event::MouseMotion { x, y, .. } => {
                    viewstate.mouse_pos = Vector2::new(x, y);
                }
                Event::MouseWheel { y, .. } => {
                    let old_scale = viewstate.zoom;

                    let step: f32 = 1.1;
                    let factor = step.powf(y);
                    if !factor.is_finite() {
                        continue;
                    }

                    let mut new_scale = old_scale * factor;

                    if new_scale < 0.0001 {
                        new_scale = 0.0001;
                    }
                    if new_scale > 10000.0 {
                        new_scale = 10000.0;
                    }

                    if (new_scale / old_scale - 1.0).abs() < 1e-6 {
                        continue;
                    }

                    let k = new_scale / old_scale;
                    viewstate.camera_pos.x =
                        k * viewstate.camera_pos.x + (1.0 - k) * viewstate.mouse_pos.x;
                    viewstate.camera_pos.y =
                        k * viewstate.camera_pos.y + (1.0 - k) * viewstate.mouse_pos.y;

                    viewstate.zoom = new_scale;
                }
                _ => {}
            }
        }
        
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
                drag_start.y - viewstate.mouse_pos.y,
            );
            viewstate.camera_pos = Vector2::new(
                camera_start.x - -mouse_delta.x,
                camera_start.y - mouse_delta.y,
            );
        }
        let wasd_speed =
            1.0 * frame_time.as_micros() as f32 / (1000.0 - if gol.paused { 500.0 } else { 0.0 });

        if wasd_state.0 {
            viewstate.camera_pos.y += wasd_speed;
        }
        if wasd_state.1 {
            viewstate.camera_pos.y -= wasd_speed;
        }
        if wasd_state.2 {
            viewstate.camera_pos.x += wasd_speed;
        }
        if wasd_state.3 {
            viewstate.camera_pos.x -= wasd_speed;
        }

        if !gol.paused && !update_in_progress {
            let current_grid_snapshot = gol.grid.clone();
            if next_grid_request_tx.send(current_grid_snapshot).is_ok() {
                update_in_progress = true;
            }
        }

        match next_grid_result_rx.try_recv() {
            Ok(update) => {
                thread::sleep(Duration::from_millis(speed as u64 / update.compute_time.as_millis().max(1) as u64));
                gol.grid = update.next_grid;
                update_in_progress = false;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                update_in_progress = false;
            }
        }

        {
            let mut render_ctx = RenderCtx {
                gol: gol.clone(),
                frame_time,
                viewstate,
                speed,
                canvas: &mut canvas,
                texture_creator: &mut texture_creator,
                font: &font,
            };
            draw_frame(&mut render_ctx);
        }

        let last_time = Instant::now();
        frame_time = last_time - start_time;
    }
}
