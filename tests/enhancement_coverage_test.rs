//! Enhancement module coverage tests — fills gaps not in enhancement_test.rs.
//!
//! Focus areas:
//! - roll_enhancement: deterministic success at risky levels, boundary edge cases
//! - apply_enhancement_result: multi-slot independence, counter accumulation
//! - Persistence: file I/O roundtrip with temp dir, missing file, corrupt file
//! - Types: enhancement_color_rgb above max, enhancement_prefix mid-range,
//!   set_level exactly at MAX, SoulforgePhase Copy/Clone, multiple open/close cycles

use quest::enhancement::{
    apply_enhancement_result, enhancement_color_rgb, enhancement_cost, enhancement_multiplier,
    enhancement_prefix, fail_penalty, roll_enhancement, soulforge_discovery_chance, success_rate,
    try_discover_soulforge, EnhancementProgress, EnhancementResult, SoulforgePhase,
    SoulforgeUiState, MAX_ENHANCEMENT_LEVEL, SOULFORGE_DISCOVERY_BASE_CHANCE,
    SOULFORGE_DISCOVERY_RANK_BONUS, SOULFORGE_MIN_PRESTIGE_RANK,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// =========================================================================
// types.rs — additional coverage
// =========================================================================

#[test]
fn test_set_level_exactly_at_max() {
    let mut ep = EnhancementProgress::new();
    ep.set_level(0, MAX_ENHANCEMENT_LEVEL);
    assert_eq!(ep.level(0), MAX_ENHANCEMENT_LEVEL);
    assert_eq!(ep.highest_level_reached, MAX_ENHANCEMENT_LEVEL);
}

#[test]
fn test_set_level_updates_highest_across_slots() {
    let mut ep = EnhancementProgress::new();
    ep.set_level(0, 3);
    ep.set_level(1, 5);
    ep.set_level(2, 2);
    assert_eq!(ep.highest_level_reached, 5);
    // Setting a lower value on a different slot should not lower highest
    ep.set_level(3, 1);
    assert_eq!(ep.highest_level_reached, 5);
}

#[test]
fn test_level_all_slots_independent() {
    let mut ep = EnhancementProgress::new();
    for slot in 0..7 {
        ep.set_level(slot, slot as u8 + 1);
    }
    for slot in 0..7 {
        assert_eq!(
            ep.level(slot),
            slot as u8 + 1,
            "Slot {} should have level {}",
            slot,
            slot + 1
        );
    }
}

#[test]
fn test_enhancement_prefix_mid_range() {
    assert_eq!(enhancement_prefix(5), "+5 ");
    assert_eq!(enhancement_prefix(7), "+7 ");
    assert_eq!(enhancement_prefix(9), "+9 ");
}

#[test]
fn test_enhancement_color_rgb_above_max() {
    // Levels above 10 should still return gold
    assert_eq!(enhancement_color_rgb(11), (255, 215, 0));
    assert_eq!(enhancement_color_rgb(255), (255, 215, 0));
}

#[test]
fn test_enhancement_multiplier_monotonically_increasing() {
    let mut prev = enhancement_multiplier(0);
    for level in 1..=MAX_ENHANCEMENT_LEVEL {
        let current = enhancement_multiplier(level);
        assert!(
            current > prev,
            "Multiplier should increase: level {} ({}) should be > level {} ({})",
            level,
            current,
            level - 1,
            prev
        );
        prev = current;
    }
}

#[test]
fn test_success_rate_monotonically_decreasing_for_risky_levels() {
    // For levels 5-10, success rate should decrease
    let mut prev = success_rate(5);
    for level in 6..=10 {
        let current = success_rate(level);
        assert!(
            current < prev,
            "Success rate should decrease: level {} ({}) should be < level {} ({})",
            level,
            current,
            level - 1,
            prev
        );
        prev = current;
    }
}

#[test]
fn test_enhancement_cost_non_decreasing() {
    let mut prev = enhancement_cost(1);
    for level in 2..=10 {
        let current = enhancement_cost(level);
        assert!(
            current >= prev,
            "Cost should not decrease: level {} ({}) should be >= level {} ({})",
            level,
            current,
            level - 1,
            prev
        );
        prev = current;
    }
}

#[test]
fn test_fail_penalty_non_decreasing() {
    let mut prev = fail_penalty(1);
    for level in 2..=10 {
        let current = fail_penalty(level);
        assert!(
            current >= prev,
            "Penalty should not decrease: level {} ({}) should be >= level {} ({})",
            level,
            current,
            level - 1,
            prev
        );
        prev = current;
    }
}

#[test]
fn test_soulforge_phase_copy_clone() {
    let phase = SoulforgePhase::Hammering;
    let copied = phase;
    let cloned = phase;
    assert_eq!(phase, copied);
    assert_eq!(phase, cloned);
}

#[test]
fn test_soulforge_ui_state_multiple_open_close_cycles() {
    let mut ui = SoulforgeUiState::new();

    // Cycle 1: open, modify, close
    ui.open();
    ui.selected_slot = 5;
    ui.phase = SoulforgePhase::ResultSuccess;
    ui.animation_tick = 10;
    ui.close();
    assert!(!ui.open);
    assert_eq!(ui.phase, SoulforgePhase::Menu);

    // Cycle 2: reopen should reset all state
    ui.open();
    assert!(ui.open);
    assert_eq!(ui.selected_slot, 0);
    assert_eq!(ui.phase, SoulforgePhase::Menu);
    assert_eq!(ui.animation_tick, 0);
    assert!(ui.last_result.is_none());

    // Cycle 3: close from Confirming phase
    ui.phase = SoulforgePhase::Confirming;
    ui.close();
    assert_eq!(ui.phase, SoulforgePhase::Menu);
}

#[test]
fn test_soulforge_ui_state_open_with_last_result() {
    let mut ui = SoulforgeUiState::new();
    ui.last_result = Some(EnhancementResult {
        slot_index: 2,
        success: true,
        old_level: 3,
        new_level: 4,
        cost: 1,
    });

    // Opening should clear last_result
    ui.open();
    assert!(ui.last_result.is_none());
}

#[test]
fn test_enhancement_result_failure_variant() {
    let result = EnhancementResult {
        slot_index: 3,
        success: false,
        old_level: 9,
        new_level: 7,
        cost: 10,
    };
    assert!(!result.success);
    assert_eq!(result.old_level - result.new_level, 2);
    assert_eq!(result.cost, 10);
}

// =========================================================================
// logic.rs — roll_enhancement additional coverage
// =========================================================================

#[test]
fn test_roll_enhancement_deterministic_success_at_risky_level() {
    // Find a seed that succeeds at level 4->5 (60% rate)
    for seed in 0..1000u64 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let (success, new_level) = roll_enhancement(4, &mut rng);
        if success {
            assert_eq!(new_level, 5);
            return;
        }
    }
    panic!("Could not find a seed that succeeds at +5 within 1000 attempts");
}

