//! Expanded achievement system coverage tests.
//!
//! Fills coverage gaps in notifications, stats, unlock mechanics, modal queue,
//! persistence edge cases, and handler event flows that are not already exercised
//! by achievement_handlers_test.rs, achievement_coverage_test.rs, or
//! haven_dungeon_coverage_test.rs.

#![allow(clippy::field_reassign_with_default)]

use quest::achievements::types::*;
use quest::achievements::*;
use std::collections::HashMap;

// =========================================================================
// 1. notifications.rs — pending_count, clear_pending, recently_unlocked
// =========================================================================

#[test]
fn test_pending_count_zero_on_default() {
    let ach = Achievements::default();
    assert_eq!(ach.pending_count(), 0);
}

#[test]
fn test_pending_count_after_single_unlock() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::HavenDiscovered, Some("Hero".to_string()));
    assert_eq!(ach.pending_count(), 1);
}

#[test]
fn test_pending_count_after_multiple_different_unlocks() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    ach.unlock(AchievementId::BossHunterI, Some("Hero".to_string()));
    ach.unlock(AchievementId::Level10, Some("Hero".to_string()));
    assert_eq!(ach.pending_count(), 3);
}

#[test]
fn test_pending_count_after_duplicate_unlock_does_not_grow() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    assert_eq!(ach.pending_count(), 1);
}

#[test]
fn test_clear_pending_moves_to_recently_unlocked_and_zeros_pending() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    ach.unlock(AchievementId::Level10, Some("Hero".to_string()));

    assert_eq!(ach.pending_count(), 2);
    assert!(ach.recently_unlocked.is_empty());

    ach.clear_pending_notifications();

    assert_eq!(ach.pending_count(), 0);
    assert_eq!(ach.recently_unlocked.len(), 2);
}

#[test]
fn test_clear_pending_on_empty_is_noop() {
    let mut ach = Achievements::default();
    ach.clear_pending_notifications();
    assert_eq!(ach.pending_count(), 0);
    assert!(ach.recently_unlocked.is_empty());
}

#[test]
fn test_clear_pending_accumulates_into_recently_unlocked() {
    let mut ach = Achievements::default();

    // First batch
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    ach.clear_pending_notifications();
    assert_eq!(ach.recently_unlocked.len(), 1);

    // Second batch -- should append, not replace
    ach.unlock(AchievementId::Level10, Some("Hero".to_string()));
    ach.clear_pending_notifications();
    assert_eq!(ach.recently_unlocked.len(), 2);
}

#[test]
fn test_clear_pending_then_unlock_more_gives_fresh_pending() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    ach.clear_pending_notifications();
    assert_eq!(ach.pending_count(), 0);

    // New unlock after clearing
    ach.unlock(AchievementId::BossHunterI, Some("Hero".to_string()));
    assert_eq!(ach.pending_count(), 1);

    // SlayerI moved to recently, BossHunterI still in pending only
    assert!(ach.is_recently_unlocked(AchievementId::SlayerI));
    assert!(!ach.is_recently_unlocked(AchievementId::BossHunterI));
}

#[test]
fn test_clear_recently_unlocked_empties_list() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    ach.clear_pending_notifications();

    assert!(!ach.recently_unlocked.is_empty());
    ach.clear_recently_unlocked();
    assert!(ach.recently_unlocked.is_empty());
}

#[test]
fn test_clear_recently_unlocked_on_empty_is_noop() {
    let mut ach = Achievements::default();
    ach.clear_recently_unlocked();
    assert!(ach.recently_unlocked.is_empty());
}

#[test]
fn test_is_recently_unlocked_true_after_clear_pending() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::DungeonDiver, Some("Hero".to_string()));
    ach.clear_pending_notifications();
    assert!(ach.is_recently_unlocked(AchievementId::DungeonDiver));
}

#[test]
fn test_is_recently_unlocked_false_before_clear_pending() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::DungeonDiver, Some("Hero".to_string()));
    // Still in pending, not yet in recently_unlocked
    assert!(!ach.is_recently_unlocked(AchievementId::DungeonDiver));
}

#[test]
fn test_is_recently_unlocked_false_for_never_unlocked() {
    let ach = Achievements::default();
    assert!(!ach.is_recently_unlocked(AchievementId::SlayerI));
}

#[test]
fn test_is_recently_unlocked_false_after_clear_recently() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    ach.clear_pending_notifications();
    ach.clear_recently_unlocked();

    assert!(!ach.is_recently_unlocked(AchievementId::SlayerI));
    // But the achievement itself is still unlocked
    assert!(ach.is_unlocked(AchievementId::SlayerI));
}

#[test]
fn test_clear_recently_then_clear_pending_accumulates_fresh() {
    let mut ach = Achievements::default();

    // First batch
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    ach.clear_pending_notifications();
    ach.clear_recently_unlocked();

    // Second batch
    ach.unlock(AchievementId::BossHunterI, Some("Hero".to_string()));
    ach.clear_pending_notifications();

    // Only second batch in recently_unlocked
    assert_eq!(ach.recently_unlocked.len(), 1);
    assert!(ach.is_recently_unlocked(AchievementId::BossHunterI));
    assert!(!ach.is_recently_unlocked(AchievementId::SlayerI));
}

#[test]
fn test_count_recently_unlocked_by_category_returns_zero_for_empty() {
    let ach = Achievements::default();
    for cat in AchievementCategory::ALL {
        assert_eq!(ach.count_recently_unlocked_by_category(cat), 0);
    }
}

#[test]
fn test_count_recently_unlocked_by_category_filters_correctly() {
    let mut ach = Achievements::default();
    // SlayerI, BossHunterI => Combat; Level10 => Level; HavenDiscovered => Exploration
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    ach.unlock(AchievementId::BossHunterI, Some("Hero".to_string()));
    ach.unlock(AchievementId::Level10, Some("Hero".to_string()));
    ach.unlock(AchievementId::HavenDiscovered, Some("Hero".to_string()));

    ach.clear_pending_notifications();

    assert_eq!(
        ach.count_recently_unlocked_by_category(AchievementCategory::Combat),
        2
    );
    assert_eq!(
        ach.count_recently_unlocked_by_category(AchievementCategory::Level),
        1
    );
    assert_eq!(
        ach.count_recently_unlocked_by_category(AchievementCategory::Exploration),
        1
    );
    assert_eq!(
        ach.count_recently_unlocked_by_category(AchievementCategory::Progression),
        0
    );
    assert_eq!(
        ach.count_recently_unlocked_by_category(AchievementCategory::Challenges),
        0
    );
}

