//! Core game state and logic.

#![allow(unused_imports)]

pub mod constants;
pub mod discoveries;
pub mod discovery_facade;
pub mod enemy_spawning;
pub mod game_logic;
pub mod game_state;
pub mod offline;
pub mod paths;
pub mod recent_drops;
pub mod tick;
pub mod tick_stages;
pub mod tick_types;
pub mod ticker;
pub mod xp;

pub mod power_rating;

pub mod combat_context;
pub mod game_state_serde;
pub mod player_identity;
pub mod progression_state;
pub mod session_state;
pub mod tick_context;

pub use constants::*;
pub use game_logic::*;
pub use game_state::*;
pub use tick_types::{TickEvent, TickResult};
