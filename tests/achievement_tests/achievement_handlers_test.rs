//! Integration tests for achievements/handlers.rs and achievements/stats.rs.
//!
//! Tests all event handlers (on_*), state synchronization (sync_*),
//! and statistics methods (total_count, unlocked_count, etc.).

#![allow(clippy::field_reassign_with_default)]

use quest::achievements::types::*;
use quest::achievements::{AchievementCategory, AchievementId, MinigameDifficulty, MinigameType};
use quest::haven::types::HavenRoomId;
use std::collections::HashMap;

use super::helpers::{build_haven_max_tiers, build_haven_tiers};

// =========================================================================
// on_enemy_killed — kill counter and Slayer milestones
// =========================================================================

#[test]
fn test_on_enemy_killed_increments_kill_count() {
    let mut ach = Achievements::default();
    ach.on_enemy_killed(false, Some("Hero"));
    assert_eq!(ach.total_kills, 1);
}

#[test]
fn test_on_enemy_killed_non_boss_does_not_increment_boss_count() {
    let mut ach = Achievements::default();
    ach.on_enemy_killed(false, Some("Hero"));
    assert_eq!(ach.total_bosses_defeated, 0);
}

#[test]
fn test_on_enemy_killed_boss_increments_both_counters() {
    let mut ach = Achievements::default();
    ach.on_enemy_killed(true, Some("Hero"));
    assert_eq!(ach.total_kills, 1);
    assert_eq!(ach.total_bosses_defeated, 1);
}

#[test]
fn test_slayer_i_at_exactly_100_kills() {
    let mut ach = Achievements::default();
    ach.total_kills = 99;
    assert!(!ach.is_unlocked(AchievementId::SlayerI));
    ach.on_enemy_killed(false, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SlayerI));
}

#[test]
fn test_slayer_ii_at_500_kills() {
    let mut ach = Achievements::default();
    ach.total_kills = 499;
    ach.on_enemy_killed(false, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SlayerII));
}

#[test]
fn test_slayer_iii_at_1000_kills() {
    let mut ach = Achievements::default();
    ach.total_kills = 999;
    ach.on_enemy_killed(false, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SlayerIII));
}

#[test]
fn test_slayer_iv_at_5000_kills() {
    let mut ach = Achievements::default();
    ach.total_kills = 4999;
    ach.on_enemy_killed(false, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SlayerIV));
}

#[test]
fn test_slayer_v_at_10000_kills() {
    let mut ach = Achievements::default();
    ach.total_kills = 9999;
    ach.on_enemy_killed(false, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SlayerV));
}

#[test]
fn test_slayer_below_threshold_stores_progress() {
    let mut ach = Achievements::default();
    for _ in 0..50 {
        ach.on_enemy_killed(false, None);
    }
    assert!(!ach.is_unlocked(AchievementId::SlayerI));
    let progress = ach.get_progress(AchievementId::SlayerI).unwrap();
    assert_eq!(progress.current, 50);
    assert_eq!(progress.target, 100);
}

// =========================================================================
// on_enemy_killed — Boss Hunter milestones
// =========================================================================

#[test]
fn test_boss_hunter_i_at_first_boss() {
    let mut ach = Achievements::default();
    ach.on_enemy_killed(true, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::BossHunterI));
}

#[test]
fn test_boss_hunter_ii_at_10_bosses() {
    let mut ach = Achievements::default();
    ach.total_bosses_defeated = 9;
    ach.total_kills = 9; // bosses also increment total_kills
    ach.on_enemy_killed(true, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::BossHunterII));
}

#[test]
fn test_boss_hunter_iii_at_50_bosses() {
    let mut ach = Achievements::default();
    ach.total_bosses_defeated = 49;
    ach.total_kills = 49;
    ach.on_enemy_killed(true, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::BossHunterIII));
}

#[test]
fn test_boss_hunter_iv_at_100_bosses() {
    let mut ach = Achievements::default();
    ach.total_bosses_defeated = 99;
    ach.total_kills = 99;
    // Also unlock SlayerI since total_kills will be 100
    ach.on_enemy_killed(true, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::BossHunterIV));
    assert!(ach.is_unlocked(AchievementId::SlayerI)); // side effect
}

#[test]
fn test_boss_hunter_v_at_500_bosses() {
    let mut ach = Achievements::default();
    ach.total_bosses_defeated = 499;
    ach.total_kills = 499;
    ach.on_enemy_killed(true, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::BossHunterV));
}

// =========================================================================
// on_level_up — Level milestones
// =========================================================================

#[test]
fn test_on_level_up_tracks_highest_level() {
    let mut ach = Achievements::default();
    ach.on_level_up(50, Some("Hero"));
    assert_eq!(ach.highest_level, 50);

    // Lower level should not decrease highest
    ach.on_level_up(30, Some("Hero"));
    assert_eq!(ach.highest_level, 50);

    // Higher level should update
    ach.on_level_up(75, Some("Hero"));
    assert_eq!(ach.highest_level, 75);
}

#[test]
fn test_level_10_milestone() {
    let mut ach = Achievements::default();
    ach.on_level_up(9, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::Level10));
    ach.on_level_up(10, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::Level10));
}

#[test]
fn test_level_25_milestone() {
    let mut ach = Achievements::default();
    ach.on_level_up(25, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::Level25));
}

#[test]
fn test_level_50_milestone() {
    let mut ach = Achievements::default();
    ach.on_level_up(50, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::Level50));
}

#[test]
fn test_level_100_milestone() {
    let mut ach = Achievements::default();
    ach.on_level_up(100, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::Level100));
}

#[test]
fn test_level_milestones_all() {
    let mut ach = Achievements::default();
    let milestones = [
        (10, AchievementId::Level10),
        (25, AchievementId::Level25),
        (50, AchievementId::Level50),
        (100, AchievementId::Level100),
        (150, AchievementId::Level150),
        (200, AchievementId::Level200),
        (250, AchievementId::Level250),
        (500, AchievementId::Level500),
        (750, AchievementId::Level750),
        (1000, AchievementId::Level1000),
        (1500, AchievementId::Level1500),
    ];
    for (level, id) in milestones {
        ach.on_level_up(level, Some("Hero"));
        assert!(ach.is_unlocked(id), "Expected {:?} at level {}", id, level);
    }
}

