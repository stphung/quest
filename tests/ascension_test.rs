use quest::ascension::logic::{ascend, can_ascend, AscendResult};
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