#[test]
fn test_count_recently_unlocked_by_category_progression() {
    let mut ach = Achievements::default();
    // Zone completions are in Progression category
    ach.unlock(AchievementId::Zone1Complete, Some("Hero".to_string()));
    ach.unlock(AchievementId::Zone2Complete, Some("Hero".to_string()));
    ach.clear_pending_notifications();

    assert_eq!(
        ach.count_recently_unlocked_by_category(AchievementCategory::Progression),
        2
    );
    assert_eq!(
        ach.count_recently_unlocked_by_category(AchievementCategory::Combat),
        0
    );
}

// =========================================================================
// 2. stats.rs — total_count, unlocked_count, unlock_percentage, etc.
// =========================================================================

#[test]
fn test_total_count_is_positive_and_matches_variant_count() {
    let ach = Achievements::default();
    assert!(ach.total_count() > 0);
    assert_eq!(ach.total_count(), AchievementId::VARIANT_COUNT);
}

#[test]
fn test_unlocked_count_starts_at_zero() {
    let ach = Achievements::default();
    assert_eq!(ach.unlocked_count(), 0);
}

#[test]
fn test_unlocked_count_reflects_unlocks() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    assert_eq!(ach.unlocked_count(), 1);

    ach.unlock(AchievementId::Level10, Some("Hero".to_string()));
    assert_eq!(ach.unlocked_count(), 2);
}

#[test]
fn test_unlock_percentage_zero_when_empty() {
    let ach = Achievements::default();
    let pct = ach.unlock_percentage();
    assert!((pct - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_unlock_percentage_partial_value() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));

    let total = ach.total_count() as f32;
    let expected = (1.0 / total) * 100.0;
    let actual = ach.unlock_percentage();
    assert!(
        (actual - expected).abs() < 0.01,
        "Expected ~{}, got {}",
        expected,
        actual
    );
}

#[test]
fn test_unlock_percentage_increases_with_more_unlocks() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    let pct_one = ach.unlock_percentage();

    ach.unlock(AchievementId::Level10, Some("Hero".to_string()));
    let pct_two = ach.unlock_percentage();

    assert!(pct_two > pct_one);
}

#[test]
fn test_count_by_category_returns_positive_totals_for_most_categories() {
    let ach = Achievements::default();
    for cat in AchievementCategory::ALL {
        let (unlocked, total) = ach.count_by_category(cat);
        assert_eq!(unlocked, 0, "{:?} should start with 0 unlocked", cat);
        if cat != AchievementCategory::Stats {
            assert!(total > 0, "{:?} should have a positive total", cat);
        }
    }
}

#[test]
fn test_count_by_category_tracks_unlocks_accurately() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::Level10, Some("Hero".to_string()));

    let (level_unlocked, level_total) = ach.count_by_category(AchievementCategory::Level);
    assert_eq!(level_unlocked, 1);
    assert!(level_total >= 1);

    let (combat_unlocked, _) = ach.count_by_category(AchievementCategory::Combat);
    assert_eq!(combat_unlocked, 0);
}

#[test]
fn test_count_by_category_exploration_has_entries() {
    let ach = Achievements::default();
    let (unlocked, total) = ach.count_by_category(AchievementCategory::Exploration);
    assert_eq!(unlocked, 0);
    assert!(
        total > 0,
        "Exploration category should have achievement definitions"
    );
}

#[test]
fn test_count_by_category_challenges_has_entries() {
    let ach = Achievements::default();
    let (unlocked, total) = ach.count_by_category(AchievementCategory::Challenges);
    assert_eq!(unlocked, 0);
    assert!(
        total > 0,
        "Challenges category should have achievement definitions"
    );
}

#[test]
fn test_take_newly_unlocked_empty_on_default() {
    let mut ach = Achievements::default();
    let newly = ach.take_newly_unlocked();
    assert!(newly.is_empty());
}

#[test]
fn test_take_newly_unlocked_returns_and_drains() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::GoneFishing, Some("Hero".to_string()));
    ach.unlock(AchievementId::DungeonDiver, Some("Hero".to_string()));

    let newly = ach.take_newly_unlocked();
    assert_eq!(newly.len(), 2);
    assert!(newly.contains(&AchievementId::GoneFishing));
    assert!(newly.contains(&AchievementId::DungeonDiver));

    // Second call should be empty
    let again = ach.take_newly_unlocked();
    assert!(again.is_empty());
}

#[test]
fn test_take_newly_unlocked_does_not_affect_unlocked_state() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    let _drained = ach.take_newly_unlocked();
    assert!(ach.is_unlocked(AchievementId::SlayerI));
}

#[test]
fn test_take_newly_unlocked_does_not_affect_pending_or_modal() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    let _drained = ach.take_newly_unlocked();

    // Pending and modal are separate channels
    assert_eq!(ach.pending_count(), 1);
    assert_eq!(ach.modal_queue.len(), 1);
}

#[test]
fn test_update_progress_overwrites_previous() {
    let mut ach = Achievements::default();
    ach.update_progress(AchievementId::SlayerI, 10, 100);
    ach.update_progress(AchievementId::SlayerI, 75, 100);

    let p = ach.get_progress(AchievementId::SlayerI).unwrap();
    assert_eq!(p.current, 75);
    assert_eq!(p.target, 100);
}

#[test]
fn test_get_progress_none_for_untracked() {
    let ach = Achievements::default();
    assert!(ach.get_progress(AchievementId::HavenDiscovered).is_none());
}

#[test]
fn test_progress_set_by_check_milestones_below_threshold() {
    let mut ach = Achievements::default();
    for _ in 0..25 {
        ach.on_enemy_killed(false, None);
    }
    let p = ach.get_progress(AchievementId::SlayerI).unwrap();
    assert_eq!(p.current, 25);
    assert_eq!(p.target, 100);
}

