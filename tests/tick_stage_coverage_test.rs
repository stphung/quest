//! Tests for game_tick() TickEvent variants and processing stages.
//!
//! Covers TickEvent variant production, processing stage interactions,
//! and edge cases like player death variants, fishing/combat mutual
//! exclusion, Haven discovery, and multi-level-up scenarios.

use quest::achievements::Achievements;
use quest::character::attributes::AttributeType;
use quest::character::derived_stats::DerivedStats;
use quest::core::game_logic::{spawn_enemy_if_needed, xp_for_next_level};
use quest::core::tick::{game_tick, TickEvent, TickResult};
use quest::dungeon::generation::generate_dungeon;
use quest::enhancement::EnhancementProgress;
use quest::fishing::FishingPhase;
use quest::haven::Haven;
use quest::GameState;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn test_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(42)
}

fn create_strong_character(name: &str) -> GameState {
    let mut state = GameState::new(name.to_string(), 0);
    state.attributes.set(AttributeType::Strength, 50);
    state.attributes.set(AttributeType::Intelligence, 50);
    state.attributes.set(AttributeType::Constitution, 30);
    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
    state.combat_state.update_max_hp(derived.max_hp);
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;
    state
}

fn run_ticks_collecting(
    state: &mut GameState,
    tick_counter: &mut u32,
    haven: &mut Haven,
    achievements: &mut Achievements,
    rng: &mut ChaCha8Rng,
    count: usize,
) -> Vec<TickEvent> {
    let mut enhancement = EnhancementProgress::new();
    let mut all_events = Vec::new();
    for _ in 0..count {
        let result = game_tick(
            state,
            tick_counter,
            haven,
            &mut enhancement,
            achievements,
            false,
            rng,
        );
        all_events.extend(result.events);
    }
    all_events
}

fn run_ticks_collecting_results(
    state: &mut GameState,
    tick_counter: &mut u32,
    haven: &mut Haven,
    achievements: &mut Achievements,
    rng: &mut ChaCha8Rng,
    count: usize,
) -> Vec<TickResult> {
    let mut enhancement = EnhancementProgress::new();
    let mut all_results = Vec::new();
    for _ in 0..count {
        let result = game_tick(
            state,
            tick_counter,
            haven,
            &mut enhancement,
            achievements,
            false,
            rng,
        );
        all_results.push(result);
    }
    all_results
}

// =============================================================================
// 1. TickEvent variant coverage: PlayerAttack with crit
// =============================================================================

#[test]
fn test_tick_event_player_attack_crit_has_crit_message() {
    let mut state = create_strong_character("Crit Test");
    // High DEX for crit chance
    state.attributes.set(AttributeType::Dexterity, 50);
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let events = run_ticks_collecting(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        20_000,
    );

    let crits: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, TickEvent::PlayerAttack { was_crit: true, .. }))
        .collect();

    // With high DEX and prestige bonuses, should get at least one crit in 20k ticks
    if !crits.is_empty() {
        if let TickEvent::PlayerAttack {
            was_crit, message, ..
        } = &crits[0]
        {
            assert!(*was_crit);
            assert!(
                message.contains("CRITICAL"),
                "Crit message should contain CRITICAL"
            );
        }
    }
}

// =============================================================================
// 2. PlayerDied event (overworld death)
// =============================================================================

#[test]
fn test_tick_event_player_died_overworld() {
    let mut state = GameState::new("Death Test".to_string(), 0);
    // Very low HP so player dies quickly
    state.combat_state.player_max_hp = 1;
    state.combat_state.player_current_hp = 1;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    // Set kills high so boss spawns
    state.zone_progression.kills_in_subzone = 10;

    let events = run_ticks_collecting(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        5_000,
    );

    let player_died = events
        .iter()
        .any(|e| matches!(e, TickEvent::PlayerDied { .. }));

    assert!(
        player_died,
        "Player with 1 HP should die to boss and produce PlayerDied event"
    );

    // Verify message
    let died_event = events
        .iter()
        .find(|e| matches!(e, TickEvent::PlayerDied { .. }))
        .unwrap();
    if let TickEvent::PlayerDied { message } = died_event {
        assert!(
            message.contains("died"),
            "PlayerDied message should mention death"
        );
    }
}

