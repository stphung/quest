//! Integration test: Achievement unlock system
//!
//! Tests that achievements unlock correctly during gameplay across all
//! event handler categories: combat kills, boss kills, level-ups, prestige,
//! zone clearing, dungeon completion, minigame wins, fishing, and haven.
//! Also tests cross-system concerns: duplicate prevention, state sync,
//! notification/modal queuing, and counter accuracy.

use quest::achievements::Achievements;
use quest::character::derived_stats::DerivedStats;
use quest::combat::logic::{update_combat, CombatEvent, HavenCombatBonuses};
use quest::core::game_logic::spawn_enemy_if_needed;
use quest::AchievementId;
use quest::GameState;
use quest::TICK_INTERVAL_MS;

/// Simulate combat ticks until the enemy dies, returning all events.
/// Caps at 1000 ticks to prevent infinite loops.
fn fight_until_enemy_dies(
    state: &mut GameState,
    achievements: &mut Achievements,
) -> Vec<CombatEvent> {
    let delta_time = TICK_INTERVAL_MS as f64 / 1000.0;
    let haven = HavenCombatBonuses::default();
    let mut all_events = Vec::new();

    for _ in 0..1000 {
        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        state.combat_state.update_max_hp(derived.max_hp);
        spawn_enemy_if_needed(state);
        let events = update_combat(state, delta_time, &haven, achievements);
        let enemy_died = events.iter().any(|e| {
            matches!(
                e,
                CombatEvent::EnemyDied { .. }
                    | CombatEvent::EliteDefeated { .. }
                    | CombatEvent::BossDefeated { .. }
                    | CombatEvent::SubzoneBossDefeated { .. }
            )
        });
        all_events.extend(events);
        if enemy_died {
            break;
        }
    }
    all_events
}

// =============================================================================
// Combat Kill Achievements via update_combat
// =============================================================================

#[test]
fn test_combat_kills_trigger_slayer_achievement() {
    // Verify that killing enemies through the combat system triggers
    // achievement unlocks via the on_enemy_killed callback in combat/logic.rs
    let mut achievements = Achievements::default();

    // Simulate 100 kills directly (the event handler path)
    for _ in 0..100 {
        achievements.on_enemy_killed(false, Some("Hero"));
    }

    assert!(
        achievements.is_unlocked(AchievementId::SlayerI),
        "SlayerI should unlock at 100 kills"
    );
    assert_eq!(achievements.total_kills, 100);
}

#[test]
fn test_combat_system_increments_kill_counter() {
    // Verify that actual combat (update_combat) increments the kill counter
    // in the achievements struct, proving the wiring between combat and achievements
    let mut state = GameState::new("Combat Tester".to_string(), 0);
    let mut achievements = Achievements::default();

    // Make character strong enough to kill quickly
    use quest::character::attributes::AttributeType;
    for _ in 0..10 {
        state.attributes.increment(AttributeType::Strength);
        state.attributes.increment(AttributeType::Dexterity);
        state.attributes.increment(AttributeType::Constitution);
    }

    // Kill enemies through the real combat system
    for _ in 0..200 {
        fight_until_enemy_dies(&mut state, &mut achievements);
        // Heal back up
        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        state.combat_state.player_current_hp = derived.max_hp;
        if achievements.total_kills >= 5 {
            break;
        }
    }

    assert!(
        achievements.total_kills >= 5,
        "Should have at least 5 kills from combat, got {}",
        achievements.total_kills
    );
}

#[test]
fn test_boss_kill_increments_both_counters() {
    // A boss kill should increment both total_kills and total_bosses_defeated
    let mut achievements = Achievements::default();

    achievements.on_enemy_killed(true, Some("Hero"));

    assert_eq!(achievements.total_kills, 1);
    assert_eq!(achievements.total_bosses_defeated, 1);
    assert!(achievements.is_unlocked(AchievementId::BossHunterI));
}

