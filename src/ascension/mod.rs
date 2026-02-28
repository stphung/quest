//! Ascension system — per-character combat power multiplier gated by Deep milestones.

pub mod logic;
pub mod types;

// Re-exports consumed by tick pipeline (Task 14), achievements (Task 13), and UI (Task 17)
#[allow(unused_imports)]
pub use logic::{ascend, can_ascend, AscendResult};
#[allow(unused_imports)]
pub use types::{ascension_combat_multiplier, ascension_cost, ascension_deep_gate};
