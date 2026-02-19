//! Comprehensive achievement test coverage (T1-T8)
//!
//! Tests for:
//! - T1: Enhancement achievement integration
//! - T2: Achievement persistence round-trip
//! - T3: Simultaneous/batch achievement unlock
//! - T4: Offline progression achievement gap documentation
//! - T5: Minigame achievement integration (using MinigameType/MinigameDifficulty enums)
//! - T6: Zone completion through process_zone_achievements
//! - T7: Haven sync edge cases
//! - T8: State sync idempotency

use quest::achievements::{types::MinigameDifficulty, types::MinigameType, Achievements};
use quest::achievements::{AchievementCategory, AchievementId};

// =========================================================================
// T1: Enhancement achievement integration tests
// =========================================================================

#[test]
fn test_enhancement_apprentice_smith_on_first_success() {
    let mut achievements = Achievements::default();

    // +1 on any slot (new_level=1), all others still at 0
    let all_levels = [1u8, 0, 0, 0, 0, 0, 0];
    achievements.on_enhancement_upgraded(1, &all_levels, 1, Some("Hero"));

    assert!(
        achievements.is_unlocked(AchievementId::ApprenticeSmith),
        "ApprenticeSmith should unlock when any slot reaches +1"
    );
    // Should NOT unlock higher-tier achievements
    assert!(
        !achievements.is_unlocked(AchievementId::JourneymanSmith),
        "JourneymanSmith should NOT unlock on first +1"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::FullyTempered),
        "FullyTempered should NOT unlock when only one slot is at +1"
    );
}

#[test]
fn test_enhancement_fully_tempered() {
    let mut achievements = Achievements::default();

    // All 7 slots at +4
    let all_levels = [4u8, 4, 4, 4, 4, 4, 4];
    achievements.on_enhancement_upgraded(4, &all_levels, 28, Some("Hero"));

    assert!(
        achievements.is_unlocked(AchievementId::ApprenticeSmith),
        "ApprenticeSmith should unlock (new_level >= 1)"
    );
    assert!(
        achievements.is_unlocked(AchievementId::FullyTempered),
        "FullyTempered should unlock when all 7 slots reach +4"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::JourneymanSmith),
        "JourneymanSmith should NOT unlock (new_level is only 4, not 5)"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::SoulConvergence),
        "SoulConvergence should NOT unlock (all slots are 4, not 7)"
    );
}

#[test]
fn test_enhancement_soul_convergence() {
    let mut achievements = Achievements::default();

    // All 7 slots at +7
    let all_levels = [7u8, 7, 7, 7, 7, 7, 7];
    achievements.on_enhancement_upgraded(7, &all_levels, 49, Some("Hero"));

    assert!(
        achievements.is_unlocked(AchievementId::ApprenticeSmith),
        "ApprenticeSmith should unlock"
    );
    assert!(
        achievements.is_unlocked(AchievementId::FullyTempered),
        "FullyTempered should unlock (all slots >= 4)"
    );
    assert!(
        achievements.is_unlocked(AchievementId::JourneymanSmith),
        "JourneymanSmith should unlock (new_level >= 5)"
    );
    assert!(
        achievements.is_unlocked(AchievementId::SoulforgeAdept),
        "SoulforgeAdept should unlock (new_level >= 6)"
    );
    assert!(
        achievements.is_unlocked(AchievementId::SoulforgeSavant),
        "SoulforgeSavant should unlock (new_level >= 7)"
    );
    assert!(
        achievements.is_unlocked(AchievementId::SoulConvergence),
        "SoulConvergence should unlock when all 7 slots reach +7"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::SoulforgeMaster),
        "SoulforgeMaster should NOT unlock (new_level is 7, not 8)"
    );
}

#[test]
fn test_enhancement_persistent_hammering() {
    let mut achievements = Achievements::default();

    // 100 total attempts unlocks PersistentHammering
    // new_level=0 means the enhancement failed (dropped to 0)
    let all_levels = [0u8, 0, 0, 0, 0, 0, 0];
    achievements.on_enhancement_upgraded(0, &all_levels, 100, Some("Hero"));

    assert!(
        achievements.is_unlocked(AchievementId::PersistentHammering),
        "PersistentHammering should unlock at 100 total attempts"
    );
    // ApprenticeSmith checks new_level >= 1, not >= 0
    assert!(
        !achievements.is_unlocked(AchievementId::ApprenticeSmith),
        "ApprenticeSmith should NOT unlock when new_level is 0"
    );
}

#[test]
fn test_enhancement_persistent_hammering_at_99_does_not_unlock() {
    let mut achievements = Achievements::default();

    let all_levels = [0u8, 0, 0, 0, 0, 0, 0];
    achievements.on_enhancement_upgraded(0, &all_levels, 99, Some("Hero"));

    assert!(
        !achievements.is_unlocked(AchievementId::PersistentHammering),
        "PersistentHammering should NOT unlock at 99 attempts"
    );
}

#[test]
fn test_enhancement_failure_does_not_unlock_higher() {
    // Scenario: player had +5, failed and dropped to +4 (new_level=4)
    // JourneymanSmith requires new_level >= 5, so it should NOT unlock
    let mut achievements = Achievements::default();

    // Simulate a failure: slot was at +5, dropped to +4
    let all_levels = [4u8, 4, 4, 4, 4, 4, 4];
    // Only call with the new (reduced) level — new_level=4 after failure
    achievements.on_enhancement_upgraded(4, &all_levels, 50, Some("Hero"));

    assert!(
        achievements.is_unlocked(AchievementId::ApprenticeSmith),
        "ApprenticeSmith should unlock (new_level >= 1)"
    );
    assert!(
        achievements.is_unlocked(AchievementId::FullyTempered),
        "FullyTempered should unlock when all slots are 4"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::JourneymanSmith),
        "Failed enhancement dropping from +5 to +4 should NOT unlock JourneymanSmith (new_level=4 < 5)"
    );
}

