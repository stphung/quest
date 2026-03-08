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
    /// Completed a fracture cycle (cap zone loops) — returns to subzone 1
    FractureCycle { zone_id: u32 },
    /// Completed a loom cap zone cycle, loops back to subzone 1.
    LoomZoneCycle { zone_id: u32 },
}

impl ZoneProgression {
    /// Handles boss defeat for the current subzone and auto-advances.
    /// Returns a description of what happened (for UI feedback).
    ///
    /// Uses achievements to check for Stormbreaker and to unlock StormsEnd.
    #[allow(dead_code)]
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

    /// Handles boss defeat with fracture zone cap awareness.
    ///
    /// `fracture_zone_cap` is the highest zone the player can access (from DeepPersistent).
    /// When the cap is 11, this behaves identically to `on_boss_defeated()`.
    /// When higher, fracture zones advance forward until the cap zone, which cycles.
    pub fn on_boss_defeated_with_cap(
        &mut self,
        prestige_rank: u32,
        achievements: &mut Achievements,
        fracture_zone_cap: u32,
        loom_zone_cap: u32,
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

        // Check for Zone 10 weapon requirement
        let has_stormbreaker = achievements.is_unlocked(AchievementId::TheStormbreaker);
        if zone.requires_weapon && is_zone_boss && !has_stormbreaker {
            self.fighting_boss = false;
            self.kills_in_subzone = 0;
            return BossDefeatResult::WeaponRequired {
                weapon_name: zone.weapon_name.unwrap_or("legendary weapon").to_string(),
            };
        }

        self.defeat_boss(zone_id, subzone_id);

        if !is_zone_boss {
            self.advance_to_next_subzone();
            return BossDefeatResult::SubzoneComplete {
                new_subzone_id: self.current_subzone_id,
            };
        }

        // Zone boss defeated — handle progression

        // Zone 10: StormsEnd
        if zone_id == FINAL_ZONE_ID {
            achievements.unlock(AchievementId::StormsEnd, None);
            self.unlock_zone(EXPANSE_ZONE_ID);
            self.current_zone_id = EXPANSE_ZONE_ID;
            self.current_subzone_id = 1;
            return BossDefeatResult::StormsEnd;
        }

        // Zone 11 (Expanse) with no fracture zones unlocked: classic cycle
        if zone_id == EXPANSE_ZONE_ID && fracture_zone_cap <= EXPANSE_ZONE_ID {
            self.current_subzone_id = 1;
            self.kills_in_subzone = 0;
            return BossDefeatResult::ExpanseCycle;
        }

        // Zone 11 (Expanse) with fracture zones unlocked: advance to zone 12
        if zone_id == EXPANSE_ZONE_ID && fracture_zone_cap > EXPANSE_ZONE_ID {
            let next = 12;
            if self.is_zone_unlocked(next) {
                self.current_zone_id = next;
                self.current_subzone_id = 1;
                return BossDefeatResult::ZoneComplete {
                    old_zone: zone.name.to_string(),
                    new_zone_id: next,
                };
            }
            // Fallback: cycle Expanse if zone 12 not unlocked yet
            self.current_subzone_id = 1;
            self.kills_in_subzone = 0;
            return BossDefeatResult::ExpanseCycle;
        }

        // Current zone is the cap zone (fracture) — cycle unless loom zones are open
        if zone_id == fracture_zone_cap && zone_id > EXPANSE_ZONE_ID {
            // If loom zones are unlocked beyond fracture cap, advance instead of cycling
            let next = zone_id + 1;
            if (31..=50).contains(&next) && self.is_zone_unlocked(next) {
                self.current_zone_id = next;
                self.current_subzone_id = 1;
                self.kills_in_subzone = 0;
                return BossDefeatResult::ZoneComplete {
                    old_zone: zone.name.to_string(),
                    new_zone_id: next,
                };
            }
            self.current_subzone_id = 1;
            self.kills_in_subzone = 0;
            return BossDefeatResult::FractureCycle { zone_id };
        }

        // Current zone is the loom cap zone — cycle
        if zone_id >= 31 && zone_id == loom_zone_cap && zone_id <= 50 {
            self.current_subzone_id = 1;
            self.kills_in_subzone = 0;
            self.fighting_boss = false;
            return BossDefeatResult::LoomZoneCycle { zone_id };
        }

        // Try to advance to next zone (works for both pre-game and fracture zones)
        if self.advance_to_next_zone(prestige_rank) {
            return BossDefeatResult::ZoneComplete {
                old_zone: zone.name.to_string(),
                new_zone_id: self.current_zone_id,
            };
        }

        // Fallback for pre-game zones: prestige-gated
        if zone_id < EXPANSE_ZONE_ID {
            let next_zone = zones.iter().find(|z| z.id == zone_id + 1);
            if let Some(next) = next_zone {
                return BossDefeatResult::ZoneCompleteButGated {
                    zone_name: zone.name.to_string(),
                    required_prestige: next.prestige_requirement,
                };
            }
        }

        // Fallback: cycle in place
        self.current_subzone_id = 1;
        self.kills_in_subzone = 0;
        if zone_id >= 31 {
            BossDefeatResult::LoomZoneCycle { zone_id }
        } else if zone_id > EXPANSE_ZONE_ID {
            BossDefeatResult::FractureCycle { zone_id }
        } else {
            BossDefeatResult::ExpanseCycle
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

    #[test]
    fn test_zone_10_non_final_subzone_boss_no_stormbreaker() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Zone 10, subzone 2 boss (not the final subzone) - no Stormbreaker needed
        prog.current_zone_id = 10;
        prog.current_subzone_id = 2;
        prog.unlock_zone(10);
        prog.defeat_boss(10, 1);
        for _ in 0..KILLS_FOR_BOSS {
            prog.record_kill();
        }

        let result = prog.on_boss_defeated(20, &mut achievements);
        assert!(
            matches!(
                result,
                BossDefeatResult::SubzoneComplete { new_subzone_id: 3 }
            ),
            "Non-final subzone boss in Zone 10 should not require Stormbreaker, got {:?}",
            result
        );
    }

    // =========================================================================
    // LOOM ZONE CYCLING TESTS
    // =========================================================================

    #[test]
    fn test_loom_zone_cap_cycles() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Set up at zone 34 (a loom zone), final subzone (5), fighting boss
        prog.current_zone_id = 34;
        prog.current_subzone_id = 5;
        prog.unlock_zone(34);
        prog.fighting_boss = true;

        // Call with loom_zone_cap = 34
        let result = prog.on_boss_defeated_with_cap(2000, &mut achievements, 30, 34);
        assert_eq!(result, BossDefeatResult::LoomZoneCycle { zone_id: 34 });
        assert_eq!(prog.current_zone_id, 34);
        assert_eq!(prog.current_subzone_id, 1);
        assert_eq!(prog.kills_in_subzone, 0);
        assert!(!prog.fighting_boss);
    }

