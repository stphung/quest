//! The Vessel — Act 2 launch gate (sub-project 1).
//!
//! After the player clears Zone 50, a signal from a living branch of
//! Yggdrasil is discovered. Launching the Vessel requires holding
//! 250,000 Prestige Ranks and burning them in a single all-or-nothing
//! action. See `docs/superpowers/specs/2026-03-27-vessel-launch-gate-design.md`.

pub mod colony;
pub mod junction;
pub mod letters;
pub mod nights;
pub mod persistence;
pub mod pilgrims;
pub mod refits;
pub mod route;
pub mod scenes;
pub mod souls;
pub mod voyage;
pub mod weather;

use crate::core::game_state::GameState;

/// Transient voyage UI state (not serialized) — which surface of the
/// Crossing screen is open. Lives here (like `DeepUiState`) so input and
/// rendering share one struct.
#[derive(Debug, Default)]
pub struct VoyageUiState {
    pub view: VoyageView,
    /// A multi-beat arrival scene being read (spec 4).
    pub scene_play: Option<ScenePlay>,
    /// A one-line moment being read (arc beats, boardings, recoveries).
    pub scene_modal: Option<SceneModal>,
    /// Queued log moments shown one at a time after the current one closes.
    pub moments: std::collections::VecDeque<SceneModal>,
}

/// A scene playback in progress: which paragraph the reader is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenePlay {
    pub playback: voyage::ScenePlayback,
    pub index: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VoyageView {
    #[default]
    Chart,
    /// Choosing a road at a junction; `selected` indexes the card list.
    Junction {
        selected: usize,
    },
    /// The trim panel; `selected` indexes `voyage::Trim::ALL`.
    Trim {
        selected: usize,
    },
    Rumors,
    /// The Souls panel; `selected` indexes the met-soul list.
    Souls {
        selected: usize,
    },
    /// The watch forecast; `selected` indexes the next three nights.
    Watch {
        selected: usize,
    },
    /// Choosing who steps ashore so a pending ask can be accepted;
    /// `selected` indexes the aboard list.
    Farewell {
        selected: usize,
    },
    /// The manifest — the harbor's first room (spec 7, arrived only).
    Manifest {
        scroll: u16,
    },
    /// The keepsake chart, pannable; `(x, y)` is the canvas point the
    /// viewport centers on (spec 7, arrived only).
    Keepsake {
        x: u16,
        y: u16,
    },
    /// The record — the complete Log with the letters bound in
    /// (spec 7, arrived only).
    Record {
        scroll: u16,
    },
    /// The Reckoning — the colony's numbers pane (spec 9).
    Reckoning,
}

/// A scene being read: title line + body text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneModal {
    pub title: String,
    pub body: String,
}

/// Release kill-switch for Act 2. While `false`, the Vessel is fully
/// invisible: no discovery modal, no ticker whispers, no stats-panel row,
/// no `[V]` hotkey — and therefore no path to the launch burn.
///
/// Zone 50 detection still records `vessel_signal_discovered` in saves so
/// that already-qualified players light up the moment Act 2 is enabled.
///
/// **Enabling Act 2 is a one-line change: flip this to `true`.**
/// For previewing on a release build without recompiling (or driving the
/// game in tests/screenshots), set the `QUEST_ACT2=1` environment variable.
pub const ACT2_ENABLED: bool = false;

/// Runtime check for the Act 2 kill-switch: the compiled default, or the
/// `QUEST_ACT2=1` environment override (read once).
pub fn act2_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| ACT2_ENABLED || std::env::var("QUEST_ACT2").is_ok_and(|v| v == "1"))
}

/// Prestige rank cost of launching the Vessel, burned in one action.
pub const LAUNCH_PR_COST: u32 = 250_000;

/// Woven Patterns required to launch (the complete Loom becomes the hull).
pub const LAUNCH_REQUIRED_PATTERNS: usize = 28;

/// Ascension level required to launch.
pub const LAUNCH_REQUIRED_ASCENSION: u32 = 10;

/// Seconds of play time between atmospheric ticker whispers.
pub const WHISPER_INTERVAL_SECONDS: u64 = 60;

