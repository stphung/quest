//! Challenge minigames: Chess, Gomoku, Minesweeper, Morris, Rune.

#![allow(unused_imports)]

pub mod chess;
pub mod gomoku;
pub mod menu;
pub mod minesweeper;
pub mod morris;
pub mod rune;
pub mod go;

pub use chess::{ChessDifficulty, ChessGame, ChessResult};
pub use go::{GoDifficulty, GoGame, GoMove, GoResult, Stone, BOARD_SIZE as GO_BOARD_SIZE};
pub use gomoku::{GomokuDifficulty, GomokuGame, GomokuResult, Player as GomokuPlayer, BOARD_SIZE};
pub use menu::*;
pub use minesweeper::{MinesweeperDifficulty, MinesweeperGame, MinesweeperResult};
pub use morris::{
    MorrisDifficulty, MorrisGame, MorrisPhase, MorrisResult, Player as MorrisPlayer, ADJACENCIES,
};
pub use rune::{FeedbackMark, RuneDifficulty, RuneGame, RuneResult, RUNE_SYMBOLS};

/// A currently active challenge minigame. Only one can be active at a time.
#[derive(Debug, Clone)]
pub enum ActiveMinigame {
    Chess(Box<ChessGame>),
    Morris(MorrisGame),
    Gomoku(GomokuGame),
    Minesweeper(MinesweeperGame),
    Rune(RuneGame),
}