#[test]
fn test_level_up_with_none_character_name() {
    let mut ach = Achievements::default();
    ach.on_level_up(10, None);
    assert!(ach.is_unlocked(AchievementId::Level10));
    let record = ach.unlocked.get(&AchievementId::Level10).unwrap();
    assert!(record.character_name.is_none());
}

#[test]
fn test_level_overshoot_unlocks_all_prior() {
    let mut ach = Achievements::default();
    ach.on_level_up(200, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::Level10));
    assert!(ach.is_unlocked(AchievementId::Level25));
    assert!(ach.is_unlocked(AchievementId::Level50));
    assert!(ach.is_unlocked(AchievementId::Level100));
    assert!(ach.is_unlocked(AchievementId::Level150));
    assert!(ach.is_unlocked(AchievementId::Level200));
    assert!(!ach.is_unlocked(AchievementId::Level250));
}

// =========================================================================
// on_prestige — Prestige milestones
// =========================================================================

#[test]
fn test_on_prestige_tracks_highest_rank() {
    let mut ach = Achievements::default();
    ach.on_prestige(10, Some("Hero"));
    assert_eq!(ach.highest_prestige_rank, 10);
    ach.on_prestige(5, Some("Hero"));
    assert_eq!(ach.highest_prestige_rank, 10);
    ach.on_prestige(20, Some("Hero"));
    assert_eq!(ach.highest_prestige_rank, 20);
}

#[test]
fn test_first_prestige_at_rank_1() {
    let mut ach = Achievements::default();
    ach.on_prestige(1, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FirstPrestige));
}

#[test]
fn test_prestige_v_at_rank_5() {
    let mut ach = Achievements::default();
    ach.on_prestige(5, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::PrestigeV));
}

#[test]
fn test_prestige_milestones_all() {
    let mut ach = Achievements::default();
    let milestones = [
        (1, AchievementId::FirstPrestige),
        (5, AchievementId::PrestigeV),
        (10, AchievementId::PrestigeX),
        (15, AchievementId::PrestigeXV),
        (20, AchievementId::PrestigeXX),
        (25, AchievementId::PrestigeXXV),
        (30, AchievementId::PrestigeXXX),
        (40, AchievementId::PrestigeXL),
        (50, AchievementId::PrestigeL),
        (70, AchievementId::PrestigeLXX),
        (90, AchievementId::PrestigeXC),
        (100, AchievementId::Eternal),
        (150, AchievementId::Prestige150),
        (200, AchievementId::Prestige200),
        (300, AchievementId::Prestige300),
        (500, AchievementId::Prestige500),
        (700, AchievementId::Prestige700),
        (1000, AchievementId::Prestige1000),
        (10000, AchievementId::Prestige10000),
    ];
    for (rank, id) in milestones {
        ach.on_prestige(rank, Some("Hero"));
        assert!(ach.is_unlocked(id), "Expected {:?} at rank {}", id, rank);
    }
}

#[test]
fn test_prestige_overshoot_unlocks_prior() {
    let mut ach = Achievements::default();
    ach.on_prestige(100, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FirstPrestige));
    assert!(ach.is_unlocked(AchievementId::PrestigeV));
    assert!(ach.is_unlocked(AchievementId::PrestigeX));
    assert!(ach.is_unlocked(AchievementId::Eternal));
}

// =========================================================================
// on_zone_fully_cleared — Zone completion & Expanse
// =========================================================================

#[test]
fn test_zone_completion_all_zones() {
    let zones = [
        (1, AchievementId::Zone1Complete),
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

    for (zone_id, id) in zones {
        let mut ach = Achievements::default();
        ach.on_zone_fully_cleared(zone_id, Some("Hero"));
        assert!(
            ach.is_unlocked(id),
            "Zone {} should unlock {:?}",
            zone_id,
            id
        );
    }
}

#[test]
fn test_zone_cleared_increments_counter() {
    let mut ach = Achievements::default();
    assert_eq!(ach.zones_fully_cleared, 0);
    ach.on_zone_fully_cleared(1, Some("Hero"));
    assert_eq!(ach.zones_fully_cleared, 1);
    ach.on_zone_fully_cleared(2, Some("Hero"));
    assert_eq!(ach.zones_fully_cleared, 2);
}

#[test]
fn test_zone_11_expanse_unlocks_beyond_infinity() {
    let mut ach = Achievements::default();
    ach.on_zone_fully_cleared(11, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::BeyondInfinity));
    assert_eq!(ach.expanse_cycles_completed, 1);
}

#[test]
fn test_zone_11_multiple_cycles() {
    let mut ach = Achievements::default();
    ach.on_zone_fully_cleared(11, Some("Hero"));
    ach.on_zone_fully_cleared(11, Some("Hero"));
    ach.on_zone_fully_cleared(11, Some("Hero"));
    assert_eq!(ach.expanse_cycles_completed, 3);
    assert_eq!(ach.zones_fully_cleared, 3);
}

#[test]
fn test_zone_11_does_not_unlock_zone_10() {
    let mut ach = Achievements::default();
    ach.on_zone_fully_cleared(11, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::Zone10Complete));
    assert!(!ach.is_unlocked(AchievementId::Zone1Complete));
}

#[test]
fn test_invalid_zone_id_does_nothing() {
    let mut ach = Achievements::default();
    ach.on_zone_fully_cleared(0, Some("Hero"));
    ach.on_zone_fully_cleared(12, Some("Hero"));
    ach.on_zone_fully_cleared(255, Some("Hero"));
    // zones_fully_cleared incremented but no achievement unlocked
    assert_eq!(ach.zones_fully_cleared, 3);
    assert!(!ach.is_unlocked(AchievementId::Zone1Complete));
    assert!(!ach.is_unlocked(AchievementId::BeyondInfinity));
}

// =========================================================================
// on_fishing_rank_up — Fisherman milestones
// =========================================================================

#[test]
fn test_fishing_rank_tracks_highest() {
    let mut ach = Achievements::default();
    ach.on_fishing_rank_up(15, Some("Hero"));
    assert_eq!(ach.highest_fishing_rank, 15);
    ach.on_fishing_rank_up(10, Some("Hero"));
    assert_eq!(ach.highest_fishing_rank, 15);
    ach.on_fishing_rank_up(25, Some("Hero"));
    assert_eq!(ach.highest_fishing_rank, 25);
}

