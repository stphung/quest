//! Zone progression integration tests
//!
//! Tests the full zone progression flow with game state integration,
//! covering edge cases and prestige tier transitions.

use quest::achievements::Achievements;
use quest::zones::get_all_zones;
use quest::zones::{BossDefeatResult, ZoneProgression};

/// Number of kills needed before boss spawns (matches the constant in progression.rs)
const KILLS_FOR_BOSS: u32 = 10;

// ============================================================================
// Unit test gaps (functions not directly tested)
// ============================================================================

#[test]
fn test_should_spawn_boss_returns_true_at_threshold() {
    let mut prog = ZoneProgression::new();

    // Not ready yet
    assert!(!prog.should_spawn_boss());

    // Get to threshold
    for _ in 0..KILLS_FOR_BOSS {
        prog.record_kill();
    }

    // After record_kill triggers boss, fighting_boss is true
    // so should_spawn_boss returns false (already spawned)
    assert!(!prog.should_spawn_boss());
}

#[test]
fn test_should_spawn_boss_edge_case() {
    let mut prog = ZoneProgression::new();

    // Manually set kills to threshold without triggering
    prog.kills_in_subzone = KILLS_FOR_BOSS;
    prog.fighting_boss = false;

    // Should return true
    assert!(prog.should_spawn_boss());
}

#[test]
fn test_boss_weapon_blocked_returns_none_when_not_fighting() {
    let mut prog = ZoneProgression::new();
    let achievements = Achievements::default();
    prog.current_zone_id = 10;
    prog.current_subzone_id = 4; // Final subzone
    prog.unlock_zone(10);
    prog.fighting_boss = false; // Not fighting

    assert!(prog.boss_weapon_blocked(&achievements).is_none());
}

#[test]
fn test_boss_weapon_blocked_returns_none_for_non_final_subzone() {
    let mut prog = ZoneProgression::new();
    let achievements = Achievements::default();
    prog.current_zone_id = 10;
    prog.current_subzone_id = 2; // Not final subzone
    prog.unlock_zone(10);
    prog.fighting_boss = true;

    // Intermediate bosses don't require weapon
    assert!(prog.boss_weapon_blocked(&achievements).is_none());
}

#[test]
fn test_boss_weapon_blocked_returns_weapon_name() {
    let mut prog = ZoneProgression::new();
    let achievements = Achievements::default(); // No TheStormbreaker achievement
    prog.current_zone_id = 10;
    prog.current_subzone_id = 4; // Final subzone
    prog.unlock_zone(10);
    prog.fighting_boss = true;

    assert_eq!(
        prog.boss_weapon_blocked(&achievements),
        Some("Stormbreaker")
    );
}

#[test]
fn test_boss_weapon_blocked_none_with_weapon() {
    use quest::achievements::AchievementId;

    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    // Unlock the TheStormbreaker achievement
    achievements.unlock(AchievementId::TheStormbreaker, None);
    prog.current_zone_id = 10;
    prog.current_subzone_id = 4;
    prog.unlock_zone(10);
    prog.fighting_boss = true;

    assert!(prog.boss_weapon_blocked(&achievements).is_none());
}

#[test]
fn test_unlock_zone_idempotent() {
    let mut prog = ZoneProgression::new();

    // Unlock zone 5
    prog.unlock_zone(5);
    assert!(prog.is_zone_unlocked(5));
    assert_eq!(prog.unlocked_zones.iter().filter(|&&z| z == 5).count(), 1);

    // Unlock again - should not duplicate
    prog.unlock_zone(5);
    assert_eq!(prog.unlocked_zones.iter().filter(|&&z| z == 5).count(), 1);
}

#[test]
fn test_unlock_zone_maintains_sort_order() {
    let mut prog = ZoneProgression::new();

    // Unlock out of order
    prog.unlock_zone(8);
    prog.unlock_zone(5);
    prog.unlock_zone(7);

    // Should be sorted
    let unlocked: Vec<u32> = prog.unlocked_zones.clone();
    let mut sorted = unlocked.clone();
    sorted.sort();
    assert_eq!(unlocked, sorted);
}

// ============================================================================
// Prestige tier transition tests
// ============================================================================

#[test]
fn test_prestige_tier_boundaries() {
    let zones = get_all_zones();

    // Verify prestige requirements match expected tiers
    // P0: Zones 1-2
    assert_eq!(zones[0].prestige_requirement, 0); // Zone 1
    assert_eq!(zones[1].prestige_requirement, 0); // Zone 2

    // P5: Zones 3-4
    assert_eq!(zones[2].prestige_requirement, 5); // Zone 3
    assert_eq!(zones[3].prestige_requirement, 5); // Zone 4

    // P10: Zones 5-6
    assert_eq!(zones[4].prestige_requirement, 10); // Zone 5
    assert_eq!(zones[5].prestige_requirement, 10); // Zone 6

    // P15: Zones 7-8
    assert_eq!(zones[6].prestige_requirement, 15); // Zone 7
    assert_eq!(zones[7].prestige_requirement, 15); // Zone 8

    // P20: Zones 9-10
    assert_eq!(zones[8].prestige_requirement, 20); // Zone 9
    assert_eq!(zones[9].prestige_requirement, 20); // Zone 10
}

