//! Zone and subzone progression system.
//!
//! The zone system provides a sense of traveling through themed areas,
//! with boss gates between subzones and prestige gates between zone tiers.

pub mod access;
pub mod advancement;
pub mod boss_defeat;
mod data;
pub mod gates;
pub mod postgame;
mod progression;

pub use boss_defeat::BossDefeatResult;
pub use data::{get_all_zones, get_zone, Subzone, Zone};
pub use access::sync_account_zone_unlocks;
pub use postgame::PostgameRegion;
pub use progression::ZoneProgression;
