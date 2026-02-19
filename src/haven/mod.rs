//! Haven base building system — account-level skill tree.
//!
//! The Haven persists across all prestige resets and benefits every character.
//! Players spend prestige ranks and fishing ranks to build and upgrade rooms.

pub mod logic;
pub mod room_defs;
pub mod types;

pub use logic::*;
pub use room_defs::{tier_cost, HavenRoomId};
pub use types::*;
