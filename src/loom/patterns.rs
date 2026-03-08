//! Woven Pattern tracking, completion, and progression.
//!
//! Patterns are the sole progression gate in the Loom of Worlds.
//! Each tick, the sustain timer advances for requirements whose production
//! rate meets or exceeds the required threshold. When the timer reaches
//! the target duration, the requirement completes. When all requirements
//! complete, the pattern completes and the next pattern unlocks.
#![allow(dead_code)]

use super::types::{LoomPersistent, Resource};
use std::collections::HashMap;

// ── Completion check ───────────────────────────────────────────────────────────

/// Check if all requirements of the active pattern have been individually completed.
///
/// Returns `true` if every requirement has `completed == true`,
/// `false` if any requirement is incomplete or if there is no active pattern.
pub fn active_pattern_requirements_met(persistent: &LoomPersistent) -> bool {
    let Some(pattern) = persistent.patterns.get(persistent.active_pattern) else {
        return false;
    };
    if pattern.completed {
        return false;
    }
    pattern.requirements.iter().all(|req| req.completed)
}

// ── Sustain tick ─────────────────────────────────────────────────────────────

/// Tick the sustain timer for the active pattern.
///
/// Called once per game tick. `delta_seconds` is the wall-clock time elapsed
/// since the last tick (typically 0.1s for a 100ms tick interval).
/// `rates` maps each resource to its current production rate in units/hour.
///
/// For each incomplete requirement, if the current rate meets or exceeds
/// the `required_rate` threshold, `sustained_secs` advances by `delta_seconds`.
/// When `sustained_secs` reaches `sustain_duration_secs`, the requirement
/// completes. If the rate drops below threshold, progress simply pauses
/// (no decay).
///
/// Returns `true` if a pattern was completed during this tick (so the caller
/// can emit a `TickEvent` and set `loom_changed = true`).
pub fn tick_pattern_sustain(
    persistent: &mut LoomPersistent,
    rates: &HashMap<Resource, f64>,
    delta_seconds: f64,
) -> bool {
    let Some(pattern) = persistent.patterns.get_mut(persistent.active_pattern) else {
        return false;
    };
    if pattern.completed {
        return false;
    }

    for req in &mut pattern.requirements {
        if req.completed {
            continue;
        }
        let rate = rates.get(&req.resource).copied().unwrap_or(0.0);
        if rate >= req.required_rate {
            req.sustained_secs += delta_seconds;
            if req.sustained_secs >= req.sustain_duration_secs {
                req.sustained_secs = req.sustain_duration_secs;
                req.completed = true;
            }
        }
        // Simple pause: do nothing when rate < threshold. No decay.
    }

    if pattern.requirements.iter().all(|req| req.completed) {
        complete_active_pattern(persistent);
        return true;
    }
    false
}

// ── Pattern completion ─────────────────────────────────────────────────────────

/// Mark the active pattern as complete and advance to the next pattern.
///
/// - Sets `completed = true` on the current pattern.
/// - Advances `active_pattern` to the next uncompleted pattern.
/// - Grants any associated rewards (currently a no-op placeholder; rewards
///   will be layered in as the recipe/node upgrade systems stabilize).
fn complete_active_pattern(persistent: &mut LoomPersistent) {
    if let Some(pattern) = persistent.patterns.get_mut(persistent.active_pattern) {
        pattern.completed = true;
    }

    advance_to_next_pattern(persistent);
    // Future: grant_pattern_rewards(persistent, completed_index);
}

/// Advance `active_pattern` to the first uncompleted pattern after the current one.
fn advance_to_next_pattern(persistent: &mut LoomPersistent) {
    let start = persistent.active_pattern + 1;
    for i in start..persistent.patterns.len() {
        if !persistent.patterns[i].completed {
            persistent.active_pattern = i;
            return;
        }
    }
    // All patterns completed — keep active_pattern at the last index.
    // The UI will show the completion state.
}

// ── Query helpers ──────────────────────────────────────────────────────────────

