//! Ascension system — per-character combat power multiplier gated by Deep milestones.

pub mod logic;
pub mod types;

pub use logic::{ascend, can_ascend, AscendResult};
pub use types::{ascension_combat_multiplier, ascension_cost, ascension_deep_gate};
