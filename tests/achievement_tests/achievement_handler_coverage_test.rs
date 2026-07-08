//! Coverage tests for achievements/handlers.rs — targeting branches not covered
//! by the existing achievement_handlers_test.rs and types.rs unit tests.
//!
//! Focus areas:
//! - All Deep event handlers (on_deep_*)
//! - sync_from_deep with various states
//! - sync_from_haven edge cases
//! - refresh_progress for Deep series
//! - Enhancement level boundary cases

#![allow(clippy::field_reassign_with_default)]

use quest::achievements::types::{AchievementId, Achievements};
use quest::achievements::{MinigameDifficulty, MinigameType};
use quest::haven::types::HavenRoomId;
use std::collections::HashMap;

// =========================================================================
// Helper utilities
// =========================================================================

use super::helpers::{
    build_haven_max_tiers as build_all_rooms_at_max, build_haven_tiers as build_all_rooms_at_tier,
};

// =========================================================================
// on_deep_discovered
// =========================================================================

#[test]
fn test_on_deep_discovered_unlocks_achievement() {
    let mut ach = Achievements::default();
    assert!(!ach.is_unlocked(AchievementId::TheDeepDiscovered));
    ach.on_deep_discovered(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::TheDeepDiscovered));
}

#[test]
fn test_on_deep_discovered_idempotent() {
    let mut ach = Achievements::default();
    ach.on_deep_discovered(Some("Hero"));
    ach.on_deep_discovered(Some("Hero"));
    assert_eq!(ach.unlocked_count(), 1);
}

#[test]
fn test_on_deep_discovered_none_character_name() {
    let mut ach = Achievements::default();
    ach.on_deep_discovered(None);
    assert!(ach.is_unlocked(AchievementId::TheDeepDiscovered));
    let record = ach.unlocked.get(&AchievementId::TheDeepDiscovered).unwrap();
    assert!(record.character_name.is_none());
}

// =========================================================================
// on_deep_mission_complete — Mission completion milestones
// =========================================================================

#[test]
fn test_first_mission_complete_at_1() {
    let mut ach = Achievements::default();
    ach.on_deep_mission_complete(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FirstMissionComplete));
    assert_eq!(ach.total_deep_missions_completed, 1);
}

#[test]
fn test_deep_missions_x_at_10() {
    let mut ach = Achievements::default();
    ach.total_deep_missions_completed = 9;
    ach.on_deep_mission_complete(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DeepMissionsX));
}

#[test]
fn test_deep_missions_xxv_at_25() {
    let mut ach = Achievements::default();
    ach.total_deep_missions_completed = 24;
    ach.on_deep_mission_complete(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DeepMissionsXXV));
}

#[test]
fn test_deep_missions_l_at_50() {
    let mut ach = Achievements::default();
    ach.total_deep_missions_completed = 49;
    ach.on_deep_mission_complete(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DeepMissionsL));
}

#[test]
fn test_deep_missions_c_at_100() {
    let mut ach = Achievements::default();
    ach.total_deep_missions_completed = 99;
    ach.on_deep_mission_complete(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DeepMissionsC));
}

#[test]
fn test_deep_missions_counter_increments() {
    let mut ach = Achievements::default();
    ach.on_deep_mission_complete(Some("Hero"));
    ach.on_deep_mission_complete(Some("Hero"));
    ach.on_deep_mission_complete(Some("Hero"));
    assert_eq!(ach.total_deep_missions_completed, 3);
}

#[test]
fn test_deep_mission_before_threshold_no_unlock() {
    let mut ach = Achievements::default();
    // 9 missions — DeepMissionsX needs 10
    for _ in 0..9 {
        ach.on_deep_mission_complete(Some("Hero"));
    }
    assert!(ach.is_unlocked(AchievementId::FirstMissionComplete));
    assert!(!ach.is_unlocked(AchievementId::DeepMissionsX));
}

// =========================================================================
// on_deep_breakthrough — FirstBreakthrough + layer milestones
// =========================================================================

#[test]
fn test_first_breakthrough_unlocked_on_any_layer() {
    let mut ach = Achievements::default();
    assert!(!ach.is_unlocked(AchievementId::FirstBreakthrough));
    ach.on_deep_breakthrough(1, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FirstBreakthrough));
}

#[test]
fn test_breakthrough_tracks_highest_layer() {
    let mut ach = Achievements::default();
    ach.on_deep_breakthrough(8, Some("Hero"));
    assert_eq!(ach.highest_deep_layer, 8);

    // Lower layer should not decrease highest
    ach.on_deep_breakthrough(3, Some("Hero"));
    assert_eq!(ach.highest_deep_layer, 8);

    // Higher layer should update
    ach.on_deep_breakthrough(15, Some("Hero"));
    assert_eq!(ach.highest_deep_layer, 15);
}

#[test]
fn test_power_core_i_at_layer_3() {
    let mut ach = Achievements::default();
    ach.on_deep_breakthrough(3, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::PowerCoreI));
    assert!(!ach.is_unlocked(AchievementId::Layer5Cleared));
}

#[test]
fn test_layer5_cleared_at_layer_5() {
    let mut ach = Achievements::default();
    ach.on_deep_breakthrough(5, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::Layer5Cleared));
    assert!(ach.is_unlocked(AchievementId::PowerCoreI));
}

