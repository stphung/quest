//! Flappy Bird challenge minigame.
//!
//! A real-time action challenge where the player navigates a bird through
//! scrolling pipe obstacles by pressing a key to flap. Gravity pulls the
//! bird down each tick, and hitting a pipe or the floor ends the game.

pub mod logic;
pub mod types;

pub use logic::*;
pub use types::*;
