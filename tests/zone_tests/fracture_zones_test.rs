//! Integration tests for fracture zone boss defeat and cycling logic.

use quest::achievements::Achievements;
use quest::core::constants::{EXPANSE_ZONE_ID, KILLS_FOR_BOSS};
use quest::zones::{BossDefeatResult, ZoneProgression};

#[test]
fn test_zone_11_boss_with_cap_11_returns_expanse_cycle() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = EXPANSE_ZONE_ID;
    prog.current_subzone_id = 4;
    prog.unlock_zone(EXPANSE_ZONE_ID);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(25, &mut achievements, 11, 30);
    assert_eq!(result, BossDefeatResult::ExpanseCycle);
}

#[test]
fn test_zone_11_boss_with_cap_14_advances_to_zone_12() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = EXPANSE_ZONE_ID;
    prog.current_subzone_id = 4;
    prog.unlock_zone(EXPANSE_ZONE_ID);
    prog.unlock_zone(12);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14, 30);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 12);
        }
        _ => panic!("Expected ZoneComplete to zone 12, got {:?}", result),
    }
}

#[test]
fn test_zone_14_boss_with_cap_14_returns_fracture_cycle() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 14;
    prog.current_subzone_id = 5;
    prog.unlock_zone(14);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14, 30);
    assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 14 });
    assert_eq!(prog.current_subzone_id, 1);
}

#[test]
fn test_zone_14_boss_with_cap_17_advances_to_zone_15() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 14;
    prog.current_subzone_id = 5;
    prog.unlock_zone(14);
    prog.unlock_zone(15);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(75, &mut achievements, 17, 30);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 15);
        }
        _ => panic!("Expected ZoneComplete to zone 15, got {:?}", result),
    }
}

#[test]
fn test_zone_20_boss_with_cap_20_returns_fracture_cycle() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 20;
    prog.current_subzone_id = 5;
    prog.unlock_zone(20);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(100, &mut achievements, 20, 30);
    assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 20 });
}

#[test]
fn test_zone_20_boss_with_cap_23_advances_to_zone_21() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 20;
    prog.current_subzone_id = 5;
    prog.unlock_zone(20);
    prog.unlock_zone(21);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(150, &mut achievements, 23, 30);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 21);
        }
        _ => panic!("Expected ZoneComplete to zone 21, got {:?}", result),
    }
}

#[test]
fn test_zone_23_boss_with_cap_23_returns_fracture_cycle() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 23;
    prog.current_subzone_id = 5;
    prog.unlock_zone(23);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(150, &mut achievements, 23, 30);
    assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 23 });
    assert_eq!(prog.current_subzone_id, 1);
}

#[test]
fn test_zone_23_boss_with_cap_26_advances_to_zone_24() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 23;
    prog.current_subzone_id = 5;
    prog.unlock_zone(23);
    prog.unlock_zone(24);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(200, &mut achievements, 26, 30);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 24);
        }
        _ => panic!("Expected ZoneComplete to zone 24, got {:?}", result),
    }
}

#[test]
fn test_zone_26_boss_with_cap_26_returns_fracture_cycle() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 26;
    prog.current_subzone_id = 5;
    prog.unlock_zone(26);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(200, &mut achievements, 26, 30);
    assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 26 });
    assert_eq!(prog.current_subzone_id, 1);
}

#[test]
fn test_zone_26_boss_with_cap_30_advances_to_zone_27() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 26;
    prog.current_subzone_id = 5;
    prog.unlock_zone(26);
    prog.unlock_zone(27);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(300, &mut achievements, 30, 30);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 27);
        }
        _ => panic!("Expected ZoneComplete to zone 27, got {:?}", result),
    }
}

#[test]
fn test_zone_30_boss_with_cap_30_returns_fracture_cycle() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 30;
    prog.current_subzone_id = 5;
    prog.unlock_zone(30);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(300, &mut achievements, 30, 30);
    assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 30 });
    assert_eq!(prog.current_subzone_id, 1);
}

#[test]
fn test_fracture_subzone_advance() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 12;
    prog.current_subzone_id = 1;
    prog.unlock_zone(12);
    for _ in 0..KILLS_FOR_BOSS {
        prog.record_kill();
    }

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14, 30);
    assert!(matches!(
        result,
        BossDefeatResult::SubzoneComplete { new_subzone_id: 2 }
    ));
}

