use quest::ascension::logic::{ascend, can_ascend, AscendResult};
use quest::ascension::types::{ascension_combat_multiplier, ascension_cost, ascension_deep_gate};
use quest::GameState;

#[test]
fn test_can_ascend_basic() {
    // Level 0 -> 1: needs 35 PR and Deep layer 3
    assert!(can_ascend(0, 35, 3));
    assert!(!can_ascend(0, 34, 3)); // insufficient PR
    assert!(!can_ascend(0, 35, 2)); // deep gate not met
}

#[test]
fn test_cannot_ascend_past_max_level() {
    // Level 6 is max — cannot ascend further regardless of PR or depth
    assert!(!can_ascend(6, 10000, 30));
    assert!(!can_ascend(6, 575, 0));
}

#[test]
fn test_ascend_deducts_pr() {
    let mut state = GameState::new("Test".to_string(), 0);
    state.prestige_rank = 50;
    state.ascension_level = 0;

    let result = ascend(&mut state, 3);
    assert_eq!(
        result,
        AscendResult::Success {
            new_level: 1,
            multiplier: 2.0
        }
    );
    assert_eq!(state.prestige_rank, 15); // 50 - 35
    assert_eq!(state.ascension_level, 1);
}

#[test]
fn test_ascend_insufficient_pr() {
    let mut state = GameState::new("Test".to_string(), 0);
    state.prestige_rank = 30;
    state.ascension_level = 0;

    let result = ascend(&mut state, 3);
    assert_eq!(
        result,
        AscendResult::InsufficientPR {
            needed: 35,
            have: 30
        }
    );
    assert_eq!(state.prestige_rank, 30); // unchanged
    assert_eq!(state.ascension_level, 0); // unchanged
}

#[test]
fn test_ascend_deep_gate_not_met() {
    let mut state = GameState::new("Test".to_string(), 0);
    state.prestige_rank = 100;
    state.ascension_level = 0;

    let result = ascend(&mut state, 2); // need layer 3
    assert_eq!(
        result,
        AscendResult::DeepGateNotMet {
            needed_layer: 3,
            current_layer: 2
        }
    );
    assert_eq!(state.prestige_rank, 100); // unchanged
}

#[test]
fn test_ascension_level_serialization() {
    let mut state = GameState::new("Test".to_string(), 0);
    state.ascension_level = 4;

    let json = serde_json::to_string(&state).unwrap();
    let loaded: GameState = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.ascension_level, 4);
}

#[test]
fn test_ascension_level_defaults_to_zero() {
    // Simulate loading from old save without ascension_level
    let state = GameState::new("Test".to_string(), 0);
    assert_eq!(state.ascension_level, 0);
}

#[test]
fn test_new_character_starts_at_ascension_zero() {
    let state = GameState::new("Hero".to_string(), 0);
    assert_eq!(state.ascension_level, 0);
}

#[test]
fn test_ascend_at_max_level_returns_max_level_reached() {
    let mut state = GameState::new("Test".to_string(), 0);
    state.prestige_rank = 10000;
    state.ascension_level = 6; // max

    let result = ascend(&mut state, 30);
    assert_eq!(result, AscendResult::MaxLevelReached);
    assert_eq!(state.ascension_level, 6); // unchanged
    assert_eq!(state.prestige_rank, 10000); // unchanged
}

// --- Edge case tests ---

#[test]
fn test_can_ascend_exact_pr_boundary_level_1() {
    // Exactly 35 PR with layer 3: should succeed
    assert!(can_ascend(0, 35, 3));
}

#[test]
fn test_can_ascend_one_less_pr_than_needed_level_1() {
    // 34 PR (one less than 35 required): should fail
    assert!(!can_ascend(0, 34, 3));
}

#[test]
fn test_can_ascend_exact_deep_layer_gate_level_1() {
    // Exactly layer 3 with sufficient PR: should succeed
    assert!(can_ascend(0, 35, 3));
}

#[test]
fn test_can_ascend_deep_layer_2_for_level_1_fails() {
    // Layer 2 (one below required gate 3): should fail
    assert!(!can_ascend(0, 10000, 2));
}