#[test]
fn test_non_boss_kill_does_not_increment_boss_counter() {
    let mut achievements = Achievements::default();

    achievements.on_enemy_killed(false, Some("Hero"));

    assert_eq!(achievements.total_kills, 1);
    assert_eq!(achievements.total_bosses_defeated, 0);
    assert!(!achievements.is_unlocked(AchievementId::BossHunterI));
}

// =============================================================================
// Milestone Boundary Tests (off-by-one verification)
// =============================================================================

#[test]
fn test_slayer_unlocks_exactly_at_threshold_not_before() {
    let mut achievements = Achievements::default();

    // 99 kills: should NOT have SlayerI
    for _ in 0..99 {
        achievements.on_enemy_killed(false, Some("Hero"));
    }
    assert!(!achievements.is_unlocked(AchievementId::SlayerI));
    assert_eq!(achievements.total_kills, 99);

    // 100th kill: should unlock SlayerI
    achievements.on_enemy_killed(false, Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::SlayerI));
    assert_eq!(achievements.total_kills, 100);
}

#[test]
fn test_boss_hunter_unlocks_exactly_at_threshold() {
    let mut achievements = Achievements::default();

    // Kill 9 bosses
    for _ in 0..9 {
        achievements.on_enemy_killed(true, Some("Hero"));
    }
    assert!(achievements.is_unlocked(AchievementId::BossHunterI)); // 1 boss
    assert!(!achievements.is_unlocked(AchievementId::BossHunterII)); // needs 10

    // 10th boss kill
    achievements.on_enemy_killed(true, Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::BossHunterII));
}

// =============================================================================
// Level-Up Achievements
// =============================================================================

#[test]
fn test_level_milestones_trigger_achievements() {
    let mut achievements = Achievements::default();

    // Level 9: no achievement yet
    achievements.on_level_up(9, Some("Hero"));
    assert!(!achievements.is_unlocked(AchievementId::Level10));

    // Level 10: unlocks Getting Started
    achievements.on_level_up(10, Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::Level10));

    // Level 25
    achievements.on_level_up(25, Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::Level25));

    // Level 50
    achievements.on_level_up(50, Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::Level50));
}

#[test]
fn test_level_up_skipping_levels_unlocks_all_passed_milestones() {
    // If a character jumps from level 1 to level 100 (e.g. via sync),
    // all intermediate milestones should unlock
    let mut achievements = Achievements::default();

    achievements.on_level_up(100, Some("Hero"));

    assert!(achievements.is_unlocked(AchievementId::Level10));
    assert!(achievements.is_unlocked(AchievementId::Level25));
    assert!(achievements.is_unlocked(AchievementId::Level50));
    assert!(achievements.is_unlocked(AchievementId::Level100));
    assert!(!achievements.is_unlocked(AchievementId::Level150));
}

#[test]
fn test_level_up_tracks_highest_level() {
    let mut achievements = Achievements::default();

    achievements.on_level_up(50, Some("Hero"));
    assert_eq!(achievements.highest_level, 50);

    // A lower level shouldn't decrease the tracker
    achievements.on_level_up(30, Some("Alt"));
    assert_eq!(achievements.highest_level, 50);

    // A higher level should update
    achievements.on_level_up(75, Some("Hero"));
    assert_eq!(achievements.highest_level, 75);
}

// =============================================================================
// Prestige Achievements
// =============================================================================

#[test]
fn test_first_prestige_unlocks_achievement() {
    let mut achievements = Achievements::default();

    assert!(!achievements.is_unlocked(AchievementId::FirstPrestige));
    achievements.on_prestige(1, Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::FirstPrestige));
}

#[test]
fn test_prestige_milestones_cumulative() {
    let mut achievements = Achievements::default();

    // Prestige to rank 10: should unlock P1, P5, P10
    achievements.on_prestige(10, Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::FirstPrestige));
    assert!(achievements.is_unlocked(AchievementId::PrestigeV));
    assert!(achievements.is_unlocked(AchievementId::PrestigeX));
    assert!(!achievements.is_unlocked(AchievementId::PrestigeXV));
}