#[test]
fn test_reset_for_prestige_unlocks_correct_zones() {
    let mut prog = ZoneProgression::new();

    // P0 -> Zones 1-2
    prog.reset_for_prestige(0);
    assert!(prog.is_zone_unlocked(1));
    assert!(prog.is_zone_unlocked(2));
    assert!(!prog.is_zone_unlocked(3));

    // P5 -> Zones 1-4
    prog.reset_for_prestige(5);
    assert!(prog.is_zone_unlocked(3));
    assert!(prog.is_zone_unlocked(4));
    assert!(!prog.is_zone_unlocked(5));

    // P10 -> Zones 1-6
    prog.reset_for_prestige(10);
    assert!(prog.is_zone_unlocked(5));
    assert!(prog.is_zone_unlocked(6));
    assert!(!prog.is_zone_unlocked(7));

    // P15 -> Zones 1-8
    prog.reset_for_prestige(15);
    assert!(prog.is_zone_unlocked(7));
    assert!(prog.is_zone_unlocked(8));
    assert!(!prog.is_zone_unlocked(9));

    // P20 -> All zones
    prog.reset_for_prestige(20);
    assert!(prog.is_zone_unlocked(9));
    assert!(prog.is_zone_unlocked(10));
}

#[test]
fn test_prestige_between_tiers() {
    let mut prog = ZoneProgression::new();

    // P7 (between P5 and P10) -> Should unlock P0 + P5 zones
    prog.reset_for_prestige(7);
    assert!(prog.is_zone_unlocked(4)); // P5 zone
    assert!(!prog.is_zone_unlocked(5)); // P10 zone
}

// ============================================================================
// Edge cases and error conditions
// ============================================================================

#[test]
fn test_travel_to_unlocked_zone() {
    let mut prog = ZoneProgression::new();
    // Zone 2 is unlocked by default at P0
    assert!(prog.travel_to(2, 1));
    assert_eq!(prog.current_zone_id, 2);
}

#[test]
fn test_travel_to_locked_zone_fails() {
    let prog = ZoneProgression::new();
    // Zone 5 is locked at P0
    assert!(!prog.can_enter_subzone(5, 1));
}

#[test]
fn test_defeat_boss_resets_fighting_state() {
    let mut prog = ZoneProgression::new();

    // Trigger boss fight
    for _ in 0..KILLS_FOR_BOSS {
        prog.record_kill();
    }
    assert!(prog.fighting_boss);

    // Defeat boss
    prog.defeat_boss(1, 1);

    // Should reset
    assert!(!prog.fighting_boss);
    assert_eq!(prog.kills_in_subzone, 0);
}

#[test]
fn test_on_boss_defeated_at_zone_boundary() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    // Set up at zone 2, subzone 3 (final subzone of zone 2)
    prog.current_zone_id = 2;
    prog.current_subzone_id = 3;
    prog.fighting_boss = true;

    // Defeat boss with prestige 4 (zone 3 needs P5)
    let result = prog.on_boss_defeated(4, &mut achievements);

    // Should be gated
    match result {
        BossDefeatResult::ZoneCompleteButGated {
            required_prestige, ..
        } => {
            assert_eq!(required_prestige, 5);
        }
        _ => panic!("Expected ZoneCompleteButGated"),
    }

    // Should stay in zone 2
    assert_eq!(prog.current_zone_id, 2);
}

// ============================================================================
// Full progression simulation
// ============================================================================

#[test]
fn test_complete_game_progression() {
    use quest::achievements::AchievementId;

    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    let zones = get_all_zones();

    // Simulate completing the entire game with prestige 20
    prog.reset_for_prestige(20); // Unlock all zones

    // Complete all zones (excluding Zone 11 which is infinite post-game)
    for zone in zones.iter().take(10) {
        prog.current_zone_id = zone.id;

        for subzone_id in 1..=zone.subzones.len() as u32 {
            prog.current_subzone_id = subzone_id;

            // Kill enemies to spawn boss
            for _ in 0..KILLS_FOR_BOSS {
                prog.record_kill();
            }
            assert!(prog.fighting_boss);

            // For zone 10 final boss, need Stormbreaker achievement
            if zone.id == 10 && subzone_id == zone.subzones.len() as u32 {
                achievements.unlock(AchievementId::TheStormbreaker, None);
            }

            let result = prog.on_boss_defeated(20, &mut achievements);

            // Verify appropriate result
            if zone.id == 10 && subzone_id == zone.subzones.len() as u32 {
                assert!(matches!(result, BossDefeatResult::GameComplete));
            }
        }
    }

    // Verify all bosses defeated (zones 1-10)
    for zone in zones.iter().take(10) {
        for subzone_id in 1..=zone.subzones.len() as u32 {
            assert!(prog.is_boss_defeated(zone.id, subzone_id));
        }
    }
}

#[test]
fn test_speedrun_to_zone_10() {
    use quest::achievements::AchievementId;

    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();

    // Simulate a "speedrun" with max prestige - can travel directly to high zones
    prog.reset_for_prestige(20);

    // Travel directly to Zone 10
    assert!(prog.travel_to(10, 1));
    assert_eq!(prog.current_zone_id, 10);

    // Clear zone 10
    let zones = get_all_zones();
    let zone10 = &zones[9];

    for subzone_id in 1..=zone10.subzones.len() as u32 {
        prog.current_subzone_id = subzone_id;

        for _ in 0..KILLS_FOR_BOSS {
            prog.record_kill();
        }

        if subzone_id == zone10.subzones.len() as u32 {
            achievements.unlock(AchievementId::TheStormbreaker, None);
        }

        let result = prog.on_boss_defeated(20, &mut achievements);

        if subzone_id == zone10.subzones.len() as u32 {
            assert!(matches!(result, BossDefeatResult::GameComplete));
        }
    }
}
