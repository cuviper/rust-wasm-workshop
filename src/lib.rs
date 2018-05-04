#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;

use std::fmt;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);

    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<u8>,
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        let width = 64;
        let height = 64;

        let cells: Vec<_> = (0..width * height)
            .map(|i| {
                if random() < 0.5 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe::from_cells(width, height, &cells)
    }

    fn from_cells(width: u32, height: u32, cells: &[Cell]) -> Universe {
        assert_eq!((width * height) as usize, cells.len());
        let cells = cells.chunks(8).map(|chunk| {
            chunk.iter().rev().fold(0, |byte, &cell| (byte << 1) | (cell as u8))
        }).collect();
        Universe { width, height, cells, }
    }

    fn to_cells(&self) -> Vec<Cell> {
        let len = (self.width * self.height) as usize;
        let mut cells = Vec::with_capacity(len);
        for i in 0..(len + 7) / 8 {
            cells.extend((0..8).map(|j| self.get_cell(i * 8 + j)));
        }
        cells.truncate(len);
        cells
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn get_row_column(&self, index: usize) -> (u32, u32) {
        (index as u32 / self.width, index as u32 % self.width)
    }

    fn get_cell(&self, index: usize) -> Cell {
        let byte = index / 8;
        let bit = index % 8;
        if self.cells[byte] & (1 << bit) == 0 {
            Cell::Dead
        } else {
            Cell::Alive
        }
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.get_cell(idx) as u8;
            }
        }
        count
    }

    pub fn tick(&mut self) {
        let len = (self.width * self.height) as usize;
        self.cells = (0..(len + 7) / 8).map(|i| {
            (0..8).fold(0, |acc, j| {
                let index = i * 8 + j;
                let (row, col) = self.get_row_column(index);
                let n = self.live_neighbor_count(row, col);
                let cell = match (self.get_cell(index), n) {
                    (Cell::Alive, 2) | (_, 3) => Cell::Alive,
                    _ => Cell::Dead,
                };
                acc | ((cell as u8) << j)
            })
        }).collect();
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const u8 {
        self.cells.as_ptr()
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..self.height {
            for col in 0..self.width {
                let i = self.get_index(row, col);
                let c = match self.get_cell(i) {
                    Cell::Alive => '◼',
                    Cell::Dead => '◻',
                };
                write!(f, "{} ", c)?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

#[test]
fn universe_displays_correctly() {
    let universe = Universe::from_cells(
        4,
        4,
        &[
            Cell::Dead,  Cell::Dead,  Cell::Dead,  Cell::Dead,
            Cell::Dead,  Cell::Dead,  Cell::Dead,  Cell::Alive,
            Cell::Dead,  Cell::Dead,  Cell::Alive, Cell::Alive,
            Cell::Dead,  Cell::Alive, Cell::Alive, Cell::Alive,
        ],
    );

    assert_eq!(
        universe.to_string(),
        "◻ ◻ ◻ ◻ \n\
         ◻ ◻ ◻ ◼ \n\
         ◻ ◻ ◼ ◼ \n\
         ◻ ◼ ◼ ◼ \n"
    );
}

use Cell::*;

fn assert_tick(w: u32, h: u32, before: &[Cell], after: &[Cell]) {
    assert_eq!(before.len(), after.len());
    assert_eq!(w as usize * h as usize, before.len());

    let mut universe = Universe::from_cells(
        w,
        h,
        &before,
    );
    universe.tick();

    assert_eq!(&universe.to_cells()[..], after);
}

#[test]
fn tick_rule_1() {
    assert_tick(
        5,
        5,
        &[
            Dead, Dead, Dead,  Dead, Dead,
            Dead, Dead, Dead,  Dead, Dead,
            Dead, Dead, Alive, Dead, Dead,
            Dead, Dead, Dead,  Dead, Dead,
            Dead, Dead, Dead,  Dead, Dead,
        ],
        &[
            Dead, Dead, Dead, Dead, Dead,
            Dead, Dead, Dead, Dead, Dead,
            Dead, Dead, Dead, Dead, Dead,
            Dead, Dead, Dead, Dead, Dead,
            Dead, Dead, Dead, Dead, Dead,
        ],
    );
}

#[test]
fn tick_rule_2() {
    assert_tick(
        5,
        5,
        &[
            Dead, Dead,  Dead,  Dead, Dead,
            Dead, Dead,  Dead,  Dead, Dead,
            Dead, Alive, Alive, Dead, Dead,
            Dead, Alive, Alive, Dead, Dead,
            Dead, Dead,  Dead,  Dead, Dead,
        ],
        &[
            Dead, Dead,  Dead,  Dead, Dead,
            Dead, Dead,  Dead,  Dead, Dead,
            Dead, Alive, Alive, Dead, Dead,
            Dead, Alive, Alive, Dead, Dead,
            Dead, Dead,  Dead,  Dead, Dead,
        ],
    );
}

#[test]
fn tick_rules_3_and_4() {
    assert_tick(
        5,
        5,
        &[
            Dead, Dead,  Dead,  Dead,  Dead,
            Dead, Dead,  Alive, Dead,  Dead,
            Dead, Alive, Alive, Alive, Dead,
            Dead, Dead,  Alive, Dead,  Dead,
            Dead, Dead,  Dead,  Dead,  Dead,
        ],
        &[
            Dead, Dead,  Dead,  Dead,  Dead,
            Dead, Alive, Alive, Alive, Dead,
            Dead, Alive, Dead,  Alive, Dead,
            Dead, Alive, Alive, Alive, Dead,
            Dead, Dead,  Dead,  Dead,  Dead,
        ],
    );
}

#[test]
fn tick_cells_on_edge() {
    assert_tick(
        5,
        5,
        &[
            Dead,  Dead, Dead, Dead,  Dead,
            Dead,  Dead, Dead, Dead,  Dead,
            Alive, Dead, Dead, Alive, Alive,
            Dead,  Dead, Dead, Dead,  Dead,
            Dead,  Dead, Dead, Dead,  Dead,
        ],
        &[
            Dead, Dead, Dead, Dead, Dead,
            Dead, Dead, Dead, Dead, Alive,
            Dead, Dead, Dead, Dead, Alive,
            Dead, Dead, Dead, Dead, Alive,
            Dead, Dead, Dead, Dead, Dead,
        ],
    );
}
