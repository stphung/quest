//! The Deep — mercenary expedition system.
//!
//! An endgame (P15+) account-level system where players recruit and manage a
//! mercenary company, sending squads on long-duration wall-clock missions that
//! push deeper into a vast underground structure called The Deep.
//!
//! ## Persistence
//! - Persists across prestiges: guild_rank, layers (cleared, infrastructure, familiarity)
//! - Resets on prestige: mercenaries, active_missions, warband_marks

pub mod clock;
pub mod discovery;
pub mod events;
pub mod guild;
pub mod infrastructure;
pub mod layers;
pub mod mercenaries;
pub mod missions;
pub mod persistence;
pub mod types;
pub mod ui_state;

#[allow(unused_imports)]
pub use types::*;
#[allow(unused_imports)]
pub use ui_state::*;
