//! Zone and subzone progression system.
//!
//! The zone system provides a sense of traveling through themed areas,
//! with boss gates between subzones and prestige gates between zone tiers.

mod data;
mod progression;

#[allow(unused_imports)]
pub use data::*;
#[allow(unused_imports)]
pub use progression::{BossDefeatResult, ZoneProgression, KILLS_FOR_BOSS};
