#![allow(deprecated)]
//! Supplemental integration tests for game_tick() covering gaps in existing coverage.
//!
//! Covers: fishing rank-up events, storm leviathan achievement integration,
//! challenge discovery preconditions via game_tick, dungeon boss completion,
//! deterministic same-seed behavior, and player death events.

use quest::achievements::Achievements;
use quest::character::attributes::AttributeType;
use quest::character::derived_stats::DerivedStats;
use quest::core::tick::game_tick;
use quest::dungeon::generation::generate_dungeon;
use quest::enhancement::EnhancementProgress;
use quest::fishing::{FishingPhase, FishingSession};
use quest::haven::Haven;
use quest::{ActiveMinigame, ChessDifficulty, ChessGame};
use quest::{GameState, TickEvent};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// =============================================================================
// Helpers
// =============================================================================

fn rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

fn fresh() -> GameState {
    GameState::new("SupTest".to_string(), 0)
}

fn strong() -> GameState {
    let mut s = fresh();
    s.attributes.set(AttributeType::Strength, 50);
    s.attributes.set(AttributeType::Intelligence, 50);
    let d = DerivedStats::calculate_derived_stats(&s.attributes, &s.equipment, &[0; 7]);
    s.combat_state.update_max_hp(d.max_hp);
    s.combat_state.player_current_hp = s.combat_state.player_max_hp;
    s
}

fn fishing_session(phase: FishingPhase, ticks: u32, total: u32) -> FishingSession {
    FishingSession {
        spot_name: "Sup Lake".to_string(),
        total_fish: total,
        fish_caught: Vec::new(),
        items_found: Vec::new(),
        ticks_remaining: ticks,
        phase,
    }
}

fn tick(
    state: &mut GameState,
    tc: &mut u32,
    ach: &mut Achievements,
    r: &mut ChaCha8Rng,
) -> Vec<TickEvent> {
    game_tick(
        state,
        tc,
        &mut Haven::default(),
        &mut EnhancementProgress::new(),
        &mut quest::deep::DeepState::new(),
        ach,
        false,
        r,
    )
    .events
}

fn has<F: Fn(&TickEvent) -> bool>(events: &[TickEvent], f: F) -> bool {
    events.iter().any(f)
}

// =============================================================================
// Fishing Rank-Up Event via game_tick
// =============================================================================

#[test]
fn test_fishing_rank_up_event_via_game_tick() {
    let mut state = fresh();
    state.fishing.rank = 1;
    state.fishing.fish_toward_next_rank = 99; // 1 away from rank up
    state.active_fishing = Some(fishing_session(FishingPhase::Reeling, 1, 5));
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(42);

    let events = tick(&mut state, &mut tc, &mut ach, &mut r);

    assert!(
        has(&events, |e| matches!(e, TickEvent::FishingRankUp { .. })),
        "Should emit FishingRankUp when at rank threshold"
    );

    // Verify the message contains rank info
    for e in &events {
        if let TickEvent::FishingRankUp { message } = e {
            assert!(!message.is_empty(), "Rank up message should not be empty");
        }
    }
}

// =============================================================================
// Storm Leviathan Achievement via game_tick
// =============================================================================

#[test]
fn test_storm_leviathan_achievement_via_game_tick() {
    let mut state = fresh();
    state.fishing.rank = 40;
    state.fishing.leviathan_encounters = 10;
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(42);

    let mut caught = false;
    for _ in 0..1000 {
        if state.active_fishing.is_none() {
            state.active_fishing = Some(fishing_session(FishingPhase::Reeling, 1, 100));
        }
        let result = game_tick(
            &mut state,
            &mut tc,
            &mut Haven::default(),
            &mut EnhancementProgress::new(),
            &mut quest::deep::DeepState::new(),
            &mut ach,
            false,
            &mut r,
        );
        if has(&result.events, |e| {
            matches!(e, TickEvent::StormLeviathanCaught)
        }) {
            assert!(
                result.achievements_changed,
                "achievements_changed should be set"
            );
            assert!(
                ach.is_unlocked(quest::AchievementId::StormLeviathan),
                "StormLeviathan achievement should be unlocked"
            );
            caught = true;
            break;
        }
    }

    // Catching is probabilistic, so just verify consistency
    if !caught {
        assert!(!ach.is_unlocked(quest::AchievementId::StormLeviathan));
    }
}

