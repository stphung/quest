//! The Vessel — Act 2 launch gate (sub-project 1).
//!
//! After the player clears Zone 50, a signal from a living branch of
//! Yggdrasil is discovered. Launching the Vessel requires holding
//! 100,000 Prestige Ranks and burning them in a single all-or-nothing
//! action. See `docs/superpowers/specs/2026-03-27-vessel-launch-gate-design.md`.

use crate::core::game_state::GameState;

/// Prestige rank cost of launching the Vessel, burned in one action.
pub const LAUNCH_PR_COST: u32 = 100_000;

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

/// Burns 100,000 PR in a single action and marks the Vessel launched.
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
        state.prestige_rank = 100_000;
        state
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
        s.prestige_rank = 99_999;
        assert!(!can_launch(&s, 28));
    }

    #[test]
    fn launch_burns_exactly_the_cost_and_sets_flag() {
        let mut state = gated_state();
        state.prestige_rank = 103_218;
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
