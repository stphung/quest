//! Stormglass currency system.
//!
//! Stormglass is a character-level currency earned through gameplay and spent
//! at the Stormglass Exchange overlay.

pub mod earning;
#[allow(dead_code)] // Consumed incrementally by later tasks (UI, input, tick injection)
pub mod sigils;
pub mod spending;
pub mod types;
