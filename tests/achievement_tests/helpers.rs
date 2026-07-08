//! Shared helpers for achievement handler tests.

use quest::haven::types::HavenRoomId;
use std::collections::HashMap;

/// Build a HashMap with all buildable rooms (excluding StormForge) set to `tier`.
pub fn build_haven_tiers(tier: u8) -> HashMap<HavenRoomId, u8> {
    HavenRoomId::ALL
        .iter()
        .filter(|r| **r != HavenRoomId::StormForge)
        .map(|r| (*r, tier))
        .collect()
}

/// Build a HashMap with every room at its max tier.
pub fn build_haven_max_tiers() -> HashMap<HavenRoomId, u8> {
    HavenRoomId::ALL
        .iter()
        .map(|r| (*r, r.max_tier()))
        .collect()
}