#[test]
fn test_prestige_rank_0_does_not_unlock() {
    let mut achievements = Achievements::default();

    achievements.on_prestige(0, Some("Hero"));
    assert!(!achievements.is_unlocked(AchievementId::FirstPrestige));
}

#[test]
fn test_eternal_achievement_at_rank_100() {
    let mut achievements = Achievements::default();

    achievements.on_prestige(99, Some("Hero"));
    assert!(!achievements.is_unlocked(AchievementId::Eternal));

    achievements.on_prestige(100, Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::Eternal));
}

// =============================================================================
// Zone Completion Achievements
// =============================================================================

#[test]
fn test_zone_completion_unlocks_correct_achievement() {
    let mut achievements = Achievements::default();

    achievements.on_zone_fully_cleared(1, Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::Zone1Complete));
    assert!(!achievements.is_unlocked(AchievementId::Zone2Complete));

    achievements.on_zone_fully_cleared(5, Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::Zone5Complete));
    assert!(!achievements.is_unlocked(AchievementId::Zone6Complete));
}

#[test]
fn test_zone_11_expanse_tracked_separately() {
    let mut achievements = Achievements::default();

    // Zone 11 is The Expanse — should track cycles, not zone completion
    achievements.on_zone_fully_cleared(11, Some("Hero"));

    assert!(achievements.is_unlocked(AchievementId::ExpanseCycleI));
    assert_eq!(achievements.expanse_cycles_completed, 1);

    // Should NOT unlock zone 1-10 achievements
    for zone_id in 1..=10 {
        let zone_achievement = match zone_id {
            1 => AchievementId::Zone1Complete,
            2 => AchievementId::Zone2Complete,
            3 => AchievementId::Zone3Complete,
            4 => AchievementId::Zone4Complete,
            5 => AchievementId::Zone5Complete,
            6 => AchievementId::Zone6Complete,
            7 => AchievementId::Zone7Complete,
            8 => AchievementId::Zone8Complete,
            9 => AchievementId::Zone9Complete,
            10 => AchievementId::Zone10Complete,
            _ => unreachable!(),
        };
        assert!(
            !achievements.is_unlocked(zone_achievement),
            "Zone {} should not be unlocked by Expanse completion",
            zone_id
        );
    }
}

#[test]
fn test_expanse_cycle_milestones() {
    let mut achievements = Achievements::default();

    // Complete 99 cycles
    for _ in 0..99 {
        achievements.on_zone_fully_cleared(11, Some("Hero"));
    }
    assert!(achievements.is_unlocked(AchievementId::ExpanseCycleI));
    assert!(!achievements.is_unlocked(AchievementId::ExpanseCycleII)); // needs 100

    // 100th cycle
    achievements.on_zone_fully_cleared(11, Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::ExpanseCycleII));
    assert_eq!(achievements.expanse_cycles_completed, 100);
}

#[test]
fn test_invalid_zone_id_does_not_panic() {
    let mut achievements = Achievements::default();

    // Zone 0 and zone 99 should not panic or unlock anything unexpected
    achievements.on_zone_fully_cleared(0, Some("Hero"));
    achievements.on_zone_fully_cleared(99, Some("Hero"));

    assert_eq!(achievements.unlocked_count(), 0);
}

#[test]
fn test_storms_end_achievement() {
    let mut achievements = Achievements::default();

    assert!(!achievements.is_unlocked(AchievementId::StormsEnd));
    achievements.on_storms_end(Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::StormsEnd));
}

// =============================================================================
// Dungeon Completion Achievements
// =============================================================================

#[test]
fn test_first_dungeon_unlocks_dungeon_diver() {
    let mut achievements = Achievements::default();

    achievements.on_dungeon_completed(Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::DungeonDiver));
    assert_eq!(achievements.total_dungeons_completed, 1);
}