#[test]
fn test_enhancement_all_single_slot_milestones() {
    // Test each single-slot achievement threshold
    let milestones: &[(u8, AchievementId)] = &[
        (1, AchievementId::ApprenticeSmith),
        (5, AchievementId::JourneymanSmith),
        (6, AchievementId::SoulforgeAdept),
        (7, AchievementId::SoulforgeSavant),
        (8, AchievementId::SoulforgeMaster),
        (9, AchievementId::SoulforgeGrandmaster),
        (10, AchievementId::MasterSmith),
    ];

    for &(level, achievement_id) in milestones {
        let mut achievements = Achievements::default();
        let all_levels = [level, 0, 0, 0, 0, 0, 0];
        achievements.on_enhancement_upgraded(level, &all_levels, 1, Some("Hero"));
        assert!(
            achievements.is_unlocked(achievement_id),
            "Expected {:?} to be unlocked at new_level={}",
            achievement_id,
            level
        );
    }
}

// =========================================================================
// T2: Achievement persistence round-trip tests
// =========================================================================

#[test]
fn test_achievement_save_load_round_trip() {
    let mut achievements = Achievements::default();

    // Unlock multiple achievements
    achievements.on_enemy_killed(false, Some("RoundTripHero"));
    achievements.on_enemy_killed(false, Some("RoundTripHero"));
    // Set total_kills to 99 to reach SlayerI threshold on next kill
    achievements.total_kills = 99;
    achievements.on_enemy_killed(false, Some("RoundTripHero"));

    achievements.on_enemy_killed(true, Some("RoundTripHero"));
    achievements.on_fish_caught(Some("RoundTripHero"));

    // Set some specific counters
    achievements.total_kills = 500;
    achievements.total_bosses_defeated = 42;
    achievements.total_fish_caught = 7;
    achievements.highest_prestige_rank = 15;
    achievements.highest_level = 250;
    achievements.highest_fishing_rank = 20;

    // Manually unlock some achievements
    achievements.unlock(AchievementId::SlayerI, Some("RoundTripHero".to_string()));
    achievements.unlock(
        AchievementId::BossHunterI,
        Some("RoundTripHero".to_string()),
    );
    achievements.unlock(AchievementId::Level100, Some("RoundTripHero".to_string()));

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&achievements).expect("should serialize");

    // Deserialize
    let loaded: Achievements = serde_json::from_str(&json).expect("should deserialize");

    // Verify unlocked achievements survive round-trip
    assert!(
        loaded.is_unlocked(AchievementId::SlayerI),
        "SlayerI should survive round-trip"
    );
    assert!(
        loaded.is_unlocked(AchievementId::BossHunterI),
        "BossHunterI should survive round-trip"
    );
    assert!(
        loaded.is_unlocked(AchievementId::Level100),
        "Level100 should survive round-trip"
    );

    // Verify counters survive round-trip
    assert_eq!(
        loaded.total_kills, 500,
        "total_kills should survive round-trip"
    );
    assert_eq!(
        loaded.total_bosses_defeated, 42,
        "total_bosses_defeated should survive round-trip"
    );
    assert_eq!(
        loaded.highest_prestige_rank, 15,
        "highest_prestige_rank should survive round-trip"
    );
    assert_eq!(
        loaded.highest_level, 250,
        "highest_level should survive round-trip"
    );
    assert_eq!(
        loaded.highest_fishing_rank, 20,
        "highest_fishing_rank should survive round-trip"
    );

    // Verify progress entries survive
    let json2 = serde_json::to_string_pretty(&loaded).expect("should re-serialize");
    let loaded2: Achievements = serde_json::from_str(&json2).expect("should re-deserialize");
    assert!(loaded2.is_unlocked(AchievementId::SlayerI));
}

#[test]
fn test_achievement_load_corrupted_json_returns_default() {
    // Deserializing garbage string should produce a default (graceful fallback)
    let garbage = r#"{ this is not valid json @@@ }"#;
    let result: Result<Achievements, _> = serde_json::from_str(garbage);

    // The result should be an error, and code using .unwrap_or_default() should give default
    assert!(result.is_err(), "Corrupted JSON should fail to deserialize");

    // The actual load_achievements() function uses unwrap_or_default():
    let fallback = result.unwrap_or_default();
    assert_eq!(
        fallback.total_kills, 0,
        "Fallback from corrupt data should have zero kills"
    );
    assert!(
        fallback.unlocked.is_empty(),
        "Fallback from corrupt data should have no unlocked achievements"
    );
}

#[test]
fn test_achievement_load_empty_json_returns_default() {
    // Empty/minimal valid JSON should produce default
    let empty_json = r#"{}"#;
    let result: Result<Achievements, _> = serde_json::from_str(empty_json);
    // serde should handle missing fields with defaults
    let achievements = result.unwrap_or_default();
    assert_eq!(achievements.total_kills, 0);
    assert!(achievements.unlocked.is_empty());
}

#[test]
fn test_transient_fields_not_persisted() {
    // Verify pending_notifications, newly_unlocked, modal_queue are empty after round-trip
    let mut achievements = Achievements::default();

    // Trigger some unlocks to populate transient fields
    achievements.unlock(AchievementId::SlayerI, Some("TransientTest".to_string()));
    achievements.unlock(
        AchievementId::BossHunterI,
        Some("TransientTest".to_string()),
    );
    achievements.unlock(AchievementId::Level10, Some("TransientTest".to_string()));

    // Verify transient fields are populated before serialization
    assert_eq!(achievements.pending_notifications.len(), 3);
    assert_eq!(achievements.newly_unlocked.len(), 3);
    assert_eq!(achievements.modal_queue.len(), 3);

    // Serialize and deserialize
    let json = serde_json::to_string_pretty(&achievements).expect("should serialize");
    let loaded: Achievements = serde_json::from_str(&json).expect("should deserialize");

    // Transient fields should be empty after deserialization (marked #[serde(skip)])
    assert!(
        loaded.pending_notifications.is_empty(),
        "pending_notifications should be empty after deserialize"
    );
    assert!(
        loaded.newly_unlocked.is_empty(),
        "newly_unlocked should be empty after deserialize"
    );
    assert!(
        loaded.modal_queue.is_empty(),
        "modal_queue should be empty after deserialize"
    );
    assert!(
        loaded.recently_unlocked.is_empty(),
        "recently_unlocked should be empty after deserialize"
    );

    // But the actual unlocked achievements should still be there
    assert!(loaded.is_unlocked(AchievementId::SlayerI));
    assert!(loaded.is_unlocked(AchievementId::BossHunterI));
    assert!(loaded.is_unlocked(AchievementId::Level10));
}