/// Atmospheric ticker messages shown after the signal is discovered.
pub const VESSEL_WHISPERS: [&str; 5] = [
    "The Loom trembles. Something distant answers.",
    "A signal pulses from beyond the branches.",
    "The Origin Thread frays. The roots grow cold.",
    "The weave resonates with something far away.",
    "Yggdrasil shudders. A beacon calls.",
];

/// Returns the whisper for a given rotation index (wraps).
pub fn whisper_message(index: u64) -> &'static str {
    VESSEL_WHISPERS[(index as usize) % VESSEL_WHISPERS.len()]
}

/// True when every launch gate is met: the signal has been discovered
/// (which implies Zone 50 was cleared), Ascension X is reached, all 28
/// Woven Patterns are complete, and the player holds the full burn cost.
pub fn can_launch(state: &GameState, completed_patterns: usize) -> bool {
    state.vessel_signal_discovered
        && !state.vessel_launched
        && state.ascension_level >= LAUNCH_REQUIRED_ASCENSION
        && completed_patterns >= LAUNCH_REQUIRED_PATTERNS
        && state.prestige_rank >= LAUNCH_PR_COST
}

/// Burns `LAUNCH_PR_COST` PR in a single action and marks the Vessel launched.
/// Returns false (and changes nothing) if any gate is unmet.
pub fn perform_launch(state: &mut GameState, completed_patterns: usize) -> bool {
    if !can_launch(state, completed_patterns) {
        return false;
    }
    state.prestige_rank -= LAUNCH_PR_COST;
    state.recalculate_prestige_bonuses();
    state.derived_stats_dirty = true;
    state.vessel_launched = true;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gated_state() -> GameState {
        let mut state = GameState::new("Voyager".to_string(), 0);
        state.vessel_signal_discovered = true;
        state.ascension_level = 10;
        state.prestige_rank = 250_000;
        state
    }

    /// Release guard: Act 2 ships dark. Enabling it is a deliberate
    /// two-line change — flip `ACT2_ENABLED` to `true` AND update this
    /// assertion. This test failing on a PR means someone flipped the
    /// switch; make sure that was the intent.
    #[test]
    #[allow(clippy::assertions_on_constants)] // The constant IS the thing under test.
    fn act2_kill_switch_is_off_for_release() {
        assert!(
            !ACT2_ENABLED,
            "Act 2 kill-switch is ON — if this is the launch PR, update this test too"
        );
    }

    #[test]
    fn can_launch_requires_all_four_gates() {
        let state = gated_state();
        assert!(can_launch(&state, 28));

        let mut s = gated_state();
        s.vessel_signal_discovered = false;
        assert!(!can_launch(&s, 28));

        let mut s = gated_state();
        s.ascension_level = 9;
        assert!(!can_launch(&s, 28));

        assert!(!can_launch(&gated_state(), 27));

        let mut s = gated_state();
        s.prestige_rank = 249_999;
        assert!(!can_launch(&s, 28));
    }

    #[test]
    fn launch_burns_exactly_the_cost_and_sets_flag() {
        let mut state = gated_state();
        state.prestige_rank = 253_218;
        assert!(perform_launch(&mut state, 28));
        assert_eq!(state.prestige_rank, 3_218);
        assert!(state.vessel_launched);
        assert!(state.derived_stats_dirty);
    }

    #[test]
    fn launch_refused_below_cost_and_after_launch() {
        let mut state = gated_state();
        state.prestige_rank = 50_000;
        assert!(!perform_launch(&mut state, 28));
        assert_eq!(state.prestige_rank, 50_000);
        assert!(!state.vessel_launched);

        let mut state = gated_state();
        assert!(perform_launch(&mut state, 28));
        let rank_after = state.prestige_rank;
        assert!(!perform_launch(&mut state, 28));
        assert_eq!(state.prestige_rank, rank_after);
    }

    #[test]
    fn whispers_rotate_and_wrap() {
        assert_eq!(whisper_message(0), VESSEL_WHISPERS[0]);
        assert_eq!(whisper_message(5), VESSEL_WHISPERS[0]);
        assert_eq!(whisper_message(7), VESSEL_WHISPERS[2]);
    }
}