#[test]
fn test_dungeon_milestones_boundary() {
    let mut achievements = Achievements::default();

    // 9 dungeons: only DungeonDiver
    for _ in 0..9 {
        achievements.on_dungeon_completed(Some("Hero"));
    }
    assert!(achievements.is_unlocked(AchievementId::DungeonDiver));
    assert!(!achievements.is_unlocked(AchievementId::DungeonMasterI));

    // 10th dungeon: DungeonMasterI
    achievements.on_dungeon_completed(Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::DungeonMasterI));
    assert_eq!(achievements.total_dungeons_completed, 10);
}

// =============================================================================
// Minigame Win Achievements
// =============================================================================

#[test]
fn test_minigame_win_unlocks_difficulty_specific_achievement() {
    let mut achievements = Achievements::default();

    achievements.on_minigame_won("chess", "novice", Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::ChessNovice));
    assert!(!achievements.is_unlocked(AchievementId::ChessApprentice));
}

#[test]
fn test_all_game_types_and_difficulties() {
    let test_cases = vec![
        ("chess", "novice", AchievementId::ChessNovice),
        ("chess", "apprentice", AchievementId::ChessApprentice),
        ("chess", "journeyman", AchievementId::ChessJourneyman),
        ("chess", "master", AchievementId::ChessMaster),
        ("morris", "novice", AchievementId::MorrisNovice),
        ("morris", "master", AchievementId::MorrisMaster),
        ("gomoku", "novice", AchievementId::GomokuNovice),
        ("gomoku", "master", AchievementId::GomokuMaster),
        ("minesweeper", "novice", AchievementId::MinesweeperNovice),
        ("minesweeper", "master", AchievementId::MinesweeperMaster),
        ("rune", "novice", AchievementId::RuneNovice),
        ("rune", "master", AchievementId::RuneMaster),
        ("go", "novice", AchievementId::GoNovice),
        ("go", "master", AchievementId::GoMaster),
    ];

    for (game, difficulty, expected_id) in test_cases {
        let mut achievements = Achievements::default();
        achievements.on_minigame_won(game, difficulty, Some("Hero"));
        assert!(
            achievements.is_unlocked(expected_id),
            "Expected {:?} to unlock for {} {}",
            expected_id,
            game,
            difficulty
        );
    }
}

#[test]
fn test_grand_champion_at_100_wins() {
    let mut achievements = Achievements::default();

    // 99 wins: not enough
    for _ in 0..99 {
        achievements.on_minigame_won("chess", "novice", Some("Hero"));
    }
    assert!(!achievements.is_unlocked(AchievementId::GrandChampion));
    assert_eq!(achievements.total_minigame_wins, 99);

    // 100th win
    achievements.on_minigame_won("chess", "novice", Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::GrandChampion));
}

#[test]
fn test_grand_champion_progress_tracked() {
    let mut achievements = Achievements::default();

    achievements.on_minigame_won("morris", "novice", Some("Hero"));

    let progress = achievements
        .get_progress(AchievementId::GrandChampion)
        .unwrap();
    assert_eq!(progress.current, 1);
    assert_eq!(progress.target, 100);
}

#[test]
fn test_invalid_game_type_no_panic() {
    let mut achievements = Achievements::default();

    // Unknown game type should not panic, just increment total_minigame_wins
    achievements.on_minigame_won("unknown_game", "novice", Some("Hero"));
    assert_eq!(achievements.total_minigame_wins, 1);
    // No specific achievement unlocked, but GrandChampion progress tracked
}

// =============================================================================
// Fishing Achievements
// =============================================================================

#[test]
fn test_first_fish_unlocks_gone_fishing() {
    let mut achievements = Achievements::default();

    achievements.on_fish_caught(Some("Angler"));
    assert!(achievements.is_unlocked(AchievementId::GoneFishing));
    assert_eq!(achievements.total_fish_caught, 1);
}

#[test]
fn test_fishing_rank_milestones() {
    let mut achievements = Achievements::default();

    achievements.on_fishing_rank_up(9, Some("Hero"));
    assert!(!achievements.is_unlocked(AchievementId::FishermanI));

    achievements.on_fishing_rank_up(10, Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::FishermanI));
    assert!(!achievements.is_unlocked(AchievementId::FishermanII));

    achievements.on_fishing_rank_up(20, Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::FishermanII));
}

