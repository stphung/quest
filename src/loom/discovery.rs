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
        // ── Chapter I: The Awakening (1-8) ── ~3 days (72 hours) ──
        pattern(
            0,
            "First Thread",
            "My first thread. Clumsy, fraying at the edges, but it held. The Loom hummed when I touched it \u{2014} a low, patient sound. I think it\u{2019}s been waiting for someone.",
            vec![(Resource::Ember, 25.0, 2.0)],
        ),
        pattern(
            1,
            "Still Waters",
            "I sat with the silence today. Hours of nothing, and then \u{2014} a thread appeared where none had been. The Loom rewards stillness. I need to remember that.",
            vec![(Resource::Silence, 25.0, 2.0)],
        ),
        pattern(
            2,
            "Echoing Halls",
            "The old patterns are still here, woven into the Archive. Someone came before me. Their threads are faded but I can feel the shape of what they built. I won\u{2019}t make their mistakes.",
            vec![(Resource::Memory, 25.0, 4.0)],
        ),
        pattern(
            3,
            "Harmonic Pulse",
            "I can hear it now \u{2014} a pulse beneath everything. The Loom has a heartbeat, and it\u{2019}s been beating long before I arrived. My threads are starting to match its rhythm.",
            vec![(Resource::Resonance, 25.0, 4.0)],
        ),
        pattern(
            4,
            "Mirror and Void",
            "I was afraid to combine them. Void and reflection seemed like they\u{2019}d destroy each other. Instead they braided together so naturally I wondered if they were ever truly apart.",
            vec![
                (Resource::Reflection, 35.0, 6.0),
                (Resource::VoidEssence, 35.0, 6.0),
            ],
        ),
        pattern(
            5,
            "Full Circle",
            "Six threads, woven together for the first time. The sound the Loom made \u{2014} I don\u{2019}t have words for it. A chord that resonated in my chest. I laughed. I actually laughed.",
            vec![
                (Resource::Ember, 30.0, 10.0),
                (Resource::Reflection, 30.0, 10.0),
                (Resource::VoidEssence, 30.0, 10.0),
                (Resource::Memory, 30.0, 10.0),
                (Resource::Silence, 30.0, 10.0),
                (Resource::Resonance, 30.0, 10.0),
            ],
        ),
        pattern(
            6,
            "The Catalyst",
            "I forged light from fire and void today \u{2014} poured everything I had into it. The Loom drank it up, hungry, insatiable. I think it\u{2019}s testing how much I can sustain.",
            vec![(Resource::ForgedLight, 22.0, 16.0)],
        ),
        pattern(
            7,
            "Echo of Flame",
            "Glass and song, woven side by side for the first time. Two confluences flowing together \u{2014} they don\u{2019}t merge, but they resonate. The Loom hums differently when they\u{2019}re close.",
            vec![(Resource::EchoGlass, 20.0, 20.0), (Resource::StillbornSong, 18.0, 20.0)],
        ),
        // ── Chapter II: The Deepening (9-16) ── ~10 days (236 hours) ──
        pattern(
            8,
            "Forged in Fire",
            "I bent light into something permanent today. The Loom bends with me now, not against me. There was a time I wrestled with every thread. Now they come when I call.",
            vec![(Resource::ForgedLight, 35.0, 16.0)],
        ),
        pattern(
            9,
            "Glass Resonance",
            "Glass from chaos. I keep expecting the delicate things to shatter, but they hold. The Loom trusts me with fragile work now. I\u{2019}m not sure I\u{2019}ve earned that yet.",
            vec![
                (Resource::EchoGlass, 30.0, 16.0),
                (Resource::ForgedLight, 25.0, 16.0),
            ],
        ),
        pattern(
            10,
            "The Unsung",
            "I wove a song that shouldn\u{2019}t exist \u{2014} born from silence and stillness, shaped into sound. It plays quietly in the back of my mind. I don\u{2019}t think it will ever stop.",
            vec![
                (Resource::StillbornSong, 35.0, 24.0),
                (Resource::Resonance, 50.0, 24.0),
            ],
        ),
        pattern(
            11,
            "Void Distillation",
            "I distilled the void today. Held nothingness in my hands and refined it until something remained. There\u{2019}s substance in emptiness. The Loom knew that all along.",
            vec![(Resource::PurifiedVoid, 20.0, 24.0)],
        ),
        pattern(
            12,
            "Crossed Streams",
            "Two streams crossed without breaking. Light and glass, flowing through each other. A year ago this would have been impossible. Now it feels like the obvious next step.",
            vec![
                (Resource::ForgedLight, 30.0, 24.0),
                (Resource::PurifiedVoid, 18.0, 24.0),
            ],
        ),
        pattern(
            13,
            "The Asymmetry",
            "Not everything needs to balance. I forced symmetry for so long and the Loom resisted. Today I let the pattern lean, let it breathe unevenly, and it sang louder than anything I\u{2019}ve made.",
            vec![
                (Resource::CondensedEmber, 20.0, 36.0),
                (Resource::EmberEcho, 18.0, 36.0),
            ],
        ),
        pattern(
            14,
            "Pressure Test",
            "Three forces, one thread. Condensed ember, echo, and purified void \u{2014} all pulling in different directions. My hands don\u{2019}t shake anymore. I held them steady and the thread held too.",
            vec![
                (Resource::ForgedLight, 45.0, 36.0),
                (Resource::EchoGlass, 35.0, 36.0),
            ],
        ),
        pattern(
            15,
            "Three Confluences",
            "The great rivers converged today. Light, glass, and song \u{2014} all flowing together through a single point. I stood at the center and directed the flow. The Loom trembled. So did I.",
            vec![
                (Resource::ForgedLight, 40.0, 60.0),
                (Resource::EchoGlass, 40.0, 60.0),
                (Resource::StillbornSong, 40.0, 60.0),
            ],
        ),
        // ── Chapter III: The Unraveling (17-28) ── ~22 days (534 hours) ──
        pattern(
            16,
            "The Amplifier",
            "More. Always more. The Loom\u{2019}s appetite has grown with mine and I\u{2019}m no longer sure which of us is driving. The threads come faster now. I have to keep up or be swept away.",
            vec![(Resource::ForgedLight, 60.0, 18.0)],
        ),
        pattern(
            17,
            "Purified Cascade",
            "I can see the tangles now \u{2014} the clean paths through the chaos. What used to take hours of careful work now flows in minutes. My hands remember what my mind hasn\u{2019}t figured out yet.",
            vec![
                (Resource::PurifiedVoid, 35.0, 24.0),
                (Resource::ForgedLight, 35.0, 24.0),
            ],
        ),
        pattern(
            18,
            "Resonance Cascade",
            "The hum became a roar today. Every thread in the Loom vibrated at once, and for a moment I couldn\u{2019}t tell where the sound ended and I began. I had to shut it down. I wasn\u{2019}t ready.",
            vec![
                (Resource::StillbornSong, 50.0, 24.0),
                (Resource::Resonance, 75.0, 24.0),
            ],
        ),
        pattern(
            19,
            "First Weave",
            "I touched reality. Not metaphor \u{2014} I felt the weave of the world give way beneath my fingers. Just for a moment. Just a thread\u{2019}s width. But the hole I made hasn\u{2019}t closed. It\u{2019}s waiting.",
            vec![(Resource::WovenReality, 15.0, 30.0)],
        ),
        pattern(
            20,
            "The Unraveling",
            "To weave the new, I had to unravel the old. I pulled a thread I\u{2019}d been proud of \u{2014} one of my first \u{2014} and watched it dissolve. The Loom doesn\u{2019}t mourn. I\u{2019}m learning not to either.",
            vec![
                (Resource::WovenReality, 25.0, 36.0),
                (Resource::PurifiedVoid, 20.0, 36.0),
            ],
        ),
        pattern(
            21,
            "Grand Harmony",
            "Everything aligned. Every thread, every node, every silence. Nine resources flowing in perfect concert for thirty-six hours. I didn\u{2019}t sleep. I didn\u{2019}t need to. The Loom sustained me.",
            vec![
                (Resource::Ember, 62.5, 36.0),
                (Resource::Reflection, 62.5, 36.0),
                (Resource::VoidEssence, 62.5, 36.0),
                (Resource::Memory, 62.5, 36.0),
                (Resource::Silence, 62.5, 36.0),
                (Resource::Resonance, 62.5, 36.0),
                (Resource::ForgedLight, 25.0, 36.0),
                (Resource::EchoGlass, 25.0, 36.0),
            ],
        ),
        pattern(
            22,
            "The Knot",
            "Some patterns resist. They have their own will \u{2014} a stubbornness I recognize because it mirrors my own. I stopped forcing and started negotiating. The knot loosened when I asked nicely.",
            vec![
                (Resource::CondensedEmber, 30.0, 36.0),
                (Resource::EmberEcho, 25.0, 36.0),
            ],
        ),
        pattern(
            23,
            "Strange Alchemy",
            "The old rules dissolved today. Combinations that shouldn\u{2019}t work, that violate everything I learned in the first chapter \u{2014} they produce the most beautiful results. The Loom is rewriting me.",
            vec![
                (Resource::EmberEcho, 30.0, 42.0),
                (Resource::PurifiedVoid, 30.0, 42.0),
                (Resource::StillbornSong, 35.0, 42.0),
            ],
        ),
        pattern(
            24,
            "Refined Purpose",
            "The void isn\u{2019}t empty. It never was. I spent all this time filling it when I should have been listening. There\u{2019}s a voice in the emptiness, and it\u{2019}s been telling me what comes next.",
            vec![
                (Resource::WovenReality, 25.0, 48.0),
                (Resource::EmberEcho, 20.0, 48.0),
                (Resource::Reflection, 75.0, 48.0),
            ],
        ),
        pattern(
            25,
            "The Flood",
            "Reality poured through the Loom today like water through a broken dam. I barely held on. My hands are raw, my vision is blurred, but I kept the threads intact. Barely. Barely was enough.",
            vec![
                (Resource::WovenReality, 40.0, 48.0),
                (Resource::Ember, 100.0, 48.0),
            ],
        ),
        pattern(
            26,
            "Everything Flows",
            "I stopped fighting the current. Thirteen resources, all flowing at once, all connected. I\u{2019}m not directing anymore \u{2014} I\u{2019}m part of the flow. The distinction between weaver and woven is dissolving.",
            vec![
                (Resource::ForgedLight, 60.0, 72.0),
                (Resource::EchoGlass, 55.0, 72.0),
                (Resource::StillbornSong, 50.0, 72.0),
                (Resource::CondensedEmber, 25.0, 72.0),
                (Resource::PurifiedVoid, 20.0, 72.0),
            ],
        ),
        pattern(
            27,
            "Mended Loom",
            "The last thread is in place. The Loom is whole \u{2014} not repaired, but completed. It was never broken. It was waiting to be finished. And now that it is, I understand: I didn\u{2019}t mend the Loom. The Loom mended me.",
            vec![
                (Resource::WovenReality, 50.0, 120.0),
                (Resource::EmberEcho, 30.0, 120.0),
                (Resource::Reflection, 125.0, 120.0),
            ],
        ),
        // ── The Eternal Weave ── endgame sink, never completes ──
        eternal_pattern(
            28,
            "The Eternal Weave",
            "There is no final thread. The Loom weaves on.",
            vec![(Resource::WovenReality, 1.0, 1.0)],
        ),
    ]
}

