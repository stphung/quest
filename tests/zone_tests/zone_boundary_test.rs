//! Zone boundary condition integration tests
//!
//! Tests boundary conditions and edge cases for zone/subzone progression that
//! are not already covered by tests embedded in the source or zone_progression_test.rs.
//!
//! Focus areas:
//! - travel_to() boundary cases (subzone 0, out-of-range zones)
//! - advance_to_next_subzone() at zone boundaries
//! - Kill count reset after boss defeat
//! - Exact prestige gate boundaries (P4 vs P5, P9 vs P10, etc.)
//! - Zone 11 subzone cycling with correct killed-boss tracking
//! - BossDefeatResult variants for each zone transition type

use quest::achievements::{AchievementId, Achievements};
use quest::core::constants::{EXPANSE_ZONE_ID, FINAL_ZONE_ID, KILLS_FOR_BOSS};
use quest::zones::get_all_zones;
use quest::zones::{BossDefeatResult, ZoneProgression};

// ============================================================================
// travel_to() boundary conditions
// ============================================================================

#[test]
fn test_travel_to_subzone_1_of_unlocked_zone_always_accessible() {
    let mut prog = ZoneProgression::new();
    // Subzone IDs are 1-based; subzone 1 of any unlocked zone is always accessible
    // Zone 1 and 2 are unlocked by default at P0
    assert!(
        prog.can_enter_subzone(1, 1),
        "Zone 1, subzone 1 always accessible"
    );
    assert!(
        prog.can_enter_subzone(2, 1),
        "Zone 2, subzone 1 always accessible"
    );

    // Travel to first subzone of each unlocked zone works
    assert!(prog.travel_to(1, 1));
    assert_eq!(prog.current_zone_id, 1);
    assert_eq!(prog.current_subzone_id, 1);

    assert!(prog.travel_to(2, 1));
    assert_eq!(prog.current_zone_id, 2);
    assert_eq!(prog.current_subzone_id, 1);
}

#[test]
fn test_travel_to_nonexistent_zone_100() {
    let mut prog = ZoneProgression::new();
    // Zone 100 doesn't exist in the game
    assert!(!prog.travel_to(100, 1));
    assert_eq!(prog.current_zone_id, 1); // Should stay at start
}

#[test]
fn test_travel_to_zone_2_subzone_1_always_accessible() {
    let mut prog = ZoneProgression::new();
    // Zone 2 is unlocked by default (P0), subzone 1 is always accessible
    assert!(prog.travel_to(2, 1));
    assert_eq!(prog.current_zone_id, 2);
    assert_eq!(prog.current_subzone_id, 1);
}

#[test]
fn test_travel_to_later_subzone_requires_boss_defeated() {
    let mut prog = ZoneProgression::new();
    // Can't go to subzone 2 without defeating subzone 1 boss
    assert!(!prog.travel_to(1, 2));
    assert_eq!(prog.current_subzone_id, 1);

    // After defeating boss, can travel to subzone 2
    prog.defeat_boss(1, 1);
    assert!(prog.travel_to(1, 2));
    assert_eq!(prog.current_subzone_id, 2);
}

#[test]
fn test_travel_to_does_not_change_state_on_failure() {
    let mut prog = ZoneProgression::new();
    prog.current_zone_id = 2;
    prog.current_subzone_id = 1;

    // Try to travel to locked zone
    let result = prog.travel_to(5, 1);
    assert!(!result);

    // State unchanged
    assert_eq!(prog.current_zone_id, 2);
    assert_eq!(prog.current_subzone_id, 1);
}

// ============================================================================
// advance_to_next_subzone() at zone boundaries
// ============================================================================

#[test]
fn test_advance_subzone_requires_boss_defeated_first() {
    let mut prog = ZoneProgression::new();
    // At subzone 1, no boss defeated - cannot advance
    assert!(!prog.advance_to_next_subzone());
    assert_eq!(prog.current_subzone_id, 1);
}