/// Returns `true` if all 18 patterns have been completed.
pub fn all_patterns_complete(persistent: &LoomPersistent) -> bool {
    !persistent.patterns.is_empty() && persistent.patterns.iter().all(|p| p.completed)
}

/// Returns `(sustained_secs, sustain_duration_secs, completed)` for each requirement
/// of the active pattern. Useful for the pattern bar UI to render individual progress.
pub fn active_pattern_requirement_status(persistent: &LoomPersistent) -> Vec<(f64, f64, bool)> {
    let Some(pattern) = persistent.patterns.get(persistent.active_pattern) else {
        return Vec::new();
    };
    pattern
        .requirements
        .iter()
        .map(|req| (req.sustained_secs, req.sustain_duration_secs, req.completed))
        .collect()
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loom::discovery::complete_discovery;
    use crate::loom::types::{LoomState, Resource};

    fn state_with_patterns() -> LoomState {
        let mut state = LoomState::new();
        complete_discovery(&mut state);
        // Set non-zero rate thresholds and durations for testing.
        // (Discovery patterns currently use legacy amount-based values;
        // Task 5 will replace them with proper rate-based definitions.)
        for pattern in &mut state.persistent.patterns {
            for req in &mut pattern.requirements {
                if req.required_rate == 0.0 {
                    req.required_rate = 10.0; // 10 units/hr threshold
                }
                if req.sustain_duration_secs == 0.0 {
                    req.sustain_duration_secs = 3600.0; // 1 hour sustain
                }
            }
        }
        state
    }

    fn rates(pairs: &[(Resource, f64)]) -> HashMap<Resource, f64> {
        pairs.iter().cloned().collect()
    }

    #[test]
    fn test_sustain_advances_when_rate_meets_threshold() {
        let mut state = state_with_patterns();
        let threshold = state.persistent.patterns[0].requirements[0].required_rate;
        let r = rates(&[(Resource::Ember, threshold + 5.0)]);
        tick_pattern_sustain(&mut state.persistent, &r, 1.0);
        let req = &state.persistent.patterns[0].requirements[0];
        assert!((req.sustained_secs - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_sustain_pauses_when_rate_below_threshold() {
        let mut state = state_with_patterns();
        state.persistent.patterns[0].requirements[0].sustained_secs = 100.0;
        let r = rates(&[(Resource::Ember, 1.0)]); // well below any threshold
        tick_pattern_sustain(&mut state.persistent, &r, 1.0);
        assert!((state.persistent.patterns[0].requirements[0].sustained_secs - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_sustain_never_decays() {
        let mut state = state_with_patterns();
        state.persistent.patterns[0].requirements[0].sustained_secs = 50.0;
        let r = rates(&[]);
        tick_pattern_sustain(&mut state.persistent, &r, 10.0);
        assert!(state.persistent.patterns[0].requirements[0].sustained_secs >= 50.0);
    }

    #[test]
    fn test_requirement_completes_when_duration_reached() {
        let mut state = state_with_patterns();
        let req = &state.persistent.patterns[0].requirements[0];
        let duration = req.sustain_duration_secs;
        let threshold = req.required_rate;
        state.persistent.patterns[0].requirements[0].sustained_secs = duration - 0.5;
        let r = rates(&[(Resource::Ember, threshold + 10.0)]);
        tick_pattern_sustain(&mut state.persistent, &r, 1.0);
        assert!(state.persistent.patterns[0].requirements[0].completed);
    }

    #[test]
    fn test_pattern_completes_when_all_requirements_complete() {
        let mut state = state_with_patterns();
        for req in &mut state.persistent.patterns[0].requirements {
            req.sustained_secs = req.sustain_duration_secs - 0.1;
        }
        let r = rates(&[(Resource::Ember, 100.0)]);
        let completed = tick_pattern_sustain(&mut state.persistent, &r, 1.0);
        assert!(completed);
        assert!(state.persistent.patterns[0].completed);
    }

    #[test]
    fn test_active_pattern_advances_after_completion() {
        let mut state = state_with_patterns();
        for req in &mut state.persistent.patterns[0].requirements {
            req.sustained_secs = req.sustain_duration_secs - 0.1;
        }
        let r = rates(&[(Resource::Ember, 100.0)]);
        tick_pattern_sustain(&mut state.persistent, &r, 1.0);
        assert_eq!(state.persistent.active_pattern, 1);
    }

    #[test]
    fn test_already_completed_requirement_not_advanced() {
        let mut state = state_with_patterns();
        state.persistent.patterns[0].requirements[0].completed = true;
        state.persistent.patterns[0].requirements[0].sustained_secs = 100.0;
        let r = rates(&[(Resource::Ember, 100.0)]);
        tick_pattern_sustain(&mut state.persistent, &r, 1.0);
        assert!((state.persistent.patterns[0].requirements[0].sustained_secs - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_no_progress_on_zero_delta() {
        let mut state = state_with_patterns();
        let r = rates(&[(Resource::Ember, 100.0)]);
        tick_pattern_sustain(&mut state.persistent, &r, 0.0);
        assert!((state.persistent.patterns[0].requirements[0].sustained_secs).abs() < 1e-9);
    }

    #[test]
    fn test_exact_threshold_advances() {
        let mut state = state_with_patterns();
        let threshold = state.persistent.patterns[0].requirements[0].required_rate;
        let r = rates(&[(Resource::Ember, threshold)]); // exact threshold
        tick_pattern_sustain(&mut state.persistent, &r, 1.0);
        assert!(state.persistent.patterns[0].requirements[0].sustained_secs > 0.0);
    }

    // ── active_pattern_requirements_met ──

    #[test]
    fn test_requirements_met_when_all_completed() {
        let mut state = state_with_patterns();
        for req in &mut state.persistent.patterns[0].requirements {
            req.completed = true;
        }
        assert!(active_pattern_requirements_met(&state.persistent));
    }

    #[test]
    fn test_requirements_not_met_when_pattern_already_completed() {
        let mut state = state_with_patterns();
        state.persistent.patterns[0].completed = true;
        assert!(!active_pattern_requirements_met(&state.persistent));
    }

    // ── all_patterns_complete ──

    #[test]
    fn test_not_all_complete_when_some_remain() {
        let state = state_with_patterns();
        assert!(!all_patterns_complete(&state.persistent));
    }

    #[test]
    fn test_all_complete_when_every_pattern_marked() {
        let mut state = state_with_patterns();
        for p in &mut state.persistent.patterns {
            p.completed = true;
        }
        assert!(all_patterns_complete(&state.persistent));
    }

    // ── active_pattern_requirement_status ──

    #[test]
    fn test_requirement_status_returns_sustained_and_duration() {
        let mut state = state_with_patterns();
        state.persistent.patterns[0].requirements[0].sustained_secs = 100.0;
        let status = active_pattern_requirement_status(&state.persistent);
        assert_eq!(status.len(), 1);
        let (sustained, duration, completed) = status[0];
        assert!((sustained - 100.0).abs() < 1e-9);
        assert!(duration > 0.0);
        assert!(!completed);
    }

    #[test]
    fn test_requirement_status_empty_when_no_patterns() {
        let state = LoomState::new();
        let status = active_pattern_requirement_status(&state.persistent);
        assert!(status.is_empty());
    }

    // ── advance_to_next_pattern ──

    #[test]
    fn test_advance_skips_already_completed_patterns() {
        let mut state = state_with_patterns();
        state.persistent.patterns[0].completed = true;
        state.persistent.patterns[1].completed = true;
        state.persistent.active_pattern = 0;
        advance_to_next_pattern(&mut state.persistent);
        assert_eq!(state.persistent.active_pattern, 2);
    }

    #[test]
    fn test_advance_stays_at_last_when_all_complete() {
        let mut state = state_with_patterns();
        for p in &mut state.persistent.patterns {
            p.completed = true;
        }
        let last = state.persistent.patterns.len() - 1;
        state.persistent.active_pattern = last;
        advance_to_next_pattern(&mut state.persistent);
        assert_eq!(state.persistent.active_pattern, last);
    }
}