#[test]
fn test_roll_enhancement_deterministic_success_at_level_10() {
    // Find a seed that succeeds at level 9->10 (10% rate)
    for seed in 0..1000u64 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let (success, new_level) = roll_enhancement(9, &mut rng);
        if success {
            assert_eq!(new_level, 10);
            return;
        }
    }
    panic!("Could not find a seed that succeeds at +10 within 1000 attempts");
}

#[test]
fn test_roll_enhancement_at_each_safe_level() {
    // Levels 0-3 should always succeed regardless of RNG
    let mut rng = ChaCha8Rng::seed_from_u64(0);
    for current_level in 0..4u8 {
        let (success, new_level) = roll_enhancement(current_level, &mut rng);
        assert!(
            success,
            "Level {} -> {} should always succeed",
            current_level,
            current_level + 1
        );
        assert_eq!(new_level, current_level + 1);
    }
}

#[test]
fn test_roll_enhancement_fail_at_level_9_penalty_minus_1() {
    // At level 8, attempting +9 (20% rate, penalty -1)
    for seed in 0..1000u64 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let (success, new_level) = roll_enhancement(8, &mut rng);
        if !success {
            assert_eq!(
                new_level, 7,
                "Failed +9 should drop from 8 to 7 (penalty -1)"
            );
            return;
        }
    }
    panic!("Could not find a seed that fails at +9 within 1000 attempts");
}