#[test]
fn test_fisherman_i_at_rank_10() {
    let mut ach = Achievements::default();
    ach.on_fishing_rank_up(9, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::FishermanI));
    ach.on_fishing_rank_up(10, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FishermanI));
}

#[test]
fn test_fisherman_ii_at_rank_20() {
    let mut ach = Achievements::default();
    ach.on_fishing_rank_up(20, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FishermanII));
}

#[test]
fn test_fisherman_iii_at_rank_30() {
    let mut ach = Achievements::default();
    ach.on_fishing_rank_up(30, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FishermanIII));
}

#[test]
fn test_fisherman_iv_at_rank_40() {
    let mut ach = Achievements::default();
    ach.on_fishing_rank_up(40, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FishermanIV));
}

// =========================================================================
// Fishing rank from challenge rewards — regression test for bug where
// challenge rewards granting fishing ranks didn't trigger achievements
// =========================================================================

#[test]
fn test_fishing_rank_40_from_challenge_reward_unlocks_fisherman_iv() {
    // Simulates the flow in track_input_achievements: if fishing rank changed
    // after input (e.g., challenge reward), on_fishing_rank_up should be called.
    let mut ach = Achievements::default();

    // Before challenge: rank 36, no FishermanIV
    ach.on_fishing_rank_up(36, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::FishermanIV));
    assert!(ach.is_unlocked(AchievementId::FishermanIII)); // rank 30+ unlocked

    // Challenge reward bumps rank to 40 — on_fishing_rank_up should be called
    ach.on_fishing_rank_up(40, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FishermanIV));
}

#[test]
fn test_sync_from_game_state_catches_missed_fishing_achievement() {
    // Verifies the retroactive sync path: if a player already has rank 40
    // but FishermanIV was never unlocked (the bug), sync_from_game_state
    // should catch it on next character load.
    let mut ach = Achievements::default();

    // Before sync: not unlocked
    assert!(!ach.is_unlocked(AchievementId::FishermanIV));

    // Sync with fishing rank 40 (simulating character load)
    ach.sync_from_game_state(
        100,
        20,
        40,
        0,
        &std::collections::BTreeSet::new(),
        Some("Hero"),
    );

    // After sync: should be unlocked
    assert!(ach.is_unlocked(AchievementId::FishermanIV));
}

// =========================================================================
// on_fish_caught — Fish catch counter milestones
// =========================================================================

#[test]
fn test_fish_caught_increments_counter() {
    let mut ach = Achievements::default();
    ach.on_fish_caught(Some("Hero"));
    assert_eq!(ach.total_fish_caught, 1);
    ach.on_fish_caught(Some("Hero"));
    assert_eq!(ach.total_fish_caught, 2);
}

#[test]
fn test_gone_fishing_at_first_catch() {
    let mut ach = Achievements::default();
    ach.on_fish_caught(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::GoneFishing));
}

#[test]
fn test_fish_catcher_i_at_100() {
    let mut ach = Achievements::default();
    ach.total_fish_caught = 99;
    ach.on_fish_caught(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FishCatcherI));
}

#[test]
fn test_fish_catcher_ii_at_1000() {
    let mut ach = Achievements::default();
    ach.total_fish_caught = 999;
    ach.on_fish_caught(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FishCatcherII));
}

#[test]
fn test_fish_catcher_iii_at_10000() {
    let mut ach = Achievements::default();
    ach.total_fish_caught = 9999;
    ach.on_fish_caught(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FishCatcherIII));
}

// =========================================================================
// on_storm_leviathan_caught
// =========================================================================

#[test]
fn test_storm_leviathan_caught() {
    let mut ach = Achievements::default();
    assert!(!ach.is_unlocked(AchievementId::StormLeviathan));
    ach.on_storm_leviathan_caught(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::StormLeviathan));
}

#[test]
fn test_storm_leviathan_idempotent() {
    let mut ach = Achievements::default();
    ach.on_storm_leviathan_caught(Some("Hero"));
    ach.on_storm_leviathan_caught(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::StormLeviathan));
    assert_eq!(ach.unlocked.len(), 1); // only one unlock entry
}

// =========================================================================
// on_storms_end
// =========================================================================

#[test]
fn test_storms_end() {
    let mut ach = Achievements::default();
    assert!(!ach.is_unlocked(AchievementId::StormsEnd));
    ach.on_storms_end(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::StormsEnd));
}

// =========================================================================
// on_dungeon_completed — Dungeon milestones
// =========================================================================

#[test]
fn test_dungeon_completed_increments_counter() {
    let mut ach = Achievements::default();
    ach.on_dungeon_completed(Some("Hero"));
    assert_eq!(ach.total_dungeons_completed, 1);
}

#[test]
fn test_dungeon_diver_at_first_completion() {
    let mut ach = Achievements::default();
    ach.on_dungeon_completed(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DungeonDiver));
}

#[test]
fn test_dungeon_master_i_at_10() {
    let mut ach = Achievements::default();
    ach.total_dungeons_completed = 9;
    ach.on_dungeon_completed(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DungeonMasterI));
}

#[test]
fn test_dungeon_master_ii_at_50() {
    let mut ach = Achievements::default();
    ach.total_dungeons_completed = 49;
    ach.on_dungeon_completed(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DungeonMasterII));
}

#[test]
fn test_dungeon_master_iii_at_100() {
    let mut ach = Achievements::default();
    ach.total_dungeons_completed = 99;
    ach.on_dungeon_completed(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::DungeonMasterIII));
}

#[test]
fn test_dungeon_milestones_all() {
    let milestones = [
        (1, AchievementId::DungeonDiver),
        (10, AchievementId::DungeonMasterI),
        (50, AchievementId::DungeonMasterII),
        (100, AchievementId::DungeonMasterIII),
        (1000, AchievementId::DungeonMasterIV),
        (5000, AchievementId::DungeonMasterV),
        (10000, AchievementId::DungeonMasterVI),
        (25000, AchievementId::DungeonMasterVII),
        (100000, AchievementId::DungeonMasterVIII),
        (500000, AchievementId::DungeonMasterIX),
        (1000000, AchievementId::DungeonMasterX),
    ];
    for (count, id) in milestones {
        let mut ach = Achievements::default();
        ach.total_dungeons_completed = count - 1;
        ach.on_dungeon_completed(Some("Hero"));
        assert!(
            ach.is_unlocked(id),
            "Expected {:?} at {} dungeons",
            id,
            count
        );
    }
}

