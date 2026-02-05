//! Challenge minigames: Chess, Gomoku, Minesweeper, Morris, Rune.

#![allow(unused_imports)]

pub mod chess;
pub mod gomoku;
pub mod menu;
pub mod minesweeper;
pub mod morris;
pub mod rune;

pub use chess::{ChessDifficulty, ChessGame, ChessResult};
pub use gomoku::{GomokuDifficulty, GomokuGame, GomokuResult, Player as GomokuPlayer, BOARD_SIZE};
pub use menu::*;
pub use minesweeper::{MinesweeperDifficulty, MinesweeperGame, MinesweeperResult};
pub use morris::{
    MorrisDifficulty, MorrisGame, MorrisPhase, MorrisResult, Player as MorrisPlayer, ADJACENCIES,
};
pub use rune::{FeedbackMark, RuneDifficulty, RuneGame, RuneResult, RUNE_SYMBOLS};
