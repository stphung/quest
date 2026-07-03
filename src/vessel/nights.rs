//! Nights and the watch (sub-project 5).
//!
//! Every leg-night has a type, drawn deterministically from
//! `(voyage_seed, day_index)` and forecast three nights ahead. Standing a
//! night is a soul's job: the Watch-station soul stands by default, any
//! soul can be assigned per night (costing them a rest day), and a
//! Watch-affine stander resolves one grade kinder. Nights price provisions
//! and hope — they never injure, remove, or advance-to-loss a soul.
//!
//! See `docs/superpowers/specs/2026-07-03-vessel-underway-design.md`.

/// The five night types. Quiet nights need no watch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NightKind {
    Quiet,
    Cold,
    Hungry,
    Singing,
    Strange,
}

impl NightKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            NightKind::Quiet => "quiet",
            NightKind::Cold => "cold",
            NightKind::Hungry => "hungry",
            NightKind::Singing => "singing",
            NightKind::Strange => "strange",
        }
    }

    pub fn needs_watch(&self) -> bool {
        !matches!(self, NightKind::Quiet)
    }
}

/// How a typed night was met.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NightOutcome {
    /// Nobody stood it. Allowed, priced, never catastrophic.
    Unstood,
    /// A soul stood it.
    Stood,
    /// A Watch-affine soul stood it: one grade kinder.
    StoodAffine,
    /// A matched-course pilgrim crew covered it (base Stood grade).
    StoodByPilgrim,
}

/// What a resolved night costs (negative) or pays.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NightEffect {
    pub provisions: f64,
    pub hope: i8,
    /// The night yields a rumor (singing/strange, stood by the right soul).
    pub rumor: bool,
}

const NONE: NightEffect = NightEffect {
    provisions: 0.0,
    hope: 0,
    rumor: false,
};

/// Raw draw from the seeded sequence: ~40% quiet, ~15% each of the rest.
/// The per-leg strange cap is the voyage's job (it holds the leg state).
pub fn raw_night_kind(seed: u64, day: u64) -> NightKind {
    // Reuse the weather mixer family, salted differently.
    let mut x = seed ^ day.wrapping_mul(0x2545_f491_4f6c_dd1d).wrapping_add(0x6e79);
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    match (x ^ (x >> 31)) % 100 {
        0..=39 => NightKind::Quiet,
        40..=54 => NightKind::Cold,
        55..=69 => NightKind::Hungry,
        70..=84 => NightKind::Singing,
        _ => NightKind::Strange,
    }
}

/// The outcome matrix. Small numbers, no dice, no harm to souls — the
/// worst any night does is priced provisions/hope and a secret kept.
pub fn night_effect(kind: NightKind, outcome: NightOutcome) -> NightEffect {
    use NightKind::*;
    use NightOutcome::*;
    match (kind, outcome) {
        (Quiet, _) => NONE,
        (Cold, Unstood) => NightEffect {
            provisions: -5.0,
            ..NONE
        },
        (Cold, Stood | StoodByPilgrim) => NightEffect {
            provisions: -2.0,
            ..NONE
        },
        (Cold, StoodAffine) => NightEffect {
            provisions: -1.0,
            ..NONE
        },
        (Hungry, Unstood) => NightEffect {
            provisions: -6.0,
            ..NONE
        },
        (Hungry, Stood | StoodByPilgrim) => NightEffect {
            provisions: -3.0,
            ..NONE
        },
        (Hungry, StoodAffine) => NightEffect {
            provisions: -2.0,
            ..NONE
        },
        (Singing, StoodAffine) => NightEffect {
            rumor: true,
            ..NONE
        },
        (Singing, _) => NONE,
        (Strange, Unstood) => NightEffect { hope: -2, ..NONE },
        (Strange, StoodAffine) => NightEffect {
            rumor: true,
            ..NONE
        },
        (Strange, Stood | StoodByPilgrim) => NONE,
    }
}