#[test]
fn test_power_core_ii_at_layer_7() {
    let mut ach = Achievements::default();
    ach.on_deep_breakthrough(7, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::PowerCoreII));
    assert!(ach.is_unlocked(AchievementId::Layer5Cleared));
}

#[test]
fn test_layer10_cleared_at_layer_10() {
    let mut ach = Achievements::default();
    ach.on_deep_breakthrough(10, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::Layer10Cleared));
}

#[test]
fn test_power_core_iii_at_layer_12() {
    let mut ach = Achievements::default();
    ach.on_deep_breakthrough(12, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::PowerCoreIII));
    assert!(ach.is_unlocked(AchievementId::Layer10Cleared));
}

#[test]
fn test_power_core_iv_at_layer_18() {
    let mut ach = Achievements::default();
    ach.on_deep_breakthrough(18, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::PowerCoreIV));
    assert!(ach.is_unlocked(AchievementId::Layer15Cleared));
}

#[test]
fn test_layer25_cleared_at_layer_25() {
    let mut ach = Achievements::default();
    ach.on_deep_breakthrough(25, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::Layer25Cleared));
    assert!(!ach.is_unlocked(AchievementId::VoidExplorer));
}

#[test]
fn test_void_explorer_at_layer_26() {
    let mut ach = Achievements::default();
    ach.on_deep_breakthrough(26, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::VoidExplorer));
    assert!(ach.is_unlocked(AchievementId::Layer25Cleared));
}

#[test]
fn test_breakthrough_at_layer_below_3_unlocks_only_first_breakthrough() {
    let mut ach = Achievements::default();
    ach.on_deep_breakthrough(2, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FirstBreakthrough));
    assert!(!ach.is_unlocked(AchievementId::PowerCoreI));
    assert!(!ach.is_unlocked(AchievementId::Layer5Cleared));
}

#[test]
fn test_breakthrough_layer_overshoot_unlocks_all_prior() {
    let mut ach = Achievements::default();
    ach.on_deep_breakthrough(30, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FirstBreakthrough));
    // Layer milestones
    assert!(ach.is_unlocked(AchievementId::Layer5Cleared));
    assert!(ach.is_unlocked(AchievementId::Layer10Cleared));
    assert!(ach.is_unlocked(AchievementId::Layer15Cleared));
    assert!(ach.is_unlocked(AchievementId::Layer20Cleared));
    assert!(ach.is_unlocked(AchievementId::Layer25Cleared));
    assert!(ach.is_unlocked(AchievementId::VoidExplorer));
    // Power Core milestones
    assert!(ach.is_unlocked(AchievementId::PowerCoreI));
    assert!(ach.is_unlocked(AchievementId::PowerCoreII));
    assert!(ach.is_unlocked(AchievementId::PowerCoreIII));
    assert!(ach.is_unlocked(AchievementId::PowerCoreIV));
    assert!(ach.is_unlocked(AchievementId::PowerCoreV));
    assert!(ach.is_unlocked(AchievementId::PowerCoreVI));
}

// =========================================================================
// on_deep_guild_rank_up — Guild rank milestones
// =========================================================================

#[test]
fn test_guild_rank_2_unlocks_at_rank_2() {
    let mut ach = Achievements::default();
    ach.on_deep_guild_rank_up(2, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::GuildRank2));
    assert!(!ach.is_unlocked(AchievementId::GuildRank3));
}

#[test]
fn test_guild_rank_3_unlocks_at_rank_3() {
    let mut ach = Achievements::default();
    ach.on_deep_guild_rank_up(3, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::GuildRank3));
    assert!(ach.is_unlocked(AchievementId::GuildRank2));
}

#[test]
fn test_guild_rank_4_unlocks_at_rank_4() {
    let mut ach = Achievements::default();
    ach.on_deep_guild_rank_up(4, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::GuildRank4));
}

#[test]
fn test_guild_rank_5_unlocks_at_rank_5() {
    let mut ach = Achievements::default();
    ach.on_deep_guild_rank_up(5, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::GuildRank5));
    assert!(ach.is_unlocked(AchievementId::GuildRank4));
    assert!(ach.is_unlocked(AchievementId::GuildRank3));
    assert!(ach.is_unlocked(AchievementId::GuildRank2));
}

#[test]
fn test_guild_rank_tracks_highest() {
    let mut ach = Achievements::default();
    ach.on_deep_guild_rank_up(4, Some("Hero"));
    assert_eq!(ach.highest_guild_rank, 4);

    // Lower rank should not decrease highest
    ach.on_deep_guild_rank_up(2, Some("Hero"));
    assert_eq!(ach.highest_guild_rank, 4);

    // Higher rank should update
    ach.on_deep_guild_rank_up(5, Some("Hero"));
    assert_eq!(ach.highest_guild_rank, 5);
}

#[test]
fn test_guild_rank_1_does_not_unlock_any_achievement() {
    // Rank 1 is starting rank — no achievement for it
    let mut ach = Achievements::default();
    ach.on_deep_guild_rank_up(1, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::GuildRank2));
}

// =========================================================================
// on_deep_merc_lost
// =========================================================================

#[test]
fn test_on_deep_merc_lost_unlocks_first_merc_lost() {
    let mut ach = Achievements::default();
    assert!(!ach.is_unlocked(AchievementId::FirstMercLost));
    ach.on_deep_merc_lost(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FirstMercLost));
}