#[test]
fn test_zone_12_boss_advances_to_13() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 12;
    prog.current_subzone_id = 5; // final subzone
    prog.unlock_zone(12);
    prog.unlock_zone(13);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14, 30);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 13);
        }
        _ => panic!("Expected ZoneComplete to zone 13, got {:?}", result),
    }
}

#[test]
fn test_zone_17_boss_with_cap_17_returns_fracture_cycle() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 17;
    prog.current_subzone_id = 5;
    prog.unlock_zone(17);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(75, &mut achievements, 17, 30);
    assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 17 });
    assert_eq!(prog.current_subzone_id, 1);
}

#[test]
fn test_zone_17_boss_with_cap_20_advances_to_zone_18() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 17;
    prog.current_subzone_id = 5;
    prog.unlock_zone(17);
    prog.unlock_zone(18);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(100, &mut achievements, 20, 30);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 18);
        }
        _ => panic!("Expected ZoneComplete to zone 18, got {:?}", result),
    }
}

#[test]
fn test_fracture_cycle_resets_kills_and_subzone() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 14;
    prog.current_subzone_id = 5;
    prog.unlock_zone(14);
    prog.kills_in_subzone = 10;
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14, 30);
    assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 14 });
    assert_eq!(prog.current_subzone_id, 1);
    assert_eq!(prog.kills_in_subzone, 0);
}

#[test]
fn test_original_on_boss_defeated_still_works() {
    // Ensure the original method (without cap) still works for zones 1-11
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    prog.current_zone_id = EXPANSE_ZONE_ID;
    prog.current_subzone_id = 4;
    prog.unlock_zone(EXPANSE_ZONE_ID);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated(25, &mut achievements);
    assert_eq!(result, BossDefeatResult::ExpanseCycle);
}

// =========================================================================
// ZONE 10 WEAPON GATE USING on_boss_defeated_with_cap
// =========================================================================

#[test]
fn test_zone_10_without_stormbreaker_using_cap_variant() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    // Zone 10 has 4 subzones; set to final subzone
    prog.current_zone_id = 10;
    prog.current_subzone_id = 4;
    prog.unlock_zone(10);
    prog.fighting_boss = true;

    // No Stormbreaker — should return WeaponRequired
    let result = prog.on_boss_defeated_with_cap(20, &mut achievements, 14, 30);
    match result {
        BossDefeatResult::WeaponRequired { weapon_name } => {
            assert_eq!(weapon_name, "Stormbreaker");
        }
        _ => panic!("Expected WeaponRequired, got {:?}", result),
    }
    // Boss should NOT be recorded as defeated
    assert!(!prog.is_boss_defeated(10, 4));
    // State reset so player can try again
    assert!(!prog.fighting_boss);
    assert_eq!(prog.kills_in_subzone, 0);
}

#[test]
fn test_zone_10_with_stormbreaker_using_cap_variant() {
    use quest::achievements::AchievementId;
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::TheStormbreaker, None);

    prog.current_zone_id = 10;
    prog.current_subzone_id = 4;
    prog.unlock_zone(10);
    prog.fighting_boss = true;

    // Has Stormbreaker — should trigger StormsEnd and advance to Zone 11
    let result = prog.on_boss_defeated_with_cap(20, &mut achievements, 14, 30);
    assert_eq!(result, BossDefeatResult::StormsEnd);
    assert!(achievements.is_unlocked(AchievementId::StormsEnd));
    assert!(prog.is_zone_unlocked(EXPANSE_ZONE_ID));
    assert_eq!(prog.current_zone_id, EXPANSE_ZONE_ID);
    assert_eq!(prog.current_subzone_id, 1);
}

// =========================================================================
// Z11 EXPANSE FALLBACK: cap > 11 but zone 12 not yet unlocked
// =========================================================================

#[test]
fn test_zone_11_boss_cap_gt_11_but_zone_12_not_unlocked_falls_back_to_expanse_cycle() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    // cap = 14 (fracture unlocked) but zone 12 is NOT in unlocked_zones
    prog.current_zone_id = EXPANSE_ZONE_ID;
    prog.current_subzone_id = 4;
    prog.unlock_zone(EXPANSE_ZONE_ID);
    // Deliberately NOT unlocking zone 12
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14, 30);
    // Should fall back to ExpanseCycle because zone 12 is locked
    assert_eq!(result, BossDefeatResult::ExpanseCycle);
    assert_eq!(prog.current_zone_id, EXPANSE_ZONE_ID);
    assert_eq!(prog.current_subzone_id, 1);
}

