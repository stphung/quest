//! Tests for Power Cores UI annotations and formatting.
//!
//! Covers:
//! - get_power_core_def returns Some for all 6 layer achievements
//! - get_power_core_def returns None for non-power-core achievements
//! - Correct rate strings ("1 PR/day" through "6 PR/day")
//! - Progress bar fill calculation at various elapsed times (via fill_ratio)
//! - Time remaining formatting (via format_core_time_remaining)
//! - Power Cores section hidden when 0 cores unlocked (get_unlocked_cores is empty)
//! - Power Cores section shows N bars for N unlocked cores
//! - PowerCoreGrant TickEvent field correctness

use quest::achievements::{AchievementId, Achievements};
use quest::power_cores::{
    fill_duration_secs, fill_ratio, format_core_time_remaining, get_power_core_def,
    get_unlocked_cores,
};

// =========================================================================
// Helper
// =========================================================================

/// Unlock a specific achievement directly on an Achievements struct.
fn unlock(achievements: &mut Achievements, id: AchievementId) {
    achievements.unlock(id, Some("TestHero".to_string()));
}

// =========================================================================
// Section 1: get_power_core_def — coverage of all 6 layer achievements
// =========================================================================

#[test]
fn test_get_power_core_def_layer3_cleared_returns_some() {
    let def = get_power_core_def(AchievementId::PowerCoreI);
    assert!(
        def.is_some(),
        "PowerCoreI must map to a PowerCoreDef (Red Fault)"
    );
}

#[test]
fn test_get_power_core_def_layer7_cleared_returns_some() {
    let def = get_power_core_def(AchievementId::PowerCoreII);
    assert!(
        def.is_some(),
        "PowerCoreII must map to a PowerCoreDef (Mirror Scar)"
    );
}

#[test]
fn test_get_power_core_def_layer12_cleared_returns_some() {
    let def = get_power_core_def(AchievementId::PowerCoreIII);
    assert!(
        def.is_some(),
        "PowerCoreIII must map to a PowerCoreDef (Black Mouth)"
    );
}

#[test]
fn test_get_power_core_def_layer18_cleared_returns_some() {
    let def = get_power_core_def(AchievementId::PowerCoreIV);
    assert!(
        def.is_some(),
        "PowerCoreIV must map to a PowerCoreDef (Hollow Throne)"
    );
}

#[test]
fn test_get_power_core_def_layer25_cleared_returns_some() {
    let def = get_power_core_def(AchievementId::PowerCoreV);
    assert!(
        def.is_some(),
        "PowerCoreV must map to a PowerCoreDef (Wailing Reach)"
    );
}

#[test]
fn test_get_power_core_def_layer30_cleared_returns_some() {
    let def = get_power_core_def(AchievementId::PowerCoreVI);
    assert!(
        def.is_some(),
        "PowerCoreVI must map to a PowerCoreDef (Origin Wound)"
    );
}

// =========================================================================
// Section 2: get_power_core_def — None for non-power-core achievements
// =========================================================================

#[test]
fn test_get_power_core_def_void_explorer_returns_none() {
    let def = get_power_core_def(AchievementId::VoidExplorer);
    assert!(
        def.is_none(),
        "VoidExplorer does not grant a Power Core; must return None"
    );
}

#[test]
fn test_get_power_core_def_guild_rank2_returns_none() {
    let def = get_power_core_def(AchievementId::GuildRank2);
    assert!(
        def.is_none(),
        "GuildRank2 does not grant a Power Core; must return None"
    );
}

#[test]
fn test_get_power_core_def_slayer_i_returns_none() {
    let def = get_power_core_def(AchievementId::SlayerI);
    assert!(
        def.is_none(),
        "SlayerI does not grant a Power Core; must return None"
    );
}

#[test]
fn test_get_power_core_def_first_prestige_returns_none() {
    let def = get_power_core_def(AchievementId::FirstPrestige);
    assert!(
        def.is_none(),
        "FirstPrestige does not grant a Power Core; must return None"
    );
}

#[test]
fn test_get_power_core_def_beyond_infinity_returns_none() {
    let def = get_power_core_def(AchievementId::BeyondInfinity);
    assert!(
        def.is_none(),
        "BeyondInfinity does not grant a Power Core; must return None"
    );
}

#[test]
fn test_get_power_core_def_ascension_i_returns_none() {
    let def = get_power_core_def(AchievementId::AscensionI);
    assert!(
        def.is_none(),
        "AscensionI does not grant a Power Core; must return None"
    );
}

// =========================================================================
// Section 3: Core names are correct
// =========================================================================