#[test]
fn test_on_deep_merc_lost_idempotent() {
    let mut ach = Achievements::default();
    ach.on_deep_merc_lost(Some("Hero"));
    let count_before = ach.unlocked_count();
    ach.on_deep_merc_lost(Some("Hero"));
    assert_eq!(ach.unlocked_count(), count_before);
}

#[test]
fn test_on_deep_merc_lost_records_character_name() {
    let mut ach = Achievements::default();
    ach.on_deep_merc_lost(Some("Khalan"));
    let record = ach.unlocked.get(&AchievementId::FirstMercLost).unwrap();
    assert_eq!(record.character_name, Some("Khalan".to_string()));
}

// =========================================================================
// on_deep_gateway_opened
// =========================================================================

#[test]
fn test_on_deep_gateway_opened_unlocks_achievement() {
    let mut ach = Achievements::default();
    assert!(!ach.is_unlocked(AchievementId::GatewayOpened));
    ach.on_deep_gateway_opened(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::GatewayOpened));
}

#[test]
fn test_on_deep_gateway_opened_idempotent() {
    let mut ach = Achievements::default();
    ach.on_deep_gateway_opened(Some("Hero"));
    let count_before = ach.unlocked_count();
    ach.on_deep_gateway_opened(Some("Hero"));
    assert_eq!(ach.unlocked_count(), count_before);
}

#[test]
fn test_on_deep_gateway_opened_none_character() {
    let mut ach = Achievements::default();
    ach.on_deep_gateway_opened(None);
    assert!(ach.is_unlocked(AchievementId::GatewayOpened));
    let record = ach.unlocked.get(&AchievementId::GatewayOpened).unwrap();
    assert!(record.character_name.is_none());
}

// =========================================================================
// on_soulforge_discovered
// =========================================================================

#[test]
fn test_soulforge_discovered_sets_achievement() {
    let mut ach = Achievements::default();
    assert!(!ach.is_unlocked(AchievementId::SoulforgeDiscovered));
    ach.on_soulforge_discovered(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SoulforgeDiscovered));
}

#[test]
fn test_soulforge_discovered_idempotent() {
    let mut ach = Achievements::default();
    ach.on_soulforge_discovered(Some("Hero"));
    ach.on_soulforge_discovered(Some("Hero"));
    assert_eq!(ach.unlocked_count(), 1);
}

// =========================================================================
// on_storms_end
// =========================================================================

#[test]
fn test_storms_end_unlocks_achievement() {
    let mut ach = Achievements::default();
    assert!(!ach.is_unlocked(AchievementId::StormsEnd));
    ach.on_storms_end(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::StormsEnd));
}

#[test]
fn test_storms_end_records_character_name() {
    let mut ach = Achievements::default();
    ach.on_storms_end(Some("Stormcaller"));
    let record = ach.unlocked.get(&AchievementId::StormsEnd).unwrap();
    assert_eq!(record.character_name, Some("Stormcaller".to_string()));
}

// =========================================================================
// on_enhancement_upgraded — boundary conditions not covered elsewhere
// =========================================================================

#[test]
fn test_enhancement_level_4_no_journeyman() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(4, &[4, 0, 0, 0, 0, 0, 0], 4, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::ApprenticeSmith));
    assert!(!ach.is_unlocked(AchievementId::JourneymanSmith)); // needs 5
    assert!(!ach.is_unlocked(AchievementId::FullyTempered)); // needs all 7 slots at 4
}

#[test]
fn test_enhancement_level_2_only_apprentice() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(2, &[2, 0, 0, 0, 0, 0, 0], 2, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::ApprenticeSmith));
    assert!(!ach.is_unlocked(AchievementId::JourneymanSmith));
    assert!(!ach.is_unlocked(AchievementId::SoulforgeAdept));
    assert!(!ach.is_unlocked(AchievementId::SoulforgeSavant));
    assert!(!ach.is_unlocked(AchievementId::SoulforgeMaster));
    assert!(!ach.is_unlocked(AchievementId::SoulforgeGrandmaster));
    assert!(!ach.is_unlocked(AchievementId::SoulforgeAscendant));
}

#[test]
fn test_enhancement_fully_tempered_requires_all_slots_at_4() {
    // 6 slots at 4, one at 3 — should NOT unlock
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(4, &[4, 4, 4, 4, 4, 4, 3], 27, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::FullyTempered));

    // Now all 7 at 4
    let mut ach2 = Achievements::default();
    ach2.on_enhancement_upgraded(4, &[4, 4, 4, 4, 4, 4, 4], 28, Some("Hero"));
    assert!(ach2.is_unlocked(AchievementId::FullyTempered));
}

#[test]
fn test_enhancement_soul_convergence_requires_all_slots_at_7() {
    // 6 slots at 7, one at 6 — should NOT unlock
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(7, &[7, 7, 7, 7, 7, 7, 6], 48, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::SoulConvergence));

    // All 7 at 7
    let mut ach2 = Achievements::default();
    ach2.on_enhancement_upgraded(7, &[7, 7, 7, 7, 7, 7, 7], 49, Some("Hero"));
    assert!(ach2.is_unlocked(AchievementId::SoulConvergence));
}

#[test]
fn test_enhancement_persistent_hammering_at_exactly_100() {
    let mut ach = Achievements::default();
    // 99 attempts — not yet
    ach.on_enhancement_upgraded(1, &[1, 0, 0, 0, 0, 0, 0], 99, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::PersistentHammering));

    // 100 attempts — triggers
    let mut ach2 = Achievements::default();
    ach2.on_enhancement_upgraded(1, &[1, 0, 0, 0, 0, 0, 0], 100, Some("Hero"));
    assert!(ach2.is_unlocked(AchievementId::PersistentHammering));
}

