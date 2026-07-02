//! Zone progression state and logic.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::data::get_zone;
pub use crate::core::constants::KILLS_FOR_BOSS;

/// Tracks the player's progression through zones and subzones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneProgression {
    /// Current zone ID (1-10)
    pub current_zone_id: u32,
    /// Current subzone ID within the zone
    pub current_subzone_id: u32,
    /// Set of defeated bosses as (zone_id, subzone_id) pairs
    pub defeated_bosses: BTreeSet<(u32, u32)>,
    /// Set of unlocked zone IDs
    pub unlocked_zones: BTreeSet<u32>,
    /// Kills in current subzone (resets when boss spawns or subzone changes)
    #[serde(default)]
    pub kills_in_subzone: u32,
    /// Whether currently fighting a subzone boss
    #[serde(default)]
    pub fighting_boss: bool,
    /// Whether player has forged Stormbreaker (required to defeat Zone 10 boss)
    #[serde(default)]
    pub has_stormbreaker: bool,
    /// Zone the player last death-loop retreated from (transient, like `consecutive_deaths`)
    #[serde(skip)]
    pub death_retreat_zone: Option<u32>,
    /// Consecutive death-loop retreats from `death_retreat_zone` (transient)
    #[serde(skip)]
    pub death_retreat_count: u32,
    /// Safe-zone boss cycles remaining before auto-advancing back into
    /// `death_retreat_zone` (transient)
    #[serde(skip)]
    pub frontier_cooldown_cycles: u32,
}

impl Default for ZoneProgression {
    fn default() -> Self {
        Self::new()
    }
}

impl ZoneProgression {
    /// Creates a new zone progression starting in Zone 1, Subzone 1.
    pub fn new() -> Self {
        Self {
            current_zone_id: 1,
            current_subzone_id: 1,
            defeated_bosses: BTreeSet::new(),
            unlocked_zones: [1, 2].into_iter().collect(), // Start with zones 1-2 unlocked (P0 zones)
            kills_in_subzone: 0,
            fighting_boss: false,
            has_stormbreaker: false, // Must be forged to defeat Zone 10 boss
            death_retreat_zone: None,
            death_retreat_count: 0,
            frontier_cooldown_cycles: 0,
        }
    }

    /// Records a death-loop retreat from the current zone.
    ///
    /// Repeated retreats from the same zone grow the frontier cooldown
    /// (capped), so the player spends progressively longer in the safe zone
    /// before auto-advancing back into the zone that keeps killing them.
    pub fn record_death_retreat(&mut self) {
        let zone = self.current_zone_id;
        if self.death_retreat_zone == Some(zone) {
            self.death_retreat_count += 1;
        } else {
            self.death_retreat_zone = Some(zone);
            self.death_retreat_count = 1;
        }
        self.frontier_cooldown_cycles = self
            .death_retreat_count
            .min(crate::core::constants::FRONTIER_BACKOFF_MAX_CYCLES);
    }

    /// Returns true if auto-advancement into `zone_id` is blocked by frontier backoff.
    pub fn frontier_backoff_blocks(&self, zone_id: u32) -> bool {
        self.frontier_cooldown_cycles > 0 && self.death_retreat_zone == Some(zone_id)
    }

    /// Clears death-retreat memory (called when the player proves they can
    /// survive the zone, or on prestige reset).
    pub fn clear_death_retreat(&mut self) {
        self.death_retreat_zone = None;
        self.death_retreat_count = 0;
        self.frontier_cooldown_cycles = 0;
    }

    /// Records a kill in the current subzone. Returns true if boss should spawn.
    pub fn record_kill(&mut self) -> bool {
        if self.fighting_boss {
            return false; // Already fighting boss
        }

        self.kills_in_subzone += 1;

        if self.kills_in_subzone >= KILLS_FOR_BOSS {
            self.fighting_boss = true;
            true
        } else {
            false
        }
    }

    /// Returns true if boss should be spawned (enough kills and not already fighting)
    pub fn should_spawn_boss(&self) -> bool {
        self.kills_in_subzone >= KILLS_FOR_BOSS && !self.fighting_boss
    }

    /// Returns kills remaining until boss spawns
    pub fn kills_until_boss(&self) -> u32 {
        if self.fighting_boss {
            0
        } else {
            KILLS_FOR_BOSS.saturating_sub(self.kills_in_subzone)
        }
    }

    /// Unlocks a zone.
    pub fn unlock_zone(&mut self, zone_id: u32) {
        self.unlocked_zones.insert(zone_id);
    }

    /// Records a boss defeat.
    pub fn defeat_boss(&mut self, zone_id: u32, subzone_id: u32) {
        self.defeated_bosses.insert((zone_id, subzone_id));
        // Reset kill counter and boss flag
        self.kills_in_subzone = 0;
        self.fighting_boss = false;
    }

