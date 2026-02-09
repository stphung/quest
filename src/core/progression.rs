//! Shared progression trait for game and simulator.
//!
//! This trait defines the core progression interface that both
//! the real game (ZoneProgression) and simulator (SimProgression) implement.

// Allow dead code - trait and helpers are being integrated incrementally
#![allow(dead_code)]

use crate::core::balance::KILLS_PER_BOSS;

/// Core progression state and operations.
///
/// Implemented by both ZoneProgression (game) and SimProgression (simulator).
pub trait Progression {
    // === State Accessors ===

    /// Current zone ID (1-10).
    fn current_zone(&self) -> u32;

    /// Current subzone ID within the zone.
    fn current_subzone(&self) -> u32;

    /// Kills accumulated in current subzone.
    fn kills_in_subzone(&self) -> u32;

    /// Current prestige rank.
    fn prestige_rank(&self) -> u32;

    // === Combat Tracking ===

    /// Record a kill. Returns true if boss should spawn.
    fn record_kill(&mut self) -> bool;

    /// Record a death.
    /// If `was_boss_fight` is true, resets kill progress (matches real game).
    fn record_death(&mut self, was_boss_fight: bool);

    // === Boss Logic ===

    /// Check if boss should spawn (reached kill threshold).
    fn should_spawn_boss(&self) -> bool {
        self.kills_in_subzone() >= KILLS_PER_BOSS
    }

    /// Kills remaining until boss spawns.
    fn kills_until_boss(&self) -> u32 {
        KILLS_PER_BOSS.saturating_sub(self.kills_in_subzone())
    }

    // === Zone Advancement ===

    /// Called after defeating a boss. Advances subzone or zone.
    fn advance_after_boss(&mut self);

    /// Check if at the maximum zone for current prestige rank.
    fn at_max_zone_for_prestige(&self) -> bool {
        max_zone_for_prestige(self.prestige_rank()) <= self.current_zone()
    }
}

/// Returns the maximum zone accessible at a given prestige rank.
///
/// Zone requirements:
/// - Zone 1-2: P0
/// - Zone 3-4: P5  
/// - Zone 5-6: P10
/// - Zone 7-8: P15
/// - Zone 9-10: P20
pub fn max_zone_for_prestige(prestige_rank: u32) -> u32 {
    match prestige_rank {
        0..=4 => 2,
        5..=9 => 4,
        10..=14 => 6,
        15..=19 => 8,
        _ => 10,
    }
}

/// Returns the minimum prestige rank required for a zone.
pub fn prestige_required_for_zone(zone_id: u32) -> u32 {
    match zone_id {
        1..=2 => 0,
        3..=4 => 5,
        5..=6 => 10,
        7..=8 => 15,
        9..=10 => 20,
        _ => 0, // Zone 11+ has special unlock rules
    }
}

/// Check if a prestige rank allows access to a zone.
pub fn can_access_zone(prestige_rank: u32, zone_id: u32) -> bool {
    prestige_rank >= prestige_required_for_zone(zone_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_zone_for_prestige() {
        assert_eq!(max_zone_for_prestige(0), 2);
        assert_eq!(max_zone_for_prestige(4), 2);
        assert_eq!(max_zone_for_prestige(5), 4);
        assert_eq!(max_zone_for_prestige(10), 6);
        assert_eq!(max_zone_for_prestige(15), 8);
        assert_eq!(max_zone_for_prestige(20), 10);
        assert_eq!(max_zone_for_prestige(25), 10);
    }

    #[test]
    fn test_prestige_required_for_zone() {
        assert_eq!(prestige_required_for_zone(1), 0);
        assert_eq!(prestige_required_for_zone(2), 0);
        assert_eq!(prestige_required_for_zone(3), 5);
        assert_eq!(prestige_required_for_zone(4), 5);
        assert_eq!(prestige_required_for_zone(5), 10);
        assert_eq!(prestige_required_for_zone(10), 20);
    }

    #[test]
    fn test_can_access_zone() {
        assert!(can_access_zone(0, 1));
        assert!(can_access_zone(0, 2));
        assert!(!can_access_zone(0, 3));
        assert!(can_access_zone(5, 3));
        assert!(can_access_zone(5, 4));
        assert!(!can_access_zone(5, 5));
        assert!(can_access_zone(20, 10));
    }
}
