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
use quest::power_cores::{
    apply_offline_power_cores, fill_duration_secs, init_new_core, tick_power_cores, GeneratorTimer,
    PassivesState, ALL_POWER_CORES,
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

fn insert_timer(passives: &mut PassivesState, key: &str, last_granted_at: i64) {
    passives.generators.insert(
        key.to_string(),
        GeneratorTimer { last_granted_at },
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 1: Core does NOT grant PR before fill_duration elapsed
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn core_does_not_grant_before_fill_duration() {
    let mut state = make_state();
    let mut passives = PassivesState::default();
    let achievements = ach_layer3();
    let mut result = TickResult::default();

    insert_timer(&mut passives, "power_core_1", now());

    let pr_before = state.prestige_rank;
    tick_power_cores(&mut state, &mut passives, &achievements, &mut result);

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
        !result.passives_changed,
        "passives_changed must be false when nothing is granted"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2: Core grants exactly +1 PR when fill_duration elapsed
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn core_grants_one_pr_when_fill_duration_elapsed() {
    let mut state = make_state();
    let mut passives = PassivesState::default();
    let achievements = ach_layer3();
    let mut result = TickResult::default();

    let fill = fill_duration_secs(2); // 43200s
    insert_timer(&mut passives, "power_core_1", now() - fill - 1);

    let pr_before = state.prestige_rank;
    tick_power_cores(&mut state, &mut passives, &achievements, &mut result);

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
        result.passives_changed,
        "passives_changed must be set after a grant"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 3: Timer resets after granting
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn timer_resets_after_granting() {
    let mut state = make_state();
    let mut passives = PassivesState::default();
    let achievements = ach_layer3();
    let mut result = TickResult::default();

    let fill = fill_duration_secs(2);
    let old_timestamp = now() - fill - 1;
    insert_timer(&mut passives, "power_core_1", old_timestamp);

    tick_power_cores(&mut state, &mut passives, &achievements, &mut result);

    let new_timestamp = passives
        .generators
        .get("power_core_1")
        .map(|t| t.last_granted_at)
        .expect("timestamp should be present after grant");

    assert!(
        new_timestamp > old_timestamp,
        "last_granted_at must advance after a grant"
    );
    assert_eq!(
        new_timestamp,
        old_timestamp + fill,
        "timestamp must advance by exactly one fill duration"
    );

    // A second tick immediately after should NOT grant again.
    let mut result2 = TickResult::default();
    let pr_after_first = state.prestige_rank;
    tick_power_cores(&mut state, &mut passives, &achievements, &mut result2);
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
    let mut passives = PassivesState::default();

    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::PowerCoreI, None);
    achievements.unlock(AchievementId::PowerCoreII, None);

    let mut result = TickResult::default();

    let fill_layer3 = fill_duration_secs(2);
    let fill_layer7 = fill_duration_secs(3);

    insert_timer(&mut passives, "power_core_1", now() - fill_layer3 - 1);
    insert_timer(&mut passives, "power_core_2", now() - fill_layer7 - 1);

    let pr_before = state.prestige_rank;
    tick_power_cores(&mut state, &mut passives, &achievements, &mut result);

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

    assert!(!passives.generators.contains_key("power_core_3"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 5: Offline catchup: 48h offline + 1 PR/day core → +2 PR
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn offline_catchup_48h_one_core_grants_four_pr() {
    let mut state = make_state();
    let mut passives = PassivesState::default();
    let achievements = ach_layer3();

    let fill = fill_duration_secs(2);
    let elapsed_48h: i64 = 48 * 3600;

    insert_timer(&mut passives, "power_core_1", now() - elapsed_48h - 1);

    let pr_before = state.prestige_rank;
    let granted = apply_offline_power_cores(&mut state, &mut passives, &achievements);

    assert_eq!(
        granted, 4,
        "48h offline with 2 PR/day should grant exactly 4 PR"
    );
    assert_eq!(
        state.prestige_rank,
        pr_before + 4,
        "prestige_rank should reflect offline grant"
    );

    let new_ts = passives.generators["power_core_1"].last_granted_at;
    let expected_advance = fill * 4;
    let old_ts = now() - elapsed_48h - 1;
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
    let mut passives = PassivesState::default();
    let achievements = ach_all_cores();

    let elapsed_24h: i64 = 86400 + 60;

    for def in ALL_POWER_CORES {
        insert_timer(&mut passives, def.key, now() - elapsed_24h);
    }

    let pr_before = state.prestige_rank;
    let granted = apply_offline_power_cores(&mut state, &mut passives, &achievements);

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
    let mut passives = PassivesState::default();
    let achievements = ach_layer3();
    let mut result = TickResult::default();

    let fill = fill_duration_secs(2);
    let elapsed_9h: i64 = 9 * 3600;

    assert!(elapsed_9h < fill, "sanity check: 9h < 12h fill");

    insert_timer(&mut passives, "power_core_1", now() - elapsed_9h);

    let pr_before = state.prestige_rank;
    tick_power_cores(&mut state, &mut passives, &achievements, &mut result);

    assert_eq!(
        state.prestige_rank, pr_before,
        "no PR should be granted at 75% fill progress (9h of 12h)"
    );
    assert_eq!(
        count_power_core_granted_events(&result.events),
        0,
        "no PowerCoreGranted event at 75% fill (9h of 12h)"
    );

    let ts = passives.generators["power_core_1"].last_granted_at;
    let expected_ts = now() - elapsed_9h;
    assert!(
        (ts - expected_ts).abs() <= 2,
        "partial progress timestamp must not change (got {ts}, expected ~{expected_ts})"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 8: Only unlocked cores process (locked cores = no effect)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn locked_cores_do_not_grant() {
    let mut state = make_state();
    let mut passives = PassivesState::default();
    let achievements = Achievements::default(); // nothing unlocked
    let mut result = TickResult::default();

    let far_past = now() - 86400 * 30;
    for def in ALL_POWER_CORES {
        insert_timer(&mut passives, def.key, far_past);
    }

    let pr_before = state.prestige_rank;
    tick_power_cores(&mut state, &mut passives, &achievements, &mut result);

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
        !result.passives_changed,
        "passives_changed must remain false when nothing is processed"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 9: prestige_rank is incremented on grant
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn prestige_rank_incremented_on_grant() {
    let mut state = make_state();
    state.prestige_rank = 100;
    let mut passives = PassivesState::default();
    let achievements = ach_layer3();
    let mut result = TickResult::default();

    let fill = fill_duration_secs(2);
    insert_timer(&mut passives, "power_core_1", now() - fill - 1);

    tick_power_cores(&mut state, &mut passives, &achievements, &mut result);

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
    let mut passives = PassivesState::default();
    let achievements = ach_layer3();
    let mut result = TickResult::default();

    let fill = fill_duration_secs(2);
    insert_timer(&mut passives, "power_core_1", now() - fill - 1);

    tick_power_cores(&mut state, &mut passives, &achievements, &mut result);

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
    let mut passives = PassivesState::default();
    let achievements = ach_layer3();

    let fill = fill_duration_secs(2);
    insert_timer(&mut passives, "power_core_1", now() - fill - 1);

    let mut result1 = TickResult::default();
    tick_power_cores(&mut state, &mut passives, &achievements, &mut result1);
    let pr_after_first = state.prestige_rank;
    assert_eq!(count_power_core_granted_events(&result1.events), 1);

    let mut result2 = TickResult::default();
    tick_power_cores(&mut state, &mut passives, &achievements, &mut result2);

    assert_eq!(
        state.prestige_rank, pr_after_first,
        "second rapid tick must not double-grant"
    );
    assert_eq!(
        count_power_core_granted_events(&result2.events),
        0,
        "no PowerCoreGranted on second rapid tick"
    );

    for _ in 0..10 {
        let mut r = TickResult::default();
        tick_power_cores(&mut state, &mut passives, &achievements, &mut r);
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
    let mut passives = PassivesState::default();
    let achievements = ach_layer3();
    let mut result = TickResult::default();

    assert!(!passives.generators.contains_key("power_core_1"));

    let pr_before = state.prestige_rank;
    let time_before = now();
    tick_power_cores(&mut state, &mut passives, &achievements, &mut result);
    let time_after = now();

    assert_eq!(
        state.prestige_rank, pr_before,
        "no PR should be granted on first tick (initialises timer)"
    );
    assert_eq!(
        count_power_core_granted_events(&result.events),
        0,
        "no PowerCoreGranted on initialisation tick"
    );

    let ts = passives
        .generators
        .get("power_core_1")
        .map(|t| t.last_granted_at)
        .expect("timestamp must be set after first tick");

    assert!(
        ts >= time_before && ts <= time_after + 1,
        "initial timestamp must be set to current time (got {ts}, expected {time_before}..{time_after})"
    );

    assert!(
        result.passives_changed,
        "passives_changed must be true after initialising a new core"
    );

    // After the core is initialised with `init_new_core`, the same behaviour holds.
    let mut passives2 = PassivesState::default();
    init_new_core(&mut passives2, "power_core_2");

    let ts2 = passives2
        .generators
        .get("power_core_2")
        .map(|t| t.last_granted_at)
        .expect("init_new_core must set timestamp");

    assert!(
        (ts2 - now()).abs() <= 2,
        "init_new_core must set timestamp to current time"
    );
}