// =========================================================================
// on_minigame_won — Per-game per-difficulty achievements
// =========================================================================

#[test]
fn test_minigame_won_increments_counter() {
    let mut ach = Achievements::default();
    ach.on_minigame_won(
        MinigameType::Chess,
        MinigameDifficulty::Novice,
        Some("Hero"),
    );
    assert_eq!(ach.total_minigame_wins, 1);
}

#[test]
fn test_chess_all_difficulties() {
    let cases = [
        (MinigameDifficulty::Novice, AchievementId::ChessNovice),
        (
            MinigameDifficulty::Apprentice,
            AchievementId::ChessApprentice,
        ),
        (
            MinigameDifficulty::Journeyman,
            AchievementId::ChessJourneyman,
        ),
        (MinigameDifficulty::Master, AchievementId::ChessMaster),
    ];
    for (diff, id) in cases {
        let mut ach = Achievements::default();
        ach.on_minigame_won(MinigameType::Chess, diff, Some("Hero"));
        assert!(
            ach.is_unlocked(id),
            "Chess {:?} should unlock {:?}",
            diff,
            id
        );
    }
}

#[test]
fn test_morris_all_difficulties() {
    let cases = [
        (MinigameDifficulty::Novice, AchievementId::MorrisNovice),
        (
            MinigameDifficulty::Apprentice,
            AchievementId::MorrisApprentice,
        ),
        (
            MinigameDifficulty::Journeyman,
            AchievementId::MorrisJourneyman,
        ),
        (MinigameDifficulty::Master, AchievementId::MorrisMaster),
    ];
    for (diff, id) in cases {
        let mut ach = Achievements::default();
        ach.on_minigame_won(MinigameType::Morris, diff, Some("Hero"));
        assert!(ach.is_unlocked(id));
    }
}

#[test]
fn test_gomoku_all_difficulties() {
    let cases = [
        (MinigameDifficulty::Novice, AchievementId::GomokuNovice),
        (
            MinigameDifficulty::Apprentice,
            AchievementId::GomokuApprentice,
        ),
        (
            MinigameDifficulty::Journeyman,
            AchievementId::GomokuJourneyman,
        ),
        (MinigameDifficulty::Master, AchievementId::GomokuMaster),
    ];
    for (diff, id) in cases {
        let mut ach = Achievements::default();
        ach.on_minigame_won(MinigameType::Gomoku, diff, Some("Hero"));
        assert!(ach.is_unlocked(id));
    }
}

#[test]
fn test_minesweeper_all_difficulties() {
    let cases = [
        (MinigameDifficulty::Novice, AchievementId::MinesweeperNovice),
        (
            MinigameDifficulty::Apprentice,
            AchievementId::MinesweeperApprentice,
        ),
        (
            MinigameDifficulty::Journeyman,
            AchievementId::MinesweeperJourneyman,
        ),
        (MinigameDifficulty::Master, AchievementId::MinesweeperMaster),
    ];
    for (diff, id) in cases {
        let mut ach = Achievements::default();
        ach.on_minigame_won(MinigameType::Minesweeper, diff, Some("Hero"));
        assert!(ach.is_unlocked(id));
    }
}

#[test]
fn test_rune_all_difficulties() {
    let cases = [
        (MinigameDifficulty::Novice, AchievementId::RuneNovice),
        (
            MinigameDifficulty::Apprentice,
            AchievementId::RuneApprentice,
        ),
        (
            MinigameDifficulty::Journeyman,
            AchievementId::RuneJourneyman,
        ),
        (MinigameDifficulty::Master, AchievementId::RuneMaster),
    ];
    for (diff, id) in cases {
        let mut ach = Achievements::default();
        ach.on_minigame_won(MinigameType::Rune, diff, Some("Hero"));
        assert!(ach.is_unlocked(id));
    }
}

#[test]
fn test_go_all_difficulties() {
    let cases = [
        (MinigameDifficulty::Novice, AchievementId::GoNovice),
        (MinigameDifficulty::Apprentice, AchievementId::GoApprentice),
        (MinigameDifficulty::Journeyman, AchievementId::GoJourneyman),
        (MinigameDifficulty::Master, AchievementId::GoMaster),
    ];
    for (diff, id) in cases {
        let mut ach = Achievements::default();
        ach.on_minigame_won(MinigameType::Go, diff, Some("Hero"));
        assert!(ach.is_unlocked(id));
    }
}

#[test]
fn test_flappy_all_difficulties() {
    let cases = [
        (MinigameDifficulty::Novice, AchievementId::FlappyNovice),
        (
            MinigameDifficulty::Apprentice,
            AchievementId::FlappyApprentice,
        ),
        (
            MinigameDifficulty::Journeyman,
            AchievementId::FlappyJourneyman,
        ),
        (MinigameDifficulty::Master, AchievementId::FlappyMaster),
    ];
    for (diff, id) in cases {
        let mut ach = Achievements::default();
        ach.on_minigame_won(MinigameType::FlappyBird, diff, Some("Hero"));
        assert!(ach.is_unlocked(id));
    }
}

#[test]
fn test_snake_all_difficulties() {
    let cases = [
        (MinigameDifficulty::Novice, AchievementId::SnakeNovice),
        (
            MinigameDifficulty::Apprentice,
            AchievementId::SnakeApprentice,
        ),
        (
            MinigameDifficulty::Journeyman,
            AchievementId::SnakeJourneyman,
        ),
        (MinigameDifficulty::Master, AchievementId::SnakeMaster),
    ];
    for (diff, id) in cases {
        let mut ach = Achievements::default();
        ach.on_minigame_won(MinigameType::Snake, diff, Some("Hero"));
        assert!(ach.is_unlocked(id));
    }
}

