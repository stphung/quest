use quest::ascension::logic::{ascend, can_ascend, AscendResult};
use quest::GameState;

#[test]
fn test_can_ascend_basic() {
    // Level 0 -> 1: needs 10 PR and Deep layer 3
    assert!(can_ascend(0, 10, 3));
    assert!(!can_ascend(0, 9, 3)); // insufficient PR
    assert!(!can_ascend(0, 10, 2)); // deep gate not met
}

#[test]
fn test_can_ascend_level_7_no_deep_gate() {
    // Level 6 -> 7: needs 80 PR, no Deep gate
    assert!(can_ascend(6, 80, 0)); // deepest_layer doesn't matter
    assert!(!can_ascend(6, 79, 30)); // PR insufficient
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
    assert_eq!(state.prestige_rank, 40); // 50 - 10
    assert_eq!(state.ascension_level, 1);
}

#[test]
fn test_ascend_insufficient_pr() {
    let mut state = GameState::new("Test".to_string(), 0);
    state.prestige_rank = 5;
    state.ascension_level = 0;

    let result = ascend(&mut state, 3);
    assert_eq!(
        result,
        AscendResult::InsufficientPR {
            needed: 10,
            have: 5
        }
    );
    assert_eq!(state.prestige_rank, 5); // unchanged
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
