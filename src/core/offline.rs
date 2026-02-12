//! Offline progression system.
//!
//! Calculates XP gained while the player is offline, simulating monster kills
//! at a reduced rate. Supports Haven bonuses for increased offline XP.

use super::constants::*;
use super::game_logic::{apply_tick_xp, xp_gain_per_tick};
use super::game_state::GameState;
use crate::character::attributes::AttributeType;
use chrono::Utc;

/// Report of offline progression results
#[derive(Debug, Default, Clone)]
pub struct OfflineReport {
    pub elapsed_seconds: i64,
    pub total_level_ups: u32,
    pub xp_gained: u64,
    pub level_before: u32,
    pub level_after: u32,
    /// Effective offline XP rate as a percentage of online rate
    pub offline_rate_percent: f64,
    /// Haven bonus percentage (0.0 if Haven not discovered)
    pub haven_bonus_percent: f64,
}

/// Calculates the XP gained during offline time.
///
/// Based on simulated monster kills instead of passive time.
/// `haven_offline_xp_percent` is the Hearthstone bonus (0.0 if not built).
pub fn calculate_offline_xp(
    elapsed_seconds: i64,
    prestige_rank: u32,
    wis_modifier: i32,
    cha_modifier: i32,
    haven_offline_xp_percent: f64,
) -> f64 {
    let capped_seconds = elapsed_seconds.min(MAX_OFFLINE_SECONDS);

    // Estimate kills: average 1 kill every 5 seconds (includes combat + regen time)
    let estimated_kills = (capped_seconds as f64 / 5.0) * OFFLINE_MULTIPLIER;

    // Average XP per kill
    let xp_per_tick_rate = xp_gain_per_tick(prestige_rank, wis_modifier, cha_modifier);
    let avg_xp_per_kill = (COMBAT_XP_MIN_TICKS + COMBAT_XP_MAX_TICKS) as f64 / 2.0;
    let xp_per_kill = xp_per_tick_rate * avg_xp_per_kill;

    // Apply Haven Hearthstone bonus
    let base_xp = estimated_kills * xp_per_kill;
    base_xp * (1.0 + haven_offline_xp_percent / 100.0)
}