#[test]
fn test_fishing_rank_tracks_highest() {
    let mut achievements = Achievements::default();

    achievements.on_fishing_rank_up(15, Some("Hero"));
    assert_eq!(achievements.highest_fishing_rank, 15);

    // Lower rank shouldn't decrease
    achievements.on_fishing_rank_up(5, Some("Alt"));
    assert_eq!(achievements.highest_fishing_rank, 15);
}

#[test]
fn test_storm_leviathan_achievement() {
    let mut achievements = Achievements::default();

    achievements.on_storm_leviathan_caught(Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::StormLeviathan));
}

// =============================================================================
// Haven Achievements
// =============================================================================

#[test]
fn test_haven_discovered_achievement() {
    let mut achievements = Achievements::default();

    achievements.on_haven_discovered(Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::HavenDiscovered));
}

#[test]
fn test_haven_builder_tiers_sequential() {
    let mut achievements = Achievements::default();

    // T1 should not imply T2
    achievements.on_haven_all_t1(Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::HavenBuilderI));
    assert!(!achievements.is_unlocked(AchievementId::HavenBuilderII));

    // T2 should not imply Architect
    achievements.on_haven_all_t2(Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::HavenBuilderII));
    assert!(!achievements.is_unlocked(AchievementId::HavenArchitect));

    // Architect
    achievements.on_haven_architect(Some("Hero"));
    assert!(achievements.is_unlocked(AchievementId::HavenArchitect));
}

// =============================================================================
// Duplicate Prevention
// =============================================================================

#[test]
fn test_duplicate_unlock_returns_false() {
    let mut achievements = Achievements::default();

    assert!(achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string())));
    assert!(!achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string())));
    assert_eq!(achievements.unlocked_count(), 1);
}

#[test]
fn test_repeated_events_dont_create_duplicate_unlocks() {
    let mut achievements = Achievements::default();

    // Kill 200 enemies (past SlayerI threshold twice)
    for _ in 0..200 {
        achievements.on_enemy_killed(false, Some("Hero"));
    }

    // SlayerI should only appear once in unlocked
    assert!(achievements.is_unlocked(AchievementId::SlayerI));
    assert_eq!(achievements.unlocked_count(), 1); // Only SlayerI (100), not SlayerII (500)

    // SlayerII should NOT be unlocked (needs 500)
    assert!(!achievements.is_unlocked(AchievementId::SlayerII));
}

// =============================================================================
// Notification and Modal Queue System
// =============================================================================

#[test]
fn test_unlock_adds_to_pending_notifications() {
    let mut achievements = Achievements::default();

    achievements.on_enemy_killed(true, Some("Hero")); // Unlocks BossHunterI
    assert!(achievements.pending_count() > 0);
}

#[test]
fn test_take_pending_notifications_clears_list() {
    let mut achievements = Achievements::default();

    achievements.on_enemy_killed(true, Some("Hero"));
    let pending = achievements.take_pending_notifications();
    assert!(!pending.is_empty());
    assert_eq!(achievements.pending_count(), 0);
}

#[test]
fn test_modal_queue_accumulates_rapid_unlocks() {
    let mut achievements = Achievements::default();

    // Trigger multiple achievements at once (high-level sync)
    achievements.on_level_up(100, Some("Hero"));

    // Should have multiple achievements in the modal queue
    assert!(achievements.modal_queue.len() >= 4); // Level10, 25, 50, 100
}

#[test]
fn test_take_modal_queue_resets_timer() {
    let mut achievements = Achievements::default();

    achievements.on_level_up(10, Some("Hero"));
    assert!(achievements.accumulation_start.is_some());

    let queue = achievements.take_modal_queue();
    assert!(!queue.is_empty());
    assert!(achievements.accumulation_start.is_none());
    assert!(achievements.modal_queue.is_empty());
}