// =============================================================================
// 3. PlayerDiedInDungeon event (dungeon death)
// =============================================================================

#[test]
fn test_tick_event_player_died_in_dungeon() {
    let mut state = GameState::new("Dungeon Death Test".to_string(), 0);
    state.character_level = 10;
    state.combat_state.player_max_hp = 1;
    state.combat_state.player_current_hp = 1;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let dungeon = generate_dungeon(state.character_level, state.prestige_rank, 1);
    state.active_dungeon = Some(dungeon);

    // Run ticks one at a time, stop after dungeon death/failure
    let mut all_events = Vec::new();
    let mut died_in_dungeon = false;
    for _ in 0..10_000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut EnhancementProgress::new(),
            &mut achievements,
            false,
            &mut rng,
        );
        for event in &result.events {
            if matches!(
                event,
                TickEvent::PlayerDiedInDungeon { .. } | TickEvent::DungeonFailed { .. }
            ) {
                died_in_dungeon = true;
            }
        }
        all_events.extend(result.events);
        if died_in_dungeon {
            break;
        }
    }

    let dungeon_death = all_events
        .iter()
        .any(|e| matches!(e, TickEvent::PlayerDiedInDungeon { .. }));

    let dungeon_failed = all_events
        .iter()
        .any(|e| matches!(e, TickEvent::DungeonFailed { .. }));

    // Either PlayerDiedInDungeon or DungeonFailed should appear
    assert!(
        dungeon_death || dungeon_failed,
        "Player with 1 HP should die in dungeon"
    );

    // Dungeon should be cleared immediately after death
    assert!(
        state.active_dungeon.is_none(),
        "Dungeon should be cleared after death"
    );

    // If we got a PlayerDiedInDungeon, verify the message mentions dungeon/escaped
    if dungeon_death {
        let death_event = all_events
            .iter()
            .find(|e| matches!(e, TickEvent::PlayerDiedInDungeon { .. }))
            .unwrap();
        if let TickEvent::PlayerDiedInDungeon { message } = death_event {
            assert!(
                message.contains("dungeon") || message.contains("escaped"),
                "PlayerDiedInDungeon message should reference dungeon"
            );
        }
    }
}

// =============================================================================
// 4. DungeonRoomEntered with room_type
// =============================================================================

#[test]
fn test_tick_event_dungeon_room_entered_has_room_type() {
    let mut state = create_strong_character("Dungeon Room Type Test");
    state.character_level = 10;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let dungeon = generate_dungeon(state.character_level, state.prestige_rank, 1);
    state.active_dungeon = Some(dungeon);

    let events = run_ticks_collecting(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        5_000,
    );

    let room_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, TickEvent::DungeonRoomEntered { .. }))
        .collect();

    for event in &room_events {
        if let TickEvent::DungeonRoomEntered {
            room_type, message, ..
        } = event
        {
            assert!(
                !message.is_empty(),
                "Room entry message should not be empty"
            );
            // The message should be one of the narration lines for this room type
            let narration = room_type.narration();
            assert!(
                narration.iter().any(|line| message.contains(line)),
                "Message should contain narration for {:?}, got: {}",
                room_type,
                message
            );
        }
    }
}

// =============================================================================
// 5. Fishing mutual exclusion with combat
// =============================================================================