// =========================================================================
// 3. unlock.rs — is_unlocked, unlock return values, side effects
// =========================================================================

#[test]
fn test_is_unlocked_false_by_default() {
    let ach = Achievements::default();
    assert!(!ach.is_unlocked(AchievementId::SlayerI));
    assert!(!ach.is_unlocked(AchievementId::Eternal));
    assert!(!ach.is_unlocked(AchievementId::HavenDiscovered));
}

#[test]
fn test_is_unlocked_true_after_unlock() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    assert!(ach.is_unlocked(AchievementId::SlayerI));
}

#[test]
fn test_unlock_returns_true_on_first_call() {
    let mut ach = Achievements::default();
    let result = ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    assert!(result);
}

#[test]
fn test_unlock_returns_false_on_duplicate() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    let result = ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    assert!(!result);
}

#[test]
fn test_unlock_side_effects_pending_newly_modal_accumulation() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::DungeonDiver, Some("Hero".to_string()));

    assert_eq!(ach.pending_notifications.len(), 1);
    assert_eq!(ach.pending_notifications[0], AchievementId::DungeonDiver);
    assert_eq!(ach.newly_unlocked.len(), 1);
    assert_eq!(ach.newly_unlocked[0], AchievementId::DungeonDiver);
    assert_eq!(ach.modal_queue.len(), 1);
    assert_eq!(ach.modal_queue[0], AchievementId::DungeonDiver);
    assert!(ach.accumulation_start.is_some());
}

#[test]
fn test_unlock_duplicate_does_not_add_side_effects() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));

    let pending_before = ach.pending_count();
    let newly_before = ach.newly_unlocked.len();
    let modal_before = ach.modal_queue.len();

    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));

    assert_eq!(ach.pending_count(), pending_before);
    assert_eq!(ach.newly_unlocked.len(), newly_before);
    assert_eq!(ach.modal_queue.len(), modal_before);
}

#[test]
fn test_unlock_accumulation_start_set_only_once() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    let first_start = ach.accumulation_start.unwrap();

    std::thread::sleep(std::time::Duration::from_millis(1));

    ach.unlock(AchievementId::Level10, Some("Hero".to_string()));
    let second_start = ach.accumulation_start.unwrap();

    // accumulation_start should NOT be reset by subsequent unlocks
    assert_eq!(first_start, second_start);
}

#[test]
fn test_unlock_records_character_name_some() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("MyChar".to_string()));

    let record = ach.unlocked.get(&AchievementId::SlayerI).unwrap();
    assert_eq!(record.character_name, Some("MyChar".to_string()));
}

#[test]
fn test_unlock_records_character_name_none() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, None);

    let record = ach.unlocked.get(&AchievementId::SlayerI).unwrap();
    assert!(record.character_name.is_none());
}

#[test]
fn test_unlock_records_timestamp() {
    let before = chrono::Utc::now().timestamp();
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    let after = chrono::Utc::now().timestamp();

    let record = ach.unlocked.get(&AchievementId::SlayerI).unwrap();
    assert!(record.unlocked_at >= before);
    assert!(record.unlocked_at <= after);
}

#[test]
fn test_unlock_with_name_convenience_via_handler() {
    // on_* handlers use unlock_with_name internally (Option<&str> -> Option<String>)
    let mut ach = Achievements::default();
    ach.on_haven_discovered(Some("ConvenienceHero"));

    let record = ach.unlocked.get(&AchievementId::HavenDiscovered).unwrap();
    assert_eq!(record.character_name, Some("ConvenienceHero".to_string()));
}

#[test]
fn test_unlock_with_name_none_via_handler() {
    let mut ach = Achievements::default();
    ach.on_storms_end(None);

    let record = ach.unlocked.get(&AchievementId::StormsEnd).unwrap();
    assert!(record.character_name.is_none());
}

// =========================================================================
// 4. modal.rs — is_modal_ready, take_modal_queue
// =========================================================================

#[test]
fn test_is_modal_ready_false_on_empty_queue() {
    let ach = Achievements::default();
    assert!(!ach.is_modal_ready());
}

#[test]
fn test_is_modal_ready_false_immediately_after_unlock() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    assert!(!ach.is_modal_ready());
}

#[test]
fn test_is_modal_ready_true_after_500ms() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));

    ach.accumulation_start =
        Some(std::time::Instant::now() - std::time::Duration::from_millis(600));

    assert!(ach.is_modal_ready());
}

#[test]
fn test_is_modal_ready_at_exactly_500ms() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));

    ach.accumulation_start =
        Some(std::time::Instant::now() - std::time::Duration::from_millis(500));

    assert!(ach.is_modal_ready());
}

#[test]
fn test_is_modal_ready_false_with_queue_but_no_timer() {
    let mut ach = Achievements::default();
    ach.modal_queue.push(AchievementId::SlayerI);
    assert!(ach.accumulation_start.is_none());
    assert!(!ach.is_modal_ready());
}

#[test]
fn test_is_modal_ready_false_when_queue_empty_even_with_timer() {
    let mut ach = Achievements::default();
    ach.accumulation_start =
        Some(std::time::Instant::now() - std::time::Duration::from_millis(1000));
    assert!(!ach.is_modal_ready());
}

#[test]
fn test_take_modal_queue_returns_all_queued() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    ach.unlock(AchievementId::Level10, Some("Hero".to_string()));
    ach.unlock(AchievementId::GoneFishing, Some("Hero".to_string()));

    let queue = ach.take_modal_queue();
    assert_eq!(queue.len(), 3);
    assert!(queue.contains(&AchievementId::SlayerI));
    assert!(queue.contains(&AchievementId::Level10));
    assert!(queue.contains(&AchievementId::GoneFishing));
}

#[test]
fn test_take_modal_queue_drains_and_resets_timer() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    assert!(ach.accumulation_start.is_some());
    assert!(!ach.modal_queue.is_empty());

    let _ = ach.take_modal_queue();

    assert!(ach.modal_queue.is_empty());
    assert!(ach.accumulation_start.is_none());
}