#[test]
fn test_enhancement_all_levels_simultaneously() {
    // Level 10 with all slots at 10 and 200 attempts — should unlock entire chain
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(10, &[10, 10, 10, 10, 10, 10, 10], 200, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::ApprenticeSmith));
    assert!(ach.is_unlocked(AchievementId::JourneymanSmith));
    assert!(ach.is_unlocked(AchievementId::SoulforgeAdept));
    assert!(ach.is_unlocked(AchievementId::SoulforgeSavant));
    assert!(ach.is_unlocked(AchievementId::SoulforgeMaster));
    assert!(ach.is_unlocked(AchievementId::SoulforgeGrandmaster));
    assert!(ach.is_unlocked(AchievementId::SoulforgeAscendant));
    assert!(ach.is_unlocked(AchievementId::FullyTempered));
    assert!(ach.is_unlocked(AchievementId::SoulConvergence));
    assert!(ach.is_unlocked(AchievementId::PersistentHammering));
}

// =========================================================================
// on_zone_fully_cleared — all 10 zones + zone 11 + edge cases
// =========================================================================

#[test]
fn test_zone_completion_all_individual_zones() {
    let cases = [
        (1u32, AchievementId::Zone1Complete),
        (2, AchievementId::Zone2Complete),
        (3, AchievementId::Zone3Complete),
        (4, AchievementId::Zone4Complete),
        (5, AchievementId::Zone5Complete),
        (6, AchievementId::Zone6Complete),
        (7, AchievementId::Zone7Complete),
        (8, AchievementId::Zone8Complete),
        (9, AchievementId::Zone9Complete),
        (10, AchievementId::Zone10Complete),
    ];
    for (zone_id, expected_id) in cases {
        let mut ach = Achievements::default();
        ach.on_zone_fully_cleared(zone_id, Some("Hero"));
        assert!(
            ach.is_unlocked(expected_id),
            "Zone {} should unlock {:?}",
            zone_id,
            expected_id
        );
        // Verify counter was incremented
        assert_eq!(ach.zones_fully_cleared, 1);
    }
}

#[test]
fn test_zone_11_unlocks_beyond_infinity_and_increments_expanse_counter() {
    let mut ach = Achievements::default();
    ach.on_zone_fully_cleared(11, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::BeyondInfinity));
    assert_eq!(ach.expanse_cycles_completed, 1);
    assert_eq!(ach.zones_fully_cleared, 1);
}

#[test]
fn test_zone_11_does_not_unlock_zone_10_complete() {
    let mut ach = Achievements::default();
    ach.on_zone_fully_cleared(11, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::Zone10Complete));
}

#[test]
fn test_zone_0_does_nothing_except_increment_counter() {
    let mut ach = Achievements::default();
    ach.on_zone_fully_cleared(0, Some("Hero"));
    assert_eq!(ach.zones_fully_cleared, 1);
    assert!(!ach.is_unlocked(AchievementId::Zone1Complete));
    assert!(!ach.is_unlocked(AchievementId::BeyondInfinity));
}

#[test]
fn test_zone_12_does_nothing_except_increment_counter() {
    let mut ach = Achievements::default();
    ach.on_zone_fully_cleared(12, Some("Hero"));
    assert_eq!(ach.zones_fully_cleared, 1);
    assert!(!ach.is_unlocked(AchievementId::BeyondInfinity));
}

// =========================================================================
// sync_from_deep — state synchronization for Deep system
// =========================================================================

#[test]
fn test_sync_from_deep_not_discovered_skips_discovery_achievement() {
    let mut ach = Achievements::default();
    ach.sync_from_deep(false, 1, 0, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::TheDeepDiscovered));
}

#[test]
fn test_sync_from_deep_discovered_unlocks_achievement() {
    let mut ach = Achievements::default();
    ach.sync_from_deep(true, 1, 0, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::TheDeepDiscovered));
}

#[test]
fn test_sync_from_deep_guild_rank_1_skips_rank_achievements() {
    // Rank 1 is the starting rank — sync should not unlock any rank achievement
    let mut ach = Achievements::default();
    ach.sync_from_deep(true, 1, 0, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::GuildRank2));
}

#[test]
fn test_sync_from_deep_guild_rank_2_unlocks_achievement() {
    let mut ach = Achievements::default();
    ach.sync_from_deep(true, 2, 0, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::GuildRank2));
    assert!(!ach.is_unlocked(AchievementId::GuildRank3));
}

#[test]
fn test_sync_from_deep_guild_rank_5_unlocks_all() {
    let mut ach = Achievements::default();
    ach.sync_from_deep(true, 5, 0, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::GuildRank2));
    assert!(ach.is_unlocked(AchievementId::GuildRank3));
    assert!(ach.is_unlocked(AchievementId::GuildRank4));
    assert!(ach.is_unlocked(AchievementId::GuildRank5));
}

#[test]
fn test_sync_from_deep_guild_rank_updates_highest() {
    let mut ach = Achievements::default();
    ach.sync_from_deep(false, 3, 0, Some("Hero"));
    assert_eq!(ach.highest_guild_rank, 3);
}