#[test]
fn test_advance_subzone_succeeds_after_boss_defeated() {
    let mut prog = ZoneProgression::new();
    prog.defeat_boss(1, 1);
    assert!(prog.advance_to_next_subzone());
    assert_eq!(prog.current_subzone_id, 2);
}

#[test]
fn test_advance_to_next_subzone_at_zone_last_subzone_fails() {
    let zones = get_all_zones();
    let zone1 = &zones[0]; // Zone 1 has 3 subzones
    let last_subzone = zone1.subzones.len() as u32;

    let mut prog = ZoneProgression::new();
    prog.current_subzone_id = last_subzone;
    prog.defeat_boss(1, last_subzone);

    // advance_to_next_subzone should fail because we're already at max
    let advanced = prog.advance_to_next_subzone();
    assert!(!advanced, "Should not advance past last subzone");
    assert_eq!(prog.current_subzone_id, last_subzone);
}

#[test]
fn test_advance_subzone_sequential_for_4_subzone_zone() {
    // Zone 5 (Volcanic Wastes) has 4 subzones; verify sequential advancement
    let mut prog = ZoneProgression::new();
    prog.reset_for_prestige(10); // Unlock zone 5
    prog.current_zone_id = 5;
    prog.current_subzone_id = 1;

    for expected_subzone in 2..=4u32 {
        prog.defeat_boss(5, expected_subzone - 1);
        let advanced = prog.advance_to_next_subzone();
        assert!(advanced, "Should advance to subzone {expected_subzone}");
        assert_eq!(prog.current_subzone_id, expected_subzone);
    }

    // Cannot advance past subzone 4
    prog.defeat_boss(5, 4);
    assert!(!prog.advance_to_next_subzone());
}

// ============================================================================
// Kill count reset after boss defeat
// ============================================================================

#[test]
fn test_kills_until_boss_after_boss_defeat_resets_to_zero() {
    let mut prog = ZoneProgression::new();

    // Accumulate kills to trigger boss
    for _ in 0..KILLS_FOR_BOSS {
        prog.record_kill();
    }
    assert!(prog.fighting_boss);
    assert_eq!(prog.kills_until_boss(), 0);

    // Defeat the boss - resets state
    prog.defeat_boss(1, 1);
    assert!(!prog.fighting_boss);
    assert_eq!(prog.kills_in_subzone, 0);

    // After defeat, need full KILLS_FOR_BOSS again to spawn next boss
    assert_eq!(prog.kills_until_boss(), KILLS_FOR_BOSS);
}

#[test]
fn test_record_kill_returns_false_while_fighting_boss() {
    let mut prog = ZoneProgression::new();

    for _ in 0..KILLS_FOR_BOSS {
        prog.record_kill();
    }
    assert!(prog.fighting_boss);

    // Additional kills return false when already fighting boss
    let spawns = prog.record_kill();
    assert!(!spawns);
    // Kill count doesn't increment during boss fight
    assert_eq!(prog.kills_in_subzone, KILLS_FOR_BOSS);
}

#[test]
fn test_boss_spawn_kills_exactly_at_threshold() {
    let mut prog = ZoneProgression::new();

    // Record KILLS_FOR_BOSS - 1 kills; should not trigger boss
    for i in 0..KILLS_FOR_BOSS - 1 {
        let triggers = prog.record_kill();
        assert!(!triggers, "Kill {} should not trigger boss", i + 1);
    }
    assert!(!prog.fighting_boss);

    // The final kill triggers the boss
    let triggers = prog.record_kill();
    assert!(triggers, "Kill {} should trigger boss", KILLS_FOR_BOSS);
    assert!(prog.fighting_boss);
}

// ============================================================================
// Exact prestige gate boundaries
// ============================================================================