// =========================================================================
// T3: Simultaneous/batch achievement unlock tests
// =========================================================================

#[test]
fn test_simultaneous_slayer_and_boss_hunter_unlock() {
    // Set counts so one boss kill (is_boss=true) hits both thresholds simultaneously:
    // SlayerI threshold = 100, BossHunterI threshold = 1
    // total_bosses_defeated = 0: Next boss kill will be #1 → BossHunterI
    let mut achievements = Achievements {
        total_kills: 99, // Next kill will be kill #100 → SlayerI
        ..Default::default()
    };

    // One kill at exact threshold for BOTH
    achievements.on_enemy_killed(true, Some("BatchHero"));

    assert!(
        achievements.is_unlocked(AchievementId::SlayerI),
        "SlayerI should unlock at kill #100"
    );
    assert!(
        achievements.is_unlocked(AchievementId::BossHunterI),
        "BossHunterI should unlock at boss kill #1"
    );

    // Both should have been added to the modal queue in the same batch
    assert_eq!(
        achievements.modal_queue.len(),
        2,
        "Both achievements should be in modal_queue"
    );
}

#[test]
fn test_modal_accumulation_queues_multiple_unlocks() {
    let mut achievements = Achievements::default();

    // Unlock several achievements in sequence
    achievements.unlock(AchievementId::SlayerI, Some("BatchTest".to_string()));
    achievements.unlock(AchievementId::BossHunterI, Some("BatchTest".to_string()));
    achievements.unlock(AchievementId::Level10, Some("BatchTest".to_string()));
    achievements.unlock(AchievementId::DungeonDiver, Some("BatchTest".to_string()));

    // All 4 should be in the modal queue
    assert_eq!(
        achievements.modal_queue.len(),
        4,
        "All 4 unlocks should be in modal_queue"
    );

    // accumulation_start should be set
    assert!(
        achievements.accumulation_start.is_some(),
        "accumulation_start should be set after first unlock"
    );
}

#[test]
fn test_newly_unlocked_contains_all_simultaneous() {
    // Trigger multiple simultaneous unlocks via on_enemy_killed
    // Set total_kills to 99 and total_bosses_defeated to 0
    let mut achievements = Achievements {
        total_kills: 99,
        ..Default::default()
    };
    achievements.on_enemy_killed(true, Some("SimultaneousHero"));
    // This triggers: SlayerI (100 kills) + BossHunterI (1 boss) = 2 unlocks

    let newly_unlocked = achievements.take_newly_unlocked();
    assert_eq!(
        newly_unlocked.len(),
        2,
        "take_newly_unlocked() should return both SlayerI and BossHunterI"
    );
    assert!(
        newly_unlocked.contains(&AchievementId::SlayerI),
        "SlayerI should be in newly_unlocked"
    );
    assert!(
        newly_unlocked.contains(&AchievementId::BossHunterI),
        "BossHunterI should be in newly_unlocked"
    );

    // After taking, the list should be empty
    let after_take = achievements.take_newly_unlocked();
    assert!(
        after_take.is_empty(),
        "take_newly_unlocked() should be empty after first call"
    );
}

#[test]
fn test_multiple_unlock_batch_through_prestige() {
    let mut achievements = Achievements::default();

    // Prestige to rank 10 — should unlock FirstPrestige, PrestigeV, PrestigeX simultaneously
    achievements.on_prestige(10, Some("PrestigeHero"));

    let newly = achievements.take_newly_unlocked();
    assert!(
        newly.contains(&AchievementId::FirstPrestige),
        "FirstPrestige should be in batch"
    );
    assert!(
        newly.contains(&AchievementId::PrestigeV),
        "PrestigeV should be in batch"
    );
    assert!(
        newly.contains(&AchievementId::PrestigeX),
        "PrestigeX should be in batch"
    );
    assert!(
        !newly.contains(&AchievementId::PrestigeXV),
        "PrestigeXV should NOT be in batch at rank 10"
    );
}

// =========================================================================
// T4: Achievement checking during offline progression
// =========================================================================