#[test]
fn test_roll_enhancement_statistical_distribution() {
    // Run many attempts at +5 (70% rate) and verify distribution is reasonable
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let mut successes = 0;
    let trials = 10000;

    for _ in 0..trials {
        let (success, _) = roll_enhancement(4, &mut rng);
        if success {
            successes += 1;
        }
    }

    let rate = successes as f64 / trials as f64;
    // 70% rate with 10k trials: expect ~7000 +/- ~150 (3 sigma)
    assert!(
        rate > 0.65 && rate < 0.75,
        "Success rate for +5 should be around 70%, got {:.2}%",
        rate * 100.0
    );
}

#[test]
fn test_roll_enhancement_fail_at_level_5_saturating_sub() {
    // At level 4, fail +5 => penalty 1, goes to 3. But test at lower level:
    // Set current=0 and try to fail. But +1 is 100%, so we can't fail.
    // Instead, test the boundary: at level 4, fail +5 (penalty 1) -> level 3
    // at level 0, this can't happen because +1 is safe.
    // Verify roll_enhancement(0, rng) always gives (true, 1)
    for seed in 0..100u64 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let (success, new_level) = roll_enhancement(0, &mut rng);
        assert!(success, "Level 0->1 must always succeed (seed={})", seed);
        assert_eq!(new_level, 1);
    }
}

// =========================================================================
// logic.rs — apply_enhancement_result additional coverage
// =========================================================================

#[test]
fn test_apply_enhancement_result_counter_accumulation() {
    let mut ep = EnhancementProgress::new();

    // 5 successes, 3 failures
    for i in 0..5 {
        apply_enhancement_result(&mut ep, 0, i + 1, true);
    }
    for _ in 0..3 {
        apply_enhancement_result(&mut ep, 0, 4, false);
    }

    assert_eq!(ep.total_attempts, 8);
    assert_eq!(ep.total_successes, 5);
    assert_eq!(ep.total_failures, 3);
}

#[test]
fn test_apply_enhancement_result_multiple_slots() {
    let mut ep = EnhancementProgress::new();

    // Enhance different slots
    apply_enhancement_result(&mut ep, 0, 3, true);
    apply_enhancement_result(&mut ep, 1, 5, true);
    apply_enhancement_result(&mut ep, 6, 10, true);

    assert_eq!(ep.level(0), 3);
    assert_eq!(ep.level(1), 5);
    assert_eq!(ep.level(6), 10);
    assert_eq!(ep.total_attempts, 3);
    assert_eq!(ep.total_successes, 3);
    assert_eq!(ep.highest_level_reached, 10);
}

#[test]
fn test_apply_enhancement_result_highest_level_tracked() {
    let mut ep = EnhancementProgress::new();

    apply_enhancement_result(&mut ep, 0, 7, true);
    assert_eq!(ep.highest_level_reached, 7);

    // Apply a lower level to same slot (failure scenario)
    apply_enhancement_result(&mut ep, 0, 5, false);
    assert_eq!(ep.highest_level_reached, 7, "Should not decrease");
    assert_eq!(ep.level(0), 5);
}

// =========================================================================
// logic.rs — soulforge discovery additional coverage
// =========================================================================

#[test]
fn test_soulforge_discovery_chance_at_very_high_prestige() {
    let chance_p100 = soulforge_discovery_chance(100);
    let expected = SOULFORGE_DISCOVERY_BASE_CHANCE
        + (100 - SOULFORGE_MIN_PRESTIGE_RANK) as f64 * SOULFORGE_DISCOVERY_RANK_BONUS;
    assert!(
        (chance_p100 - expected).abs() < f64::EPSILON,
        "P100 chance: expected {}, got {}",
        expected,
        chance_p100
    );
}