#[test]
fn test_can_ascend_exact_boundaries_all_levels() {
    // Verify exact boundary conditions for each level 1-6
    let costs = [35u32, 65, 120, 200, 325, 500];
    let gates = [3u32, 7, 12, 18, 25, 30];

    for (i, (&cost, &gate)) in costs.iter().zip(gates.iter()).enumerate() {
        let current_level = i as u32;
        // Exact boundary: should succeed
        assert!(
            can_ascend(current_level, cost, gate),
            "Expected can_ascend({current_level}, {cost}, {gate}) to be true"
        );
        // One less PR: should fail
        assert!(
            !can_ascend(current_level, cost - 1, gate),
            "Expected can_ascend({current_level}, {}, {gate}) to be false",
            cost - 1
        );
        // One less layer: should fail
        if gate > 0 {
            assert!(
                !can_ascend(current_level, cost, gate - 1),
                "Expected can_ascend({current_level}, {cost}, {}) to be false",
                gate - 1
            );
        }
    }
}

#[test]
fn test_ascend_to_level_vi_succeeds() {
    // Ascending from level 5 to level VI (the cap): should succeed
    let mut state = GameState::new("Test".to_string(), 0);
    state.prestige_rank = 600; // more than 500 required
    state.ascension_level = 5;

    let result = ascend(&mut state, 30); // gate is layer 30
    assert_eq!(
        result,
        AscendResult::Success {
            new_level: 6,
            multiplier: 64.0
        }
    );
    assert_eq!(state.ascension_level, 6);
    assert_eq!(state.prestige_rank, 100); // 600 - 500
}

#[test]
fn test_ascend_beyond_vi_returns_max_level_reached() {
    // Ascension is capped at level VI — attempting level VII returns MaxLevelReached
    let mut state = GameState::new("Test".to_string(), 0);
    state.prestige_rank = 10000;
    state.ascension_level = 6;

    let result = ascend(&mut state, 30);
    assert_eq!(result, AscendResult::MaxLevelReached);
    // State is unchanged
    assert_eq!(state.ascension_level, 6);
    assert_eq!(state.prestige_rank, 10000);
}

#[test]
fn test_ascension_multiplier_at_each_level_i_through_vi() {
    // Level 0 (no ascension): 1.0x
    assert!((ascension_combat_multiplier(0) - 1.0).abs() < 1e-10);
    // Level I: 2x
    assert!((ascension_combat_multiplier(1) - 2.0).abs() < 1e-10);
    // Level II: 4x
    assert!((ascension_combat_multiplier(2) - 4.0).abs() < 1e-10);
    // Level III: 8x
    assert!((ascension_combat_multiplier(3) - 8.0).abs() < 1e-10);
    // Level IV: 16x
    assert!((ascension_combat_multiplier(4) - 16.0).abs() < 1e-10);
    // Level V: 32x
    assert!((ascension_combat_multiplier(5) - 32.0).abs() < 1e-10);
    // Level VI: 64x
    assert!((ascension_combat_multiplier(6) - 64.0).abs() < 1e-10);
}

#[test]
fn test_ascension_multiplier_at_level_vii_plus_diminishing_returns() {
    // Level VII: 64 * 1.5^1 = 96
    assert!((ascension_combat_multiplier(7) - 96.0).abs() < 1e-10);
    // Level VIII: 64 * 1.5^2 = 144
    assert!((ascension_combat_multiplier(8) - 144.0).abs() < 1e-10);
    // Level IX: 64 * 1.5^3 = 216
    assert!((ascension_combat_multiplier(9) - 216.0).abs() < 1e-10);
    // Level X: 64 * 1.5^4 = 324
    assert!((ascension_combat_multiplier(10) - 324.0).abs() < 1e-10);
}

#[test]
fn test_ascension_multiplier_at_level_zero_is_one() {
    // No ascension = 1.0x (no multiplier)
    let m = ascension_combat_multiplier(0);
    assert!((m - 1.0).abs() < 1e-10, "Expected 1.0 at level 0, got {m}");
}

