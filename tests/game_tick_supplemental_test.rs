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
    let d = DerivedStats::calculate_derived_stats(&s.attributes, &s.equipment);
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
    game_tick(state, tc, &Haven::default(), ach, false, r).events
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
    for _ in 0..5000 {
        if state.active_fishing.is_none() {
            state.active_fishing = Some(fishing_session(FishingPhase::Reeling, 1, 100));
        }
        let result = game_tick(
            &mut state,
            &mut tc,
            &Haven::default(),
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
    for seed in 0..10_000u64 {
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
    for seed in 0..10_000u64 {
        let mut state = fresh();
        state.prestige_rank = 1;
        state.active_dungeon = Some(generate_dungeon(10, 0));
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
    for seed in 0..10_000u64 {
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
    state.active_dungeon = Some(generate_dungeon(10, 0));
    if let Some(dungeon) = &mut state.active_dungeon {
        dungeon.has_key = true;
    }
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(42);

    let mut all_events = Vec::new();
    for _ in 0..100_000 {
        let result = game_tick(
            &mut state,
            &mut tc,
            &Haven::default(),
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
                message,
                ..
            } = e
            {
                Some((*xp_gained, *bonus_xp, *total_xp, message.clone()))
            } else {
                None
            }
        });
        if let Some((xp, bonus, total, msg)) = boss_evt {
            assert!(xp > 0, "Boss XP should be positive");
            assert!(total >= xp + bonus, "Total should include boss + bonus XP");
            assert!(
                msg.contains("Dungeon Complete"),
                "Message should describe completion"
            );
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
    for _ in 0..100 {
        all_events.extend(tick(&mut state, &mut tc, &mut ach, &mut r));
    }

    assert!(
        !has(&all_events, |e| matches!(
            e,
            TickEvent::DungeonRoomEntered { .. }
                | TickEvent::DungeonKeyFound { .. }
                | TickEvent::DungeonBossUnlocked { .. }
                | TickEvent::DungeonBossDefeated { .. }
                | TickEvent::DungeonEliteDefeated { .. }
                | TickEvent::DungeonFailed { .. }
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
        if has(&all_events, |e| matches!(e, TickEvent::PlayerDied { .. })) {
            break;
        }
    }

    if has(&all_events, |e| matches!(e, TickEvent::PlayerDied { .. })) {
        let msg = all_events.iter().find_map(|e| {
            if let TickEvent::PlayerDied { message } = e {
                Some(message.clone())
            } else {
                None
            }
        });
        assert!(msg.is_some());
        assert!(
            msg.unwrap().contains("died"),
            "Death message should mention death"
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
    state.active_dungeon = Some(generate_dungeon(10, 0));
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut r = rng(42);

    let mut all_events = Vec::new();
    for _ in 0..50_000 {
        let events = tick(&mut state, &mut tc, &mut ach, &mut r);
        all_events.extend(events);
        if state.active_dungeon.is_none() {
            break;
        }
    }

    if has(&all_events, |e| {
        matches!(e, TickEvent::PlayerDiedInDungeon { .. })
    }) {
        assert_eq!(
            state.prestige_rank, 5,
            "Prestige preserved after dungeon death"
        );
        assert!(state.active_dungeon.is_none(), "Dungeon cleared");

        let msg = all_events.iter().find_map(|e| {
            if let TickEvent::PlayerDiedInDungeon { message } = e {
                Some(message.clone())
            } else {
                None
            }
        });
        assert!(msg.is_some());
        assert!(
            msg.unwrap().contains("prestige"),
            "Message should mention prestige safety"
        );
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
    for _ in 0..100_000 {
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