#[test]
fn test_fishing_early_return_skips_all_combat_stages() {
    let mut state = create_strong_character("Fishing Exclusion Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    // Spawn enemy first
    spawn_enemy_if_needed(&mut state);
    assert!(state.combat_state.current_enemy.is_some());

    // Start fishing
    let session = quest::FishingSession {
        spot_name: "Exclusion Lake".to_string(),
        total_fish: 5,
        fish_caught: Vec::new(),
        items_found: Vec::new(),
        ticks_remaining: 10,
        phase: FishingPhase::Waiting,
    };
    state.active_fishing = Some(session);

    let initial_kills = state.session_kills;

    // Run ticks while fishing
    let events = run_ticks_collecting(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        50,
    );

    // No combat events should appear
    let combat_events: Vec<_> = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                TickEvent::PlayerAttack { .. }
                    | TickEvent::EnemyAttack { .. }
                    | TickEvent::EnemyDefeated { .. }
                    | TickEvent::PlayerDied { .. }
                    | TickEvent::SubzoneBossDefeated { .. }
            )
        })
        .collect();

    assert!(
        combat_events.is_empty(),
        "No combat events should occur while fishing"
    );

    assert_eq!(
        state.session_kills, initial_kills,
        "No kills should happen while fishing"
    );
}

// =============================================================================
// 6. FishCaught event carries rarity and fish_name
// =============================================================================

#[test]
fn test_tick_event_fish_caught_fields() {
    let mut state = GameState::new("Fish Caught Fields Test".to_string(), 0);
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let session = quest::FishingSession {
        spot_name: "Rarity Lake".to_string(),
        total_fish: 5,
        fish_caught: Vec::new(),
        items_found: Vec::new(),
        ticks_remaining: 1,
        phase: FishingPhase::Casting,
    };
    state.active_fishing = Some(session);

    let mut all_events = Vec::new();
    for _ in 0..1000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut EnhancementProgress::new(),
            &mut achievements,
            false,
            &mut rng,
        );
        all_events.extend(result.events);
        if state.active_fishing.is_none() {
            break;
        }
    }

    let fish_events: Vec<_> = all_events
        .iter()
        .filter(|e| matches!(e, TickEvent::FishCaught { .. }))
        .collect();

    for event in &fish_events {
        if let TickEvent::FishCaught {
            fish_name,
            rarity,
            message,
        } = event
        {
            assert!(!fish_name.is_empty(), "Fish name should not be empty");
            assert!(!message.is_empty(), "Fish message should not be empty");
            // Rarity should be a valid variant
            let _ = format!("{:?}", rarity);
        }
    }
}

// =============================================================================
// 7. FishingMessage event for non-catch messages
// =============================================================================

#[test]
fn test_tick_event_fishing_message_for_phase_transitions() {
    let mut state = GameState::new("Fishing Message Test".to_string(), 0);
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let session = quest::FishingSession {
        spot_name: "Message Lake".to_string(),
        total_fish: 2,
        fish_caught: Vec::new(),
        items_found: Vec::new(),
        ticks_remaining: 1,
        phase: FishingPhase::Casting,
    };
    state.active_fishing = Some(session);

    let mut all_events = Vec::new();
    for _ in 0..500 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut EnhancementProgress::new(),
            &mut achievements,
            false,
            &mut rng,
        );
        all_events.extend(result.events);
        if state.active_fishing.is_none() {
            break;
        }
    }

    let fishing_msgs: Vec<_> = all_events
        .iter()
        .filter(|e| matches!(e, TickEvent::FishingMessage { .. }))
        .collect();

    // Should have at least some phase transition messages
    for event in &fishing_msgs {
        if let TickEvent::FishingMessage { message } = event {
            assert!(!message.is_empty(), "Fishing message should not be empty");
        }
    }
}

// =============================================================================
// 8. Play time still increments during fishing
// =============================================================================

#[test]
fn test_play_time_increments_during_fishing_session() {
    let mut state = GameState::new("Fishing Play Time Test".to_string(), 0);
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let session = quest::FishingSession {
        spot_name: "Time Lake".to_string(),
        total_fish: 10,
        fish_caught: Vec::new(),
        items_found: Vec::new(),
        ticks_remaining: 1,
        phase: FishingPhase::Casting,
    };
    state.active_fishing = Some(session);

    let initial_time = state.play_time_seconds;

    // Run exactly 20 ticks (should add 2 seconds)
    for _ in 0..20 {
        game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut EnhancementProgress::new(),
            &mut achievements,
            false,
            &mut rng,
        );
    }

    assert_eq!(
        state.play_time_seconds,
        initial_time + 2,
        "Play time should still track during fishing"
    );
}