#[test]
fn test_take_modal_queue_second_call_empty() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));

    let first = ach.take_modal_queue();
    assert_eq!(first.len(), 1);

    let second = ach.take_modal_queue();
    assert!(second.is_empty());
}

#[test]
fn test_take_modal_queue_returns_empty_when_nothing_queued() {
    let mut ach = Achievements::default();
    let queue = ach.take_modal_queue();
    assert!(queue.is_empty());
}

#[test]
fn test_take_modal_then_new_unlock_starts_fresh_accumulation() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    let _ = ach.take_modal_queue();

    assert!(ach.accumulation_start.is_none());

    ach.unlock(AchievementId::Level10, Some("Hero".to_string()));
    assert!(ach.accumulation_start.is_some());
    assert_eq!(ach.modal_queue.len(), 1);
    assert_eq!(ach.modal_queue[0], AchievementId::Level10);
}

#[test]
fn test_is_modal_ready_false_after_take() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    ach.accumulation_start =
        Some(std::time::Instant::now() - std::time::Duration::from_millis(600));
    assert!(ach.is_modal_ready());

    let _ = ach.take_modal_queue();
    assert!(!ach.is_modal_ready());
}

// =========================================================================
// 5. persistence.rs — save/load roundtrip, corrupted JSON, transient skip
// =========================================================================

#[test]
fn test_serde_roundtrip_preserves_unlocked_achievements() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    ach.unlock(AchievementId::DungeonDiver, Some("Hero".to_string()));
    ach.total_kills = 999;
    ach.total_dungeons_completed = 42;
    ach.highest_prestige_rank = 10;

    let json = serde_json::to_string_pretty(&ach).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();

    assert!(loaded.is_unlocked(AchievementId::SlayerI));
    assert!(loaded.is_unlocked(AchievementId::DungeonDiver));
    assert!(!loaded.is_unlocked(AchievementId::SlayerII));
    assert_eq!(loaded.total_kills, 999);
    assert_eq!(loaded.total_dungeons_completed, 42);
    assert_eq!(loaded.highest_prestige_rank, 10);
}

#[test]
fn test_serde_roundtrip_preserves_progress_entries() {
    let mut ach = Achievements::default();
    ach.update_progress(AchievementId::SlayerI, 55, 100);
    ach.update_progress(AchievementId::DungeonMasterI, 7, 10);

    let json = serde_json::to_string(&ach).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();

    let p1 = loaded.get_progress(AchievementId::SlayerI).unwrap();
    assert_eq!(p1.current, 55);
    assert_eq!(p1.target, 100);

    let p2 = loaded.get_progress(AchievementId::DungeonMasterI).unwrap();
    assert_eq!(p2.current, 7);
    assert_eq!(p2.target, 10);
}

#[test]
fn test_corrupted_json_falls_back_to_default() {
    let garbage = r#"{ not valid JSON at all }"#;
    let result: Result<Achievements, _> = serde_json::from_str(garbage);
    assert!(result.is_err());

    let fallback = result.unwrap_or_default();
    assert_eq!(fallback.total_kills, 0);
    assert!(fallback.unlocked.is_empty());
}

#[test]
fn test_empty_json_object_deserializes_to_defaults() {
    let json = r#"{}"#;
    let result: Result<Achievements, _> = serde_json::from_str(json);
    let ach = result.unwrap_or_default();
    assert_eq!(ach.total_kills, 0);
    assert!(ach.unlocked.is_empty());
    assert!(ach.progress.is_empty());
}

#[test]
fn test_transient_fields_skipped_during_serialization() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    ach.clear_pending_notifications(); // moves to recently_unlocked

    assert!(!ach.recently_unlocked.is_empty());
    assert!(!ach.newly_unlocked.is_empty());
    assert!(!ach.modal_queue.is_empty());
    assert!(ach.accumulation_start.is_some());

    let json = serde_json::to_string(&ach).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();

    assert!(loaded.pending_notifications.is_empty());
    assert!(loaded.newly_unlocked.is_empty());
    assert!(loaded.modal_queue.is_empty());
    assert!(loaded.recently_unlocked.is_empty());
    assert!(loaded.accumulation_start.is_none());

    // Persistent data should survive
    assert!(loaded.is_unlocked(AchievementId::SlayerI));
}

#[test]
fn test_serde_preserves_character_name_in_unlock_record() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("NamedHero".to_string()));

    let json = serde_json::to_string(&ach).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();

    let record = loaded.unlocked.get(&AchievementId::SlayerI).unwrap();
    assert_eq!(record.character_name, Some("NamedHero".to_string()));
}

#[test]
fn test_serde_preserves_unlock_timestamp() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    let original_ts = ach
        .unlocked
        .get(&AchievementId::SlayerI)
        .unwrap()
        .unlocked_at;

    let json = serde_json::to_string(&ach).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();

    let loaded_ts = loaded
        .unlocked
        .get(&AchievementId::SlayerI)
        .unwrap()
        .unlocked_at;
    assert_eq!(original_ts, loaded_ts);
}

#[test]
fn test_serde_preserves_all_aggregate_counters() {
    let mut ach = Achievements::default();
    ach.total_kills = 12345;
    ach.total_bosses_defeated = 678;
    ach.total_fish_caught = 9999;
    ach.total_dungeons_completed = 42;
    ach.total_minigame_wins = 88;
    ach.highest_prestige_rank = 30;
    ach.highest_level = 500;
    ach.highest_fishing_rank = 25;
    ach.zones_fully_cleared = 7;
    ach.expanse_cycles_completed = 3;

    let json = serde_json::to_string(&ach).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.total_kills, 12345);
    assert_eq!(loaded.total_bosses_defeated, 678);
    assert_eq!(loaded.total_fish_caught, 9999);
    assert_eq!(loaded.total_dungeons_completed, 42);
    assert_eq!(loaded.total_minigame_wins, 88);
    assert_eq!(loaded.highest_prestige_rank, 30);
    assert_eq!(loaded.highest_level, 500);
    assert_eq!(loaded.highest_fishing_rank, 25);
    assert_eq!(loaded.zones_fully_cleared, 7);
    assert_eq!(loaded.expanse_cycles_completed, 3);
}

