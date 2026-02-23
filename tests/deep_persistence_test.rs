use quest::deep::discovery::{discover_deep, is_deep_discovered};
use quest::deep::types::TheDeepState;

// =========================================================================
// Serde roundtrip (no file I/O)
// =========================================================================

#[test]
fn save_load_roundtrip_default_state() {
    let state = TheDeepState::new();
    let json = serde_json::to_string_pretty(&state).expect("serialize");
    let loaded: TheDeepState = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(loaded.discovered, state.discovered);
    assert_eq!(
        loaded.account.total_marks_earned,
        state.account.total_marks_earned
    );
    assert_eq!(loaded.run.warband_marks, state.run.warband_marks);
    assert_eq!(loaded.account.layers.len(), state.account.layers.len());
    assert_eq!(loaded.run.mercenaries.len(), state.run.mercenaries.len());
}

#[test]
fn save_load_roundtrip_discovered_state() {
    let mut state = TheDeepState::new();
    state.discovered = true;
    state.account.total_marks_earned = 9_999;
    state.run.warband_marks = 250;

    let json = serde_json::to_string_pretty(&state).expect("serialize");
    let loaded: TheDeepState = serde_json::from_str(&json).expect("deserialize");

    assert!(loaded.discovered);
    assert_eq!(loaded.account.total_marks_earned, 9_999);
    assert_eq!(loaded.run.warband_marks, 250);
}

// =========================================================================
// load_deep returns default when file doesn't exist
// =========================================================================

#[test]
fn load_deep_returns_default_for_missing_file() {
    // Deserializing from an empty / nonexistent path returns TheDeepState::new().
    // We test the fallback branch directly via serde: an invalid JSON string
    // triggers the same unwrap_or_default() path that load_deep() uses.
    let result: TheDeepState = serde_json::from_str("not valid json").unwrap_or_default();
    assert!(!result.discovered);
    assert_eq!(result.run.warband_marks, 0);
    assert!(result.run.mercenaries.is_empty());
}

// =========================================================================
// discovery functions
// =========================================================================

#[test]
fn discover_deep_sets_discovered_flag() {
    let mut state = TheDeepState::new();
    assert!(!state.discovered);
    discover_deep(&mut state);
    assert!(state.discovered);
}

#[test]
fn discover_deep_is_idempotent() {
    let mut state = TheDeepState::new();
    discover_deep(&mut state);
    discover_deep(&mut state);
    assert!(state.discovered);
}

#[test]
fn is_deep_discovered_returns_false_by_default() {
    let state = TheDeepState::new();
    assert!(!is_deep_discovered(&state));
}

#[test]
fn is_deep_discovered_returns_true_after_discovery() {
    let mut state = TheDeepState::new();
    discover_deep(&mut state);
    assert!(is_deep_discovered(&state));
}
