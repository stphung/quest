//! Zone progression state and logic.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use super::data::{get_all_zones, Zone};
use crate::achievements::{AchievementId, Achievements};
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

    /// Checks if the current boss requires a weapon the player doesn't have.
    /// Returns Some(weapon_name) if blocked, None if can proceed.
    ///
    /// Uses the TheStormbreaker achievement to check if the player has forged Stormbreaker.
    pub fn boss_weapon_blocked(&self, achievements: &Achievements) -> Option<&'static str> {
        if !self.fighting_boss {
            return None;
        }

        let zones = get_all_zones();
        let zone = zones.iter().find(|z| z.id == self.current_zone_id)?;

        // Only the zone's final boss requires the weapon
        let is_zone_boss = self.current_subzone_id == zone.subzones.len() as u32;
        // Check achievement instead of has_stormbreaker flag
        let has_stormbreaker = achievements.is_unlocked(AchievementId::TheStormbreaker);
        let needs_weapon = zone.requires_weapon && is_zone_boss && !has_stormbreaker;

        if needs_weapon {
            zone.weapon_name
        } else {
            None
        }
    }

    /// Checks if a boss has been defeated.
    pub fn is_boss_defeated(&self, zone_id: u32, subzone_id: u32) -> bool {
        self.defeated_bosses.contains(&(zone_id, subzone_id))
    }

    /// Checks if a zone is unlocked.
    pub fn is_zone_unlocked(&self, zone_id: u32) -> bool {
        self.unlocked_zones.contains(&zone_id)
    }

    /// Checks if a zone can be unlocked based on prestige rank.
    pub fn can_unlock_zone(&self, zone: &Zone, prestige_rank: u32) -> bool {
        // Check prestige requirement
        if prestige_rank < zone.prestige_requirement {
            return false;
        }

        // Check if previous zone's final boss is defeated (if not first zone)
        if zone.id > 1 {
            let prev_zone_id = zone.id - 1;
            if let Some(prev_zone) = get_all_zones().iter().find(|z| z.id == prev_zone_id) {
                let last_subzone_id = prev_zone.subzones.len() as u32;
                if !self.is_boss_defeated(prev_zone_id, last_subzone_id) {
                    return false;
                }
            }
        }

        true
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

    /// Checks if the player can enter a specific subzone.
    pub fn can_enter_subzone(&self, zone_id: u32, subzone_id: u32) -> bool {
        // Zone must be unlocked
        if !self.is_zone_unlocked(zone_id) {
            return false;
        }

        // First subzone is always accessible if zone is unlocked
        if subzone_id == 1 {
            return true;
        }

        // Need previous subzone's boss defeated
        self.is_boss_defeated(zone_id, subzone_id - 1)
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
    use crate::achievements::Achievements;

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
    fn test_subzone_access() {
        let mut prog = ZoneProgression::new();

        // Can enter first subzone
        assert!(prog.can_enter_subzone(1, 1));

        // Cannot enter second subzone without defeating first boss
        assert!(!prog.can_enter_subzone(1, 2));

        // Defeat first boss
        prog.defeat_boss(1, 1);

        // Now can enter second subzone
        assert!(prog.can_enter_subzone(1, 2));
    }

    #[test]
    fn test_zone_unlock_prestige_gate() {
        let prog = ZoneProgression::new();
        let zones = get_all_zones();

        // Zone 3 requires prestige 5
        assert!(!prog.can_unlock_zone(&zones[2], 0));
        assert!(!prog.can_unlock_zone(&zones[2], 4));
        // Note: Also needs zone 2's boss defeated
    }

    #[test]
    fn test_zone_unlock_boss_gate() {
        let mut prog = ZoneProgression::new();
        let zones = get_all_zones();

        // Zone 3 requires P5 AND zone 2's final boss defeated
        // With P5 but no boss defeated
        assert!(!prog.can_unlock_zone(&zones[2], 5));

        // Defeat zone 2's bosses
        prog.defeat_boss(2, 1);
        prog.defeat_boss(2, 2);
        prog.defeat_boss(2, 3);

        // Now should be able to unlock
        assert!(prog.can_unlock_zone(&zones[2], 5));
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

    // =========================================================================
    // WEAPON GATE QUERY TESTS (boss_weapon_blocked)
    // =========================================================================

    #[test]
    fn test_weapon_gate_blocks_zone_10_final_boss_without_stormbreaker() {
        let mut prog = ZoneProgression::new();
        let achievements = Achievements::default();

        prog.current_zone_id = 10;
        prog.current_subzone_id = 4;
        prog.unlock_zone(10);
        prog.fighting_boss = true;

        let blocked = prog.boss_weapon_blocked(&achievements);
        assert_eq!(blocked, Some("Stormbreaker"));
    }

    #[test]
    fn test_weapon_gate_allows_zone_10_final_boss_with_stormbreaker() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::TheStormbreaker, None);

        prog.current_zone_id = 10;
        prog.current_subzone_id = 4;
        prog.unlock_zone(10);
        prog.fighting_boss = true;

        let blocked = prog.boss_weapon_blocked(&achievements);
        assert!(blocked.is_none());
    }

    #[test]
    fn test_weapon_gate_does_not_apply_to_non_final_subzone() {
        let mut prog = ZoneProgression::new();
        let achievements = Achievements::default();

        // Zone 10, subzone 3 (not the final subzone 4), no stormbreaker
        prog.current_zone_id = 10;
        prog.current_subzone_id = 3;
        prog.unlock_zone(10);
        prog.fighting_boss = true;

        let blocked = prog.boss_weapon_blocked(&achievements);
        assert!(blocked.is_none());
    }

    #[test]
    fn test_weapon_gate_does_not_apply_to_other_zones() {
        let mut prog = ZoneProgression::new();
        let achievements = Achievements::default();

        // Zone 5, final subzone (4), no stormbreaker - should not be blocked
        prog.current_zone_id = 5;
        prog.current_subzone_id = 4;
        prog.unlock_zone(5);
        prog.fighting_boss = true;

        let blocked = prog.boss_weapon_blocked(&achievements);
        assert!(blocked.is_none());
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
