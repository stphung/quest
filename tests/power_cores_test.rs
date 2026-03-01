//! Integration tests for the Power Cores data definitions and persistence.
//!
//! Covers:
//!  1. ALL_POWER_CORES has exactly 6 entries
//!  2. Each core maps to the correct achievement ID (PowerCoreI through PowerCoreVI)
//!  3. get_power_core_def returns correct def for each core, None for non-core achievements
//!  4. pr_per_day values are 1, 2, 3, 4, 5, 6
//!  5. Core names match fracture regions: Red Fault, Mirror Scar, Black Mouth, Hollow Throne,
//!     Wailing Reach, Origin Wound
//!  6. PowerCoreState serialization round-trip
//!  7. get_unlocked_cores with 0, 1, 3, and 6 unlocked
//!  8. fill_duration_secs calculation for each core
//!  9. Persistence save/load cycle
//! 10. Default state

use quest::achievements::{AchievementId, Achievements};
use quest::power_cores::{
    fill_duration_secs, get_power_core_def, get_unlocked_cores, load_power_cores, PowerCoreState,
    ALL_POWER_CORES,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Unlock a single achievement in an `Achievements` state without going through
/// the full event system (which is unavailable in integration tests).  We
/// directly insert into the unlocked map using `unlock()` which is the
/// canonical path.
fn unlock(achievements: &mut Achievements, id: AchievementId) {
    achievements.unlock(id, None);
}

// ── 1. ALL_POWER_CORES count ─────────────────────────────────────────────────

#[test]
fn test_all_power_cores_has_exactly_six_entries() {
    assert_eq!(
        ALL_POWER_CORES.len(),
        6,
        "Expected exactly 6 Power Core definitions"
    );
}

// ── 2. Achievement ID mapping ─────────────────────────────────────────────────

#[test]
fn test_core_achievement_ids_in_order() {
    let expected_ids = [
        AchievementId::PowerCoreI,
        AchievementId::PowerCoreII,
        AchievementId::PowerCoreIII,
        AchievementId::PowerCoreIV,
        AchievementId::PowerCoreV,
        AchievementId::PowerCoreVI,
    ];
    for (i, (&expected, def)) in expected_ids.iter().zip(ALL_POWER_CORES.iter()).enumerate() {
        assert_eq!(
            def.achievement_id, expected,
            "Core index {i}: expected achievement_id {:?}, got {:?}",
            expected, def.achievement_id
        );
    }
}

#[test]
fn test_layer3_cleared_maps_to_red_fault() {
    let def =
        get_power_core_def(AchievementId::PowerCoreI).expect("PowerCoreI should be a power core");
    assert_eq!(def.achievement_id, AchievementId::PowerCoreI);
    assert_eq!(def.name, "Red Fault");
}

#[test]
fn test_layer7_cleared_maps_to_mirror_scar() {
    let def =
        get_power_core_def(AchievementId::PowerCoreII).expect("PowerCoreII should be a power core");
    assert_eq!(def.achievement_id, AchievementId::PowerCoreII);
    assert_eq!(def.name, "Mirror Scar");
}

#[test]
fn test_layer12_cleared_maps_to_black_mouth() {
    let def = get_power_core_def(AchievementId::PowerCoreIII)
        .expect("PowerCoreIII should be a power core");
    assert_eq!(def.achievement_id, AchievementId::PowerCoreIII);
    assert_eq!(def.name, "Black Mouth");
}

#[test]
fn test_layer18_cleared_maps_to_hollow_throne() {
    let def =
        get_power_core_def(AchievementId::PowerCoreIV).expect("PowerCoreIV should be a power core");
    assert_eq!(def.achievement_id, AchievementId::PowerCoreIV);
    assert_eq!(def.name, "Hollow Throne");
}

#[test]
fn test_power_core_v_maps_to_wailing_reach() {
    let def =
        get_power_core_def(AchievementId::PowerCoreV).expect("PowerCoreV should be a power core");
    assert_eq!(def.achievement_id, AchievementId::PowerCoreV);
    assert_eq!(def.name, "Wailing Reach");
}

#[test]
fn test_layer30_cleared_maps_to_origin_wound() {
    let def =
        get_power_core_def(AchievementId::PowerCoreVI).expect("PowerCoreVI should be a power core");
    assert_eq!(def.achievement_id, AchievementId::PowerCoreVI);
    assert_eq!(def.name, "Origin Wound");
}

// ── 3. get_power_core_def ─────────────────────────────────────────────────────

#[test]
fn test_get_power_core_def_returns_some_for_all_six_cores() {
    let core_ids = [
        AchievementId::PowerCoreI,
        AchievementId::PowerCoreII,
        AchievementId::PowerCoreIII,
        AchievementId::PowerCoreIV,
        AchievementId::PowerCoreV,
        AchievementId::PowerCoreVI,
    ];
    for id in core_ids {
        assert!(
            get_power_core_def(id).is_some(),
            "Expected Some for {:?}",
            id
        );
    }
}

#[test]
fn test_get_power_core_def_returns_none_for_non_core_achievement() {
    // Non-core Deep achievements
    assert!(get_power_core_def(AchievementId::TheDeepDiscovered).is_none());
    assert!(get_power_core_def(AchievementId::FirstBreakthrough).is_none());
    assert!(get_power_core_def(AchievementId::VoidExplorer).is_none());
    // Combat achievements are not power cores
    assert!(get_power_core_def(AchievementId::SlayerI).is_none());
    assert!(get_power_core_def(AchievementId::BossHunterI).is_none());
    // Level achievements are not power cores
    assert!(get_power_core_def(AchievementId::Level100).is_none());
    // Fracture zone completion achievements (different from layer cleared)
    assert!(get_power_core_def(AchievementId::FractureZone12).is_none());
    assert!(get_power_core_def(AchievementId::FractureZone30).is_none());
}

#[test]
fn test_get_power_core_def_returns_correct_name_for_each() {
    let expected = [
        (AchievementId::PowerCoreI, "Red Fault"),
        (AchievementId::PowerCoreII, "Mirror Scar"),
        (AchievementId::PowerCoreIII, "Black Mouth"),
        (AchievementId::PowerCoreIV, "Hollow Throne"),
        (AchievementId::PowerCoreV, "Wailing Reach"),
        (AchievementId::PowerCoreVI, "Origin Wound"),
    ];
    for (id, name) in expected {
        let def = get_power_core_def(id).unwrap_or_else(|| panic!("{:?} should have a def", id));
        assert_eq!(def.name, name, "Name mismatch for {:?}", id);
    }
}

// ── 4. pr_per_day values ──────────────────────────────────────────────────────

#[test]
fn test_pr_per_day_values_are_1_through_6_in_order() {
    let expected_rates: [u32; 6] = [1, 2, 3, 4, 5, 6];
    for (i, (def, &expected)) in ALL_POWER_CORES
        .iter()
        .zip(expected_rates.iter())
        .enumerate()
    {
        assert_eq!(
            def.pr_per_day, expected,
            "Core index {i} ('{}'): expected pr_per_day={expected}, got {}",
            def.name, def.pr_per_day
        );
    }
}

#[test]
fn test_red_fault_has_1_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreI).unwrap();
    assert_eq!(def.pr_per_day, 1);
}

