//! Zone progression state and logic.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use super::data::get_all_zones;
pub use crate::core::constants::KILLS_FOR_BOSS;

/// Tracks the player's progression through zones and subzones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneProgression {
    /// Current zone ID (1-10)
    pub current_zone_id: u32,
    /// Current subzone ID within the zone
    pub current_subzone_id: u32,
    /// List of defeated bosses as (zone_id, subzone_id) pairs
    pub defeated_bosses: Vec<(u32, u32)>,
    /// List of unlocked zone IDs
    pub unlocked_zones: Vec<u32>,
    /// Kills in current subzone (resets when boss spawns or subzone changes)
    #[serde(default)]
    pub kills_in_subzone: u32,
    /// Whether currently fighting a subzone boss
    #[serde(default)]
    pub fighting_boss: bool,
    /// Whether player has forged Stormbreaker (required to defeat Zone 10 boss)
    #[serde(default)]
    pub has_stormbreaker: bool,
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
            defeated_bosses: vec![],
            unlocked_zones: vec![1, 2], // Start with zones 1-2 unlocked (P0 zones)
            kills_in_subzone: 0,
            fighting_boss: false,
            has_stormbreaker: false, // Must be forged to defeat Zone 10 boss
        }
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
        if !self.unlocked_zones.contains(&zone_id) {
            self.unlocked_zones.push(zone_id);
            self.unlocked_zones.sort();
        }
    }

    /// Records a boss defeat.
    pub fn defeat_boss(&mut self, zone_id: u32, subzone_id: u32) {
        if !self.is_boss_defeated(zone_id, subzone_id) {
            self.defeated_bosses.push((zone_id, subzone_id));
        }
        // Reset kill counter and boss flag
        self.kills_in_subzone = 0;
        self.fighting_boss = false;
    }

    /// Gets the current zone and subzone names.
    pub fn current_location_names(&self) -> (String, String) {
        let zones = get_all_zones();
        if let Some(zone) = zones.iter().find(|z| z.id == self.current_zone_id) {
            if let Some(subzone) = zone
                .subzones
                .iter()
                .find(|s| s.id == self.current_subzone_id)
            {
                return (zone.name.to_string(), subzone.name.to_string());
            }
        }
        ("Unknown".to_string(), "Unknown".to_string())
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
            1
        );
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
