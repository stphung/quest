//! Zone access gate queries (weapon gates, subzone access, zone unlock checks).

#![allow(dead_code)]

use super::data::Zone;
use super::progression::ZoneProgression;
use crate::achievements::{AchievementId, Achievements};

impl ZoneProgression {
    /// Checks if the current boss requires a weapon the player doesn't have.
    /// Returns Some(weapon_name) if blocked, None if can proceed.
    ///
    /// Uses the TheStormbreaker achievement to check if the player has forged Stormbreaker.
    pub fn boss_weapon_blocked(&self, achievements: &Achievements) -> Option<&'static str> {
        if !self.fighting_boss {
            return None;
        }

        let zone = super::data::get_zone(self.current_zone_id)?;

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
            if let Some(prev_zone) = super::data::get_zone(prev_zone_id) {
                let last_subzone_id = prev_zone.subzones.len() as u32;
                if !self.is_boss_defeated(prev_zone_id, last_subzone_id) {
                    return false;
                }
            }
        }

        true
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
}

#[cfg(test)]
mod tests {
    use crate::achievements::{AchievementId, Achievements};
    use crate::zones::data::get_all_zones;
    use crate::zones::ZoneProgression;

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
    fn test_weapon_gate_returns_none_when_not_fighting_boss() {
        let mut prog = ZoneProgression::new();
        let achievements = Achievements::default();

        // At zone 10 final subzone but not fighting boss
        prog.current_zone_id = 10;
        prog.current_subzone_id = 4;
        prog.unlock_zone(10);
        prog.fighting_boss = false;

        assert!(prog.boss_weapon_blocked(&achievements).is_none());
    }

    #[test]
    fn test_can_enter_subzone_locked_zone() {
        let prog = ZoneProgression::new();

        // Zone 5 is not unlocked at P0
        assert!(!prog.can_enter_subzone(5, 1));
    }

    #[test]
    fn test_can_unlock_zone_1_always_true() {
        let prog = ZoneProgression::new();
        let zones = get_all_zones();

        // Zone 1 has no previous zone requirement and P0 prestige
        assert!(prog.can_unlock_zone(&zones[0], 0));
    }

    #[test]
    fn test_can_unlock_zone_exact_prestige_boundary() {
        let mut prog = ZoneProgression::new();
        let zones = get_all_zones();

        // Defeat zone 2's bosses to satisfy boss gate
        prog.defeat_boss(2, 1);
        prog.defeat_boss(2, 2);
        prog.defeat_boss(2, 3);

        // Zone 3 requires exactly P5
        assert!(!prog.can_unlock_zone(&zones[2], 4));
        assert!(prog.can_unlock_zone(&zones[2], 5));
        assert!(prog.can_unlock_zone(&zones[2], 6));
    }
}
