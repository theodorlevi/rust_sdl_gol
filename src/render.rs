use crate::gol::{Grid, GOL};
use crate::ViewState;
use sdl3::pixels::Color;
use sdl3::render::{Canvas, FPoint, FRect};
use sdl3::ttf::Font;
use sdl3::video::Window;
use std::time::Duration;

pub fn main_draw(canvas: &mut Canvas<Window>, gol: &mut GOL, frame_time: Duration, font: &Font, viewstate: &mut ViewState) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    if viewstate.zoom <= 10.0 {
        canvas.set_draw_color(Color::RGB(32, 32, 32));
        draw_grid(canvas, &gol, viewstate);
    }

    draw_cells(&mut gol.grid, canvas, viewstate);

    if viewstate.zoom > 10.0 {
        canvas.set_draw_color(Color::RGB(32, 32, 32));
        draw_grid(canvas, &gol, viewstate);
    }

    draw_selection(canvas, &gol.grid, viewstate);

    draw_text(
        &font,
        canvas,
        frame_time.as_millis().to_string().as_str(),
        24.0,
        Color::RGB(255, 255, 255),
        10.0,
        16.0
    );

    draw_text(
        &font,
        canvas,
        format!("{}x zoom", viewstate.zoom).as_str(),
        24.0,
        Color::RGB(255, 255, 255),
        10.0,
        48.0
    );

    if gol.paused {
        draw_text(
            &font,
            canvas,
            "PAUSED",
            24.0,
            Color::RGB(255, 0, 0),
            10.0,
            32.0
        )
    }
}

fn round_down_to_multiple(n: f32, step: f32) -> f32 {
    (n / step).floor() * step
}

fn draw_text(
    font: &Font,
    canvas: &mut Canvas<Window>,
    text: &str,
    font_size: f32,
    color: Color,
    x: f32,
    y: f32, ) {
    let text_texture = canvas.create_texture_from_surface(
        font.render(
            format!("{}", text)
                .as_str())
            .blended(color)
            .unwrap()).unwrap();

    canvas.copy(
        &text_texture,
        None,
        FRect {
            x,
            y,
            w: (font_size / 2.0) * text.len() as f32,
            h: font_size
        }
    ).unwrap();
}

fn draw_cells(grid: &mut Grid, canvas: &mut Canvas<Window>, viewstate: &mut ViewState) {
    let mut i = 0;
    let mut j = 0;

    for row in grid.get_grid() {
        for col in row {
            match col {
                false => {
                    i += 1;
                    continue;
                }
                true => {
                    canvas.set_draw_color(Color::RGB(255, 255, 255));
                    canvas.fill_rect(FRect {
                        x: i as f32 * viewstate.zoom + viewstate.camera_pos.x,
                        y: j as f32 * viewstate.zoom + viewstate.camera_pos.y,
                        w: viewstate.zoom,
                        h: viewstate.zoom,
                    }).unwrap();
                }
            }
            i += 1;
        }
        i = 0;
        j += 1;
    }
}

fn draw_selection(canvas: &mut Canvas<Window>, grid: &Grid, viewstate: &mut ViewState) {
    let (
        mouse_x,
        mouse_y,
        scale,
        cam_x,
        cam_y) = (
        viewstate.mouse_pos.x,
        viewstate.mouse_pos.y,
        viewstate.zoom,
        viewstate.camera_pos.x,
        viewstate.camera_pos.y
    );

    let world_x = mouse_x - cam_x;
    let world_y = mouse_y - cam_y;

    if world_x < 0.0 || world_y < 0.0 {
        return;
    }

    let select_world_x = round_down_to_multiple(world_x, scale);
    let select_world_y = round_down_to_multiple(world_y, scale);

    let row = (select_world_y / scale) as usize;
    let col = (select_world_x / scale) as usize;

    if grid.get_cell(row, col) {
        canvas.set_draw_color(Color::RGB(255, 0, 0));
    } else {
        canvas.set_draw_color(Color::RGB(0, 255, 0));
    }

    let screen_x = select_world_x + cam_x;
    let screen_y = select_world_y + cam_y;

    canvas.draw_rect(FRect {
        x: screen_x,
        y: screen_y,
        w: scale,
        h: scale,
    }).unwrap();
}

fn draw_grid(canvas: &mut Canvas<Window>, gol: &GOL, viewstate: &mut ViewState) {
    let mut grid_spacing = 1;

    if  viewstate.zoom < 2.0 {
        grid_spacing = 8;
    } else if  viewstate.zoom < 4.0 {
        grid_spacing = 5;
    } else if viewstate.zoom < 6.0 {
        grid_spacing = 4;
    } else if viewstate.zoom < 8.0 {
        grid_spacing = 3;
    } else if viewstate.zoom < 10.0 {
        grid_spacing = 2;
    }

    for row_index in 0..gol.grid.grid.len() {
        if row_index % grid_spacing == 0 {
            canvas.draw_line(
                FPoint {
                    x: viewstate.camera_pos.x,
                    y: row_index as f32 * viewstate.zoom + viewstate.camera_pos.y,
                },
                FPoint {
                    x: (viewstate.zoom * gol.grid.grid_size as f32) + viewstate.camera_pos.x,
                    y: row_index as f32 * viewstate.zoom + viewstate.camera_pos.y,
                }
            ).unwrap();
        }
    }

    for col_index in 0..gol.grid.grid_size {
        if col_index % grid_spacing == 0 {
            canvas.draw_line(
                FPoint {
                    x: col_index as f32 * viewstate.zoom + viewstate.camera_pos.x,
                    y: viewstate.camera_pos.y,
                },
                FPoint {
                    x: col_index as f32 * viewstate.zoom + viewstate.camera_pos.x,
                    y: (viewstate.zoom * gol.grid.grid.len() as f32) + viewstate.camera_pos.y,
                }
            ).unwrap();
        }
    }
}