//! Integration tests for the Power Cores data definitions and persistence.
//!
//! Covers:
//!  1. ALL_POWER_CORES has exactly 6 entries
//!  2. Each core maps to the correct achievement ID (PowerCoreI through PowerCoreVI)
//!  3. get_power_core_def returns correct def for each core, None for non-core achievements
//!  4. pr_per_day values are 2, 3, 5, 8, 12, 18
//!  5. Core names match fracture regions: Red Fault, Mirror Scar, Black Mouth, Hollow Throne,
//!     Wailing Reach, Origin Wound
//!  6. PassivesState serialization round-trip
//!  7. get_unlocked_cores with 0, 1, 3, and 6 unlocked
//!  8. fill_duration_secs calculation for each core
//!  9. Persistence save/load cycle
//! 10. Default state

use quest::achievements::{AchievementId, Achievements};
use quest::power_cores::{
    fill_duration_secs, get_power_core_def, get_unlocked_cores, load_passives, GeneratorTimer,
    PassivesState, ALL_POWER_CORES,
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
fn test_core_keys_in_order() {
    let expected_keys = [
        "power_core_1",
        "power_core_2",
        "power_core_3",
        "power_core_4",
        "power_core_5",
        "power_core_6",
    ];
    for (i, (&expected, def)) in expected_keys.iter().zip(ALL_POWER_CORES.iter()).enumerate() {
        assert_eq!(
            def.key, expected,
            "Core index {i}: expected key '{expected}', got '{}'",
            def.key
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
fn test_pr_per_day_values_match_accelerating_curve() {
    let expected_rates: [u32; 6] = [2, 3, 5, 8, 12, 18];
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
fn test_red_fault_has_2_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreI).unwrap();
    assert_eq!(def.pr_per_day, 2);
}

#[test]
fn test_mirror_scar_has_3_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreII).unwrap();
    assert_eq!(def.pr_per_day, 3);
}

#[test]
fn test_black_mouth_has_5_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreIII).unwrap();
    assert_eq!(def.pr_per_day, 5);
}

#[test]
fn test_hollow_throne_has_8_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreIV).unwrap();
    assert_eq!(def.pr_per_day, 8);
}

#[test]
fn test_wailing_reach_has_12_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreV).unwrap();
    assert_eq!(def.pr_per_day, 12);
}

#[test]
fn test_origin_wound_has_18_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreVI).unwrap();
    assert_eq!(def.pr_per_day, 18);
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

// ── 6. PassivesState serialization round-trip ────────────────────────────────

#[test]
fn test_passives_state_serialization_round_trip_empty() {
    let state = PassivesState::default();
    let json = serde_json::to_string(&state).expect("serialization should succeed");
    let loaded: PassivesState =
        serde_json::from_str(&json).expect("deserialization should succeed");
    assert_eq!(loaded.generators.len(), state.generators.len());
    assert!(loaded.generators.is_empty());
}

#[test]
fn test_passives_state_serialization_round_trip_with_timestamps() {
    let mut state = PassivesState::default();
    state.generators.insert(
        "power_core_1".to_string(),
        GeneratorTimer {
            last_granted_at: 1_700_000_000,
        },
    );
    state.generators.insert(
        "power_core_2".to_string(),
        GeneratorTimer {
            last_granted_at: 1_700_100_000,
        },
    );
    state.generators.insert(
        "power_core_3".to_string(),
        GeneratorTimer {
            last_granted_at: 1_700_200_000,
        },
    );

    let json = serde_json::to_string_pretty(&state).expect("serialization should succeed");
    let loaded: PassivesState =
        serde_json::from_str(&json).expect("deserialization should succeed");

    assert_eq!(loaded.generators.len(), 3);
    assert_eq!(
        loaded.generators["power_core_1"].last_granted_at,
        1_700_000_000
    );
    assert_eq!(
        loaded.generators["power_core_2"].last_granted_at,
        1_700_100_000
    );
    assert_eq!(
        loaded.generators["power_core_3"].last_granted_at,
        1_700_200_000
    );
    // Unset cores should be absent
    assert!(!loaded.generators.contains_key("power_core_4"));
    assert!(!loaded.generators.contains_key("power_core_5"));
    assert!(!loaded.generators.contains_key("power_core_6"));
}

