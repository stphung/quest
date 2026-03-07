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
        pattern(0, "First Thread", vec![(Resource::Ember, 2.0)], 1800),
        pattern(
            1,
            "The Bridge",
            vec![(Resource::Ember, 3.0), (Resource::Reflection, 1.0)],
            3600,
        ),
        pattern(
            2,
            "Long Road",
            vec![(Resource::Ember, 2.0), (Resource::Memory, 1.0)],
            3600,
        ),
        pattern(
            3,
            "Balancing Act",
            vec![
                (Resource::Ember, 2.0),
                (Resource::Reflection, 2.0),
                (Resource::VoidEssence, 2.0),
            ],
            5400,
        ),
        pattern(
            4,
            "Full Circle",
            vec![
                (Resource::Ember, 1.0),
                (Resource::Reflection, 1.0),
                (Resource::VoidEssence, 1.0),
                (Resource::Memory, 1.0),
                (Resource::Silence, 1.0),
                (Resource::Resonance, 1.0),
            ],
            7200,
        ),
        pattern(
            5,
            "The Catalyst",
            vec![(Resource::CondensedEmber, 1.0)],
            7200,
        ),
        // Mastery Arc (7-12)
        pattern(
            6,
            "Crossed Streams",
            vec![(Resource::CondensedEmber, 1.0), (Resource::EmberEcho, 1.0)],
            7200,
        ),
        pattern(
            7,
            "The Diversion",
            vec![(Resource::ForgedLight, 1.0), (Resource::Ember, 3.0)],
            9000,
        ),
        pattern(
            8,
            "Three Confluences",
            vec![
                (Resource::ForgedLight, 1.0),
                (Resource::EchoGlass, 1.0),
                (Resource::StillbornSong, 1.0),
            ],
            10800,
        ),
        pattern(
            9,
            "Pressure Test",
            vec![(Resource::ForgedLight, 2.0), (Resource::EchoGlass, 2.0)],
            10800,
        ),
        pattern(
            10,
            "The Bottleneck",
            vec![(Resource::StillbornSong, 3.0)],
            10800,
        ),
        pattern(
            11,
            "Shifting Gears",
            vec![(Resource::ForgedLight, 3.0)],
            7200,
        ),
        // Endgame Arc (13-18)
        pattern(
            12,
            "Harmony",
            vec![
                (Resource::Ember, 5.0),
                (Resource::Reflection, 5.0),
                (Resource::VoidEssence, 5.0),
                (Resource::Memory, 5.0),
                (Resource::Silence, 5.0),
                (Resource::Resonance, 5.0),
            ],
            14400,
        ),
        pattern(
            13,
            "The Triad",
            vec![
                (Resource::Ember, 3.0),
                (Resource::Reflection, 3.0),
                (Resource::VoidEssence, 3.0),
                (Resource::Memory, 3.0),
                (Resource::Silence, 3.0),
                (Resource::Resonance, 3.0),
                (Resource::ForgedLight, 3.0),
                (Resource::EchoGlass, 3.0),
                (Resource::StillbornSong, 3.0),
            ],
            14400,
        ),
        pattern(
            14,
            "Razor's Edge",
            vec![(Resource::ForgedLight, 4.0), (Resource::EchoGlass, 4.0)],
            14400,
        ),
        pattern(
            15,
            "Resonance Cascade",
            vec![(Resource::Resonance, 10.0)],
            14400,
        ),
        pattern(
            16,
            "The Unraveling",
            vec![(Resource::WovenReality, 1.0)],
            21600,
        ),
        pattern(
            17,
            "Mended Loom",
            vec![
                (Resource::WovenReality, 3.0),
                (Resource::Ember, 5.0),
                (Resource::Reflection, 5.0),
                (Resource::VoidEssence, 5.0),
                (Resource::Memory, 5.0),
                (Resource::Silence, 5.0),
                (Resource::Resonance, 5.0),
                (Resource::ForgedLight, 3.0),
                (Resource::EchoGlass, 3.0),
                (Resource::StillbornSong, 3.0),
            ],
            28800,
        ),
    ]
}

fn pattern(
    index: u32,
    name: &str,
    reqs: Vec<(Resource, f64)>,
    sustain_seconds: u32,
) -> WovenPattern {
    WovenPattern {
        index,
        name: name.to_string(),
        requirements: reqs
            .into_iter()
            .map(|(resource, rate)| PatternRequirement {
                resource,
                rate_per_hour: rate,
            })
            .collect(),
        sustain_seconds,
        sustained_seconds: 0,
        sustained_seconds_frac: 0.0,
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

        // Mark first pattern as having progress.
        loom.persistent.patterns[0].sustained_seconds = 100;

        // Calling again should be a no-op (re-entry guard).
        complete_discovery(&mut loom);
        assert_eq!(loom.persistent.patterns.len(), 18);
        assert_eq!(
            loom.persistent.patterns[0].sustained_seconds, 100,
            "pattern progress must be preserved on re-call"
        );
    }
}