    /// Gets the current zone and subzone names.
    /// Uses O(1) direct zone indexing and returns static string references (zero allocation).
    pub fn current_location_names(&self) -> (&'static str, &'static str) {
        if let Some(zone) = get_zone(self.current_zone_id) {
            if let Some(subzone) = zone
                .subzones
                .iter()
                .find(|s| s.id == self.current_subzone_id)
            {
                return (zone.name, subzone.name);
            }
        }
        ("Unknown", "Unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_progression_default() {
        let prog = ZoneProgression::new();
        assert_eq!(prog.current_zone_id, 1);
        assert_eq!(prog.current_subzone_id, 1);
        assert!(prog.is_zone_unlocked(1));
        assert!(prog.is_zone_unlocked(2));
        assert!(!prog.is_zone_unlocked(3));
    }

    #[test]
    fn test_boss_defeat_tracking() {
        let mut prog = ZoneProgression::new();

        assert!(!prog.is_boss_defeated(1, 1));
        prog.defeat_boss(1, 1);
        assert!(prog.is_boss_defeated(1, 1));

        // Defeating same boss again should not duplicate
        prog.defeat_boss(1, 1);
        assert_eq!(
            prog.defeated_bosses
                .iter()
                .filter(|&&b| b == (1, 1))
                .count(),
            1 // BTreeSet naturally deduplicates
        );
    }

    #[test]
    fn test_death_retreat_tracking() {
        let mut prog = ZoneProgression::new();
        assert_eq!(prog.death_retreat_zone, None);
        assert!(!prog.frontier_backoff_blocks(1));

        // Repeated retreats from the same zone grow the cooldown
        prog.current_zone_id = 42;
        prog.record_death_retreat();
        assert_eq!(prog.death_retreat_zone, Some(42));
        assert_eq!(prog.death_retreat_count, 1);
        assert_eq!(prog.frontier_cooldown_cycles, 1);
        assert!(prog.frontier_backoff_blocks(42));
        assert!(!prog.frontier_backoff_blocks(41));

        prog.record_death_retreat();
        assert_eq!(prog.death_retreat_count, 2);
        assert_eq!(prog.frontier_cooldown_cycles, 2);

        // Cooldown is capped
        for _ in 0..20 {
            prog.record_death_retreat();
        }
        assert_eq!(
            prog.frontier_cooldown_cycles,
            crate::core::constants::FRONTIER_BACKOFF_MAX_CYCLES
        );

        // Retreating from a different zone resets the count
        prog.current_zone_id = 30;
        prog.record_death_retreat();
        assert_eq!(prog.death_retreat_zone, Some(30));
        assert_eq!(prog.death_retreat_count, 1);
        assert_eq!(prog.frontier_cooldown_cycles, 1);
        assert!(!prog.frontier_backoff_blocks(42));

        prog.clear_death_retreat();
        assert_eq!(prog.death_retreat_zone, None);
        assert_eq!(prog.frontier_cooldown_cycles, 0);
    }

    #[test]
    fn test_death_retreat_state_is_transient() {
        let mut prog = ZoneProgression::new();
        prog.current_zone_id = 42;
        prog.record_death_retreat();

        let json = serde_json::to_string(&prog).unwrap();
        let loaded: ZoneProgression = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.death_retreat_zone, None);
        assert_eq!(loaded.death_retreat_count, 0);
        assert_eq!(loaded.frontier_cooldown_cycles, 0);
    }

    #[test]
    fn test_current_location_names() {
        let prog = ZoneProgression::new();
        let (zone, subzone) = prog.current_location_names();
        assert_eq!(zone, "Meadow");
        assert_eq!(subzone, "Sunny Fields");
    }

    #[test]
    fn test_kill_tracking() {
        let mut prog = ZoneProgression::new();

        // Initial state
        assert_eq!(prog.kills_in_subzone, 0);
        assert!(!prog.fighting_boss);
        assert_eq!(prog.kills_until_boss(), KILLS_FOR_BOSS);

        // Record kills
        for i in 1..KILLS_FOR_BOSS {
            let boss_spawns = prog.record_kill();
            assert!(!boss_spawns);
            assert_eq!(prog.kills_in_subzone, i);
            assert_eq!(prog.kills_until_boss(), KILLS_FOR_BOSS - i);
        }

        // Final kill triggers boss
        let boss_spawns = prog.record_kill();
        assert!(boss_spawns);
        assert!(prog.fighting_boss);
        assert_eq!(prog.kills_until_boss(), 0);
    }

    #[test]
    fn test_record_kill_during_boss_fight() {
        let mut prog = ZoneProgression::new();

        // Get to boss
        for _ in 0..KILLS_FOR_BOSS {
            prog.record_kill();
        }
        assert!(prog.fighting_boss);

        // Recording kills during boss fight should not increment
        let boss_spawns = prog.record_kill();
        assert!(!boss_spawns);
        assert_eq!(prog.kills_in_subzone, KILLS_FOR_BOSS);
    }

    #[test]
    fn test_should_spawn_boss_at_exactly_10_kills() {
        let mut prog = ZoneProgression::new();

        // Record 9 kills - should not spawn
        for _ in 0..9 {
            prog.record_kill();
        }
        assert!(!prog.should_spawn_boss());
        assert!(!prog.fighting_boss);

        // 10th kill triggers boss
        prog.record_kill();
        // After record_kill sets fighting_boss=true, should_spawn_boss returns false
        // because the condition is kills >= KILLS_FOR_BOSS && !fighting_boss
        assert!(!prog.should_spawn_boss());
        assert!(prog.fighting_boss);
        assert_eq!(prog.kills_in_subzone, KILLS_FOR_BOSS);
    }
}