/// Processes offline progression and updates game state.
///
/// `haven_offline_xp_percent` is the Hearthstone bonus (0.0 if not built).
pub fn process_offline_progression(
    state: &mut GameState,
    haven_offline_xp_percent: f64,
) -> OfflineReport {
    let current_time = Utc::now().timestamp();
    let elapsed_seconds = current_time - state.last_save_time;

    if elapsed_seconds <= 0 {
        return OfflineReport::default();
    }

    let wis_mod = state.attributes.modifier(AttributeType::Wisdom);
    let cha_mod = state.attributes.modifier(AttributeType::Charisma);
    let offline_xp = calculate_offline_xp(
        elapsed_seconds,
        state.prestige_rank,
        wis_mod,
        cha_mod,
        haven_offline_xp_percent,
    );

    let level_before = state.character_level;
    let (total_level_ups, _) = apply_tick_xp(state, offline_xp);
    let level_after = state.character_level;

    state.last_save_time = current_time;

    let offline_rate_percent =
        OFFLINE_MULTIPLIER * (1.0 + haven_offline_xp_percent / 100.0) * 100.0;

    OfflineReport {
        elapsed_seconds,
        total_level_ups,
        xp_gained: offline_xp as u64,
        level_before,
        level_after,
        offline_rate_percent,
        haven_bonus_percent: haven_offline_xp_percent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_offline_xp_basic() {
        // 1 hour offline, rank 0, no modifiers
        let xp = calculate_offline_xp(3600, 0, 0, 0, 0.0);

        // 3600 seconds / 5 = 720 estimated kills * 0.25 offline multiplier = 180 kills
        // XP per kill at rank 0 = 1.0 * 300 (avg) = 300
        // Total = 180 * 300 = 54,000 (roughly)
        assert!(xp > 25000.0 && xp < 100000.0);
    }

    #[test]
    fn test_calculate_offline_xp_capped_at_max() {
        // Test that offline XP is capped at MAX_OFFLINE_SECONDS (7 days)
        let one_week = 7 * 24 * 3600;
        let two_weeks = 14 * 24 * 3600;

        let xp_one_week = calculate_offline_xp(one_week, 0, 0, 0, 0.0);
        let xp_two_weeks = calculate_offline_xp(two_weeks, 0, 0, 0, 0.0);

        // Should be capped, so two weeks = one week
        assert!((xp_one_week - xp_two_weeks).abs() < 1.0);
    }

    #[test]
    fn test_calculate_offline_xp_with_prestige() {
        let base_xp = calculate_offline_xp(3600, 0, 0, 0, 0.0);
        let prestige_xp = calculate_offline_xp(3600, 1, 0, 0, 0.0);

        // Prestige 1 has 1.5x multiplier (using 1 + 0.5*rank^0.7 formula)
        assert!(prestige_xp > base_xp);
        let ratio = prestige_xp / base_xp;
        assert!((ratio - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_calculate_offline_xp_with_wisdom() {
        let base_xp = calculate_offline_xp(3600, 0, 0, 0, 0.0);
        let wis_xp = calculate_offline_xp(3600, 0, 5, 0, 0.0); // +5 WIS modifier

        // WIS +5 gives 1.25x multiplier
        assert!(wis_xp > base_xp);
        let ratio = wis_xp / base_xp;
        assert!((ratio - 1.25).abs() < 0.01);
    }

    #[test]
    fn test_calculate_offline_xp_with_haven_bonus() {
        let base_xp = calculate_offline_xp(3600, 0, 0, 0, 0.0);
        let haven_xp = calculate_offline_xp(3600, 0, 0, 0, 100.0); // +100% from Hearthstone T3

        // Haven +100% should double offline XP
        let ratio = haven_xp / base_xp;
        assert!(
            (ratio - 2.0).abs() < 0.01,
            "Haven +100% offline XP should double base XP, got {:.2}x",
            ratio
        );
    }

    #[test]
    fn test_calculate_offline_xp_very_small_elapsed_produces_nonzero() {
        // Even 1 second of offline time should produce some XP
        let xp = calculate_offline_xp(1, 0, 0, 0, 0.0);

        assert!(
            xp > 0.0,
            "1 second offline should produce non-zero XP, got {}",
            xp
        );
    }

    #[test]
    fn test_process_offline_progression_updates_last_save_time() {
        let mut state = GameState::new("Suspend Test".to_string(), 0);

        // Set last_save_time to 2 hours ago
        let two_hours_ago = chrono::Utc::now().timestamp() - 7200;
        state.last_save_time = two_hours_ago;

        let _report = process_offline_progression(&mut state, 0.0);

        // After processing, last_save_time should be updated to approximately now
        let now = chrono::Utc::now().timestamp();
        assert!(
            (state.last_save_time - now).abs() <= 2,
            "last_save_time should be updated to current time after processing, \
             got delta of {} seconds",
            (state.last_save_time - now).abs()
        );
        assert!(
            state.last_save_time > two_hours_ago,
            "last_save_time should advance past the old value"
        );
    }

    #[test]
    fn test_process_offline_progression_zero_elapsed_returns_default() {
        let mut state = GameState::new("Zero Elapsed Test".to_string(), 0);

        // Set last_save_time to exactly now (zero elapsed)
        state.last_save_time = chrono::Utc::now().timestamp();

        let report = process_offline_progression(&mut state, 0.0);

        assert_eq!(
            report.xp_gained, 0,
            "Zero elapsed time should produce no XP"
        );
        assert_eq!(
            report.total_level_ups, 0,
            "Zero elapsed time should produce no level ups"
        );
    }

    #[test]
    fn test_process_offline_progression_negative_elapsed_returns_default() {
        let mut state = GameState::new("Negative Elapsed Test".to_string(), 0);

        // Set last_save_time to the future (negative elapsed)
        state.last_save_time = chrono::Utc::now().timestamp() + 3600;

        let report = process_offline_progression(&mut state, 0.0);

        assert_eq!(
            report.xp_gained, 0,
            "Negative elapsed time should produce no XP"
        );
        assert_eq!(
            report.total_level_ups, 0,
            "Negative elapsed time should produce no level ups"
        );
        assert_eq!(
            report.elapsed_seconds, 0,
            "Negative elapsed should report 0 elapsed seconds in default report"
        );
    }

    #[test]
    fn test_last_save_time_sync_prevents_double_counting() {
        let mut state = GameState::new("Double Count Test".to_string(), 0);

        // Set last_save_time to 1 hour ago
        state.last_save_time = chrono::Utc::now().timestamp() - 3600;

        // First call: should process the full hour of offline time
        let report1 = process_offline_progression(&mut state, 0.0);
        assert!(
            report1.xp_gained > 0,
            "First call should gain XP from the 1-hour gap"
        );

        // Capture the updated last_save_time
        let updated_save_time = state.last_save_time;
        let now = chrono::Utc::now().timestamp();
        assert!(
            (updated_save_time - now).abs() <= 2,
            "last_save_time should be synced to current time after first call"
        );

        // Second call immediately after: should gain zero or near-zero XP
        // because last_save_time was just synced
        let report2 = process_offline_progression(&mut state, 0.0);
        assert!(
            report2.xp_gained < report1.xp_gained / 100,
            "Second immediate call should gain negligible XP (got {} vs first call {}), \
             last_save_time sync should prevent double-counting",
            report2.xp_gained,
            report1.xp_gained
        );
    }
}