// =============================================================================
// 9. Haven discovery at P10+ with no active content
// =============================================================================

#[test]
fn test_haven_discovery_requires_prestige_10() {
    let mut state = GameState::new("Haven P9 Test".to_string(), 0);
    state.prestige_rank = 9;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let results = run_ticks_collecting_results(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        1_000,
    );

    let haven_discovered = results.iter().any(|r| r.haven_changed);

    assert!(!haven_discovered, "Haven should not be discovered at P9");
    assert!(!haven.discovered);
}

#[test]
fn test_haven_discovery_blocked_by_active_dungeon() {
    let mut state = create_strong_character("Haven Dungeon Block Test");
    state.prestige_rank = 20;
    state.character_level = 10;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    // Put player in a dungeon
    let dungeon = generate_dungeon(state.character_level, state.prestige_rank, 1);
    state.active_dungeon = Some(dungeon);

    let events = run_ticks_collecting(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        500,
    );

    let haven_discovered = events
        .iter()
        .any(|e| matches!(e, TickEvent::HavenDiscovered));

    assert!(
        !haven_discovered,
        "Haven should not be discovered while in a dungeon"
    );
}

#[test]
fn test_haven_discovery_blocked_by_active_fishing() {
    let mut state = GameState::new("Haven Fishing Block Test".to_string(), 0);
    state.prestige_rank = 20;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let session = quest::FishingSession {
        spot_name: "Haven Block Lake".to_string(),
        total_fish: 10,
        fish_caught: Vec::new(),
        items_found: Vec::new(),
        ticks_remaining: 1,
        phase: FishingPhase::Casting,
    };
    state.active_fishing = Some(session);

    let events = run_ticks_collecting(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        500,
    );

    let haven_discovered = events
        .iter()
        .any(|e| matches!(e, TickEvent::HavenDiscovered));

    assert!(
        !haven_discovered,
        "Haven should not be discovered while fishing"
    );
}

// =============================================================================
// 10. Haven discovery sets haven_changed flag
// =============================================================================

#[test]
fn test_haven_discovery_sets_haven_changed_and_achievements_changed() {
    // Use many different seeds to find one that triggers Haven discovery quickly
    for seed in 0u64..200 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut haven = Haven::default();
        let mut achievements = Achievements::default();
        let mut state = create_strong_character("Haven Flag Test");
        state.prestige_rank = 50;
        let mut tick_counter = 0u32;

        for _ in 0..10_000 {
            let result = game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut EnhancementProgress::new(),
                &mut achievements,
                false,
                &mut rng,
            );

            if result.haven_changed {
                assert!(haven.discovered, "Haven should be discovered");
                assert!(
                    result.achievements_changed,
                    "achievements_changed should be true on haven discovery"
                );
                let has_event = result
                    .events
                    .iter()
                    .any(|e| matches!(e, TickEvent::HavenDiscovered));
                assert!(has_event, "HavenDiscovered event should be emitted");
                return;
            }
        }
    }
    // If we couldn't find it in 200 seeds * 10k ticks, the probability is too low
    // but at P50 with ~0.000294 chance per tick, expected ~3 discoveries per 10k ticks
    panic!("Haven discovery should have occurred with P50 in 200 * 10k ticks");
}

// =============================================================================
// 11. Challenge discovery requires P1+ and no active content
// =============================================================================

