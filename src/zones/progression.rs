//! Zone progression state and logic.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use super::data::{get_all_zones, Zone};
use crate::achievements::{AchievementId, Achievements};
pub use crate::core::constants::KILLS_FOR_BOSS;
use crate::core::constants::{EXPANSE_ZONE_ID, FINAL_ZONE_ID};

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
            if let Some(prev_zone) = get_all_zones().into_iter().find(|z| z.id == prev_zone_id) {
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

    /// Handles boss defeat for the current subzone and auto-advances.
    /// Returns a description of what happened (for UI feedback).
    ///
    /// Uses achievements to check for Stormbreaker and to unlock StormsEnd.
    pub fn on_boss_defeated(
        &mut self,
        prestige_rank: u32,
        achievements: &mut Achievements,
    ) -> BossDefeatResult {
        let zone_id = self.current_zone_id;
        let subzone_id = self.current_subzone_id;

        let zones = get_all_zones();
        let Some(zone) = zones.iter().find(|z| z.id == zone_id) else {
            return BossDefeatResult::SubzoneComplete {
                new_subzone_id: self.current_subzone_id,
            };
        };

        let is_zone_boss = subzone_id == zone.subzones.len() as u32;

        // Check for Zone 10 final boss weapon requirement (use achievement)
        let has_stormbreaker = achievements.is_unlocked(AchievementId::TheStormbreaker);
        if zone.requires_weapon && is_zone_boss && !has_stormbreaker {
            // Can't defeat this boss without the weapon - boss survives!
            // Reset fighting state so player can try again (after getting weapon)
            self.fighting_boss = false;
            self.kills_in_subzone = 0;
            return BossDefeatResult::WeaponRequired {
                weapon_name: zone.weapon_name.unwrap_or("legendary weapon").to_string(),
            };
        }

        // Record the defeat
        self.defeat_boss(zone_id, subzone_id);

        // Special handling for The Expanse - infinite cycling
        if zone_id == EXPANSE_ZONE_ID && is_zone_boss {
            // Cycle back to subzone 1
            self.current_subzone_id = 1;
            self.kills_in_subzone = 0;
            return BossDefeatResult::ExpanseCycle;
        }

        if is_zone_boss {
            // Final zone completion triggers StormsEnd achievement and unlocks The Expanse
            if zone_id == FINAL_ZONE_ID {
                achievements.unlock(AchievementId::StormsEnd, None);
                // Unlock The Expanse and advance to it
                self.unlock_zone(EXPANSE_ZONE_ID);
                self.current_zone_id = EXPANSE_ZONE_ID;
                self.current_subzone_id = 1;
                return BossDefeatResult::StormsEnd;
            }

            // Try to advance to next zone
            if self.advance_to_next_zone(prestige_rank) {
                return BossDefeatResult::ZoneComplete {
                    old_zone: zone.name.to_string(),
                    new_zone_id: self.current_zone_id,
                };
            }

            // Can't advance - either no more zones or prestige-gated
            let next_zone = zones.iter().find(|z| z.id == zone_id + 1);
            if let Some(next) = next_zone {
                return BossDefeatResult::ZoneCompleteButGated {
                    zone_name: zone.name.to_string(),
                    required_prestige: next.prestige_requirement,
                };
            }
            return BossDefeatResult::StormsEnd;
        }

        // Advance to next subzone
        self.advance_to_next_subzone();
        BossDefeatResult::SubzoneComplete {
            new_subzone_id: self.current_subzone_id,
        }
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

    /// Advances to the next subzone within the current zone.
    /// Returns true if successful.
    pub fn advance_to_next_subzone(&mut self) -> bool {
        let zones = get_all_zones();
        let zone = zones.iter().find(|z| z.id == self.current_zone_id);

        if let Some(zone) = zone {
            let max_subzone = zone.subzones.len() as u32;
            if self.current_subzone_id < max_subzone {
                // Check if current boss is defeated
                if self.is_boss_defeated(self.current_zone_id, self.current_subzone_id) {
                    self.current_subzone_id += 1;
                    return true;
                }
            }
        }
        false
    }

    /// Advances to the next zone.
    /// Returns true if successful.
    pub fn advance_to_next_zone(&mut self, prestige_rank: u32) -> bool {
        let zones = get_all_zones();
        let next_zone_id = self.current_zone_id + 1;

        if let Some(next_zone) = zones.iter().find(|z| z.id == next_zone_id) {
            if self.can_unlock_zone(next_zone, prestige_rank) {
                self.unlock_zone(next_zone_id);
                self.current_zone_id = next_zone_id;
                self.current_subzone_id = 1;
                return true;
            }
        }
        false
    }

    /// Sets the current zone and subzone directly.
    /// Used when traveling to previously unlocked areas.
    pub fn travel_to(&mut self, zone_id: u32, subzone_id: u32) -> bool {
        if self.can_enter_subzone(zone_id, subzone_id) {
            self.current_zone_id = zone_id;
            self.current_subzone_id = subzone_id;
            return true;
        }
        false
    }

    /// Resets progression for a new prestige cycle.
    /// Keeps zones unlocked based on the new prestige rank.
    pub fn reset_for_prestige(&mut self, new_prestige_rank: u32) {
        // Reset position to start
        self.current_zone_id = 1;
        self.current_subzone_id = 1;

        // Reset kill tracking
        self.kills_in_subzone = 0;
        self.fighting_boss = false;

        // Clear defeated bosses
        self.defeated_bosses.clear();

        // Recalculate unlocked zones based on new prestige rank
        let zones = get_all_zones();
        self.unlocked_zones = zones
            .iter()
            .filter(|z| z.prestige_requirement <= new_prestige_rank)
            .map(|z| z.id)
            .collect();
        self.unlocked_zones.sort();
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

/// Result of defeating a boss
#[derive(Debug, Clone, PartialEq)]
pub enum BossDefeatResult {
    /// Moved to next subzone within same zone
    SubzoneComplete { new_subzone_id: u32 },
    /// Completed zone and moved to next zone
    ZoneComplete { old_zone: String, new_zone_id: u32 },
    /// Completed zone but next zone requires higher prestige
    ZoneCompleteButGated {
        zone_name: String,
        required_prestige: u32,
    },
    /// Completed the final zone (Zone 10)
    StormsEnd,
    /// Boss requires a legendary weapon to defeat (Zone 10)
    WeaponRequired { weapon_name: String },
    /// Completed a cycle of The Expanse (Zone 11) - returns to subzone 1
    ExpanseCycle,
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
    fn test_advance_subzone() {
        let mut prog = ZoneProgression::new();

        // Cannot advance without defeating boss
        assert!(!prog.advance_to_next_subzone());
        assert_eq!(prog.current_subzone_id, 1);

        // Defeat boss and advance
        prog.defeat_boss(1, 1);
        assert!(prog.advance_to_next_subzone());
        assert_eq!(prog.current_subzone_id, 2);
    }

    #[test]
    fn test_advance_zone() {
        let mut prog = ZoneProgression::new();

        // Defeat all zone 1 bosses
        prog.defeat_boss(1, 1);
        prog.defeat_boss(1, 2);
        prog.defeat_boss(1, 3);

        // Should advance to zone 2 (P0 requirement met)
        assert!(prog.advance_to_next_zone(0));
        assert_eq!(prog.current_zone_id, 2);
        assert_eq!(prog.current_subzone_id, 1);
    }

    #[test]
    fn test_advance_zone_prestige_blocked() {
        let mut prog = ZoneProgression::new();

        // Progress through zones 1 and 2
        for subzone in 1..=3 {
            prog.defeat_boss(1, subzone);
        }
        prog.advance_to_next_zone(0);

        for subzone in 1..=3 {
            prog.defeat_boss(2, subzone);
        }

        // Try to advance to zone 3 with P0 (needs P5)
        assert!(!prog.advance_to_next_zone(0));
        assert_eq!(prog.current_zone_id, 2);

        // With P5, should work
        assert!(prog.advance_to_next_zone(5));
        assert_eq!(prog.current_zone_id, 3);
    }

    #[test]
    fn test_reset_for_prestige() {
        let mut prog = ZoneProgression::new();

        // Make some progress
        prog.current_zone_id = 3;
        prog.current_subzone_id = 2;
        prog.defeat_boss(1, 1);
        prog.defeat_boss(1, 2);
        prog.unlock_zone(3);

        // Reset with P5
        prog.reset_for_prestige(5);

        // Should be back at start
        assert_eq!(prog.current_zone_id, 1);
        assert_eq!(prog.current_subzone_id, 1);
        assert!(prog.defeated_bosses.is_empty());

        // Should have zones 1-4 unlocked (P0 and P5 zones)
        assert!(prog.is_zone_unlocked(1));
        assert!(prog.is_zone_unlocked(2));
        assert!(prog.is_zone_unlocked(3));
        assert!(prog.is_zone_unlocked(4));
        assert!(!prog.is_zone_unlocked(5)); // Needs P10
    }

    #[test]
    fn test_travel_to() {
        let mut prog = ZoneProgression::new();

        // Can travel to unlocked zone's first subzone
        assert!(prog.travel_to(2, 1));
        assert_eq!(prog.current_zone_id, 2);
        assert_eq!(prog.current_subzone_id, 1);

        // Cannot travel to locked zone
        assert!(!prog.travel_to(3, 1));

        // Cannot travel to locked subzone
        assert!(!prog.travel_to(2, 2));

        // Defeat boss, then can travel
        prog.defeat_boss(2, 1);
        assert!(prog.travel_to(2, 2));
    }

    #[test]
    fn test_current_location_names() {
        let prog = ZoneProgression::new();
        let (zone, subzone) = prog.current_location_names();
        assert_eq!(zone, "Meadow");
        assert_eq!(subzone, "Sunny Fields");
    }

    #[test]
    fn test_full_progression_flow() {
        let mut prog = ZoneProgression::new();
        let _zones = get_all_zones(); // Used to verify zone data is available

        // === ZONE 1: Meadow (3 subzones) ===
        assert_eq!(prog.current_zone_id, 1);
        assert_eq!(prog.current_subzone_id, 1);

        // Clear all subzones in Zone 1
        for subzone_id in 1..=3 {
            assert!(prog.can_enter_subzone(1, subzone_id) || subzone_id == 1);
            prog.defeat_boss(1, subzone_id);
            if subzone_id < 3 {
                assert!(prog.advance_to_next_subzone());
            }
        }
        assert_eq!(prog.current_subzone_id, 3);

        // Advance to Zone 2 (no prestige required)
        assert!(prog.advance_to_next_zone(0));
        assert_eq!(prog.current_zone_id, 2);
        assert_eq!(prog.current_subzone_id, 1);

        // === ZONE 2: Dark Forest (3 subzones) ===
        for subzone_id in 1..=3 {
            prog.defeat_boss(2, subzone_id);
            if subzone_id < 3 {
                prog.advance_to_next_subzone();
            }
        }

        // Try to advance to Zone 3 - BLOCKED by P5 requirement
        assert!(!prog.advance_to_next_zone(0));
        assert!(!prog.advance_to_next_zone(4));
        assert_eq!(prog.current_zone_id, 2);

        // With P5, can advance to Zone 3
        assert!(prog.advance_to_next_zone(5));
        assert_eq!(prog.current_zone_id, 3);

        // === Simulate Prestige ===
        prog.reset_for_prestige(6); // Prestige to rank 6

        // Should be back at Zone 1, Subzone 1
        assert_eq!(prog.current_zone_id, 1);
        assert_eq!(prog.current_subzone_id, 1);

        // Bosses are reset
        assert!(!prog.is_boss_defeated(1, 1));
        assert!(!prog.is_boss_defeated(2, 3));

        // But zones 1-6 should be unlocked (P0 + P5 + P10 partial)
        assert!(prog.is_zone_unlocked(1));
        assert!(prog.is_zone_unlocked(2));
        assert!(prog.is_zone_unlocked(3));
        assert!(prog.is_zone_unlocked(4));
        // P10 zones not unlocked yet (need P10)
        assert!(!prog.is_zone_unlocked(5));
        assert!(!prog.is_zone_unlocked(6));

        // Can immediately travel to Zone 4 (P5 requirement met, zone unlocked)
        assert!(prog.travel_to(4, 1));
        assert_eq!(prog.current_zone_id, 4);
    }

    #[test]
    fn test_zone_10_is_endgame() {
        let zones = get_all_zones();
        let zone10 = &zones[9];

        // Zone 10 is the endgame zone requiring a weapon
        assert_eq!(zone10.id, 10);
        assert_eq!(zone10.name, "Storm Citadel");
        assert!(zone10.requires_weapon);
        assert_eq!(zone10.weapon_name, Some("Stormbreaker"));
        assert_eq!(zone10.prestige_requirement, 20);
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
    fn test_on_boss_defeated_advances_subzone() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Get to boss
        for _ in 0..KILLS_FOR_BOSS {
            prog.record_kill();
        }
        assert!(prog.fighting_boss);
        assert_eq!(prog.current_subzone_id, 1);

        // Defeat boss
        let result = prog.on_boss_defeated(0, &mut achievements);
        assert!(matches!(
            result,
            BossDefeatResult::SubzoneComplete { new_subzone_id: 2 }
        ));
        assert_eq!(prog.current_subzone_id, 2);
        assert!(!prog.fighting_boss);
        assert_eq!(prog.kills_in_subzone, 0);
    }

    #[test]
    fn test_on_boss_defeated_zone_complete() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Clear subzones 1 and 2
        for _subzone in 1..=2 {
            for _ in 0..KILLS_FOR_BOSS {
                prog.record_kill();
            }
            prog.on_boss_defeated(0, &mut achievements);
        }
        assert_eq!(prog.current_subzone_id, 3);

        // Clear subzone 3 (final subzone of zone 1)
        for _ in 0..KILLS_FOR_BOSS {
            prog.record_kill();
        }

        let result = prog.on_boss_defeated(0, &mut achievements);
        match result {
            BossDefeatResult::ZoneComplete {
                old_zone,
                new_zone_id,
            } => {
                assert_eq!(old_zone, "Meadow");
                assert_eq!(new_zone_id, 2);
            }
            _ => panic!("Expected ZoneComplete, got {:?}", result),
        }
        assert_eq!(prog.current_zone_id, 2);
        assert_eq!(prog.current_subzone_id, 1);
    }

    #[test]
    fn test_on_boss_defeated_prestige_gated() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Clear zone 1
        for _subzone in 1..=3 {
            for _ in 0..KILLS_FOR_BOSS {
                prog.record_kill();
            }
            prog.on_boss_defeated(0, &mut achievements);
        }
        assert_eq!(prog.current_zone_id, 2);

        // Clear zone 2
        for _subzone in 1..=3 {
            for _ in 0..KILLS_FOR_BOSS {
                prog.record_kill();
            }
            prog.on_boss_defeated(0, &mut achievements);
        }

        // Should be gated at zone 3 (needs P5)
        match prog.on_boss_defeated(0, &mut achievements) {
            BossDefeatResult::ZoneCompleteButGated {
                zone_name,
                required_prestige,
            } => {
                assert_eq!(zone_name, "Dark Forest");
                assert_eq!(required_prestige, 5);
            }
            _ => {
                // We might have already advanced, check if we're stuck
                assert_eq!(prog.current_zone_id, 2);
            }
        }
    }

    #[test]
    fn test_zone_10_boss_requires_stormbreaker() {
        use crate::achievements::AchievementId;

        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Simulate being at Zone 10, final subzone (4), fighting boss
        prog.current_zone_id = 10;
        prog.current_subzone_id = 4;
        prog.unlock_zone(10);
        prog.fighting_boss = true;

        // Try to defeat boss without Stormbreaker achievement
        assert!(!achievements.is_unlocked(AchievementId::TheStormbreaker));
        let result = prog.on_boss_defeated(20, &mut achievements);

        match result {
            BossDefeatResult::WeaponRequired { weapon_name } => {
                assert_eq!(weapon_name, "Stormbreaker");
            }
            _ => panic!("Expected WeaponRequired, got {:?}", result),
        }

        // Boss should NOT be defeated
        assert!(!prog.is_boss_defeated(10, 4));
        // Should be reset to fight again
        assert!(!prog.fighting_boss);
        assert_eq!(prog.kills_in_subzone, 0);

        // Now unlock Stormbreaker achievement and try again
        achievements.unlock(AchievementId::TheStormbreaker, None);
        prog.fighting_boss = true;

        let result = prog.on_boss_defeated(20, &mut achievements);

        // Should complete the game
        assert!(matches!(result, BossDefeatResult::StormsEnd));
        assert!(prog.is_boss_defeated(10, 4));
    }

    #[test]
    fn test_reset_for_prestige_clears_kill_tracking() {
        let mut prog = ZoneProgression::new();

        // Accumulate some kills and trigger boss
        for _ in 0..KILLS_FOR_BOSS {
            prog.record_kill();
        }
        assert_eq!(prog.kills_in_subzone, KILLS_FOR_BOSS);
        assert!(prog.fighting_boss);

        // Prestige reset
        prog.reset_for_prestige(1);

        assert_eq!(prog.kills_in_subzone, 0);
        assert!(!prog.fighting_boss);
        assert_eq!(prog.current_zone_id, 1);
        assert_eq!(prog.current_subzone_id, 1);
        assert!(prog.defeated_bosses.is_empty());
    }
}