#[test]
fn test_total_pr_cost_i_through_vi_is_1245() {
    let total: u32 = (1..=6).map(ascension_cost).sum();
    assert_eq!(total, 1245, "Total PR for levels I-VI should be 1,245");
}

#[test]
fn test_ascension_cost_each_level() {
    assert_eq!(ascension_cost(1), 35);
    assert_eq!(ascension_cost(2), 65);
    assert_eq!(ascension_cost(3), 120);
    assert_eq!(ascension_cost(4), 200);
    assert_eq!(ascension_cost(5), 325);
    assert_eq!(ascension_cost(6), 500);
    // Level VII+: 500 + 75*(level-6)
    assert_eq!(ascension_cost(7), 575); // 500 + 75*1
    assert_eq!(ascension_cost(8), 650); // 500 + 75*2
    assert_eq!(ascension_cost(10), 800); // 500 + 75*4
}

#[test]
fn test_ascension_deep_gate_each_level() {
    assert_eq!(ascension_deep_gate(1), Some(3));
    assert_eq!(ascension_deep_gate(2), Some(7));
    assert_eq!(ascension_deep_gate(3), Some(12));
    assert_eq!(ascension_deep_gate(4), Some(18));
    assert_eq!(ascension_deep_gate(5), Some(25));
    assert_eq!(ascension_deep_gate(6), Some(30));
    // Level VII+ has no Deep gate (PR only)
    assert_eq!(ascension_deep_gate(7), None);
    assert_eq!(ascension_deep_gate(100), None);
}

#[test]
fn test_ascend_exact_pr_with_exact_gate_level_1() {
    // Exact boundary: 35 PR, layer 3 — should succeed and deduct exactly 35 PR
    let mut state = GameState::new("Test".to_string(), 0);
    state.prestige_rank = 35;
    state.ascension_level = 0;

    let result = ascend(&mut state, 3);
    assert_eq!(
        result,
        AscendResult::Success {
            new_level: 1,
            multiplier: 2.0
        }
    );
    assert_eq!(state.prestige_rank, 0); // exactly spent
    assert_eq!(state.ascension_level, 1);
}

#[test]
fn test_ascend_pr_deducted_correctly_for_all_levels() {
    // Walk through all 6 levels, verifying correct PR deduction at each step
    let mut state = GameState::new("Test".to_string(), 0);
    // Fund with total PR needed
    state.prestige_rank = 1245;

    let costs = [35u32, 65, 120, 200, 325, 500];
    let gates = [3u32, 7, 12, 18, 25, 30];
    let mut expected_remaining = 1245u32;

    for (i, (&cost, &gate)) in costs.iter().zip(gates.iter()).enumerate() {
        let expected_level = (i + 1) as u32;
        let result = ascend(&mut state, gate);
        expected_remaining -= cost;

        assert_eq!(
            result,
            AscendResult::Success {
                new_level: expected_level,
                multiplier: ascension_combat_multiplier(expected_level),
            },
            "Expected Success at level {expected_level}"
        );
        assert_eq!(
            state.prestige_rank, expected_remaining,
            "PR should be {expected_remaining} after ascending to level {expected_level}"
        );
        assert_eq!(state.ascension_level, expected_level);
    }

    // After VI, further ascend should return MaxLevelReached
    let result = ascend(&mut state, 30);
    assert_eq!(result, AscendResult::MaxLevelReached);
}

#[test]
fn test_can_ascend_level_5_to_6_exact_boundary() {
    // Exactly 500 PR and layer 30: should succeed
    assert!(can_ascend(5, 500, 30));
    // 499 PR: should fail
    assert!(!can_ascend(5, 499, 30));
    // Layer 29: should fail
    assert!(!can_ascend(5, 500, 29));
}

#[test]
fn test_ascension_multiplier_doubles_each_level() {
    // Verify each level I-VI is exactly 2x the previous level
    for level in 1..=6u32 {
        let current = ascension_combat_multiplier(level);
        let previous = ascension_combat_multiplier(level - 1);
        assert!(
            (current - previous * 2.0).abs() < 1e-10,
            "Level {level} multiplier ({current}) should be 2x level {} ({previous})",
            level - 1
        );
    }
}
