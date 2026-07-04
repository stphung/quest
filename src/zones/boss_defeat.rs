//! Boss defeat handling and result types.

use super::data::get_zone;
use super::progression::ZoneProgression;
use crate::achievements::{AchievementId, Achievements};
use crate::core::constants::{EXPANSE_ZONE_ID, FINAL_ZONE_ID};

/// Result of defeating a boss.
/// Uses `&'static str` for zone/weapon names since they come from static zone data (zero allocation).
#[derive(Debug, Clone, PartialEq)]
pub enum BossDefeatResult {
    /// Moved to next subzone within same zone
    SubzoneComplete { new_subzone_id: u32 },
    /// Completed zone and moved to next zone
    ZoneComplete {
        old_zone: &'static str,
        old_zone_id: u32,
        new_zone_id: u32,
    },
    /// Completed zone but next zone requires higher prestige
    ZoneCompleteButGated {
        zone_name: &'static str,
        old_zone_id: u32,
        required_prestige: u32,
    },
    /// Completed the final zone (Zone 10)
    StormsEnd,
    /// Boss requires a legendary weapon to defeat (Zone 10)
    WeaponRequired { weapon_name: &'static str },
    /// Completed a cycle of The Expanse (Zone 11) - returns to subzone 1
    ExpanseCycle,
    /// Completed a fracture cycle (cap zone loops) — returns to subzone 1
    FractureCycle { zone_id: u32 },
    /// Completed a loom cap zone cycle, loops back to subzone 1.
    LoomZoneCycle { zone_id: u32 },
    /// Completed the zone but the next zone recently caused a death-loop
    /// retreat — cycling here until the frontier cooldown is consumed.
    FrontierBackoff { zone_id: u32, blocked_zone_id: u32 },
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

        let Some(zone) = get_zone(zone_id) else {
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
                weapon_name: zone.weapon_name.unwrap_or("legendary weapon"),
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
                    old_zone: zone.name,
                    old_zone_id: zone_id,
                    new_zone_id: self.current_zone_id,
                };
            }

            // Can't advance - either no more zones or prestige-gated
            let next_zone = get_zone(zone_id + 1);
            if let Some(next) = next_zone {
                return BossDefeatResult::ZoneCompleteButGated {
                    zone_name: zone.name,
                    old_zone_id: zone_id,
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

        let Some(zone) = get_zone(zone_id) else {
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
                weapon_name: zone.weapon_name.unwrap_or("legendary weapon"),
            };
        }

        self.defeat_boss(zone_id, subzone_id);

        // Defeating any boss in the zone we last retreated from proves the
        // player can survive it — clear the death-retreat memory.
        if self.death_retreat_zone == Some(zone_id) {
            self.clear_death_retreat();
        }

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

        // Frontier backoff: the next zone recently death-looped us. Consume one
        // cooldown cycle and loop this zone instead of auto-advancing back into
        // the zone that keeps killing us (#576).
        let next_zone_id = zone_id + 1;
        if self.frontier_backoff_blocks(next_zone_id) {
            self.frontier_cooldown_cycles -= 1;
            self.current_subzone_id = 1;
            self.kills_in_subzone = 0;
            return BossDefeatResult::FrontierBackoff {
                zone_id,
                blocked_zone_id: next_zone_id,
            };
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
                    old_zone: zone.name,
                    old_zone_id: zone_id,
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
                    old_zone: zone.name,
                    old_zone_id: zone_id,
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
                old_zone: zone.name,
                old_zone_id: zone_id,
                new_zone_id: self.current_zone_id,
            };
        }