#[test]
fn test_zone_unlock_at_exact_p5_boundary() {
    let mut prog = ZoneProgression::new();
    let zones = get_all_zones();
    let zone3 = zones.iter().find(|z| z.id == 3).unwrap();

    // Defeat zone 2's last boss to satisfy boss gate
    prog.defeat_boss(2, 3);

    // P4: cannot unlock zone 3
    assert!(!prog.can_unlock_zone(zone3, 4));

    // P5: can unlock zone 3
    assert!(prog.can_unlock_zone(zone3, 5));

    // P6: also can unlock
    assert!(prog.can_unlock_zone(zone3, 6));
}

#[test]
fn test_zone_unlock_at_exact_p10_boundary() {
    let mut prog = ZoneProgression::new();
    let zones = get_all_zones();
    let zone5 = zones.iter().find(|z| z.id == 5).unwrap();

    // Defeat zone 4's last boss (3 subzones)
    prog.defeat_boss(4, 3);

    // P9: cannot unlock zone 5
    assert!(!prog.can_unlock_zone(zone5, 9));

    // P10: can unlock zone 5
    assert!(prog.can_unlock_zone(zone5, 10));
}

#[test]
fn test_zone_unlock_at_exact_p15_boundary() {
    let mut prog = ZoneProgression::new();
    let zones = get_all_zones();
    let zone7 = zones.iter().find(|z| z.id == 7).unwrap();

    // Defeat zone 6's last boss (4 subzones)
    prog.defeat_boss(6, 4);

    // P14: cannot unlock zone 7
    assert!(!prog.can_unlock_zone(zone7, 14));

    // P15: can unlock zone 7
    assert!(prog.can_unlock_zone(zone7, 15));
}

#[test]
fn test_zone_unlock_at_exact_p20_boundary() {
    let mut prog = ZoneProgression::new();
    let zones = get_all_zones();
    let zone9 = zones.iter().find(|z| z.id == 9).unwrap();

    // Defeat zone 8's last boss (4 subzones)
    prog.defeat_boss(8, 4);

    // P19: cannot unlock zone 9
    assert!(!prog.can_unlock_zone(zone9, 19));

    // P20: can unlock zone 9
    assert!(prog.can_unlock_zone(zone9, 20));
}

#[test]
fn test_reset_for_prestige_p4_does_not_unlock_p5_zones() {
    let mut prog = ZoneProgression::new();
    prog.reset_for_prestige(4);

    // P5 zones should NOT be unlocked at P4
    assert!(
        !prog.is_zone_unlocked(3),
        "Zone 3 requires P5, should not be at P4"
    );
    assert!(
        !prog.is_zone_unlocked(4),
        "Zone 4 requires P5, should not be at P4"
    );
    // P0 zones should still be unlocked
    assert!(prog.is_zone_unlocked(1));
    assert!(prog.is_zone_unlocked(2));
}

#[test]
fn test_reset_for_prestige_p5_unlocks_exactly_zones_1_to_4() {
    let mut prog = ZoneProgression::new();
    prog.reset_for_prestige(5);

    for z in 1..=4 {
        assert!(
            prog.is_zone_unlocked(z),
            "Zone {z} should be unlocked at P5"
        );
    }
    assert!(!prog.is_zone_unlocked(5), "Zone 5 requires P10, not P5");
}

#[test]
fn test_reset_for_prestige_p9_does_not_unlock_p10_zones() {
    let mut prog = ZoneProgression::new();
    prog.reset_for_prestige(9);

    assert!(!prog.is_zone_unlocked(5), "Zone 5 requires P10, not at P9");
    assert!(!prog.is_zone_unlocked(6), "Zone 6 requires P10, not at P9");
    // P5 zones should be unlocked
    assert!(prog.is_zone_unlocked(3));
    assert!(prog.is_zone_unlocked(4));
}