/// The night's log entry, in prose. `stander` is the soul's name when one
/// stood it; pilgrim nights arrive in a stranger's voice.
pub fn log_line(kind: NightKind, outcome: NightOutcome, stander: Option<&str>) -> String {
    use NightKind::*;
    use NightOutcome::*;
    let name = stander.unwrap_or("No one");
    match (kind, outcome) {
        (Quiet, _) => "A quiet night. The ship sailed herself and asked for nothing.".to_string(),
        (Cold, Unstood) => "A cold night, unstood. Ice got into the stores and \
             nobody caught it until morning."
            .to_string(),
        (Cold, Stood | StoodAffine) => format!(
            "{name} stood the cold watch, cracking ice off the lines all \
             night. The stores stayed dry."
        ),
        (Cold, StoodByPilgrim) => "A cold night — covered by the ship keeping pace \
             alongside. 'Sleep,' their lantern signaled. 'We have it.'"
            .to_string(),
        (Hungry, Unstood) => "A hungry night, unstood. Something small and patient \
             got into the hold and ate like something large and hurried."
            .to_string(),
        (Hungry, Stood | StoodAffine) => format!(
            "{name} sat up with the hold and lost only what politeness \
             required."
        ),
        (Hungry, StoodByPilgrim) => "A hungry night — the pilgrims' cook rowed over \
             with a pot of something hot, and the hold went unrobbed."
            .to_string(),
        (Singing, Unstood) => "A singing night, and nobody on deck to hear it. In the \
             morning the crew found the rail wet with handprints. Something \
             was missed."
            .to_string(),
        (Singing, Stood | StoodByPilgrim) => format!(
            "A singing night. {name} listened until dawn and wrote none of \
             it down, on principle."
        ),
        (Singing, StoodAffine) => format!(
            "A singing night. {name} sang back — and the night, pleased to \
             be answered, left something useful in the song."
        ),
        (Strange, Unstood) => "A strange night, unstood. The night keeps whatever it \
             was going to say."
            .to_string(),
        (Strange, Stood | StoodByPilgrim) => format!(
            "A strange night. {name} stood it steadily and let it be \
             strange, which is the whole trick."
        ),
        (Strange, StoodAffine) => format!(
            "A strange night. {name} met its eye — and it blinked first, \
             and paid a forfeit on its way out."
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sequence_is_deterministic_and_roughly_distributed() {
        for day in 0..50 {
            assert_eq!(raw_night_kind(9, day), raw_night_kind(9, day));
        }
        let mut counts = [0u32; 5];
        for day in 0..4000 {
            let i = match raw_night_kind(1, day) {
                NightKind::Quiet => 0,
                NightKind::Cold => 1,
                NightKind::Hungry => 2,
                NightKind::Singing => 3,
                NightKind::Strange => 4,
            };
            counts[i] += 1;
        }
        assert!(counts[0] > 1200, "quiet ~40% (got {})", counts[0]);
        for c in &counts[1..] {
            assert!((300..1000).contains(c), "typed ~15% (got {c})");
        }
    }

    #[test]
    fn the_outcome_matrix_never_harms_a_soul_and_grades_kindly() {
        use NightKind::*;
        use NightOutcome::*;
        for kind in [Quiet, Cold, Hungry, Singing, Strange] {
            for outcome in [Unstood, Stood, StoodAffine, StoodByPilgrim] {
                let e = night_effect(kind, outcome);
                // Bounded prices, no catastrophe.
                assert!(e.provisions >= -6.0 && e.hope >= -2);
                // Standing is never worse than not standing.
                let unstood = night_effect(kind, Unstood);
                if outcome != Unstood {
                    assert!(e.provisions >= unstood.provisions);
                    assert!(e.hope >= unstood.hope);
                }
            }
        }
        // Affine is at least as kind as plain stood, and pays on song.
        assert!(night_effect(Singing, StoodAffine).rumor);
        assert!(night_effect(Strange, StoodAffine).rumor);
    }

    #[test]
    fn every_kind_and_outcome_resolves_to_a_log_line() {
        use NightKind::*;
        use NightOutcome::*;
        for kind in [Quiet, Cold, Hungry, Singing, Strange] {
            for outcome in [Unstood, Stood, StoodAffine, StoodByPilgrim] {
                let line = log_line(kind, outcome, Some("Runa"));
                assert!(!line.is_empty());
            }
        }
    }
}