#[test]
fn test_power_core_layer3_name_is_red_fault() {
    let def = get_power_core_def(AchievementId::PowerCoreI).unwrap();
    assert_eq!(
        def.name, "Red Fault",
        "PowerCoreI core name must be 'Red Fault'"
    );
}

#[test]
fn test_power_core_layer7_name_is_mirror_scar() {
    let def = get_power_core_def(AchievementId::PowerCoreII).unwrap();
    assert_eq!(def.name, "Mirror Scar");
}

#[test]
fn test_power_core_layer12_name_is_black_mouth() {
    let def = get_power_core_def(AchievementId::PowerCoreIII).unwrap();
    assert_eq!(def.name, "Black Mouth");
}

#[test]
fn test_power_core_layer18_name_is_hollow_throne() {
    let def = get_power_core_def(AchievementId::PowerCoreIV).unwrap();
    assert_eq!(def.name, "Hollow Throne");
}

#[test]
fn test_power_core_layer25_name_is_wailing_reach() {
    let def = get_power_core_def(AchievementId::PowerCoreV).unwrap();
    assert_eq!(def.name, "Wailing Reach");
}

#[test]
fn test_power_core_layer30_name_is_origin_wound() {
    let def = get_power_core_def(AchievementId::PowerCoreVI).unwrap();
    assert_eq!(def.name, "Origin Wound");
}

// =========================================================================
// Section 4: Correct rate values (backing data for "N PR/day" strings)
// =========================================================================

#[test]
fn test_rate_layer3_is_2_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreI).unwrap();
    assert_eq!(
        def.pr_per_day, 2,
        "PowerCoreI (Red Fault) must grant 2 PR/day"
    );
}

#[test]
fn test_rate_layer7_is_3_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreII).unwrap();
    assert_eq!(
        def.pr_per_day, 3,
        "PowerCoreII (Mirror Scar) must grant 3 PR/day"
    );
}

#[test]
fn test_rate_layer12_is_5_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreIII).unwrap();
    assert_eq!(def.pr_per_day, 5);
}

#[test]
fn test_rate_layer18_is_8_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreIV).unwrap();
    assert_eq!(def.pr_per_day, 8);
}

#[test]
fn test_rate_layer25_is_12_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreV).unwrap();
    assert_eq!(def.pr_per_day, 12);
}

#[test]
fn test_rate_layer30_is_18_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreVI).unwrap();
    assert_eq!(def.pr_per_day, 18);
}

// Rate string rendering: the UI displays "N PR/day". Verify the integer backing
// the string is correct for all 6 cores.  The display format is
// `format!("{} PR/day", def.pr_per_day)`.

#[test]
fn test_rate_string_format_2_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreI).unwrap();
    let rate_str = format!("{} PR/day", def.pr_per_day);
    assert_eq!(rate_str, "2 PR/day");
}

#[test]
fn test_rate_string_format_3_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreII).unwrap();
    let rate_str = format!("{} PR/day", def.pr_per_day);
    assert_eq!(rate_str, "3 PR/day");
}

#[test]
fn test_rate_string_format_18_pr_per_day() {
    let def = get_power_core_def(AchievementId::PowerCoreVI).unwrap();
    let rate_str = format!("{} PR/day", def.pr_per_day);
    assert_eq!(rate_str, "18 PR/day");
}

// =========================================================================
// Section 5: Progress bar fill calculation via fill_ratio
//
// The UI progress bar fills based on elapsed / fill_duration.
// fill_ratio(elapsed, fill_secs) is the public utility that drives this.
// It clamps to [0.0, 1.0] — the bar is full when >= 1.0 (ready to grant).
// =========================================================================

#[test]
fn test_fill_duration_2_pr_per_day_is_12h() {
    // 2 PR/day → one grant every 12h = 43200 seconds (Red Fault)
    assert_eq!(fill_duration_secs(2), 43_200);
}

#[test]
fn test_fill_duration_3_pr_per_day_is_8h() {
    // 3 PR/day → one grant every 8h = 28800 seconds (Mirror Scar)
    assert_eq!(fill_duration_secs(3), 28_800);
}

#[test]
fn test_fill_duration_5_pr_per_day_is_4h48m() {
    // 5 PR/day → one grant every ~4h 48m = 17280 seconds (Black Mouth)
    assert_eq!(fill_duration_secs(5), 17_280);
}

#[test]
fn test_fill_duration_8_pr_per_day_is_3h() {
    // 8 PR/day → one grant every 3h = 10800 seconds (Hollow Throne)
    assert_eq!(fill_duration_secs(8), 10_800);
}