// =========================================================================
// NON-CAP FRACTURE ZONE ADVANCEMENT: Z13 → Z14 within a chapter
// =========================================================================

#[test]
fn test_zone_13_boss_with_cap_14_advances_to_zone_14() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    prog.current_zone_id = 13;
    prog.current_subzone_id = 5; // final subzone of Z13
    prog.unlock_zone(13);
    prog.unlock_zone(14);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14, 30);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 14);
        }
        _ => panic!("Expected ZoneComplete to zone 14, got {:?}", result),
    }
    assert_eq!(prog.current_zone_id, 14);
    assert_eq!(prog.current_subzone_id, 1);
}

// =========================================================================
// FRACTURE CYCLE RECORDS BOSS DEFEAT
// =========================================================================

#[test]
fn test_fracture_cycle_records_boss_defeat_in_defeated_bosses() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    prog.current_zone_id = 14;
    prog.current_subzone_id = 5;
    prog.unlock_zone(14);
    prog.fighting_boss = true;

    assert!(!prog.is_boss_defeated(14, 5));

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14, 30);
    assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 14 });

    // Boss defeat should be recorded even when cycling
    assert!(prog.is_boss_defeated(14, 5));
}

// =========================================================================
// MULTIPLE CONSECUTIVE FRACTURE CYCLES (looping behavior)
// =========================================================================

#[test]
fn test_fracture_cap_zone_can_be_defeated_multiple_times() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    prog.current_zone_id = 17;
    prog.unlock_zone(17);

    for cycle in 0..3 {
        // Clear subzones 1-4
        for subzone in 1u32..=4 {
            prog.current_subzone_id = subzone;
            for _ in 0..KILLS_FOR_BOSS {
                prog.record_kill();
            }
            let result = prog.on_boss_defeated_with_cap(75, &mut achievements, 17, 30);
            assert!(
                matches!(result, BossDefeatResult::SubzoneComplete { .. }),
                "Cycle {cycle}, subzone {subzone}: expected SubzoneComplete, got {:?}",
                result
            );
        }

        // Defeat zone boss (subzone 5)
        prog.current_subzone_id = 5;
        for _ in 0..KILLS_FOR_BOSS {
            prog.record_kill();
        }
        let result = prog.on_boss_defeated_with_cap(75, &mut achievements, 17, 30);
        assert_eq!(
            result,
            BossDefeatResult::FractureCycle { zone_id: 17 },
            "Cycle {cycle}: expected FractureCycle"
        );
        assert_eq!(
            prog.current_zone_id, 17,
            "Should remain in zone 17 after cycle"
        );
        assert_eq!(
            prog.current_subzone_id, 1,
            "Should reset to subzone 1 after cycle"
        );
        assert_eq!(
            prog.kills_in_subzone, 0,
            "Kills should be reset after cycle"
        );
    }
}

// =========================================================================
// PRESTIGE RESET WHEN AT FRACTURE ZONE
// =========================================================================

#[test]
fn test_prestige_reset_from_fracture_zone_clears_state() {
    let mut prog = ZoneProgression::new();

    // Simulate player at Zone 14 with progress
    prog.current_zone_id = 14;
    prog.current_subzone_id = 3;
    prog.unlock_zone(12);
    prog.unlock_zone(13);
    prog.unlock_zone(14);
    prog.kills_in_subzone = 7;
    prog.fighting_boss = false;
    prog.defeat_boss(12, 1);
    prog.defeat_boss(12, 2);
    prog.defeat_boss(13, 5);

    // Prestige reset (P50 allows zones up through Z14 by prestige, but fracture zones
    // have prestige_requirement = 0 so they are re-unlocked by prestige alone)
    prog.reset_for_prestige(50);

    // Must return to Zone 1, Subzone 1
    assert_eq!(prog.current_zone_id, 1);
    assert_eq!(prog.current_subzone_id, 1);
    assert_eq!(prog.kills_in_subzone, 0);
    assert!(!prog.fighting_boss);
    assert!(prog.defeated_bosses.is_empty());

    // Zones 1-11 should be unlocked (prestige_requirement <= 50 for all pre-endgame zones)
    for zone_id in 1..=10 {
        assert!(
            prog.is_zone_unlocked(zone_id),
            "Zone {zone_id} should be unlocked at P50"
        );
    }
    // Zone 11 (Expanse) has prestige_requirement = 0 so it gets re-unlocked too
    assert!(prog.is_zone_unlocked(EXPANSE_ZONE_ID));
    // Fracture zones also have prestige_requirement = 0, so they get re-unlocked by reset
    // (Access control for fracture zones is managed separately via sync_account_zone_unlocks)
}