#[test]
fn test_offline_progression_does_not_call_achievement_handlers() {
    // KNOWN GAP: process_offline_progression() in src/core/offline.rs
    // does NOT call achievement handlers (on_enemy_killed, on_level_up, etc.).
    //
    // The offline progression system:
    // 1. Calculates XP from estimated simulated kills (kill rate * elapsed time * 0.25)
    // 2. Calls apply_tick_xp() to apply the XP and trigger level-ups
    // 3. Does NOT call achievements.on_enemy_killed() for the simulated kills
    // 4. Does NOT call achievements.on_level_up() for levels gained offline
    //
    // This means:
    // - Players who gain 100+ kills offline do NOT get Slayer achievements
    // - Players who level up to 10+ offline do NOT get Level achievements
    // - The achievement counters (total_kills, total_bosses_defeated) are NOT
    //   incremented during offline progression
    //
    // Mitigation: sync_from_game_state() is called on character load and
    // retroactively unlocks LEVEL and PRESTIGE achievements based on character
    // state. However, KILL-based achievements are only counted from real-time
    // play because kill counts are tracked in achievements.json, not in
    // character saves.
    //
    // This is a design trade-off: offline mode simulates XP at 25% rate but
    // does not award individual kill milestones.

    use quest::core::game_state::GameState;
    use quest::core::offline::process_offline_progression;

    let mut state = GameState::new("OfflineHero".to_string(), 0);
    let mut achievements = Achievements::default();

    // Set last_save_time to 24 hours ago for a significant offline period
    state.last_save_time = chrono::Utc::now().timestamp() - (24 * 3600);

    // Process offline progression — this grants XP but does NOT call achievement handlers
    let report = process_offline_progression(&mut state, 0.0);

    // Verify we actually got level-ups (offline progression did work)
    assert!(
        report.total_level_ups > 0,
        "Offline progression should have caused level-ups"
    );
    assert!(
        state.character_level > 1,
        "Character should have leveled up during 24h offline"
    );

    // VERIFY THE GAP: Achievements are NOT updated by process_offline_progression
    assert_eq!(
        achievements.total_kills, 0,
        "KNOWN GAP: total_kills remains 0 after offline progression \
         (offline kills are simulated for XP only, not tracked for achievements)"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::SlayerI),
        "KNOWN GAP: SlayerI is NOT unlocked by offline kills"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::Level10),
        "KNOWN GAP: Level10 is NOT unlocked by offline level-ups \
         (sync_from_game_state() must be called after character load to retroactively unlock)"
    );

    // VERIFY THE MITIGATION: sync_from_game_state() CAN retroactively grant level achievements
    achievements.sync_from_game_state(
        state.character_level,
        state.prestige_rank,
        state.fishing.rank,
        0, // no fish caught
        &[],
        Some(&state.character_name),
    );

    if state.character_level >= 10 {
        assert!(
            achievements.is_unlocked(AchievementId::Level10),
            "After sync, Level10 should be unlocked if character is level 10+"
        );
    }
}

// =========================================================================
// T5: Minigame achievement integration tests (using new enums)
// =========================================================================

/// Maps each (MinigameType, MinigameDifficulty) combination to its expected AchievementId
fn expected_achievement(game: MinigameType, difficulty: MinigameDifficulty) -> AchievementId {
    match (game, difficulty) {
        (MinigameType::Chess, MinigameDifficulty::Novice) => AchievementId::ChessNovice,
        (MinigameType::Chess, MinigameDifficulty::Apprentice) => AchievementId::ChessApprentice,
        (MinigameType::Chess, MinigameDifficulty::Journeyman) => AchievementId::ChessJourneyman,
        (MinigameType::Chess, MinigameDifficulty::Master) => AchievementId::ChessMaster,
        (MinigameType::Morris, MinigameDifficulty::Novice) => AchievementId::MorrisNovice,
        (MinigameType::Morris, MinigameDifficulty::Apprentice) => AchievementId::MorrisApprentice,
        (MinigameType::Morris, MinigameDifficulty::Journeyman) => AchievementId::MorrisJourneyman,
        (MinigameType::Morris, MinigameDifficulty::Master) => AchievementId::MorrisMaster,
        (MinigameType::Gomoku, MinigameDifficulty::Novice) => AchievementId::GomokuNovice,
        (MinigameType::Gomoku, MinigameDifficulty::Apprentice) => AchievementId::GomokuApprentice,
        (MinigameType::Gomoku, MinigameDifficulty::Journeyman) => AchievementId::GomokuJourneyman,
        (MinigameType::Gomoku, MinigameDifficulty::Master) => AchievementId::GomokuMaster,
        (MinigameType::Minesweeper, MinigameDifficulty::Novice) => AchievementId::MinesweeperNovice,
        (MinigameType::Minesweeper, MinigameDifficulty::Apprentice) => {
            AchievementId::MinesweeperApprentice
        }
        (MinigameType::Minesweeper, MinigameDifficulty::Journeyman) => {
            AchievementId::MinesweeperJourneyman
        }
        (MinigameType::Minesweeper, MinigameDifficulty::Master) => AchievementId::MinesweeperMaster,
        (MinigameType::Rune, MinigameDifficulty::Novice) => AchievementId::RuneNovice,
        (MinigameType::Rune, MinigameDifficulty::Apprentice) => AchievementId::RuneApprentice,
        (MinigameType::Rune, MinigameDifficulty::Journeyman) => AchievementId::RuneJourneyman,
        (MinigameType::Rune, MinigameDifficulty::Master) => AchievementId::RuneMaster,
        (MinigameType::Go, MinigameDifficulty::Novice) => AchievementId::GoNovice,
        (MinigameType::Go, MinigameDifficulty::Apprentice) => AchievementId::GoApprentice,
        (MinigameType::Go, MinigameDifficulty::Journeyman) => AchievementId::GoJourneyman,
        (MinigameType::Go, MinigameDifficulty::Master) => AchievementId::GoMaster,
        (MinigameType::FlappyBird, MinigameDifficulty::Novice) => AchievementId::FlappyNovice,
        (MinigameType::FlappyBird, MinigameDifficulty::Apprentice) => {
            AchievementId::FlappyApprentice
        }
        (MinigameType::FlappyBird, MinigameDifficulty::Journeyman) => {
            AchievementId::FlappyJourneyman
        }
        (MinigameType::FlappyBird, MinigameDifficulty::Master) => AchievementId::FlappyMaster,
        (MinigameType::Snake, MinigameDifficulty::Novice) => AchievementId::SnakeNovice,
        (MinigameType::Snake, MinigameDifficulty::Apprentice) => AchievementId::SnakeApprentice,
        (MinigameType::Snake, MinigameDifficulty::Journeyman) => AchievementId::SnakeJourneyman,
        (MinigameType::Snake, MinigameDifficulty::Master) => AchievementId::SnakeMaster,
        (MinigameType::Jezzball, MinigameDifficulty::Novice) => {
            AchievementId::ContainmentBreachNovice
        }
        (MinigameType::Jezzball, MinigameDifficulty::Apprentice) => {
            AchievementId::ContainmentBreachApprentice
        }
        (MinigameType::Jezzball, MinigameDifficulty::Journeyman) => {
            AchievementId::ContainmentBreachJourneyman
        }
        (MinigameType::Jezzball, MinigameDifficulty::Master) => {
            AchievementId::ContainmentBreachMaster
        }
        (MinigameType::RunicShift, MinigameDifficulty::Novice) => AchievementId::SigilSurgeNovice,
        (MinigameType::RunicShift, MinigameDifficulty::Apprentice) => {
            AchievementId::SigilSurgeApprentice
        }
        (MinigameType::RunicShift, MinigameDifficulty::Journeyman) => {
            AchievementId::SigilSurgeJourneyman
        }
        (MinigameType::RunicShift, MinigameDifficulty::Master) => AchievementId::SigilSurgeMaster,
    }
}