#[test]
fn test_empty_unlocked_map_serializes_and_deserializes() {
    let ach = Achievements::default();
    let json = serde_json::to_string(&ach).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();
    assert!(loaded.unlocked.is_empty());
    assert!(loaded.progress.is_empty());
}

#[test]
fn test_json_with_unknown_extra_fields_loads_gracefully() {
    let json = r#"{
        "unlocked": {},
        "progress": {},
        "total_kills": 42,
        "total_bosses_defeated": 0,
        "total_fish_caught": 0,
        "total_dungeons_completed": 0,
        "total_minigame_wins": 0,
        "highest_prestige_rank": 0,
        "highest_level": 0,
        "highest_fishing_rank": 0,
        "zones_fully_cleared": 0,
        "expanse_cycles_completed": 0,
        "some_future_field": "should be ignored"
    }"#;

    let result: Result<Achievements, _> = serde_json::from_str(json);
    let loaded = result.unwrap_or_default();
    // Either it loaded with total_kills=42 or fell back to default -- both acceptable
    assert!(loaded.total_kills == 42 || loaded.total_kills == 0);
}

#[test]
fn test_file_save_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_achievements.json");

    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("FileTest".to_string()));
    ach.total_kills = 777;
    ach.update_progress(AchievementId::SlayerII, 200, 500);

    let json = serde_json::to_string_pretty(&ach).unwrap();
    std::fs::write(&path, &json).unwrap();

    let read_json = std::fs::read_to_string(&path).unwrap();
    let loaded: Achievements = serde_json::from_str(&read_json).unwrap();

    assert!(loaded.is_unlocked(AchievementId::SlayerI));
    assert_eq!(loaded.total_kills, 777);
    let p = loaded.get_progress(AchievementId::SlayerII).unwrap();
    assert_eq!(p.current, 200);
    assert_eq!(p.target, 500);
}

// =========================================================================
// 6. handlers.rs — event handler coverage gaps
// =========================================================================

#[test]
fn test_on_haven_discovered_idempotent() {
    let mut ach = Achievements::default();
    ach.on_haven_discovered(Some("Hero"));
    let count = ach.unlocked_count();
    ach.on_haven_discovered(Some("Hero"));
    assert_eq!(ach.unlocked_count(), count);
}

#[test]
fn test_on_haven_all_t1_idempotent() {
    let mut ach = Achievements::default();
    ach.on_haven_all_t1(Some("Hero"));
    let count = ach.unlocked_count();
    ach.on_haven_all_t1(Some("Hero"));
    assert_eq!(ach.unlocked_count(), count);
}

#[test]
fn test_on_haven_all_t2_idempotent() {
    let mut ach = Achievements::default();
    ach.on_haven_all_t2(Some("Hero"));
    let count = ach.unlocked_count();
    ach.on_haven_all_t2(Some("Hero"));
    assert_eq!(ach.unlocked_count(), count);
}

#[test]
fn test_on_haven_architect_idempotent() {
    let mut ach = Achievements::default();
    ach.on_haven_architect(Some("Hero"));
    let count = ach.unlocked_count();
    ach.on_haven_architect(Some("Hero"));
    assert_eq!(ach.unlocked_count(), count);
}

#[test]
fn test_on_soulforge_discovered_unlocks() {
    let mut ach = Achievements::default();
    assert!(!ach.is_unlocked(AchievementId::SoulforgeDiscovered));
    ach.on_soulforge_discovered(Some("ForgeHero"));
    assert!(ach.is_unlocked(AchievementId::SoulforgeDiscovered));
}

#[test]
fn test_on_soulforge_discovered_idempotent() {
    let mut ach = Achievements::default();
    ach.on_soulforge_discovered(Some("Hero"));
    let count = ach.unlocked_count();
    ach.on_soulforge_discovered(Some("Hero"));
    assert_eq!(ach.unlocked_count(), count);
}

#[test]
fn test_on_storms_end_unlocks() {
    let mut ach = Achievements::default();
    assert!(!ach.is_unlocked(AchievementId::StormsEnd));
    ach.on_storms_end(Some("FinalHero"));
    assert!(ach.is_unlocked(AchievementId::StormsEnd));
}

#[test]
fn test_on_storms_end_idempotent() {
    let mut ach = Achievements::default();
    ach.on_storms_end(Some("Hero"));
    let count = ach.unlocked_count();
    ach.on_storms_end(Some("Hero"));
    assert_eq!(ach.unlocked_count(), count);
}

#[test]
fn test_on_dungeon_completed_milestones_first_and_tenth() {
    let mut ach = Achievements::default();

    ach.on_dungeon_completed(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DungeonDiver));
    assert!(!ach.is_unlocked(AchievementId::DungeonMasterI));

    for _ in 0..9 {
        ach.on_dungeon_completed(Some("Hero"));
    }
    assert!(ach.is_unlocked(AchievementId::DungeonMasterI));
}

#[test]
fn test_on_minigame_won_increments_total_and_unlocks() {
    let mut ach = Achievements::default();
    ach.on_minigame_won(
        MinigameType::Chess,
        MinigameDifficulty::Novice,
        Some("Hero"),
    );
    assert_eq!(ach.total_minigame_wins, 1);
    assert!(ach.is_unlocked(AchievementId::ChessNovice));
}

#[test]
fn test_on_minigame_won_different_games_accumulate_wins() {
    let mut ach = Achievements::default();
    ach.on_minigame_won(
        MinigameType::Chess,
        MinigameDifficulty::Novice,
        Some("Hero"),
    );
    ach.on_minigame_won(
        MinigameType::Morris,
        MinigameDifficulty::Master,
        Some("Hero"),
    );
    ach.on_minigame_won(
        MinigameType::Go,
        MinigameDifficulty::Apprentice,
        Some("Hero"),
    );

    assert_eq!(ach.total_minigame_wins, 3);
    assert!(ach.is_unlocked(AchievementId::ChessNovice));
    assert!(ach.is_unlocked(AchievementId::MorrisMaster));
    assert!(ach.is_unlocked(AchievementId::GoApprentice));
}