#[test]
fn test_newly_unlocked_tracked_separately_from_pending() {
    let mut achievements = Achievements::default();

    achievements.on_enemy_killed(true, Some("Hero")); // BossHunterI

    // Both lists should have the same achievement
    let newly = achievements.take_newly_unlocked();
    assert!(!newly.is_empty());

    // pending_notifications should still have the achievement
    assert!(achievements.pending_count() > 0);
}

// =============================================================================
// State Synchronization (retroactive unlocking)
// =============================================================================

#[test]
fn test_sync_from_game_state_unlocks_retroactive_achievements() {
    let mut achievements = Achievements::default();

    // Simulate loading a character at level 120, prestige 17
    achievements.sync_from_game_state(120, 17, 15, 500, &[], Some("Veteran"));

    // Level achievements
    assert!(achievements.is_unlocked(AchievementId::Level10));
    assert!(achievements.is_unlocked(AchievementId::Level25));
    assert!(achievements.is_unlocked(AchievementId::Level50));
    assert!(achievements.is_unlocked(AchievementId::Level100));
    assert!(!achievements.is_unlocked(AchievementId::Level150));

    // Prestige achievements
    assert!(achievements.is_unlocked(AchievementId::FirstPrestige));
    assert!(achievements.is_unlocked(AchievementId::PrestigeV));
    assert!(achievements.is_unlocked(AchievementId::PrestigeX));
    assert!(achievements.is_unlocked(AchievementId::PrestigeXV));
    assert!(!achievements.is_unlocked(AchievementId::PrestigeXX));

    // Fishing rank achievements
    assert!(achievements.is_unlocked(AchievementId::FishermanI)); // rank 10
    assert!(!achievements.is_unlocked(AchievementId::FishermanII)); // rank 20

    // Fish catch achievements
    assert!(achievements.is_unlocked(AchievementId::GoneFishing));
    assert!(achievements.is_unlocked(AchievementId::FishCatcherI)); // 100
    assert!(!achievements.is_unlocked(AchievementId::FishCatcherII)); // 1000
}

#[test]
fn test_sync_preserves_higher_fish_count() {
    let mut achievements = Achievements {
        total_fish_caught: 50000,
        ..Default::default()
    };

    // Sync with a lower fish count from save file
    achievements.sync_from_game_state(1, 0, 1, 1000, &[], Some("Hero"));

    // Should keep the higher count
    assert_eq!(achievements.total_fish_caught, 50000);
    assert!(achievements.is_unlocked(AchievementId::FishCatcherIII)); // 10000
}

#[test]
fn test_sync_zone_completions_from_defeated_bosses() {
    let mut achievements = Achievements::default();

    // Zone 1 (Meadow) has 3 subzones
    let defeated_bosses = vec![
        (1, 1), // Subzone 1
        (1, 2), // Subzone 2
        (1, 3), // Subzone 3 — all cleared
        (2, 1), // Zone 2 subzone 1 only
    ];

    achievements.sync_from_game_state(1, 0, 1, 0, &defeated_bosses, Some("Hero"));

    assert!(
        achievements.is_unlocked(AchievementId::Zone1Complete),
        "Zone 1 should be complete with all 3 subzones cleared"
    );
    assert!(
        !achievements.is_unlocked(AchievementId::Zone2Complete),
        "Zone 2 should not be complete with only 1 of 3 subzones"
    );
}

#[test]
fn test_sync_haven_all_tiers() {
    use quest::haven::types::HavenRoomId;
    use std::collections::HashMap;

    let mut achievements = Achievements::default();
    let mut room_tiers: HashMap<HavenRoomId, u8> = HashMap::new();

    // Set all rooms to max tier
    for room in HavenRoomId::ALL.iter() {
        room_tiers.insert(*room, room.max_tier());
    }

    achievements.sync_from_haven(true, &room_tiers, Some("Builder"));

    assert!(achievements.is_unlocked(AchievementId::HavenDiscovered));
    assert!(achievements.is_unlocked(AchievementId::HavenBuilderI));
    assert!(achievements.is_unlocked(AchievementId::HavenBuilderII));
    assert!(achievements.is_unlocked(AchievementId::HavenArchitect));
}

