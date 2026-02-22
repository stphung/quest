//! Wall-clock time utilities for The Deep mission system.
//!
//! All functions accept an injected `now_unix: i64` Unix timestamp so that
//! mission logic remains deterministically testable without calling any system
//! clock functions.

use crate::deep::types::{ActiveMission, MissionEvent};

// =========================================================================
// Progress
// =========================================================================

/// Returns how far through a mission we are, as a percent in 0..=100.
///
/// - Negative elapsed (clock skew) → 0
/// - Elapsed >= duration → 100
pub fn mission_progress_percent(mission: &ActiveMission, now_unix: i64) -> u8 {
    let duration = mission.duration_secs as i64;
    if duration == 0 {
        return 100;
    }
    let elapsed = now_unix - mission.start_time;
    if elapsed <= 0 {
        return 0;
    }
    let progress = (elapsed * 100) / duration;
    progress.clamp(0, 100) as u8
}

// =========================================================================
// Completion
// =========================================================================

/// Returns `true` if the mission has completed (wall-clock duration elapsed).
pub fn is_mission_complete(mission: &ActiveMission, now_unix: i64) -> bool {
    let end = mission.start_time.saturating_add(mission.duration_secs as i64);
    now_unix >= end
}

// =========================================================================
// Pending events
// =========================================================================

/// Returns the indices of events that have triggered but are not yet resolved.
///
/// An event triggers when `mission_progress_percent >= event.trigger_at_percent`.
pub fn pending_events(mission: &ActiveMission, now_unix: i64) -> Vec<usize> {
    let progress = mission_progress_percent(mission, now_unix);
    mission
        .events
        .iter()
        .enumerate()
        .filter(|(_, e): &(usize, &MissionEvent)| {
            !e.resolved && progress >= e.trigger_at_percent
        })
        .map(|(i, _)| i)
        .collect()
}

// =========================================================================
// Time remaining
// =========================================================================

/// Returns seconds remaining until the mission completes, or 0 if already done.
pub fn time_remaining_secs(mission: &ActiveMission, now_unix: i64) -> u64 {
    let end = mission.start_time.saturating_add(mission.duration_secs as i64);
    if now_unix >= end {
        0
    } else {
        (end - now_unix) as u64
    }
}

// =========================================================================
// Duration formatting
// =========================================================================

