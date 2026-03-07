//! Woven Pattern tracking, completion, and progression.
//!
//! Patterns are the sole progression gate in the Loom of Worlds.
//! Each tick, if all required production rates are being sustained,
//! the sustain timer increments. When the timer reaches the required
//! duration, the pattern completes and the next pattern unlocks.
#![allow(dead_code)]

use super::types::{LoomPersistent, Resource};
use std::collections::HashMap;

// ── Rate checking ──────────────────────────────────────────────────────────────

/// Check if all requirements of the active pattern are met given current rates.
///
/// Returns `true` if every required resource is meeting or exceeding its rate,
/// `false` if any rate is deficient or if there is no active pattern.
pub fn active_pattern_requirements_met(
    persistent: &LoomPersistent,
    rates: &HashMap<Resource, f64>,
) -> bool {
    let Some(pattern) = persistent.patterns.get(persistent.active_pattern) else {
        return false;
    };
    if pattern.completed {
        return false;
    }
    pattern.requirements.iter().all(|req| {
        let current = rates.get(&req.resource).copied().unwrap_or(0.0);
        current >= req.rate_per_hour
    })
}

// ── Sustain timer ──────────────────────────────────────────────────────────────

/// Tick the sustain timer for the active pattern.
///
/// Called once per game tick. `delta_seconds` is the wall-clock time elapsed
/// since the last tick (typically 0.1s for a 100ms tick interval).
/// `rates` maps each resource to its current production rate in units/hour.
///
/// - If all requirements are met: increments `sustained_seconds` by `delta_seconds`.
/// - If any requirement is unmet: pauses (does not reset).
/// - When `sustained_seconds` reaches `sustain_seconds`: completes the pattern.
///
/// Returns `true` if a pattern was completed during this tick (so the caller
/// can emit a `TickEvent` and set `loom_changed = true`).
pub fn tick_pattern_sustain(
    persistent: &mut LoomPersistent,
    rates: &HashMap<Resource, f64>,
    delta_seconds: f64,
) -> bool {
    if !active_pattern_requirements_met(persistent, rates) {
        return false;
    }

    let Some(pattern) = persistent.patterns.get_mut(persistent.active_pattern) else {
        return false;
    };

    // Accumulate fractional seconds so sub-second ticks (0.1s each) sum correctly.
    // Use a tiny epsilon when flooring to avoid IEEE 754 issues where 10 × 0.1
    // accumulates to 0.9999... instead of 1.0.
    const EPSILON: f64 = 1e-9;
    pattern.sustained_seconds_frac += delta_seconds;
    let carry = (pattern.sustained_seconds_frac + EPSILON).floor() as u32;
    if carry > 0 {
        pattern.sustained_seconds_frac = (pattern.sustained_seconds_frac - carry as f64).max(0.0);
        pattern.sustained_seconds =
            (pattern.sustained_seconds + carry).min(pattern.sustain_seconds);
    }

    if pattern.sustained_seconds >= pattern.sustain_seconds {
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

/// Returns a per-requirement met/unmet status for the active pattern.
/// Useful for the pattern bar UI to render individual checkmarks.
pub fn active_pattern_requirement_status(
    persistent: &LoomPersistent,
    rates: &HashMap<Resource, f64>,
) -> Vec<bool> {
    let Some(pattern) = persistent.patterns.get(persistent.active_pattern) else {
        return Vec::new();
    };
    pattern
        .requirements
        .iter()
        .map(|req| {
            let current = rates.get(&req.resource).copied().unwrap_or(0.0);
            current >= req.rate_per_hour
        })
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
    fn test_requirements_met_when_all_rates_sufficient() {
        let state = state_with_patterns();
        // Pattern 0 "First Thread": Ember 2/hr
        let r = rates(&[(Resource::Ember, 2.0)]);
        assert!(active_pattern_requirements_met(&state.persistent, &r));
    }

    #[test]
    fn test_requirements_not_met_when_rate_too_low() {
        let state = state_with_patterns();
        // Pattern 0: needs Ember 2/hr; provide 1/hr
        let r = rates(&[(Resource::Ember, 1.0)]);
        assert!(!active_pattern_requirements_met(&state.persistent, &r));
    }

    #[test]
    fn test_requirements_not_met_when_resource_absent() {
        let state = state_with_patterns();
        // Pattern 0: needs Ember 2/hr; provide nothing
        let r = rates(&[]);
        assert!(!active_pattern_requirements_met(&state.persistent, &r));
    }

    #[test]
    fn test_requirements_met_exactly_at_threshold() {
        let state = state_with_patterns();
        // Pattern 0: needs Ember 2.0/hr; provide exactly 2.0/hr
        let r = rates(&[(Resource::Ember, 2.0)]);
        assert!(active_pattern_requirements_met(&state.persistent, &r));
    }

    #[test]
    fn test_requirements_not_met_when_pattern_already_completed() {
        let mut state = state_with_patterns();
        // Manually mark pattern 0 as completed without advancing active_pattern.
        state.persistent.patterns[0].completed = true;
        let r = rates(&[(Resource::Ember, 5.0)]);
        // active_pattern is still 0, which is completed → should return false.
        assert!(!active_pattern_requirements_met(&state.persistent, &r));
    }

    // ── tick_pattern_sustain ──────────────────────────────────────────────────

    #[test]
    fn test_sustain_timer_increments_when_rates_met() {
        let mut state = state_with_patterns();
        let r = rates(&[(Resource::Ember, 3.0)]); // above 2/hr threshold

        tick_pattern_sustain(&mut state.persistent, &r, 1.0);

        assert_eq!(state.persistent.patterns[0].sustained_seconds, 1);
    }

    #[test]
    fn test_sustain_timer_pauses_when_rates_unmet() {
        let mut state = state_with_patterns();
        state.persistent.patterns[0].sustained_seconds = 10;

        let r = rates(&[(Resource::Ember, 0.5)]); // below threshold
        tick_pattern_sustain(&mut state.persistent, &r, 1.0);

        // Timer should not change.
        assert_eq!(state.persistent.patterns[0].sustained_seconds, 10);
    }

    #[test]
    fn test_sustain_timer_does_not_reset_when_rates_recover() {
        let mut state = state_with_patterns();
        state.persistent.patterns[0].sustained_seconds = 100;

        let r = rates(&[(Resource::Ember, 3.0)]);
        tick_pattern_sustain(&mut state.persistent, &r, 1.0);

        // Should have continued from 100, not reset.
        assert_eq!(state.persistent.patterns[0].sustained_seconds, 101);
    }

    #[test]
    fn test_pattern_completes_when_sustain_reached() {
        let mut state = state_with_patterns();
        // Set timer just below the threshold (pattern 0 sustain = 1800s).
        state.persistent.patterns[0].sustained_seconds = 1799;

        let r = rates(&[(Resource::Ember, 3.0)]);
        let completed = tick_pattern_sustain(&mut state.persistent, &r, 1.0);

        assert!(completed);
        assert!(state.persistent.patterns[0].completed);
    }

    #[test]
    fn test_active_pattern_advances_after_completion() {
        let mut state = state_with_patterns();
        state.persistent.patterns[0].sustained_seconds = 1799;

        let r = rates(&[(Resource::Ember, 3.0)]);
        tick_pattern_sustain(&mut state.persistent, &r, 1.0);

        // active_pattern should now point to pattern 1.
        assert_eq!(state.persistent.active_pattern, 1);
    }

    #[test]
    fn test_no_completion_returned_when_rates_unmet() {
        let mut state = state_with_patterns();
        state.persistent.patterns[0].sustained_seconds = 1799;

        let r = rates(&[(Resource::Ember, 0.0)]); // unmet
        let completed = tick_pattern_sustain(&mut state.persistent, &r, 1.0);

        assert!(!completed);
        assert!(!state.persistent.patterns[0].completed);
    }

    #[test]
    fn test_sub_second_ticks_accumulate_correctly() {
        let mut state = state_with_patterns();
        let r = rates(&[(Resource::Ember, 3.0)]);

        // 10 ticks of 0.1s each should sum to 1 second.
        for _ in 0..10 {
            tick_pattern_sustain(&mut state.persistent, &r, 0.1);
        }

        assert_eq!(state.persistent.patterns[0].sustained_seconds, 1);
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

    // ── active_pattern_requirement_status ────────────────────────────────────

    #[test]
    fn test_requirement_status_all_met() {
        let state = state_with_patterns();
        let r = rates(&[(Resource::Ember, 5.0)]);
        let status = active_pattern_requirement_status(&state.persistent, &r);
        assert_eq!(status, vec![true]);
    }

    #[test]
    fn test_requirement_status_partial() {
        let mut state = state_with_patterns();
        // Advance to pattern 1 "The Bridge": Ember 3/hr + Reflection 1/hr
        state.persistent.patterns[0].completed = true;
        state.persistent.active_pattern = 1;

        let r = rates(&[(Resource::Ember, 5.0), (Resource::Reflection, 0.5)]);
        let status = active_pattern_requirement_status(&state.persistent, &r);
        assert_eq!(status, vec![true, false]);
    }

    #[test]
    fn test_requirement_status_empty_when_no_patterns() {
        let state = LoomState::new();
        let r = rates(&[]);
        let status = active_pattern_requirement_status(&state.persistent, &r);
        assert!(status.is_empty());
    }

    // ── advance_to_next_pattern ───────────────────────────────────────────────

    #[test]
    fn test_advance_skips_already_completed_patterns() {
        let mut state = state_with_patterns();
        // Complete patterns 0 and 1 manually.
        state.persistent.patterns[0].completed = true;
        state.persistent.patterns[1].completed = true;
        state.persistent.active_pattern = 0;

        advance_to_next_pattern(&mut state.persistent);

        // Should skip to pattern 2.
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

        // No uncompleted pattern exists; active_pattern should remain at last.
        assert_eq!(state.persistent.active_pattern, last);
    }

    // ── multi-requirement patterns ────────────────────────────────────────────

    #[test]
    fn test_requirements_met_with_multiple_resources_all_satisfied() {
        let mut state = state_with_patterns();
        // Advance to pattern 3 "Balancing Act": Ember 2/hr + Reflection 2/hr + VoidEssence 2/hr.
        state.persistent.patterns[0].completed = true;
        state.persistent.patterns[1].completed = true;
        state.persistent.patterns[2].completed = true;
        state.persistent.active_pattern = 3;

        let r = rates(&[
            (Resource::Ember, 2.0),
            (Resource::Reflection, 2.0),
            (Resource::VoidEssence, 2.0),
        ]);
        assert!(active_pattern_requirements_met(&state.persistent, &r));
    }

    #[test]
    fn test_requirements_not_met_when_one_of_multiple_resources_missing() {
        let mut state = state_with_patterns();
        // Pattern 3 "Balancing Act": requires Ember, Reflection, VoidEssence all at 2/hr.
        state.persistent.patterns[0].completed = true;
        state.persistent.patterns[1].completed = true;
        state.persistent.patterns[2].completed = true;
        state.persistent.active_pattern = 3;

        // Provide only two of the three required resources.
        let r = rates(&[(Resource::Ember, 2.0), (Resource::Reflection, 2.0)]);
        assert!(!active_pattern_requirements_met(&state.persistent, &r));
    }

    // ── tick_pattern_sustain edge cases ───────────────────────────────────────

    #[test]
    fn test_sustain_timer_does_not_complete_before_threshold() {
        let mut state = state_with_patterns();
        // Pattern 0 sustain = 1800s. After 1799 ticks of 1s each, should not complete.
        let r = rates(&[(Resource::Ember, 3.0)]);
        let mut completed = false;
        for _ in 0..1799 {
            completed = tick_pattern_sustain(&mut state.persistent, &r, 1.0);
            if completed {
                break;
            }
        }
        assert!(!completed);
        assert_eq!(state.persistent.patterns[0].sustained_seconds, 1799);
        assert!(!state.persistent.patterns[0].completed);
    }

    #[test]
    fn test_sustain_timer_capped_at_sustain_seconds() {
        let mut state = state_with_patterns();
        // Set timer just one tick below completion.
        state.persistent.patterns[0].sustained_seconds = 1799;

        let r = rates(&[(Resource::Ember, 3.0)]);
        tick_pattern_sustain(&mut state.persistent, &r, 1.0);

        // sustained_seconds should be capped at sustain_seconds (1800), not exceed it.
        assert!(state.persistent.patterns[0].sustained_seconds <= 1800);
    }

    // ── all_patterns_complete edge cases ──────────────────────────────────────

    #[test]
    fn test_all_complete_returns_false_with_one_remaining() {
        let mut state = state_with_patterns();
        let n = state.persistent.patterns.len();
        // Complete all except the last.
        for p in state.persistent.patterns.iter_mut().take(n - 1) {
            p.completed = true;
        }
        assert!(!all_patterns_complete(&state.persistent));
    }

    // ── active_pattern_requirement_status ─────────────────────────────────────

    #[test]
    fn test_requirement_status_all_unmet() {
        let state = state_with_patterns();
        // Pattern 0 needs Ember 2/hr; provide 0.
        let r = rates(&[]);
        let status = active_pattern_requirement_status(&state.persistent, &r);
        assert_eq!(status, vec![false]);
    }

    #[test]
    fn test_requirement_status_length_matches_requirement_count() {
        let mut state = state_with_patterns();
        // Advance to pattern 4 "Full Circle" which has 6 requirements.
        for i in 0..4 {
            state.persistent.patterns[i].completed = true;
        }
        state.persistent.active_pattern = 4;

        let r = rates(&[]);
        let status = active_pattern_requirement_status(&state.persistent, &r);
        // Pattern 4 has 6 requirements (all six base resources).
        assert_eq!(status.len(), 6);
    }

    // ── tick_pattern_sustain with zero delta ──────────────────────────────────

    #[test]
    fn test_sustain_timer_no_progress_on_zero_delta() {
        let mut state = state_with_patterns();
        let r = rates(&[(Resource::Ember, 3.0)]);
        tick_pattern_sustain(&mut state.persistent, &r, 0.0);
        assert_eq!(state.persistent.patterns[0].sustained_seconds, 0);
        assert!((state.persistent.patterns[0].sustained_seconds_frac).abs() < 1e-9);
    }
}