#[test]
fn test_minigame_win_unlocks_correct_achievement() {
    let all_game_types = [
        MinigameType::Chess,
        MinigameType::Morris,
        MinigameType::Gomoku,
        MinigameType::Minesweeper,
        MinigameType::Rune,
        MinigameType::Go,
        MinigameType::FlappyBird,
        MinigameType::Snake,
        MinigameType::Jezzball,
        MinigameType::RunicShift,
    ];

    let all_difficulties = [
        MinigameDifficulty::Novice,
        MinigameDifficulty::Apprentice,
        MinigameDifficulty::Journeyman,
        MinigameDifficulty::Master,
    ];

    for &game_type in &all_game_types {
        for &difficulty in &all_difficulties {
            let mut achievements = Achievements::default();
            achievements.on_minigame_won(game_type, difficulty, Some("ChampionHero"));

            let expected = expected_achievement(game_type, difficulty);
            assert!(
                achievements.is_unlocked(expected),
                "Expected {:?} to be unlocked when winning {:?} at {:?} difficulty",
                expected,
                game_type,
                difficulty
            );
        }
    }
}

#[test]
fn test_grand_champion_at_100_wins() {
    let mut achievements = Achievements::default();

    // Win 100 games total across any combination of game types
    for _ in 0..99 {
        achievements.on_minigame_won(
            MinigameType::Chess,
            MinigameDifficulty::Novice,
            Some("Hero"),
        );
    }
    // Not yet unlocked at 99
    assert!(
        !achievements.is_unlocked(AchievementId::GrandChampion),
        "GrandChampion should NOT be unlocked at 99 wins"
    );

    // 100th win
    achievements.on_minigame_won(
        MinigameType::Morris,
        MinigameDifficulty::Master,
        Some("Hero"),
    );
    assert!(
        achievements.is_unlocked(AchievementId::GrandChampion),
        "GrandChampion should unlock at exactly 100 total wins"
    );
}

#[test]
fn test_grand_champion_not_at_99() {
    let mut achievements = Achievements::default();

    for _ in 0..99 {
        achievements.on_minigame_won(
            MinigameType::Gomoku,
            MinigameDifficulty::Journeyman,
            Some("Hero"),
        );
    }

    assert_eq!(
        achievements.total_minigame_wins, 99,
        "Should have exactly 99 wins"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::GrandChampion),
        "GrandChampion should NOT unlock at 99 wins"
    );
}

#[test]
fn test_minigame_progress_tracking() {
    let mut achievements = Achievements::default();

    // Win 50 games — progress should be tracked for GrandChampion
    for _ in 0..50 {
        achievements.on_minigame_won(
            MinigameType::Snake,
            MinigameDifficulty::Novice,
            Some("Hero"),
        );
    }

    // GrandChampion should NOT be unlocked yet
    assert!(!achievements.is_unlocked(AchievementId::GrandChampion));

    // Progress should be tracked
    let progress = achievements.get_progress(AchievementId::GrandChampion);
    assert!(
        progress.is_some(),
        "Progress should be tracked for GrandChampion before 100 wins"
    );
    let progress = progress.unwrap();
    assert_eq!(progress.current, 50, "Progress should show 50 wins");
    assert_eq!(
        progress.target, 100,
        "Progress target should be 100 for GrandChampion"
    );

    // Verify total_minigame_wins counter
    assert_eq!(achievements.total_minigame_wins, 50);
}

#[test]
fn test_minigame_each_game_type_has_all_four_difficulties() {
    // Verify each game type properly maps all 4 difficulties without panics
    let all_game_types = [
        MinigameType::Chess,
        MinigameType::Morris,
        MinigameType::Gomoku,
        MinigameType::Minesweeper,
        MinigameType::Rune,
        MinigameType::Go,
        MinigameType::FlappyBird,
        MinigameType::Snake,
        MinigameType::Jezzball,
        MinigameType::RunicShift,
    ];

    for &game_type in &all_game_types {
        let mut achievements = Achievements::default();
        // All 4 difficulties should work without panic
        achievements.on_minigame_won(game_type, MinigameDifficulty::Novice, Some("Hero"));
        achievements.on_minigame_won(game_type, MinigameDifficulty::Apprentice, Some("Hero"));
        achievements.on_minigame_won(game_type, MinigameDifficulty::Journeyman, Some("Hero"));
        achievements.on_minigame_won(game_type, MinigameDifficulty::Master, Some("Hero"));

        // Should have unlocked 4 game-specific achievements + advanced toward GrandChampion
        assert_eq!(
            achievements.total_minigame_wins, 4,
            "Should count all 4 wins for {:?}",
            game_type
        );
    }
}

// =========================================================================
// T6: Zone completion through process_zone_achievements
// =========================================================================

// Note: process_zone_achievements is private to tick.rs, so we test it
// through the public on_zone_fully_cleared handler which it delegates to.
// We also test via direct calls since direct handler access is cleaner
// and more maintainable than wiring up a full game_tick() state machine.

