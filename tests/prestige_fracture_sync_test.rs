//! Integration tests for prestige reset syncing fracture zone access.
//!
//! Verifies that after prestige, fracture zones (11-20) remain accessible
//! when the account has earned them via StormsEnd achievement and Deep breakthroughs.
//!
//! Note: Currently fracture zones have `prestige_requirement: 0`, so
//! `reset_for_prestige()` already unlocks them. `sync_account_zone_unlocks()`
//! provides a redundant safety net ensuring account-level unlocks are applied.

use quest::achievements::{AchievementId, Achievements};
use quest::character::prestige::perform_prestige;
use quest::core::game_state::GameState;
use quest::zones::sync_account_zone_unlocks;

/// Helper: create a GameState at a given prestige rank with level high enough to prestige again.
fn make_state_ready_to_prestige(current_rank: u32) -> GameState {
    let mut state = GameState::new("Test Hero".to_string(), 0);
    state.prestige_rank = current_rank;
    // Set level high enough to prestige from any rank
    state.character_level = 300;
    state
}

#[test]
fn test_prestige_then_sync_preserves_zone_11() {
    let mut state = make_state_ready_to_prestige(20);
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::StormsEnd, None);

    // Before prestige, unlock zone 11
    state.zone_progression.unlock_zone(11);
    assert!(state.zone_progression.is_zone_unlocked(11));

    // Prestige resets zone progression
    perform_prestige(&mut state);

    // Sync should ensure zone 11 is unlocked
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        11,
    );

    assert!(state.zone_progression.is_zone_unlocked(11));
}

#[test]
fn test_prestige_then_sync_preserves_fracture_zones_12_through_14() {
    let mut state = make_state_ready_to_prestige(20);
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::StormsEnd, None);

    // Unlock fracture zones before prestige
    for z in 11..=14 {
        state.zone_progression.unlock_zone(z);
    }

    perform_prestige(&mut state);

    // Sync with cap 14 should ensure zones 11-14 are unlocked
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        14,
    );

    assert!(state.zone_progression.is_zone_unlocked(11));
    assert!(state.zone_progression.is_zone_unlocked(12));
    assert!(state.zone_progression.is_zone_unlocked(13));
    assert!(state.zone_progression.is_zone_unlocked(14));
}

#[test]
fn test_prestige_then_sync_preserves_all_fracture_zones_cap_20() {
    let mut state = make_state_ready_to_prestige(20);
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::StormsEnd, None);

    perform_prestige(&mut state);

    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        20,
    );

    for z in 11..=20 {
        assert!(
            state.zone_progression.is_zone_unlocked(z),
            "Zone {} should be unlocked after prestige + sync with cap 20",
            z
        );
    }
}

#[test]
fn test_sync_without_storms_end_does_not_add_zone_11() {
    // Test sync_account_zone_unlocks in isolation (not after prestige)
    // to verify it respects StormsEnd gate
    let mut prog = quest::zones::ZoneProgression::new();
    let achievements = Achievements::default();

    sync_account_zone_unlocks(
        &mut prog,
        achievements.is_unlocked(AchievementId::StormsEnd),
        11,
    );

    assert!(!prog.is_zone_unlocked(11));
}

#[test]
fn test_prestige_resets_position_but_sync_keeps_zone_access() {
    let mut state = make_state_ready_to_prestige(20);
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::StormsEnd, None);

    // Player was in zone 14 before prestige
    state.zone_progression.current_zone_id = 14;
    state.zone_progression.current_subzone_id = 3;
    for z in 11..=14 {
        state.zone_progression.unlock_zone(z);
    }

    perform_prestige(&mut state);

    // Position should be reset to zone 1
    assert_eq!(state.zone_progression.current_zone_id, 1);
    assert_eq!(state.zone_progression.current_subzone_id, 1);

    // Sync restores access
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        14,
    );

    // Zones 11-14 should be accessible even though position is zone 1
    assert!(state.zone_progression.is_zone_unlocked(14));
}

#[test]
fn test_multiple_prestiges_preserve_fracture_access() {
    let mut state = make_state_ready_to_prestige(20);
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::StormsEnd, None);

    // First prestige + sync
    perform_prestige(&mut state);
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        17,
    );
    assert!(state.zone_progression.is_zone_unlocked(17));

    // Level up again for second prestige
    state.character_level = 300;

    // Second prestige + sync
    perform_prestige(&mut state);
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        17,
    );
    assert!(state.zone_progression.is_zone_unlocked(17));
    assert!(state.zone_progression.is_zone_unlocked(11));
}

#[test]
fn test_prestige_sync_with_expanded_cap() {
    let mut state = make_state_ready_to_prestige(20);
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::StormsEnd, None);

    // First prestige with cap 14
    perform_prestige(&mut state);
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        14,
    );
    assert!(state.zone_progression.is_zone_unlocked(14));

    // Level up and prestige again, now cap is 17
    state.character_level = 300;
    perform_prestige(&mut state);
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        17,
    );

    // Should now have zones up to 17
    assert!(state.zone_progression.is_zone_unlocked(17));
}