#[test]
fn test_jezzball_all_difficulties() {
    let cases = [
        (
            MinigameDifficulty::Novice,
            AchievementId::ContainmentBreachNovice,
        ),
        (
            MinigameDifficulty::Apprentice,
            AchievementId::ContainmentBreachApprentice,
        ),
        (
            MinigameDifficulty::Journeyman,
            AchievementId::ContainmentBreachJourneyman,
        ),
        (
            MinigameDifficulty::Master,
            AchievementId::ContainmentBreachMaster,
        ),
    ];
    for (diff, id) in cases {
        let mut ach = Achievements::default();
        ach.on_minigame_won(MinigameType::Jezzball, diff, Some("Hero"));
        assert!(ach.is_unlocked(id));
    }
}

#[test]
fn test_runic_shift_all_difficulties() {
    let cases = [
        (MinigameDifficulty::Novice, AchievementId::SigilSurgeNovice),
        (
            MinigameDifficulty::Apprentice,
            AchievementId::SigilSurgeApprentice,
        ),
        (
            MinigameDifficulty::Journeyman,
            AchievementId::SigilSurgeJourneyman,
        ),
        (MinigameDifficulty::Master, AchievementId::SigilSurgeMaster),
    ];
    for (diff, id) in cases {
        let mut ach = Achievements::default();
        ach.on_minigame_won(MinigameType::RunicShift, diff, Some("Hero"));
        assert!(ach.is_unlocked(id));
    }
}

#[test]
fn test_grand_champion_at_100_wins() {
    let mut ach = Achievements::default();
    ach.total_minigame_wins = 99;
    ach.on_minigame_won(
        MinigameType::Chess,
        MinigameDifficulty::Novice,
        Some("Hero"),
    );
    assert!(ach.is_unlocked(AchievementId::GrandChampion));
}

#[test]
fn test_grand_champion_not_before_100() {
    let mut ach = Achievements::default();
    ach.total_minigame_wins = 98;
    ach.on_minigame_won(
        MinigameType::Chess,
        MinigameDifficulty::Novice,
        Some("Hero"),
    );
    assert!(!ach.is_unlocked(AchievementId::GrandChampion));
}

// =========================================================================
// on_haven_discovered
// =========================================================================

#[test]
fn test_haven_discovered() {
    let mut ach = Achievements::default();
    assert!(!ach.is_unlocked(AchievementId::HavenDiscovered));
    ach.on_haven_discovered(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenDiscovered));
}

// =========================================================================
// Haven builder tiers (on_haven_all_t1, t2, architect)
// =========================================================================

#[test]
fn test_haven_builder_i() {
    let mut ach = Achievements::default();
    ach.on_haven_all_t1(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderI));
}

#[test]
fn test_haven_builder_ii() {
    let mut ach = Achievements::default();
    ach.on_haven_all_t2(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderII));
}

#[test]
fn test_haven_architect() {
    let mut ach = Achievements::default();
    ach.on_haven_architect(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenArchitect));
}

// =========================================================================
// on_soulforge_discovered
// =========================================================================

#[test]
fn test_soulforge_discovered() {
    let mut ach = Achievements::default();
    assert!(!ach.is_unlocked(AchievementId::SoulforgeDiscovered));
    ach.on_soulforge_discovered(Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SoulforgeDiscovered));
}

// =========================================================================
// on_enhancement_upgraded — Smith progression
// =========================================================================

#[test]
fn test_apprentice_smith_at_level_1() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(1, &[1, 0, 0, 0, 0, 0, 0], 1, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::ApprenticeSmith));
    assert!(!ach.is_unlocked(AchievementId::JourneymanSmith));
}

#[test]
fn test_journeyman_smith_at_level_5() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(5, &[5, 0, 0, 0, 0, 0, 0], 5, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::JourneymanSmith));
    assert!(ach.is_unlocked(AchievementId::ApprenticeSmith));
}

#[test]
fn test_soulforge_adept_at_level_6() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(6, &[6, 0, 0, 0, 0, 0, 0], 6, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SoulforgeAdept));
}

#[test]
fn test_soulforge_savant_at_level_7() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(7, &[7, 0, 0, 0, 0, 0, 0], 7, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SoulforgeSavant));
}

#[test]
fn test_soulforge_master_at_level_8() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(8, &[8, 0, 0, 0, 0, 0, 0], 8, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SoulforgeMaster));
}

#[test]
fn test_soulforge_grandmaster_at_level_9() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(9, &[9, 0, 0, 0, 0, 0, 0], 9, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SoulforgeGrandmaster));
}

#[test]
fn test_master_smith_at_level_10() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(10, &[10, 0, 0, 0, 0, 0, 0], 10, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SoulforgeAscendant));
}

#[test]
fn test_fully_tempered_all_slots_at_4() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(4, &[4, 4, 4, 4, 4, 4, 4], 28, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::FullyTempered));
}

#[test]
fn test_fully_tempered_not_if_one_slot_below_4() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(4, &[4, 4, 4, 3, 4, 4, 4], 27, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::FullyTempered));
}

#[test]
fn test_soul_convergence_all_slots_at_7() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(7, &[7, 7, 7, 7, 7, 7, 7], 49, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::SoulConvergence));
}

#[test]
fn test_soul_convergence_not_if_one_slot_below_7() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(7, &[7, 7, 7, 6, 7, 7, 7], 48, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::SoulConvergence));
}

#[test]
fn test_persistent_hammering_at_100_attempts() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(1, &[1, 0, 0, 0, 0, 0, 0], 100, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::PersistentHammering));
}

#[test]
fn test_persistent_hammering_not_at_99_attempts() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(1, &[1, 0, 0, 0, 0, 0, 0], 99, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::PersistentHammering));
}

#[test]
fn test_enhancement_level_0_unlocks_nothing() {
    let mut ach = Achievements::default();
    ach.on_enhancement_upgraded(0, &[0, 0, 0, 0, 0, 0, 0], 1, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::ApprenticeSmith));
}

