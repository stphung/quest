//! Core game state and logic.

#![allow(unused_imports)]

pub mod balance;
pub mod constants;
pub mod game_logic;
pub mod game_state;
pub mod progression;

// Re-export selectively to avoid ambiguity
pub use constants::*;
pub use game_logic::*;
pub use game_state::*;
// balance module accessed via crate::core::balance::
// progression module accessed via crate::core::progression::