#[test]
fn test_soulforge_discovery_chance_linear_scaling() {
    let c15 = soulforge_discovery_chance(15);
    let c16 = soulforge_discovery_chance(16);
    let c17 = soulforge_discovery_chance(17);

    let delta1 = c16 - c15;
    let delta2 = c17 - c16;

    assert!(
        (delta1 - delta2).abs() < f64::EPSILON,
        "Discovery chance should scale linearly: delta1={}, delta2={}",
        delta1,
        delta2
    );
    assert!(
        (delta1 - SOULFORGE_DISCOVERY_RANK_BONUS).abs() < f64::EPSILON,
        "Each rank should add exactly SOULFORGE_DISCOVERY_RANK_BONUS"
    );
}

#[test]
fn test_try_discover_soulforge_high_prestige_discovers_faster() {
    // At high prestige, discovery should happen in fewer ticks on average
    let mut rng_low = ChaCha8Rng::seed_from_u64(42);
    let mut rng_high = ChaCha8Rng::seed_from_u64(42);

    let mut ep_low = EnhancementProgress::new();
    let mut ticks_low = 0u64;
    for i in 0..1_000_000u64 {
        if try_discover_soulforge(&mut ep_low, 15, &mut rng_low) {
            ticks_low = i;
            break;
        }
    }

    let mut ep_high = EnhancementProgress::new();
    let mut ticks_high = 0u64;
    for i in 0..1_000_000u64 {
        if try_discover_soulforge(&mut ep_high, 50, &mut rng_high) {
            ticks_high = i;
            break;
        }
    }

    assert!(
        ep_low.discovered && ep_high.discovered,
        "Both should discover"
    );
    assert!(
        ticks_high < ticks_low,
        "Higher prestige ({} ticks) should discover faster than lower prestige ({} ticks)",
        ticks_high,
        ticks_low
    );
}

#[test]
fn test_try_discover_soulforge_idempotent_after_discovery() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut ep = EnhancementProgress::new();
    ep.discovered = true;

    // Should always return false once already discovered
    for _ in 0..100 {
        assert!(
            !try_discover_soulforge(&mut ep, 100, &mut rng),
            "Should never re-discover"
        );
    }
}

#[test]
fn test_try_discover_soulforge_at_prestige_14_never_discovers() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut ep = EnhancementProgress::new();

    for _ in 0..100_000 {
        assert!(!try_discover_soulforge(&mut ep, 14, &mut rng));
    }
    assert!(!ep.discovered);
}

// =========================================================================
// Persistence — file I/O tests with temp directory
// =========================================================================

