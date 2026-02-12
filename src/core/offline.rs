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

    // =========================================================================
    // Offline XP — Duration Scaling & Linearity Tests
    // =========================================================================

    #[test]
    fn test_offline_xp_scales_with_duration() {
        let xp_1h = calculate_offline_xp(3600, 0, 0, 0, 0.0);
        let xp_8h = calculate_offline_xp(8 * 3600, 0, 0, 0, 0.0);
        let xp_24h = calculate_offline_xp(24 * 3600, 0, 0, 0, 0.0);

        assert!(
            xp_8h > xp_1h,
            "8hr XP ({}) should exceed 1hr XP ({})",
            xp_8h,
            xp_1h
        );
        assert!(
            xp_24h > xp_8h,
            "24hr XP ({}) should exceed 8hr XP ({})",
            xp_24h,
            xp_8h
        );
    }

    #[test]
    fn test_offline_xp_linear_scaling() {
        // Before hitting the 7-day cap, doubling time should approximately double XP
        let xp_1h = calculate_offline_xp(3600, 0, 0, 0, 0.0);
        let xp_2h = calculate_offline_xp(7200, 0, 0, 0, 0.0);
        let xp_4h = calculate_offline_xp(14400, 0, 0, 0, 0.0);

        let ratio_2h_1h = xp_2h / xp_1h;
        let ratio_4h_2h = xp_4h / xp_2h;

        assert!(
            (ratio_2h_1h - 2.0).abs() < 0.01,
            "2h/1h XP ratio should be 2.0, got {:.4}",
            ratio_2h_1h
        );
        assert!(
            (ratio_4h_2h - 2.0).abs() < 0.01,
            "4h/2h XP ratio should be 2.0, got {:.4}",
            ratio_4h_2h
        );
    }

    // =========================================================================
    // Offline XP — Kill Estimation Formula Verification
    // =========================================================================

    #[test]
    fn test_offline_xp_kill_estimation_formula() {
        // Verify the exact formula: estimated_kills = (seconds / 5.0) * OFFLINE_MULTIPLIER
        // XP = estimated_kills * xp_per_tick_rate * avg_ticks_per_kill
        let seconds: i64 = 3600;
        let xp = calculate_offline_xp(seconds, 0, 0, 0, 0.0);

        let estimated_kills = (seconds as f64 / 5.0) * OFFLINE_MULTIPLIER;
        let xp_per_tick_rate = xp_gain_per_tick(0, 0, 0); // 1.0 at rank 0
        let avg_ticks_per_kill = (COMBAT_XP_MIN_TICKS + COMBAT_XP_MAX_TICKS) as f64 / 2.0; // 300
        let expected = estimated_kills * xp_per_tick_rate * avg_ticks_per_kill;

        assert!(
            (xp - expected).abs() < 0.001,
            "XP should match kill formula exactly: got {}, expected {}",
            xp,
            expected
        );
    }

    #[test]
    fn test_offline_multiplier_is_25_percent() {
        // The OFFLINE_MULTIPLIER constant should be 0.25 (25% rate)
        assert_eq!(
            OFFLINE_MULTIPLIER, 0.25,
            "OFFLINE_MULTIPLIER should be 0.25 (25%)"
        );
    }

    // =========================================================================
    // Offline XP — 7-Day Cap Exact Verification
    // =========================================================================

    #[test]
    fn test_offline_xp_cap_exact_boundary() {
        let seven_days = 7 * 24 * 3600;
        let seven_days_minus_one = seven_days - 1;
        let seven_days_plus_one = seven_days + 1;

        let xp_just_under = calculate_offline_xp(seven_days_minus_one, 0, 0, 0, 0.0);
        let xp_at_cap = calculate_offline_xp(seven_days, 0, 0, 0, 0.0);
        let xp_over_cap = calculate_offline_xp(seven_days_plus_one, 0, 0, 0, 0.0);

        // Just under should be slightly less than at cap
        assert!(
            xp_just_under < xp_at_cap,
            "XP just under 7 days ({}) should be less than XP at 7 days ({})",
            xp_just_under,
            xp_at_cap
        );

        // Over cap should equal exactly at cap (capped)
        assert!(
            (xp_at_cap - xp_over_cap).abs() < 0.001,
            "XP over 7 days ({}) should equal XP at 7 days ({})",
            xp_over_cap,
            xp_at_cap
        );
    }

    #[test]
    fn test_max_offline_seconds_is_seven_days() {
        assert_eq!(
            MAX_OFFLINE_SECONDS,
            7 * 24 * 60 * 60,
            "MAX_OFFLINE_SECONDS should be 7 days in seconds (604800)"
        );
    }

    // =========================================================================
    // Offline XP — Prestige Rank Scaling Tests
    // =========================================================================

    #[test]
    fn test_offline_xp_prestige_rank_scaling() {
        let xp_p0 = calculate_offline_xp(3600, 0, 0, 0, 0.0);
        let xp_p5 = calculate_offline_xp(3600, 5, 0, 0, 0.0);
        let xp_p10 = calculate_offline_xp(3600, 10, 0, 0, 0.0);
        let xp_p20 = calculate_offline_xp(3600, 20, 0, 0, 0.0);

        // Higher prestige should give more XP
        assert!(xp_p5 > xp_p0, "P5 ({}) > P0 ({})", xp_p5, xp_p0);
        assert!(xp_p10 > xp_p5, "P10 ({}) > P5 ({})", xp_p10, xp_p5);
        assert!(xp_p20 > xp_p10, "P20 ({}) > P10 ({})", xp_p20, xp_p10);

        // Verify ratios match prestige multiplier formula: 1 + 0.5 * rank^0.7
        let ratio_p5 = xp_p5 / xp_p0;
        let expected_p5 = 1.0 + 0.5 * 5.0_f64.powf(0.7); // ~2.585
        assert!(
            (ratio_p5 - expected_p5).abs() < 0.01,
            "P5/P0 ratio should be ~{:.3}, got {:.3}",
            expected_p5,
            ratio_p5
        );

        let ratio_p10 = xp_p10 / xp_p0;
        let expected_p10 = 1.0 + 0.5 * 10.0_f64.powf(0.7); // ~3.507
        assert!(
            (ratio_p10 - expected_p10).abs() < 0.01,
            "P10/P0 ratio should be ~{:.3}, got {:.3}",
            expected_p10,
            ratio_p10
        );

        let ratio_p20 = xp_p20 / xp_p0;
        let expected_p20 = 1.0 + 0.5 * 20.0_f64.powf(0.7); // ~5.075
        assert!(
            (ratio_p20 - expected_p20).abs() < 0.01,
            "P20/P0 ratio should be ~{:.3}, got {:.3}",
            expected_p20,
            ratio_p20
        );
    }

    // =========================================================================
    // Offline XP — Haven Bonus Scaling Tests
    // =========================================================================

    #[test]
    fn test_offline_xp_haven_bonus_scaling() {
        let base_xp = calculate_offline_xp(3600, 0, 0, 0, 0.0);
        let xp_50 = calculate_offline_xp(3600, 0, 0, 0, 50.0);
        let xp_100 = calculate_offline_xp(3600, 0, 0, 0, 100.0);

        // 50% bonus → 1.5x
        let ratio_50 = xp_50 / base_xp;
        assert!(
            (ratio_50 - 1.5).abs() < 0.01,
            "Haven +50% should give 1.5x XP, got {:.3}x",
            ratio_50
        );

        // 100% bonus → 2.0x
        let ratio_100 = xp_100 / base_xp;
        assert!(
            (ratio_100 - 2.0).abs() < 0.01,
            "Haven +100% should give 2.0x XP, got {:.3}x",
            ratio_100
        );
    }

    // =========================================================================
    // Offline XP — WIS and CHA Modifier Tests
    // =========================================================================

    #[test]
    fn test_offline_xp_cha_modifier_increases_xp() {
        let base_xp = calculate_offline_xp(3600, 0, 0, 0, 0.0);
        let cha_xp = calculate_offline_xp(3600, 0, 0, 3, 0.0); // CHA +3

        // CHA +3 → prestige_multiplier adds 0.3 → 1.3x at rank 0
        let ratio = cha_xp / base_xp;
        assert!(
            (ratio - 1.3).abs() < 0.01,
            "CHA +3 should give 1.3x XP, got {:.3}x",
            ratio
        );
    }

    #[test]
    fn test_offline_xp_wis_and_cha_combined() {
        let base_xp = calculate_offline_xp(3600, 0, 0, 0, 0.0);
        let combined_xp = calculate_offline_xp(3600, 0, 5, 3, 0.0); // WIS +5, CHA +3

        // WIS +5 → wis_mult = 1.25, CHA +3 → prestige_mult = 1.3
        // Combined: 1.25 * 1.3 = 1.625
        let ratio = combined_xp / base_xp;
        assert!(
            (ratio - 1.625).abs() < 0.01,
            "WIS +5 / CHA +3 combined should give 1.625x XP, got {:.3}x",
            ratio
        );
    }

    #[test]
    fn test_offline_xp_negative_wis_reduces_xp() {
        let base_xp = calculate_offline_xp(3600, 0, 0, 0, 0.0);
        let neg_wis_xp = calculate_offline_xp(3600, 0, -2, 0, 0.0); // WIS -2

        // WIS -2 → wis_mult = 1.0 + (-2 * 0.05) = 0.9
        let ratio = neg_wis_xp / base_xp;
        assert!(
            (ratio - 0.9).abs() < 0.01,
            "WIS -2 should give 0.9x XP, got {:.3}x",
            ratio
        );
    }

    // =========================================================================
    // Offline XP — Level-Up During Offline Progression
    // =========================================================================

    #[test]
    fn test_offline_progression_causes_level_ups() {
        let mut state = GameState::new("Offline Level Test".to_string(), 0);

        // Set last_save_time to 24 hours ago for significant XP gain
        state.last_save_time = chrono::Utc::now().timestamp() - (24 * 3600);

        let report = process_offline_progression(&mut state, 0.0);

        assert!(
            report.total_level_ups > 0,
            "24 hours offline should cause at least one level-up, got 0"
        );
        assert!(
            report.level_after > report.level_before,
            "Level should increase: before={}, after={}",
            report.level_before,
            report.level_after
        );
        assert_eq!(
            report.level_after - report.level_before,
            report.total_level_ups,
            "Level difference should equal total_level_ups"
        );
    }

    #[test]
    fn test_offline_progression_report_fields_consistent() {
        let mut state = GameState::new("Report Test".to_string(), 0);
        state.last_save_time = chrono::Utc::now().timestamp() - 3600;

        let report = process_offline_progression(&mut state, 25.0);

        assert!(report.elapsed_seconds > 0, "Should have positive elapsed");
        assert!(report.xp_gained > 0, "Should have gained XP");
        assert!(
            (report.haven_bonus_percent - 25.0).abs() < 0.01,
            "Haven bonus should be 25.0, got {}",
            report.haven_bonus_percent
        );
        // Offline rate: OFFLINE_MULTIPLIER * (1 + 25/100) * 100 = 0.25 * 1.25 * 100 = 31.25
        assert!(
            (report.offline_rate_percent - 31.25).abs() < 0.01,
            "Offline rate percent should be 31.25, got {}",
            report.offline_rate_percent
        );
    }

    // =========================================================================
    // Offline XP — Combined Modifiers Test
    // =========================================================================

    #[test]
    fn test_offline_xp_all_modifiers_combined() {
        // Test that all modifiers compose multiplicatively
        let base_xp = calculate_offline_xp(3600, 0, 0, 0, 0.0);
        // P5 + WIS +3 + CHA +2 + Haven 50%
        let full_xp = calculate_offline_xp(3600, 5, 3, 2, 50.0);

        // Expected multipliers:
        // prestige_mult(5, cha=2) = (1 + 0.5 * 5^0.7) + (2 * 0.1) = ~2.585 + 0.2 = ~2.785
        let prestige_mult = 1.0 + 0.5 * 5.0_f64.powf(0.7) + 0.2;
        // wis_mult = 1 + 3 * 0.05 = 1.15
        let wis_mult = 1.15;
        // haven_mult = 1 + 50/100 = 1.5
        let haven_mult = 1.5;

        let expected_ratio = prestige_mult * wis_mult * haven_mult;
        let actual_ratio = full_xp / base_xp;

        assert!(
            (actual_ratio - expected_ratio).abs() < 0.05,
            "Combined modifiers should give ~{:.3}x, got {:.3}x",
            expected_ratio,
            actual_ratio
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
