use axum::{
    extract::{Path, State},
    http::{Response, StatusCode},
};
use log::info;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub(super) enum Error {
    #[error("Invalid column number: {0}")]
    InvalidColumn(usize),
    #[error("Column is full: {0}")]
    ColumnFull(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum SquareState {
    Empty,
    Cookie,
    Milk,
}

impl Display for SquareState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SquareState::Empty => "⬛",
                SquareState::Cookie => "🍪",
                SquareState::Milk => "🥛",
            }
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum GameState {
    Ongoing,
    Cookie,
    Milk,
    Draw,
}

const BOARD_SIZE: usize = 4;
const BOARD_AREA: usize = BOARD_SIZE * BOARD_SIZE;

/// The game board
/// Indexed Column first,  bottom to top
/// |---+---+---+---|
/// |0,3|1,3|2,3|3,3|
/// |---+---+---+---|
/// |0,2|1,2|2,2|3,2|
/// |---+---+---+---|
/// |0,1|1,1|2,1|3,1|
/// |---+---+---+---|
/// |0,0|1,0|2,0|3,0|
/// |---+---+---+---|
pub(super) struct Board {
    squares: [SquareState; BOARD_AREA],
}

impl Board {
    pub fn new() -> Self {
        Self {
            squares: [SquareState::Empty; 16],
        }
    }

    pub fn random(rng: &mut rand::rngs::StdRng) -> Self {
        let mut board = Self::new();
        for row in 0..4 {
            let row = 3 - row;
            for column in 0..4 {
                let square = board.cell_mut(row, column);
                *square = match rng.gen::<bool>() {
                    true => SquareState::Cookie,
                    false => SquareState::Milk,
                };
                info!("random value for [{}][{}] = {}", row, column, square);
            }
        }
        board
    }

    fn cell(&self, row: usize, column: usize) -> &SquareState {
        &self.squares[row * 4 + column]
    }

    fn cell_mut(&mut self, row: usize, column: usize) -> &mut SquareState {
        &mut self.squares[row * 4 + column]
    }

    fn reset(&mut self) {
        self.squares = [SquareState::Empty; 16];
    }

    fn place_item(&mut self, column_idx: usize, item: SquareState) -> Result<(), Error> {
        if column_idx >= BOARD_SIZE {
            return Err(Error::InvalidColumn(column_idx));
        }
        for row in 0..BOARD_SIZE {
            let square = self.cell_mut(row, column_idx);
            if *square == SquareState::Empty {
                *square = item;
                info!("Placed {:?} in column {}, row {}", item, column_idx, row);
                info!("Square value: {square}");
                return Ok(());
            }
        }
        Err(Error::ColumnFull(column_idx))
    }

    fn game_over(&self) -> GameState {
        // Check for horizontal wins
        for row in 0..BOARD_SIZE {
            for column in 0..BOARD_SIZE - 3 {
                let square = self.cell(row, column);
                if *square != SquareState::Empty
                    && square == self.cell(row, column + 1)
                    && square == self.cell(row, column + 2)
                    && square == self.cell(row, column + 3)
                {
                    match square {
                        SquareState::Cookie => return GameState::Cookie,
                        SquareState::Milk => return GameState::Milk,
                        _ => unreachable!("We already made sure the square wasn't empty"),
                    };
                }
            }
        }
        // Check for vertical wins
        for column in 0..BOARD_SIZE {
            for row in 0..BOARD_SIZE - 3 {
                let square = self.cell(row, column);
                if *square != SquareState::Empty
                    && square == self.cell(row + 1, column)
                    && square == self.cell(row + 2, column)
                    && square == self.cell(row + 3, column)
                {
                    match square {
                        SquareState::Cookie => return GameState::Cookie,
                        SquareState::Milk => return GameState::Milk,
                        _ => unreachable!("We already made sure the square wasn't empty"),
                    };
                }
            }
        }
        // Check for diagonal wins
        for row in 0..BOARD_SIZE - 3 {
            for column in 0..BOARD_SIZE - 3 {
                let square = self.cell(row, column);
                if *square != SquareState::Empty
                    && square == self.cell(row + 1, column + 1)
                    && square == self.cell(row + 2, column + 2)
                    && square == self.cell(row + 3, column + 3)
                {
                    match square {
                        SquareState::Cookie => return GameState::Cookie,
                        SquareState::Milk => return GameState::Milk,
                        _ => unreachable!("We already made sure the square wasn't empty"),
                    };
                }
            }
        }
        for row in 0..BOARD_SIZE - 3 {
            for column in 3..BOARD_SIZE {
                let square = self.cell(row, column);
                if *square != SquareState::Empty
                    && square == self.cell(row + 1, column - 1)
                    && square == self.cell(row + 2, column - 2)
                    && square == self.cell(row + 3, column - 3)
                {
                    match square {
                        SquareState::Cookie => return GameState::Cookie,
                        SquareState::Milk => return GameState::Milk,
                        _ => unreachable!("We already made sure the square wasn't empty"),
                    };
                }
            }
        }
        for square in self.squares.iter() {
            if *square == SquareState::Empty {
                return GameState::Ongoing;
            }
        }
        GameState::Draw
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..BOARD_SIZE {
            write!(f, "⬜")?;
            for column in 0..BOARD_SIZE {
                write!(f, "{}", self.cell(BOARD_SIZE - 1 - row, column))?;
            }
            write!(f, "⬜")?;
            writeln!(f)?;
        }
        for _ in 0..6 {
            write!(f, "⬜")?;
        }
        write!(f, "\n")?;
        let result = self.game_over();
        match result {
            GameState::Cookie => {
                write!(f, "🍪 wins!\n")?;
            }
            GameState::Milk => {
                write!(f, "🥛 wins!\n")?;
            }
            GameState::Draw => {
                write!(f, "No winner.\n")?;
            }
            _ => (),
        }
        Ok(())
    }
}

pub(super) struct AppState {
    board: Board,
    rng: rand::rngs::StdRng,
}

impl AppState {
    pub fn construct() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            board: Board::new(),
            rng: rand::SeedableRng::seed_from_u64(2024),
        }))
    }
}