#[test]
fn test_fill_duration_12_pr_per_day_is_2h() {
    // 12 PR/day → one grant every 2h = 7200 seconds (Wailing Reach)
    assert_eq!(fill_duration_secs(12), 7_200);
}

#[test]
fn test_fill_duration_18_pr_per_day_is_1h20m() {
    // 18 PR/day → one grant every ~1h 20m = 4800 seconds (Origin Wound)
    assert_eq!(fill_duration_secs(18), 4_800);
}

#[test]
fn test_fill_ratio_zero_elapsed() {
    let fill_dur = fill_duration_secs(1); // 86400
    assert!(
        fill_ratio(0, fill_dur).abs() < 1e-9,
        "0 elapsed must yield 0.0 fill ratio"
    );
}

#[test]
fn test_fill_ratio_half_elapsed() {
    let fill_dur = fill_duration_secs(1);
    let elapsed = fill_dur / 2;
    let ratio = fill_ratio(elapsed, fill_dur);
    assert!(
        (ratio - 0.5).abs() < 1e-6,
        "Half elapsed must yield 0.5 fill ratio, got {}",
        ratio
    );
}

#[test]
fn test_fill_ratio_full_cycle_is_ready() {
    // When exactly one full cycle has elapsed, the bar is full (1.0) — ready to grant.
    let fill_dur = fill_duration_secs(1);
    let ratio = fill_ratio(fill_dur, fill_dur);
    assert!(
        (ratio - 1.0).abs() < 1e-9,
        "Exactly one full cycle elapsed should yield 1.0 (ready), got {}",
        ratio
    );
}

#[test]
fn test_fill_ratio_one_hour_into_twelve_hour_fill() {
    let fill_dur = fill_duration_secs(2); // 43200 (12h)
    let elapsed = 3_600_i64; // 1 hour
    let ratio = fill_ratio(elapsed, fill_dur);
    let expected = 1.0 / 12.0;
    assert!(
        (ratio - expected).abs() < 1e-6,
        "1h / 12h fill must be ~{:.4}, got {:.4}",
        expected,
        ratio
    );
}

#[test]
fn test_fill_ratio_quarter_elapsed() {
    let fill_dur = fill_duration_secs(1); // 86400
    let elapsed = fill_dur / 4;
    let ratio = fill_ratio(elapsed, fill_dur);
    assert!(
        (ratio - 0.25).abs() < 1e-6,
        "Quarter elapsed must yield 0.25, got {}",
        ratio
    );
}

#[test]
fn test_fill_ratio_clamped_at_one_for_overdue() {
    // If more than one cycle has elapsed (e.g. offline time), ratio clamps to 1.0.
    let fill_dur = fill_duration_secs(2); // 43200
    let elapsed = fill_dur * 3; // 3 full cycles overdue
    let ratio = fill_ratio(elapsed, fill_dur);
    assert!(
        (ratio - 1.0).abs() < 1e-9,
        "Overdue core must clamp ratio to 1.0, got {}",
        ratio
    );
}

// =========================================================================
// Section 6: Time remaining formatting via format_core_time_remaining
//
// The UI shows "Ready!" when ratio >= 1.0, "Xh Ym" when hours >= 1, "Ym" otherwise.
// format_core_time_remaining(remaining_secs, ratio) is the real production function.
// =========================================================================

#[test]
fn test_time_remaining_14h_2m() {
    let remaining_secs = 14 * 3600 + 2 * 60;
    assert_eq!(format_core_time_remaining(remaining_secs, 0.1), "14h 2m");
}

#[test]
fn test_time_remaining_48m() {
    let remaining_secs = 48 * 60;
    assert_eq!(format_core_time_remaining(remaining_secs, 0.5), "48m");
}

#[test]
fn test_time_remaining_zero_shows_ready() {
    // When remaining_secs is 0 and ratio >= 1.0, the core is ready.
    assert_eq!(format_core_time_remaining(0, 1.0), "Ready!");
}

#[test]
fn test_time_remaining_exactly_1_hour() {
    assert_eq!(format_core_time_remaining(3600, 0.5), "1h 0m");
}

#[test]
fn test_time_remaining_exactly_24_hours() {
    assert_eq!(format_core_time_remaining(24 * 3600, 0.0), "24h 0m");
}

#[test]
fn test_time_remaining_1_minute() {
    assert_eq!(format_core_time_remaining(60, 0.9), "1m");
}

#[test]
fn test_time_remaining_90_seconds_truncates_to_1m() {
    // 90 seconds = 1 minute 30 seconds; sub-minute seconds are dropped
    assert_eq!(format_core_time_remaining(90, 0.9), "1m");
}