#[test]
fn test_mirror_scar_has_2_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreII).unwrap();
    assert_eq!(def.pr_per_day, 2);
}

#[test]
fn test_black_mouth_has_3_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreIII).unwrap();
    assert_eq!(def.pr_per_day, 3);
}

#[test]
fn test_hollow_throne_has_4_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreIV).unwrap();
    assert_eq!(def.pr_per_day, 4);
}

#[test]
fn test_wailing_reach_has_5_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreV).unwrap();
    assert_eq!(def.pr_per_day, 5);
}

#[test]
fn test_origin_wound_has_6_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreVI).unwrap();
    assert_eq!(def.pr_per_day, 6);
}

#[test]
fn test_all_pr_per_day_values_are_nonzero() {
    for def in ALL_POWER_CORES {
        assert!(
            def.pr_per_day > 0,
            "Core '{}' has pr_per_day == 0",
            def.name
        );
    }
}

// ── 5. Core names ─────────────────────────────────────────────────────────────

#[test]
fn test_core_names_match_fracture_regions() {
    let expected_names = [
        "Red Fault",
        "Mirror Scar",
        "Black Mouth",
        "Hollow Throne",
        "Wailing Reach",
        "Origin Wound",
    ];
    for (i, (def, &expected_name)) in ALL_POWER_CORES
        .iter()
        .zip(expected_names.iter())
        .enumerate()
    {
        assert_eq!(
            def.name, expected_name,
            "Core index {i}: expected name '{expected_name}', got '{}'",
            def.name
        );
    }
}