#[test]
fn test_enhancement_level_10_unlocks_entire_chain() {
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
// sync_from_game_state — Retroactive unlocking
// =========================================================================

#[test]
fn test_sync_level_achievements() {
    let mut ach = Achievements::default();
    ach.sync_from_game_state(
        120,
        0,
        0,
        0,
        &std::collections::BTreeSet::new(),
        Some("Hero"),
    );
    assert!(ach.is_unlocked(AchievementId::Level10));
    assert!(ach.is_unlocked(AchievementId::Level25));
    assert!(ach.is_unlocked(AchievementId::Level50));
    assert!(ach.is_unlocked(AchievementId::Level100));
    assert!(!ach.is_unlocked(AchievementId::Level150));
}

#[test]
fn test_sync_prestige_achievements() {
    let mut ach = Achievements::default();
    ach.sync_from_game_state(
        1,
        17,
        0,
        0,
        &std::collections::BTreeSet::new(),
        Some("Hero"),
    );
    assert!(ach.is_unlocked(AchievementId::FirstPrestige));
    assert!(ach.is_unlocked(AchievementId::PrestigeV));
    assert!(ach.is_unlocked(AchievementId::PrestigeX));
    assert!(ach.is_unlocked(AchievementId::PrestigeXV));
    assert!(!ach.is_unlocked(AchievementId::PrestigeXX));
}

#[test]
fn test_sync_prestige_zero_skips() {
    let mut ach = Achievements::default();
    ach.sync_from_game_state(1, 0, 0, 0, &std::collections::BTreeSet::new(), Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::FirstPrestige));
}

#[test]
fn test_sync_fishing_rank_achievements() {
    let mut ach = Achievements::default();
    ach.sync_from_game_state(
        1,
        0,
        15,
        0,
        &std::collections::BTreeSet::new(),
        Some("Hero"),
    );
    assert!(ach.is_unlocked(AchievementId::FishermanI));
    assert!(!ach.is_unlocked(AchievementId::FishermanII));
}

#[test]
fn test_sync_fishing_rank_zero_skips() {
    let mut ach = Achievements::default();
    ach.sync_from_game_state(1, 0, 0, 0, &std::collections::BTreeSet::new(), Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::FishermanI));
}

#[test]
fn test_sync_fish_count() {
    let mut ach = Achievements::default();
    ach.sync_from_game_state(
        1,
        0,
        0,
        500,
        &std::collections::BTreeSet::new(),
        Some("Hero"),
    );
    assert!(ach.is_unlocked(AchievementId::GoneFishing));
    assert!(ach.is_unlocked(AchievementId::FishCatcherI));
    assert!(!ach.is_unlocked(AchievementId::FishCatcherII));
    assert_eq!(ach.total_fish_caught, 500);
}

#[test]
fn test_sync_does_not_decrease_fish_count() {
    let mut ach = Achievements {
        total_fish_caught: 50000,
        ..Default::default()
    };
    ach.sync_from_game_state(
        1,
        0,
        0,
        1000,
        &std::collections::BTreeSet::new(),
        Some("Hero"),
    );
    assert_eq!(ach.total_fish_caught, 50000);
    assert!(ach.is_unlocked(AchievementId::FishCatcherIII)); // 10000
}

#[test]
fn test_sync_zone_completions() {
    let mut ach = Achievements::default();
    // Zone 1 has 3 subzones
    let defeated_bosses = std::collections::BTreeSet::from([(1, 1), (1, 2), (1, 3)]);
    ach.sync_from_game_state(1, 0, 0, 0, &defeated_bosses, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::Zone1Complete));
}

#[test]
fn test_sync_zone_incomplete_does_not_unlock() {
    let mut ach = Achievements::default();
    // Zone 1 has 3 subzones, only 2 cleared
    let defeated_bosses = std::collections::BTreeSet::from([(1, 1), (1, 2)]);
    ach.sync_from_game_state(1, 0, 0, 0, &defeated_bosses, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::Zone1Complete));
}

#[test]
fn test_sync_full_progression() {
    let mut ach = Achievements::default();
    let defeated_bosses = std::collections::BTreeSet::from([
        (1, 1),
        (1, 2),
        (1, 3), // Zone 1 complete
        (2, 1),
        (2, 2),
        (2, 3), // Zone 2 complete
        (3, 1),
        (3, 2),
        (3, 3), // Zone 3 complete
        (4, 1),
        (4, 2),
        (4, 3), // Zone 4 complete
    ]);
    ach.sync_from_game_state(150, 25, 20, 5000, &defeated_bosses, Some("Veteran"));

    // Level
    assert!(ach.is_unlocked(AchievementId::Level100));
    assert!(ach.is_unlocked(AchievementId::Level150));
    assert!(!ach.is_unlocked(AchievementId::Level200));

    // Prestige
    assert!(ach.is_unlocked(AchievementId::PrestigeXXV));
    assert!(!ach.is_unlocked(AchievementId::PrestigeXXX));

    // Fishing
    assert!(ach.is_unlocked(AchievementId::FishermanII));
    assert!(!ach.is_unlocked(AchievementId::FishermanIII));

    // Fish catches
    assert!(ach.is_unlocked(AchievementId::FishCatcherII)); // 1000
    assert!(!ach.is_unlocked(AchievementId::FishCatcherIII)); // 10000

    // Zone completions
    assert!(ach.is_unlocked(AchievementId::Zone1Complete));
    assert!(ach.is_unlocked(AchievementId::Zone4Complete));
    assert!(!ach.is_unlocked(AchievementId::Zone5Complete));
}

// =========================================================================
// sync_from_haven — Haven-related retroactive unlocking
// =========================================================================

#[test]
fn test_sync_haven_not_discovered() {
    let mut ach = Achievements::default();
    let room_tiers = HashMap::new();
    ach.sync_from_haven(false, &room_tiers, Some("Hero"));
    assert!(!ach.is_unlocked(AchievementId::HavenDiscovered));
}

#[test]
fn test_sync_haven_discovered() {
    let mut ach = Achievements::default();
    let room_tiers = HashMap::new();
    ach.sync_from_haven(true, &room_tiers, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenDiscovered));
}

#[test]
fn test_sync_haven_builder_i() {
    let mut ach = Achievements::default();
    let room_tiers = build_haven_tiers(1);
    ach.sync_from_haven(true, &room_tiers, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderI));
    assert!(!ach.is_unlocked(AchievementId::HavenBuilderII));
}