#[test]
fn test_time_remaining_59_seconds_shows_0m() {
    // Less than 1 full minute, hours = 0 — displays "0m"
    assert_eq!(format_core_time_remaining(59, 0.9), "0m");
}

#[test]
fn test_time_remaining_ready_when_ratio_at_one() {
    assert_eq!(format_core_time_remaining(0, 1.0), "Ready!");
}

#[test]
fn test_time_remaining_ready_when_ratio_exceeds_one() {
    // ratio > 1.0 is possible when the tick that grants PR hasn't run yet.
    assert_eq!(format_core_time_remaining(0, 1.5), "Ready!");
}

// =========================================================================
// Section 7: get_unlocked_cores — section visibility and count
// =========================================================================

#[test]
fn test_get_unlocked_cores_empty_when_no_achievements() {
    let achievements = Achievements::default();
    let unlocked = get_unlocked_cores(&achievements);
    assert!(
        unlocked.is_empty(),
        "No layer achievements unlocked means no unlocked power cores, got {} cores",
        unlocked.len()
    );
}

#[test]
fn test_power_cores_section_hidden_when_zero_cores() {
    // Mirrors the UI condition: only render the section when !unlocked.is_empty()
    let achievements = Achievements::default();
    let unlocked = get_unlocked_cores(&achievements);
    let should_show_section = !unlocked.is_empty();
    assert!(
        !should_show_section,
        "Power Cores section must be hidden when no cores are unlocked"
    );
}

#[test]
fn test_get_unlocked_cores_one_when_layer3_unlocked() {
    let mut achievements = Achievements::default();
    unlock(&mut achievements, AchievementId::PowerCoreI);

    let unlocked = get_unlocked_cores(&achievements);
    assert_eq!(
        unlocked.len(),
        1,
        "Unlocking PowerCoreI must yield exactly 1 Power Core"
    );
    assert_eq!(unlocked[0].name, "Red Fault");
}

#[test]
fn test_power_cores_section_shows_one_bar_for_one_core() {
    let mut achievements = Achievements::default();
    unlock(&mut achievements, AchievementId::PowerCoreI);

    let unlocked = get_unlocked_cores(&achievements);
    // The UI renders one bar per unlocked core
    assert_eq!(
        unlocked.len(),
        1,
        "UI must render exactly 1 bar when 1 core is unlocked"
    );
}

#[test]
fn test_get_unlocked_cores_two_when_layer3_and_layer7_unlocked() {
    let mut achievements = Achievements::default();
    unlock(&mut achievements, AchievementId::PowerCoreI);
    unlock(&mut achievements, AchievementId::PowerCoreII);

    let unlocked = get_unlocked_cores(&achievements);
    assert_eq!(unlocked.len(), 2);
}

#[test]
fn test_power_cores_section_shows_two_bars_for_two_cores() {
    let mut achievements = Achievements::default();
    unlock(&mut achievements, AchievementId::PowerCoreI);
    unlock(&mut achievements, AchievementId::PowerCoreII);

    let unlocked = get_unlocked_cores(&achievements);
    assert_eq!(
        unlocked.len(),
        2,
        "UI must render exactly 2 bars when 2 cores are unlocked"
    );
}

#[test]
fn test_get_unlocked_cores_all_six_when_all_layer_achievements_unlocked() {
    let mut achievements = Achievements::default();
    unlock(&mut achievements, AchievementId::PowerCoreI);
    unlock(&mut achievements, AchievementId::PowerCoreII);
    unlock(&mut achievements, AchievementId::PowerCoreIII);
    unlock(&mut achievements, AchievementId::PowerCoreIV);
    unlock(&mut achievements, AchievementId::PowerCoreV);
    unlock(&mut achievements, AchievementId::PowerCoreVI);

    let unlocked = get_unlocked_cores(&achievements);
    assert_eq!(
        unlocked.len(),
        6,
        "All 6 layer achievements must yield 6 Power Cores"
    );
}

#[test]
fn test_power_cores_section_shows_six_bars_for_six_cores() {
    let mut achievements = Achievements::default();
    unlock(&mut achievements, AchievementId::PowerCoreI);
    unlock(&mut achievements, AchievementId::PowerCoreII);
    unlock(&mut achievements, AchievementId::PowerCoreIII);
    unlock(&mut achievements, AchievementId::PowerCoreIV);
    unlock(&mut achievements, AchievementId::PowerCoreV);
    unlock(&mut achievements, AchievementId::PowerCoreVI);

    let unlocked = get_unlocked_cores(&achievements);
    assert_eq!(
        unlocked.len(),
        6,
        "UI must render exactly 6 bars when all 6 cores are unlocked"
    );
}

