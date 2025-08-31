use crate::gol::GOL;
use std::time::Duration;
use raylib::prelude::{Font, RaylibDrawHandle};

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
    pub viewstate: ViewState,
    pub speed: usize,
    pub canvas: RaylibDrawHandle<'a>,
    pub font: Option<Font>,
    pub show_help: bool,
    pub ui_wants_mouse: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub next_grid: crate::gol::Grid,
    pub compute_time: Duration,
}