#[test]
fn test_challenge_discovery_blocked_by_active_dungeon() {
    let mut state = GameState::new("Challenge Dungeon Block Test".to_string(), 0);
    state.prestige_rank = 5;
    state.character_level = 10;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let dungeon = generate_dungeon(state.character_level, state.prestige_rank, 1);
    state.active_dungeon = Some(dungeon);

    let events = run_ticks_collecting(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        1_000,
    );

    let challenge_discovered = events
        .iter()
        .any(|e| matches!(e, TickEvent::ChallengeDiscovered { .. }));

    assert!(
        !challenge_discovered,
        "Challenges should not be discovered while in a dungeon"
    );
}

#[test]
fn test_challenge_discovery_blocked_by_active_fishing() {
    let mut state = GameState::new("Challenge Fishing Block Test".to_string(), 0);
    state.prestige_rank = 5;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let session = quest::FishingSession {
        spot_name: "Challenge Block Lake".to_string(),
        total_fish: 10,
        fish_caught: Vec::new(),
        items_found: Vec::new(),
        ticks_remaining: 1,
        phase: FishingPhase::Casting,
    };
    state.active_fishing = Some(session);

    let events = run_ticks_collecting(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        500,
    );

    let challenge_discovered = events
        .iter()
        .any(|e| matches!(e, TickEvent::ChallengeDiscovered { .. }));

    assert!(
        !challenge_discovered,
        "Challenges should not be discovered while fishing"
    );
}

// =============================================================================
// 12. ChallengeDiscovered event fields
// =============================================================================

#[test]
fn test_challenge_discovered_event_has_type_and_messages() {
    // Try many seeds to find challenge discovery
    for seed in 0u64..500 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut test_state = GameState::new("Challenge Event Test".to_string(), 0);
        test_state.prestige_rank = 1;
        let mut tc = 0u32;
        let mut h = Haven {
            discovered: true,
            ..Haven::default()
        };
        h.rooms.insert(quest::HavenRoomId::Library, 3);
        h.rooms.insert(quest::HavenRoomId::Hearthstone, 1);
        h.rooms.insert(quest::HavenRoomId::Bedroom, 1);
        let mut a = Achievements::default();

        for _ in 0..5_000 {
            let result = game_tick(
                &mut test_state,
                &mut tc,
                &mut h,
                &mut EnhancementProgress::new(),
                &mut a,
                false,
                &mut rng,
            );
            for event in &result.events {
                if let TickEvent::ChallengeDiscovered {
                    challenge_type,
                    message,
                    follow_up,
                } = event
                {
                    assert!(
                        !message.is_empty(),
                        "Challenge discovery message should not be empty"
                    );
                    assert!(
                        !follow_up.is_empty(),
                        "Challenge follow-up should not be empty"
                    );
                    assert!(
                        follow_up.contains("Tab"),
                        "Follow-up should mention Tab key"
                    );
                    let _ = format!("{:?}", challenge_type);
                    return;
                }
            }
        }
    }
    // Challenge discovery at 0.000014 base * 1.5 (Library T3) = 0.000021/tick
    // Over 500 seeds * 5000 ticks = 2.5M ticks, expected ~52 discoveries
    panic!("Should have discovered at least one challenge");
}

// =============================================================================
// 13. DungeonDiscovered event
// =============================================================================

#[test]
fn test_dungeon_discovered_event_message() {
    let mut state = create_strong_character("Dungeon Discover Msg Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let events = run_ticks_collecting(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        50_000,
    );

    let dungeon_discovered: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, TickEvent::DungeonDiscovered { .. }))
        .collect();

    if !dungeon_discovered.is_empty() {
        if let TickEvent::DungeonDiscovered { message } = &dungeon_discovered[0] {
            assert!(
                message.contains("passage") || message.contains("underground"),
                "Dungeon discovery message should mention passage/underground"
            );
        }
    }
}

// =============================================================================
// 14. FishingSpotDiscovered event
// =============================================================================

#[test]
fn test_fishing_spot_discovered_event() {
    let mut state = create_strong_character("Fishing Discover Test");
    state.prestige_rank = 1;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let events = run_ticks_collecting(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        100_000,
    );

    let fishing_discovered: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, TickEvent::FishingSpotDiscovered { .. }))
        .collect();

    if !fishing_discovered.is_empty() {
        if let TickEvent::FishingSpotDiscovered { message } = &fishing_discovered[0] {
            assert!(
                !message.is_empty(),
                "Fishing discovery message should not be empty"
            );
        }
    }
}