#[test]
fn test_reset_for_prestige_p19_does_not_unlock_p20_zones() {
    let mut prog = ZoneProgression::new();
    prog.reset_for_prestige(19);

    // P20 zones should not be unlocked at P19
    assert!(!prog.is_zone_unlocked(9), "Zone 9 requires P20");
    assert!(!prog.is_zone_unlocked(10), "Zone 10 requires P20");
    // P15 zones should be unlocked
    assert!(prog.is_zone_unlocked(7));
    assert!(prog.is_zone_unlocked(8));
}

// ============================================================================
// Zone 11 (The Expanse) cycling behavior
// ============================================================================

#[test]
fn test_expanse_zone_id_is_zone_11() {
    assert_eq!(EXPANSE_ZONE_ID, 11, "Expanse must be zone 11");
}

#[test]
fn test_final_zone_id_is_zone_10() {
    assert_eq!(FINAL_ZONE_ID, 10, "Final zone must be zone 10");
}

#[test]
fn test_expanse_cycling_resets_subzone_to_1() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    prog.current_zone_id = EXPANSE_ZONE_ID;
    prog.current_subzone_id = 4;
    prog.unlock_zone(EXPANSE_ZONE_ID);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated(25, &mut achievements);
    assert_eq!(
        result,
        BossDefeatResult::ExpanseCycle,
        "Zone 11 final boss should cycle"
    );
    assert_eq!(prog.current_subzone_id, 1);
    assert_eq!(prog.kills_in_subzone, 0);
    assert!(!prog.fighting_boss);
}

#[test]
fn test_expanse_non_final_subzone_boss_advances_normally() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    prog.current_zone_id = EXPANSE_ZONE_ID;
    prog.current_subzone_id = 2;
    prog.unlock_zone(EXPANSE_ZONE_ID);
    prog.defeat_boss(EXPANSE_ZONE_ID, 1);
    for _ in 0..KILLS_FOR_BOSS {
        prog.record_kill();
    }

    let result = prog.on_boss_defeated(25, &mut achievements);
    assert!(
        matches!(
            result,
            BossDefeatResult::SubzoneComplete { new_subzone_id: 3 }
        ),
        "Non-final subzone boss in Zone 11 should advance normally, got {:?}",
        result
    );
    assert_eq!(prog.current_subzone_id, 3);
}

#[test]
fn test_expanse_cycle_boss_defeat_is_recorded() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    prog.current_zone_id = EXPANSE_ZONE_ID;
    prog.current_subzone_id = 4;
    prog.unlock_zone(EXPANSE_ZONE_ID);
    prog.fighting_boss = true;

    prog.on_boss_defeated(25, &mut achievements);

    // Boss defeat should be recorded for achievement tracking
    assert!(
        prog.is_boss_defeated(EXPANSE_ZONE_ID, 4),
        "Expanse cycle boss defeat should be recorded"
    );
}

#[test]
fn test_expanse_zone_has_prestige_requirement() {
    let zones = get_all_zones();
    let expanse = zones.iter().find(|z| z.id == EXPANSE_ZONE_ID).unwrap();
    // The Expanse is dual-gated: StormsEnd achievement + P25 prestige
    assert_eq!(
        expanse.prestige_requirement, 25,
        "Zone 11 should require P25 prestige"
    );
}

#[test]
fn test_expanse_zone_has_4_subzones() {
    let zones = get_all_zones();
    let expanse = zones.iter().find(|z| z.id == EXPANSE_ZONE_ID).unwrap();
    assert_eq!(expanse.subzones.len(), 4, "Zone 11 should have 4 subzones");
}

// ============================================================================
// BossDefeatResult variant coverage
// ============================================================================

#[test]
fn test_subzone_complete_result_advances_correctly() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    for _ in 0..KILLS_FOR_BOSS {
        prog.record_kill();
    }

    let result = prog.on_boss_defeated(0, &mut achievements);
    match result {
        BossDefeatResult::SubzoneComplete { new_subzone_id } => {
            assert_eq!(new_subzone_id, 2);
        }
        other => panic!("Expected SubzoneComplete, got {other:?}"),
    }
}