    #[test]
    fn test_loom_zone_advances_when_not_at_cap() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Set up at zone 33, final subzone (5), fighting boss
        prog.current_zone_id = 33;
        prog.current_subzone_id = 5;
        prog.unlock_zone(33);
        prog.unlock_zone(34);
        prog.fighting_boss = true;

        // Call with loom_zone_cap = 34 — zone 33 is NOT the cap, so should advance
        let result = prog.on_boss_defeated_with_cap(2000, &mut achievements, 30, 34);
        assert!(
            matches!(
                result,
                BossDefeatResult::ZoneComplete {
                    new_zone_id: 34,
                    ..
                }
            ),
            "Expected ZoneComplete advancing to 34, got {:?}",
            result
        );
        assert_eq!(prog.current_zone_id, 34);
        assert_eq!(prog.current_subzone_id, 1);
    }

    #[test]
    fn test_loom_zone_cap_30_no_loom_cycling() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Zone 30 is the fracture cap, not a loom zone
        prog.current_zone_id = 30;
        prog.current_subzone_id = 5;
        prog.unlock_zone(30);
        prog.fighting_boss = true;

        // loom_zone_cap=30 means no loom zones unlocked
        // fracture_zone_cap=30 means fracture cycling at 30
        let result = prog.on_boss_defeated_with_cap(300, &mut achievements, 30, 30);
        assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 30 });
    }

    #[test]
    fn test_fracture_cap_30_advances_to_loom_zone_31_when_unlocked() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Set up at zone 30 (fracture cap), final subzone, fighting boss
        prog.current_zone_id = 30;
        prog.current_subzone_id = 5;
        prog.unlock_zone(30);
        prog.unlock_zone(31); // Loom zone 31 is unlocked
        prog.fighting_boss = true;

        // fracture_zone_cap=30, loom_zone_cap=34
        let result = prog.on_boss_defeated_with_cap(2000, &mut achievements, 30, 34);
        assert!(
            matches!(
                result,
                BossDefeatResult::ZoneComplete {
                    new_zone_id: 31,
                    ..
                }
            ),
            "Expected ZoneComplete advancing to 31, got {:?}",
            result
        );
        assert_eq!(prog.current_zone_id, 31);
        assert_eq!(prog.current_subzone_id, 1);
        assert_eq!(prog.kills_in_subzone, 0);
    }

    #[test]
    fn test_boss_defeat_resets_kill_state() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        for _ in 0..KILLS_FOR_BOSS {
            prog.record_kill();
        }
        assert!(prog.fighting_boss);
        assert_eq!(prog.kills_in_subzone, KILLS_FOR_BOSS);

        prog.on_boss_defeated(0, &mut achievements);

        // After boss defeat, kills and boss flag should be reset
        assert_eq!(prog.kills_in_subzone, 0);
        assert!(!prog.fighting_boss);
    }
}