pub(super) async fn board_state(State(state): State<Arc<Mutex<AppState>>>) -> String {
    state.lock().unwrap().board.to_string()
}

pub(super) async fn reset_board(State(state): State<Arc<Mutex<AppState>>>) -> String {
    let mut state = state.lock().unwrap();
    state.board.reset();
    state.rng = rand::SeedableRng::seed_from_u64(2024);
    state.board.to_string()
}

pub(super) async fn place(
    State(state): State<Arc<Mutex<AppState>>>,
    Path((team, mut column)): Path<(SquareState, usize)>,
) -> Response<String> {
    info!("Placed {:?} in column {}", team, column);

    if column >= 1 && column <= 4 {
        column -= 1;
    } else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Invalid column number".into())
            .unwrap();
    }
    let mut state = state.lock().unwrap();
    let result = state.board.game_over();
    if result != GameState::Ongoing {
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(state.board.to_string().into())
            .unwrap();
    }
    let response_code: StatusCode;
    match state.board.place_item(column, team) {
        Ok(_) => {
            response_code = StatusCode::OK;
        }
        Err(_) => response_code = StatusCode::SERVICE_UNAVAILABLE,
    };

    print!("{}", state.board.to_string());

    Response::builder()
        .status(response_code)
        .body(state.board.to_string().into())
        .unwrap()
}

pub(super) async fn random_board(State(state): State<Arc<Mutex<AppState>>>) -> String {
    let mut state = state.lock().unwrap();
    let board = Board::random(&mut state.rng);
    info!("Random board:\n{board}");
    board.to_string()
}