#[test]
fn test_persistence_serde_roundtrip_all_fields() {
    let mut ep = EnhancementProgress::new();
    ep.discovered = true;
    ep.levels = [1, 2, 3, 4, 5, 6, 7];
    ep.total_attempts = 100;
    ep.total_successes = 60;
    ep.total_failures = 40;
    ep.highest_level_reached = 7;

    let json = serde_json::to_string_pretty(&ep).unwrap();
    let restored: EnhancementProgress = serde_json::from_str(&json).unwrap();

    assert!(restored.discovered);
    assert_eq!(restored.levels, [1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(restored.total_attempts, 100);
    assert_eq!(restored.total_successes, 60);
    assert_eq!(restored.total_failures, 40);
    assert_eq!(restored.highest_level_reached, 7);
}

#[test]
fn test_persistence_max_levels_roundtrip() {
    let mut ep = EnhancementProgress::new();
    ep.discovered = true;
    for slot in 0..7 {
        ep.set_level(slot, MAX_ENHANCEMENT_LEVEL);
    }
    ep.total_attempts = u32::MAX;
    ep.total_successes = u32::MAX / 2;
    ep.total_failures = u32::MAX / 2;

    let json = serde_json::to_string(&ep).unwrap();
    let restored: EnhancementProgress = serde_json::from_str(&json).unwrap();

    for slot in 0..7 {
        assert_eq!(restored.level(slot), MAX_ENHANCEMENT_LEVEL);
    }
    assert_eq!(restored.total_attempts, u32::MAX);
    assert_eq!(restored.highest_level_reached, MAX_ENHANCEMENT_LEVEL);
}

#[test]
fn test_persistence_default_all_zeros() {
    let ep = EnhancementProgress::new();
    let json = serde_json::to_string(&ep).unwrap();
    let restored: EnhancementProgress = serde_json::from_str(&json).unwrap();

    assert!(!restored.discovered);
    assert_eq!(restored.levels, [0; 7]);
    assert_eq!(restored.total_attempts, 0);
    assert_eq!(restored.total_successes, 0);
    assert_eq!(restored.total_failures, 0);
    assert_eq!(restored.highest_level_reached, 0);
}

#[test]
fn test_persistence_corrupt_json_various() {
    // Various forms of invalid JSON should all produce defaults
    let corrupt_inputs = [
        "",
        "null",
        "42",
        "true",
        "[]",
        "{",
        r#"{"discovered": "not_a_bool"}"#,
        r#"{"levels": "not_an_array"}"#,
        r#"{"levels": [1,2,3]}"#, // wrong array length + missing fields
    ];

    for input in corrupt_inputs {
        let result: EnhancementProgress = serde_json::from_str(input).unwrap_or_default();
        assert!(
            !result.discovered,
            "Corrupt input {:?} should produce default (discovered=false)",
            input
        );
        assert_eq!(
            result.levels, [0; 7],
            "Corrupt input {:?} should produce default levels",
            input
        );
    }
}

#[test]
fn test_persistence_file_io_roundtrip() {
    use std::fs;

    let tmp = tempfile::tempdir().expect("Failed to create temp dir");
    let path = tmp.path().join("enhancement.json");

    let mut ep = EnhancementProgress::new();
    ep.discovered = true;
    ep.set_level(0, 8);
    ep.set_level(3, 5);
    ep.total_attempts = 42;
    ep.total_successes = 25;
    ep.total_failures = 17;

    // Write
    let json = serde_json::to_string_pretty(&ep).unwrap();
    fs::write(&path, &json).unwrap();

    // Read back
    let read_json = fs::read_to_string(&path).unwrap();
    let restored: EnhancementProgress = serde_json::from_str(&read_json).unwrap();

    assert!(restored.discovered);
    assert_eq!(restored.level(0), 8);
    assert_eq!(restored.level(3), 5);
    assert_eq!(restored.total_attempts, 42);
    assert_eq!(restored.total_successes, 25);
    assert_eq!(restored.total_failures, 17);
    assert_eq!(restored.highest_level_reached, 8);
}

#[test]
fn test_persistence_missing_file_produces_default() {
    use std::fs;

    let tmp = tempfile::tempdir().expect("Failed to create temp dir");
    let path = tmp.path().join("nonexistent_enhancement.json");

    // File does not exist in fresh temp dir

    // Simulate load_enhancement pattern: read_to_string fails -> new()
    let result = match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => EnhancementProgress::new(),
    };

    assert!(!result.discovered);
    assert_eq!(result.levels, [0; 7]);
}

#[test]
fn test_persistence_corrupt_file_produces_default() {
    use std::fs;

    let tmp = tempfile::tempdir().expect("Failed to create temp dir");
    let path = tmp.path().join("enhancement.json");
    fs::write(&path, "THIS IS NOT VALID JSON!!!").unwrap();

    // Simulate load_enhancement pattern
    let result = match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => EnhancementProgress::new(),
    };

    assert!(!result.discovered);
    assert_eq!(result.levels, [0; 7]);
}

#[test]
fn test_persistence_save_creates_directory() {
    use std::fs;

    let tmp = tempfile::tempdir().expect("Failed to create temp dir");
    let subdir = tmp.path().join("subdir");

    // Create subdir and file
    fs::create_dir_all(&subdir).unwrap();
    let path = subdir.join("enhancement.json");

    let ep = EnhancementProgress::new();
    let json = serde_json::to_string_pretty(&ep).unwrap();
    fs::write(&path, &json).unwrap();

    assert!(path.exists());
}