        // Fallback for pre-game zones: prestige-gated
        if zone_id < EXPANSE_ZONE_ID {
            let next_zone = get_zone(zone_id + 1);
            if let Some(next) = next_zone {
                return BossDefeatResult::ZoneCompleteButGated {
                    zone_name: zone.name,
                    old_zone_id: zone_id,
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
                ..
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
                ..
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

    // =========================================================================
    // FRONTIER BACKOFF TESTS (death loop at zone frontiers, #576)
    // =========================================================================

    /// Puts the progression at zone 1's final subzone, fighting the zone boss,
    /// with earlier subzone bosses defeated.
    fn setup_zone_1_boss_fight(prog: &mut ZoneProgression) {
        prog.defeat_boss(1, 1);
        prog.defeat_boss(1, 2);
        prog.current_zone_id = 1;
        prog.current_subzone_id = 3;
        prog.fighting_boss = true;
    }

    #[test]
    fn test_frontier_backoff_cycles_instead_of_advancing() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Simulate two death-loop retreats from zone 2
        prog.current_zone_id = 2;
        prog.record_death_retreat();
        prog.record_death_retreat();
        assert_eq!(prog.frontier_cooldown_cycles, 2);

        // Defeat zone 1's boss — should cycle zone 1, not advance into zone 2
        setup_zone_1_boss_fight(&mut prog);
        let result = prog.on_boss_defeated_with_cap(0, &mut achievements, 11, 30);
        assert_eq!(
            result,
            BossDefeatResult::FrontierBackoff {
                zone_id: 1,
                blocked_zone_id: 2
            }
        );
        assert_eq!(prog.current_zone_id, 1);
        assert_eq!(prog.current_subzone_id, 1);
        assert_eq!(prog.frontier_cooldown_cycles, 1);

        // Second cycle consumes the last cooldown
        setup_zone_1_boss_fight(&mut prog);
        let result = prog.on_boss_defeated_with_cap(0, &mut achievements, 11, 30);
        assert!(matches!(result, BossDefeatResult::FrontierBackoff { .. }));
        assert_eq!(prog.frontier_cooldown_cycles, 0);

        // Cooldown exhausted — now advances into zone 2 again
        setup_zone_1_boss_fight(&mut prog);
        let result = prog.on_boss_defeated_with_cap(0, &mut achievements, 11, 30);
        assert!(
            matches!(
                result,
                BossDefeatResult::ZoneComplete { new_zone_id: 2, .. }
            ),
            "Expected ZoneComplete after cooldown exhausted, got {:?}",
            result
        );
        assert_eq!(prog.current_zone_id, 2);
    }

    #[test]
    fn test_boss_defeat_in_retreat_zone_clears_memory() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Retreated from zone 2 once, then re-entered and beat a boss there
        prog.current_zone_id = 2;
        prog.record_death_retreat();
        assert_eq!(prog.death_retreat_zone, Some(2));

        prog.current_subzone_id = 1;
        prog.fighting_boss = true;
        let result = prog.on_boss_defeated_with_cap(0, &mut achievements, 11, 30);
        assert!(matches!(
            result,
            BossDefeatResult::SubzoneComplete { new_subzone_id: 2 }
        ));
        assert_eq!(prog.death_retreat_zone, None);
        assert_eq!(prog.frontier_cooldown_cycles, 0);
    }

    #[test]
    fn test_frontier_backoff_does_not_block_other_zones() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Retreated from zone 5 — advancing 1 -> 2 must be unaffected
        prog.current_zone_id = 5;
        prog.record_death_retreat();

        setup_zone_1_boss_fight(&mut prog);
        let result = prog.on_boss_defeated_with_cap(0, &mut achievements, 11, 30);
        assert!(matches!(
            result,
            BossDefeatResult::ZoneComplete { new_zone_id: 2, .. }
        ));
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

    #[test]
    fn test_on_boss_defeated_final_zone_with_no_successor_returns_storms_end() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Zone 50 is the last zone in the data table -- get_zone(51) is None.
        // The legacy on_boss_defeated() (no fracture-cap awareness) falls back
        // to StormsEnd when there's simply nothing left to advance into.
        let zone50 = get_zone(50).expect("zone 50 must exist");
        let last_subzone = zone50.subzones.len() as u32;
        for sz in 1..last_subzone {
            prog.defeat_boss(50, sz);
        }
        prog.current_zone_id = 50;
        prog.current_subzone_id = last_subzone;
        prog.unlock_zone(50);
        prog.fighting_boss = true;

        let result = prog.on_boss_defeated(50000, &mut achievements);
        assert_eq!(result, BossDefeatResult::StormsEnd);
        assert!(prog.is_boss_defeated(50, last_subzone));
    }

    // =========================================================================
    // UNKNOWN ZONE ID EARLY RETURN (get_zone returns None)
    // =========================================================================

    #[test]
    fn test_on_boss_defeated_unknown_zone_returns_subzone_complete() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // No zone with this ID exists.
        prog.current_zone_id = 999;
        prog.current_subzone_id = 3;
        prog.fighting_boss = true;