#[test]
fn test_sync_from_deep_guild_rank_does_not_decrease_existing_highest() {
    let mut ach = Achievements::default();
    ach.highest_guild_rank = 5;
    ach.sync_from_deep(false, 2, 0, Some("Hero"));
    // The existing highest should be preserved
    assert_eq!(ach.highest_guild_rank, 5);
}

#[test]
fn test_sync_from_deep_layer_0_skips_layer_achievements() {
    let mut ach = Achievements::default();
    ach.sync_from_deep(false, 1, 0, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::FirstBreakthrough));
    assert!(!ach.is_unlocked(AchievementId::Layer5Cleared));
    assert!(!ach.is_unlocked(AchievementId::PowerCoreI));
}

#[test]
fn test_sync_from_deep_layer_1_unlocks_first_breakthrough() {
    let mut ach = Achievements::default();
    ach.sync_from_deep(false, 1, 1, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FirstBreakthrough));
    assert!(!ach.is_unlocked(AchievementId::PowerCoreI));
}

#[test]
fn test_sync_from_deep_layer_10_unlocks_layer_milestones() {
    let mut ach = Achievements::default();
    ach.sync_from_deep(false, 1, 10, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FirstBreakthrough));
    // Layer milestones at 5 and 10
    assert!(ach.is_unlocked(AchievementId::Layer5Cleared));
    assert!(ach.is_unlocked(AchievementId::Layer10Cleared));
    assert!(!ach.is_unlocked(AchievementId::Layer15Cleared));
    // Power Core milestones at 3 and 7
    assert!(ach.is_unlocked(AchievementId::PowerCoreI));
    assert!(ach.is_unlocked(AchievementId::PowerCoreII));
    assert!(!ach.is_unlocked(AchievementId::PowerCoreIII));
}

#[test]
fn test_sync_from_deep_layer_updates_highest() {
    let mut ach = Achievements::default();
    ach.sync_from_deep(false, 1, 20, Some("Hero"));
    assert_eq!(ach.highest_deep_layer, 20);
}

#[test]
fn test_sync_from_deep_layer_does_not_decrease_existing_highest() {
    let mut ach = Achievements::default();
    ach.highest_deep_layer = 25;
    ach.sync_from_deep(false, 1, 10, Some("Hero"));
    // Existing highest should be preserved
    assert_eq!(ach.highest_deep_layer, 25);
}

#[test]
fn test_sync_from_deep_full_progression() {
    let mut ach = Achievements::default();
    ach.sync_from_deep(true, 5, 26, Some("GuildMaster"));

    // Discovery
    assert!(ach.is_unlocked(AchievementId::TheDeepDiscovered));

    // Guild rank milestones
    assert!(ach.is_unlocked(AchievementId::GuildRank2));
    assert!(ach.is_unlocked(AchievementId::GuildRank3));
    assert!(ach.is_unlocked(AchievementId::GuildRank4));
    assert!(ach.is_unlocked(AchievementId::GuildRank5));

    // Layer milestones
    assert!(ach.is_unlocked(AchievementId::FirstBreakthrough));
    assert!(ach.is_unlocked(AchievementId::Layer5Cleared));
    assert!(ach.is_unlocked(AchievementId::Layer10Cleared));
    assert!(ach.is_unlocked(AchievementId::Layer15Cleared));
    assert!(ach.is_unlocked(AchievementId::Layer20Cleared));
    assert!(ach.is_unlocked(AchievementId::Layer25Cleared));
    assert!(ach.is_unlocked(AchievementId::VoidExplorer));
    // Power Core milestones
    assert!(ach.is_unlocked(AchievementId::PowerCoreI));
    assert!(ach.is_unlocked(AchievementId::PowerCoreII));
    assert!(ach.is_unlocked(AchievementId::PowerCoreIII));
    assert!(ach.is_unlocked(AchievementId::PowerCoreIV));
    assert!(ach.is_unlocked(AchievementId::PowerCoreV));

    // Counters updated
    assert_eq!(ach.highest_guild_rank, 5);
    assert_eq!(ach.highest_deep_layer, 26);
}

#[test]
fn test_sync_from_deep_calls_refresh_progress() {
    // After sync, progress bars should be populated for Deep series
    let mut ach = Achievements::default();
    ach.total_deep_missions_completed = 7;
    ach.sync_from_deep(false, 1, 3, Some("Hero"));

    // refresh_progress is called internally — deep missions progress should be set
    let p = ach
        .get_progress(AchievementId::DeepMissionsX)
        .expect("progress should exist after refresh");
    assert_eq!(p.current, 7);
    assert_eq!(p.target, 10);
}

// =========================================================================
// sync_from_haven — Haven state synchronization edge cases
// =========================================================================

#[test]
fn test_sync_from_haven_not_discovered_no_achievement() {
    let mut ach = Achievements::default();
    ach.sync_from_haven(false, &HashMap::new(), Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::HavenDiscovered));
}

#[test]
fn test_sync_from_haven_discovered_empty_rooms() {
    let mut ach = Achievements::default();
    ach.sync_from_haven(true, &HashMap::new(), Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenDiscovered));
    assert!(!ach.is_unlocked(AchievementId::HavenBuilderI));
}

#[test]
fn test_sync_from_haven_all_t1_unlocks_builder_i() {
    let mut ach = Achievements::default();
    let tiers = build_all_rooms_at_tier(1);
    ach.sync_from_haven(true, &tiers, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderI));
    assert!(!ach.is_unlocked(AchievementId::HavenBuilderII));
}

