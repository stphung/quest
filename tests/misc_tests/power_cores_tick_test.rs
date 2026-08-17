//! Integration tests for Power Cores tick processing and PR granting.
//!
//! Tests cover:
//! 1. Core does NOT grant PR before fill_duration elapsed
//! 2. Core grants +1 PR when fill_duration elapsed
//! 3. Timer resets after granting
//! 4. Multiple cores grant independently
//! 5. Offline catchup: 48h offline + 2 PR/day → +4 PR
//! 6. Offline catchup: 24h offline with all 6 cores → correct total PR
//! 7. Partial progress preserved (18h into 24h cycle = 75%)
//! 8. Only unlocked cores process (locked = no effect)
//! 9. prestige_rank incremented on grant
//! 10. TickEvent::PowerCoreGranted emitted
//! 11. Rapid ticks don't double-grant
//! 12. Newly unlocked core starts from current time

use chrono::Utc;
use quest::achievements::{AchievementId, Achievements};
use quest::core::game_state::GameState;
use quest::core::tick_types::{TickEvent, TickResult};
use quest::deep::DeepState;
use quest::power_cores::{
    apply_offline_power_cores, fill_duration_secs, init_new_core, tick_power_cores, ALL_POWER_CORES,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn make_state() -> GameState {
    GameState::new("PowerCoreTest".to_string(), 0)
}

/// Unlock a single PowerCoreI achievement (Red Fault, 2 PR/day).
fn ach_layer3() -> Achievements {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::PowerCoreI, None);
    ach
}

/// Unlock all six layer achievements.
fn ach_all_cores() -> Achievements {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::PowerCoreI, None);
    ach.unlock(AchievementId::PowerCoreII, None);
    ach.unlock(AchievementId::PowerCoreIII, None);
    ach.unlock(AchievementId::PowerCoreIV, None);
    ach.unlock(AchievementId::PowerCoreV, None);
    ach.unlock(AchievementId::PowerCoreVI, None);
    ach
}

fn now() -> i64 {
    Utc::now().timestamp()
}