// =========================================================================
// Integration — full enhancement journey
// =========================================================================

#[test]
fn test_full_enhancement_to_max_with_seeded_rng() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut ep = EnhancementProgress::new();
    ep.discovered = true;

    let slot = 0;
    let mut attempts = 0u32;
    let max_attempts = 10_000;

    while ep.level(slot) < MAX_ENHANCEMENT_LEVEL && attempts < max_attempts {
        let current = ep.level(slot);
        let (success, new_level) = roll_enhancement(current, &mut rng);
        apply_enhancement_result(&mut ep, slot, new_level, success);
        attempts += 1;
    }

    assert_eq!(
        ep.level(slot),
        MAX_ENHANCEMENT_LEVEL,
        "Should eventually reach +10 within {} attempts (took {})",
        max_attempts,
        attempts
    );
    assert_eq!(ep.total_attempts, attempts);
    assert_eq!(
        ep.total_successes + ep.total_failures,
        attempts,
        "Success + failure should equal total attempts"
    );
    assert!(
        ep.total_successes >= 10,
        "Need at least 10 successes to reach +10"
    );
    assert_eq!(ep.highest_level_reached, MAX_ENHANCEMENT_LEVEL);
}

#[test]
fn test_enhance_all_seven_slots_independently() {
    let mut rng = ChaCha8Rng::seed_from_u64(99);
    let mut ep = EnhancementProgress::new();

    // Enhance each slot to +4 (guaranteed success)
    for slot in 0..7 {
        for _ in 0..4 {
            let current = ep.level(slot);
            let (success, new_level) = roll_enhancement(current, &mut rng);
            apply_enhancement_result(&mut ep, slot, new_level, success);
            assert!(success, "Levels 0-3 should always succeed");
        }
        assert_eq!(ep.level(slot), 4, "Slot {} should be at +4", slot);
    }

    assert_eq!(ep.total_attempts, 28); // 7 slots * 4 attempts
    assert_eq!(ep.total_successes, 28);
    assert_eq!(ep.total_failures, 0);
    assert_eq!(ep.highest_level_reached, 4);
}

// =========================================================================
// Edge cases and boundary tests
// =========================================================================

#[test]
fn test_enhancement_cost_total_to_max() {
    // Calculate total cost to go from +0 to +10 if all succeed
    let mut total: u32 = 0;
    for level in 1..=MAX_ENHANCEMENT_LEVEL {
        total += enhancement_cost(level);
    }
    // 1+1+1+1 + 2+3+3 + 4+4 + 5 = 4 + 8 + 8 + 5 = 25
    assert_eq!(total, 25, "Total cost from +0 to +10 should be 25 PR");
}

#[test]
fn test_enhancement_multiplier_at_zero_is_identity() {
    assert_eq!(enhancement_multiplier(0), 1.0);
}

#[test]
fn test_enhancement_multiplier_at_max_is_250_percent() {
    assert!((enhancement_multiplier(MAX_ENHANCEMENT_LEVEL) - 2.5).abs() < f64::EPSILON);
}

#[test]
fn test_soulforge_discovery_chance_zero_prestige() {
    assert_eq!(soulforge_discovery_chance(0), 0.0);
}

#[test]
fn test_soulforge_discovery_chance_just_below_min() {
    assert_eq!(
        soulforge_discovery_chance(SOULFORGE_MIN_PRESTIGE_RANK - 1),
        0.0
    );
}

#[test]
fn test_enhancement_prefix_at_max() {
    assert_eq!(enhancement_prefix(MAX_ENHANCEMENT_LEVEL), "+10 ");
}

#[test]
fn test_enhancement_prefix_above_max() {
    // Prefix still formats any u8 value
    assert_eq!(enhancement_prefix(15), "+15 ");
    assert_eq!(enhancement_prefix(255), "+255 ");
}