        let result = prog.on_boss_defeated(0, &mut achievements);
        assert_eq!(
            result,
            BossDefeatResult::SubzoneComplete { new_subzone_id: 3 }
        );
        // State is left untouched (no defeat recorded, no advancement)
        assert!(!prog.is_boss_defeated(999, 3));
    }

    #[test]
    fn test_on_boss_defeated_with_cap_unknown_zone_returns_subzone_complete() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        prog.current_zone_id = 12345;
        prog.current_subzone_id = 7;
        prog.fighting_boss = true;

        let result = prog.on_boss_defeated_with_cap(0, &mut achievements, 30, 50);
        assert_eq!(
            result,
            BossDefeatResult::SubzoneComplete { new_subzone_id: 7 }
        );
    }

    // =========================================================================
    // on_boss_defeated_with_cap: WEAPON GATE / STORMS END PARITY
    // =========================================================================

    #[test]
    fn test_with_cap_weapon_required_for_zone_10_without_stormbreaker() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        prog.current_zone_id = 10;
        prog.current_subzone_id = 4;
        prog.unlock_zone(10);
        prog.fighting_boss = true;

        let result = prog.on_boss_defeated_with_cap(20, &mut achievements, 11, 30);
        match result {
            BossDefeatResult::WeaponRequired { weapon_name } => {
                assert_eq!(weapon_name, "Stormbreaker");
            }
            _ => panic!("Expected WeaponRequired, got {:?}", result),
        }
        assert!(!prog.is_boss_defeated(10, 4));
        assert!(!prog.fighting_boss);
        assert_eq!(prog.kills_in_subzone, 0);
    }

    #[test]
    fn test_with_cap_storms_end_unlocks_expanse() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::TheStormbreaker, None);

        prog.current_zone_id = 10;
        prog.current_subzone_id = 4;
        prog.unlock_zone(10);
        prog.fighting_boss = true;
        prog.defeat_boss(10, 1);
        prog.defeat_boss(10, 2);
        prog.defeat_boss(10, 3);

        let result = prog.on_boss_defeated_with_cap(20, &mut achievements, 11, 30);
        assert_eq!(result, BossDefeatResult::StormsEnd);
        assert!(achievements.is_unlocked(AchievementId::StormsEnd));
        assert!(prog.is_zone_unlocked(EXPANSE_ZONE_ID));
        assert_eq!(prog.current_zone_id, EXPANSE_ZONE_ID);
        assert_eq!(prog.current_subzone_id, 1);
    }

    // =========================================================================
    // DEATH-RETREAT MEMORY CLEARED ON ZONE-BOSS DEFEAT (not just subzone boss)
    // =========================================================================

    #[test]
    fn test_zone_boss_defeat_clears_death_retreat_memory_for_own_zone() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Player retreated from zone 1 previously (e.g. re-entered and died again later),
        // then comes back and clears the final zone boss.
        prog.current_zone_id = 1;
        prog.record_death_retreat();
        assert_eq!(prog.death_retreat_zone, Some(1));

        prog.defeat_boss(1, 1);
        prog.defeat_boss(1, 2);
        prog.current_subzone_id = 3;
        prog.fighting_boss = true;

        let result = prog.on_boss_defeated_with_cap(0, &mut achievements, 11, 30);
        assert!(matches!(
            result,
            BossDefeatResult::ZoneComplete { new_zone_id: 2, .. }
        ));
        assert_eq!(prog.death_retreat_zone, None);
        assert_eq!(prog.frontier_cooldown_cycles, 0);
    }

    // =========================================================================
    // ZONE 11 -> ZONE 12 TRANSITION (fracture zones unlocked)
    // =========================================================================

    #[test]
    fn test_with_cap_expanse_classic_cycle_when_fracture_cap_at_11() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // fracture_zone_cap == EXPANSE_ZONE_ID (11): no fracture zones unlocked
        // yet, so on_boss_defeated_with_cap takes its own "classic cycle"
        // branch (distinct from the legacy on_boss_defeated's Expanse handling).
        prog.current_zone_id = EXPANSE_ZONE_ID;
        prog.current_subzone_id = 4;
        prog.unlock_zone(EXPANSE_ZONE_ID);
        prog.fighting_boss = true;

        let result = prog.on_boss_defeated_with_cap(20, &mut achievements, 11, 30);
        assert_eq!(result, BossDefeatResult::ExpanseCycle);
        assert_eq!(prog.current_zone_id, EXPANSE_ZONE_ID);
        assert_eq!(prog.current_subzone_id, 1);
        assert_eq!(prog.kills_in_subzone, 0);
    }

    #[test]
    fn test_expanse_advances_to_zone_12_when_fracture_unlocked_and_zone_ready() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        prog.current_zone_id = EXPANSE_ZONE_ID;
        prog.current_subzone_id = 4;
        prog.unlock_zone(EXPANSE_ZONE_ID);
        prog.unlock_zone(12);
        prog.fighting_boss = true;

        // fracture_zone_cap > EXPANSE_ZONE_ID, and zone 12 is unlocked
        let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14, 30);
        assert!(
            matches!(
                result,
                BossDefeatResult::ZoneComplete {
                    old_zone_id: EXPANSE_ZONE_ID,
                    new_zone_id: 12,
                    ..
                }
            ),
            "Expected ZoneComplete advancing Expanse -> 12, got {:?}",
            result
        );
        assert_eq!(prog.current_zone_id, 12);
        assert_eq!(prog.current_subzone_id, 1);
    }

    #[test]
    fn test_expanse_falls_back_to_cycle_when_zone_12_not_yet_unlocked() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // fracture_zone_cap says fracture zones exist, but zone 12 itself
        // hasn't been unlocked yet (e.g. sync hasn't run) -- must not panic
        // or advance into a locked zone.
        prog.current_zone_id = EXPANSE_ZONE_ID;
        prog.current_subzone_id = 4;
        prog.unlock_zone(EXPANSE_ZONE_ID);
        prog.fighting_boss = true;

        let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14, 30);
        assert_eq!(result, BossDefeatResult::ExpanseCycle);
        assert_eq!(prog.current_zone_id, EXPANSE_ZONE_ID);
        assert_eq!(prog.current_subzone_id, 1);
        assert_eq!(prog.kills_in_subzone, 0);
    }

    // =========================================================================
    // FRACTURE REGION UNLOCK THRESHOLDS (Deep layers 3/7/12/18/25/30)
    // =========================================================================

    /// Sets up `prog` at the given zone's final subzone, ready to defeat the zone boss.
    fn setup_zone_boss_fight(prog: &mut ZoneProgression, zone_id: u32) {
        let zone = get_zone(zone_id).expect("zone must exist");
        let last_subzone = zone.subzones.len() as u32;
        for sz in 1..last_subzone {
            prog.defeat_boss(zone_id, sz);
        }
        prog.current_zone_id = zone_id;
        prog.current_subzone_id = last_subzone;
        prog.unlock_zone(zone_id);
        prog.fighting_boss = true;
    }

    #[test]
    fn test_red_fault_cap_zone_14_cycles_at_deep_layer_3() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        setup_zone_boss_fight(&mut prog, 14);
        // fracture_zone_cap = 14 (Deep Layer 3 breakthrough, RedFault only)
        let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14, 30);
        assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 14 });
        assert_eq!(prog.current_subzone_id, 1);
    }

    #[test]
    fn test_mirror_scar_cap_zone_17_cycles_at_deep_layer_7() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        setup_zone_boss_fight(&mut prog, 17);
        let result = prog.on_boss_defeated_with_cap(75, &mut achievements, 17, 30);
        assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 17 });
    }

    #[test]
    fn test_black_mouth_cap_zone_20_cycles_at_deep_layer_12() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        setup_zone_boss_fight(&mut prog, 20);
        let result = prog.on_boss_defeated_with_cap(100, &mut achievements, 20, 30);
        assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 20 });
    }

    #[test]
    fn test_hollow_throne_cap_zone_23_cycles_at_deep_layer_18() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        setup_zone_boss_fight(&mut prog, 23);
        let result = prog.on_boss_defeated_with_cap(150, &mut achievements, 23, 30);
        assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 23 });
    }

    #[test]
    fn test_wailing_reach_cap_zone_26_cycles_at_deep_layer_25() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        setup_zone_boss_fight(&mut prog, 26);
        let result = prog.on_boss_defeated_with_cap(200, &mut achievements, 26, 30);
        assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 26 });
    }

    #[test]
    fn test_origin_wound_cap_zone_30_cycles_at_deep_layer_30() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        setup_zone_boss_fight(&mut prog, 30);
        let result = prog.on_boss_defeated_with_cap(300, &mut achievements, 30, 30);
        assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 30 });
    }

    #[test]
    fn test_fracture_zone_just_below_cap_advances_forward() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Zone 13, one below the RedFault cap (14) -- should advance forward,
        // not cycle, since it isn't the cap zone yet.
        setup_zone_boss_fight(&mut prog, 13);
        prog.unlock_zone(14);
        let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14, 30);
        assert!(
            matches!(
                result,
                BossDefeatResult::ZoneComplete {
                    new_zone_id: 14,
                    ..
                }
            ),
            "Expected advance to 14, got {:?}",
            result
        );
    }

    #[test]
    fn test_fracture_zone_below_cap_but_prestige_gated_for_next_chapter_cycles_in_place() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Deep Layer 7 has broken through (fracture_zone_cap = 17, MirrorScar
        // reachable), but the player's prestige rank (50) hasn't yet reached
        // the P75 required for zone 15. Zone 14 is not the cap, so the
        // explicit cap-cycle branch is skipped -- but advance_to_next_zone
        // still fails on the prestige gate, so we fall back to cycling zone
        // 14 in place (fracture zones never emit ZoneCompleteButGated).
        setup_zone_boss_fight(&mut prog, 14);
        let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 17, 30);
        assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 14 });
        assert_eq!(prog.current_zone_id, 14);
        assert_eq!(prog.current_subzone_id, 1);
    }

    // =========================================================================
    // LOOM ZONE FALLBACK: PRESTIGE-GATED NEXT CHAPTER CYCLES IN PLACE
    // =========================================================================

    #[test]
    fn test_loom_zone_below_cap_but_prestige_gated_for_next_chapter_cycles_in_place() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Zone 34 (top of Thread Wilds, P2000) is not the loom cap (38), and
        // the player's prestige (2000) satisfies zone 34 but not zone 35's
        // P5000 requirement -- advance fails, falls back to cycling zone 34.
        setup_zone_boss_fight(&mut prog, 34);
        let result = prog.on_boss_defeated_with_cap(2000, &mut achievements, 30, 38);
        assert_eq!(result, BossDefeatResult::LoomZoneCycle { zone_id: 34 });
        assert_eq!(prog.current_zone_id, 34);
        assert_eq!(prog.current_subzone_id, 1);
        assert!(!prog.fighting_boss);
    }

    // =========================================================================
    // on_boss_defeated_with_cap: PRE-GAME PRESTIGE-GATED ZONE COMPLETION
    // =========================================================================

    #[test]
    fn test_with_cap_pre_game_zone_completion_prestige_gated() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // Clear zone 1 and zone 2 via with_cap, then get gated entering zone 3 (needs P5).
        for _subzone in 1..=3 {
            for _ in 0..KILLS_FOR_BOSS {
                prog.record_kill();
            }
            prog.on_boss_defeated_with_cap(0, &mut achievements, 11, 30);
        }
        for _subzone in 1..=3 {
            for _ in 0..KILLS_FOR_BOSS {
                prog.record_kill();
            }
            prog.on_boss_defeated_with_cap(0, &mut achievements, 11, 30);
        }
        assert_eq!(prog.current_zone_id, 2);

        match prog.on_boss_defeated_with_cap(0, &mut achievements, 11, 30) {
            BossDefeatResult::ZoneCompleteButGated {
                zone_name,
                required_prestige,
                ..
            } => {
                assert_eq!(zone_name, "Dark Forest");
                assert_eq!(required_prestige, 5);
            }
            other => panic!("Expected ZoneCompleteButGated, got {:?}", other),
        }
        // Still stuck in zone 2 -- gating does not advance
        assert_eq!(prog.current_zone_id, 2);
    }

    // =========================================================================
    // NO-PRESTIGE VS P15+ EDGE CASE: EARLY GAME BOSS DEFEAT
    // =========================================================================

    #[test]
    fn test_zone_1_boss_defeat_with_zero_prestige_rank() {
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        // A brand-new character (P0, no prestige ever) can still clear zone 1's
        // subzone bosses and advance into zone 2 (P0 requirement).
        for _ in 0..KILLS_FOR_BOSS {
            prog.record_kill();
        }
        let result = prog.on_boss_defeated_with_cap(0, &mut achievements, 11, 30);
        assert!(matches!(
            result,
            BossDefeatResult::SubzoneComplete { new_subzone_id: 2 }
        ));
    }

    #[test]
    fn test_fracture_zone_boss_defeat_requires_p15_or_higher_in_practice() {
        // Fracture zones start at P50 (RedFault); this test documents that
        // defeating a fracture-zone boss at P0 still resolves deterministically
        // (the cap-cycle/advance logic does not itself re-check prestige for
        // the *current* zone, only for the *next* one) -- it should cycle in
        // place rather than panic or silently no-op.
        let mut prog = ZoneProgression::new();
        let mut achievements = Achievements::default();

        setup_zone_boss_fight(&mut prog, 12);
        prog.unlock_zone(13);
        // Even though prestige_rank passed is 0 (well below any fracture
        // requirement), zone 13 has no additional prestige requirement beyond
        // zone 12's own, and advance_to_next_zone's prestige check is against
        // the *next* zone (also P50) -- so at P0 it is gated, cycling in place.
        let result = prog.on_boss_defeated_with_cap(0, &mut achievements, 14, 30);
        assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 12 });
    }
}
