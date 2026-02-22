//! The Deep — mercenary expedition system.
//!
//! An endgame (P15+) account-level system where players recruit and manage a
//! mercenary company, sending squads on long-duration wall-clock missions that
//! push deeper into a vast underground structure called The Deep.
//!
//! ## Persistence
//! - Persists across prestiges: guild_rank, layers (cleared, infrastructure, familiarity)
//! - Resets on prestige: mercenaries, active_missions, warband_marks

pub mod types;

pub use types::*;
