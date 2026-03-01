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

    let result = prog.on_boss_defeated_with_cap(25, &mut achievements, 11);
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

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14);
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

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14);
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

    let result = prog.on_boss_defeated_with_cap(75, &mut achievements, 17);
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

    let result = prog.on_boss_defeated_with_cap(100, &mut achievements, 20);
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

    let result = prog.on_boss_defeated_with_cap(150, &mut achievements, 23);
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

    let result = prog.on_boss_defeated_with_cap(150, &mut achievements, 23);
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

    let result = prog.on_boss_defeated_with_cap(200, &mut achievements, 26);
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

    let result = prog.on_boss_defeated_with_cap(200, &mut achievements, 26);
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

    let result = prog.on_boss_defeated_with_cap(300, &mut achievements, 30);
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

    let result = prog.on_boss_defeated_with_cap(300, &mut achievements, 30);
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

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14);
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

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14);
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

    let result = prog.on_boss_defeated_with_cap(75, &mut achievements, 17);
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

    let result = prog.on_boss_defeated_with_cap(100, &mut achievements, 20);
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

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 14);
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