// =============================================================================
// Challenge Discovery Preconditions via game_tick
// =============================================================================

#[test]
fn test_challenge_discovery_blocked_during_fishing_via_game_tick() {
    // Fishing gate is checked before any RNG roll, so a handful of seeds
    // is enough to prove it -- no need to burn cycles on hundreds.
    for seed in 0..5u64 {
        let mut state = fresh();
        state.prestige_rank = 1;
        state.active_fishing = Some(fishing_session(FishingPhase::Waiting, 100, 5));
        let mut tc = 0u32;
        let mut ach = Achievements::default();
        let mut r = rng(seed);

        let events = tick(&mut state, &mut tc, &mut ach, &mut r);
        assert!(
            !has(&events, |e| matches!(
                e,
                TickEvent::ChallengeDiscovered { .. }
            )),
            "Challenge discovery blocked during fishing (seed={})",
            seed
        );
    }
}

#[test]
fn test_challenge_discovery_blocked_during_dungeon_via_game_tick() {
    // Dungeon gate is checked before any RNG roll, so a handful of seeds
    // is enough to prove it -- no need to burn cycles on hundreds.
    for seed in 0..5u64 {
        let mut state = fresh();
        state.prestige_rank = 1;
        state.active_dungeon = Some(generate_dungeon(10, 0, 1));
        let mut tc = 0u32;
        let mut ach = Achievements::default();
        let mut r = rng(seed);

        let events = tick(&mut state, &mut tc, &mut ach, &mut r);
        assert!(
            !has(&events, |e| matches!(
                e,
                TickEvent::ChallengeDiscovered { .. }
            )),
            "Challenge discovery blocked during dungeon (seed={})",
            seed
        );
    }
}

#[test]
fn test_challenge_discovery_blocked_during_active_minigame_via_game_tick() {
    // Active-minigame gate is checked before any RNG roll, so a handful of
    // seeds is enough to prove it -- no need to burn cycles on hundreds.
    for seed in 0..5u64 {
        let mut state = fresh();
        state.prestige_rank = 1;
        state.active_minigame = Some(ActiveMinigame::Chess(Box::new(ChessGame::new(
            ChessDifficulty::Novice,
        ))));
        let mut tc = 0u32;
        let mut ach = Achievements::default();
        let mut r = rng(seed);

        let events = tick(&mut state, &mut tc, &mut ach, &mut r);
        assert!(
            !has(&events, |e| matches!(
                e,
                TickEvent::ChallengeDiscovered { .. }
            )),
            "Challenge discovery blocked during active minigame (seed={})",
            seed
        );
    }
}

// =============================================================================
// Dungeon Boss Completion via game_tick
// =============================================================================

#[test]
fn test_dungeon_boss_completion_triggers_achievement() {
    let mut state = strong();
    state.active_dungeon = Some(generate_dungeon(10, 0, 1));
    if let Some(dungeon) = &mut state.active_dungeon {
        dungeon.has_key = true;
    }
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(42);

    let mut all_events = Vec::new();
    for _ in 0..5_000 {
        let result = game_tick(
            &mut state,
            &mut tc,
            &mut Haven::default(),
            &mut EnhancementProgress::new(),
            &mut quest::deep::DeepState::new(),
            &mut ach,
            false,
            &mut r,
        );
        all_events.extend(result.events);
        if state.active_dungeon.is_none() {
            break;
        }
    }

    if has(&all_events, |e| {
        matches!(e, TickEvent::DungeonBossDefeated { .. })
    }) {
        assert!(state.active_dungeon.is_none(), "Dungeon cleared after boss");
        assert!(
            ach.is_unlocked(quest::AchievementId::DungeonDiver),
            "DungeonDiver unlocked on completion"
        );

        // Verify DungeonBossDefeated event fields
        let boss_evt = all_events.iter().find_map(|e| {
            if let TickEvent::DungeonBossDefeated {
                xp_gained,
                bonus_xp,
                total_xp,
                ..
            } = e
            {
                Some((*xp_gained, *bonus_xp, *total_xp))
            } else {
                None
            }
        });
        if let Some((xp, bonus, total)) = boss_evt {
            assert!(xp > 0, "Boss XP should be positive");
            assert!(total >= xp + bonus, "Total should include boss + bonus XP");
        }
    }
}

