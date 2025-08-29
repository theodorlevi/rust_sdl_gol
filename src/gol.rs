#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Vec2Isize {
    pub(crate) x: isize,
    pub(crate) y: isize,
}

impl Vec2Isize {
    fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Grid {
    pub grid: Vec<Vec2Isize>,
    pub grid_max: Vec2Isize,
    pub grid_min: Vec2Isize,
}

impl Grid {
    pub fn new() -> Grid {
        Default::default()
    }
    pub fn get_cell(&self, x: isize, y: isize) -> (bool, usize) {
        let mut i: usize = 0;
        for cell in &self.grid {
            if *cell == Vec2Isize::new(x, y) {
                return (true, i);
            }
            i += 1;
        }
        (false, i)
    }
    pub fn set_cell(&mut self, x: isize, y: isize, state: bool) {

        let gotten_cell = self.get_cell(x, y);

        if gotten_cell.0 {
            if state {
                return;
            } else {
                self.grid.remove(gotten_cell.1);
            }
        } else {
            if state {
                self.grid.push(Vec2Isize::new(x, y));
            } else {
                return;
            }
        }
    }
    pub fn clear_all(&mut self) {
        self.grid = Vec::new();
    }
    pub fn get_grid(&mut self) -> Vec<Vec2Isize> {
        self.grid.clone()
    }
}

impl Default for Grid {
    fn default() -> Self {
        let grid: Vec<Vec2Isize> = vec![];

        Grid {
            grid,
            grid_min: Vec2Isize::new(0, 0),
            grid_max: Vec2Isize::new(0, 0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

    pub fn update(&mut self) {
        if self.paused { return; }

        let current = self.grid.get_grid();
        if current.is_empty() {
            return;
        }

        let mut alive: std::collections::HashSet<(isize, isize)> = std::collections::HashSet::with_capacity(current.len());
        for c in &current {
            alive.insert((c.x, c.y));
        }

        let mut counts: std::collections::HashMap<(isize, isize), u8> = std::collections::HashMap::with_capacity(alive.len() * 4);
        for (x, y) in &alive {
            let (x, y) = (*x, *y);
            let mut inc = |nx: isize, ny: isize| {
                counts
                    .entry((nx, ny))
                    .and_modify(|c| { if *c < 8 { *c += 1; } })
                    .or_insert(1);
            };
            inc(x - 1, y - 1);
            inc(x - 1, y    );
            inc(x - 1, y + 1);
            inc(x,     y - 1);
            inc(x,     y + 1);
            inc(x + 1, y - 1);
            inc(x + 1, y    );
            inc(x + 1, y + 1);
        }

        let mut next_cells: Vec<Vec2Isize> = Vec::with_capacity(current.len());
        for ((x, y), n) in counts.into_iter() {
            let is_alive = alive.contains(&(x, y));
            let next_alive = match (is_alive, n) {
                (true, 2) | (_, 3) => true, // survive on 2 or 3; birth on 3
                _ => false,
            };
            if next_alive {
                next_cells.push(Vec2Isize::new(x, y));
            }
        }

        self.grid.grid = next_cells;
    }

    fn get_neighbours(&self, x: isize, y: isize) -> u8 {
        let mut neighbours: u8 = 0;
        
        if self.grid.get_cell(x - 1, y - 1).0 { neighbours = neighbours.saturating_add(1); }
        if self.grid.get_cell(x - 1, y    ).0 { neighbours = neighbours.saturating_add(1); }
        if self.grid.get_cell(x - 1, y + 1).0 { neighbours = neighbours.saturating_add(1); }
        if self.grid.get_cell(x,     y - 1).0 { neighbours = neighbours.saturating_add(1); }
        if self.grid.get_cell(x,     y + 1).0 { neighbours = neighbours.saturating_add(1); }
        if self.grid.get_cell(x + 1, y - 1).0 { neighbours = neighbours.saturating_add(1); }
        if self.grid.get_cell(x + 1, y    ).0 { neighbours = neighbours.saturating_add(1); }
        if self.grid.get_cell(x + 1, y + 1).0 { neighbours = neighbours.saturating_add(1); }

        neighbours
    }

    pub fn pause(&mut self) {
        self.paused = !self.paused;
    }
}