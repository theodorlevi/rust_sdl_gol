use rayon::iter::ParallelIterator;
use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator};
use std::collections::HashSet;
use std::thread;

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub struct Vec2Isize {
    pub(crate) x: isize,
    pub(crate) y: isize,
}

impl Vec2Isize {
    fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct Grid {
    pub grid: HashSet<Vec2Isize>,
}

impl Grid {
    pub fn new() -> Grid {
        Default::default()
    }
    pub fn get_cell(&self, x: isize, y: isize) -> bool {
        self.grid.contains(&Vec2Isize::new(x, y))
    }

    pub fn set_cell(&mut self, x: isize, y: isize, state: bool) {

        let gotten_cell = self.get_cell(x, y);

        if gotten_cell {
            if state {
                return;
            } else {
                self.grid.remove(&Vec2Isize::new(x, y));
            }
        } else {
            if state {
                self.grid.insert(Vec2Isize::new(x, y));
            } else {
                return;
            }
        }
    }

    pub fn clear_all(&mut self) {
        self.grid = HashSet::new();
    }
    pub fn get_grid(&mut self) -> HashSet<Vec2Isize> {
        self.grid.clone()
    }
}

impl Default for Grid {
    fn default() -> Self {
        let grid: HashSet<Vec2Isize> = HashSet::new();

        Grid {
            grid,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GOL {
    pub grid: Grid,
    pub paused: bool,
}

impl GOL {
    pub fn new(grid: Grid) -> GOL {
        GOL {
            grid,
            paused: true,
        }
    }

    pub(crate) fn update_from(grid: &Grid, delay: u64) -> Grid {
        let start_time = std::time::Instant::now();
        rayon::ThreadPoolBuilder::new()
            .num_threads(rayon::current_num_threads())
            .build()
            .ok();

        let current = &grid.grid;
        if current.is_empty() {
            return Grid { grid: HashSet::new() };
        }

        let alive: HashSet<Vec2Isize> = current.par_iter()
            .map(|c| Vec2Isize::new(c.x, c.y))
            .collect();

        let all_candidates: HashSet<Vec2Isize> = alive
            .par_iter()
            .flat_map(|candidate| {
                let (x, y) = (candidate.x, candidate.y);
                [
                    Vec2Isize::new(x - 1, y - 1), Vec2Isize::new(x - 1, y), Vec2Isize::new(x - 1, y + 1),
                    Vec2Isize::new(x, y - 1), Vec2Isize::new(x, y), Vec2Isize::new(x, y + 1),
                    Vec2Isize::new(x + 1, y - 1), Vec2Isize::new(x + 1, y), Vec2Isize::new(x + 1, y + 1),
                ].into_par_iter()
            })
            .collect();

        let next_cells: HashSet<Vec2Isize> = all_candidates
            .par_iter()
            .filter_map(|next_cell| {
                let (x, y) = (next_cell.x, next_cell.y);
                let mut count = 0u8;

                if alive.contains(&Vec2Isize::new(x - 1, y - 1)) { count += 1; }
                if alive.contains(&Vec2Isize::new(x - 1, y)) { count += 1; }
                if alive.contains(&Vec2Isize::new(x - 1, y + 1)) { count += 1; }
                if alive.contains(&Vec2Isize::new(x, y - 1)) { count += 1; }
                if alive.contains(&Vec2Isize::new(x, y + 1)) { count += 1; }
                if alive.contains(&Vec2Isize::new(x + 1, y - 1)) { count += 1; }
                if alive.contains(&Vec2Isize::new(x + 1, y)) { count += 1; }
                if alive.contains(&Vec2Isize::new(x + 1, y + 1)) { count += 1; }

                let is_alive = alive.contains(&Vec2Isize::new(x, y));
                match (is_alive, count) {
                    (true, 2) | (_, 3) => Some(Vec2Isize::new(x, y)),
                    _ => None,
                }
            })
            .collect();
        let end_time = std::time::Instant::now();
        thread::sleep(std::time::Duration::from_micros((delay * 1000).saturating_sub(end_time.duration_since(start_time).as_micros() as u64)));
        Grid { grid: next_cells }
    }

    pub fn pause(&mut self) {
        self.paused = !self.paused;
    }
}