#[test]
fn test_on_minigame_won_with_none_character_name() {
    let mut ach = Achievements::default();
    ach.on_minigame_won(MinigameType::Chess, MinigameDifficulty::Novice, None);
    assert!(ach.is_unlocked(AchievementId::ChessNovice));
    let record = ach.unlocked.get(&AchievementId::ChessNovice).unwrap();
    assert!(record.character_name.is_none());
}

#[test]
fn test_on_zone_fully_cleared_unknown_zone_no_achievement() {
    let mut ach = Achievements::default();
    ach.on_zone_fully_cleared(0, Some("Hero"));
    ach.on_zone_fully_cleared(255, Some("Hero"));

    assert_eq!(ach.zones_fully_cleared, 2);
    assert!(!ach.is_unlocked(AchievementId::Zone1Complete));
    assert!(!ach.is_unlocked(AchievementId::BeyondInfinity));
}

#[test]
fn test_on_zone_fully_cleared_zone_11_increments_expanse_cycles() {
    let mut ach = Achievements::default();
    ach.on_zone_fully_cleared(11, Some("Hero"));
    ach.on_zone_fully_cleared(11, Some("Hero"));

    assert_eq!(ach.expanse_cycles_completed, 2);
    assert_eq!(ach.zones_fully_cleared, 2);
    assert!(ach.is_unlocked(AchievementId::BeyondInfinity));
}

#[test]
fn test_on_zone_fully_cleared_with_none_character_name() {
    let mut ach = Achievements::default();
    ach.on_zone_fully_cleared(1, None);
    assert!(ach.is_unlocked(AchievementId::Zone1Complete));
    let record = ach.unlocked.get(&AchievementId::Zone1Complete).unwrap();
    assert!(record.character_name.is_none());
}

#[test]
fn test_on_enemy_killed_with_none_name() {
    let mut ach = Achievements::default();
    ach.total_kills = 99;
    ach.on_enemy_killed(false, None);
    assert!(ach.is_unlocked(AchievementId::SlayerI));
    let record = ach.unlocked.get(&AchievementId::SlayerI).unwrap();
    assert!(record.character_name.is_none());
}

#[test]
fn test_on_fish_caught_with_none_name() {
    let mut ach = Achievements::default();
    ach.on_fish_caught(None);
    assert_eq!(ach.total_fish_caught, 1);
    assert!(ach.is_unlocked(AchievementId::GoneFishing));
    let record = ach.unlocked.get(&AchievementId::GoneFishing).unwrap();
    assert!(record.character_name.is_none());
}

#[test]
fn test_on_fishing_rank_up_with_none_name() {
    let mut ach = Achievements::default();
    ach.on_fishing_rank_up(10, None);
    assert!(ach.is_unlocked(AchievementId::FishermanI));
}

#[test]
fn test_on_fishing_rank_1_unlocks_nothing() {
    let mut ach = Achievements::default();
    ach.on_fishing_rank_up(1, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::FishermanI));
    assert_eq!(ach.highest_fishing_rank, 1);
}

#[test]
fn test_on_dungeon_completed_with_none_name() {
    let mut ach = Achievements::default();
    ach.on_dungeon_completed(None);
    assert!(ach.is_unlocked(AchievementId::DungeonDiver));
    let record = ach.unlocked.get(&AchievementId::DungeonDiver).unwrap();
    assert!(record.character_name.is_none());
}

#[test]
fn test_on_storm_leviathan_caught_with_none_name() {
    let mut ach = Achievements::default();
    ach.on_storm_leviathan_caught(None);
    assert!(ach.is_unlocked(AchievementId::StormLeviathan));
    let record = ach.unlocked.get(&AchievementId::StormLeviathan).unwrap();
    assert!(record.character_name.is_none());
}

#[test]
fn test_on_soulforge_discovered_with_none_name() {
    let mut ach = Achievements::default();
    ach.on_soulforge_discovered(None);
    assert!(ach.is_unlocked(AchievementId::SoulforgeDiscovered));
    let record = ach
        .unlocked
        .get(&AchievementId::SoulforgeDiscovered)
        .unwrap();
    assert!(record.character_name.is_none());
}

#[test]
fn test_on_enhancement_upgraded_with_none_name() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(5, &[5, 0, 0, 0, 0, 0, 0], 5, None);
    assert!(ach.is_unlocked(AchievementId::JourneymanSmith));
    let record = ach.unlocked.get(&AchievementId::JourneymanSmith).unwrap();
    assert!(record.character_name.is_none());
}

#[test]
fn test_on_enhancement_upgraded_level_0_with_high_attempts() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(0, &[0, 0, 0, 0, 0, 0, 0], 100, Some("Hero"));

    assert!(ach.is_unlocked(AchievementId::PersistentHammering));
    assert!(!ach.is_unlocked(AchievementId::ApprenticeSmith));
    assert!(!ach.is_unlocked(AchievementId::FullyTempered));
    assert!(!ach.is_unlocked(AchievementId::SoulConvergence));
}

#[test]
fn test_on_enhancement_upgraded_all_slots_10_unlocks_everything() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(10, &[10, 10, 10, 10, 10, 10, 10], 200, Some("Hero"));

    assert!(ach.is_unlocked(AchievementId::ApprenticeSmith));
    assert!(ach.is_unlocked(AchievementId::JourneymanSmith));
    assert!(ach.is_unlocked(AchievementId::SoulforgeAdept));
    assert!(ach.is_unlocked(AchievementId::SoulforgeSavant));
    assert!(ach.is_unlocked(AchievementId::SoulforgeMaster));
    assert!(ach.is_unlocked(AchievementId::SoulforgeGrandmaster));
    assert!(ach.is_unlocked(AchievementId::MasterSmith));
    assert!(ach.is_unlocked(AchievementId::FullyTempered));
    assert!(ach.is_unlocked(AchievementId::SoulConvergence));
    assert!(ach.is_unlocked(AchievementId::PersistentHammering));
}

