use crate::gol::{Grid, GOL};
use crate::ViewState;
use sdl3::pixels::Color;
use sdl3::render::{Canvas, FRect, TextureCreator};
use sdl3::ttf::Font;
use sdl3::video::{Window, WindowContext};
use std::time::Duration;

pub fn main_draw(canvas: &mut Canvas<Window>, gol: &mut GOL, frame_time: Duration, font: &Font, viewstate: &mut ViewState, texture_creator: &mut TextureCreator<WindowContext>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    draw_cells(&mut gol.grid, canvas, viewstate);

    draw_selection(canvas, &gol.grid, viewstate);

    draw_text(
        &font,
        canvas,
        frame_time.as_millis().to_string().as_str(),
        24.0,
        Color::RGB(255, 255, 255),
        10.0,
        16.0,
        texture_creator,
    );

    draw_text(
        &font,
        canvas,
        format!("{}x zoom", viewstate.zoom).as_str(),
        24.0,
        Color::RGB(255, 255, 255),
        10.0,
        48.0,
        texture_creator,
    );

    if gol.paused {
        draw_text(
            &font,
            canvas,
            "PAUSED",
            24.0,
            Color::RGB(255, 0, 0),
            10.0,
            32.0,
            texture_creator
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
    texture_creator: &mut TextureCreator<WindowContext>,) {
    
    let text_texture = texture_creator.create_texture_from_surface(
        font.render(
            format!("{}", text)
                .as_str())
            .blended(color)
            .unwrap()
        ).unwrap();

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
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for cell in grid.get_grid() {
        canvas.fill_rect(
            FRect {
                x: cell.y as f32 * viewstate.zoom + viewstate.camera_pos.x,
                y: cell.x as f32 * viewstate.zoom + viewstate.camera_pos.y,
                w: viewstate.zoom,
                h: viewstate.zoom,
            }
        ).unwrap();
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

    let select_world_x = round_down_to_multiple(world_x, scale);
    let select_world_y = round_down_to_multiple(world_y, scale);

    let x = (select_world_y / scale) as isize;
    let y = (select_world_x / scale) as isize;

    if grid.get_cell(x, y).0 {
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