//! Integration tests for the deep::clock module.

use quest::deep::clock::{
    format_duration, is_mission_complete, mission_progress_percent, pending_events,
    time_remaining_secs,
};
use quest::deep::types::{
    ActiveMission, EventResolution, EventType, MissionEvent, MissionType,
};

// =========================================================================
// Helpers
// =========================================================================

fn make_mission(start_time: i64, duration_secs: u64) -> ActiveMission {
    ActiveMission {
        id: 1,
        mission_type: MissionType::Expedition,
        layer: 1,
        squad: vec![],
        start_time,
        duration_secs,
        cost: 50,
        events: vec![],
        events_resolved: 0,
    }
}

fn make_event(trigger_at_percent: u8, resolved: bool) -> MissionEvent {
    MissionEvent {
        trigger_at_percent,
        event_type: EventType::CaveIn,
        resolved,
        resolution: if resolved {
            Some(EventResolution::Safe)
        } else {
            None
        },
    }
}

// =========================================================================
// mission_progress_percent
// =========================================================================

#[test]
fn progress_zero_at_mission_start() {
    let m = make_mission(1000, 3600);
    assert_eq!(mission_progress_percent(&m, 1000), 0);
}

#[test]
fn progress_25_percent() {
    let m = make_mission(0, 4000);
    assert_eq!(mission_progress_percent(&m, 1000), 25);
}

#[test]
fn progress_50_percent() {
    let m = make_mission(0, 3600);
    assert_eq!(mission_progress_percent(&m, 1800), 50);
}

#[test]
fn progress_100_percent_exactly_at_end() {
    let m = make_mission(0, 3600);
    assert_eq!(mission_progress_percent(&m, 3600), 100);
}

#[test]
fn progress_clamped_to_100_when_over() {
    let m = make_mission(0, 3600);
    assert_eq!(mission_progress_percent(&m, 100_000), 100);
}

#[test]
fn progress_zero_when_now_before_start() {
    let m = make_mission(5000, 3600);
    assert_eq!(mission_progress_percent(&m, 1000), 0);
}

#[test]
fn progress_zero_when_now_equals_start() {
    let m = make_mission(5000, 3600);
    assert_eq!(mission_progress_percent(&m, 5000), 0);
}

#[test]
fn progress_100_when_duration_zero() {
    let m = make_mission(1000, 0);
    assert_eq!(mission_progress_percent(&m, 1000), 100);
}

// =========================================================================
// is_mission_complete
// =========================================================================

#[test]
fn complete_exactly_at_end_time() {
    let m = make_mission(1000, 3600);
    assert!(is_mission_complete(&m, 4600));
}

#[test]
fn not_complete_one_second_before_end() {
    let m = make_mission(1000, 3600);
    assert!(!is_mission_complete(&m, 4599));
}

#[test]
fn complete_well_after_end() {
    let m = make_mission(0, 60);
    assert!(is_mission_complete(&m, 999_999));
}

#[test]
fn not_complete_before_start() {
    let m = make_mission(10_000, 3600);
    assert!(!is_mission_complete(&m, 100));
}

#[test]
fn complete_zero_duration_at_start() {
    let m = make_mission(1000, 0);
    assert!(is_mission_complete(&m, 1000));
}

// =========================================================================
// pending_events
// =========================================================================

#[test]
fn no_pending_when_no_events_triggered() {
    let mut m = make_mission(0, 4000);
    m.events = vec![make_event(50, false)];
    // 0% progress — event at 50% not triggered yet
    assert_eq!(pending_events(&m, 0), Vec::<usize>::new());
}

#[test]
fn pending_event_returned_when_triggered() {
    let mut m = make_mission(0, 4000);
    m.events = vec![make_event(25, false)];
    // 50% progress — event at 25% has triggered
    assert_eq!(pending_events(&m, 2000), vec![0]);
}

#[test]
fn resolved_events_excluded_from_pending() {
    let mut m = make_mission(0, 4000);
    m.events = vec![make_event(25, true)];
    assert_eq!(pending_events(&m, 2000), Vec::<usize>::new());
}

#[test]
fn pending_returns_correct_subset() {
    let mut m = make_mission(0, 4000);
    m.events = vec![
        make_event(25, false), // 0 — triggered at 50% progress
        make_event(50, true),  // 1 — already resolved
        make_event(75, false), // 2 — not triggered at 50% progress
    ];
    // 50% progress
    assert_eq!(pending_events(&m, 2000), vec![0]);
}

#[test]
fn multiple_pending_events_all_returned() {
    let mut m = make_mission(0, 4000);
    m.events = vec![
        make_event(25, false), // triggered at 76% progress
        make_event(50, false), // triggered at 76% progress
        make_event(75, false), // triggered at 76% progress
    ];
    // 76% progress
    let result = pending_events(&m, 3040);
    assert_eq!(result, vec![0, 1, 2]);
}

// =========================================================================
// time_remaining_secs
// =========================================================================

#[test]
fn remaining_full_at_start() {
    let m = make_mission(1000, 3600);
    assert_eq!(time_remaining_secs(&m, 1000), 3600);
}

#[test]
fn remaining_half_duration() {
    let m = make_mission(0, 3600);
    assert_eq!(time_remaining_secs(&m, 1800), 1800);
}

#[test]
fn remaining_zero_at_completion() {
    let m = make_mission(0, 3600);
    assert_eq!(time_remaining_secs(&m, 3600), 0);
}

#[test]
fn remaining_zero_past_completion() {
    let m = make_mission(0, 3600);
    assert_eq!(time_remaining_secs(&m, 7200), 0);
}

#[test]
fn remaining_one_second_before_end() {
    let m = make_mission(0, 3600);
    assert_eq!(time_remaining_secs(&m, 3599), 1);
}

// =========================================================================
// format_duration
// =========================================================================

#[test]
fn format_zero_seconds_is_complete() {
    assert_eq!(format_duration(0), "Complete");
}

#[test]
fn format_45_minutes() {
    assert_eq!(format_duration(2700), "45m");
}

#[test]
fn format_2_hours_30_minutes() {
    assert_eq!(format_duration(9000), "2h 30m");
}

#[test]
fn format_exactly_1_hour() {
    assert_eq!(format_duration(3600), "1h");
}

#[test]
fn format_1_day_5_hours() {
    assert_eq!(format_duration(104_400), "1d 5h");
}

#[test]
fn format_exactly_1_day() {
    assert_eq!(format_duration(86_400), "1d");
}

#[test]
fn format_2_days_no_hours() {
    assert_eq!(format_duration(172_800), "2d");
}

#[test]
fn format_less_than_one_minute_rounds_up() {
    assert_eq!(format_duration(30), "1m");
}

#[test]
fn format_exactly_one_minute() {
    assert_eq!(format_duration(60), "1m");
}

#[test]
fn format_large_duration_multi_day() {
    // 2 days + 3 hours = 183_600 seconds
    assert_eq!(format_duration(183_600), "2d 3h");
}
