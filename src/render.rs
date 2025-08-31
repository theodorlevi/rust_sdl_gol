use crate::gol::Grid;
use crate::types::{RenderCtx, ViewState};
use raylib::drawing::RaylibDrawHandle;
use raylib::prelude::{Color, RayImGUITrait, RaylibDraw, RaylibDrawGui};
use std::cell::Cell;
use std::cell::RefCell;

pub fn draw_frame(render_ctx: &mut RenderCtx) {
    if let Some(font) = render_ctx.font.as_mut() {
        render_ctx.canvas.gui_set_font(font);
    }
    render_ctx.canvas.clear_background(Color::BLACK);
    
    draw_cells(&mut render_ctx.gol.grid, &mut render_ctx.canvas, render_ctx.viewstate);
    draw_selection(&mut render_ctx.canvas, &render_ctx.gol.grid, render_ctx.viewstate);

    let paused_cell = RefCell::new(render_ctx.gol.paused);
    let speed_cell = RefCell::new(render_ctx.speed as i32);
    let cells_alive = render_ctx.gol.grid.get_grid().len();
    let zoom_val = render_ctx.viewstate.zoom;

    let want_mouse_cell = Cell::new(false);
    render_ctx.canvas.draw_imgui(|ui| {
        ui.window("Controls").build(|| {
            ui.text("Game of Life");
            ui.separator();
            {
                let mut p = paused_cell.borrow_mut();
                ui.checkbox("Paused", &mut *p);
            }
            ui.text("Speed (ms per gen)");
            {
                let mut s = speed_cell.borrow_mut();
                ui.slider("##speed", 0, 100, &mut *s);
            }
            ui.separator();
            ui.text(format!("Cells alive: {}", cells_alive));
            ui.text(format!("Zoom: {:.2}", zoom_val));
            ui.separator();
            ui.text("Press F1 for help")
        });

        if render_ctx.show_help {
            ui.window("Help/About").build(|| {
                ui.text("This is a simple implementation of Conway's Game of Life.");
                ui.text("It is written in Rust using the Raylib library.");
                ui.text("The source code is available on GitHub.");
                ui.text("https://github.com/theodorlevi/rust-gol-raylib");
                ui.separator();
                ui.text("Controls:");
                ui.text("W,A,S,D to Move camera");
                ui.text("Left mouse to create a live cell");
                ui.text("Right mouse to kill a live cell");
                ui.text("Space to Pause/unpause");
                ui.text("Tab to step forward one generation");
                ui.text("R - Reset");
        });
        }
        want_mouse_cell.set(ui.io().want_capture_mouse);
    });
    render_ctx.gol.paused = paused_cell.into_inner();
    render_ctx.speed = speed_cell.into_inner().max(0) as usize;
    render_ctx.ui_wants_mouse = want_mouse_cell.get();
}

fn round_down_to_multiple(n: f32, step: f32) -> f32 {
    (n / step).floor() * step
}

fn draw_cells(grid: &mut Grid, canvas: &mut RaylibDrawHandle, viewstate: ViewState) {
    for cell in grid.get_grid() {
        if viewstate.zoom <= 1.0 {
            canvas
                .draw_pixel(
                    (cell.y as f32 * viewstate.zoom + viewstate.camera_pos.x) as i32,
                    (cell.x as f32 * viewstate.zoom + viewstate.camera_pos.y) as i32,
                    Color::WHITE,
                );
        } else {
            canvas
                .draw_rectangle(
                    (cell.y as f32 * viewstate.zoom + viewstate.camera_pos.x) as i32,
                    (cell.x as f32 * viewstate.zoom + viewstate.camera_pos.y) as i32,
                    viewstate.zoom as i32,
                    viewstate.zoom as i32,
                    Color::WHITE,
                );
        }
    }
}

fn draw_selection(canvas: &mut RaylibDrawHandle, grid: &Grid, viewstate: ViewState) {
    let (mouse_x, mouse_y, scale, cam_x, cam_y) = (
        viewstate.mouse_pos.x,
        viewstate.mouse_pos.y,
        viewstate.zoom,
        viewstate.camera_pos.x,
        viewstate.camera_pos.y,
    );

    let world_x = mouse_x - cam_x;
    let world_y = mouse_y - cam_y;

    let select_world_x = round_down_to_multiple(world_x, scale);
    let select_world_y = round_down_to_multiple(world_y, scale);

    let x = (select_world_y / scale) as isize;
    let y = (select_world_x / scale) as isize;

    let screen_x = select_world_x + cam_x;
    let screen_y = select_world_y + cam_y;

    canvas
        .draw_rectangle_lines(
            screen_x as i32,
            screen_y as i32,
            scale as i32,
            scale as i32,
            if grid.get_cell(x, y) { Color::RED } else { Color::GREEN },
        );
}
