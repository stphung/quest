use super::types::{LoomState, PatternRequirement, Resource, WovenPattern};

pub fn complete_discovery(loom: &mut LoomState) {
    if loom.persistent.discovered {
        return;
    }
    loom.persistent.discovered = true;
    loom.persistent.patterns = create_pattern_sequence();
}

fn create_pattern_sequence() -> Vec<WovenPattern> {
    vec![
        // Teaching Arc (1-6)
        pattern(0, "First Thread", vec![(Resource::Ember, 10.0)]),
        pattern(
            1,
            "The Bridge",
            vec![(Resource::Ember, 30.0), (Resource::Reflection, 10.0)],
        ),
        pattern(
            2,
            "Long Road",
            vec![(Resource::Ember, 20.0), (Resource::Memory, 10.0)],
        ),
        pattern(
            3,
            "Balancing Act",
            vec![
                (Resource::Ember, 30.0),
                (Resource::Reflection, 30.0),
                (Resource::VoidEssence, 30.0),
            ],
        ),
        pattern(
            4,
            "Full Circle",
            vec![
                (Resource::Ember, 20.0),
                (Resource::Reflection, 20.0),
                (Resource::VoidEssence, 20.0),
                (Resource::Memory, 20.0),
                (Resource::Silence, 20.0),
                (Resource::Resonance, 20.0),
            ],
        ),
        pattern(5, "The Catalyst", vec![(Resource::CondensedEmber, 20.0)]),
        // Mastery Arc (7-12)
        pattern(
            6,
            "Crossed Streams",
            vec![
                (Resource::CondensedEmber, 20.0),
                (Resource::EmberEcho, 20.0),
            ],
        ),
        pattern(
            7,
            "The Diversion",
            vec![(Resource::ForgedLight, 25.0), (Resource::Ember, 75.0)],
        ),
        pattern(
            8,
            "Three Confluences",
            vec![
                (Resource::ForgedLight, 30.0),
                (Resource::EchoGlass, 30.0),
                (Resource::StillbornSong, 30.0),
            ],
        ),
        pattern(
            9,
            "Pressure Test",
            vec![(Resource::ForgedLight, 60.0), (Resource::EchoGlass, 60.0)],
        ),
        pattern(10, "The Bottleneck", vec![(Resource::StillbornSong, 90.0)]),
        pattern(11, "Shifting Gears", vec![(Resource::ForgedLight, 60.0)]),
        // Endgame Arc (13-18)
        pattern(
            12,
            "Harmony",
            vec![
                (Resource::Ember, 200.0),
                (Resource::Reflection, 200.0),
                (Resource::VoidEssence, 200.0),
                (Resource::Memory, 200.0),
                (Resource::Silence, 200.0),
                (Resource::Resonance, 200.0),
            ],
        ),
        pattern(
            13,
            "The Triad",
            vec![
                (Resource::Ember, 120.0),
                (Resource::Reflection, 120.0),
                (Resource::VoidEssence, 120.0),
                (Resource::Memory, 120.0),
                (Resource::Silence, 120.0),
                (Resource::Resonance, 120.0),
                (Resource::ForgedLight, 120.0),
                (Resource::EchoGlass, 120.0),
                (Resource::StillbornSong, 120.0),
            ],
        ),
        pattern(
            14,
            "Razor's Edge",
            vec![(Resource::ForgedLight, 160.0), (Resource::EchoGlass, 160.0)],
        ),
        pattern(15, "Resonance Cascade", vec![(Resource::Resonance, 400.0)]),
        pattern(16, "The Unraveling", vec![(Resource::WovenReality, 60.0)]),
        pattern(
            17,
            "Mended Loom",
            vec![
                (Resource::WovenReality, 240.0),
                (Resource::Ember, 400.0),
                (Resource::Reflection, 400.0),
                (Resource::VoidEssence, 400.0),
                (Resource::Memory, 400.0),
                (Resource::Silence, 400.0),
                (Resource::Resonance, 400.0),
                (Resource::ForgedLight, 240.0),
                (Resource::EchoGlass, 240.0),
                (Resource::StillbornSong, 240.0),
            ],
        ),
    ]
}

fn pattern(index: u32, name: &str, reqs: Vec<(Resource, f64)>) -> WovenPattern {
    WovenPattern {
        index,
        name: name.to_string(),
        requirements: reqs
            .into_iter()
            .map(|(resource, amount)| PatternRequirement {
                resource,
                required_rate: 0.0,
                sustain_duration_secs: 0.0,
                sustained_secs: 0.0,
                completed: false,
                amount,
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
        assert_eq!(loom.persistent.patterns.len(), 18);
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
        assert_eq!(loom.persistent.patterns.len(), 18);

        // Mark first pattern as having accumulated progress.
        loom.persistent.patterns[0].requirements[0].accumulated = 0.5;

        // Calling again should be a no-op (re-entry guard).
        complete_discovery(&mut loom);
        assert_eq!(loom.persistent.patterns.len(), 18);
        assert!(
            (loom.persistent.patterns[0].requirements[0].accumulated - 0.5).abs() < 1e-9,
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
    fn test_all_patterns_start_with_zero_accumulated() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);

        for pattern in &loom.persistent.patterns {
            for req in &pattern.requirements {
                assert!(
                    req.accumulated.abs() < 1e-9,
                    "pattern '{}' req {:?} should start with 0 accumulated",
                    pattern.name,
                    req.resource
                );
            }
        }
    }

    // ── pattern amounts ───────────────────────────────────────────────────────

    #[test]
    fn test_first_pattern_amount_is_ten() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);
        let first = &loom.persistent.patterns[0];
        assert_eq!(first.requirements.len(), 1);
        assert!(
            (first.requirements[0].amount - 10.0).abs() < 1e-9,
            "First Thread Ember amount should be 10.0, got {}",
            first.requirements[0].amount
        );
    }

    #[test]
    fn test_all_pattern_amounts_are_positive() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);

        for pattern in &loom.persistent.patterns {
            for req in &pattern.requirements {
                assert!(
                    req.amount > 0.0,
                    "pattern '{}' has non-positive amount for {:?}: {}",
                    pattern.name,
                    req.resource,
                    req.amount
                );
            }
        }
    }

    #[test]
    fn test_final_pattern_has_largest_total_amount() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);

        // "Mended Loom" (index 17) is the capstone and should have the largest
        // total amount (sum of all requirements) of any pattern in the sequence.
        let total_amounts: Vec<f64> = loom
            .persistent
            .patterns
            .iter()
            .map(|p| p.requirements.iter().map(|r| r.amount).sum::<f64>())
            .collect();
        let max_total = total_amounts
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let last_total = *total_amounts.last().unwrap();
        assert!(
            (last_total - max_total).abs() < 1e-9,
            "Mended Loom should have the largest total amount ({} vs max {})",
            last_total,
            max_total
        );
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
    }

    // ── active_pattern initial value ─────────────────────────────────────────

    #[test]
    fn test_active_pattern_starts_at_zero_after_discovery() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);
        assert_eq!(loom.persistent.active_pattern, 0);
    }
}