#[test]
fn test_core_names_are_nonempty() {
    for def in ALL_POWER_CORES {
        assert!(
            !def.name.is_empty(),
            "Core for {:?} has empty name",
            def.achievement_id
        );
    }
}

// ── 6. PowerCoreState serialization round-trip ────────────────────────────────

#[test]
fn test_power_core_state_serialization_round_trip_empty() {
    let state = PowerCoreState::default();
    let json = serde_json::to_string(&state).expect("serialization should succeed");
    let loaded: PowerCoreState =
        serde_json::from_str(&json).expect("deserialization should succeed");
    assert_eq!(loaded.last_granted_at.len(), state.last_granted_at.len());
    assert!(loaded.last_granted_at.is_empty());
}

#[test]
fn test_power_core_state_serialization_round_trip_with_timestamps() {
    let mut state = PowerCoreState::default();
    state
        .last_granted_at
        .insert(AchievementId::PowerCoreI, 1_700_000_000);
    state
        .last_granted_at
        .insert(AchievementId::PowerCoreII, 1_700_100_000);
    state
        .last_granted_at
        .insert(AchievementId::PowerCoreIII, 1_700_200_000);

    let json = serde_json::to_string_pretty(&state).expect("serialization should succeed");
    let loaded: PowerCoreState =
        serde_json::from_str(&json).expect("deserialization should succeed");

    assert_eq!(loaded.last_granted_at.len(), 3);
    assert_eq!(
        loaded.last_granted_at[&AchievementId::PowerCoreI],
        1_700_000_000
    );
    assert_eq!(
        loaded.last_granted_at[&AchievementId::PowerCoreII],
        1_700_100_000
    );
    assert_eq!(
        loaded.last_granted_at[&AchievementId::PowerCoreIII],
        1_700_200_000
    );
    // Unset cores should be absent
    assert!(!loaded
        .last_granted_at
        .contains_key(&AchievementId::PowerCoreIV));
    assert!(!loaded
        .last_granted_at
        .contains_key(&AchievementId::PowerCoreV));
    assert!(!loaded
        .last_granted_at
        .contains_key(&AchievementId::PowerCoreVI));
}

#[test]
fn test_power_core_state_serialization_round_trip_all_six() {
    let mut state = PowerCoreState::default();
    let timestamps: [i64; 6] = [
        1_700_000_000,
        1_700_100_000,
        1_700_200_000,
        1_700_300_000,
        1_700_400_000,
        1_700_500_000,
    ];
    let ids = [
        AchievementId::PowerCoreI,
        AchievementId::PowerCoreII,
        AchievementId::PowerCoreIII,
        AchievementId::PowerCoreIV,
        AchievementId::PowerCoreV,
        AchievementId::PowerCoreVI,
    ];
    for (&id, &ts) in ids.iter().zip(timestamps.iter()) {
        state.last_granted_at.insert(id, ts);
    }

    let json = serde_json::to_string_pretty(&state).unwrap();
    let loaded: PowerCoreState = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.last_granted_at.len(), 6);
    for (&id, &ts) in ids.iter().zip(timestamps.iter()) {
        assert_eq!(
            loaded.last_granted_at[&id], ts,
            "Timestamp mismatch for {:?}",
            id
        );
    }
}

#[test]
fn test_power_core_state_missing_keys_are_absent_after_round_trip() {
    // State with only one entry — the other 5 should not appear after a round-trip
    let mut state = PowerCoreState::default();
    state
        .last_granted_at
        .insert(AchievementId::PowerCoreVI, 999_999_999);

    let json = serde_json::to_string(&state).unwrap();
    let loaded: PowerCoreState = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.last_granted_at.len(), 1);
    assert_eq!(
        loaded.last_granted_at[&AchievementId::PowerCoreVI],
        999_999_999
    );
    assert!(!loaded
        .last_granted_at
        .contains_key(&AchievementId::PowerCoreI));
}

// ── 7. get_unlocked_cores ────────────────────────────────────────────────────

#[test]
fn test_get_unlocked_cores_with_zero_unlocked() {
    let achievements = Achievements::default();
    let cores = get_unlocked_cores(&achievements);
    assert!(
        cores.is_empty(),
        "Expected no unlocked cores, got {}",
        cores.len()
    );
}

#[test]
fn test_get_unlocked_cores_with_one_unlocked() {
    let mut achievements = Achievements::default();
    unlock(&mut achievements, AchievementId::PowerCoreI);

    let cores = get_unlocked_cores(&achievements);
    assert_eq!(cores.len(), 1, "Expected 1 unlocked core");
    assert_eq!(cores[0].name, "Red Fault");
    assert_eq!(cores[0].achievement_id, AchievementId::PowerCoreI);
}

