use std::io;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::render::{FPoint, FRect};
use std::time::{Duration, Instant};
use sdl3::ttf;
use sdl3::ttf::Font;

const SCALE: u32 = 8;
const GRID_SIZE: usize = 128;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Grid {
    grid: [[bool; GRID_SIZE]; GRID_SIZE],
}

impl Grid {
    fn new() -> Grid {
        Default::default()
    }
    fn get_cell(&self, row: usize, col: usize) -> bool {
        self.grid[row][col]
    }
    fn set_cell(&mut self, row: usize, col: usize, state: bool) {
        self.grid[row][col] = state;
    }
    fn clear_all(&mut self) {
        self.grid = [[false; GRID_SIZE]; GRID_SIZE];
    }
    fn flip_cell(&mut self, row: usize, col: usize) {
        self.grid[row][col] = !self.grid[row][col];
    }
}

impl Default for Grid {
    fn default() -> Self {
        Grid {
            grid: [[false; GRID_SIZE]; GRID_SIZE],
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct GOL {
    grid: Grid,
    paused: bool,
}

impl GOL {
    fn new(grid: Grid) -> GOL {
        GOL {
            grid,
            paused: false,
        }
    }

    fn update(&mut self) {
        if self.paused { return; }

        let mut row_i = 0;
        let mut col_i = 0;

        let self_copy = self.clone();

        for row in self.grid.grid {
            for col in row {
                let neighbours = self_copy.get_neighbours(row_i, col_i);

                if col {
                    if neighbours.len() <= 1 || neighbours.len() >= 4 {
                        self.grid.set_cell(row_i, col_i, false);
                    }
                } else {
                    if neighbours.len() == 3 {
                        self.grid.set_cell(row_i, col_i, true);
                    }
                }
                col_i += 1;
            }
            col_i = 0;
            row_i += 1;
        }
    }

    fn get_neighbours(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        let mut neighbours = Vec::new();

        let total_rows = self.grid.grid.len() as isize;
        let total_cols = self.grid.grid[0].len() as isize;

        let current_row_signed = row as isize;
        let current_col_signed = col as isize;

        for row_offset in -1..=1 {
            for col_offset in -1..=1 {
                if row_offset == 0 && col_offset == 0 { continue; }

                let mut neighbor_row_signed = current_row_signed + row_offset;
                let mut neighbor_col_signed = current_col_signed + col_offset;

                if neighbor_row_signed < 0 { neighbor_row_signed += total_rows; }
                if neighbor_row_signed >= total_rows { neighbor_row_signed -= total_rows; }

                if neighbor_col_signed < 0 { neighbor_col_signed += total_cols; }
                if neighbor_col_signed >= total_cols { neighbor_col_signed -= total_cols; }

                let neighbor_row = neighbor_row_signed as usize;
                let neighbor_col = neighbor_col_signed as usize;

                if self.grid.get_cell(neighbor_row, neighbor_col) {
                    neighbours.push((neighbor_row, neighbor_col));
                }
            }
        }

        neighbours
    }

    fn pause(&mut self) {
        self.paused = !self.paused;
    }
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
    let font = ttf_context.load_font("/usr/share/fonts/TTF/JetBrainsMono-Regular.ttf", 32.0).unwrap();

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

        let frametime_text = canvas.create_texture_from_surface(
            font.render(
                format!("{}", frame_time.as_millis().to_string()
                ).as_str()
            ).blended(Color::WHITE).unwrap()
        ).unwrap();

        canvas.copy(
            &frametime_text,
            None,
            FRect {
                x: 12.0,
                y: 12.0,
                w: 16.0 * frame_time.as_millis().to_string().len() as f32,
                h: 32.0
            }
        ).unwrap();

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
