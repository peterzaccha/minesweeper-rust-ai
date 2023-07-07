use std::{collections::HashMap, fmt::Debug};

use js_sys::{Array, Uint32Array};
use rand::Rng;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum CellInteraction {
    Opened,
    Flagged,
    Closed,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Cell {
    has_mine: bool,
    pub interaction: CellInteraction,
    counter: u8,
}

#[wasm_bindgen]
impl Cell {
    pub fn new(has_mine: bool) -> Self {
        Self {
            has_mine,
            interaction: CellInteraction::Closed,
            counter: 0,
        }
    }

    fn open(&mut self) {
        self.interaction = CellInteraction::Opened;
    }
    fn flag(&mut self) {
        self.interaction = CellInteraction::Flagged;
    }
    fn unflag(&mut self) {
        self.interaction = CellInteraction::Closed;
    }
    pub fn counter(&self) -> Option<u8> {
        if matches!(self.interaction, CellInteraction::Opened) {
            return Some(self.counter);
        }
        None
    }
    pub fn has_mine(&self) -> Option<bool> {
        if matches!(self.interaction, CellInteraction::Opened) {
            return Some(self.has_mine);
        }
        None
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut numbers: HashMap<u8, &str> = HashMap::new();

        numbers.insert(1, "1️⃣ ");
        numbers.insert(2, "2️⃣ ");
        numbers.insert(3, "3️⃣ ");
        numbers.insert(4, "4️⃣ ");
        numbers.insert(5, "5️⃣ ");
        numbers.insert(6, "6️⃣ ");
        numbers.insert(7, "7️⃣ ");
        numbers.insert(8, "8️⃣ ");
        match self.interaction {
            CellInteraction::Opened => match self.has_mine {
                true => f.write_str("💣")?,
                false => {
                    if self.counter > 0 {
                        f.write_str(numbers.get(&self.counter).unwrap())?;
                        // f.write_str(format!("{}", self.counter).as_str())?;
                    } else {
                        f.write_str("⬜️")?;
                    }
                }
            },
            CellInteraction::Flagged => f.write_str("🏳️ ")?,
            CellInteraction::Closed => f.write_str("🟨")?,
        };
        Ok(())
    }
}
#[derive(Debug, Clone, Copy)]
#[wasm_bindgen]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[wasm_bindgen]
impl Position {
    #[wasm_bindgen(constructor)]
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

// #[derive(Serialize, Deserialize)]
pub struct Grid(pub Vec<Vec<Cell>>);

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum GameStatus {
    OnGoing,
    Lose,
    Win,
}

#[wasm_bindgen]
pub struct Game {
    grid: Grid,
    pub status: GameStatus,
    pub width: usize,
    pub height: usize,
}

impl Game {
    pub fn grid(&self) -> &Grid {
        &self.grid
    }
    pub fn neighbors(&self, position: &Position) -> Vec<Position> {
        let mut neighnors: Vec<Position> = vec![];
        for x in 0.max(position.x as i32 - 1) as usize..self.width.min(position.x + 2) {
            for y in 0.max(position.y as i32 - 1) as usize..self.height.min(position.y + 2) {
                // if x != position.x && y != position.y {
                neighnors.push(Position::new(x, y));
                // }
            }
        }
        neighnors
    }
}
#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize, mine_chance: f64) -> Self {
        let mut new = Self {
            width,
            height,
            grid: {
                let mut grid: Grid = Grid(vec![]);
                let mut rng = rand::thread_rng();

                for i in 0..height {
                    grid.0.push(vec![]);
                    for _ in 0..width {
                        let random_number: f64 = rng.gen();
                        grid.0[i].push(Cell::new(random_number < mine_chance));
                    }
                }
                grid
            },
            status: GameStatus::OnGoing,
        };
        new.set_counts();

        println!("{:?}", new.neighbors(&Position::new(0, 0)));
        new
    }

    pub fn get_grid_js_value(&self) -> Array {
        let arr = Array::new_with_length(self.height as u32);
        for i in 0..self.height {
            let row = Array::new_with_length(self.width as u32);
            for j in 0..self.width {
                row.set(j as u32, self.grid.0[i][j].into())
            }
            arr.set(i as u32, row.into());
        }

        arr
    }
    fn set_counts(&mut self) {
        for i in 0..self.height {
            for j in 0..self.width {
                let mut counter = 0;
                for pos in self.neighbors(&Position::new(j, i)) {
                    if self.grid.0[pos.y][pos.x].has_mine {
                        counter += 1;
                    }
                }
                self.grid.0[i][j].counter = counter
            }
        }
    }

    pub fn open_all(&mut self) {
        for i in 0..self.height {
            for j in 0..self.width {
                self.grid.0[i][j].interaction = CellInteraction::Opened
            }
        }
    }

    pub fn unflag(&mut self, position: Position) {
        let cell = &mut self.grid.0[position.y][position.x];
        match cell.interaction {
            CellInteraction::Flagged => {
                cell.unflag();
                self.check_win();
            }
            _ => {}
        }
    }
    pub fn flag(&mut self, position: &Position) {
        let cell = &mut self.grid.0[position.y][position.x];
        match cell.interaction {
            CellInteraction::Closed => {
                cell.flag();
                self.check_win();
            }
            _ => {}
        }
    }

    pub fn check_win(&mut self) {
        if matches!(self.status, GameStatus::Lose) {
            return;
        }
        for i in 0..self.height {
            for j in 0..self.width {
                let cell = &self.grid.0[i][j];
                if matches!(cell.interaction, CellInteraction::Closed)
                    || (matches!(cell.interaction, CellInteraction::Flagged) && !cell.has_mine)
                {
                    return;
                }
            }
        }
        self.status = GameStatus::Win
    }
    pub fn open(&mut self, position: &Position) {
        if matches!(self.status, GameStatus::Lose | GameStatus::Win) {
            return;
        }
        let cell = &mut self.grid.0[position.y][position.x];
        match cell.interaction {
            CellInteraction::Closed => {
                cell.open();
                if cell.has_mine {
                    self.status = GameStatus::Lose;
                    self.open_all();
                } else if cell.counter == 0 {
                    for n in self.neighbors(position) {
                        self.open(&n)
                    }
                }
                self.check_win();
            }
            _ => {}
        }
    }
    pub fn open_random(&mut self) {
        let total = (self.width * self.height) as f64;
        let mut prop = 1.0 / total as f64;
        let mut counter = 0.0;
        for i in 0..self.height {
            for j in 0..self.width {
                let mut rng = rand::thread_rng();
                counter += 1.0;
                prop = counter / total;
                let random_number: f64 = rng.gen();
                if prop > random_number
                    && matches!(self.grid.0[i][j].interaction, CellInteraction::Closed)
                {
                    self.open(&Position::new(j, i));
                    return;
                }
            }
        }
    }
}
impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.grid.0 {
            for cell in row {
                f.write_str(format!("{:?}", cell).as_str())?;
            }
            f.write_str("\n")?;
        }
        f.write_str(format!("Lose: {:?}", self.status).as_str())?;
        Ok(())
    }
}

#[test]
fn test() {
    let mut game = Game::new(30, 30, 0.1);
    // game.open(Position::new(0, 0));
    game.open_random();
    // game.open(Position::new(0, 1));
    // game.open(Position::new(1, 1));

    println!("{game:?}")
}