#[test]
fn test_zone_complete_result_moves_to_next_zone() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    // Clear all of zone 1
    for subzone in 1..=3 {
        prog.current_subzone_id = subzone;
        for _ in 0..KILLS_FOR_BOSS {
            prog.record_kill();
        }
        let result = prog.on_boss_defeated(0, &mut achievements);
        if subzone == 3 {
            match result {
                BossDefeatResult::ZoneComplete {
                    old_zone,
                    new_zone_id,
                    ..
                } => {
                    assert_eq!(old_zone, "Meadow");
                    assert_eq!(new_zone_id, 2);
                }
                other => panic!("Expected ZoneComplete for zone 1 final boss, got {other:?}"),
            }
        }
    }
    assert_eq!(prog.current_zone_id, 2);
    assert_eq!(prog.current_subzone_id, 1);
}

#[test]
fn test_zone_complete_but_gated_result_at_p4() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    // Clear zone 1 (prestige 0 gates ok)
    for subzone in 1..=3 {
        prog.current_subzone_id = subzone;
        for _ in 0..KILLS_FOR_BOSS {
            prog.record_kill();
        }
        prog.on_boss_defeated(0, &mut achievements);
    }
    assert_eq!(prog.current_zone_id, 2);

    // Clear zone 2 with P4 (zone 3 requires P5)
    for subzone in 1..=3 {
        prog.current_subzone_id = subzone;
        for _ in 0..KILLS_FOR_BOSS {
            prog.record_kill();
        }
        let result = prog.on_boss_defeated(4, &mut achievements);
        if subzone == 3 {
            match result {
                BossDefeatResult::ZoneCompleteButGated {
                    zone_name,
                    required_prestige,
                    ..
                } => {
                    assert_eq!(zone_name, "Dark Forest");
                    assert_eq!(required_prestige, 5);
                }
                other => panic!("Expected ZoneCompleteButGated at P4, got {other:?}"),
            }
            // Should remain in zone 2
            assert_eq!(prog.current_zone_id, 2);
        }
    }
}

#[test]
fn test_storms_end_result_unlocks_expanse() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::TheStormbreaker, None);

    prog.current_zone_id = FINAL_ZONE_ID;
    prog.current_subzone_id = 4; // Zone 10's final subzone
    prog.unlock_zone(FINAL_ZONE_ID);
    // Defeat all previous zone 10 subzone bosses
    for sub in 1..=3 {
        prog.defeat_boss(FINAL_ZONE_ID, sub);
    }
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated(20, &mut achievements);
    assert_eq!(
        result,
        BossDefeatResult::StormsEnd,
        "Zone 10 final boss with Stormbreaker should give StormsEnd"
    );

    // Expanse should be unlocked and player moved there
    assert!(prog.is_zone_unlocked(EXPANSE_ZONE_ID));
    assert_eq!(prog.current_zone_id, EXPANSE_ZONE_ID);
    assert_eq!(prog.current_subzone_id, 1);

    // StormsEnd achievement should be unlocked
    assert!(achievements.is_unlocked(AchievementId::StormsEnd));
}

#[test]
fn test_weapon_required_result_prevents_zone_10_final_boss() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default(); // No Stormbreaker

    prog.current_zone_id = FINAL_ZONE_ID;
    prog.current_subzone_id = 4;
    prog.unlock_zone(FINAL_ZONE_ID);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated(20, &mut achievements);
    match result {
        BossDefeatResult::WeaponRequired { weapon_name } => {
            assert_eq!(weapon_name, "Stormbreaker");
        }
        other => panic!("Expected WeaponRequired, got {other:?}"),
    }

    // Boss NOT recorded as defeated
    assert!(!prog.is_boss_defeated(FINAL_ZONE_ID, 4));
    // Fight resets so player can try again
    assert!(!prog.fighting_boss);
    assert_eq!(prog.kills_in_subzone, 0);
}

