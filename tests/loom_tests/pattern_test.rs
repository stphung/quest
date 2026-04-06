//! Pattern sustain, NaN/Infinity guards, simultaneous requirements,
//! zone cap boundaries, and WR→PR conversion.

use super::helpers::*;
use quest::loom::logic::loom_zone_cap_for_patterns;
use quest::loom::patterns::{active_pattern_requirements_met, all_patterns_complete};
use quest::loom::types::*;
use std::collections::HashMap;

// ── NaN and Infinity guards ──────────────────────────────────────────────────

#[test]
fn test_pattern_sustain_nan_rate_does_not_advance() {
    let mut loom = setup_loom();
    loom.persistent.active_pattern = 0;

    let mut rates = HashMap::new();
    rates.insert(Resource::Ember, f64::NAN);

    let sustained_before = loom.persistent.patterns[0].requirements[0].sustained_secs;
    quest::loom::patterns::tick_pattern_sustain(&mut loom.persistent, &rates, 10.0);
    let sustained_after = loom.persistent.patterns[0].requirements[0].sustained_secs;

    assert!(
        (sustained_after - sustained_before).abs() < 0.001,
        "NaN rate should not advance pattern sustain timer"
    );
}

#[test]
fn test_pattern_sustain_infinity_rate_does_not_advance() {
    let mut loom = setup_loom();
    loom.persistent.active_pattern = 0;

    let mut rates = HashMap::new();
    rates.insert(Resource::Ember, f64::INFINITY);

    let sustained_before = loom.persistent.patterns[0].requirements[0].sustained_secs;
    quest::loom::patterns::tick_pattern_sustain(&mut loom.persistent, &rates, 10.0);
    let sustained_after = loom.persistent.patterns[0].requirements[0].sustained_secs;

    assert!(
        (sustained_after - sustained_before).abs() < 0.001,
        "Infinite rate should not advance pattern sustain timer"
    );
}

#[test]
fn test_pattern_sustain_negative_delta_is_noop() {
    let mut loom = setup_loom();
    loom.persistent.active_pattern = 0;

    let mut rates = HashMap::new();
    rates.insert(Resource::Ember, 100.0);

    let sustained_before = loom.persistent.patterns[0].requirements[0].sustained_secs;
    quest::loom::patterns::tick_pattern_sustain(&mut loom.persistent, &rates, -5.0);
    let sustained_after = loom.persistent.patterns[0].requirements[0].sustained_secs;

    assert!(
        (sustained_after - sustained_before).abs() < 0.001,
        "Negative delta should not change sustain timer"
    );
}

// ── Simultaneous requirements ────────────────────────────────────────────────

#[test]
fn test_simultaneous_requirements_no_advance_when_one_drops() {
    let mut loom = setup_loom();

    loom.persistent.patterns[0] = WovenPattern {
        index: 0,
        name: "Test Pattern".to_string(),
        requirements: vec![
            PatternRequirement {
                resource: Resource::Ember,
                required_rate: 25.0,
                sustain_duration_secs: 100.0,
                sustained_secs: 50.0,
                completed: false,
                amount: 0.0,
                accumulated: 0.0,
            },
            PatternRequirement {
                resource: Resource::VoidEssence,
                required_rate: 25.0,
                sustain_duration_secs: 100.0,
                sustained_secs: 50.0,
                completed: false,
                amount: 0.0,
                accumulated: 0.0,
            },
        ],
        completed: false,
        flavor: String::new(),
    };
    loom.persistent.active_pattern = 0;

    let mut rates = HashMap::new();
    rates.insert(Resource::Ember, 30.0);
    rates.insert(Resource::VoidEssence, 0.0);

    quest::loom::patterns::tick_pattern_sustain(&mut loom.persistent, &rates, 1.0);

    let req_ember = &loom.persistent.patterns[0].requirements[0];
    let req_void = &loom.persistent.patterns[0].requirements[1];
    assert!(
        (req_ember.sustained_secs - 50.0).abs() < 0.01,
        "Ember requirement should not advance when VoidEssence is below threshold"
    );
    assert!(
        (req_void.sustained_secs - 50.0).abs() < 0.01,
        "VoidEssence requirement should not advance"
    );
}

// ── Zone cap boundaries ──────────────────────────────────────────────────────

#[test]
fn test_loom_zone_cap_boundaries_exact() {
    // Verify every boundary transition (>=4→34, >=8→38, >=16→42, >=22→46, >=28→50)
    assert_eq!(loom_zone_cap_for_patterns(0), 0);
    assert_eq!(loom_zone_cap_for_patterns(3), 0);
    assert_eq!(loom_zone_cap_for_patterns(4), 34); // 3→4 transition
    assert_eq!(loom_zone_cap_for_patterns(7), 34);
    assert_eq!(loom_zone_cap_for_patterns(8), 38); // 7→8 transition
    assert_eq!(loom_zone_cap_for_patterns(14), 38); // still in >=8 tier
    assert_eq!(loom_zone_cap_for_patterns(15), 38); // still in >=8 tier
    assert_eq!(loom_zone_cap_for_patterns(16), 42); // 15→16 transition
    assert_eq!(loom_zone_cap_for_patterns(21), 42); // still in >=16 tier
    assert_eq!(loom_zone_cap_for_patterns(22), 46); // 21→22 transition
    assert_eq!(loom_zone_cap_for_patterns(27), 46); // still in >=22 tier
    assert_eq!(loom_zone_cap_for_patterns(28), 50); // 27→28 transition
    assert_eq!(loom_zone_cap_for_patterns(100), 50); // beyond max
}

// ── all_patterns_complete ────────────────────────────────────────────────────

#[test]
fn test_all_patterns_complete_empty_is_false() {
    let persistent = LoomPersistent::default();
    assert!(persistent.patterns.is_empty());
    assert!(!all_patterns_complete(&persistent));
}

#[test]
fn test_all_patterns_complete_partial_is_false() {
    let mut loom = setup_loom(); // Has 28 patterns
    complete_n_patterns(&mut loom, 27);
    assert!(!all_patterns_complete(&loom.persistent));
}

#[test]
fn test_all_patterns_complete_all_28_is_true() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 28);
    assert!(all_patterns_complete(&loom.persistent));
}

// ── active_pattern_requirements_met ──────────────────────────────────────────

#[test]
fn test_active_pattern_requirements_met_out_of_bounds() {
    let mut loom = setup_loom();
    loom.persistent.active_pattern = 999;
    assert!(!active_pattern_requirements_met(&loom.persistent));
}

#[test]
fn test_active_pattern_requirements_met_already_completed() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);
    loom.persistent.active_pattern = 0; // Pattern 0 is completed
    assert!(
        !active_pattern_requirements_met(&loom.persistent),
        "Already completed pattern should return false"
    );
}

#[test]
fn test_active_pattern_requirements_met_partial() {
    let mut loom = setup_loom();
    loom.persistent.active_pattern = 0;
    // First pattern has 1 requirement — leave it incomplete
    assert!(!active_pattern_requirements_met(&loom.persistent));
}

#[test]
fn test_active_pattern_requirements_met_all_reqs_done() {
    let mut loom = setup_loom();
    loom.persistent.active_pattern = 0;
    // Mark all requirements as completed but NOT the pattern itself
    for req in &mut loom.persistent.patterns[0].requirements {
        req.completed = true;
    }
    assert!(
        !loom.persistent.patterns[0].completed,
        "Pattern should not be marked complete yet"
    );
    assert!(active_pattern_requirements_met(&loom.persistent));
}