#[test]
fn test_passives_state_serialization_round_trip_all_six() {
    let mut state = PassivesState::default();
    let timestamps: [i64; 6] = [
        1_700_000_000,
        1_700_100_000,
        1_700_200_000,
        1_700_300_000,
        1_700_400_000,
        1_700_500_000,
    ];
    let keys = [
        "power_core_1",
        "power_core_2",
        "power_core_3",
        "power_core_4",
        "power_core_5",
        "power_core_6",
    ];
    for (&key, &ts) in keys.iter().zip(timestamps.iter()) {
        state.generators.insert(
            key.to_string(),
            GeneratorTimer {
                last_granted_at: ts,
            },
        );
    }

    let json = serde_json::to_string_pretty(&state).unwrap();
    let loaded: PassivesState = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.generators.len(), 6);
    for (&key, &ts) in keys.iter().zip(timestamps.iter()) {
        assert_eq!(
            loaded.generators[key].last_granted_at, ts,
            "Timestamp mismatch for '{key}'"
        );
    }
}

#[test]
fn test_passives_state_missing_keys_are_absent_after_round_trip() {
    let mut state = PassivesState::default();
    state.generators.insert(
        "power_core_6".to_string(),
        GeneratorTimer {
            last_granted_at: 999_999_999,
        },
    );

    let json = serde_json::to_string(&state).unwrap();
    let loaded: PassivesState = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.generators.len(), 1);
    assert_eq!(
        loaded.generators["power_core_6"].last_granted_at,
        999_999_999
    );
    assert!(!loaded.generators.contains_key("power_core_1"));
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
    unlock(&mut achievements, AchievementId::PowerCoreV);
    unlock(&mut achievements, AchievementId::PowerCoreVI);

    let cores = get_unlocked_cores(&achievements);
    assert_eq!(cores.len(), 2);
    let names: Vec<&str> = cores.iter().map(|c| c.name).collect();
    assert!(names.contains(&"Wailing Reach"));
    assert!(names.contains(&"Origin Wound"));
    assert!(!names.contains(&"Red Fault"));
    assert!(!names.contains(&"Mirror Scar"));
    assert!(!names.contains(&"Black Mouth"));
    assert!(!names.contains(&"Hollow Throne"));
}

// ── 8. fill_duration_secs ────────────────────────────────────────────────────

#[test]
fn test_fill_duration_secs_red_fault_is_43200() {
    assert_eq!(fill_duration_secs(2), 43200);
}

#[test]
fn test_fill_duration_secs_mirror_scar_is_28800() {
    assert_eq!(fill_duration_secs(3), 28800);
}

#[test]
fn test_fill_duration_secs_black_mouth_is_17280() {
    assert_eq!(fill_duration_secs(5), 17280);
}

#[test]
fn test_fill_duration_secs_hollow_throne_is_10800() {
    assert_eq!(fill_duration_secs(8), 10800);
}

#[test]
fn test_fill_duration_secs_wailing_reach_is_7200() {
    assert_eq!(fill_duration_secs(12), 7200);
}

#[test]
fn test_fill_duration_secs_origin_wound_is_4800() {
    assert_eq!(fill_duration_secs(18), 4800);
}

