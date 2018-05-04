#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;

use std::fmt;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);
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
    cells: Vec<Cell>,
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        let width = 64;
        let height = 64;

        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width,
            height,
            cells,
        }
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
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    pub fn tick(&mut self) {
        self.cells = self.cells.iter().enumerate().map(|(i, cell)| {
            let (row, col) = self.get_row_column(i);
            let n = self.live_neighbor_count(row, col);
            match (cell, n) {
                (Cell::Alive, 2) | (_, 3) => Cell::Alive,
                _ => Cell::Dead,
            }
        }).collect();
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..self.height {
            for col in 0..self.width {
                let i = self.get_index(row, col);
                let c = match self.cells[i as usize] {
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
    let universe = Universe {
        width: 4,
        height: 4,
        cells: vec![
            Cell::Dead,  Cell::Dead,  Cell::Dead,  Cell::Dead,
            Cell::Dead,  Cell::Dead,  Cell::Dead,  Cell::Alive,
            Cell::Dead,  Cell::Dead,  Cell::Alive, Cell::Alive,
            Cell::Dead,  Cell::Alive, Cell::Alive, Cell::Alive,
        ],
    };

    assert_eq!(
        universe.to_string(),
        "◻ ◻ ◻ ◻ \n\
         ◻ ◻ ◻ ◼ \n\
         ◻ ◻ ◼ ◼ \n\
         ◻ ◼ ◼ ◼ \n"
    );
}

use Cell::*;

fn assert_tick(w: u32, h: u32, before: Vec<Cell>, after: Vec<Cell>) {
    assert_eq!(before.len(), after.len());
    assert_eq!(w as usize * h as usize, before.len());

    let mut universe = Universe {
        width: w,
        height: h,
        cells: before,
    };
    universe.tick();

    assert_eq!(
        &universe.cells[..],
        &after[..]
    );
}

#[test]
fn tick_rule_1() {
    assert_tick(
        5,
        5,
        vec![
            Dead, Dead, Dead,  Dead, Dead,
            Dead, Dead, Dead,  Dead, Dead,
            Dead, Dead, Alive, Dead, Dead,
            Dead, Dead, Dead,  Dead, Dead,
            Dead, Dead, Dead,  Dead, Dead,
        ],
        vec![
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
        vec![
            Dead, Dead,  Dead,  Dead, Dead,
            Dead, Dead,  Dead,  Dead, Dead,
            Dead, Alive, Alive, Dead, Dead,
            Dead, Alive, Alive, Dead, Dead,
            Dead, Dead,  Dead,  Dead, Dead,
        ],
        vec![
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
        vec![
            Dead, Dead,  Dead,  Dead,  Dead,
            Dead, Dead,  Alive, Dead,  Dead,
            Dead, Alive, Alive, Alive, Dead,
            Dead, Dead,  Alive, Dead,  Dead,
            Dead, Dead,  Dead,  Dead,  Dead,
        ],
        vec![
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
        vec![
            Dead,  Dead, Dead, Dead,  Dead,
            Dead,  Dead, Dead, Dead,  Dead,
            Alive, Dead, Dead, Alive, Alive,
            Dead,  Dead, Dead, Dead,  Dead,
            Dead,  Dead, Dead, Dead,  Dead,
        ],
        vec![
            Dead, Dead, Dead, Dead, Dead,
            Dead, Dead, Dead, Dead, Alive,
            Dead, Dead, Dead, Dead, Alive,
            Dead, Dead, Dead, Dead, Alive,
            Dead, Dead, Dead, Dead, Dead,
        ],
    );
}