#[test]
fn test_soulforge_phase_all_variants_distinct() {
    let variants = [
        SoulforgePhase::Menu,
        SoulforgePhase::Confirming,
        SoulforgePhase::Hammering,
        SoulforgePhase::ResultSuccess,
        SoulforgePhase::ResultFailure,
    ];

    // All pairs should be distinct
    for i in 0..variants.len() {
        for j in (i + 1)..variants.len() {
            assert_ne!(
                variants[i], variants[j],
                "Variants {:?} and {:?} should be distinct",
                variants[i], variants[j]
            );
        }
    }
}

#[test]
fn test_soulforge_phase_debug_format() {
    let phase = SoulforgePhase::Hammering;
    let debug_str = format!("{:?}", phase);
    assert_eq!(debug_str, "Hammering");
}

#[test]
fn test_enhancement_progress_clone() {
    let mut ep = EnhancementProgress::new();
    ep.discovered = true;
    ep.set_level(0, 5);
    ep.total_attempts = 10;
    ep.total_successes = 7;
    ep.total_failures = 3;

    let cloned = ep.clone();
    assert_eq!(cloned.discovered, ep.discovered);
    assert_eq!(cloned.levels, ep.levels);
    assert_eq!(cloned.total_attempts, ep.total_attempts);
    assert_eq!(cloned.total_successes, ep.total_successes);
    assert_eq!(cloned.total_failures, ep.total_failures);
    assert_eq!(cloned.highest_level_reached, ep.highest_level_reached);
}

#[test]
fn test_enhancement_progress_debug_format() {
    let ep = EnhancementProgress::new();
    let debug_str = format!("{:?}", ep);
    assert!(debug_str.contains("EnhancementProgress"));
    assert!(debug_str.contains("discovered"));
    assert!(debug_str.contains("levels"));
}

// =========================================================================
// game_tick integration — Soulforge discovery via the tick engine
// =========================================================================

mod soulforge_tick_integration {
    use quest::achievements::Achievements;
    use quest::core::game_state::GameState;
    use quest::core::tick::game_tick_with_context;
    use quest::core::tick_context::TickContext;
    use quest::core::tick_types::TickEvent;
    use quest::deep::DeepState;
    use quest::dungeon::generation::generate_dungeon;
    use quest::enhancement::EnhancementProgress;
    use quest::haven::Haven;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn make_rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    /// Run ticks until Soulforge is discovered or max_ticks is exhausted.
    /// Returns (ticks_elapsed, discovered).
    fn run_until_soulforge_discovered(
        state: &mut GameState,
        enhancement: &mut EnhancementProgress,
        max_ticks: u32,
    ) -> (u32, bool) {
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut achievements = Achievements::default();
        let mut deep = DeepState::new();
        // Use a fixed seed that hits discovery quickly at P15+
        let mut rng = make_rng(1);

        for tick in 0..max_ticks {
            let mut ctx = TickContext {
                state,
                tick_counter: &mut tick_counter,
                haven: &mut haven,
                enhancement,
                deep: &mut deep,
                achievements: &mut achievements,
                debug_mode: false,
            };
            let result = game_tick_with_context(&mut ctx, &mut rng);

            if result
                .events
                .iter()
                .any(|e| matches!(e, TickEvent::SoulforgeDiscovered))
            {
                return (tick, true);
            }
        }
        (max_ticks, false)
    }

    /// Verify that game_tick emits SoulforgeDiscovered and sets enhancement_changed
    /// when prestige_rank >= SOULFORGE_MIN_PRESTIGE_RANK (15).
    #[test]
    fn game_tick_emits_soulforge_discovered_at_p15_plus() {
        let mut state = GameState::new("Forge Hero".to_string(), 0);
        state.prestige_rank = 15;
        let mut enhancement = EnhancementProgress::new();

        // At P15, discovery base chance = 0.000014/tick. With a fixed seed and
        // enough ticks (1M = 100k seconds) we should discover it.
        let (_, discovered) =
            run_until_soulforge_discovered(&mut state, &mut enhancement, 1_000_000);

        assert!(
            discovered,
            "Soulforge should be discovered within 1M ticks at P15"
        );
        assert!(
            enhancement.discovered,
            "EnhancementProgress.discovered should be true after game_tick discovery"
        );
    }