#[test]
fn test_sync_from_haven_all_t2_unlocks_builder_i_and_ii() {
    let mut ach = Achievements::default();
    let tiers = build_all_rooms_at_tier(2);
    ach.sync_from_haven(true, &tiers, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderI));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderII));
    assert!(!ach.is_unlocked(AchievementId::HavenArchitect));
}

#[test]
fn test_sync_from_haven_max_tiers_unlocks_architect() {
    let mut ach = Achievements::default();
    let tiers = build_all_rooms_at_max();
    ach.sync_from_haven(true, &tiers, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderI));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderII));
    assert!(ach.is_unlocked(AchievementId::HavenArchitect));
}

#[test]
fn test_sync_from_haven_partial_rooms_no_builder_i() {
    let mut ach = Achievements::default();
    // Only 2 rooms at T1, not all
    let mut tiers = HashMap::new();
    tiers.insert(HavenRoomId::Hearthstone, 1u8);
    tiers.insert(HavenRoomId::Armory, 1u8);
    ach.sync_from_haven(true, &tiers, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenDiscovered));
    assert!(!ach.is_unlocked(AchievementId::HavenBuilderI));
}

#[test]
fn test_sync_from_haven_stormforge_excluded_from_tier_check() {
    // StormForge is excluded from tier checks — all other rooms at T1 should unlock BuilderI
    let mut tiers = build_all_rooms_at_tier(1);
    // Add StormForge at T0 (not built) — should not block BuilderI
    tiers.insert(HavenRoomId::StormForge, 0);
    let mut ach = Achievements::default();
    ach.sync_from_haven(true, &tiers, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderI));
}

#[test]
fn test_sync_from_haven_architect_rooms_with_low_max_tier_count_as_t3() {
    // FishingDock has max_tier=4, but T3 >= 3 should count as passing the Architect check.
    let mut tiers: HashMap<HavenRoomId, u8> = HavenRoomId::ALL
        .iter()
        .filter(|r| **r != HavenRoomId::StormForge)
        .map(|r| (*r, 3))
        .collect();
    // Ensure FishingDock is exactly at T3 (below its max of 4)
    tiers.insert(HavenRoomId::FishingDock, 3);

    let mut ach = Achievements::default();
    ach.sync_from_haven(true, &tiers, Some("Hero"));
    // tier(3) >= 3 → passes even though max is 4
    assert!(ach.is_unlocked(AchievementId::HavenArchitect));
}

// =========================================================================
// refresh_progress — verifies all series are updated
// =========================================================================

#[test]
fn test_refresh_progress_updates_deep_mission_series() {
    let mut ach = Achievements::default();
    ach.total_deep_missions_completed = 15;
    ach.refresh_progress();

    let p = ach
        .get_progress(AchievementId::DeepMissionsX)
        .expect("should have progress for DeepMissionsX");
    assert_eq!(p.current, 15);
    assert_eq!(p.target, 10); // already exceeded, but progress should still record it
}

#[test]
fn test_refresh_progress_updates_deep_layer_series() {
    let mut ach = Achievements::default();
    ach.highest_deep_layer = 8;
    ach.refresh_progress();

    let p = ach
        .get_progress(AchievementId::Layer10Cleared)
        .expect("should have progress for Layer10Cleared");
    assert_eq!(p.current, 8);
    assert_eq!(p.target, 10);
}

#[test]
fn test_refresh_progress_updates_guild_rank_series() {
    let mut ach = Achievements::default();
    ach.highest_guild_rank = 1;
    ach.refresh_progress();

    let p = ach
        .get_progress(AchievementId::GuildRank2)
        .expect("should have progress for GuildRank2");
    assert_eq!(p.current, 1);
    assert_eq!(p.target, 2);
}

#[test]
fn test_refresh_progress_updates_all_series_simultaneously() {
    let mut ach = Achievements::default();
    ach.total_kills = 80;
    ach.total_bosses_defeated = 8;
    ach.total_dungeons_completed = 7;
    ach.total_fish_caught = 60;
    ach.total_minigame_wins = 50;
    ach.total_deep_missions_completed = 5;
    ach.highest_deep_layer = 4;
    ach.highest_guild_rank = 1;

    ach.refresh_progress();

    // Slayer series
    let slayer_p = ach.get_progress(AchievementId::SlayerI).unwrap();
    assert_eq!(slayer_p.current, 80);
    assert_eq!(slayer_p.target, 100);

    // Boss hunter series
    let boss_p = ach.get_progress(AchievementId::BossHunterII).unwrap();
    assert_eq!(boss_p.current, 8);
    assert_eq!(boss_p.target, 10);

    // Deep missions series
    let missions_p = ach.get_progress(AchievementId::DeepMissionsX).unwrap();
    assert_eq!(missions_p.current, 5);
    assert_eq!(missions_p.target, 10);

    // Deep layer series
    let layer_p = ach.get_progress(AchievementId::Layer5Cleared).unwrap();
    assert_eq!(layer_p.current, 4);
    assert_eq!(layer_p.target, 5);

    // Guild rank series
    let rank_p = ach.get_progress(AchievementId::GuildRank2).unwrap();
    assert_eq!(rank_p.current, 1);
    assert_eq!(rank_p.target, 2);
}

// =========================================================================
// on_minigame_won — ensure all 10 game types fire correctly
// (supplementing existing tests with cross-verification)
// =========================================================================