// ============================================================================
// Zone data integrity
// ============================================================================

#[test]
fn test_all_zones_have_expected_subzone_counts() {
    let zones = get_all_zones();

    // P0 (Zones 1-2): 3 subzones each
    for &zone_id in &[1u32, 2] {
        let z = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(z.subzones.len(), 3, "Zone {zone_id} should have 3 subzones");
    }
    // P5 (Zones 3-4): 3 subzones each
    for &zone_id in &[3u32, 4] {
        let z = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(z.subzones.len(), 3, "Zone {zone_id} should have 3 subzones");
    }
    // P10+ (Zones 5-10): 4 subzones each
    for zone_id in 5u32..=10 {
        let z = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(z.subzones.len(), 4, "Zone {zone_id} should have 4 subzones");
    }
    // Zone 11 (The Expanse): 4 subzones
    let expanse = zones.iter().find(|z| z.id == EXPANSE_ZONE_ID).unwrap();
    assert_eq!(expanse.subzones.len(), 4, "Zone 11 should have 4 subzones");
}

#[test]
fn test_zone_10_requires_weapon() {
    let zones = get_all_zones();
    let zone10 = zones.iter().find(|z| z.id == 10).unwrap();
    assert!(zone10.requires_weapon, "Zone 10 must require a weapon");
    assert_eq!(zone10.weapon_name, Some("Stormbreaker"));
}

#[test]
fn test_no_other_zone_requires_weapon() {
    let zones = get_all_zones();
    for zone in zones.iter() {
        if zone.id != 10 {
            assert!(
                !zone.requires_weapon,
                "Only Zone 10 should require a weapon; Zone {} does too",
                zone.id
            );
        }
    }
}

#[test]
fn test_zone_subzones_have_sequential_ids() {
    let zones = get_all_zones();
    for zone in zones.iter() {
        for (i, subzone) in zone.subzones.iter().enumerate() {
            assert_eq!(
                subzone.id,
                (i + 1) as u32,
                "Zone {} subzone at index {} should have id {}",
                zone.id,
                i,
                i + 1
            );
        }
    }
}

#[test]
fn test_zone_subzone_final_boss_flag_correct() {
    let zones = get_all_zones();
    for zone in zones.iter() {
        let len = zone.subzones.len();
        for (i, subzone) in zone.subzones.iter().enumerate() {
            let is_last = i == len - 1;
            assert_eq!(
                subzone.boss.is_zone_boss, is_last,
                "Zone {} subzone {} is_zone_boss should be {}",
                zone.id, subzone.id, is_last
            );
        }
    }
}

// ============================================================================
// can_unlock_zone() with boss gate requirement
// ============================================================================

#[test]
fn test_can_unlock_zone_1_always_works_at_p0() {
    let prog = ZoneProgression::new();
    let zones = get_all_zones();
    let zone1 = &zones[0];
    // Zone 1 has no previous zone boss requirement
    assert!(prog.can_unlock_zone(zone1, 0));
}

#[test]
fn test_can_unlock_zone_requires_prev_zone_boss_defeated() {
    let prog = ZoneProgression::new();
    let zones = get_all_zones();
    let zone3 = zones.iter().find(|z| z.id == 3).unwrap();

    // P5 but zone 2 boss not defeated
    assert!(
        !prog.can_unlock_zone(zone3, 5),
        "Zone 3 needs zone 2 final boss defeated"
    );
}

#[test]
fn test_can_unlock_zone_requires_both_prestige_and_boss() {
    let mut prog = ZoneProgression::new();
    let zones = get_all_zones();
    let zone3 = zones.iter().find(|z| z.id == 3).unwrap();

    // Defeat zone 2 final boss
    prog.defeat_boss(2, 3);

    // Without enough prestige, still fails
    assert!(!prog.can_unlock_zone(zone3, 4));

    // With enough prestige AND boss defeated, succeeds
    assert!(prog.can_unlock_zone(zone3, 5));
}