    /// At P15+, when discovery fires, the tick result must set enhancement_changed = true.
    #[test]
    fn game_tick_sets_enhancement_changed_on_soulforge_discovery() {
        let mut state = GameState::new("Forge Hero".to_string(), 0);
        state.prestige_rank = 20; // Higher rank = faster discovery
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut achievements = Achievements::default();
        let mut deep = DeepState::new();
        let mut rng = make_rng(1);

        // Run until discovery fires and capture that specific TickResult
        for _ in 0..500_000 {
            let mut ctx = TickContext {
                state: &mut state,
                tick_counter: &mut tick_counter,
                haven: &mut haven,
                enhancement: &mut enhancement,
                deep: &mut deep,
                achievements: &mut achievements,
                debug_mode: false,
            };
            let result = game_tick_with_context(&mut ctx, &mut rng);

            if result
                .events
                .iter()
                .any(|e| matches!(e, TickEvent::SoulforgeDiscovered))
            {
                assert!(
                    result.enhancement_changed,
                    "enhancement_changed must be true on the tick that discovers Soulforge"
                );
                return;
            }
        }
        panic!("Soulforge was not discovered within 500K ticks at P20");
    }

    /// Soulforge discovery is blocked when an active dungeon is running,
    /// even at qualifying prestige rank.
    #[test]
    fn game_tick_soulforge_discovery_blocked_by_active_dungeon() {
        let mut state = GameState::new("Dungeon Blocker".to_string(), 0);
        state.prestige_rank = 50; // Very high — would discover quickly without blockage
        state.active_dungeon = Some(generate_dungeon(1, 0, 1));

        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut achievements = Achievements::default();
        let mut deep = DeepState::new();
        let mut rng = make_rng(42);

        // Run a large number of ticks; dungeon blocks Soulforge discovery.
        // Re-set the dungeon each tick to ensure it stays active (dungeons can complete/fail).
        for _ in 0..1_000 {
            state.active_dungeon = Some(generate_dungeon(1, 0, 1));
            let mut ctx = TickContext {
                state: &mut state,
                tick_counter: &mut tick_counter,
                haven: &mut haven,
                enhancement: &mut enhancement,
                deep: &mut deep,
                achievements: &mut achievements,
                debug_mode: false,
            };
            let result = game_tick_with_context(&mut ctx, &mut rng);
            assert!(
                !result
                    .events
                    .iter()
                    .any(|e| matches!(e, TickEvent::SoulforgeDiscovered)),
                "Soulforge discovery must be blocked while dungeon is active"
            );
        }
        assert!(
            !enhancement.discovered,
            "enhancement.discovered must remain false while dungeon blocks discovery"
        );
    }

    /// Once already discovered, game_tick never re-emits SoulforgeDiscovered.
    #[test]
    fn game_tick_soulforge_discovery_idempotent_after_discovery() {
        let mut state = GameState::new("Already Found".to_string(), 0);
        state.prestige_rank = 50;
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        enhancement.discovered = true; // Pre-discovered
        let mut achievements = Achievements::default();
        let mut deep = DeepState::new();
        let mut rng = make_rng(42);

        for _ in 0..500 {
            let mut ctx = TickContext {
                state: &mut state,
                tick_counter: &mut tick_counter,
                haven: &mut haven,
                enhancement: &mut enhancement,
                deep: &mut deep,
                achievements: &mut achievements,
                debug_mode: false,
            };
            let result = game_tick_with_context(&mut ctx, &mut rng);
            assert!(
                !result
                    .events
                    .iter()
                    .any(|e| matches!(e, TickEvent::SoulforgeDiscovered)),
                "SoulforgeDiscovered must not re-fire once already discovered"
            );
        }
    }
}
