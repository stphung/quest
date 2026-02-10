//! Core game state and logic.
//!
//! This module contains the core game engine components:
//!
//! - **game_state**: The complete game state (GameState struct)
//! - **game_logic**: XP calculations, level-ups, enemy spawning
//! - **game_loop**: The GameLoop trait and TickResult for simulation
//! - **core_game**: CoreGame struct for simulation, resolve_combat_tick for interactive
//! - **combat_math**: Damage calculations shared between engines
//! - **balance**: Game balance constants
//! - **progression**: Zone unlock and prestige requirements

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

// Re-export core game components for interactive game use
pub use core_game::{resolve_combat_tick, CombatBonuses, CoreGame};
pub use game_loop::{GameLoop, TickResult};
// balance module accessed via crate::core::balance::
// progression module accessed via crate::core::progression::
// combat_math module accessed via crate::core::combat_math::
