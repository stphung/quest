use super::types::{LoomState, PatternRequirement, Resource, WovenPattern};

pub fn complete_discovery(loom: &mut LoomState) {
    if loom.persistent.discovered {
        return;
    }
    loom.persistent.discovered = true;
    loom.persistent.patterns = create_pattern_sequence();
    // Unlock all 6 extractors immediately (no archetype selection needed).
    super::logic::initialize_loom(loom);
}

fn create_pattern_sequence() -> Vec<WovenPattern> {
    vec![
        // ── Teaching Arc (1-8) ── ~3 days (72 hours) ──
        pattern(0, "First Thread", vec![(Resource::Ember, 25.0, 2.0)]),
        pattern(1, "Still Waters", vec![(Resource::Silence, 25.0, 2.0)]),
        pattern(2, "Echoing Halls", vec![(Resource::Memory, 25.0, 4.0)]),
        pattern(3, "Harmonic Pulse", vec![(Resource::Resonance, 25.0, 4.0)]),
        pattern(
            4,
            "Mirror and Void",
            vec![
                (Resource::Reflection, 30.0, 6.0),
                (Resource::VoidEssence, 30.0, 6.0),
            ],
        ),
        pattern(
            5,
            "Full Circle",
            vec![
                (Resource::Ember, 20.0, 10.0),
                (Resource::Reflection, 20.0, 10.0),
                (Resource::VoidEssence, 20.0, 10.0),
                (Resource::Memory, 20.0, 10.0),
                (Resource::Silence, 20.0, 10.0),
                (Resource::Resonance, 20.0, 10.0),
            ],
        ),
        pattern(
            6,
            "The Catalyst",
            vec![(Resource::CondensedEmber, 8.0, 16.0)],
        ),
        pattern(7, "Echo of Flame", vec![(Resource::EmberEcho, 8.0, 28.0)]),
        // ── Mastery Arc (9-16) ── ~10 days (236 hours) ──
        pattern(
            8,
            "Forged in Fire",
            vec![(Resource::ForgedLight, 15.0, 16.0)],
        ),
        pattern(
            9,
            "Glass Resonance",
            vec![(Resource::EchoGlass, 15.0, 16.0)],
        ),
        pattern(
            10,
            "The Unsung",
            vec![(Resource::StillbornSong, 15.0, 24.0)],
        ),
        pattern(
            11,
            "Void Distillation",
            vec![(Resource::PurifiedVoid, 10.0, 24.0)],
        ),
        pattern(
            12,
            "Crossed Streams",
            vec![
                (Resource::ForgedLight, 12.0, 24.0),
                (Resource::EchoGlass, 12.0, 24.0),
            ],
        ),
        pattern(
            13,
            "The Asymmetry",
            vec![
                (Resource::ForgedLight, 25.0, 36.0),
                (Resource::StillbornSong, 8.0, 36.0),
            ],
        ),
        pattern(
            14,
            "Pressure Test",
            vec![
                (Resource::CondensedEmber, 15.0, 36.0),
                (Resource::EmberEcho, 10.0, 36.0),
                (Resource::PurifiedVoid, 10.0, 36.0),
            ],
        ),
        pattern(
            15,
            "Three Confluences",
            vec![
                (Resource::ForgedLight, 18.0, 60.0),
                (Resource::EchoGlass, 18.0, 60.0),
                (Resource::StillbornSong, 18.0, 60.0),
            ],
        ),
        // ── Endgame Arc (17-28) ── ~22 days (534 hours) ──
        pattern(
            16,
            "The Amplifier",
            vec![(Resource::ForgedLight, 35.0, 18.0)],
        ),
        pattern(
            17,
            "Purified Cascade",
            vec![
                (Resource::PurifiedVoid, 20.0, 24.0),
                (Resource::ForgedLight, 20.0, 24.0),
            ],
        ),
        pattern(
            18,
            "Resonance Cascade",
            vec![
                (Resource::Resonance, 150.0, 24.0),
                (Resource::StillbornSong, 25.0, 24.0),
            ],
        ),
        pattern(19, "First Weave", vec![(Resource::WovenReality, 5.0, 30.0)]),
        pattern(
            20,
            "The Unraveling",
            vec![
                (Resource::WovenReality, 15.0, 36.0),
                (Resource::PurifiedVoid, 15.0, 36.0),
            ],
        ),
        pattern(
            21,
            "Grand Harmony",
            vec![
                (Resource::Ember, 100.0, 36.0),
                (Resource::Reflection, 100.0, 36.0),
                (Resource::VoidEssence, 100.0, 36.0),
                (Resource::Memory, 100.0, 36.0),
                (Resource::Silence, 100.0, 36.0),
                (Resource::Resonance, 100.0, 36.0),
                (Resource::ForgedLight, 30.0, 36.0),
                (Resource::EchoGlass, 30.0, 36.0),
                (Resource::StillbornSong, 30.0, 36.0),
            ],
        ),
        pattern(
            22,
            "The Knot",
            vec![
                (Resource::ForgedLight, 25.0, 36.0),
                (Resource::PurifiedVoid, 15.0, 36.0),
                (Resource::CondensedEmber, 12.0, 36.0),
            ],
        ),
        pattern(
            23,
            "Strange Alchemy",
            vec![
                (Resource::ForgedLight, 30.0, 42.0),
                (Resource::EchoGlass, 30.0, 42.0),
                (Resource::StillbornSong, 30.0, 42.0),
                (Resource::Ember, 80.0, 42.0),
                (Resource::VoidEssence, 80.0, 42.0),
            ],
        ),
        pattern(
            24,
            "Refined Purpose",
            vec![
                (Resource::PurifiedVoid, 30.0, 48.0),
                (Resource::ForgedLight, 25.0, 48.0),
            ],
        ),
        pattern(25, "The Flood", vec![(Resource::WovenReality, 35.0, 48.0)]),
        pattern(
            26,
            "Everything Flows",
            vec![
                (Resource::Ember, 50.0, 72.0),
                (Resource::Reflection, 50.0, 72.0),
                (Resource::VoidEssence, 50.0, 72.0),
                (Resource::Memory, 50.0, 72.0),
                (Resource::Silence, 50.0, 72.0),
                (Resource::Resonance, 50.0, 72.0),
                (Resource::ForgedLight, 20.0, 72.0),
                (Resource::EchoGlass, 20.0, 72.0),
                (Resource::StillbornSong, 20.0, 72.0),
                (Resource::CondensedEmber, 10.0, 72.0),
                (Resource::EmberEcho, 10.0, 72.0),
                (Resource::PurifiedVoid, 10.0, 72.0),
                (Resource::WovenReality, 5.0, 72.0),
            ],
        ),
        pattern(
            27,
            "Mended Loom",
            vec![
                (Resource::WovenReality, 20.0, 120.0),
                (Resource::ForgedLight, 40.0, 120.0),
                (Resource::EchoGlass, 40.0, 120.0),
                (Resource::StillbornSong, 40.0, 120.0),
                (Resource::Ember, 80.0, 120.0),
                (Resource::Silence, 80.0, 120.0),
                (Resource::Resonance, 80.0, 120.0),
            ],
        ),
    ]
}