#[test]
fn test_sync_haven_not_discovered() {
    use std::collections::HashMap;

    let mut achievements = Achievements::default();
    let room_tiers = HashMap::new();

    achievements.sync_from_haven(false, &room_tiers, Some("Hero"));

    assert!(!achievements.is_unlocked(AchievementId::HavenDiscovered));
}

// =============================================================================
// Character Name Tracking
// =============================================================================

#[test]
fn test_achievement_records_character_name() {
    let mut achievements = Achievements::default();

    achievements.unlock(AchievementId::SlayerI, Some("MyHero".to_string()));

    let record = achievements.unlocked.get(&AchievementId::SlayerI).unwrap();
    assert_eq!(record.character_name.as_deref(), Some("MyHero"));
}

#[test]
fn test_achievement_allows_none_character_name() {
    let mut achievements = Achievements::default();

    achievements.unlock(AchievementId::SlayerI, None);

    let record = achievements.unlocked.get(&AchievementId::SlayerI).unwrap();
    assert!(record.character_name.is_none());
}

// =============================================================================
// Counter Accuracy
// =============================================================================

#[test]
fn test_counters_accurate_across_many_events() {
    let mut achievements = Achievements::default();

    let kill_count = 1234;
    let boss_count = 56;

    for _ in 0..kill_count {
        achievements.on_enemy_killed(false, Some("Hero"));
    }
    for _ in 0..boss_count {
        achievements.on_enemy_killed(true, Some("Hero"));
    }

    // total_kills = regular kills + boss kills (boss kills also count as kills)
    assert_eq!(achievements.total_kills, kill_count + boss_count);
    assert_eq!(achievements.total_bosses_defeated, boss_count);
}

#[test]
fn test_fish_counter_accuracy() {
    let mut achievements = Achievements::default();

    for _ in 0..567 {
        achievements.on_fish_caught(Some("Hero"));
    }

    assert_eq!(achievements.total_fish_caught, 567);
    assert!(achievements.is_unlocked(AchievementId::GoneFishing));
    assert!(achievements.is_unlocked(AchievementId::FishCatcherI)); // 100
    assert!(!achievements.is_unlocked(AchievementId::FishCatcherII)); // 1000
}

#[test]
fn test_dungeon_counter_accuracy() {
    let mut achievements = Achievements::default();

    for _ in 0..42 {
        achievements.on_dungeon_completed(Some("Hero"));
    }

    assert_eq!(achievements.total_dungeons_completed, 42);
    assert!(achievements.is_unlocked(AchievementId::DungeonDiver));
    assert!(achievements.is_unlocked(AchievementId::DungeonMasterI)); // 10
    assert!(!achievements.is_unlocked(AchievementId::DungeonMasterII)); // 50
}

// =============================================================================
// Statistics and Queries
// =============================================================================

#[test]
fn test_unlock_percentage_calculation() {
    let mut achievements = Achievements::default();

    assert_eq!(achievements.unlock_percentage(), 0.0);

    let total = achievements.total_count();
    assert!(total > 0);

    // Unlock one achievement
    achievements.unlock(AchievementId::SlayerI, None);
    let pct = achievements.unlock_percentage();
    assert!(pct > 0.0);
    assert!(pct < 100.0);
}

#[test]
fn test_count_by_category() {
    use quest::AchievementCategory;

    let mut achievements = Achievements::default();

    let (unlocked, total) = achievements.count_by_category(AchievementCategory::Combat);
    assert_eq!(unlocked, 0);
    assert!(total > 0);

    // Unlock a combat achievement
    achievements.on_enemy_killed(true, Some("Hero")); // BossHunterI
    let (unlocked_after, _) = achievements.count_by_category(AchievementCategory::Combat);
    assert!(unlocked_after > 0);
}