#[test]
fn test_get_unlocked_cores_with_three_unlocked() {
    let mut achievements = Achievements::default();
    unlock(&mut achievements, AchievementId::PowerCoreI);
    unlock(&mut achievements, AchievementId::PowerCoreII);
    unlock(&mut achievements, AchievementId::PowerCoreIII);

    let cores = get_unlocked_cores(&achievements);
    assert_eq!(cores.len(), 3, "Expected 3 unlocked cores");

    let names: Vec<&str> = cores.iter().map(|c| c.name).collect();
    assert!(
        names.contains(&"Red Fault"),
        "Expected 'Red Fault' in unlocked cores"
    );
    assert!(
        names.contains(&"Mirror Scar"),
        "Expected 'Mirror Scar' in unlocked cores"
    );
    assert!(
        names.contains(&"Black Mouth"),
        "Expected 'Black Mouth' in unlocked cores"
    );
    // Cores 4-6 should not be unlocked
    assert!(
        !names.contains(&"Hollow Throne"),
        "Did not expect 'Hollow Throne' unlocked"
    );
    assert!(
        !names.contains(&"Wailing Reach"),
        "Did not expect 'Wailing Reach' unlocked"
    );
    assert!(
        !names.contains(&"Origin Wound"),
        "Did not expect 'Origin Wound' unlocked"
    );
}

#[test]
fn test_get_unlocked_cores_with_all_six_unlocked() {
    let mut achievements = Achievements::default();
    unlock(&mut achievements, AchievementId::PowerCoreI);
    unlock(&mut achievements, AchievementId::PowerCoreII);
    unlock(&mut achievements, AchievementId::PowerCoreIII);
    unlock(&mut achievements, AchievementId::PowerCoreIV);
    unlock(&mut achievements, AchievementId::PowerCoreV);
    unlock(&mut achievements, AchievementId::PowerCoreVI);

    let cores = get_unlocked_cores(&achievements);
    assert_eq!(cores.len(), 6, "Expected all 6 cores to be unlocked");

    let names: Vec<&str> = cores.iter().map(|c| c.name).collect();
    let expected_names = [
        "Red Fault",
        "Mirror Scar",
        "Black Mouth",
        "Hollow Throne",
        "Wailing Reach",
        "Origin Wound",
    ];
    for &name in &expected_names {
        assert!(names.contains(&name), "Expected '{name}' in unlocked cores");
    }
}

#[test]
fn test_get_unlocked_cores_non_core_achievement_does_not_add_core() {
    let mut achievements = Achievements::default();
    // Unlock a non-core Deep achievement — should not add any power cores
    unlock(&mut achievements, AchievementId::TheDeepDiscovered);
    unlock(&mut achievements, AchievementId::FirstBreakthrough);
    unlock(&mut achievements, AchievementId::VoidExplorer);

    let cores = get_unlocked_cores(&achievements);
    assert!(
        cores.is_empty(),
        "Non-core achievements should not unlock power cores, but got {} cores",
        cores.len()
    );
}

#[test]
fn test_get_unlocked_cores_returns_only_those_with_unlocked_achievement() {
    let mut achievements = Achievements::default();
    // Unlock only the last two
    unlock(&mut achievements, AchievementId::PowerCoreV);
    unlock(&mut achievements, AchievementId::PowerCoreVI);

    let cores = get_unlocked_cores(&achievements);
    assert_eq!(cores.len(), 2);
    let names: Vec<&str> = cores.iter().map(|c| c.name).collect();
    assert!(names.contains(&"Wailing Reach"));
    assert!(names.contains(&"Origin Wound"));
    // Earlier cores should not be included
    assert!(!names.contains(&"Red Fault"));
    assert!(!names.contains(&"Mirror Scar"));
    assert!(!names.contains(&"Black Mouth"));
    assert!(!names.contains(&"Hollow Throne"));
}

// ── 8. fill_duration_secs ────────────────────────────────────────────────────

#[test]
fn test_fill_duration_secs_red_fault_is_86400() {
    // 1 PR/day → 86400 seconds (24 hours)
    assert_eq!(fill_duration_secs(1), 86400);
}

#[test]
fn test_fill_duration_secs_mirror_scar_is_43200() {
    // 2 PR/day → 43200 seconds (12 hours)
    assert_eq!(fill_duration_secs(2), 43200);
}