fn pattern(index: u32, name: &str, reqs: Vec<(Resource, f64, f64)>) -> WovenPattern {
    WovenPattern {
        index,
        name: name.to_string(),
        requirements: reqs
            .into_iter()
            .map(|(resource, rate, duration_hours)| PatternRequirement {
                resource,
                required_rate: rate,
                sustain_duration_secs: duration_hours * 3600.0,
                sustained_secs: 0.0,
                completed: false,
                amount: 0.0,
                accumulated: 0.0,
            })
            .collect(),
        completed: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loom_discovery() {
        let mut loom = LoomState::new();
        assert!(!loom.persistent.discovered);

        complete_discovery(&mut loom);

        assert!(loom.persistent.discovered);
        assert_eq!(loom.persistent.patterns.len(), 28);
    }

    #[test]
    fn test_discovery_pattern_indices_are_sequential() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);

        for (i, pattern) in loom.persistent.patterns.iter().enumerate() {
            assert_eq!(
                pattern.index as usize, i,
                "Pattern at position {i} has wrong index"
            );
        }
    }

    #[test]
    fn test_discovery_all_patterns_have_requirements() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);

        for pattern in &loom.persistent.patterns {
            assert!(
                !pattern.requirements.is_empty(),
                "Pattern '{}' has no requirements",
                pattern.name
            );
        }
    }

    #[test]
    fn test_discovery_does_not_re_discover() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);
        assert!(loom.persistent.discovered);
        assert_eq!(loom.persistent.patterns.len(), 28);

        // Mark first pattern as having sustain progress.
        loom.persistent.patterns[0].requirements[0].sustained_secs = 0.5;

        // Calling again should be a no-op (re-entry guard).
        complete_discovery(&mut loom);
        assert_eq!(loom.persistent.patterns.len(), 28);
        assert!(
            (loom.persistent.patterns[0].requirements[0].sustained_secs - 0.5).abs() < 1e-9,
            "pattern progress must be preserved on re-call"
        );
    }

    // ── pattern initial state ─────────────────────────────────────────────────

    #[test]
    fn test_all_patterns_start_uncompleted() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);

        for pattern in &loom.persistent.patterns {
            assert!(
                !pattern.completed,
                "pattern '{}' should start uncompleted",
                pattern.name
            );
        }
    }

    #[test]
    fn test_all_patterns_start_with_zero_sustained() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);

        for pattern in &loom.persistent.patterns {
            for req in &pattern.requirements {
                assert!(
                    req.sustained_secs.abs() < 1e-9,
                    "pattern '{}' req {:?} should start with 0 sustained_secs",
                    pattern.name,
                    req.resource
                );
            }
        }
    }

    // ── pattern rate and duration checks ─────────────────────────────────────

    #[test]
    fn test_first_pattern_requires_ember_at_25_per_hour() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);
        let first = &loom.persistent.patterns[0];
        assert_eq!(first.requirements.len(), 1);
        assert_eq!(first.requirements[0].resource, Resource::Ember);
        assert!((first.requirements[0].required_rate - 25.0).abs() < 1e-9);
        assert!((first.requirements[0].sustain_duration_secs - 7200.0).abs() < 1e-9);
    }

    #[test]
    fn test_all_required_rates_are_positive() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);

        for pattern in &loom.persistent.patterns {
            for req in &pattern.requirements {
                assert!(
                    req.required_rate > 0.0,
                    "pattern '{}' has non-positive required_rate for {:?}: {}",
                    pattern.name,
                    req.resource,
                    req.required_rate
                );
            }
        }
    }

    #[test]
    fn test_final_pattern_has_longest_duration() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);
        let last = loom.persistent.patterns.last().unwrap();
        let last_duration = last.requirements[0].sustain_duration_secs;
        assert!((last_duration - 432_000.0).abs() < 1e-9); // 120 hours
    }

    // ── pattern name spot checks ──────────────────────────────────────────────

    #[test]
    fn test_first_pattern_name_is_first_thread() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);
        assert_eq!(loom.persistent.patterns[0].name, "First Thread");
    }

    #[test]
    fn test_last_pattern_name_is_mended_loom() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);
        let last = loom.persistent.patterns.last().unwrap();
        assert_eq!(last.name, "Mended Loom");
    }

    #[test]
    fn test_all_pattern_names_are_non_empty() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);

        for pattern in &loom.persistent.patterns {
            assert!(
                !pattern.name.is_empty(),
                "pattern at index {} has an empty name",
                pattern.index
            );
        }
    }

    // ── pattern requirements spot checks ─────────────────────────────────────

    #[test]
    fn test_first_pattern_requires_ember() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);
        let first = &loom.persistent.patterns[0];
        assert_eq!(first.requirements.len(), 1);
        assert_eq!(first.requirements[0].resource, Resource::Ember);
        assert!(first.requirements[0].required_rate > 0.0);
    }

    // ── active_pattern initial value ─────────────────────────────────────────

    #[test]
    fn test_active_pattern_starts_at_zero_after_discovery() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);
        assert_eq!(loom.persistent.active_pattern, 0);
    }
}