#[test]
fn test_sync_haven_builder_ii() {
    let mut ach = Achievements::default();
    let room_tiers = build_haven_tiers(2);
    ach.sync_from_haven(true, &room_tiers, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderI));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderII));
    assert!(!ach.is_unlocked(AchievementId::HavenArchitect));
}

#[test]
fn test_sync_haven_architect_max_tiers() {
    let mut ach = Achievements::default();
    let room_tiers = build_haven_max_tiers();
    ach.sync_from_haven(true, &room_tiers, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderI));
    assert!(ach.is_unlocked(AchievementId::HavenBuilderII));
    assert!(ach.is_unlocked(AchievementId::HavenArchitect));
}

#[test]
fn test_sync_haven_partial_t1_does_not_unlock_builder() {
    let mut ach = Achievements::default();
    // Only set a few rooms to T1
    let mut room_tiers = HashMap::new();
    room_tiers.insert(HavenRoomId::Hearthstone, 1);
    room_tiers.insert(HavenRoomId::Armory, 1);
    ach.sync_from_haven(true, &room_tiers, Some("Hero"));
    assert!(ach.is_unlocked(AchievementId::HavenDiscovered));
    assert!(!ach.is_unlocked(AchievementId::HavenBuilderI));
}

#[test]
fn test_sync_haven_t3_with_max_tier_rooms() {
    // Rooms with max_tier < 3 at their max should count as T3
    let mut ach = Achievements::default();
    let mut room_tiers: HashMap<HavenRoomId, u8> = HavenRoomId::ALL
        .iter()
        .filter(|r| **r != HavenRoomId::StormForge)
        .map(|r| (*r, 3))
        .collect();
    // FishingDock max is 4, but T3 should count toward the check
    room_tiers.insert(HavenRoomId::FishingDock, 3);
    ach.sync_from_haven(true, &room_tiers, Some("Hero"));
    // FishingDock at T3 < max_tier(4), but we need tier >= 3 for the architect check
    // The check is: tier >= 3 || tier >= max_tier
    // FishingDock has tier=3, max_tier=4, so tier >= 3 is true
    assert!(ach.is_unlocked(AchievementId::HavenArchitect));
}

// =========================================================================
// stats.rs — total_count / unlocked_count / unlock_percentage
// =========================================================================

#[test]
fn test_total_count_matches_all_achievements() {
    let ach = Achievements::default();
    let count = ach.total_count();
    assert!(count > 0);
    assert_eq!(count, AchievementId::VARIANT_COUNT);
}

#[test]
fn test_unlocked_count_starts_at_zero() {
    let ach = Achievements::default();
    assert_eq!(ach.unlocked_count(), 0);
}

#[test]
fn test_unlocked_count_increments_on_unlock() {
    let mut ach = Achievements::default();
    ach.on_enemy_killed(true, Some("Hero")); // BossHunterI + SlayerI progress
    assert_eq!(ach.unlocked_count(), 1); // BossHunterI

    ach.total_kills = 99;
    ach.on_enemy_killed(false, Some("Hero")); // SlayerI at 100
    assert_eq!(ach.unlocked_count(), 2);
}