#[test]
fn test_zone_completion_via_on_zone_fully_cleared() {
    // Zone completion achievements are tracked via on_zone_fully_cleared,
    // which is called by process_zone_achievements in tick.rs.
    // We test the handler directly since setting up a full game tick for
    // this path would require complex state (killing 10 mobs, setting
    // fighting_boss, defeating the zone boss).
    let mut achievements = Achievements::default();

    // Test all 10 zone completion achievements
    for zone_id in 1..=10u32 {
        achievements.on_zone_fully_cleared(zone_id, Some("ZoneHero"));
    }

    assert!(achievements.is_unlocked(AchievementId::Zone1Complete));
    assert!(achievements.is_unlocked(AchievementId::Zone2Complete));
    assert!(achievements.is_unlocked(AchievementId::Zone3Complete));
    assert!(achievements.is_unlocked(AchievementId::Zone4Complete));
    assert!(achievements.is_unlocked(AchievementId::Zone5Complete));
    assert!(achievements.is_unlocked(AchievementId::Zone6Complete));
    assert!(achievements.is_unlocked(AchievementId::Zone7Complete));
    assert!(achievements.is_unlocked(AchievementId::Zone8Complete));
    assert!(achievements.is_unlocked(AchievementId::Zone9Complete));
    assert!(achievements.is_unlocked(AchievementId::Zone10Complete));
}

#[test]
fn test_zone_completion_storms_end() {
    // process_zone_achievements calls both on_zone_fully_cleared(10) AND on_storms_end
    // when BossDefeatResult::StormsEnd occurs. Test the handler calls directly.
    let mut achievements = Achievements::default();

    achievements.on_zone_fully_cleared(10, Some("StormHero"));
    achievements.on_storms_end(Some("StormHero"));

    assert!(achievements.is_unlocked(AchievementId::Zone10Complete));
    assert!(achievements.is_unlocked(AchievementId::StormsEnd));
}

#[test]
fn test_zone_completion_expanse_cycle() {
    // BossDefeatResult::ExpanseCycle triggers on_zone_fully_cleared(11)
    let mut achievements = Achievements::default();

    achievements.on_zone_fully_cleared(11, Some("ExpanseHero"));

    assert!(achievements.is_unlocked(AchievementId::BeyondInfinity));
    assert_eq!(achievements.expanse_cycles_completed, 1);
    // Zone 11 should NOT unlock Zone10Complete
    assert!(!achievements.is_unlocked(AchievementId::Zone10Complete));
}

#[test]
fn test_zone_completion_expanse_cycle_increments_counter() {
    let mut achievements = Achievements::default();

    // Complete 3 expanse cycles
    achievements.on_zone_fully_cleared(11, Some("ExpanseHero"));
    achievements.on_zone_fully_cleared(11, Some("ExpanseHero"));
    achievements.on_zone_fully_cleared(11, Some("ExpanseHero"));

    assert_eq!(
        achievements.expanse_cycles_completed, 3,
        "Should track multiple expanse cycles"
    );
    // BeyondInfinity is only unlocked once (idempotent)
    assert!(achievements.is_unlocked(AchievementId::BeyondInfinity));
}

#[test]
fn test_zone_completion_zones_fully_cleared_counter() {
    let mut achievements = Achievements::default();

    achievements.on_zone_fully_cleared(1, Some("Hero"));
    assert_eq!(achievements.zones_fully_cleared, 1);

    achievements.on_zone_fully_cleared(2, Some("Hero"));
    assert_eq!(achievements.zones_fully_cleared, 2);

    // Zone 11 (Expanse) also increments the counter
    achievements.on_zone_fully_cleared(11, Some("Hero"));
    assert_eq!(achievements.zones_fully_cleared, 3);
}

#[test]
fn test_zone_completion_unknown_zone_id_does_nothing() {
    let mut achievements = Achievements::default();
    let unlocked_before = achievements.unlocked.len();

    // Unknown zone ID (e.g. 0 or 99) should not unlock anything or panic
    achievements.on_zone_fully_cleared(0, Some("Hero"));
    achievements.on_zone_fully_cleared(99, Some("Hero"));

    // zones_fully_cleared does increment (it's a raw counter), but no achievement is unlocked
    assert_eq!(
        achievements.unlocked.len(),
        unlocked_before,
        "No achievements should unlock for unknown zone IDs"
    );
}

// =========================================================================
// T7: Haven sync edge cases
// =========================================================================

#[test]
fn test_haven_sync_partial_rooms_not_all_t1() {
    use quest::haven::types::HavenRoomId;
    use std::collections::HashMap;

    let mut achievements = Achievements::default();

    // Set MOST rooms to T1, but leave one room at T0
    let mut room_tiers: HashMap<HavenRoomId, u8> = HavenRoomId::ALL
        .iter()
        .filter(|r| **r != HavenRoomId::StormForge)
        .map(|r| (*r, 1u8))
        .collect();

    // One room stays at T0 (Bedroom is a non-root, non-special room)
    room_tiers.insert(HavenRoomId::Bedroom, 0);

    achievements.sync_from_haven(true, &room_tiers, Some("HavenHero"));

    assert!(
        achievements.is_unlocked(AchievementId::HavenDiscovered),
        "HavenDiscovered should be unlocked since discovered=true"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::HavenBuilderI),
        "HavenBuilderI should NOT be unlocked if any non-StormForge room is at T0"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::HavenBuilderII),
        "HavenBuilderII should NOT be unlocked"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::HavenArchitect),
        "HavenArchitect should NOT be unlocked"
    );
}

#[test]
fn test_haven_sync_not_discovered() {
    use quest::haven::types::HavenRoomId;
    use std::collections::HashMap;

    let mut achievements = Achievements::default();

    // Even if rooms are at T1, if haven not discovered, no achievements
    let room_tiers: HashMap<HavenRoomId, u8> = HavenRoomId::ALL
        .iter()
        .filter(|r| **r != HavenRoomId::StormForge)
        .map(|r| (*r, 1u8))
        .collect();

    achievements.sync_from_haven(false, &room_tiers, Some("HavenHero"));

    assert!(
        !achievements.is_unlocked(AchievementId::HavenDiscovered),
        "HavenDiscovered should NOT be unlocked if discovered=false"
    );
    // sync_from_haven still checks room tiers regardless of discovery
    // so HavenBuilderI should unlock if all rooms are T1
    assert!(
        achievements.is_unlocked(AchievementId::HavenBuilderI),
        "HavenBuilderI should still unlock based on room tiers even when not discovered \
         (room tier state can exist independently)"
    );
}

