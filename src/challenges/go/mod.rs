//! Go (Territory Control) minigame.

#![allow(unused_imports)]

pub mod heuristics;
pub mod logic;
pub mod mcts;
pub mod types;

pub use logic::{
    apply_go_result, calculate_score, count_liberties, get_group, get_legal_moves, is_legal_move,
    make_move, process_go_ai, process_human_move, process_human_pass, process_input,
    start_go_game, would_be_captured, GoInput,
};
pub use types::*;