#[test]
fn test_fill_duration_secs_all_cores() {
    let expected: [(u32, i64); 6] = [
        (2, 43200),
        (3, 28800),
        (5, 17280),
        (8, 10800),
        (12, 7200),
        (18, 4800),
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
    let mut state = PassivesState::default();
    let entries = [
        ("power_core_1", 1_700_000_001_i64),
        ("power_core_2", 1_700_000_002),
        ("power_core_3", 1_700_000_003),
        ("power_core_4", 1_700_000_004),
        ("power_core_5", 1_700_000_005),
        ("power_core_6", 1_700_000_006),
    ];
    for (key, ts) in entries {
        state.generators.insert(
            key.to_string(),
            GeneratorTimer {
                last_granted_at: ts,
            },
        );
    }

    let json = serde_json::to_string_pretty(&state).expect("serialization should succeed");
    let loaded: PassivesState =
        serde_json::from_str(&json).expect("deserialization should succeed");

    assert_eq!(loaded.generators.len(), 6);
    for (key, ts) in entries {
        assert_eq!(
            loaded.generators[key].last_granted_at, ts,
            "Timestamp mismatch for '{key}' after save/load cycle"
        );
    }
}

#[test]
fn test_persistence_save_and_load_via_tempfile() {
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir creation should succeed");
    let path = tmp.path().join("passives.json");

    let mut state = PassivesState::default();
    state.generators.insert(
        "power_core_1".to_string(),
        GeneratorTimer {
            last_granted_at: 1_234_567_890,
        },
    );
    state.generators.insert(
        "power_core_3".to_string(),
        GeneratorTimer {
            last_granted_at: 9_876_543_210,
        },
    );

    let json = serde_json::to_string_pretty(&state).unwrap();
    std::fs::write(&path, &json).expect("write should succeed");

    let loaded_json = std::fs::read_to_string(&path).expect("read should succeed");
    let loaded: PassivesState =
        serde_json::from_str(&loaded_json).expect("deserialization should succeed");

    assert_eq!(loaded.generators.len(), 2);
    assert_eq!(
        loaded.generators["power_core_1"].last_granted_at,
        1_234_567_890
    );
    assert_eq!(
        loaded.generators["power_core_3"].last_granted_at,
        9_876_543_210
    );
    assert!(!loaded.generators.contains_key("power_core_2"));
}

#[test]
fn test_persistence_load_returns_default_for_corrupted_json() {
    let result: Result<PassivesState, _> = serde_json::from_str("not valid json");
    assert!(result.is_err(), "Invalid JSON should fail to deserialize");
    let default_state = PassivesState::default();
    assert!(default_state.generators.is_empty());
}

#[test]
fn test_load_passives_does_not_panic() {
    let _state = load_passives();
}

// ── 10. Default state ────────────────────────────────────────────────────────

#[test]
fn test_passives_state_default_has_empty_map() {
    let state = PassivesState::default();
    assert!(
        state.generators.is_empty(),
        "Default PassivesState should have no generators"
    );
}

#[test]
fn test_passives_state_new_entry_can_be_inserted() {
    let mut state = PassivesState::default();
    assert!(state.generators.is_empty());

    state.generators.insert(
        "power_core_1".to_string(),
        GeneratorTimer {
            last_granted_at: 0,
        },
    );
    assert_eq!(state.generators.len(), 1);
    assert_eq!(state.generators["power_core_1"].last_granted_at, 0);
}

#[test]
fn test_passives_state_zero_timestamp_is_valid() {
    let mut state = PassivesState::default();
    state.generators.insert(
        "power_core_1".to_string(),
        GeneratorTimer {
            last_granted_at: 0,
        },
    );

    let json = serde_json::to_string(&state).unwrap();
    let loaded: PassivesState = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.generators["power_core_1"].last_granted_at, 0);
}

#[test]
fn test_passives_state_clone_is_independent() {
    let mut state = PassivesState::default();
    state.generators.insert(
        "power_core_1".to_string(),
        GeneratorTimer {
            last_granted_at: 1000,
        },
    );

    let mut cloned = state.clone();
    cloned.generators.insert(
        "power_core_2".to_string(),
        GeneratorTimer {
            last_granted_at: 2000,
        },
    );

    assert!(!state.generators.contains_key("power_core_2"));
    assert_eq!(state.generators.len(), 1);
    assert_eq!(cloned.generators.len(), 2);
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
fn test_all_power_core_keys_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for def in ALL_POWER_CORES {
        assert!(
            seen.insert(def.key),
            "Duplicate key '{}' in ALL_POWER_CORES",
            def.key
        );
    }
}

#[test]
fn test_pr_per_day_strictly_increasing_across_cores() {
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
    let mut achievements = Achievements::default();
    unlock(&mut achievements, AchievementId::PowerCoreI);
    unlock(&mut achievements, AchievementId::PowerCoreI); // double-unlock

    let cores = get_unlocked_cores(&achievements);
    let red_fault_count = cores.iter().filter(|c| c.name == "Red Fault").count();
    assert_eq!(
        red_fault_count, 1,
        "Red Fault should appear exactly once despite duplicate unlock"
    );
}