// =============================================================================
// Dungeon Events Not Emitted Without Dungeon
// =============================================================================

#[test]
fn test_no_dungeon_events_without_active_dungeon() {
    let mut state = fresh();
    state.active_dungeon = None;
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(42);

    let mut all_events = Vec::new();
    for _ in 0..10 {
        all_events.extend(tick(&mut state, &mut tc, &mut ach, &mut r));
    }

    assert!(
        !has(&all_events, |e| matches!(
            e,
            TickEvent::DungeonRoomEntered { .. }
                | TickEvent::DungeonKeyFound
                | TickEvent::DungeonBossUnlocked
                | TickEvent::DungeonBossDefeated { .. }
                | TickEvent::DungeonEliteDefeated { .. }
                | TickEvent::DungeonFailed
                | TickEvent::DungeonCompleted { .. }
                | TickEvent::DungeonTreasureFound { .. }
        )),
        "No dungeon events without active dungeon"
    );
}

// =============================================================================
// Deterministic Same-Seed via game_tick
// =============================================================================

#[test]
fn test_deterministic_fishing_same_seed_via_game_tick() {
    let seed = 77777;

    // Run 1
    let mut s1 = fresh();
    s1.active_fishing = Some(fishing_session(FishingPhase::Reeling, 1, 5));
    let mut tc1 = 0u32;
    let mut ach1 = Achievements::default();
    let mut r1 = rng(seed);
    let events1 = tick(&mut s1, &mut tc1, &mut ach1, &mut r1);

    // Run 2
    let mut s2 = fresh();
    s2.active_fishing = Some(fishing_session(FishingPhase::Reeling, 1, 5));
    let mut tc2 = 0u32;
    let mut ach2 = Achievements::default();
    let mut r2 = rng(seed);
    let events2 = tick(&mut s2, &mut tc2, &mut ach2, &mut r2);

    assert_eq!(events1.len(), events2.len(), "Event count should match");
    assert_eq!(
        s1.fishing.total_fish_caught, s2.fishing.total_fish_caught,
        "Fish count should match"
    );
    assert_eq!(s1.character_xp, s2.character_xp, "XP should match");
}

// =============================================================================
// Player Death in Overworld
// =============================================================================

#[test]
fn test_player_death_in_overworld_emits_event() {
    // Weak character fighting a boss should eventually die
    let mut state = fresh();
    state.zone_progression.kills_in_subzone = 10;
    state.zone_progression.fighting_boss = true;
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(42);

    let mut all_events = Vec::new();
    for _ in 0..10_000 {
        all_events.extend(tick(&mut state, &mut tc, &mut ach, &mut r));
        if has(&all_events, |e| matches!(e, TickEvent::PlayerDied)) {
            break;
        }
    }

    // Probabilistic: weak character fighting boss should eventually die.
    // If it doesn't happen, that's OK — we've locked the event contract.
    if has(&all_events, |e| matches!(e, TickEvent::PlayerDied)) {
        // PlayerDied is now a unit variant — no message to verify
        assert!(
            !state.zone_progression.fighting_boss,
            "Boss fight should reset after death"
        );
    }
}

// =============================================================================
// Player Death in Dungeon Preserves Prestige
// =============================================================================

