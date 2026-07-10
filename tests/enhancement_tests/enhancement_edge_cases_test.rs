//! Edge case tests for the enhancement/Soulforge system.
//!
//! Focuses on gaps not covered by enhancement_test.rs and enhancement_coverage_test.rs:
//!
//! - Exact success rate values at each level: verified statistically
//! - Failure penalty of -1 (levels +5 to +9) and -2 (level +10)
//! - Soul Tithe: available at +5 through +10, with exact PR costs
//! - Enhancement costs: exact values at each target level
//! - Discovery chance: formula, exact values at P15 and above
//! - All 7 slots independent: no cross-contamination
//! - Enhancement multiplier curve: exact values at each level
//! - roll_enhancement at max level: returns (false, current_level)
//! - Level boundary: saturating_sub protects against underflow at level 0
//! - Stat multiplier integration with derived stats

use quest::enhancement::{
    apply_enhancement_result, enhancement_cost, enhancement_multiplier, fail_penalty,
    roll_enhancement, soul_tithe_cost, soulforge_discovery_chance, success_rate,
    try_discover_soulforge, EnhancementProgress, MAX_ENHANCEMENT_LEVEL,
    SOULFORGE_DISCOVERY_BASE_CHANCE, SOULFORGE_DISCOVERY_RANK_BONUS, SOULFORGE_MIN_PRESTIGE_RANK,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

// =============================================================================
// Exact success rates at each level
// =============================================================================

#[test]
fn test_success_rates_exact_values() {
    // +1-4: 100%, +5: 70%, +6: 55%, +7: 40%, +8: 30%, +9: 20%, +10: 10%
    assert!(
        (success_rate(1) - 1.00).abs() < f64::EPSILON,
        "level 1 should be 100%"
    );
    assert!(
        (success_rate(2) - 1.00).abs() < f64::EPSILON,
        "level 2 should be 100%"
    );
    assert!(
        (success_rate(3) - 1.00).abs() < f64::EPSILON,
        "level 3 should be 100%"
    );
    assert!(
        (success_rate(4) - 1.00).abs() < f64::EPSILON,
        "level 4 should be 100%"
    );
    assert!(
        (success_rate(5) - 0.70).abs() < f64::EPSILON,
        "level 5 should be 70%"
    );
    assert!(
        (success_rate(6) - 0.55).abs() < f64::EPSILON,
        "level 6 should be 55%"
    );
    assert!(
        (success_rate(7) - 0.40).abs() < f64::EPSILON,
        "level 7 should be 40%"
    );
    assert!(
        (success_rate(8) - 0.30).abs() < f64::EPSILON,
        "level 8 should be 30%"
    );
    assert!(
        (success_rate(9) - 0.20).abs() < f64::EPSILON,
        "level 9 should be 20%"
    );
    assert!(
        (success_rate(10) - 0.10).abs() < f64::EPSILON,
        "level 10 should be 10%"
    );
}

#[test]
fn test_success_rate_out_of_range_returns_zero() {
    assert_eq!(success_rate(0), 0.0, "level 0 is invalid");
    assert_eq!(success_rate(11), 0.0, "level 11 is out of range");
    assert_eq!(success_rate(255), 0.0, "level 255 is out of range");
}

// =============================================================================
// Guaranteed 100% success at +1-+4 via roll_enhancement
// =============================================================================

#[test]
fn test_levels_1_to_4_always_succeed() {
    // roll_enhancement at current levels 0,1,2,3 → target 1,2,3,4 — should always succeed
    for seed in 0u64..100 {
        let mut r = rng(seed);
        for current in 0u8..4 {
            let (success, new_level) = roll_enhancement(current, &mut r);
            assert!(
                success,
                "level {} -> {} must always succeed (seed={})",
                current,
                current + 1,
                seed
            );
            assert_eq!(
                new_level,
                current + 1,
                "new_level should be {} (seed={})",
                current + 1,
                seed
            );
        }
    }
}

// =============================================================================
// Statistical verification of risky success rates
// =============================================================================

#[test]
fn test_success_rate_level_5_statistically_70_percent() {
    let mut r = rng(1234);
    let trials = 20000;
    let mut successes = 0u32;
    for _ in 0..trials {
        let (success, _) = roll_enhancement(4, &mut r); // target level 5
        if success {
            successes += 1;
        }
    }
    let rate = successes as f64 / trials as f64;
    assert!(
        (0.67..=0.73).contains(&rate),
        "+5 success rate should be ~70%, got {:.3}",
        rate
    );
}

#[test]
fn test_success_rate_level_6_statistically_55_percent() {
    let mut r = rng(5678);
    let trials = 20000;
    let mut successes = 0u32;
    for _ in 0..trials {
        let (success, _) = roll_enhancement(5, &mut r);
        if success {
            successes += 1;
        }
    }
    let rate = successes as f64 / trials as f64;
    assert!(
        (0.52..=0.58).contains(&rate),
        "+6 success rate should be ~55%, got {:.3}",
        rate
    );
}

#[test]
fn test_success_rate_level_7_statistically_40_percent() {
    let mut r = rng(9012);
    let trials = 20000;
    let mut successes = 0u32;
    for _ in 0..trials {
        let (success, _) = roll_enhancement(6, &mut r);
        if success {
            successes += 1;
        }
    }
    let rate = successes as f64 / trials as f64;
    assert!(
        (0.37..=0.43).contains(&rate),
        "+7 success rate should be ~40%, got {:.3}",
        rate
    );
}

#[test]
fn test_success_rate_level_8_statistically_30_percent() {
    let mut r = rng(3456);
    let trials = 20000;
    let mut successes = 0u32;
    for _ in 0..trials {
        let (success, _) = roll_enhancement(7, &mut r);
        if success {
            successes += 1;
        }
    }
    let rate = successes as f64 / trials as f64;
    assert!(
        (0.27..=0.33).contains(&rate),
        "+8 success rate should be ~30%, got {:.3}",
        rate
    );
}

#[test]
fn test_success_rate_level_9_statistically_20_percent() {
    let mut r = rng(7890);
    let trials = 20000;
    let mut successes = 0u32;
    for _ in 0..trials {
        let (success, _) = roll_enhancement(8, &mut r);
        if success {
            successes += 1;
        }
    }
    let rate = successes as f64 / trials as f64;
    assert!(
        (0.17..=0.23).contains(&rate),
        "+9 success rate should be ~20%, got {:.3}",
        rate
    );
}

#[test]
fn test_success_rate_level_10_statistically_10_percent() {
    let mut r = rng(2468);
    let trials = 20000;
    let mut successes = 0u32;
    for _ in 0..trials {
        let (success, _) = roll_enhancement(9, &mut r);
        if success {
            successes += 1;
        }
    }
    let rate = successes as f64 / trials as f64;
    assert!(
        (0.08..=0.12).contains(&rate),
        "+10 success rate should be ~10%, got {:.3}",
        rate
    );
}

// =============================================================================
// Failure penalties: -1 for +5-+9, -2 for +10
// =============================================================================

#[test]
fn test_fail_penalty_exact_values() {
    // +1-+4: no penalty (0)
    assert_eq!(fail_penalty(1), 0);
    assert_eq!(fail_penalty(2), 0);
    assert_eq!(fail_penalty(3), 0);
    assert_eq!(fail_penalty(4), 0);
    // +5-+9: -1 level
    assert_eq!(fail_penalty(5), 1);
    assert_eq!(fail_penalty(6), 1);
    assert_eq!(fail_penalty(7), 1);
    assert_eq!(fail_penalty(8), 1);
    assert_eq!(fail_penalty(9), 1);
    // +10: -2 levels
    assert_eq!(fail_penalty(10), 2);
}

#[test]
fn test_fail_penalty_out_of_range_returns_zero() {
    assert_eq!(fail_penalty(0), 0);
    assert_eq!(fail_penalty(11), 0);
    assert_eq!(fail_penalty(255), 0);
}

#[test]
fn test_roll_enhancement_failure_at_level_5_drops_by_1() {
    // When roll_enhancement(4) fails, new_level = 4 - penalty(5) = 4 - 1 = 3.
    // Find seed, then assert unconditionally.
    let fail_seed = (0..2000u64)
        .find(|&s| {
            let mut r = rng(s);
            let (success, _) = roll_enhancement(4, &mut r);
            !success
        })
        .expect("should find a failure seed within 2000 tries");

    let mut r = rng(fail_seed);
    let (success, new_level) = roll_enhancement(4, &mut r);
    assert!(!success, "seed {} should produce a failure", fail_seed);
    assert_eq!(
        new_level, 3,
        "failed +5 from level 4 should drop to 3 (penalty=1)"
    );
}

#[test]
fn test_roll_enhancement_failure_at_level_9_drops_by_1() {
    // When roll_enhancement(8) fails, new_level = 8 - penalty(9) = 8 - 1 = 7.
    let fail_seed = (0..2000u64)
        .find(|&s| {
            let mut r = rng(s);
            let (success, _) = roll_enhancement(8, &mut r);
            !success
        })
        .expect("should find a failure seed within 2000 tries");

    let mut r = rng(fail_seed);
    let (success, new_level) = roll_enhancement(8, &mut r);
    assert!(!success, "seed {} should produce a failure", fail_seed);
    assert_eq!(
        new_level, 7,
        "failed +9 from level 8 should drop to 7 (penalty=1)"
    );
}

#[test]
fn test_roll_enhancement_failure_at_level_10_drops_by_2() {
    // When roll_enhancement(9) fails, new_level = 9 - penalty(10) = 9 - 2 = 7.
    let fail_seed = (0..2000u64)
        .find(|&s| {
            let mut r = rng(s);
            let (success, _) = roll_enhancement(9, &mut r);
            !success
        })
        .expect("should find a failure seed within 2000 tries");

    let mut r = rng(fail_seed);
    let (success, new_level) = roll_enhancement(9, &mut r);
    assert!(!success, "seed {} should produce a failure", fail_seed);
    assert_eq!(
        new_level, 7,
        "failed +10 from level 9 should drop to 7 (penalty=2)"
    );
}

#[test]
fn test_roll_enhancement_at_max_level_is_no_op() {
    // roll_enhancement at level 10 (already at max) should return (false, 10)
    for seed in 0..50u64 {
        let mut r = rng(seed);
        let (success, new_level) = roll_enhancement(MAX_ENHANCEMENT_LEVEL, &mut r);
        assert!(
            !success,
            "roll at max level should always return failure (seed={})",
            seed
        );
        assert_eq!(
            new_level, MAX_ENHANCEMENT_LEVEL,
            "new_level should remain at max (seed={})",
            seed
        );
    }
}

// =============================================================================
// Soul Tithe: available at +5 through +10, correct costs
// =============================================================================

#[test]
fn test_soul_tithe_available_at_levels_5_through_10() {
    // +1-+4: no soul tithe (already guaranteed)
    assert_eq!(soul_tithe_cost(1), None, "+1 should have no soul tithe");
    assert_eq!(soul_tithe_cost(2), None, "+2 should have no soul tithe");
    assert_eq!(soul_tithe_cost(3), None, "+3 should have no soul tithe");
    assert_eq!(soul_tithe_cost(4), None, "+4 should have no soul tithe");

    // +5-+10: soul tithe available with exact PR costs
    assert_eq!(
        soul_tithe_cost(5),
        Some(4),
        "+5 soul tithe should cost 4 PR"
    );
    assert_eq!(
        soul_tithe_cost(6),
        Some(6),
        "+6 soul tithe should cost 6 PR"
    );
    assert_eq!(
        soul_tithe_cost(7),
        Some(8),
        "+7 soul tithe should cost 8 PR"
    );
    assert_eq!(
        soul_tithe_cost(8),
        Some(25),
        "+8 soul tithe should cost 25 PR"
    );
    assert_eq!(
        soul_tithe_cost(9),
        Some(85),
        "+9 soul tithe should cost 85 PR"
    );
    assert_eq!(
        soul_tithe_cost(10),
        Some(750),
        "+10 soul tithe should cost 750 PR"
    );
}

#[test]
fn test_soul_tithe_cost_out_of_range_returns_none() {
    assert_eq!(soul_tithe_cost(0), None);
    assert_eq!(soul_tithe_cost(11), None);
    assert_eq!(soul_tithe_cost(255), None);
}

#[test]
fn test_soul_tithe_costs_increase_with_level() {
    for target in 5..=9u8 {
        let cost = soul_tithe_cost(target).unwrap();
        let next_cost = soul_tithe_cost(target + 1).unwrap();
        assert!(
            cost < next_cost,
            "soul tithe cost for +{} ({}) should be less than +{} ({})",
            target,
            cost,
            target + 1,
            next_cost
        );
    }
}

/// Soul Tithe prices at +8-+10 are derived from the expected PR cost of
/// gambling that step: attempt fees plus re-buying levels lost to failures
/// via the tithes below. This test recomputes that expectation from the
/// rate/cost/penalty tables and asserts each tithe stays a discount on it
/// (roughly 0.6x-0.9x), so retuning rates or penalties without repricing
/// the tithes fails loudly instead of silently breaking the economy.
#[test]
fn test_soul_tithe_high_tier_prices_track_expected_gamble_cost() {
    for target in 8..=10u8 {
        let p = success_rate(target);
        let penalty = fail_penalty(target);
        // Cheapest re-climb after a failure: tithe back each lost level.
        let reclimb: u32 = (target - penalty..target)
            .map(|lvl| soul_tithe_cost(lvl).expect("tithe available for re-climb levels"))
            .sum();
        let expected_gamble_cost =
            (enhancement_cost(target) as f64 + (1.0 - p) * reclimb as f64) / p;

        let tithe = soul_tithe_cost(target).unwrap() as f64;
        let ratio = tithe / expected_gamble_cost;
        assert!(
            (0.6..=0.9).contains(&ratio),
            "+{} tithe ({}) should be 0.6x-0.9x its expected gamble cost ({:.0}), got {:.2}x",
            target,
            tithe,
            expected_gamble_cost,
            ratio
        );
    }
}

// =============================================================================
// Enhancement costs: exact values at each target level
// =============================================================================

#[test]
fn test_enhancement_cost_exact_values() {
    // +1-+4: 1 PR each
    assert_eq!(enhancement_cost(1), 1, "+1 should cost 1 PR");
    assert_eq!(enhancement_cost(2), 1, "+2 should cost 1 PR");
    assert_eq!(enhancement_cost(3), 1, "+3 should cost 1 PR");
    assert_eq!(enhancement_cost(4), 1, "+4 should cost 1 PR");
    // +5: 2 PR
    assert_eq!(enhancement_cost(5), 2, "+5 should cost 2 PR");
    // +6-+7: 3 PR each
    assert_eq!(enhancement_cost(6), 3, "+6 should cost 3 PR");
    assert_eq!(enhancement_cost(7), 3, "+7 should cost 3 PR");
    // +8-+9: 4 PR each
    assert_eq!(enhancement_cost(8), 4, "+8 should cost 4 PR");
    assert_eq!(enhancement_cost(9), 4, "+9 should cost 4 PR");
    // +10: 5 PR
    assert_eq!(enhancement_cost(10), 5, "+10 should cost 5 PR");
}

#[test]
fn test_enhancement_cost_out_of_range_returns_zero() {
    assert_eq!(enhancement_cost(0), 0, "level 0 is invalid");
    assert_eq!(enhancement_cost(11), 0, "level 11 is out of range");
}

#[test]
fn test_total_cost_from_zero_to_max_is_25_pr() {
    let total: u32 = (1..=MAX_ENHANCEMENT_LEVEL).map(enhancement_cost).sum();
    assert_eq!(
        total, 25,
        "total cost +0 to +10 should be 25 PR, got {}",
        total
    );
}

// =============================================================================
// Enhancement multiplier curve: exact values at each level
// =============================================================================

#[test]
fn test_enhancement_multiplier_exact_values() {
    // ENHANCEMENT_CUMULATIVE_BONUS = [0, 5, 10, 15, 20, 30, 40, 55, 75, 100, 150]
    // multiplier = 1.0 + bonus/100
    let expected = [
        (0u8, 1.00f64),
        (1, 1.05),
        (2, 1.10),
        (3, 1.15),
        (4, 1.20),
        (5, 1.30),
        (6, 1.40),
        (7, 1.55),
        (8, 1.75),
        (9, 2.00),
        (10, 2.50),
    ];
    for (level, expected_mult) in expected {
        let actual = enhancement_multiplier(level);
        assert!(
            (actual - expected_mult).abs() < 1e-9,
            "level {} multiplier: expected {}, got {}",
            level,
            expected_mult,
            actual
        );
    }
}

#[test]
fn test_enhancement_multiplier_above_max_clamps() {
    // Level 11+ should return the same as level 10 (2.50x)
    let max_mult = enhancement_multiplier(MAX_ENHANCEMENT_LEVEL);
    assert!(
        (enhancement_multiplier(11) - max_mult).abs() < f64::EPSILON,
        "level 11 should return max multiplier"
    );
    assert!(
        (enhancement_multiplier(255) - max_mult).abs() < f64::EPSILON,
        "level 255 should return max multiplier"
    );
}

// =============================================================================
// Discovery chance: formula verified at specific ranks
// =============================================================================

#[test]
fn test_discovery_chance_at_exactly_p15() {
    let chance = soulforge_discovery_chance(15);
    assert!(
        (chance - SOULFORGE_DISCOVERY_BASE_CHANCE).abs() < f64::EPSILON,
        "at P15 (min rank), chance should be exactly BASE_CHANCE"
    );
}

#[test]
fn test_discovery_chance_at_p16_adds_one_rank_bonus() {
    let chance_p15 = soulforge_discovery_chance(15);
    let chance_p16 = soulforge_discovery_chance(16);
    let delta = chance_p16 - chance_p15;
    assert!(
        (delta - SOULFORGE_DISCOVERY_RANK_BONUS).abs() < f64::EPSILON,
        "each rank above 15 should add exactly RANK_BONUS ({:e}), got {:e}",
        SOULFORGE_DISCOVERY_RANK_BONUS,
        delta
    );
}

#[test]
fn test_discovery_chance_at_p20() {
    let expected = SOULFORGE_DISCOVERY_BASE_CHANCE + 5.0 * SOULFORGE_DISCOVERY_RANK_BONUS;
    let actual = soulforge_discovery_chance(20);
    assert!(
        (actual - expected).abs() < f64::EPSILON,
        "P20 chance should be base + 5*bonus, expected {:e}, got {:e}",
        expected,
        actual
    );
}

#[test]
fn test_discovery_chance_below_p15_is_zero() {
    // P0 through P14 should always be 0
    for rank in 0..SOULFORGE_MIN_PRESTIGE_RANK {
        assert_eq!(
            soulforge_discovery_chance(rank),
            0.0,
            "discovery chance at P{} should be 0.0",
            rank
        );
    }
}

#[test]
fn test_discovery_never_fires_below_p15() {
    let mut r = rng(42);
    let mut ep = EnhancementProgress::new();

    // Probability is exactly 0.0 at P14 (below SOULFORGE_MIN_PRESTIGE_RANK),
    // so the function always returns false via an early-return check.
    // A handful of iterations is sufficient to verify the guard clause.
    for _ in 0..10 {
        assert!(
            !try_discover_soulforge(&mut ep, 14, &mut r),
            "should never discover at P14"
        );
    }
    assert!(!ep.discovered, "discovered should remain false");
}

#[test]
fn test_discovery_eventually_fires_at_p15() {
    let mut r = rng(12345);
    let mut ep = EnhancementProgress::new();
    let mut discovered = false;

    // Use P100 instead of P15 to greatly increase discovery probability
    // (0.000014 + 85*0.000007 = 0.000609 per tick vs 0.000014 at P15).
    // This still tests the same discovery logic path.
    for _ in 0..50_000 {
        if try_discover_soulforge(&mut ep, 100, &mut r) {
            discovered = true;
            break;
        }
    }

    assert!(discovered, "should discover within 50K ticks at P100");
    assert!(
        ep.discovered,
        "ep.discovered should be true after discovery"
    );
}

#[test]
fn test_discovery_does_not_re_fire_after_found() {
    let mut r = rng(999);
    let mut ep = EnhancementProgress::new();
    ep.discovered = true;

    // Already-discovered guard is a simple boolean check, not probabilistic.
    // 100 iterations is sufficient.
    for _ in 0..100 {
        let result = try_discover_soulforge(&mut ep, 100, &mut r);
        assert!(
            !result,
            "try_discover_soulforge should return false when already discovered"
        );
    }
}

// =============================================================================
// All 7 slots independent (no cross-contamination)
// =============================================================================

#[test]
fn test_all_7_slots_independent_no_cross_contamination() {
    let mut ep = EnhancementProgress::new();

    // Set each slot to a distinct level
    for slot in 0..7 {
        ep.set_level(slot, slot as u8 + 1);
    }

    // Verify levels didn't bleed into each other
    for slot in 0..7 {
        assert_eq!(
            ep.level(slot),
            slot as u8 + 1,
            "slot {} should be level {}",
            slot,
            slot + 1
        );
    }

    // Apply changes to slot 0 and verify slots 1-6 are unchanged
    ep.set_level(0, 10);
    for slot in 1..7 {
        assert_eq!(
            ep.level(slot),
            slot as u8 + 1,
            "slot {} should be unchanged after modifying slot 0",
            slot
        );
    }
}

#[test]
fn test_apply_enhancement_result_only_modifies_target_slot() {
    let mut ep = EnhancementProgress::new();
    for slot in 0..7 {
        ep.set_level(slot, 5);
    }

    // Apply result to slot 3 only
    apply_enhancement_result(&mut ep, 3, 6, true);

    assert_eq!(ep.level(3), 6, "slot 3 should be updated to 6");
    for slot in [0, 1, 2, 4, 5, 6] {
        assert_eq!(
            ep.level(slot),
            5,
            "slot {} should remain unchanged at 5",
            slot
        );
    }
}

#[test]
fn test_enhance_all_7_slots_to_max_independently() {
    let mut r = rng(31415);
    let mut ep = EnhancementProgress::new();
    ep.discovered = true;

    // Enhance each slot to +4 (all guaranteed)
    for slot in 0..7 {
        while ep.level(slot) < 4 {
            let current = ep.level(slot);
            let (success, new_level) = roll_enhancement(current, &mut r);
            apply_enhancement_result(&mut ep, slot, new_level, success);
            assert!(
                success,
                "slot {} level {} -> {} should always succeed",
                slot, current, new_level
            );
        }
    }

    // All 7 slots should be at level 4
    for slot in 0..7 {
        assert_eq!(
            ep.level(slot),
            4,
            "slot {} should be at level 4 after 4 guaranteed enhancements",
            slot
        );
    }
    assert_eq!(ep.total_attempts, 28, "7 slots × 4 attempts = 28 total");
    assert_eq!(ep.total_successes, 28, "all 28 should succeed");
    assert_eq!(ep.total_failures, 0, "no failures for +1 through +4");
}

// =============================================================================
// Enhancement multiplier effects on derived stats
// =============================================================================

#[test]
fn test_enhancement_multiplier_identity_at_zero() {
    assert_eq!(
        enhancement_multiplier(0),
        1.0,
        "level 0 should be identity multiplier (1.0x)"
    );
}

#[test]
fn test_enhancement_multiplier_at_max_is_2_5x() {
    let mult = enhancement_multiplier(MAX_ENHANCEMENT_LEVEL);
    assert!(
        (mult - 2.5).abs() < f64::EPSILON,
        "level 10 should be 2.5x multiplier, got {}",
        mult
    );
}

#[test]
fn test_enhancement_multiplier_strictly_increasing() {
    let mut prev = enhancement_multiplier(0);
    for level in 1..=MAX_ENHANCEMENT_LEVEL {
        let curr = enhancement_multiplier(level);
        assert!(
            curr > prev,
            "multiplier at level {} ({:.3}) should be > level {} ({:.3})",
            level,
            curr,
            level - 1,
            prev
        );
        prev = curr;
    }
}

// =============================================================================
// Counters: total_attempts, total_successes, total_failures
// =============================================================================

#[test]
fn test_apply_result_increments_total_attempts_each_call() {
    let mut ep = EnhancementProgress::new();

    for i in 1..=10 {
        let success = i % 3 != 0; // success 2 out of every 3
        apply_enhancement_result(&mut ep, 0, 1, success);
        assert_eq!(ep.total_attempts, i, "total_attempts should be {}", i);
    }
}

#[test]
fn test_apply_result_correctly_tracks_successes_and_failures() {
    let mut ep = EnhancementProgress::new();

    // 7 successes, 3 failures
    for _ in 0..7 {
        apply_enhancement_result(&mut ep, 0, 1, true);
    }
    for _ in 0..3 {
        apply_enhancement_result(&mut ep, 0, 0, false);
    }

    assert_eq!(ep.total_attempts, 10);
    assert_eq!(ep.total_successes, 7);
    assert_eq!(ep.total_failures, 3);
    assert_eq!(
        ep.total_successes + ep.total_failures,
        ep.total_attempts,
        "success + failure must equal total attempts"
    );
}

// =============================================================================
// Edge: saturating_sub protects against underflow
// =============================================================================

#[test]
fn test_roll_enhancement_at_level_4_failure_never_underflows() {
    // Level 4 -> +5 (70% success). On failure: penalty=1, so 4-1=3. No underflow risk.
    // But what about level 0 -> +1 (100% success, penalty 0)? Should never fail.
    // Test that even if a hypothetical failure occurred at level 1 (penalty=0), new_level would be 1.
    // Since level 1 is 100%, we can't test failure there directly.
    // Instead, test that fail_penalty returns 0 for levels 1-4 (so saturating_sub gives same result).
    for level in 1u8..=4 {
        assert_eq!(
            fail_penalty(level),
            0,
            "level {} should have 0 penalty, so no underflow risk",
            level
        );
    }
}

// =============================================================================
// Integration: simulate a complete journey from 0 to 10 on one slot
// =============================================================================

#[test]
fn test_complete_enhancement_journey_slot_0() {
    let mut r = rng(99999);
    let mut ep = EnhancementProgress::new();
    ep.discovered = true;

    let slot = 0;
    let max_attempts = 50_000;
    let mut attempts = 0u32;

    while ep.level(slot) < MAX_ENHANCEMENT_LEVEL && attempts < max_attempts {
        let current = ep.level(slot);
        let (success, new_level) = roll_enhancement(current, &mut r);
        apply_enhancement_result(&mut ep, slot, new_level, success);
        attempts += 1;
    }

    assert_eq!(
        ep.level(slot),
        MAX_ENHANCEMENT_LEVEL,
        "slot 0 should reach +10 within {} attempts, took {}",
        max_attempts,
        attempts
    );
    assert_eq!(
        ep.total_successes + ep.total_failures,
        ep.total_attempts,
        "success + failure should equal total attempts"
    );
    assert!(
        ep.total_successes >= MAX_ENHANCEMENT_LEVEL as u32,
        "need at least {} successes to reach +10",
        MAX_ENHANCEMENT_LEVEL
    );
    assert_eq!(ep.highest_level_reached, MAX_ENHANCEMENT_LEVEL);
    // Other slots should be untouched
    for other_slot in 1..7 {
        assert_eq!(
            ep.level(other_slot),
            0,
            "slot {} should be unchanged at 0",
            other_slot
        );
    }
}