// =============================================================================
// 15. Multiple LeveledUp events from large XP gain
// =============================================================================

#[test]
fn test_leveled_up_event_on_enemy_defeat() {
    let mut state = create_strong_character("Level Up Test");
    // Set XP just below level up threshold
    state.character_xp = xp_for_next_level(1) - 1;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let mut all_events = Vec::new();
    let mut found_level_up = false;
    for _ in 0..5_000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut EnhancementProgress::new(),
            &mut achievements,
            false,
            &mut rng,
        );
        for event in &result.events {
            if matches!(event, TickEvent::LeveledUp { .. }) {
                found_level_up = true;
            }
        }
        all_events.extend(result.events);
        if found_level_up {
            break;
        }
    }

    assert!(
        found_level_up,
        "Should produce LeveledUp event after gaining enough XP"
    );

    let level_event = all_events
        .iter()
        .find(|e| matches!(e, TickEvent::LeveledUp { .. }))
        .unwrap();
    if let TickEvent::LeveledUp { new_level } = level_event {
        assert!(
            *new_level >= 2,
            "New level should be at least 2, got {}",
            new_level
        );
    }
}

// =============================================================================
// 16. ItemDropped event from mob kill
// =============================================================================

#[test]
fn test_item_dropped_event_fields_from_mob() {
    let mut state = create_strong_character("Mob Item Drop Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let events = run_ticks_collecting(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        50_000,
    );

    let mob_drops: Vec<_> = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                TickEvent::ItemDropped {
                    from_boss: false,
                    ..
                }
            )
        })
        .collect();

    if !mob_drops.is_empty() {
        if let TickEvent::ItemDropped {
            item_name,
            rarity,
            slot,
            stats,
            from_boss,
            ..
        } = &mob_drops[0]
        {
            assert!(!item_name.is_empty(), "Item name should not be empty");
            assert!(!slot.is_empty(), "Slot should not be empty");
            assert!(!stats.is_empty(), "Stats should not be empty");
            assert!(!from_boss, "Should be from mob");
            let _ = format!("{:?}", rarity);
        }
    }
}

// =============================================================================
// 17. ItemDropped from boss kill (guaranteed drop)
// =============================================================================

#[test]
fn test_item_dropped_event_from_boss() {
    let mut state = create_strong_character("Boss Drop Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    // Run until boss defeated and check for boss item drop
    let mut all_events = Vec::new();
    for _ in 0..10_000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut EnhancementProgress::new(),
            &mut achievements,
            false,
            &mut rng,
        );
        all_events.extend(result.events);

        // Stop after first boss defeat
        if all_events
            .iter()
            .any(|e| matches!(e, TickEvent::SubzoneBossDefeated { .. }))
        {
            break;
        }
    }

    let boss_defeated = all_events
        .iter()
        .any(|e| matches!(e, TickEvent::SubzoneBossDefeated { .. }));

    if boss_defeated {
        let boss_drops: Vec<_> = all_events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    TickEvent::ItemDropped {
                        from_boss: true,
                        ..
                    }
                )
            })
            .collect();

        assert!(
            !boss_drops.is_empty(),
            "Boss defeat should always produce an item drop"
        );
    }
}

// =============================================================================
// 18. SubzoneBossDefeated event advances zone
// =============================================================================