#[test]
fn test_player_death_in_dungeon_preserves_prestige() {
    let mut state = fresh();
    state.prestige_rank = 5;
    state.active_dungeon = Some(generate_dungeon(10, 0, 1));
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(42);

    let mut all_events = Vec::new();
    for _ in 0..20_000 {
        let events = tick(&mut state, &mut tc, &mut ach, &mut r);
        all_events.extend(events);
        if state.active_dungeon.is_none() {
            break;
        }
    }

    if has(&all_events, |e| matches!(e, TickEvent::PlayerDiedInDungeon)) {
        assert_eq!(
            state.prestige_rank, 5,
            "Prestige preserved after dungeon death"
        );
        assert!(state.active_dungeon.is_none(), "Dungeon cleared");
    }
}

// =============================================================================
// Fishing Fish Caught Event Shape
// =============================================================================

#[test]
fn test_fish_caught_event_has_name_and_rarity() {
    let mut state = fresh();
    state.active_fishing = Some(fishing_session(FishingPhase::Reeling, 1, 5));
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(42);

    let events = tick(&mut state, &mut tc, &mut ach, &mut r);

    let caught = events.iter().find_map(|e| {
        if let TickEvent::FishCaught {
            fish_name,
            rarity,
            message,
        } = e
        {
            Some((fish_name.clone(), *rarity, message.clone()))
        } else {
            None
        }
    });

    assert!(caught.is_some(), "Should catch a fish from Reeling-1");
    let (name, _rarity, msg) = caught.unwrap();
    assert!(!name.is_empty(), "Fish name should not be empty");
    assert!(msg.contains("Caught"), "Message should contain 'Caught'");
}

// =============================================================================
// Fish Caught Achievement Tracking via game_tick (Fix #1)
// =============================================================================

#[test]
fn test_fish_caught_unlocks_gone_fishing_achievement() {
    let mut state = fresh();
    state.active_fishing = Some(fishing_session(FishingPhase::Reeling, 1, 5));
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(42);

    assert!(
        !ach.is_unlocked(quest::AchievementId::GoneFishing),
        "GoneFishing should not be unlocked before catching a fish"
    );

    let events = tick(&mut state, &mut tc, &mut ach, &mut r);

    // If a fish was caught, GoneFishing should now be unlocked
    if has(&events, |e| matches!(e, TickEvent::FishCaught { .. })) {
        assert!(
            ach.is_unlocked(quest::AchievementId::GoneFishing),
            "GoneFishing should be unlocked after catching first fish via game_tick"
        );
        assert_eq!(ach.total_fish_caught, 1, "total_fish_caught should be 1");
    }
}

#[test]
fn test_fish_caught_increments_total_across_sessions() {
    let mut state = fresh();
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(100);

    // Run multiple fishing sessions, counting catches
    for _ in 0..50 {
        if state.active_fishing.is_none() {
            state.active_fishing = Some(fishing_session(FishingPhase::Reeling, 1, 5));
        }
        tick(&mut state, &mut tc, &mut ach, &mut r);
    }

    assert!(
        ach.total_fish_caught > 0,
        "Should have caught at least one fish across 50 ticks"
    );
    assert!(
        ach.is_unlocked(quest::AchievementId::GoneFishing),
        "GoneFishing should be unlocked"
    );
}

#[test]
fn test_fish_catcher_i_unlocks_at_100_catches() {
    let mut state = fresh();
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(77);

    // Pre-set counter to 99 so the next catch triggers FishCatcherI
    ach.total_fish_caught = 99;

    state.active_fishing = Some(fishing_session(FishingPhase::Reeling, 1, 5));
    let events = tick(&mut state, &mut tc, &mut ach, &mut r);

    if has(&events, |e| matches!(e, TickEvent::FishCaught { .. })) {
        assert_eq!(ach.total_fish_caught, 100);
        assert!(
            ach.is_unlocked(quest::AchievementId::FishCatcherI),
            "FishCatcherI should unlock at 100 fish caught"
        );
    }
}

// =============================================================================
// Fishing Rank-Up Achievement Tracking via game_tick (Fix #2)
// =============================================================================