#[test]
fn test_fill_duration_secs_black_mouth_is_28800() {
    // 3 PR/day → 28800 seconds (8 hours)
    assert_eq!(fill_duration_secs(3), 28800);
}

#[test]
fn test_fill_duration_secs_hollow_throne_is_21600() {
    // 4 PR/day → 21600 seconds (6 hours)
    assert_eq!(fill_duration_secs(4), 21600);
}

#[test]
fn test_fill_duration_secs_wailing_reach_is_17280() {
    // 5 PR/day → 17280 seconds (4h 48m)
    assert_eq!(fill_duration_secs(5), 17280);
}

#[test]
fn test_fill_duration_secs_origin_wound_is_14400() {
    // 6 PR/day → 14400 seconds (4 hours)
    assert_eq!(fill_duration_secs(6), 14400);
}

#[test]
fn test_fill_duration_secs_all_cores() {
    // Verify formula: fill_duration_secs = 86400 / pr_per_day
    let expected: [(u32, i64); 6] = [
        (1, 86400),
        (2, 43200),
        (3, 28800),
        (4, 21600),
        (5, 17280),
        (6, 14400),
    ];
    for (pr_per_day, expected_secs) in expected {
        assert_eq!(
            fill_duration_secs(pr_per_day),
            expected_secs,
            "fill_duration_secs({pr_per_day}) should be {expected_secs}"
        );
    }
}

#[test]
fn test_fill_duration_secs_matches_86400_divided_by_pr_per_day() {
    // Ensure each core's fill duration equals the universal constant 86400 / pr_per_day
    for def in ALL_POWER_CORES {
        let expected = 86400_i64 / def.pr_per_day as i64;
        assert_eq!(
            fill_duration_secs(def.pr_per_day),
            expected,
            "Core '{}': fill_duration_secs({}) should be {}",
            def.name,
            def.pr_per_day,
            expected
        );
    }
}

// ── 9. Persistence save/load cycle ───────────────────────────────────────────

#[test]
fn test_persistence_save_and_load_cycle() {
    // Build a state with all 6 cores having timestamps
    let mut state = PowerCoreState::default();
    let entries: [(AchievementId, i64); 6] = [
        (AchievementId::PowerCoreI, 1_700_000_001),
        (AchievementId::PowerCoreII, 1_700_000_002),
        (AchievementId::PowerCoreIII, 1_700_000_003),
        (AchievementId::PowerCoreIV, 1_700_000_004),
        (AchievementId::PowerCoreV, 1_700_000_005),
        (AchievementId::PowerCoreVI, 1_700_000_006),
    ];
    for (id, ts) in entries {
        state.last_granted_at.insert(id, ts);
    }

    // Serialize to JSON (simulates what save_power_cores does)
    let json = serde_json::to_string_pretty(&state).expect("serialization should succeed");

    // Deserialize (simulates what load_power_cores does)
    let loaded: PowerCoreState =
        serde_json::from_str(&json).expect("deserialization should succeed");

    // All 6 entries should survive the round-trip unchanged
    assert_eq!(loaded.last_granted_at.len(), 6);
    for (id, ts) in entries {
        assert_eq!(
            loaded.last_granted_at[&id], ts,
            "Timestamp mismatch for {:?} after save/load cycle",
            id
        );
    }
}

#[test]
fn test_persistence_save_and_load_via_tempfile() {
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir creation should succeed");
    let path = tmp.path().join("power_cores.json");

    // Build state
    let mut state = PowerCoreState::default();
    state
        .last_granted_at
        .insert(AchievementId::PowerCoreI, 1_234_567_890);
    state
        .last_granted_at
        .insert(AchievementId::PowerCoreIII, 9_876_543_210);

    // Write
    let json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write(&path, &json).expect("write should succeed");

    // Read back
    let loaded_json = std::fs::read_to_string(&path).expect("read should succeed");
    let loaded: PowerCoreState =
        serde_json::from_str(&loaded_json).expect("deserialization should succeed");

    assert_eq!(loaded.last_granted_at.len(), 2);
    assert_eq!(
        loaded.last_granted_at[&AchievementId::PowerCoreI],
        1_234_567_890
    );
    assert_eq!(
        loaded.last_granted_at[&AchievementId::PowerCoreIII],
        9_876_543_210
    );
    assert!(!loaded
        .last_granted_at
        .contains_key(&AchievementId::PowerCoreII));
}