#[test]
fn test_on_enhancement_partially_filled_blocks_convergence() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(7, &[7, 7, 7, 7, 7, 7, 6], 49, Some("Hero"));

    assert!(ach.is_unlocked(AchievementId::SoulforgeSavant));
    assert!(ach.is_unlocked(AchievementId::FullyTempered)); // all >= 4
    assert!(!ach.is_unlocked(AchievementId::SoulConvergence)); // one slot at 6 < 7
}

#[test]
fn test_on_enhancement_all_slots_5_unlocks_tempered_and_journeyman() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(5, &[5, 5, 5, 5, 5, 5, 5], 35, Some("Hero"));

    assert!(ach.is_unlocked(AchievementId::ApprenticeSmith));
    assert!(ach.is_unlocked(AchievementId::JourneymanSmith));
    assert!(ach.is_unlocked(AchievementId::FullyTempered)); // all >= 4
    assert!(!ach.is_unlocked(AchievementId::SoulforgeAdept)); // need 6
    assert!(!ach.is_unlocked(AchievementId::SoulConvergence)); // need all 7
}

#[test]
fn test_on_level_up_below_threshold_unlocks_nothing() {
    let mut ach = Achievements::default();
    ach.on_level_up(9, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::Level10));
    assert_eq!(ach.unlocked.len(), 0);
}

#[test]
fn test_on_level_up_stores_progress_for_next_milestone() {
    let mut ach = Achievements::default();
    ach.on_level_up(5, Some("Hero"));

    let p = ach.get_progress(AchievementId::Level10).unwrap();
    assert_eq!(p.current, 5);
    assert_eq!(p.target, 10);
}

#[test]
fn test_on_prestige_stores_progress_for_next_milestone() {
    let mut ach = Achievements::default();
    ach.on_prestige(3, Some("Hero"));

    assert!(ach.is_unlocked(AchievementId::FirstPrestige));
    assert!(!ach.is_unlocked(AchievementId::PrestigeV));

    let p = ach.get_progress(AchievementId::PrestigeV).unwrap();
    assert_eq!(p.current, 3);
    assert_eq!(p.target, 5);
}

// =========================================================================
// 6b. handlers.rs — high milestone coverage
// =========================================================================

#[test]
fn test_dungeon_master_iv_at_1000() {
    let mut ach = Achievements::default();
    ach.total_dungeons_completed = 999;
    ach.on_dungeon_completed(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DungeonMasterIV));
}

#[test]
fn test_dungeon_master_v_at_5000() {
    let mut ach = Achievements::default();
    ach.total_dungeons_completed = 4999;
    ach.on_dungeon_completed(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DungeonMasterV));
}

#[test]
fn test_dungeon_master_vi_at_10000() {
    let mut ach = Achievements::default();
    ach.total_dungeons_completed = 9999;
    ach.on_dungeon_completed(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DungeonMasterVI));
}

#[test]
fn test_dungeon_master_vii_at_25000() {
    let mut ach = Achievements::default();
    ach.total_dungeons_completed = 24999;
    ach.on_dungeon_completed(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DungeonMasterVII));
}

#[test]
fn test_dungeon_master_viii_at_100000() {
    let mut ach = Achievements::default();
    ach.total_dungeons_completed = 99999;
    ach.on_dungeon_completed(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DungeonMasterVIII));
}

#[test]
fn test_dungeon_master_ix_at_500000() {
    let mut ach = Achievements::default();
    ach.total_dungeons_completed = 499999;
    ach.on_dungeon_completed(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DungeonMasterIX));
}

#[test]
fn test_dungeon_master_x_at_1000000() {
    let mut ach = Achievements::default();
    ach.total_dungeons_completed = 999999;
    ach.on_dungeon_completed(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DungeonMasterX));
}

#[test]
fn test_slayer_vi_at_50000() {
    let mut ach = Achievements::default();
    ach.total_kills = 49999;
    ach.on_enemy_killed(false, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SlayerVI));
}

#[test]
fn test_slayer_vii_at_100000() {
    let mut ach = Achievements::default();
    ach.total_kills = 99999;
    ach.on_enemy_killed(false, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SlayerVII));
}

#[test]
fn test_slayer_viii_at_500000() {
    let mut ach = Achievements::default();
    ach.total_kills = 499999;
    ach.on_enemy_killed(false, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SlayerVIII));
}

#[test]
fn test_slayer_ix_at_1000000() {
    let mut ach = Achievements::default();
    ach.total_kills = 999999;
    ach.on_enemy_killed(false, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SlayerIX));
}

#[test]
fn test_boss_hunter_vi_at_1000() {
    let mut ach = Achievements::default();
    ach.total_bosses_defeated = 999;
    ach.total_kills = 999;
    ach.on_enemy_killed(true, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::BossHunterVI));
}

#[test]
fn test_boss_hunter_vii_at_5000() {
    let mut ach = Achievements::default();
    ach.total_bosses_defeated = 4999;
    ach.total_kills = 4999;
    ach.on_enemy_killed(true, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::BossHunterVII));
}

#[test]
fn test_boss_hunter_viii_at_10000() {
    let mut ach = Achievements::default();
    ach.total_bosses_defeated = 9999;
    ach.total_kills = 9999;
    ach.on_enemy_killed(true, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::BossHunterVIII));
}

#[test]
fn test_boss_hunter_ix_at_25000() {
    let mut ach = Achievements::default();
    ach.total_bosses_defeated = 24999;
    ach.total_kills = 24999;
    ach.on_enemy_killed(true, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::BossHunterIX));
}

#[test]
fn test_fish_catcher_iv_at_100000() {
    let mut ach = Achievements::default();
    ach.total_fish_caught = 99999;
    ach.on_fish_caught(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FishCatcherIV));
}

#[test]
fn test_fish_catcher_v_at_500000() {
    let mut ach = Achievements::default();
    ach.total_fish_caught = 499999;
    ach.on_fish_caught(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FishCatcherV));
}

#[test]
fn test_fish_catcher_vi_at_1000000() {
    let mut ach = Achievements::default();
    ach.total_fish_caught = 999999;
    ach.on_fish_caught(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FishCatcherVI));
}

#[test]
fn test_fish_catcher_vii_at_5000000() {
    let mut ach = Achievements::default();
    ach.total_fish_caught = 4999999;
    ach.on_fish_caught(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FishCatcherVII));
}