#[test]
fn test_prestige_reset_from_mid_fight_at_fracture_zone() {
    let mut prog = ZoneProgression::new();

    // Player in mid-fight at Zone 20 boss
    prog.current_zone_id = 20;
    prog.current_subzone_id = 5;
    prog.unlock_zone(20);
    prog.kills_in_subzone = KILLS_FOR_BOSS;
    prog.fighting_boss = true;
    prog.defeat_boss(20, 1);
    prog.defeat_boss(20, 2);
    prog.defeat_boss(20, 3);
    prog.defeat_boss(20, 4);

    prog.reset_for_prestige(100);

    assert_eq!(prog.current_zone_id, 1);
    assert_eq!(prog.current_subzone_id, 1);
    assert_eq!(prog.kills_in_subzone, 0);
    assert!(!prog.fighting_boss);
    assert!(prog.defeated_bosses.is_empty());
}

// =========================================================================
// on_boss_defeated (NON-CAP) ALWAYS CYCLES Z11 REGARDLESS OF FRACTURE ZONES
// =========================================================================

#[test]
fn test_on_boss_defeated_non_cap_always_cycles_z11_even_with_z12_unlocked() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    // Unlock fracture zones — but using the non-cap method, Z11 should still cycle
    prog.current_zone_id = EXPANSE_ZONE_ID;
    prog.current_subzone_id = 4;
    prog.unlock_zone(EXPANSE_ZONE_ID);
    prog.unlock_zone(12);
    prog.unlock_zone(13);
    prog.unlock_zone(14);
    prog.fighting_boss = true;

    // The non-cap variant should always return ExpanseCycle (no fracture awareness)
    let result = prog.on_boss_defeated(50, &mut achievements);
    assert_eq!(result, BossDefeatResult::ExpanseCycle);
    assert_eq!(prog.current_zone_id, EXPANSE_ZONE_ID);
    assert_eq!(prog.current_subzone_id, 1);
}

// =========================================================================
// EXPANSE → Z12 ADVANCE: VERIFY SUBZONE RESETS TO 1
// =========================================================================

#[test]
fn test_zone_11_advances_to_zone_12_and_resets_subzone_to_1() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    // Player at Zone 11, subzone 4 (zone boss)
    prog.current_zone_id = EXPANSE_ZONE_ID;
    prog.current_subzone_id = 4;
    prog.unlock_zone(EXPANSE_ZONE_ID);
    prog.unlock_zone(12);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14, 30);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 12);
        }
        _ => panic!("Expected ZoneComplete to zone 12, got {:?}", result),
    }
    // Verify position state after advancing
    assert_eq!(prog.current_zone_id, 12);
    assert_eq!(prog.current_subzone_id, 1);
    assert_eq!(prog.kills_in_subzone, 0);
    assert!(!prog.fighting_boss);
}

// =========================================================================
// CHAPTER BOUNDARY TRANSITIONS
// =========================================================================

#[test]
fn test_chapter_boundary_z14_to_z15_when_cap_is_17() {
    // Z14 is in Ch.1 (RedFault), Z15 is in Ch.2 (MirrorScar)
    // With cap=17, defeating Z14's zone boss should advance to Z15
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    prog.current_zone_id = 14;
    prog.current_subzone_id = 5; // final subzone of Z14
    prog.unlock_zone(14);
    prog.unlock_zone(15);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(75, &mut achievements, 17, 30);
    match result {
        BossDefeatResult::ZoneComplete {
            old_zone,
            new_zone_id,
        } => {
            assert_eq!(
                new_zone_id, 15,
                "Should advance to Z15 (first zone of Ch.2)"
            );
            // Zone name should be from Ch.1
            assert!(!old_zone.is_empty());
        }
        _ => panic!("Expected ZoneComplete to zone 15, got {:?}", result),
    }
    assert_eq!(prog.current_zone_id, 15);
    assert_eq!(prog.current_subzone_id, 1);
}