#[test]
fn test_unlock_percentage_zero_when_empty() {
    let ach = Achievements::default();
    assert!((ach.unlock_percentage() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_unlock_percentage_partial() {
    let mut ach = Achievements::default();
    ach.on_haven_discovered(Some("Hero"));
    let pct = ach.unlock_percentage();
    assert!(pct > 0.0);
    assert!(pct < 100.0);
}

#[test]
fn test_unlock_percentage_calculation() {
    let mut ach = Achievements::default();
    ach.on_haven_discovered(Some("Hero"));
    let expected = (1.0 / ach.total_count() as f32) * 100.0;
    let actual = ach.unlock_percentage();
    assert!((actual - expected).abs() < 0.01);
}

// =========================================================================
// stats.rs — count_by_category
// =========================================================================

#[test]
fn test_count_by_category_empty() {
    let ach = Achievements::default();
    let (unlocked, total) = ach.count_by_category(AchievementCategory::Combat);
    assert_eq!(unlocked, 0);
    assert!(total > 0);
}

#[test]
fn test_count_by_category_combat_after_kills() {
    let mut ach = Achievements::default();
    ach.total_kills = 99;
    ach.on_enemy_killed(true, Some("Hero")); // 100 kills -> SlayerI, 1 boss -> BossHunterI
    let (unlocked, total) = ach.count_by_category(AchievementCategory::Combat);
    assert_eq!(unlocked, 2);
    assert!(total > 2);
}

#[test]
fn test_count_by_category_does_not_cross_categories() {
    let mut ach = Achievements::default();
    ach.on_level_up(10, Some("Hero")); // Level10 is in Level category
    ach.on_prestige(1, Some("Hero")); // FirstPrestige is in Prestige category
    let (combat_unlocked, _) = ach.count_by_category(AchievementCategory::Combat);
    assert_eq!(combat_unlocked, 0);
    let (level_unlocked, _) = ach.count_by_category(AchievementCategory::Level);
    assert!(level_unlocked > 0);
    let (prestige_unlocked, _) = ach.count_by_category(AchievementCategory::Prestige);
    assert!(prestige_unlocked > 0);
}

#[test]
fn test_count_by_category_all_categories_have_totals() {
    let ach = Achievements::default();
    for cat in AchievementCategory::ALL {
        let (_, total) = ach.count_by_category(cat);
        // Stats category may have 0 total since achievements don't use it
        if cat != AchievementCategory::Stats {
            assert!(total > 0, "{:?} should have achievements", cat);
        }
    }
}

// =========================================================================
// stats.rs — take_newly_unlocked
// =========================================================================

#[test]
fn test_take_newly_unlocked_empty_initially() {
    let mut ach = Achievements::default();
    let newly = ach.take_newly_unlocked();
    assert!(newly.is_empty());
}

#[test]
fn test_take_newly_unlocked_returns_and_drains() {
    let mut ach = Achievements::default();
    ach.on_haven_discovered(Some("Hero"));
    ach.on_storms_end(Some("Hero"));

    let newly = ach.take_newly_unlocked();
    assert_eq!(newly.len(), 2);
    assert!(newly.contains(&AchievementId::HavenDiscovered));
    assert!(newly.contains(&AchievementId::StormsEnd));

    // Second call should return empty
    let again = ach.take_newly_unlocked();
    assert!(again.is_empty());
}

#[test]
fn test_take_newly_unlocked_does_not_affect_unlocked_state() {
    let mut ach = Achievements::default();
    ach.on_haven_discovered(Some("Hero"));
    let _newly = ach.take_newly_unlocked();
    // Achievement is still unlocked even after draining newly_unlocked
    assert!(ach.is_unlocked(AchievementId::HavenDiscovered));
}

// =========================================================================
// stats.rs — update_progress / get_progress
// =========================================================================

#[test]
fn test_update_and_get_progress() {
    let mut ach = Achievements::default();
    ach.update_progress(AchievementId::SlayerI, 42, 100);
    let p = ach.get_progress(AchievementId::SlayerI).unwrap();
    assert_eq!(p.current, 42);
    assert_eq!(p.target, 100);
}

#[test]
fn test_get_progress_none_for_untracked() {
    let ach = Achievements::default();
    assert!(ach.get_progress(AchievementId::SlayerI).is_none());
}

#[test]
fn test_progress_updated_by_handlers() {
    let mut ach = Achievements::default();
    for _ in 0..50 {
        ach.on_enemy_killed(false, None);
    }
    // SlayerI needs 100 kills, we have 50
    let p = ach.get_progress(AchievementId::SlayerI).unwrap();
    assert_eq!(p.current, 50);
    assert_eq!(p.target, 100);
}

// =========================================================================
// Duplicate unlock returns false / idempotency
// =========================================================================

#[test]
fn test_duplicate_unlock_via_handler() {
    let mut ach = Achievements::default();
    ach.on_haven_discovered(Some("Hero"));
    let count_before = ach.unlocked_count();
    ach.on_haven_discovered(Some("Hero"));
    assert_eq!(ach.unlocked_count(), count_before);
}

#[test]
fn test_unlock_records_character_name() {
    let mut ach = Achievements::default();
    ach.on_storms_end(Some("MyHero"));
    let record = ach.unlocked.get(&AchievementId::StormsEnd).unwrap();
    assert_eq!(record.character_name, Some("MyHero".to_string()));
}

#[test]
fn test_unlock_records_timestamp() {
    let mut ach = Achievements::default();
    let before = chrono::Utc::now().timestamp();
    ach.on_storms_end(Some("Hero"));
    let after = chrono::Utc::now().timestamp();
    let record = ach.unlocked.get(&AchievementId::StormsEnd).unwrap();
    assert!(record.unlocked_at >= before);
    assert!(record.unlocked_at <= after + 1);
}

// =========================================================================
// Notification integration (unlock -> pending_notifications + newly_unlocked)
// =========================================================================

#[test]
fn test_unlock_populates_pending_notifications() {
    let mut ach = Achievements::default();
    ach.on_haven_discovered(Some("Hero"));
    assert_eq!(ach.pending_count(), 1);
}

#[test]
fn test_unlock_populates_modal_queue() {
    let mut ach = Achievements::default();
    ach.on_haven_discovered(Some("Hero"));
    assert_eq!(ach.modal_queue.len(), 1);
    assert!(ach.accumulation_start.is_some());
}

#[test]
fn test_multiple_unlocks_in_one_handler_batch() {
    let mut ach = Achievements::default();
    // on_level_up(200) should unlock Level10, Level25, Level50, Level100, Level150, Level200
    ach.on_level_up(200, Some("Hero"));
    assert_eq!(ach.unlocked_count(), 6);
    assert_eq!(ach.pending_count(), 6);
    assert_eq!(ach.modal_queue.len(), 6);
}

// =========================================================================
// refresh_progress — Updates progress bars for all series
// =========================================================================

#[test]
fn test_refresh_progress_updates_series() {
    let mut ach = Achievements::default();
    ach.total_kills = 50;
    ach.total_bosses_defeated = 5;
    ach.total_dungeons_completed = 25;
    ach.total_fish_caught = 500;
    ach.total_minigame_wins = 42;

    ach.refresh_progress();

    // Check slayer progress
    let p = ach.get_progress(AchievementId::SlayerI).unwrap();
    assert_eq!(p.current, 50);
    assert_eq!(p.target, 100);

    // Check boss hunter progress
    let p = ach.get_progress(AchievementId::BossHunterII).unwrap();
    assert_eq!(p.current, 5);
    assert_eq!(p.target, 10);

    // Check dungeon progress
    let p = ach.get_progress(AchievementId::DungeonMasterII).unwrap();
    assert_eq!(p.current, 25);
    assert_eq!(p.target, 50);

    // Check fish catcher progress
    let p = ach.get_progress(AchievementId::FishCatcherII).unwrap();
    assert_eq!(p.current, 500);
    assert_eq!(p.target, 1000);

    // Check grand champion progress
    let p = ach.get_progress(AchievementId::GrandChampion).unwrap();
    assert_eq!(p.current, 42);
    assert_eq!(p.target, 100);
}

// =========================================================================
// Serialization round-trip
// =========================================================================

#[test]
fn test_serialization_preserves_achievements() {
    let mut ach = Achievements::default();
    ach.on_level_up(50, Some("Hero"));
    ach.on_enemy_killed(true, Some("Hero"));
    ach.total_kills = 200;

    let json = serde_json::to_string(&ach).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();

    assert!(loaded.is_unlocked(AchievementId::Level50));
    assert!(loaded.is_unlocked(AchievementId::BossHunterI));
    assert_eq!(loaded.total_kills, 200);
}

#[test]
fn test_serialization_skips_transient_fields() {
    let mut ach = Achievements::default();
    ach.on_haven_discovered(Some("Hero"));
    // These should be populated
    assert!(!ach.newly_unlocked.is_empty());
    assert!(!ach.pending_notifications.is_empty());

    let json = serde_json::to_string(&ach).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();

    // Transient fields should be empty after deserialization
    assert!(loaded.newly_unlocked.is_empty());
    assert!(loaded.pending_notifications.is_empty());
    assert!(loaded.modal_queue.is_empty());
    assert!(loaded.recently_unlocked.is_empty());
    // But the achievement is still unlocked
    assert!(loaded.is_unlocked(AchievementId::HavenDiscovered));
}