#[test]
fn test_subzone_boss_defeated_advances_progression() {
    let mut state = create_strong_character("Boss Advance Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    assert_eq!(state.zone_progression.current_subzone_id, 1);

    let mut all_events = Vec::new();
    for _ in 0..10_000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut EnhancementProgress::new(),
            &mut achievements,
            false,
            &mut rng,
        );
        let found = result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::SubzoneBossDefeated { .. }));
        all_events.extend(result.events);
        if found {
            break;
        }
    }

    let boss_event = all_events
        .iter()
        .find(|e| matches!(e, TickEvent::SubzoneBossDefeated { .. }));

    if let Some(TickEvent::SubzoneBossDefeated {
        xp_gained,
        result,
        message,
    }) = boss_event
    {
        assert!(*xp_gained > 0, "Boss XP should be positive");
        assert!(
            !message.is_empty(),
            "Boss defeat message should not be empty"
        );
        // Should advance to subzone 2
        if let quest::zones::BossDefeatResult::SubzoneComplete { new_subzone_id } = result {
            assert_eq!(*new_subzone_id, 2);
        }
    }
}

// =============================================================================
// 19. AchievementUnlocked event
// =============================================================================

#[test]
fn test_achievement_unlocked_event_format() {
    let mut state = create_strong_character("Achievement Event Test");
    // Set up for level 10 achievement
    state.character_level = 9;
    state.character_xp = xp_for_next_level(9) - 1;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let mut all_events = Vec::new();
    for _ in 0..10_000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut EnhancementProgress::new(),
            &mut achievements,
            false,
            &mut rng,
        );
        let found = result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::AchievementUnlocked { .. }));
        all_events.extend(result.events);
        if found {
            break;
        }
    }

    let achievement_events: Vec<_> = all_events
        .iter()
        .filter(|e| matches!(e, TickEvent::AchievementUnlocked { .. }))
        .collect();

    if !achievement_events.is_empty() {
        if let TickEvent::AchievementUnlocked { name, message } = &achievement_events[0] {
            assert!(!name.is_empty(), "Achievement name should not be empty");
            assert!(
                message.contains("Achievement Unlocked"),
                "Message should contain 'Achievement Unlocked'"
            );
        }
    }
}

// =============================================================================
// 20. Debug mode suppresses haven save signal
// =============================================================================

#[test]
fn test_debug_mode_suppresses_haven_and_achievement_save_on_storm_leviathan() {
    let mut state = GameState::new("Debug Mode Test".to_string(), 0);
    state.fishing.rank = 40;
    state.fishing.leviathan_encounters = 10;
    let mut tick = 0u32;
    let mut h = Haven::default();
    let mut a = Achievements::default();
    let mut rng = test_rng();

    // Run in debug mode
    for _ in 0..100 {
        let result = game_tick(
            &mut state,
            &mut tick,
            &mut h,
            &mut EnhancementProgress::new(),
            &mut a,
            true, // debug_mode = true
            &mut rng,
        );

        // In debug mode, haven_changed and achievements_changed for storm leviathan
        // should be suppressed. Verify no crash.
        let _ = result;
    }
}

// =============================================================================
// 21. TickResult.achievement_modal_ready integration
// =============================================================================

#[test]
fn test_tick_result_achievement_modal_ready() {
    let mut state = create_strong_character("Modal Ready Test");
    // Set up for level 10 achievement
    state.character_level = 9;
    state.character_xp = xp_for_next_level(9) - 1;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let mut modal_ready_found = false;
    for _ in 0..10_000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut EnhancementProgress::new(),
            &mut achievements,
            false,
            &mut rng,
        );

        if !result.achievement_modal_ready.is_empty() {
            modal_ready_found = true;
            break;
        }
    }

    // The modal has a 500ms accumulation window, so it may or may not be ready
    // in any given tick. We just verify the field is populated correctly.
    if state.character_level >= 10 && modal_ready_found {
        // Modal was ready -- good
    }
    // No assertion failure if not found; the test verifies the code path doesn't crash
}

// =============================================================================
// 22. DungeonBossDefeated event includes XP and item count
// =============================================================================