#[test]
fn test_chapter_boundary_z20_to_z21_when_cap_is_23() {
    // Z20 is in Ch.3 (BlackMouth), Z21 is in Ch.4 (HollowThrone)
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    prog.current_zone_id = 20;
    prog.current_subzone_id = 5;
    prog.unlock_zone(20);
    prog.unlock_zone(21);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(150, &mut achievements, 23, 30);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 21);
        }
        _ => panic!("Expected ZoneComplete to zone 21, got {:?}", result),
    }
    assert_eq!(prog.current_zone_id, 21);
    assert_eq!(prog.current_subzone_id, 1);
}

#[test]
fn test_chapter_boundary_z26_to_z27_when_cap_is_30() {
    // Z26 is in Ch.5 (WailingReach), Z27 is in Ch.6 (OriginWound)
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    prog.current_zone_id = 26;
    prog.current_subzone_id = 5;
    prog.unlock_zone(26);
    prog.unlock_zone(27);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(300, &mut achievements, 30, 30);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 27);
        }
        _ => panic!("Expected ZoneComplete to zone 27, got {:?}", result),
    }
    assert_eq!(prog.current_zone_id, 27);
    assert_eq!(prog.current_subzone_id, 1);
}

// =========================================================================
// ZONE 30: PERMANENT LOOP — HIGHEST POSSIBLE CAP
// =========================================================================

#[test]
fn test_zone_30_is_permanent_cap_and_cycles_indefinitely() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    // Zone 30 with cap=30: should always return FractureCycle regardless of prestige
    prog.current_zone_id = 30;
    prog.unlock_zone(30);

    for cycle in 0..3 {
        // Clear subzones 1-4
        for subzone in 1u32..=4 {
            prog.current_subzone_id = subzone;
            for _ in 0..KILLS_FOR_BOSS {
                prog.record_kill();
            }
            let result = prog.on_boss_defeated_with_cap(300, &mut achievements, 30, 30);
            assert!(
                matches!(result, BossDefeatResult::SubzoneComplete { .. }),
                "Cycle {cycle}, subzone {subzone}: expected SubzoneComplete"
            );
        }

        // Zone boss (subzone 5)
        prog.current_subzone_id = 5;
        for _ in 0..KILLS_FOR_BOSS {
            prog.record_kill();
        }
        let result = prog.on_boss_defeated_with_cap(300, &mut achievements, 30, 30);
        assert_eq!(
            result,
            BossDefeatResult::FractureCycle { zone_id: 30 },
            "Cycle {cycle}: Zone 30 must always return FractureCycle"
        );
        assert_eq!(prog.current_zone_id, 30);
        assert_eq!(prog.current_subzone_id, 1);
        // There is no zone 31 to advance to
    }
}

// =========================================================================
// EDGE CASE: Z11 WITH CAP = 12 (single fracture zone unlocked)
// =========================================================================

#[test]
fn test_zone_11_boss_with_cap_12_and_zone_12_unlocked_advances_to_zone_12() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    prog.current_zone_id = EXPANSE_ZONE_ID;
    prog.current_subzone_id = 4;
    prog.unlock_zone(EXPANSE_ZONE_ID);
    prog.unlock_zone(12);
    prog.fighting_boss = true;

    // cap=12 means only zone 12 is in fracture range (unusual edge case, but valid)
    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 12, 30);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 12);
        }
        _ => panic!("Expected ZoneComplete to zone 12, got {:?}", result),
    }
}

#[test]
fn test_zone_12_boss_with_cap_12_returns_fracture_cycle() {
    // Z12 is the cap zone when cap=12; defeating its boss should cycle
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    prog.current_zone_id = 12;
    prog.current_subzone_id = 5;
    prog.unlock_zone(12);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 12, 30);
    assert_eq!(result, BossDefeatResult::FractureCycle { zone_id: 12 });
    assert_eq!(prog.current_subzone_id, 1);
    assert_eq!(prog.kills_in_subzone, 0);
}