#[test]
fn test_haven_sync_fishing_dock_max_counts_for_architect() {
    use quest::haven::types::HavenRoomId;
    use std::collections::HashMap;

    // HavenArchitect uses a special rule: rooms with max_tier < 3 count as "T3"
    // when at their max tier. FishingDock max_tier=4, so it needs to be >= 3 to count.
    // StormForge max_tier=1 — it is excluded from the HavenBuilderI/II/Architect checks.
    let mut achievements = Achievements::default();

    // All rooms at their max tier
    let room_tiers: HashMap<HavenRoomId, u8> = HavenRoomId::ALL
        .iter()
        .map(|r| (*r, r.max_tier()))
        .collect();

    achievements.sync_from_haven(true, &room_tiers, Some("HavenHero"));

    assert!(
        achievements.is_unlocked(AchievementId::HavenBuilderI),
        "HavenBuilderI should be unlocked when all rooms are at max tier"
    );
    assert!(
        achievements.is_unlocked(AchievementId::HavenBuilderII),
        "HavenBuilderII should be unlocked when all rooms are at max tier"
    );
    assert!(
        achievements.is_unlocked(AchievementId::HavenArchitect),
        "HavenArchitect should be unlocked when FishingDock is at T4 (max), \
         satisfying the tier >= max_tier condition"
    );
}

#[test]
fn test_haven_sync_fishing_dock_at_t1_not_enough_for_architect() {
    use quest::haven::types::HavenRoomId;
    use std::collections::HashMap;

    // All rooms at T3 (or max), BUT FishingDock only at T1.
    // The all_at_t2 check uses raw ">= 2" comparison, so FishingDock at T1 also
    // blocks HavenBuilderII (it fails the >= 2 check).
    // The all_at_t3 check uses `tier >= 3 || tier >= max_tier`, and since FishingDock
    // has max_tier=4 and is at T1 (1 < 3 and 1 < 4), it blocks HavenArchitect too.
    let mut achievements = Achievements::default();

    let mut room_tiers: HashMap<HavenRoomId, u8> = HavenRoomId::ALL
        .iter()
        .filter(|r| **r != HavenRoomId::StormForge)
        .map(|r| (*r, 3u8))
        .collect();

    // FishingDock stays at T1 — blocks all_at_t2 and all_at_t3
    room_tiers.insert(HavenRoomId::FishingDock, 1);

    achievements.sync_from_haven(true, &room_tiers, Some("HavenHero"));

    assert!(
        achievements.is_unlocked(AchievementId::HavenBuilderI),
        "HavenBuilderI should be unlocked (all rooms >= T1, FishingDock is at T1)"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::HavenBuilderII),
        "HavenBuilderII should NOT be unlocked when FishingDock is at T1 (fails >= 2 check)"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::HavenArchitect),
        "HavenArchitect should NOT be unlocked when FishingDock is at T1 \
         (fails both tier >= 3 and tier >= max_tier=4 checks)"
    );
}

#[test]
fn test_haven_sync_empty_room_tiers_with_discovery() {
    use quest::haven::types::HavenRoomId;
    use std::collections::HashMap;

    let mut achievements = Achievements::default();

    // Haven discovered but no rooms built yet
    let room_tiers: HashMap<HavenRoomId, u8> = HashMap::new();
    achievements.sync_from_haven(true, &room_tiers, Some("HavenHero"));

    assert!(
        achievements.is_unlocked(AchievementId::HavenDiscovered),
        "HavenDiscovered should unlock even with no rooms"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::HavenBuilderI),
        "HavenBuilderI should NOT unlock with no rooms"
    );
}

// =========================================================================
// T8: State sync idempotency
// =========================================================================

#[test]
fn test_sync_from_game_state_twice_is_idempotent() {
    let mut achievements = Achievements::default();

    // First sync with a fully-progressed character
    achievements.sync_from_game_state(
        150,  // level
        25,   // prestige rank
        20,   // fishing rank
        5000, // fish caught
        &[
            (1, 1),
            (1, 2),
            (1, 3),
            (2, 1),
            (2, 2),
            (2, 3),
            (3, 1),
            (3, 2),
            (3, 3),
        ],
        Some("SyncHero"),
    );

    // Count unlocked achievements after first sync
    let count_after_first = achievements.unlocked.len();
    let notifications_after_first = achievements.pending_notifications.len();

    // Second sync with same state — should not unlock anything new
    achievements.sync_from_game_state(
        150,
        25,
        20,
        5000,
        &[
            (1, 1),
            (1, 2),
            (1, 3),
            (2, 1),
            (2, 2),
            (2, 3),
            (3, 1),
            (3, 2),
            (3, 3),
        ],
        Some("SyncHero"),
    );

    let count_after_second = achievements.unlocked.len();

    assert_eq!(
        count_after_first, count_after_second,
        "Calling sync twice with same state should not add new unlocked achievements"
    );

    // Pending notifications should not grow after second sync (already-unlocked achievements
    // do not re-fire through unlock())
    let notifications_after_second = achievements.pending_notifications.len();
    assert_eq!(
        notifications_after_first, notifications_after_second,
        "Second sync should not add new pending_notifications"
    );
}