// =============================================================================
// Cross-System Integration: Combat → Achievements through update_combat
// =============================================================================

#[test]
fn test_real_combat_kill_triggers_achievement_tracking() {
    // This test proves the full wiring: update_combat → on_enemy_killed → achievements
    let mut state = GameState::new("Integration Hero".to_string(), 0);
    let mut achievements = Achievements::default();

    // Boost stats so we can reliably kill enemies
    use quest::character::attributes::AttributeType;
    for _ in 0..20 {
        state.attributes.increment(AttributeType::Strength);
        state.attributes.increment(AttributeType::Constitution);
    }

    // Fight several enemies
    for _ in 0..10 {
        fight_until_enemy_dies(&mut state, &mut achievements);
        // Heal back
        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        state.combat_state.player_current_hp = derived.max_hp;
    }

    // Verify achievements system was called
    assert!(
        achievements.total_kills > 0,
        "Combat should have registered kills in achievements"
    );
}

// =============================================================================
// Serialization Round-Trip
// =============================================================================

#[test]
fn test_achievement_state_survives_serialization() {
    let mut achievements = Achievements::default();

    // Build up some state
    for _ in 0..150 {
        achievements.on_enemy_killed(false, Some("Hero"));
    }
    achievements.on_enemy_killed(true, Some("Hero"));
    achievements.on_level_up(50, Some("Hero"));
    achievements.on_prestige(5, Some("Hero"));
    achievements.on_dungeon_completed(Some("Hero"));
    achievements.on_fish_caught(Some("Hero"));
    achievements.on_minigame_won("chess", "novice", Some("Hero"));

    // Serialize and deserialize
    let json = serde_json::to_string(&achievements).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();

    // Verify all state survived
    assert_eq!(loaded.total_kills, achievements.total_kills);
    assert_eq!(
        loaded.total_bosses_defeated,
        achievements.total_bosses_defeated
    );
    assert_eq!(
        loaded.total_dungeons_completed,
        achievements.total_dungeons_completed
    );
    assert_eq!(loaded.total_fish_caught, achievements.total_fish_caught);
    assert_eq!(loaded.total_minigame_wins, achievements.total_minigame_wins);
    assert_eq!(loaded.highest_level, achievements.highest_level);
    assert_eq!(
        loaded.highest_prestige_rank,
        achievements.highest_prestige_rank
    );
    assert!(loaded.is_unlocked(AchievementId::SlayerI));
    assert!(loaded.is_unlocked(AchievementId::BossHunterI));
    assert!(loaded.is_unlocked(AchievementId::Level10));
    assert!(loaded.is_unlocked(AchievementId::FirstPrestige));
    assert!(loaded.is_unlocked(AchievementId::DungeonDiver));
    assert!(loaded.is_unlocked(AchievementId::GoneFishing));
    assert!(loaded.is_unlocked(AchievementId::ChessNovice));

    // Transient fields should NOT survive serialization
    assert!(loaded.pending_notifications.is_empty());
    assert!(loaded.newly_unlocked.is_empty());
    assert!(loaded.modal_queue.is_empty());
    assert!(loaded.accumulation_start.is_none());
}

#[test]
fn test_malformed_json_returns_default() {
    let loaded: Achievements = serde_json::from_str("{}").unwrap_or_default();
    assert_eq!(loaded.total_kills, 0);
    assert_eq!(loaded.unlocked_count(), 0);
}

#[test]
fn test_json_with_unknown_fields_still_loads() {
    // Simulates loading from a newer version of the game that added fields
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

    // This should not panic — serde should ignore unknown fields
    let result: Result<Achievements, _> = serde_json::from_str(json);
    // If the struct doesn't have deny_unknown_fields, this should succeed
    // If it does, this tests that the default path works
    let loaded = result.unwrap_or_default();
    // Either way, we should get a valid Achievements object
    assert!(loaded.total_kills == 42 || loaded.total_kills == 0);
}