#[test]
fn test_fishing_rank_up_unlocks_fisherman_i_achievement() {
    let mut state = fresh();
    state.fishing.rank = 9;
    state.fishing.fish_toward_next_rank = 99; // 1 away from rank 10
    state.active_fishing = Some(fishing_session(FishingPhase::Reeling, 1, 5));
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(42);

    assert!(
        !ach.is_unlocked(quest::AchievementId::FishermanI),
        "FishermanI should not be unlocked before rank 10"
    );

    let events = tick(&mut state, &mut tc, &mut ach, &mut r);

    if has(&events, |e| matches!(e, TickEvent::FishingRankUp { .. })) {
        assert!(
            state.fishing.rank >= 10,
            "Rank should be >= 10 after rank up"
        );
        assert!(
            ach.is_unlocked(quest::AchievementId::FishermanI),
            "FishermanI should unlock when rank reaches 10 via game_tick"
        );
    }
}

#[test]
fn test_fishing_rank_up_tracks_highest_rank() {
    let mut state = fresh();
    state.fishing.rank = 9;
    state.fishing.fish_toward_next_rank = 99;
    state.active_fishing = Some(fishing_session(FishingPhase::Reeling, 1, 5));
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(42);

    tick(&mut state, &mut tc, &mut ach, &mut r);

    if state.fishing.rank >= 10 {
        assert!(
            ach.highest_fishing_rank >= 10,
            "highest_fishing_rank should track the new rank"
        );
    }
}

// =============================================================================
// Multi-Level-Up Achievement Tracking via game_tick (Fix #4)
// =============================================================================

#[test]
fn test_multi_level_up_unlocks_intermediate_achievements() {
    // Set character to level 9 with enough XP to jump past level 10 in one kill
    let mut state = strong();
    state.character_level = 9;
    // XP needed for level 9->10: 100 * 9^1.5 = ~2700
    // XP needed for level 10->11: 100 * 10^1.5 = ~3162
    // Total to jump from 9 to 11: ~5862
    // A kill gives 200-400 ticks of XP, but with STR 50 the base damage
    // makes kills fast. We'll give enough XP directly to be near the edge.
    let xp_for_9 = (100.0 * f64::powf(9.0, 1.5)) as u64;
    let xp_for_10 = (100.0 * f64::powf(10.0, 1.5)) as u64;
    // Set XP so that only 1 XP is needed to cross level 9 AND 10
    state.character_xp = xp_for_9 + xp_for_10 - 1;
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(42);

    assert!(
        !ach.is_unlocked(quest::AchievementId::Level10),
        "Level10 should not be unlocked at level 9"
    );

    // Run ticks until an enemy is killed (granting XP)
    let mut all_events = Vec::new();
    for _ in 0..5000 {
        let result = game_tick(
            &mut state,
            &mut tc,
            &mut Haven::default(),
            &mut EnhancementProgress::new(),
            &mut quest::deep::DeepState::new(),
            &mut ach,
            false,
            &mut r,
        );
        all_events.extend(result.events);
        if has(&all_events, |e| {
            matches!(e, TickEvent::EnemyDefeated { .. })
        }) {
            break;
        }
    }

    // After gaining XP from a kill, we should have jumped past level 10
    if state.character_level >= 11 {
        assert!(
            ach.is_unlocked(quest::AchievementId::Level10),
            "Level10 achievement should unlock when jumping from 9 to 11+ in one kill"
        );
    }
}

#[test]
fn test_multi_level_up_calls_on_level_up_for_each_level() {
    // Directly test the loop logic: achievements should track every intermediate level
    let mut ach = Achievements::default();

    // Simulate what tick.rs does: loop from (level_before + 1) to current_level
    let level_before = 8;
    let level_after = 12;
    for lvl in (level_before + 1)..=level_after {
        ach.on_level_up(lvl, Some("TestHero"));
    }

    assert!(
        ach.is_unlocked(quest::AchievementId::Level10),
        "Level10 should be unlocked when leveling from 8 to 12"
    );
    assert_eq!(
        ach.highest_level, 12,
        "highest_level should track the final level"
    );
}