fn count_power_core_granted_events(events: &[TickEvent]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, TickEvent::PowerCoreGranted { .. }))
        .count()
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 1: Core does NOT grant PR before fill_duration elapsed
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn core_does_not_grant_before_fill_duration() {
    let mut state = make_state();
    let mut deep = DeepState::new();
    let achievements = ach_layer3();
    let mut result = TickResult::default();

    // Set last_granted_at to just now — zero elapsed time.
    deep.persistent
        .power_core_last_granted
        .insert(AchievementId::PowerCoreI, now());

    let pr_before = state.prestige_rank;
    tick_power_cores(&mut state, &mut deep, &achievements, &mut result);

    assert_eq!(
        state.prestige_rank, pr_before,
        "prestige_rank must not change before fill duration elapses"
    );
    assert_eq!(
        count_power_core_granted_events(&result.events),
        0,
        "no PowerCoreGranted events should be emitted"
    );
    assert!(
        !result.deep_changed,
        "deep_changed must be false when nothing is granted"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2: Core grants exactly +1 PR when fill_duration elapsed
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn core_grants_one_pr_when_fill_duration_elapsed() {
    let mut state = make_state();
    let mut deep = DeepState::new();
    let achievements = ach_layer3(); // Red Fault: 2 PR/day = 43200s fill
    let mut result = TickResult::default();

    let fill = fill_duration_secs(2); // 43200s
                                      // Simulate exactly one fill duration + 1 second of elapsed time.
    deep.persistent
        .power_core_last_granted
        .insert(AchievementId::PowerCoreI, now() - fill - 1);

    let pr_before = state.prestige_rank;
    tick_power_cores(&mut state, &mut deep, &achievements, &mut result);

    assert_eq!(
        state.prestige_rank,
        pr_before + 1,
        "prestige_rank should increase by exactly 1"
    );
    assert_eq!(
        count_power_core_granted_events(&result.events),
        1,
        "exactly one PowerCoreGranted event expected"
    );
    assert!(
        result.deep_changed,
        "deep_changed must be set after a grant"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 3: Timer resets after granting
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn timer_resets_after_granting() {
    let mut state = make_state();
    let mut deep = DeepState::new();
    let achievements = ach_layer3();
    let mut result = TickResult::default();

    let fill = fill_duration_secs(2);
    let old_timestamp = now() - fill - 1;
    deep.persistent
        .power_core_last_granted
        .insert(AchievementId::PowerCoreI, old_timestamp);

    tick_power_cores(&mut state, &mut deep, &achievements, &mut result);

    // The timestamp must have advanced by fill_secs from the old value.
    let new_timestamp = deep
        .persistent
        .power_core_last_granted
        .get(&AchievementId::PowerCoreI)
        .copied()
        .expect("timestamp should be present after grant");

    assert!(
        new_timestamp > old_timestamp,
        "last_granted_at must advance after a grant"
    );
    // new_timestamp should equal old_timestamp + fill (1 completed cycle).
    assert_eq!(
        new_timestamp,
        old_timestamp + fill,
        "timestamp must advance by exactly one fill duration"
    );

    // A second tick immediately after should NOT grant again.
    let mut result2 = TickResult::default();
    let pr_after_first = state.prestige_rank;
    tick_power_cores(&mut state, &mut deep, &achievements, &mut result2);
    assert_eq!(
        state.prestige_rank, pr_after_first,
        "no grant should occur immediately after reset"
    );
    assert_eq!(
        count_power_core_granted_events(&result2.events),
        0,
        "no events on re-tick"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 4: Multiple cores grant independently
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn multiple_cores_grant_independently() {
    let mut state = make_state();
    let mut deep = DeepState::new();

    // Unlock two cores: Layer3 (2 PR/day) and Layer7 (3 PR/day).
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::PowerCoreI, None);
    achievements.unlock(AchievementId::PowerCoreII, None);

    let mut result = TickResult::default();

    let fill_layer3 = fill_duration_secs(2);
    let fill_layer7 = fill_duration_secs(3);

    // Layer3: 1 full cycle elapsed → should grant 1 PR.
    deep.persistent
        .power_core_last_granted
        .insert(AchievementId::PowerCoreI, now() - fill_layer3 - 1);

    // Layer7: 1 full cycle elapsed → should grant 1 PR.
    deep.persistent
        .power_core_last_granted
        .insert(AchievementId::PowerCoreII, now() - fill_layer7 - 1);

    let pr_before = state.prestige_rank;
    tick_power_cores(&mut state, &mut deep, &achievements, &mut result);

    // Each core grants 1 PR independently → total +2.
    assert_eq!(
        state.prestige_rank,
        pr_before + 2,
        "each unlocked core should grant 1 PR independently"
    );
    assert_eq!(
        count_power_core_granted_events(&result.events),
        2,
        "two PowerCoreGranted events expected"
    );

    // Layer12 is NOT unlocked — its timestamp should not affect anything.
    assert!(!deep
        .persistent
        .power_core_last_granted
        .contains_key(&AchievementId::PowerCoreIII));
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 5: Offline catchup: 48h offline + 1 PR/day core → +2 PR
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn offline_catchup_48h_one_core_grants_four_pr() {
    let mut state = make_state();
    let mut deep = DeepState::new();
    let achievements = ach_layer3(); // Red Fault: 2 PR/day

    let fill = fill_duration_secs(2); // 43200s
    let elapsed_48h: i64 = 48 * 3600; // 172800s = exactly 4 fill cycles
    let old_ts = now() - elapsed_48h - 1;

    deep.persistent
        .power_core_last_granted
        .insert(AchievementId::PowerCoreI, old_ts);

    let pr_before = state.prestige_rank;
    let granted = apply_offline_power_cores(&mut state, &mut deep, &achievements);

    assert_eq!(
        granted, 4,
        "48h offline with 2 PR/day should grant exactly 4 PR"
    );
    assert_eq!(
        state.prestige_rank,
        pr_before + 4,
        "prestige_rank should reflect offline grant"
    );

    // Timestamp should have advanced by 4 fill durations.
    let new_ts = deep.persistent.power_core_last_granted[&AchievementId::PowerCoreI];
    let expected_advance = fill * 4;
    assert_eq!(
        new_ts,
        old_ts + expected_advance,
        "timestamp must advance by number of completed cycles * fill_secs"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 6: Offline catchup: 24h offline with all 6 cores → correct total PR
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn offline_catchup_24h_all_six_cores_correct_total() {
    let mut state = make_state();
    let mut deep = DeepState::new();
    let achievements = ach_all_cores();

    // 24h offline: each core completes floor(86400 / fill_duration) cycles.
    // Layer3: 2 PR/day → 2 cycles in 24h
    // Layer7: 3 PR/day → 3 cycles in 24h
    // Layer12: 5 PR/day → 5 cycles
    // Layer18: 8 PR/day → 8 cycles
    // Layer25: 12 PR/day → 12 cycles
    // Layer30: 18 PR/day → 18 cycles
    // Total: 2+3+5+8+12+18 = 48 PR
    let elapsed_24h: i64 = 86400 + 60; // 24h + 60s to pass all fill durations

    for def in ALL_POWER_CORES {
        deep.persistent
            .power_core_last_granted
            .insert(def.achievement_id, now() - elapsed_24h);
    }

    let pr_before = state.prestige_rank;
    let granted = apply_offline_power_cores(&mut state, &mut deep, &achievements);

    // With 24h+60s elapsed and fill durations of 86400/N:
    // Layer3 (43200s fill): floor(86460 / 43200) = 2 cycles
    // Layer7 (28800s fill): floor(86460 / 28800) = 3 cycles
    // Layer12 (17280s fill): floor(86460 / 17280) = 5 cycles
    // Layer18 (10800s fill): floor(86460 / 10800) = 8 cycles
    // Layer25 (7200s fill): floor(86460 / 7200) = 12 cycles
    // Layer30 (4800s fill): floor(86460 / 4800) = 18 cycles
    let expected_pr = 2 + 3 + 5 + 8 + 12 + 18;
    assert_eq!(
        granted, expected_pr,
        "24h offline with all 6 cores should grant {expected_pr} PR total"
    );
    assert_eq!(
        state.prestige_rank,
        pr_before + expected_pr,
        "prestige_rank should reflect all offline grants"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 7: Partial progress preserved (18h into 24h cycle = 75%)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn partial_progress_preserved_no_early_grant() {
    let mut state = make_state();
    let mut deep = DeepState::new();
    let achievements = ach_layer3(); // Red Fault: 2 PR/day, 43200s fill
    let mut result = TickResult::default();

    let fill = fill_duration_secs(2); // 43200s
    let elapsed_18h: i64 = 9 * 3600; // 32400s — 75% of fill, NOT yet complete

    assert!(elapsed_18h < fill, "sanity check: 9h < 12h fill");

    deep.persistent
        .power_core_last_granted
        .insert(AchievementId::PowerCoreI, now() - elapsed_18h);

    let pr_before = state.prestige_rank;
    tick_power_cores(&mut state, &mut deep, &achievements, &mut result);

    assert_eq!(
        state.prestige_rank, pr_before,
        "no PR should be granted at 75% fill progress (9h of 12h)"
    );
    assert_eq!(
        count_power_core_granted_events(&result.events),
        0,
        "no PowerCoreGranted event at 75% fill (9h of 12h)"
    );

    // The last_granted_at timestamp must remain unchanged (progress is preserved).
    let ts = deep.persistent.power_core_last_granted[&AchievementId::PowerCoreI];
    let expected_ts = now() - elapsed_18h;
    // Allow ±5 seconds of clock drift in the test.
    assert!(
        (ts - expected_ts).abs() <= 5,
        "partial progress timestamp must not change (got {ts}, expected ~{expected_ts})"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 8: Only unlocked cores process (locked cores = no effect)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn locked_cores_do_not_grant() {
    let mut state = make_state();
    let mut deep = DeepState::new();
    let achievements = Achievements::default(); // nothing unlocked
    let mut result = TickResult::default();

    // Pre-seed all cores with old timestamps.
    let far_past = now() - 86400 * 30; // 30 days ago
    for def in ALL_POWER_CORES {
        deep.persistent
            .power_core_last_granted
            .insert(def.achievement_id, far_past);
    }

    let pr_before = state.prestige_rank;
    tick_power_cores(&mut state, &mut deep, &achievements, &mut result);

    assert_eq!(
        state.prestige_rank, pr_before,
        "locked cores must not grant any PR"
    );
    assert_eq!(
        count_power_core_granted_events(&result.events),
        0,
        "no PowerCoreGranted events from locked cores"
    );
    assert!(
        !result.deep_changed,
        "deep_changed must remain false when nothing is processed"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 9: prestige_rank is incremented on grant
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn prestige_rank_incremented_on_grant() {
    let mut state = make_state();
    state.prestige_rank = 100; // Start at an arbitrary non-zero rank.
    let mut deep = DeepState::new();
    let achievements = ach_layer3();
    let mut result = TickResult::default();

    let fill = fill_duration_secs(2);
    deep.persistent
        .power_core_last_granted
        .insert(AchievementId::PowerCoreI, now() - fill - 1);

    tick_power_cores(&mut state, &mut deep, &achievements, &mut result);

    assert_eq!(
        state.prestige_rank, 101,
        "prestige_rank must increment from 100 to 101 after one grant"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 10: TickEvent::PowerCoreGranted is emitted with correct data
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn tick_event_power_core_granted_emitted_with_correct_name() {
    let mut state = make_state();
    let mut deep = DeepState::new();
    let achievements = ach_layer3(); // Red Fault
    let mut result = TickResult::default();

    let fill = fill_duration_secs(2);
    deep.persistent
        .power_core_last_granted
        .insert(AchievementId::PowerCoreI, now() - fill - 1);

    tick_power_cores(&mut state, &mut deep, &achievements, &mut result);

    assert_eq!(result.events.len(), 1);
    match &result.events[0] {
        TickEvent::PowerCoreGranted { core_name } => {
            assert_eq!(*core_name, "Red Fault", "core_name must be 'Red Fault'");
        }
        other => panic!("expected PowerCoreGranted, got {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 11: Rapid successive ticks don't double-grant
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn rapid_successive_ticks_do_not_double_grant() {
    let mut state = make_state();
    let mut deep = DeepState::new();
    let achievements = ach_layer3();

    let fill = fill_duration_secs(2);
    // Exactly one cycle has elapsed.
    deep.persistent
        .power_core_last_granted
        .insert(AchievementId::PowerCoreI, now() - fill - 1);

    // First tick: should grant 1 PR.
    let mut result1 = TickResult::default();
    tick_power_cores(&mut state, &mut deep, &achievements, &mut result1);
    let pr_after_first = state.prestige_rank;
    assert_eq!(count_power_core_granted_events(&result1.events), 1);

    // Immediately fire a second tick.
    let mut result2 = TickResult::default();
    tick_power_cores(&mut state, &mut deep, &achievements, &mut result2);

    assert_eq!(
        state.prestige_rank, pr_after_first,
        "second rapid tick must not double-grant"
    );
    assert_eq!(
        count_power_core_granted_events(&result2.events),
        0,
        "no PowerCoreGranted on second rapid tick"
    );

    // Fire 10 more rapid ticks for good measure.
    for _ in 0..10 {
        let mut r = TickResult::default();
        tick_power_cores(&mut state, &mut deep, &achievements, &mut r);
        assert_eq!(
            state.prestige_rank, pr_after_first,
            "rapid ticks must never double-grant"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 12: Newly unlocked core starts from current time (no retroactive grant)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn newly_unlocked_core_starts_from_current_time() {
    let mut state = make_state();
    let mut deep = DeepState::new(); // no entries — simulates brand-new core unlock
    let achievements = ach_layer3();
    let mut result = TickResult::default();

    // last_granted_at for PowerCoreI is missing (timestamp = 0 / absent).
    assert!(!deep
        .persistent
        .power_core_last_granted
        .contains_key(&AchievementId::PowerCoreI));

    let pr_before = state.prestige_rank;
    let time_before = now();
    tick_power_cores(&mut state, &mut deep, &achievements, &mut result);
    let time_after = now();

    // No PR should be granted on first tick — it just sets the start time.
    assert_eq!(
        state.prestige_rank, pr_before,
        "no PR should be granted on first tick (initialises timer)"
    );
    assert_eq!(
        count_power_core_granted_events(&result.events),
        0,
        "no PowerCoreGranted on initialisation tick"
    );

    // The timestamp should now be set to approximately now.
    let ts = deep
        .persistent
        .power_core_last_granted
        .get(&AchievementId::PowerCoreI)
        .copied()
        .expect("timestamp must be set after first tick");

    assert!(
        ts >= time_before && ts <= time_after + 1,
        "initial timestamp must be set to current time (got {ts}, expected {time_before}..{time_after})"
    );

    // deep_changed should be true because we initialised the timestamp.
    assert!(
        result.deep_changed,
        "deep_changed must be true after initialising a new core"
    );

    // After the core is initialised with `init_new_core`, the same behaviour holds.
    let mut deep2 = DeepState::new();
    init_new_core(&mut deep2, AchievementId::PowerCoreII);

    let ts2 = deep2
        .persistent
        .power_core_last_granted
        .get(&AchievementId::PowerCoreII)
        .copied()
        .expect("init_new_core must set timestamp");

    // Allow ±5 seconds of clock drift.
    assert!(
        (ts2 - now()).abs() <= 5,
        "init_new_core must set timestamp to current time"
    );
}
