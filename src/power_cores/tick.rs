//! Tick processing and offline catchup for Power Cores.
//!
//! Each tick, for every unlocked core, we check whether wall-clock elapsed
//! time since `last_granted_at` has reached `fill_duration_secs()`.  If so,
//! we grant +1 prestige rank and reset the timestamp.  Multiple completed
//! cycles are each counted individually (batched offline catchup).

use super::types::{fill_duration_secs, get_unlocked_cores};
use crate::achievements::Achievements;
use crate::core::game_state::GameState;
use crate::core::tick_types::{TickEvent, TickResult};
use crate::deep::DeepState;
use chrono::Utc;

/// Process one game tick for all Power Cores.
///
/// For each unlocked core whose fill timer has elapsed, this grants +1 PR to
/// `state.prestige_rank`, emits a [`TickEvent::PowerCoreGranted`], and
/// updates `last_granted_at`.
///
/// Sets `result.deep_changed` when any grant occurs so that the
/// caller knows to persist Deep state to disk.
pub fn tick_power_cores(
    state: &mut GameState,
    deep: &mut DeepState,
    achievements: &Achievements,
    result: &mut TickResult,
) {
    let now = Utc::now().timestamp();
    let unlocked = get_unlocked_cores(achievements);

    for def in unlocked {
        let fill_secs = fill_duration_secs(def.pr_per_day);
        let last = deep
            .persistent
            .power_core_last_granted
            .get(&def.achievement_id)
            .copied()
            .unwrap_or(0);

        // last == 0 means the core was just unlocked (or the save is brand-new).
        // In that case, initialise last_granted_at to now so the first cycle
        // starts from the moment of unlock.
        if last == 0 {
            deep.persistent
                .power_core_last_granted
                .insert(def.achievement_id, now);
            result.deep_changed = true;
            continue;
        }

        let elapsed = now - last;
        if elapsed < fill_secs {
            continue;
        }

        // Grant one PR per completed fill cycle.
        let completed_cycles = (elapsed / fill_secs) as u32;
        let pr_to_grant = completed_cycles; // 1 PR per cycle by design

        state.prestige_rank = state.prestige_rank.saturating_add(pr_to_grant);
        // Recalculate prestige bonuses; mark derived stats dirty so Stage 3
        // of the next tick recalculates them with the correct enhancement levels.
        state.recalculate_prestige_bonuses();
        state.derived_stats_dirty = true;

        // Advance last_granted_at by exactly the cycles that completed.
        let new_last = last + fill_secs * completed_cycles as i64;
        deep.persistent
            .power_core_last_granted
            .insert(def.achievement_id, new_last);

        // Emit one event per cycle so the log/ticker reflects each grant.
        for _ in 0..completed_cycles {
            result.events.push(TickEvent::PowerCoreGranted {
                core_name: def.name,
            });
        }

        result.deep_changed = true;
    }
}

/// Process offline catchup for Power Cores when a character is loaded.
///
/// Calculates completed cycles per core while the game was closed and grants
/// accumulated PR to `state`.  Unlike the per-tick function this does NOT
/// emit `TickEvent`s — the offline report (or a log entry) is the caller's
/// responsibility.
///
/// Returns the total prestige ranks granted, or 0 if none.
pub fn apply_offline_power_cores(
    state: &mut GameState,
    deep: &mut DeepState,
    achievements: &Achievements,
) -> u32 {
    let now = Utc::now().timestamp();
    let unlocked = get_unlocked_cores(achievements);
    let mut total_pr_granted = 0u32;

    for def in unlocked {
        let fill_secs = fill_duration_secs(def.pr_per_day);
        let last = deep
            .persistent
            .power_core_last_granted
            .get(&def.achievement_id)
            .copied()
            .unwrap_or(0);

        if last == 0 {
            // Core was never initialised — set to now, no offline grant.
            deep.persistent
                .power_core_last_granted
                .insert(def.achievement_id, now);
            continue;
        }

        let elapsed = now - last;
        if elapsed < fill_secs {
            continue;
        }

        let completed_cycles = (elapsed / fill_secs) as u32;
        let pr_to_grant = completed_cycles;

        state.prestige_rank = state.prestige_rank.saturating_add(pr_to_grant);
        total_pr_granted += pr_to_grant;

        let new_last = last + fill_secs * completed_cycles as i64;
        deep.persistent
            .power_core_last_granted
            .insert(def.achievement_id, new_last);
    }

    if total_pr_granted > 0 {
        state.recalculate_prestige_bonuses();
        state.derived_stats_dirty = true;
    }

    total_pr_granted
}

