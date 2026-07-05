//! The launch transition (spec 4, `2026-03-27-vessel-mode-transition-design.md`):
//! a 5-beat narrative sequence played once, between the launch burn and the
//! first Voyage frame. Each beat is a full-screen text beat advanced by
//! Enter — no Esc, no cancel, launch was already confirmed.
//!
//! Content is authored here, not derived from state: the transition always
//! plays the same five beats regardless of how the player got here.

/// How many beats the transition has. Beats are 1-indexed in
/// [`LaunchTransitionState`] to match the spec's own numbering.
pub const BEAT_COUNT: u8 = 5;

/// One beat's authored text, keyed by its 1-indexed beat number.
pub struct Beat {
    pub heading: &'static str,
    pub lines: &'static [&'static str],
}

const BEATS: [Beat; BEAT_COUNT as usize] = [
    Beat {
        heading: "Farewell",
        lines: &[
            "The Origin Thread has spoken.",
            "This branch of Yggdrasil is dying.",
            "Everything you built was preparation for this moment.",
        ],
    },
    Beat {
        heading: "Unweaving",
        lines: &[
            "The Loom unweaves itself.",
            "Twenty-eight patterns unspool into something new.",
            "Woven fate becomes woven hull.",
        ],
    },
    Beat {
        heading: "Construction",
        lines: &[
            "Reality folds. The Deep's roots become a keel.",
            "Haven's stones become ballast. The forge becomes an engine.",
            "A vessel takes shape from everything you were.",
        ],
    },
    Beat {
        heading: "Launch",
        lines: &[
            "A signal from 10,000 light-years away.",
            "A living branch. The last hope of a dying tree.",
            "",
            "The Vessel launches into the void.",
        ],
    },
    Beat {
        heading: "Void",
        lines: &[
            "         \u{00b7}  \u{00b7}",
            "   \u{00b7}          \u{00b7}    \u{00b7}",
            "\u{00b7}  \u{00b7}    \u{2571}\u{2550}\u{2550}\u{2550}\u{2572}",
            "       \u{2571} \u{25c6}\u{25c6}\u{25c6} \u{2572}    \u{00b7}",
            "\u{00b7}     \u{2571}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2572}        \u{00b7}",
            "     \u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}    \u{00b7}",
            "\u{00b7}        \u{2551}\u{2551}\u{2551}\u{2551}          \u{00b7}",
            "     \u{00b7}        \u{00b7}  \u{00b7}",
        ],
    },
];

/// One beat's content by 1-indexed beat number (1..=`BEAT_COUNT`). Panics on
/// an out-of-range beat — callers must keep `beat` clamped to the range.
pub fn beat(n: u8) -> &'static Beat {
    &BEATS[(n - 1) as usize]
}

/// The transition's transient progress (not serialized — an interrupted
/// transition simply restarts at beat 1 next launch, since
/// `GameState::vessel_transition_played` is the only durable record).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaunchTransitionState {
    pub beat: u8,
    /// UI-clock millis (`ui::clock::now_millis()`) when the current beat was
    /// entered — the caller (main.rs, which owns the UI clock) stamps this on
    /// creation and every `advance()`. Rendering derives each beat's
    /// animation phase from `now_millis() - beat_started_ms`, so this struct
    /// stays decoupled from the UI clock module itself.
    pub beat_started_ms: u128,
}

impl Default for LaunchTransitionState {
    fn default() -> Self {
        Self {
            beat: 1,
            beat_started_ms: 0,
        }
    }
}

impl LaunchTransitionState {
    /// Advance to the next beat. Returns `true` once the final beat has
    /// already been read and Enter is pressed again — the caller should
    /// then mark the transition complete and move on to the Voyage.
    pub fn advance(&mut self) -> bool {
        if self.beat >= BEAT_COUNT {
            true
        } else {
            self.beat += 1;
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_beat_has_content() {
        for n in 1..=BEAT_COUNT {
            let b = beat(n);
            assert!(!b.heading.is_empty());
            assert!(!b.lines.is_empty());
        }
    }

    #[test]
    fn advancing_walks_all_five_beats_then_finishes() {
        let mut t = LaunchTransitionState::default();
        assert_eq!(t.beat, 1);
        for expected in 2..=BEAT_COUNT {
            assert!(!t.advance(), "not done until the final beat is read");
            assert_eq!(t.beat, expected);
        }
        assert!(t.advance(), "the final Enter finishes the transition");
        assert_eq!(t.beat, BEAT_COUNT, "the beat counter does not overrun");
    }
}