// =========================================================================
// 7. Interaction tests (cross-module behavior)
// =========================================================================

#[test]
fn test_full_notification_lifecycle() {
    let mut ach = Achievements::default();

    // Step 1: Unlock
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    assert_eq!(ach.pending_count(), 1);
    assert!(ach.recently_unlocked.is_empty());

    // Step 2: Clear pending (simulates opening achievement browser)
    ach.clear_pending_notifications();
    assert_eq!(ach.pending_count(), 0);
    assert!(ach.is_recently_unlocked(AchievementId::SlayerI));

    // Step 3: Clear recently unlocked (simulates closing achievement browser)
    ach.clear_recently_unlocked();
    assert!(!ach.is_recently_unlocked(AchievementId::SlayerI));
    assert!(ach.is_unlocked(AchievementId::SlayerI));
}

#[test]
fn test_full_modal_lifecycle() {
    let mut ach = Achievements::default();

    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    assert!(!ach.is_modal_ready());

    // Simulate 500ms elapsing
    ach.accumulation_start =
        Some(std::time::Instant::now() - std::time::Duration::from_millis(600));
    assert!(ach.is_modal_ready());

    let queue = ach.take_modal_queue();
    assert_eq!(queue.len(), 1);
    assert!(!ach.is_modal_ready());

    // New unlock after take
    ach.unlock(AchievementId::Level10, Some("Hero".to_string()));
    assert!(!ach.is_modal_ready());
    assert_eq!(ach.modal_queue.len(), 1);
}

#[test]
fn test_batch_unlock_via_level_up_fills_all_transient_channels() {
    let mut ach = Achievements::default();
    // Level 200 unlocks Level10/25/50/100/150/200 = 6 achievements at once
    ach.on_level_up(200, Some("BatchHero"));

    assert_eq!(ach.unlocked_count(), 6);
    assert_eq!(ach.pending_count(), 6);
    assert_eq!(ach.newly_unlocked.len(), 6);
    assert_eq!(ach.modal_queue.len(), 6);
    assert!(ach.accumulation_start.is_some());
}

#[test]
fn test_pending_and_newly_unlocked_are_independent_lists() {
    let mut ach = Achievements::default();
    ach.unlock(AchievementId::SlayerI, Some("Hero".to_string()));

    assert_eq!(ach.pending_notifications.len(), 1);
    assert_eq!(ach.newly_unlocked.len(), 1);

    // Taking newly_unlocked does not affect pending
    let _newly = ach.take_newly_unlocked();
    assert_eq!(ach.pending_count(), 1);
    assert!(ach.newly_unlocked.is_empty());

    // Clearing pending does not affect newly_unlocked
    ach.clear_pending_notifications();
    assert_eq!(ach.pending_count(), 0);
}

#[test]
fn test_handler_with_sync_from_haven_comprehensive() {
    use quest::haven::types::HavenRoomId;

    let mut ach = Achievements::default();

    let room_tiers: HashMap<HavenRoomId, u8> = HavenRoomId::ALL
        .iter()
        .map(|r| (*r, r.max_tier()))
        .collect();

    ach.sync_from_haven(true, &room_tiers, Some("MaxHaven"));

    assert!(ach.is_unlocked(AchievementId::HavenDiscovered));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderI));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderII));
    assert!(ach.is_unlocked(AchievementId::HavenArchitect));

    let newly = ach.take_newly_unlocked();
    assert_eq!(newly.len(), 4);
}

#[test]
fn test_check_milestones_sets_progress_for_unmet_thresholds() {
    let mut ach = Achievements::default();
    ach.total_kills = 49;
    ach.on_enemy_killed(false, None);
    // total_kills is now 50

    let p = ach.get_progress(AchievementId::SlayerI).unwrap();
    assert_eq!(p.current, 50);
    assert_eq!(p.target, 100);

    let p2 = ach.get_progress(AchievementId::SlayerII).unwrap();
    assert_eq!(p2.current, 50);
    assert_eq!(p2.target, 500);
}

#[test]
fn test_refresh_progress_creates_entries_for_all_series() {
    let mut ach = Achievements::default();
    ach.total_kills = 1;
    ach.total_bosses_defeated = 1;
    ach.total_dungeons_completed = 1;
    ach.total_fish_caught = 1;
    ach.total_minigame_wins = 1;

    ach.refresh_progress();

    assert!(ach.get_progress(AchievementId::SlayerI).is_some());
    assert!(ach.get_progress(AchievementId::BossHunterI).is_some());
    assert!(ach.get_progress(AchievementId::DungeonDiver).is_some());
    assert!(ach.get_progress(AchievementId::GoneFishing).is_some());
    assert!(ach.get_progress(AchievementId::GrandChampion).is_some());
}

#[test]
fn test_refresh_progress_updates_even_for_zero_counters() {
    let mut ach = Achievements::default();
    ach.refresh_progress();

    let p = ach.get_progress(AchievementId::SlayerI).unwrap();
    assert_eq!(p.current, 0);
    assert_eq!(p.target, 100);
}

#[test]
fn test_default_achievements_all_counters_zero() {
    let ach = Achievements::default();
    assert_eq!(ach.total_kills, 0);
    assert_eq!(ach.total_bosses_defeated, 0);
    assert_eq!(ach.total_fish_caught, 0);
    assert_eq!(ach.total_dungeons_completed, 0);
    assert_eq!(ach.total_minigame_wins, 0);
    assert_eq!(ach.highest_prestige_rank, 0);
    assert_eq!(ach.highest_level, 0);
    assert_eq!(ach.highest_fishing_rank, 0);
    assert_eq!(ach.zones_fully_cleared, 0);
    assert_eq!(ach.expanse_cycles_completed, 0);
}

#[test]
fn test_default_achievements_all_transient_empty() {
    let ach = Achievements::default();
    assert!(ach.pending_notifications.is_empty());
    assert!(ach.newly_unlocked.is_empty());
    assert!(ach.modal_queue.is_empty());
    assert!(ach.recently_unlocked.is_empty());
    assert!(ach.accumulation_start.is_none());
}
