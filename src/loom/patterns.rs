//! Woven Pattern tracking, completion, and progression.
//!
//! Patterns are the sole progression gate in the Loom of Worlds.
//! Each tick, production rates accumulate toward per-resource amount targets.
//! When every requirement's accumulated value reaches its target amount,
//! the pattern completes and the next pattern unlocks.
#![allow(dead_code)]

use super::types::{LoomPersistent, Resource};
use std::collections::HashMap;

// ── Completion check ───────────────────────────────────────────────────────────

/// Check if all requirements of the active pattern have been fully accumulated.
///
/// Returns `true` if every required resource has `accumulated >= amount`,
/// `false` if any requirement is deficient or if there is no active pattern.
pub fn active_pattern_requirements_met(persistent: &LoomPersistent) -> bool {
    let Some(pattern) = persistent.patterns.get(persistent.active_pattern) else {
        return false;
    };
    if pattern.completed {
        return false;
    }
    pattern
        .requirements
        .iter()
        .all(|req| req.accumulated >= req.amount)
}

// ── Accumulator tick ───────────────────────────────────────────────────────────

/// Tick the accumulator for the active pattern.
///
/// Called once per game tick. `delta_seconds` is the wall-clock time elapsed
/// since the last tick (typically 0.1s for a 100ms tick interval).
/// `rates` maps each resource to its current production rate in units/hour.
///
/// Each requirement accumulates `rate * delta_hours` up to its `amount` cap.
/// When every requirement is fully accumulated, the pattern completes.
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

    let delta_hours = delta_seconds / 3600.0;
    for req in &mut pattern.requirements {
        let rate = rates.get(&req.resource).copied().unwrap_or(0.0);
        req.accumulated = (req.accumulated + rate * delta_hours).min(req.amount);
    }

    if pattern
        .requirements
        .iter()
        .all(|req| req.accumulated >= req.amount)
    {
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

/// Returns `(accumulated, amount)` for each requirement of the active pattern.
/// Useful for the pattern bar UI to render individual progress indicators.
pub fn active_pattern_requirement_status(persistent: &LoomPersistent) -> Vec<(f64, f64)> {
    let Some(pattern) = persistent.patterns.get(persistent.active_pattern) else {
        return Vec::new();
    };
    pattern
        .requirements
        .iter()
        .map(|req| (req.accumulated, req.amount))
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
        state
    }

    fn rates(pairs: &[(Resource, f64)]) -> HashMap<Resource, f64> {
        pairs.iter().cloned().collect()
    }

    // ── active_pattern_requirements_met ───────────────────────────────────────

    #[test]
    fn test_requirements_met_when_fully_accumulated() {
        let mut state = state_with_patterns();
        // Pattern 0 "First Thread": Ember amount = 1.0; set accumulated to exactly 1.0.
        state.persistent.patterns[0].requirements[0].accumulated = 1.0;
        assert!(active_pattern_requirements_met(&state.persistent));
    }

    #[test]
    fn test_requirements_not_met_when_partially_accumulated() {
        let state = state_with_patterns();
        // Pattern 0 starts with 0 accumulated; amount = 1.0 → not met.
        assert!(!active_pattern_requirements_met(&state.persistent));
    }

    #[test]
    fn test_requirements_not_met_when_pattern_already_completed() {
        let mut state = state_with_patterns();
        // Fully accumulate then mark as completed.
        state.persistent.patterns[0].requirements[0].accumulated = 1.0;
        state.persistent.patterns[0].completed = true;
        // active_pattern is still 0, which is completed → should return false.
        assert!(!active_pattern_requirements_met(&state.persistent));
    }

    // ── tick_pattern_sustain ──────────────────────────────────────────────────

    #[test]
    fn test_accumulator_increases_when_rate_provided() {
        let mut state = state_with_patterns();
        // Pattern 0 "First Thread": Ember amount = 1.0.
        // At 2.0/hr for 0.1s (tick), delta_hours = 0.1/3600 ≈ 2.778e-5 hr.
        // accumulated += 2.0 * 2.778e-5 ≈ 5.556e-5.
        let r = rates(&[(Resource::Ember, 2.0)]);
        tick_pattern_sustain(&mut state.persistent, &r, 0.1);
        let acc = state.persistent.patterns[0].requirements[0].accumulated;
        assert!(acc > 0.0, "accumulated should be positive after a tick");
        assert!(
            acc < 1.0,
            "should not complete in a single 0.1s tick at 2/hr"
        );
    }

    #[test]
    fn test_accumulator_zero_when_no_rate() {
        let mut state = state_with_patterns();
        let r = rates(&[]);
        tick_pattern_sustain(&mut state.persistent, &r, 1.0);
        let acc = state.persistent.patterns[0].requirements[0].accumulated;
        assert!((acc).abs() < 1e-9, "no rate means no accumulation");
    }

    #[test]
    fn test_accumulator_does_not_reset() {
        let mut state = state_with_patterns();
        // Pre-set some accumulated progress.
        state.persistent.patterns[0].requirements[0].accumulated = 0.5;
        // Tick with zero rate — should not reset.
        let r = rates(&[]);
        tick_pattern_sustain(&mut state.persistent, &r, 1.0);
        let acc = state.persistent.patterns[0].requirements[0].accumulated;
        assert!((acc - 0.5).abs() < 1e-9, "accumulated must not reset");
    }

    #[test]
    fn test_accumulator_capped_at_amount() {
        let mut state = state_with_patterns();
        // Supply a huge rate to ensure we don't exceed amount = 1.0.
        let r = rates(&[(Resource::Ember, 1_000_000.0)]);
        tick_pattern_sustain(&mut state.persistent, &r, 3600.0);
        let req = &state.persistent.patterns[0].requirements[0];
        assert!(
            req.accumulated <= req.amount + 1e-9,
            "accumulated must not exceed amount"
        );
    }

    #[test]
    fn test_pattern_completes_when_fully_accumulated() {
        let mut state = state_with_patterns();
        // Set accumulated just below amount, then push it over.
        state.persistent.patterns[0].requirements[0].accumulated = 0.9999;
        let r = rates(&[(Resource::Ember, 1_000_000.0)]);
        let completed = tick_pattern_sustain(&mut state.persistent, &r, 3600.0);
        assert!(completed);
        assert!(state.persistent.patterns[0].completed);
    }

    #[test]
    fn test_active_pattern_advances_after_completion() {
        let mut state = state_with_patterns();
        state.persistent.patterns[0].requirements[0].accumulated = 0.9999;
        let r = rates(&[(Resource::Ember, 1_000_000.0)]);
        tick_pattern_sustain(&mut state.persistent, &r, 3600.0);
        assert_eq!(state.persistent.active_pattern, 1);
    }

    #[test]
    fn test_no_completion_when_accumulation_insufficient() {
        let mut state = state_with_patterns();
        // Pattern 0 amount = 1.0. At 0.001/hr for 1s we accumulate ~2.78e-7.
        let r = rates(&[(Resource::Ember, 0.001)]);
        let completed = tick_pattern_sustain(&mut state.persistent, &r, 1.0);
        assert!(!completed);
        assert!(!state.persistent.patterns[0].completed);
    }

    #[test]
    fn test_sub_second_ticks_accumulate_correctly() {
        let mut state = state_with_patterns();
        // 36_000 ticks of 0.1s each = 3600s = 1 hour.
        // At 2.0/hr, should accumulate 2.0 total (capped at amount = 1.0 so pattern completes).
        let r = rates(&[(Resource::Ember, 2.0)]);
        let mut completed = false;
        for _ in 0..36_000 {
            if tick_pattern_sustain(&mut state.persistent, &r, 0.1) {
                completed = true;
                break;
            }
        }
        assert!(
            completed,
            "pattern should complete after 1hr at 2/hr for amount=1.0"
        );
    }

    #[test]
    fn test_no_progress_on_zero_delta() {
        let mut state = state_with_patterns();
        let r = rates(&[(Resource::Ember, 3.0)]);
        tick_pattern_sustain(&mut state.persistent, &r, 0.0);
        let acc = state.persistent.patterns[0].requirements[0].accumulated;
        assert!((acc).abs() < 1e-9);
    }

    // ── all_patterns_complete ─────────────────────────────────────────────────

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

    #[test]
    fn test_not_all_complete_when_patterns_empty() {
        let state = LoomState::new();
        assert!(!all_patterns_complete(&state.persistent));
    }

    #[test]
    fn test_all_complete_returns_false_with_one_remaining() {
        let mut state = state_with_patterns();
        let n = state.persistent.patterns.len();
        for p in state.persistent.patterns.iter_mut().take(n - 1) {
            p.completed = true;
        }
        assert!(!all_patterns_complete(&state.persistent));
    }

    // ── active_pattern_requirement_status ────────────────────────────────────

    #[test]
    fn test_requirement_status_returns_accumulated_and_amount() {
        let mut state = state_with_patterns();
        // Set accumulated to half of amount.
        state.persistent.patterns[0].requirements[0].accumulated = 0.5;
        let status = active_pattern_requirement_status(&state.persistent);
        assert_eq!(status.len(), 1);
        let (acc, amt) = status[0];
        assert!((acc - 0.5).abs() < 1e-9);
        assert!((amt - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_requirement_status_empty_when_no_patterns() {
        let state = LoomState::new();
        let status = active_pattern_requirement_status(&state.persistent);
        assert!(status.is_empty());
    }

    #[test]
    fn test_requirement_status_length_matches_requirement_count() {
        let mut state = state_with_patterns();
        // Advance to pattern 4 "Full Circle" which has 6 requirements.
        for i in 0..4 {
            state.persistent.patterns[i].completed = true;
        }
        state.persistent.active_pattern = 4;
        let status = active_pattern_requirement_status(&state.persistent);
        assert_eq!(status.len(), 6);
    }

    // ── advance_to_next_pattern ───────────────────────────────────────────────

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

    // ── multi-requirement patterns ────────────────────────────────────────────

    #[test]
    fn test_multi_requirement_all_accumulated_completes() {
        let mut state = state_with_patterns();
        // Advance to pattern 3 "Balancing Act": 3 reqs, each amount = 3.0.
        for i in 0..3 {
            state.persistent.patterns[i].completed = true;
        }
        state.persistent.active_pattern = 3;
        // Fully accumulate all three requirements.
        for req in &mut state.persistent.patterns[3].requirements {
            req.accumulated = req.amount;
        }
        assert!(active_pattern_requirements_met(&state.persistent));
    }

    #[test]
    fn test_multi_requirement_partial_does_not_complete() {
        let mut state = state_with_patterns();
        // Advance to pattern 3 "Balancing Act".
        for i in 0..3 {
            state.persistent.patterns[i].completed = true;
        }
        state.persistent.active_pattern = 3;
        // Accumulate only the first two requirements fully.
        state.persistent.patterns[3].requirements[0].accumulated = 3.0;
        state.persistent.patterns[3].requirements[1].accumulated = 3.0;
        // Third requirement stays at 0.
        assert!(!active_pattern_requirements_met(&state.persistent));
    }
}