/// Initialise a newly unlocked core's timestamp so its first cycle starts
/// from the moment of unlock rather than the epoch.
///
/// Call this immediately after unlocking a core's achievement.
#[allow(dead_code)]
pub fn init_new_core(deep: &mut DeepState, achievement_id: crate::achievements::AchievementId) {
    let now = Utc::now().timestamp();
    // Only initialise if not already set (idempotent).
    deep.persistent
        .power_core_last_granted
        .entry(achievement_id)
        .or_insert(now);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::{AchievementId, Achievements};
    use crate::core::game_state::GameState;
    use crate::core::tick_types::TickResult;
    use crate::power_cores::types::ALL_POWER_CORES;

    fn unlocked_achievements() -> Achievements {
        let mut ach = Achievements::default();
        // Manually unlock the PowerCoreI achievement so one core is active.
        ach.unlock(AchievementId::PowerCoreI, None);
        ach
    }

    #[test]
    fn no_grant_when_fill_timer_not_elapsed() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut deep = DeepState::default();
        let achievements = unlocked_achievements();
        let mut result = TickResult::default();

        let now = Utc::now().timestamp();
        // Last granted just now — no elapsed time.
        deep.persistent
            .power_core_last_granted
            .insert(AchievementId::PowerCoreI, now);

        let pr_before = state.prestige_rank;
        tick_power_cores(&mut state, &mut deep, &achievements, &mut result);

        assert_eq!(state.prestige_rank, pr_before);
        assert!(!result.deep_changed);
        assert!(result.events.is_empty());
    }

    #[test]
    fn grants_one_pr_after_fill_duration_elapses() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut deep = DeepState::default();
        let achievements = unlocked_achievements();
        let mut result = TickResult::default();

        let now = Utc::now().timestamp();
        let fill = fill_duration_secs(2); // 43200s for Red Fault (2 PR/day)
                                          // Simulate fill duration + 1 second of elapsed time.
        deep.persistent
            .power_core_last_granted
            .insert(AchievementId::PowerCoreI, now - fill - 1);

        let pr_before = state.prestige_rank;
        tick_power_cores(&mut state, &mut deep, &achievements, &mut result);

        assert_eq!(state.prestige_rank, pr_before + 1);
        assert!(result.deep_changed);
        assert_eq!(result.events.len(), 1);
        assert!(matches!(
            result.events[0],
            TickEvent::PowerCoreGranted {
                core_name: "Red Fault"
            }
        ));
    }

    #[test]
    fn grants_multiple_pr_for_multiple_elapsed_cycles() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut deep = DeepState::default();
        let achievements = unlocked_achievements();
        let mut result = TickResult::default();

        let now = Utc::now().timestamp();
        let fill = fill_duration_secs(2);
        // Simulate 3 complete cycles.
        deep.persistent
            .power_core_last_granted
            .insert(AchievementId::PowerCoreI, now - fill * 3 - 1);

        let pr_before = state.prestige_rank;
        tick_power_cores(&mut state, &mut deep, &achievements, &mut result);

        assert_eq!(state.prestige_rank, pr_before + 3);
        assert_eq!(result.events.len(), 3);
    }

    #[test]
    fn zero_timestamp_initialises_without_granting() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut deep = DeepState::default(); // all timestamps missing (0)
        let achievements = unlocked_achievements();
        let mut result = TickResult::default();

        let pr_before = state.prestige_rank;
        tick_power_cores(&mut state, &mut deep, &achievements, &mut result);

        // No PR granted — just initialised.
        assert_eq!(state.prestige_rank, pr_before);
        // The timestamp should now be set to approximately now.
        assert!(deep
            .persistent
            .power_core_last_granted
            .contains_key(&AchievementId::PowerCoreI));
        assert!(result.deep_changed);
    }

    #[test]
    fn offline_catchup_grants_pr() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut deep = DeepState::default();
        let achievements = unlocked_achievements();

        let now = Utc::now().timestamp();
        let fill = fill_duration_secs(2);
        deep.persistent
            .power_core_last_granted
            .insert(AchievementId::PowerCoreI, now - fill * 2 - 1);

        let pr_before = state.prestige_rank;
        let granted = apply_offline_power_cores(&mut state, &mut deep, &achievements);

        assert_eq!(granted, 2);
        assert_eq!(state.prestige_rank, pr_before + 2);
    }

    #[test]
    fn no_grant_for_locked_cores() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut deep = DeepState::default();
        let achievements = Achievements::default(); // nothing unlocked
        let mut result = TickResult::default();

        // Set timestamps far in the past for all cores.
        for def in ALL_POWER_CORES {
            deep.persistent
                .power_core_last_granted
                .insert(def.achievement_id, Utc::now().timestamp() - 86400 * 10);
        }

        let pr_before = state.prestige_rank;
        tick_power_cores(&mut state, &mut deep, &achievements, &mut result);

        // Achievements are all locked, so nothing should grant.
        assert_eq!(state.prestige_rank, pr_before);
        assert!(!result.deep_changed);
    }
}
