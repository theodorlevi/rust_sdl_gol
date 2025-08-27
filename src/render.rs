use std::time::Duration;
use sdl3::pixels::Color;
use sdl3::render::{Canvas, FPoint, FRect};
use sdl3::ttf::Font;
use sdl3::video::Window;
use crate::{GRID_SIZE, MOUSE_POS, SCALE};
use crate::gol::{Grid, GOL};

pub fn main_draw(canvas: &mut Canvas<Window>, gol: GOL, frame_time: Duration, font: &Font) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    draw_grid(canvas, gol);

    draw_cells(gol.grid, canvas);

    draw_selection(canvas, &gol.grid);

    draw_text(
        &font,
        canvas,
        frame_time.as_millis().to_string().as_str(),
        12.0,
        Color::RGB(255, 255, 255),
        10.0,
        12.0
    );

    if gol.paused {
        draw_text(
            &font,
            canvas,
            "PAUSED",
            12.0,
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
    y: f32,
) {
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
            w: font_size * text.len() as f32,
            h: font_size * 2.0
        }
    ).unwrap();
}

fn draw_cells(grid: Grid, canvas: &mut Canvas<Window>) {
    let mut i = 0;
    let mut j = 0;

    j = 0;
    for row in grid.grid {
        i = 0;
        for col in row {
            match col {
                false => {
                    i += 1;
                    continue;
                }
                true => {
                    canvas.set_draw_color(Color::RGB(255, 255, 255));
                    canvas.fill_rect(FRect {
                        x: i as f32 * SCALE as f32,
                        y: j as f32 * SCALE as f32,
                        w: SCALE as f32,
                        h: SCALE as f32,
                    }).unwrap();
                }
            }
            i += 1;
        }
        j += 1;
    }
}

fn draw_selection(canvas: &mut sdl3::render::Canvas<sdl3::video::Window>, grid: &Grid) {
    let select_x = unsafe { round_down_to_multiple(MOUSE_POS.0, SCALE as f32) };
    let select_y = unsafe { round_down_to_multiple(MOUSE_POS.1, SCALE as f32)};

    let row = (select_y / SCALE as f32) as usize;
    let col = (select_x / SCALE as f32) as usize;

    if grid.get_cell(row, col) {
        canvas.set_draw_color(Color::RGB(255, 0, 0));
    } else {
        canvas.set_draw_color(Color::RGB(0, 255, 0));
    }

    canvas.draw_rect(FRect {
        x: select_x,
        y: select_y,
        w: SCALE as f32,
        h: SCALE as f32,
    }).unwrap();
}

fn draw_grid(canvas: &mut Canvas<Window>, gol: GOL) {
    canvas.set_draw_color(Color::RGB(255, 255, 255));

    canvas.set_draw_color(Color::RGB(64, 64, 64));
    for row_index in 0..gol.grid.grid.len() {
        canvas.draw_line(
            FPoint {
                x: 0.0,
                y: row_index as f32 * SCALE as f32,
            },
            FPoint {
                x: (SCALE * GRID_SIZE as u32) as f32,
                y: row_index as f32 * SCALE as f32,
            }
        ).unwrap();
    }

    for col_index in 0..GRID_SIZE {
        canvas.draw_line(
            FPoint {
                x: col_index as f32 * SCALE as f32,
                y: 0.0,
            },
            FPoint {
                x: col_index as f32 * SCALE as f32,
                y: (SCALE * gol.grid.grid.len() as u32) as f32,
            }
        ).unwrap();
    }
}