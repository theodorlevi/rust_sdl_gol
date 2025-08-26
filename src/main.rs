mod gol;

use std::path::Path;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::render::{FPoint, FRect};
use sdl3::{ttf, Error};
use std::time::{Duration, Instant};
use log::warn;
use sdl3::ttf::{Font, Sdl3TtfContext};
use crate::gol::*;

const SCALE: u32 = 16;
const GRID_SIZE: usize = 64;
static mut MOUSE_POS: (f32, f32) = (0.0, 0.0);

fn round_down_to_multiple(n: f32, step: f32) -> f32 {
    (n / step).floor() * step
}

fn draw_selection(canvas: &mut sdl3::render::Canvas<sdl3::video::Window>) {
    let select_x = unsafe { round_down_to_multiple(MOUSE_POS.0, SCALE as f32) };
    let select_y = unsafe { round_down_to_multiple(MOUSE_POS.1, SCALE as f32)};

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.draw_rect(FRect {
        x: select_x,
        y: select_y,
        w: SCALE as f32,
        h: SCALE as f32,
    }).unwrap();
}

fn draw_text(
    font: &Font,
    canvas: &mut sdl3::render::Canvas<sdl3::video::Window>,
    text: &str,
    font_size: f32,
    color: Color,
    x: f32,
    y: f32,
) {
    let frametime_text = canvas.create_texture_from_surface(
        font.render(
            format!("{}", text)
                .as_str())
            .blended(color)
            .unwrap()).unwrap();

    draw_selection(canvas);

    canvas.copy(
        &frametime_text,
        None,
        FRect {
            x,
            y,
            w: font_size * text.len() as f32,
            h: font_size * 2.0
        }
    ).unwrap();
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

    let window = video_subsystem
        .window("rust-sdl3 demo", GRID_SIZE as u32*SCALE, GRID_SIZE as u32*SCALE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();
    let ttf_context = ttf::init().unwrap();
    let font = ttf_context.load_font(
        "/usr/share/fonts/TTF/JetBrainsMono-Regular.ttf",
        32.0).unwrap_or_else(
        |e|
            handle_font_error(e, ttf_context)
    );

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let grid = Grid::new();
    let mut gol = GOL::new(grid);

    let mut frame_time: Duration = Duration::from_millis(0);

    'running: loop {
        let start_time = Instant::now();

        canvas.set_draw_color(Color::RGB(0, 0,0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));

        let mut i: u32;
        let mut j: u32;

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

        j = 0;
        for row in gol.grid.grid {
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

        draw_text(
            &font,
            &mut canvas,
            frame_time.as_millis().to_string().as_str(),
            12.0,
            Color::RGB(255, 255, 255),
            10.0,
            12.0
        );

        if gol.paused {
            draw_text(
                &font,
                &mut canvas,
                "PAUSED",
                12.0,
                Color::RGB(255, 0, 0),
                10.0,
                32.0
            )
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,

                Event::MouseButtonDown {x, y, ..} => {
                    gol.grid.flip_cell(
                        y as usize / SCALE as usize,
                        x as usize / SCALE as usize,
                    );
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

        let last_time = Instant::now();
        frame_time = last_time - start_time;
        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 144));
    }
}
