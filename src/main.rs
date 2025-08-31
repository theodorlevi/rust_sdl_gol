mod gol;
mod render;
mod types;

use crate::gol::*;
use crate::render::draw_frame;
use log::{info};
use std::default::Default;
use std::path::Path;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::{Instant};
use raylib::prelude::{MouseButton};
use raylib::prelude::KeyboardKey::KEY_F1;
use types::{Vector2, ViewState};
use crate::types::{RenderCtx, UpdateResult};

fn probe_fonts() -> String {
    let candidates: Vec<&str> = vec![
        "/usr/share/fonts/TTF/JetBrainsMono-Medium.ttf",
        "/usr/share/fonts/truetype/jetbrains-mono/JetBrainsMono-Regular.ttf",
        "/usr/share/fonts/truetype/cantarell/Cantarell-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ];

    #[cfg(target_os = "macos")]
    {
        candidates.push("/System/Library/Fonts/Helvetica.ttc");
        candidates.push("/System/Library/Fonts/Supplemental/Arial.ttf");
    }
    #[cfg(target_os = "windows")]
    {
        candidates.push("C:\\Windows\\Fonts\\Arial.ttf");
        candidates.push("C:\\Windows\\Fonts\\consola.ttf");
    }

    for p in candidates {
        if Path::new(p).exists() {
            return p.to_string();
        }
    }

    String::new()
}

fn main() {
    info!("rayon using: {} threads", rayon::current_num_threads());

    let (mut rl, thread) = raylib::init()
        .size(1280, 720)
        .msaa_4x()
        .resizable()
        .vsync()
        .build();
    info!("initialized window");

    let mut loaded_font = {
        let path = probe_fonts();
        match rl.load_font(&thread, path.as_str()) {
            Ok(f) => Some(f),
            Err(_) => None,
        }
    };


    let grid = Grid::new();
    let mut gol = GOL::new(grid);
    info!("initialized gol");

    let mut viewstate: ViewState = Default::default();

    let mut speed = 14usize;

    let (next_grid_request_tx, next_grid_request_rx) = mpsc::channel::<(Grid, usize)>();
    let (next_grid_result_tx, next_grid_result_rx) = mpsc::channel::<UpdateResult>();

    thread::spawn(move || {
        while let Ok((current_grid_snapshot, provided_speed)) = next_grid_request_rx.recv() {
            let start = Instant::now();
            let next = GOL::update_from(&current_grid_snapshot, provided_speed as u64);
            let compute_time = Instant::now() - start;
            let result = UpdateResult { next_grid: next, compute_time };
            if next_grid_result_tx.send(result).is_err() {
                break;
            }
        }
    });

    let mut update_in_progress = false;
    let mut drag_start = Vector2::new(0.0, 0.0);
    let mut camera_start = Vector2::new(0.0, 0.0);

    let mut show_help = false;

    let mut ui_wants_mouse = false;
    while !rl.window_should_close() {
        viewstate.mouse_pos = Vector2::new(rl.get_mouse_x() as f32, rl.get_mouse_y() as f32);

        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_SPACE) {
            gol.pause();
        }

        if rl.is_key_pressed_repeat(
            raylib::consts::KeyboardKey::KEY_TAB) ||
            rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_TAB
            ) {
            gol.paused = false;
            gol.grid = GOL::update_from(&gol.grid, 0);
            gol.paused = true;
        }

        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_UP) {
            speed += 1;
        }

        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_DOWN) {
            speed -= 1;
        }

        if !ui_wants_mouse {
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                let rel_x = viewstate.mouse_pos.x - viewstate.camera_pos.x;
                let rel_y = viewstate.mouse_pos.y - viewstate.camera_pos.y;
                let col = (rel_x / viewstate.zoom).floor() as isize;
                let row = (rel_y / viewstate.zoom).floor() as isize;
                gol.grid.set_cell(row, col, true);
            } else if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
                let rel_x = viewstate.mouse_pos.x - viewstate.camera_pos.x;
                let rel_y = viewstate.mouse_pos.y - viewstate.camera_pos.y;
                let col = (rel_x / viewstate.zoom).floor() as isize;
                let row = (rel_y / viewstate.zoom).floor() as isize;
                gol.grid.set_cell(row, col, false);
            }
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_MIDDLE) {
                let drag_start = viewstate.mouse_pos;
                let camera_start = viewstate.camera_pos;

                let mouse_delta = Vector2::new(
                    viewstate.mouse_pos.x - drag_start.x,
                    drag_start.y - viewstate.mouse_pos.y,
                );
                viewstate.camera_pos = Vector2::new(
                    camera_start.x - -mouse_delta.x,
                    camera_start.y - mouse_delta.y,
                );
            }

            if rl.get_mouse_wheel_move() != 0.0 {
                let y = rl.get_mouse_wheel_move();
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

                if new_scale > 1.0 {
                    if y.is_sign_positive() {
                        new_scale = old_scale + (y - 0.5);
                    } else {
                        new_scale = old_scale + (y + 0.5);
                    }
                } else {
                    if (new_scale / old_scale - 1.0).abs() < 1e-6 {
                        continue;
                    }
                }

                let k = new_scale / old_scale;
                viewstate.camera_pos.x =
                    k * viewstate.camera_pos.x + (1.0 - k) * viewstate.mouse_pos.x;
                viewstate.camera_pos.y =
                    k * viewstate.camera_pos.y + (1.0 - k) * viewstate.mouse_pos.y;

                viewstate.zoom = new_scale;
            }
        }

        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_W) {
            viewstate.camera_pos.y += 4.0;
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_S) {
            viewstate.camera_pos.y -= 4.0;
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_A) {
            viewstate.camera_pos.x += 4.0;
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_D) {
            viewstate.camera_pos.x -= 4.0;
        }

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_MIDDLE) {
            drag_start = viewstate.mouse_pos;
            camera_start = viewstate.camera_pos;
        }

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_MIDDLE) {
            let mouse_delta = Vector2::new(
                viewstate.mouse_pos.x - drag_start.x,
                drag_start.y - viewstate.mouse_pos.y,
            );
            viewstate.camera_pos = Vector2::new(
                camera_start.x - -mouse_delta.x,
                camera_start.y - mouse_delta.y,
            );
        }

        if rl.is_key_pressed(KEY_F1) {
            show_help = !show_help;
        }

        if !gol.paused && !update_in_progress {
            let current_grid_snapshot = gol.grid.clone();
            if next_grid_request_tx.send((current_grid_snapshot, speed)).is_ok() {
                update_in_progress = true;
            }
        }

        match next_grid_result_rx.try_recv() {
            Ok(update) => {
                gol.grid = update.next_grid;
                update_in_progress = false;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                update_in_progress = false;
            }
        }

        println!("{:?}", viewstate.camera_pos);

        {
            let canvas = rl.begin_drawing(&thread);
            let mut render_ctx = RenderCtx {
                gol: gol.clone(),
                viewstate,
                speed,
                canvas,
                font: loaded_font.take(),
                show_help,
                ui_wants_mouse,
            };
            draw_frame(&mut render_ctx);
            gol.paused = render_ctx.gol.paused;
            speed = render_ctx.speed;
            loaded_font = render_ctx.font.take();
            ui_wants_mouse = render_ctx.ui_wants_mouse;
        }
    }
}
