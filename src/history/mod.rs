//! Git-based save history system.
//!
//! Every meaningful game event (level up, prestige, boss defeat, etc.) creates
//! a git commit containing the full save state. Players can browse the timeline
//! and restore any previous snapshot.

pub mod git;
pub mod types;

pub use git::HistoryError;
pub use git::HistoryRepo;
pub use types::*;
