use crate::gol::GOL;
use sdl3::render::{Canvas, TextureCreator};
use sdl3::ttf::Font;
use sdl3::video::{Window, WindowContext};
use std::time::Duration;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vector2 { x, y }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ViewState {
    pub camera_pos: Vector2,
    pub mouse_pos: Vector2,
    pub zoom: f32,
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

pub struct RenderCtx<'a> {
    pub gol: GOL,
    pub frame_time: Duration,
    pub viewstate: ViewState,
    pub speed: usize,
    pub canvas: &'a mut Canvas<Window>,
    pub texture_creator: &'a mut TextureCreator<WindowContext>,
    pub font: &'a Font<'a>,
}