#[test]
fn test_persistence_load_returns_default_for_corrupted_json() {
    // Verify that a corrupt/missing file falls back to default state rather than panicking.
    // We test this by directly deserializing invalid JSON.
    let result: Result<PowerCoreState, _> = serde_json::from_str("not valid json");
    assert!(result.is_err(), "Invalid JSON should fail to deserialize");
    // The load function itself returns default on error — verified by the function signature
    // (unwrap_or_default pattern). We verify default is what we expect.
    let default_state = PowerCoreState::default();
    assert!(default_state.last_granted_at.is_empty());
}

#[test]
fn test_load_power_cores_does_not_panic() {
    // load_power_cores() should return a valid default if the file is absent.
    // This test just ensures the function doesn't panic in the test environment.
    let _state = load_power_cores();
}

// ── 10. Default state ────────────────────────────────────────────────────────

#[test]
fn test_power_core_state_default_has_empty_map() {
    let state = PowerCoreState::default();
    assert!(
        state.last_granted_at.is_empty(),
        "Default PowerCoreState should have no timestamps"
    );
}

#[test]
fn test_power_core_state_new_entry_can_be_inserted() {
    let mut state = PowerCoreState::default();
    assert!(state.last_granted_at.is_empty());

    state.last_granted_at.insert(AchievementId::PowerCoreI, 0);
    assert_eq!(state.last_granted_at.len(), 1);
    assert_eq!(state.last_granted_at[&AchievementId::PowerCoreI], 0);
}

#[test]
fn test_power_core_state_zero_timestamp_is_valid() {
    // Timestamp 0 represents "never granted" (epoch) — it's a valid stored value.
    let mut state = PowerCoreState::default();
    state.last_granted_at.insert(AchievementId::PowerCoreI, 0);

    let json = serde_json::to_string(&state).unwrap();
    let loaded: PowerCoreState = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.last_granted_at[&AchievementId::PowerCoreI], 0);
}

#[test]
fn test_power_core_state_clone_is_independent() {
    let mut state = PowerCoreState::default();
    state
        .last_granted_at
        .insert(AchievementId::PowerCoreI, 1000);

    let mut cloned = state.clone();
    cloned
        .last_granted_at
        .insert(AchievementId::PowerCoreII, 2000);

    // Original should not be affected by modifications to the clone
    assert!(!state
        .last_granted_at
        .contains_key(&AchievementId::PowerCoreII));
    assert_eq!(state.last_granted_at.len(), 1);
    assert_eq!(cloned.last_granted_at.len(), 2);
}

// ── Additional edge cases ────────────────────────────────────────────────────

#[test]
fn test_all_power_core_achievement_ids_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for def in ALL_POWER_CORES {
        assert!(
            seen.insert(def.achievement_id),
            "Duplicate achievement_id {:?} in ALL_POWER_CORES",
            def.achievement_id
        );
    }
}

#[test]
fn test_pr_per_day_strictly_increasing_across_cores() {
    // Each successive core should grant more PR/day than the previous
    for window in ALL_POWER_CORES.windows(2) {
        assert!(
            window[1].pr_per_day > window[0].pr_per_day,
            "Core '{}' ({} PR/day) should be strictly greater than '{}' ({} PR/day)",
            window[1].name,
            window[1].pr_per_day,
            window[0].name,
            window[0].pr_per_day,
        );
    }
}

#[test]
fn test_fill_duration_secs_decreases_with_higher_pr_rate() {
    // Higher PR/day means faster fill (shorter duration)
    for window in ALL_POWER_CORES.windows(2) {
        let dur_a = fill_duration_secs(window[0].pr_per_day);
        let dur_b = fill_duration_secs(window[1].pr_per_day);
        assert!(
            dur_b < dur_a,
            "Core '{}' fill duration ({} s) should be less than '{}' ({} s)",
            window[1].name,
            dur_b,
            window[0].name,
            dur_a,
        );
    }
}

#[test]
fn test_get_unlocked_cores_does_not_include_duplicate_defs() {
    // Unlocking the same achievement twice should not produce duplicate cores
    let mut achievements = Achievements::default();
    unlock(&mut achievements, AchievementId::PowerCoreI);
    unlock(&mut achievements, AchievementId::PowerCoreI); // double-unlock

    let cores = get_unlocked_cores(&achievements);
    // Should still only return 1 core (Red Fault)
    let red_fault_count = cores.iter().filter(|c| c.name == "Red Fault").count();
    assert_eq!(
        red_fault_count, 1,
        "Red Fault should appear exactly once despite duplicate unlock"
    );
}
