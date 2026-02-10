//! Go (Territory Control) minigame.

#![allow(unused_imports)]

pub mod logic;
pub mod mcts;
pub mod types;

pub use logic::{
    calculate_score, get_legal_moves, is_legal_move, make_move, process_go_ai, process_human_move,
    process_human_pass, process_input,
};
pub use types::*;