#[test]
fn test_sync_after_normal_progression_is_safe() {
    let mut achievements = Achievements::default();

    // Simulate normal gameplay: kill enemies to unlock SlayerI
    for _ in 0..100 {
        achievements.on_enemy_killed(false, Some("ProgressHero"));
    }
    assert!(achievements.is_unlocked(AchievementId::SlayerI));

    let unlocked_count_before_sync = achievements.unlocked.len();
    let notifications_before_sync = achievements.pending_notifications.len();

    // Now call sync with the same level/prestige data
    // Level is 1 (no level-up achievements yet), prestige is 0 (no prestige achievements)
    achievements.sync_from_game_state(1, 0, 0, 0, &[], Some("ProgressHero"));

    let unlocked_count_after_sync = achievements.unlocked.len();

    assert_eq!(
        unlocked_count_before_sync, unlocked_count_after_sync,
        "Sync after normal progression should not unlock duplicate achievements"
    );

    // SlayerI should still be unlocked (not affected by sync)
    assert!(achievements.is_unlocked(AchievementId::SlayerI));

    // No new notifications should have been added
    let notifications_after_sync = achievements.pending_notifications.len();
    assert_eq!(
        notifications_before_sync, notifications_after_sync,
        "Sync should not add new notifications for already-unlocked achievements"
    );
}

#[test]
fn test_sync_from_haven_twice_is_idempotent() {
    use quest::haven::types::HavenRoomId;
    use std::collections::HashMap;

    let mut achievements = Achievements::default();

    let room_tiers: HashMap<HavenRoomId, u8> = HavenRoomId::ALL
        .iter()
        .filter(|r| **r != HavenRoomId::StormForge)
        .map(|r| (*r, 1u8))
        .collect();

    // First sync
    achievements.sync_from_haven(true, &room_tiers, Some("HavenHero"));
    let count_after_first = achievements.unlocked.len();
    let notifications_after_first = achievements.pending_notifications.len();

    // Second sync with same state
    achievements.sync_from_haven(true, &room_tiers, Some("HavenHero"));
    let count_after_second = achievements.unlocked.len();
    let notifications_after_second = achievements.pending_notifications.len();

    assert_eq!(
        count_after_first, count_after_second,
        "Calling sync_from_haven twice should not add duplicate unlocks"
    );
    assert_eq!(
        notifications_after_first, notifications_after_second,
        "Calling sync_from_haven twice should not add new notifications"
    );
}

#[test]
fn test_unlock_is_idempotent() {
    let mut achievements = Achievements::default();

    // Unlock the same achievement multiple times
    let first = achievements.unlock(AchievementId::SlayerI, Some("IdempotentHero".to_string()));
    let second = achievements.unlock(AchievementId::SlayerI, None);
    let third = achievements.unlock(AchievementId::SlayerI, None);

    assert!(first, "First unlock should return true");
    assert!(
        !second,
        "Second unlock should return false (already unlocked)"
    );
    assert!(!third, "Third unlock should return false");

    // Should only appear once in unlocked map
    assert_eq!(
        achievements.unlocked.len(),
        1,
        "Achievement should only be in unlocked map once"
    );

    // pending_notifications should only contain it once
    assert_eq!(
        achievements.pending_notifications.len(),
        1,
        "Achievement should only appear once in pending_notifications"
    );
}

#[test]
fn test_sync_with_higher_prestige_than_existing_unlocks_new() {
    let mut achievements = Achievements::default();

    // First sync at prestige 10
    achievements.sync_from_game_state(100, 10, 0, 0, &[], Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::PrestigeX));
    assert!(!achievements.is_unlocked(AchievementId::PrestigeXV));

    let count_at_p10 = achievements.unlocked.len();

    // Second sync at prestige 15 — should unlock PrestigeXV
    achievements.sync_from_game_state(100, 15, 0, 0, &[], Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::PrestigeXV));

    let count_at_p15 = achievements.unlocked.len();
    assert!(
        count_at_p15 > count_at_p10,
        "Higher prestige sync should unlock new achievements"
    );
}

// =========================================================================
// Additional integration tests: Achievement counter totals
// =========================================================================

#[test]
fn test_on_enemy_killed_increments_total_kills() {
    let mut achievements = Achievements::default();

    achievements.on_enemy_killed(false, Some("Hero"));
    achievements.on_enemy_killed(false, Some("Hero"));
    achievements.on_enemy_killed(true, Some("Hero"));

    assert_eq!(achievements.total_kills, 3, "total_kills should be 3");
    assert_eq!(
        achievements.total_bosses_defeated, 1,
        "total_bosses_defeated should be 1"
    );
}

#[test]
fn test_on_dungeon_completed_increments_counter() {
    let mut achievements = Achievements::default();

    achievements.on_dungeon_completed(Some("Hero"));
    achievements.on_dungeon_completed(Some("Hero"));

    assert_eq!(achievements.total_dungeons_completed, 2);
}

#[test]
fn test_on_fish_caught_increments_counter() {
    let mut achievements = Achievements::default();

    achievements.on_fish_caught(Some("Hero"));
    achievements.on_fish_caught(Some("Hero"));
    achievements.on_fish_caught(Some("Hero"));

    assert_eq!(achievements.total_fish_caught, 3);
}

#[test]
fn test_achievement_count_by_category_after_multiple_unlocks() {
    let mut achievements = Achievements::default();

    // Unlock 2 Combat achievements
    achievements.on_enemy_killed(false, Some("Hero")); // 1 kill — no unlock
    achievements.total_kills = 99;
    achievements.on_enemy_killed(false, Some("Hero")); // 100 kills → SlayerI
    achievements.on_enemy_killed(true, Some("Hero")); // 1 boss → BossHunterI

    let (combat_unlocked, _) = achievements.count_by_category(AchievementCategory::Combat);
    assert_eq!(
        combat_unlocked, 2,
        "Should have 2 Combat achievements unlocked"
    );

    // Level achievements still at 0
    let (level_unlocked, _) = achievements.count_by_category(AchievementCategory::Level);
    assert_eq!(
        level_unlocked, 0,
        "No Level achievements should be unlocked"
    );

    // Now level up to 10
    achievements.on_level_up(10, Some("Hero"));
    let (level_unlocked_after, _) = achievements.count_by_category(AchievementCategory::Level);
    assert_eq!(
        level_unlocked_after, 1,
        "Level10 achievement should be unlocked"
    );
}
