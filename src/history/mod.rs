//! Git-based save history system.
//!
//! Every meaningful game event (level up, prestige, boss defeat, etc.) creates
//! a git commit containing the full save state. Players can browse the timeline
//! and restore any previous snapshot.

// Many types and methods are defined now but only used by later tasks
// (timeline browser UI, restore, branch switching). Suppress dead_code
// warnings until those tasks land.
#![allow(dead_code, unused_imports)]

pub mod git;
pub mod types;

pub use git::HistoryError;
pub use git::HistoryRepo;
pub use types::*;