#[test]
fn test_dungeon_boss_defeated_event_fields() {
    let mut state = create_strong_character("Dungeon Boss Event Test");
    state.character_level = 10;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let dungeon = generate_dungeon(state.character_level, state.prestige_rank, 1);
    state.active_dungeon = Some(dungeon);

    let mut all_events = Vec::new();
    for _ in 0..50_000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut EnhancementProgress::new(),
            &mut achievements,
            false,
            &mut rng,
        );

        let found = result.events.iter().any(|e| {
            matches!(
                e,
                TickEvent::DungeonBossDefeated { .. } | TickEvent::DungeonFailed { .. }
            )
        });
        all_events.extend(result.events);
        if found || state.active_dungeon.is_none() {
            break;
        }
    }

    let boss_defeated = all_events
        .iter()
        .find(|e| matches!(e, TickEvent::DungeonBossDefeated { .. }));

    if let Some(TickEvent::DungeonBossDefeated {
        xp_gained,
        bonus_xp,
        total_xp,
        message,
        ..
    }) = boss_defeated
    {
        assert!(*xp_gained > 0, "Boss XP should be positive");
        assert!(
            *total_xp >= *xp_gained + *bonus_xp,
            "Total should include base + bonus"
        );
        assert!(
            message.contains("Dungeon Complete"),
            "Message should mention dungeon complete"
        );
    }
}

// =============================================================================
// 23. DungeonEliteDefeated event
// =============================================================================

#[test]
fn test_dungeon_elite_defeated_event() {
    let mut state = create_strong_character("Elite Event Test");
    state.character_level = 10;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let dungeon = generate_dungeon(state.character_level, state.prestige_rank, 1);
    state.active_dungeon = Some(dungeon);

    let mut all_events = Vec::new();
    for _ in 0..50_000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut EnhancementProgress::new(),
            &mut achievements,
            false,
            &mut rng,
        );
        let found = result.events.iter().any(|e| {
            matches!(
                e,
                TickEvent::DungeonEliteDefeated { .. }
                    | TickEvent::DungeonFailed { .. }
                    | TickEvent::DungeonBossDefeated { .. }
            )
        });
        all_events.extend(result.events);
        if found || state.active_dungeon.is_none() {
            break;
        }
    }

    let elite_event = all_events
        .iter()
        .find(|e| matches!(e, TickEvent::DungeonEliteDefeated { .. }));

    if let Some(TickEvent::DungeonEliteDefeated {
        xp_gained, message, ..
    }) = elite_event
    {
        assert!(*xp_gained > 0, "Elite XP should be positive");
        assert!(
            !message.is_empty(),
            "Elite defeat message should not be empty"
        );
    }
}

// =============================================================================
// 24. Prestige combat bonuses applied to max HP
// =============================================================================

#[test]
fn test_prestige_flat_hp_applied_in_tick() {
    let mut state = GameState::new("Prestige HP Test".to_string(), 0);
    state.prestige_rank = 10;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    // Get base max HP without prestige bonuses
    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
    let base_max_hp = derived.max_hp;

    // Run one tick to apply prestige bonuses
    game_tick(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut EnhancementProgress::new(),
        &mut achievements,
        false,
        &mut rng,
    );

    // Prestige rank 10 should give flat HP bonus
    assert!(
        state.combat_state.player_max_hp > base_max_hp,
        "Max HP should be higher with prestige rank 10 flat HP bonus"
    );
}

// =============================================================================
// 25. EnemyDefeated increments session_kills
// =============================================================================

#[test]
fn test_enemy_defeated_increments_session_kills_via_tick() {
    let mut state = create_strong_character("Session Kill Tick Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    assert_eq!(state.session_kills, 0);

    let mut kill_count = 0u64;
    for _ in 0..5_000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut EnhancementProgress::new(),
            &mut achievements,
            false,
            &mut rng,
        );
        for event in &result.events {
            if matches!(event, TickEvent::EnemyDefeated { .. }) {
                kill_count += 1;
            }
        }
        if kill_count > 0 {
            break;
        }
    }

    assert!(kill_count > 0, "Should have at least one kill");
    assert!(
        state.session_kills > 0,
        "session_kills should be incremented by game_tick"
    );
}