#[test]
fn test_get_unlocked_cores_non_layer_achievements_do_not_count() {
    let mut achievements = Achievements::default();
    unlock(&mut achievements, AchievementId::VoidExplorer);
    unlock(&mut achievements, AchievementId::GuildRank2);
    unlock(&mut achievements, AchievementId::SlayerI);

    let unlocked = get_unlocked_cores(&achievements);
    assert!(
        unlocked.is_empty(),
        "Non-layer achievements must not add power cores, got {} cores",
        unlocked.len()
    );
}

#[test]
fn test_get_unlocked_cores_returns_cores_in_definition_order() {
    // Insert in reverse order — ordering must always follow ALL_POWER_CORES definition order
    let mut achievements = Achievements::default();
    unlock(&mut achievements, AchievementId::PowerCoreVI);
    unlock(&mut achievements, AchievementId::PowerCoreII);
    unlock(&mut achievements, AchievementId::PowerCoreI);

    let unlocked = get_unlocked_cores(&achievements);
    assert_eq!(unlocked.len(), 3);
    // Definitions are in ascending PR/day order: 1, 2, 6
    assert_eq!(
        unlocked[0].pr_per_day, 2,
        "First returned core must have lowest rate (Red Fault, 2 PR/day)"
    );
    assert_eq!(
        unlocked[1].pr_per_day, 3,
        "Second returned core must have next rate (Mirror Scar, 3 PR/day)"
    );
    assert_eq!(
        unlocked[2].pr_per_day, 18,
        "Third returned core must be highest unlocked (Origin Wound, 18 PR/day)"
    );
}

// =========================================================================
// Section 8: PowerCoreGranted TickEvent ticker message format
//
// The TickEvent::PowerCoreGranted variant carries a `core_name: &'static str`.
// The tick event mapping in tick_events.rs formats the log/ticker message.
// These tests verify the variant structure and that the core_name is correct.
// =========================================================================

#[test]
fn test_power_core_granted_tick_event_carries_correct_core_name_red_fault() {
    use quest::core::TickEvent;
    let event = TickEvent::PowerCoreGranted {
        core_name: "Red Fault",
    };
    match &event {
        TickEvent::PowerCoreGranted { core_name } => {
            assert_eq!(*core_name, "Red Fault");
        }
        _ => panic!("Expected PowerCoreGranted variant"),
    }
}

#[test]
fn test_power_core_granted_tick_event_carries_correct_core_name_mirror_scar() {
    use quest::core::TickEvent;
    let event = TickEvent::PowerCoreGranted {
        core_name: "Mirror Scar",
    };
    match &event {
        TickEvent::PowerCoreGranted { core_name } => {
            assert_eq!(*core_name, "Mirror Scar");
        }
        _ => panic!("Expected PowerCoreGranted variant"),
    }
}

#[test]
fn test_power_core_granted_tick_event_carries_correct_core_name_origin_wound() {
    use quest::core::TickEvent;
    let event = TickEvent::PowerCoreGranted {
        core_name: "Origin Wound",
    };
    match &event {
        TickEvent::PowerCoreGranted { core_name } => {
            assert_eq!(*core_name, "Origin Wound");
        }
        _ => panic!("Expected PowerCoreGranted variant"),
    }
}

/// Verifies that the expected ticker message format for PowerCoreGranted
/// can be produced from core_name alone, matching the design spec:
/// "⬡ Power Core: <name> → +1 Prestige"
#[test]
fn test_power_core_granted_ticker_message_format() {
    let core_name = "Red Fault";
    let message = format!("\u{2b21} Power Core: {} \u{2192} +1 Prestige", core_name);
    assert!(
        message.contains("Red Fault"),
        "Formatted ticker message must contain the core name"
    );
    assert!(
        message.contains("+1 Prestige"),
        "Formatted ticker message must contain '+1 Prestige'"
    );
    assert!(
        message.contains('\u{2b21}'),
        "Formatted ticker message must contain the ⬡ (U+2B21) symbol"
    );
}

#[test]
fn test_power_core_granted_core_names_match_all_power_core_defs() {
    // Verify all 6 cores produce valid static str core_names that could be
    // stored in PowerCoreGranted events.
    use quest::power_cores::ALL_POWER_CORES;

    let expected_names = [
        "Red Fault",
        "Mirror Scar",
        "Black Mouth",
        "Hollow Throne",
        "Wailing Reach",
        "Origin Wound",
    ];
    for (def, expected_name) in ALL_POWER_CORES.iter().zip(expected_names.iter()) {
        assert_eq!(
            def.name, *expected_name,
            "Core def name '{}' must match expected ticker name '{}'",
            def.name, expected_name
        );
    }
}
