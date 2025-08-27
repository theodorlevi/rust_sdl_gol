use crate::GRID_SIZE;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Grid {
    pub grid: [[bool; GRID_SIZE]; GRID_SIZE],
}

impl Grid {
    pub fn new() -> Grid {
        Default::default()
    }
    pub fn get_cell(&self, row: usize, col: usize) -> bool {
        self.grid[row][col]
    }
    pub fn set_cell(&mut self, row: usize, col: usize, state: bool) {
        self.grid[row][col] = state;
    }
    pub fn clear_all(&mut self) {
        self.grid = [[false; GRID_SIZE]; GRID_SIZE];
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

    pub fn pause(&mut self) {
        self.paused = !self.paused;
    }
}