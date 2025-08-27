use std::time::Duration;
use sdl3::pixels::Color;
use sdl3::render::{Canvas, FPoint, FRect};
use sdl3::ttf::Font;
use sdl3::video::Window;
use crate::{GRID_SIZE, MOUSE_POS, SCALE, CAMERA_POS};
use crate::gol::{Grid, GOL};

pub fn main_draw(canvas: &mut Canvas<Window>, gol: &mut GOL, frame_time: Duration, font: &Font) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();


    canvas.set_draw_color(Color::RGB(32, 32, 32));
    draw_grid(canvas, &gol);


    draw_cells(&mut gol.grid, canvas);

    draw_selection(canvas, &gol.grid);

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
        format!("{}x zoom", unsafe {SCALE}).as_str(),
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

fn draw_cells(grid: &mut Grid, canvas: &mut Canvas<Window>) {
    let mut i = 0;
    let mut j = 0;

    j = 0;
    for row in grid.get_grid() {
        i = 0;
        for col in row {
            match col {
                false => {
                    i += 1;
                    continue;
                }
                true => unsafe {
                    canvas.set_draw_color(Color::RGB(255, 255, 255));
                    canvas.fill_rect(FRect {
                        x: i as f32 * SCALE + CAMERA_POS.1,
                        y: j as f32 * SCALE + CAMERA_POS.0,
                        w: SCALE,
                        h: SCALE,
                    }).unwrap();
                }
            }
            i += 1;
        }
        j += 1;
    }
}

fn draw_selection(canvas: &mut Canvas<Window>, grid: &Grid) {
    // Convert mouse position from screen space to world space by removing camera offset
    let (mouse_x, mouse_y, scale, cam_y, cam_x) = unsafe { (MOUSE_POS.0, MOUSE_POS.1, SCALE, CAMERA_POS.1, CAMERA_POS.0) };

    let world_x = mouse_x - cam_y;
    let world_y = mouse_y - cam_x;

    // If mouse is outside the world (negative), skip drawing selection
    if world_x < 0.0 || world_y < 0.0 {
        return;
    }

    // Snap to grid in world space
    let select_world_x = round_down_to_multiple(world_x, scale);
    let select_world_y = round_down_to_multiple(world_y, scale);

    // Compute grid coordinates from world space
    let row = (select_world_y / scale) as usize;
    let col = (select_world_x / scale) as usize;

    // Choose color based on cell state
    if grid.get_cell(row, col) {
        canvas.set_draw_color(Color::RGB(255, 0, 0));
    } else {
        canvas.set_draw_color(Color::RGB(0, 255, 0));
    }

    // Convert back to screen space for rendering by adding camera offset
    let screen_x = select_world_x + cam_y;
    let screen_y = select_world_y + cam_x;

    canvas.draw_rect(FRect {
        x: screen_x,
        y: screen_y,
        w: scale,
        h: scale,
    }).unwrap();
}

fn draw_grid(canvas: &mut Canvas<Window>, gol: &GOL) {
    let mut grid_spacing = 1;

    if unsafe {SCALE} < 2.0 {
        grid_spacing = 8;
    } else if unsafe {SCALE} < 4.0 {
        grid_spacing = 5;
    } else if unsafe {SCALE < 6.0} {
        grid_spacing = 4;
    } else if unsafe {SCALE < 8.0} {
        grid_spacing = 3;
    } else if unsafe {SCALE < 10.0} {
        grid_spacing = 2;
    }

    for row_index in 0..gol.grid.grid.len() {
        if row_index % grid_spacing == 0 {
            canvas.draw_line(
                FPoint {
                    x: unsafe {CAMERA_POS.1},
                    y: row_index as f32 * unsafe {SCALE} + unsafe {CAMERA_POS.0},
                },
                FPoint {
                    x: (unsafe {SCALE} * GRID_SIZE as f32) + unsafe {CAMERA_POS.1},
                    y: row_index as f32 * unsafe {SCALE} + unsafe {CAMERA_POS.0},
                }
            ).unwrap();
        }
    }

    for col_index in 0..GRID_SIZE {
        if col_index % grid_spacing == 0 {
            canvas.draw_line(
                FPoint {
                    x: col_index as f32 * unsafe {SCALE} + unsafe {CAMERA_POS.1},
                    y: unsafe {CAMERA_POS.0},
                },
                FPoint {
                    x: col_index as f32 * unsafe {SCALE} + unsafe {CAMERA_POS.1},
                    y: (unsafe {SCALE} * gol.grid.grid.len() as f32) + unsafe {CAMERA_POS.0},
                }
            ).unwrap();
        }
    }
}