fn pattern(index: u32, name: &str, flavor: &str, reqs: Vec<(Resource, f64, f64)>) -> WovenPattern {
    WovenPattern {
        index,
        name: name.to_string(),
        flavor: flavor.to_string(),
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
        eternal: false,
    }
}

fn eternal_pattern(
    index: u32,
    name: &str,
    flavor: &str,
    reqs: Vec<(Resource, f64, f64)>,
) -> WovenPattern {
    let mut p = pattern(index, name, flavor, reqs);
    p.eternal = true;
    p
}

/// Returns the narrative chapter name for a given pattern index (0-based).
pub fn pattern_chapter(index: u32) -> &'static str {
    match index {
        0..=7 => "Chapter I: The Awakening",
        8..=15 => "Chapter II: The Deepening",
        16..=27 => "Chapter III: The Unraveling",
        28 => "The Eternal Weave",
        _ => "",
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
        assert_eq!(loom.persistent.patterns.len(), 29);
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
        assert_eq!(loom.persistent.patterns.len(), 29);

        // Mark first pattern as having sustain progress.
        loom.persistent.patterns[0].requirements[0].sustained_secs = 0.5;

        // Calling again should be a no-op (re-entry guard).
        complete_discovery(&mut loom);
        assert_eq!(loom.persistent.patterns.len(), 29);
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
    fn test_final_non_eternal_pattern_has_longest_duration() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);
        // Mended Loom (index 27) is the last non-eternal pattern
        let mended_loom = &loom.persistent.patterns[27];
        assert_eq!(mended_loom.name, "Mended Loom");
        let duration = mended_loom.requirements[0].sustain_duration_secs;
        assert!((duration - 432_000.0).abs() < 1e-9); // 120 hours
    }

    // ── pattern name spot checks ──────────────────────────────────────────────

    #[test]
    fn test_first_pattern_name_is_first_thread() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);
        assert_eq!(loom.persistent.patterns[0].name, "First Thread");
    }

    #[test]
    fn test_last_pattern_is_eternal_weave() {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);
        let last = loom.persistent.patterns.last().unwrap();
        assert_eq!(last.name, "The Eternal Weave");
        assert!(last.eternal);
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

    // ── pattern_chapter ───────────────────────────────────────────────────────

    #[test]
    fn test_pattern_chapter_awakening_range() {
        for i in 0..=7 {
            assert_eq!(pattern_chapter(i), "Chapter I: The Awakening");
        }
    }

    #[test]
    fn test_pattern_chapter_deepening_range() {
        for i in 8..=15 {
            assert_eq!(pattern_chapter(i), "Chapter II: The Deepening");
        }
    }

    #[test]
    fn test_pattern_chapter_unraveling_range() {
        for i in 16..=27 {
            assert_eq!(pattern_chapter(i), "Chapter III: The Unraveling");
        }
    }

    #[test]
    fn test_pattern_chapter_eternal_weave() {
        assert_eq!(pattern_chapter(28), "The Eternal Weave");
    }

    #[test]
    fn test_pattern_chapter_out_of_range_returns_empty() {
        assert_eq!(pattern_chapter(29), "");
        assert_eq!(pattern_chapter(1000), "");
    }
}
