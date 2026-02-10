//! Core game state and logic.

#![allow(unused_imports)]

pub mod balance;
pub mod combat_math;
pub mod constants;
pub mod core_game;
pub mod game_logic;
pub mod game_loop;
pub mod game_state;
pub mod progression;

// Re-export selectively to avoid ambiguity
pub use constants::*;
pub use game_logic::*;
pub use game_state::*;
// balance module accessed via crate::core::balance::
// progression module accessed via crate::core::progression::
// combat_math module accessed via crate::core::combat_math::
