//! Boss defeat handling and result types.

use super::data::get_all_zones;
use super::progression::ZoneProgression;
use crate::achievements::{AchievementId, Achievements};
use crate::core::constants::{EXPANSE_ZONE_ID, FINAL_ZONE_ID};

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

impl ZoneProgression {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::Achievements;
    use crate::core::constants::KILLS_FOR_BOSS;
    use crate::zones::data::get_all_zones;

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

    // =========================================================================
    // ZONE 11 EXPANSE CYCLING TESTS
    // =========================================================================

    #[test]
    fn test_zone_11_expanse_boss_cycles_back() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Set up at Zone 11, final subzone (4), fighting boss
        prog.current_zone_id = EXPANSE_ZONE_ID;
        prog.current_subzone_id = 4;
        prog.unlock_zone(EXPANSE_ZONE_ID);
        prog.fighting_boss = true;

        let result = prog.on_boss_defeated(20, &mut achievements);
        assert_eq!(result, BossDefeatResult::ExpanseCycle);
        assert_eq!(prog.current_zone_id, EXPANSE_ZONE_ID);
        assert_eq!(prog.current_subzone_id, 1);
        assert_eq!(prog.kills_in_subzone, 0);
    }

    #[test]
    fn test_zone_11_multiple_cycles() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        prog.current_zone_id = EXPANSE_ZONE_ID;
        prog.unlock_zone(EXPANSE_ZONE_ID);

        for cycle in 0..3 {
            // Clear subzones 1-3
            for subzone in 1..=3 {
                prog.current_subzone_id = subzone;
                for _ in 0..KILLS_FOR_BOSS {
                    prog.record_kill();
                }
                let result = prog.on_boss_defeated(20, &mut achievements);
                assert!(
                    matches!(result, BossDefeatResult::SubzoneComplete { .. }),
                    "Cycle {cycle}, subzone {subzone}: expected SubzoneComplete, got {:?}",
                    result
                );
            }

            // Clear subzone 4 (zone boss) -> cycle
            prog.current_subzone_id = 4;
            for _ in 0..KILLS_FOR_BOSS {
                prog.record_kill();
            }
            let result = prog.on_boss_defeated(20, &mut achievements);
            assert_eq!(
                result,
                BossDefeatResult::ExpanseCycle,
                "Cycle {cycle}: expected ExpanseCycle"
            );
            assert_eq!(prog.current_subzone_id, 1);
        }
    }

    #[test]
    fn test_zone_11_cycling_records_boss_defeats() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        prog.current_zone_id = EXPANSE_ZONE_ID;
        prog.current_subzone_id = 4;
        prog.unlock_zone(EXPANSE_ZONE_ID);
        prog.fighting_boss = true;

        prog.on_boss_defeated(20, &mut achievements);

        // Boss defeat should be recorded (but the same boss can be defeated again)
        assert!(prog.is_boss_defeated(EXPANSE_ZONE_ID, 4));
    }

    // =========================================================================
    // WEAPON GATE EDGE CASES (ZONE 10 STORMBREAKER) - boss defeat specific
    // =========================================================================

    #[test]
    fn test_weapon_required_result_for_zone_10_without_stormbreaker() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        prog.current_zone_id = 10;
        prog.current_subzone_id = 4;
        prog.unlock_zone(10);
        prog.fighting_boss = true;

        let result = prog.on_boss_defeated(20, &mut achievements);
        match result {
            BossDefeatResult::WeaponRequired { weapon_name } => {
                assert_eq!(weapon_name, "Stormbreaker");
            }
            _ => panic!("Expected WeaponRequired, got {:?}", result),
        }
        // Boss is NOT recorded as defeated
        assert!(!prog.is_boss_defeated(10, 4));
        assert!(!prog.fighting_boss);
    }

    #[test]
    fn test_storms_end_result_for_zone_10_final_boss() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::TheStormbreaker, None);

        // Set up at Zone 10, final subzone
        prog.current_zone_id = 10;
        prog.current_subzone_id = 4;
        prog.unlock_zone(10);
        prog.fighting_boss = true;
        // Defeat all previous subzone bosses
        prog.defeat_boss(10, 1);
        prog.defeat_boss(10, 2);
        prog.defeat_boss(10, 3);

        let result = prog.on_boss_defeated(20, &mut achievements);
        assert_eq!(result, BossDefeatResult::StormsEnd);
        assert!(achievements.is_unlocked(AchievementId::StormsEnd));
        assert!(prog.is_zone_unlocked(EXPANSE_ZONE_ID));
        assert_eq!(prog.current_zone_id, EXPANSE_ZONE_ID);
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
}