#[test]
fn test_minigame_won_all_types_novice_unlocks_correct_achievement() {
    let cases = [
        (MinigameType::Chess, AchievementId::ChessNovice),
        (MinigameType::Morris, AchievementId::MorrisNovice),
        (MinigameType::Gomoku, AchievementId::GomokuNovice),
        (MinigameType::Minesweeper, AchievementId::MinesweeperNovice),
        (MinigameType::Rune, AchievementId::RuneNovice),
        (MinigameType::Go, AchievementId::GoNovice),
        (MinigameType::FlappyBird, AchievementId::FlappyNovice),
        (MinigameType::Snake, AchievementId::SnakeNovice),
        (
            MinigameType::Jezzball,
            AchievementId::ContainmentBreachNovice,
        ),
        (MinigameType::RunicShift, AchievementId::SigilSurgeNovice),
    ];
    for (game_type, expected_id) in cases {
        let mut ach = Achievements::default();
        ach.on_minigame_won(game_type, MinigameDifficulty::Novice, Some("Hero"));
        assert!(
            ach.is_unlocked(expected_id),
            "{:?} Novice should unlock {:?}",
            game_type,
            expected_id
        );
        // Grand champion progress should be tracked
        assert_eq!(ach.total_minigame_wins, 1);
    }
}

#[test]
fn test_minigame_won_all_types_master_unlocks_correct_achievement() {
    let cases = [
        (MinigameType::Chess, AchievementId::ChessMaster),
        (MinigameType::Morris, AchievementId::MorrisMaster),
        (MinigameType::Gomoku, AchievementId::GomokuMaster),
        (MinigameType::Minesweeper, AchievementId::MinesweeperMaster),
        (MinigameType::Rune, AchievementId::RuneMaster),
        (MinigameType::Go, AchievementId::GoMaster),
        (MinigameType::FlappyBird, AchievementId::FlappyMaster),
        (MinigameType::Snake, AchievementId::SnakeMaster),
        (
            MinigameType::Jezzball,
            AchievementId::ContainmentBreachMaster,
        ),
        (MinigameType::RunicShift, AchievementId::SigilSurgeMaster),
        (MinigameType::ShardFusion, AchievementId::ShardFusionMaster),
    ];
    for (game_type, expected_id) in cases {
        let mut ach = Achievements::default();
        ach.on_minigame_won(game_type, MinigameDifficulty::Master, Some("Hero"));
        assert!(
            ach.is_unlocked(expected_id),
            "{:?} Master should unlock {:?}",
            game_type,
            expected_id
        );
    }
}

#[test]
fn test_minigame_won_apprentice_and_journeyman_per_game() {
    let apprentice_cases = [
        (MinigameType::Chess, AchievementId::ChessApprentice),
        (MinigameType::Morris, AchievementId::MorrisApprentice),
        (MinigameType::Gomoku, AchievementId::GomokuApprentice),
        (
            MinigameType::Minesweeper,
            AchievementId::MinesweeperApprentice,
        ),
        (MinigameType::Rune, AchievementId::RuneApprentice),
        (MinigameType::Go, AchievementId::GoApprentice),
        (MinigameType::FlappyBird, AchievementId::FlappyApprentice),
        (MinigameType::Snake, AchievementId::SnakeApprentice),
        (
            MinigameType::Jezzball,
            AchievementId::ContainmentBreachApprentice,
        ),
        (
            MinigameType::RunicShift,
            AchievementId::SigilSurgeApprentice,
        ),
    ];
    for (game_type, expected_id) in apprentice_cases {
        let mut ach = Achievements::default();
        ach.on_minigame_won(game_type, MinigameDifficulty::Apprentice, Some("Hero"));
        assert!(
            ach.is_unlocked(expected_id),
            "{:?} Apprentice should unlock {:?}",
            game_type,
            expected_id
        );
    }

    let journeyman_cases = [
        (MinigameType::Chess, AchievementId::ChessJourneyman),
        (MinigameType::Morris, AchievementId::MorrisJourneyman),
        (MinigameType::Gomoku, AchievementId::GomokuJourneyman),
        (
            MinigameType::Minesweeper,
            AchievementId::MinesweeperJourneyman,
        ),
        (MinigameType::Rune, AchievementId::RuneJourneyman),
        (MinigameType::Go, AchievementId::GoJourneyman),
        (MinigameType::FlappyBird, AchievementId::FlappyJourneyman),
        (MinigameType::Snake, AchievementId::SnakeJourneyman),
        (
            MinigameType::Jezzball,
            AchievementId::ContainmentBreachJourneyman,
        ),
        (
            MinigameType::RunicShift,
            AchievementId::SigilSurgeJourneyman,
        ),
    ];
    for (game_type, expected_id) in journeyman_cases {
        let mut ach = Achievements::default();
        ach.on_minigame_won(game_type, MinigameDifficulty::Journeyman, Some("Hero"));
        assert!(
            ach.is_unlocked(expected_id),
            "{:?} Journeyman should unlock {:?}",
            game_type,
            expected_id
        );
    }
}

#[test]
fn test_minigame_won_grand_champion_milestone() {
    let mut ach = Achievements::default();
    // Win 99 games first
    ach.total_minigame_wins = 99;
    // Win one more to hit 100
    ach.on_minigame_won(MinigameType::Go, MinigameDifficulty::Master, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::GrandChampion));
    assert_eq!(ach.total_minigame_wins, 100);
}

// =========================================================================
// sync_from_game_state — representative state coverage
// =========================================================================