/// Formats a duration in seconds as a human-readable string.
///
/// Examples:
/// - 0 → "Complete"
/// - 2700 → "45m"
/// - 9000 → "2h 30m"
/// - 104400 → "1d 5h"
pub fn format_duration(secs: u64) -> String {
    if secs == 0 {
        return "Complete".to_string();
    }

    let days = secs / 86_400;
    let remaining_after_days = secs % 86_400;
    let hours = remaining_after_days / 3_600;
    let remaining_after_hours = remaining_after_days % 3_600;
    let minutes = remaining_after_hours / 60;

    if days > 0 {
        if hours > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{}d", days)
        }
    } else if hours > 0 {
        if minutes > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}h", hours)
        }
    } else {
        format!("{}m", minutes.max(1))
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep::types::{ActiveMission, EventResolution, EventType, MissionEvent, MissionType};

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

    // -- mission_progress_percent --

    #[test]
    fn progress_at_start_is_zero() {
        let m = make_mission(1000, 3600);
        assert_eq!(mission_progress_percent(&m, 1000), 0);
    }

    #[test]
    fn progress_at_50_percent() {
        let m = make_mission(1000, 3600);
        assert_eq!(mission_progress_percent(&m, 1000 + 1800), 50);
    }

    #[test]
    fn progress_at_100_percent() {
        let m = make_mission(1000, 3600);
        assert_eq!(mission_progress_percent(&m, 1000 + 3600), 100);
    }

    #[test]
    fn progress_over_100_clamps_to_100() {
        let m = make_mission(1000, 3600);
        assert_eq!(mission_progress_percent(&m, 1000 + 7200), 100);
    }

    #[test]
    fn progress_negative_elapsed_returns_zero() {
        let m = make_mission(5000, 3600);
        // now_unix is before start_time
        assert_eq!(mission_progress_percent(&m, 1000), 0);
    }

    #[test]
    fn progress_zero_duration_returns_100() {
        let m = make_mission(1000, 0);
        assert_eq!(mission_progress_percent(&m, 1000), 100);
    }

    // -- is_mission_complete --

    #[test]
    fn complete_exactly_at_end() {
        let m = make_mission(1000, 3600);
        assert!(is_mission_complete(&m, 1000 + 3600));
    }

    #[test]
    fn not_complete_one_second_before_end() {
        let m = make_mission(1000, 3600);
        assert!(!is_mission_complete(&m, 1000 + 3599));
    }

    #[test]
    fn complete_well_after_end() {
        let m = make_mission(0, 3600);
        assert!(is_mission_complete(&m, 100_000));
    }

    #[test]
    fn not_complete_before_start() {
        let m = make_mission(5000, 3600);
        assert!(!is_mission_complete(&m, 1000));
    }

    // -- pending_events --

    #[test]
    fn pending_events_none_when_not_triggered() {
        let mut m = make_mission(0, 3600);
        // Event at 50%, mission at 0%
        m.events = vec![make_event(50, false)];
        let now = 0; // 0% progress
        assert_eq!(pending_events(&m, now), vec![] as Vec<usize>);
    }

    #[test]
    fn pending_events_returns_index_when_triggered() {
        let mut m = make_mission(0, 3600);
        m.events = vec![make_event(25, false)];
        let now = 1800; // 50% progress — event at 25% should trigger
        assert_eq!(pending_events(&m, now), vec![0]);
    }

    #[test]
    fn pending_events_excludes_resolved() {
        let mut m = make_mission(0, 3600);
        m.events = vec![make_event(25, true)]; // already resolved
        let now = 1800;
        assert_eq!(pending_events(&m, now), vec![] as Vec<usize>);
    }

    #[test]
    fn pending_events_multiple_events() {
        let mut m = make_mission(0, 4000);
        // Three events: 25%, 50%, 75%
        m.events = vec![
            make_event(25, false),  // index 0 — should trigger at 50% progress
            make_event(50, true),   // index 1 — resolved, should NOT appear
            make_event(75, false),  // index 2 — not triggered yet
        ];
        let now = 2000; // 50% progress
        assert_eq!(pending_events(&m, now), vec![0]);
    }

    // -- time_remaining_secs --

    #[test]
    fn time_remaining_half_done() {
        let m = make_mission(0, 3600);
        assert_eq!(time_remaining_secs(&m, 1800), 1800);
    }

    #[test]
    fn time_remaining_zero_when_complete() {
        let m = make_mission(0, 3600);
        assert_eq!(time_remaining_secs(&m, 3600), 0);
    }

    #[test]
    fn time_remaining_zero_when_past_end() {
        let m = make_mission(0, 3600);
        assert_eq!(time_remaining_secs(&m, 7200), 0);
    }

    #[test]
    fn time_remaining_full_duration_at_start() {
        let m = make_mission(1000, 3600);
        assert_eq!(time_remaining_secs(&m, 1000), 3600);
    }

    // -- format_duration --

    #[test]
    fn format_zero_is_complete() {
        assert_eq!(format_duration(0), "Complete");
    }

    #[test]
    fn format_minutes_only() {
        assert_eq!(format_duration(2700), "45m");
    }

    #[test]
    fn format_hours_and_minutes() {
        assert_eq!(format_duration(9000), "2h 30m");
    }

    #[test]
    fn format_hours_only() {
        assert_eq!(format_duration(7200), "2h");
    }

    #[test]
    fn format_one_day_and_hours() {
        assert_eq!(format_duration(104400), "1d 5h");
    }

    #[test]
    fn format_days_only() {
        assert_eq!(format_duration(86400), "1d");
    }

    #[test]
    fn format_less_than_one_minute() {
        // rounds up to 1m
        assert_eq!(format_duration(30), "1m");
    }
}