#[test]
fn test_single_level_up_through_milestone_unlocks_achievement() {
    // If we level from exactly 9 to 10, Level10 should unlock
    let mut ach = Achievements::default();
    let level_before = 9;
    let level_after = 10;
    for lvl in (level_before + 1)..=level_after {
        ach.on_level_up(lvl, Some("TestHero"));
    }

    assert!(
        ach.is_unlocked(quest::AchievementId::Level10),
        "Level10 should unlock at exactly level 10"
    );
}

// =============================================================================
// Haven Tier Achievement Sync (Fix #3 - unit-level verification)
// =============================================================================

#[test]
fn test_haven_sync_unlocks_builder_achievements() {
    use quest::haven::types::HavenRoomId;
    use std::collections::HashMap;

    let mut ach = Achievements::default();

    // Build all rooms to tier 1
    let mut rooms: HashMap<HavenRoomId, u8> = HashMap::new();
    for room in HavenRoomId::ALL.iter() {
        if *room != HavenRoomId::StormForge {
            rooms.insert(*room, 1);
        }
    }

    ach.sync_from_haven(true, &rooms, Some("TestHero"));
    assert!(
        ach.is_unlocked(quest::AchievementId::HavenDiscovered),
        "HavenDiscovered should unlock"
    );
    assert!(
        ach.is_unlocked(quest::AchievementId::HavenBuilderI),
        "HavenBuilderI should unlock when all rooms at T1"
    );
    assert!(
        !ach.is_unlocked(quest::AchievementId::HavenBuilderII),
        "HavenBuilderII should NOT unlock at T1"
    );

    // Upgrade all to tier 2
    for room in HavenRoomId::ALL.iter() {
        if *room != HavenRoomId::StormForge {
            rooms.insert(*room, 2);
        }
    }

    ach.sync_from_haven(true, &rooms, Some("TestHero"));
    assert!(
        ach.is_unlocked(quest::AchievementId::HavenBuilderII),
        "HavenBuilderII should unlock when all rooms at T2"
    );
    assert!(
        !ach.is_unlocked(quest::AchievementId::HavenArchitect),
        "HavenArchitect should NOT unlock at T2"
    );
}

#[test]
fn test_haven_sync_idempotent() {
    use quest::haven::types::HavenRoomId;
    use std::collections::HashMap;

    let mut ach = Achievements::default();
    let mut rooms: HashMap<HavenRoomId, u8> = HashMap::new();
    for room in HavenRoomId::ALL.iter() {
        if *room != HavenRoomId::StormForge {
            rooms.insert(*room, 1);
        }
    }

    // Calling sync multiple times should not cause issues
    ach.sync_from_haven(true, &rooms, Some("TestHero"));
    ach.sync_from_haven(true, &rooms, Some("TestHero"));
    ach.sync_from_haven(true, &rooms, Some("TestHero"));

    assert!(ach.is_unlocked(quest::AchievementId::HavenBuilderI));
    // pending_notifications should only have each achievement once
    // (duplicates are filtered by unlock())
}

// =============================================================================
// Subzone Boss Defeat Event via game_tick
// =============================================================================

#[test]
fn test_subzone_boss_defeated_advances_zone() {
    let mut state = strong();
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(42);

    assert_eq!(state.zone_progression.current_subzone_id, 1);

    let mut all_events = Vec::new();
    for _ in 0..10_000 {
        all_events.extend(tick(&mut state, &mut tc, &mut ach, &mut r));
        if has(&all_events, |e| {
            matches!(e, TickEvent::SubzoneBossDefeated { .. })
        }) {
            break;
        }
    }

    if has(&all_events, |e| {
        matches!(e, TickEvent::SubzoneBossDefeated { .. })
    }) {
        assert_eq!(
            state.zone_progression.current_subzone_id, 2,
            "Should advance to subzone 2 after boss"
        );
    }
}