#[test]
fn test_sync_from_game_state_high_level_character() {
    let mut ach = Achievements::default();
    ach.sync_from_game_state(
        500,
        0,
        0,
        0,
        &std::collections::BTreeSet::new(),
        Some("Hero"),
    );
    assert!(ach.is_unlocked(AchievementId::Level500));
    assert!(!ach.is_unlocked(AchievementId::Level750));
}

#[test]
fn test_sync_from_game_state_prestige_100_unlocks_eternal() {
    let mut ach = Achievements::default();
    ach.sync_from_game_state(
        1,
        100,
        0,
        0,
        &std::collections::BTreeSet::new(),
        Some("Hero"),
    );
    assert!(ach.is_unlocked(AchievementId::Eternal));
}

#[test]
fn test_sync_from_game_state_fishing_rank_40_unlocks_fisherman_iv() {
    let mut ach = Achievements::default();
    ach.sync_from_game_state(
        1,
        0,
        40,
        0,
        &std::collections::BTreeSet::new(),
        Some("Hero"),
    );
    assert!(ach.is_unlocked(AchievementId::FishermanI));
    assert!(ach.is_unlocked(AchievementId::FishermanII));
    assert!(ach.is_unlocked(AchievementId::FishermanIII));
    assert!(ach.is_unlocked(AchievementId::FishermanIV));
}

#[test]
fn test_sync_from_game_state_fish_count_1000_unlocks_fishcatcher_ii() {
    let mut ach = Achievements::default();
    ach.sync_from_game_state(
        1,
        0,
        0,
        1000,
        &std::collections::BTreeSet::new(),
        Some("Hero"),
    );
    assert!(ach.is_unlocked(AchievementId::GoneFishing));
    assert!(ach.is_unlocked(AchievementId::FishCatcherI));
    assert!(ach.is_unlocked(AchievementId::FishCatcherII));
    assert!(!ach.is_unlocked(AchievementId::FishCatcherIII));
}

#[test]
fn test_sync_from_game_state_uses_max_of_saved_and_existing_fish_count() {
    // Existing achievement fish count is higher than save file — should keep higher
    let mut ach = Achievements {
        total_fish_caught: 10000,
        ..Default::default()
    };
    ach.sync_from_game_state(
        1,
        0,
        0,
        500,
        &std::collections::BTreeSet::new(),
        Some("Hero"),
    );
    // 10000 > 500, so should keep 10000
    assert_eq!(ach.total_fish_caught, 10000);
    assert!(ach.is_unlocked(AchievementId::FishCatcherIII)); // 10000
}

#[test]
fn test_sync_from_game_state_zone_10_complete() {
    let mut ach = Achievements::default();
    // Zone 10 has 4 subzones (Floating Isles / Storm Citadel have 4)
    // Per CLAUDE.md: Zone 10 Storm Citadel has 4 subzones
    let defeated_bosses: std::collections::BTreeSet<(u32, u32)> =
        [(10, 1), (10, 2), (10, 3), (10, 4)].into_iter().collect();
    ach.sync_from_game_state(1, 0, 0, 0, &defeated_bosses, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::Zone10Complete));
}

#[test]
fn test_sync_from_game_state_calls_refresh_progress() {
    let mut ach = Achievements::default();
    ach.total_kills = 50;
    ach.sync_from_game_state(1, 0, 0, 0, &std::collections::BTreeSet::new(), Some("Hero"));

    // refresh_progress is called as the last step of sync_from_game_state
    let p = ach
        .get_progress(AchievementId::SlayerI)
        .expect("SlayerI progress should be set after sync");
    assert_eq!(p.current, 50);
}

// =========================================================================
// Cross-handler interactions and notification integration
// =========================================================================

#[test]
fn test_deep_handlers_populate_newly_unlocked() {
    let mut ach = Achievements::default();
    ach.on_deep_discovered(Some("Hero"));
    let newly = ach.take_newly_unlocked();
    assert!(newly.contains(&AchievementId::TheDeepDiscovered));
}

#[test]
fn test_deep_mission_complete_populates_pending_notifications() {
    let mut ach = Achievements::default();
    ach.on_deep_mission_complete(Some("Hero"));
    assert_eq!(ach.pending_count(), 1);
    assert!(ach
        .pending_notifications
        .contains(&AchievementId::FirstMissionComplete));
}

#[test]
fn test_breakthrough_populates_modal_queue() {
    let mut ach = Achievements::default();
    ach.on_deep_breakthrough(1, Some("Hero"));
    assert!(!ach.modal_queue.is_empty());
    assert!(ach.modal_queue.contains(&AchievementId::FirstBreakthrough));
}

#[test]
fn test_guild_rank_up_populates_newly_unlocked() {
    let mut ach = Achievements::default();
    ach.on_deep_guild_rank_up(2, Some("Hero"));
    let newly = ach.take_newly_unlocked();
    assert!(newly.contains(&AchievementId::GuildRank2));
}

#[test]
fn test_merc_lost_populates_pending_notifications() {
    let mut ach = Achievements::default();
    ach.on_deep_merc_lost(Some("Hero"));
    assert_eq!(ach.pending_count(), 1);
}

#[test]
fn test_gateway_opened_populates_newly_unlocked() {
    let mut ach = Achievements::default();
    ach.on_deep_gateway_opened(Some("Hero"));
    let newly = ach.take_newly_unlocked();
    assert!(newly.contains(&AchievementId::GatewayOpened));
}
