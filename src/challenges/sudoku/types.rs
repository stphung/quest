use crate::challenges::difficulty_enum_impl;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SudokuDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(SudokuDifficulty);

impl SudokuDifficulty {
    /// Number of cells to remove from the solved board
    pub fn cells_to_remove_range(&self) -> (usize, usize) {
        match self {
            SudokuDifficulty::Novice => (39, 43),
            SudokuDifficulty::Apprentice => (47, 51),
            SudokuDifficulty::Journeyman => (53, 55),
            SudokuDifficulty::Master => (57, 59),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SudokuResult {
    Win,
    Loss,
}

#[derive(Debug, Clone)]
pub struct SudokuGame {
    pub difficulty: SudokuDifficulty,
    pub board: [[u8; 9]; 9],
    pub solution: [[u8; 9]; 9],
    pub given: [[bool; 9]; 9],
    pub conflicts: [[bool; 9]; 9],
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub game_result: Option<SudokuResult>,
    pub forfeit_pending: bool,
}

impl SudokuGame {
    pub fn new(
        difficulty: SudokuDifficulty,
        board: [[u8; 9]; 9],
        solution: [[u8; 9]; 9],
        given: [[bool; 9]; 9],
    ) -> Self {
        Self {
            difficulty,
            board,
            solution,
            given,
            conflicts: [[false; 9]; 9],
            cursor_row: 0,
            cursor_col: 0,
            game_result: None,
            forfeit_pending: false,
        }
    }
}
