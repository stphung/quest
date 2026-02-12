//! Go (Territory Control) minigame.

#![allow(unused_imports)]

pub mod logic;
pub mod mcts;
pub mod types;

pub use logic::{
    apply_go_result, calculate_score, get_legal_moves, is_legal_move, make_move,
    process_ai_thinking, process_human_move, process_human_pass, process_input, GoInput,
};
pub use types::*;
