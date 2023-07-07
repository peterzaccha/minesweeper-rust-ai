use js_sys::Array;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::game::{CellInteraction, Game, GameStatus, Position};

#[wasm_bindgen]
struct AIPlayer {
    matrix: Vec<Vec<Option<f64>>>,
    pub width: usize,
    pub height: usize,
}

enum BestMove {
    Open(Position),
    Flag(Position),
    Random,
}

impl AIPlayer {
    pub fn batch_moves(&self) -> Vec<BestMove> {
        let mut moves: Vec<BestMove> = vec![];
        for i in 0..self.height {
            for j in 0..self.width {
                if self.matrix[i][j].is_some() {
                    if self.matrix[i][j].unwrap() == 1.0 {
                        moves.push(BestMove::Flag(Position::new(j, i)));
                    } else {
                        moves.push(BestMove::Open(Position::new(j, i)));
                    }
                }
            }
        }
        moves
    }
}

#[wasm_bindgen]
impl AIPlayer {
    #[wasm_bindgen(constructor)]
    pub fn new(game: &Game) -> Self {
        Self {
            matrix: {
                let mut matrix: Vec<Vec<Option<f64>>> = vec![];
                for i in 0..game.height {
                    matrix.push(vec![]);
                    for _ in 0..game.width {
                        matrix[i].push(None);
                    }
                }
                matrix
            },
            width: game.width,
            height: game.height,
        }
    }

    pub fn get_matrix_js_value(&self) -> Array {
        let arr = Array::new_with_length(self.height as u32);
        for i in 0..self.height {
            let row = Array::new_with_length(self.width as u32);
            for j in 0..self.width {
                row.set(j as u32, self.matrix[i][j].into())
            }
            arr.set(i as u32, row.into());
        }
        arr
    }
    pub fn should_flag(&self) -> Option<Position> {
        let mut max_pos: Option<Position> = None;
        let mut max_value = self.matrix[0][0].unwrap_or(0.0);
        for i in 0..self.height {
            for j in 0..self.width {
                if self.matrix[i][j].is_some() && self.matrix[i][j].unwrap_or(0.0) == 1.0 {
                    max_value = self.matrix[i][j].unwrap_or(0.0);
                    max_pos = Some(Position::new(j, i));
                    return max_pos;
                }
            }
        }
        max_pos
    }
    pub fn best_play(&self) -> Option<Position> {
        let mut min_pos: Option<Position> = None;
        let mut min_value = self.matrix[0][0].unwrap_or(1.0);
        for i in 0..self.height {
            for j in 0..self.width {
                if self.matrix[i][j].is_some() && self.matrix[i][j].unwrap_or(1.0) == 0.0 {
                    min_value = self.matrix[i][j].unwrap_or(1.0);
                    min_pos = Some(Position::new(j, i));
                    return min_pos;
                }
            }
        }
        min_pos
    }

    pub fn calculate_matrix(&mut self, game: &Game) {
        for i in 0..game.height {
            for j in 0..game.width {
                let cell = game.grid().0[i][j];
                match cell.interaction {
                    crate::game::CellInteraction::Opened => {
                        self.matrix[i][j] = None;
                        let n = game.neighbors(&Position::new(j, i));

                        let mut mines: Vec<Position> = vec![];
                        let mut closed: Vec<Position> = vec![];
                        let mut opened: Vec<Position> = vec![];
                        for pos in n {
                            let ncell = game.grid().0[pos.y][pos.x];
                            match ncell.interaction {
                                crate::game::CellInteraction::Opened => opened.push(pos),
                                crate::game::CellInteraction::Flagged => mines.push(pos),
                                crate::game::CellInteraction::Closed => closed.push(pos),
                            };
                        }
                        if closed.len() == (cell.counter().unwrap() as usize - mines.len()) {
                            for c in closed.iter() {
                                self.matrix[c.y][c.x] = Some(1.0);
                            }
                        }
                        if mines.len() == cell.counter().unwrap() as usize {
                            for c in closed.iter() {
                                self.matrix[c.y][c.x] = Some(0.0);
                            }
                        }
                    }
                    CellInteraction::Flagged => {
                        self.matrix[i][j] = None;
                    }

                    _ => {}
                }
            }
        }
    }
}

#[test]
fn test() {
    let mut wins = 0;
    let mut loses = 0;
    for i in 0..100 {
        let mut game = Game::new(50, 50, 0.1);
        let mut ai = AIPlayer::new(&game);

        let mut best_moves: Vec<BestMove> = vec![];

        while matches!(game.status, GameStatus::OnGoing) {
            if best_moves.len() == 0 {
                game.open_random()
            } else {
                best_moves.iter().for_each(|m| match m {
                    BestMove::Open(p) => game.open(p),
                    BestMove::Flag(p) => game.flag(p),
                    BestMove::Random => {}
                })
            }
            ai.calculate_matrix(&game);
            best_moves = ai.batch_moves();
        }
        println!("{game:?}");
        println!("{:?}", game.status);
        match game.status {
            GameStatus::Lose => loses += 1,
            GameStatus::Win => wins += 1,
            GameStatus::OnGoing => unreachable!(),
        }
    }

    println!("Wins : {}", wins);
    println!("Loses : {}", loses);
    // println!("Winning Percent : {}%", wins as f32 );
}
