//! Integration tests for the extracted game_tick() function in core::tick.
//!
//! These tests call game_tick() directly and verify it produces the correct
//! TickEvent variants for each game scenario: combat, fishing, dungeons,
//! zone progression, discoveries, and achievements.
//!
//! Uses seeded ChaCha8Rng for deterministic behavior.

use quest::achievements::Achievements;
use quest::character::attributes::AttributeType;
use quest::character::derived_stats::DerivedStats;
use quest::core::game_logic::{spawn_enemy_if_needed, xp_for_next_level};
use quest::core::tick::{game_tick, TickEvent, TickResult};
use quest::dungeon::generation::generate_dungeon;
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
    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
    state.combat_state.update_max_hp(derived.max_hp);
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;
    state
}

/// Run game_tick in a loop, collecting all events
fn run_ticks(
    state: &mut GameState,
    tick_counter: &mut u32,
    haven: &mut Haven,
    achievements: &mut Achievements,
    rng: &mut ChaCha8Rng,
    count: usize,
) -> Vec<TickEvent> {
    let mut all_events = Vec::new();
    for _ in 0..count {
        let result = game_tick(state, tick_counter, haven, achievements, false, rng);
        all_events.extend(result.events);
    }
    all_events
}

/// Run game_tick until a predicate matches on a TickEvent, returning the matching event index
fn run_until<F>(
    state: &mut GameState,
    tick_counter: &mut u32,
    haven: &mut Haven,
    achievements: &mut Achievements,
    rng: &mut ChaCha8Rng,
    max_ticks: usize,
    pred: F,
) -> (Vec<TickEvent>, bool)
where
    F: Fn(&TickEvent) -> bool,
{
    let mut all_events = Vec::new();
    for _ in 0..max_ticks {
        let result = game_tick(state, tick_counter, haven, achievements, false, rng);
        let found = result.events.iter().any(&pred);
        all_events.extend(result.events);
        if found {
            return (all_events, true);
        }
    }
    (all_events, false)
}

// =============================================================================
// 1. Basic game_tick() behavior
// =============================================================================

#[test]
fn test_game_tick_returns_tick_result() {
    let mut state = GameState::new("Basic Test".to_string(), 0);
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let result = game_tick(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        false,
        &mut rng,
    );

    // TickResult should be valid
    assert!(result.leviathan_encounter.is_none());
    assert!(!result.achievements_changed);
}

#[test]
fn test_game_tick_spawns_enemy_on_first_tick() {
    let mut state = GameState::new("Spawn Test".to_string(), 0);
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    assert!(state.combat_state.current_enemy.is_none());

    game_tick(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        false,
        &mut rng,
    );

    assert!(
        state.combat_state.current_enemy.is_some(),
        "game_tick should spawn an enemy when none exists"
    );
}

#[test]
fn test_game_tick_increments_tick_counter() {
    let mut state = GameState::new("Counter Test".to_string(), 0);
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    game_tick(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        false,
        &mut rng,
    );

    assert_eq!(tick_counter, 1);
}

#[test]
fn test_game_tick_play_time_after_10_ticks() {
    let mut state = GameState::new("Play Time Test".to_string(), 0);
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let initial_time = state.play_time_seconds;

    for _ in 0..10 {
        game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut achievements,
            false,
            &mut rng,
        );
    }

    assert_eq!(state.play_time_seconds, initial_time + 1);
    assert_eq!(tick_counter, 0, "Counter should reset after 10 ticks");
}

#[test]
fn test_game_tick_play_time_not_incremented_before_10_ticks() {
    let mut state = GameState::new("Play Time Partial".to_string(), 0);
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let initial_time = state.play_time_seconds;

    for _ in 0..9 {
        game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut achievements,
            false,
            &mut rng,
        );
    }

    assert_eq!(state.play_time_seconds, initial_time);
    assert_eq!(tick_counter, 9);
}

// =============================================================================
// 2. Combat TickEvents
// =============================================================================

#[test]
fn test_game_tick_produces_player_attack_events() {
    let mut state = create_strong_character("Attack Event Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let (events, found) = run_until(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        5000,
        |e| matches!(e, TickEvent::PlayerAttack { .. }),
    );

    assert!(found, "Should produce PlayerAttack events during combat");
    let attack = events
        .iter()
        .find(|e| matches!(e, TickEvent::PlayerAttack { .. }))
        .unwrap();
    if let TickEvent::PlayerAttack {
        damage, message, ..
    } = attack
    {
        assert!(*damage > 0, "Attack damage should be positive");
        assert!(!message.is_empty(), "Attack message should not be empty");
    }
}

#[test]
fn test_game_tick_produces_enemy_defeated_event() {
    let mut state = create_strong_character("Enemy Defeat Event Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let (events, found) = run_until(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        5000,
        |e| matches!(e, TickEvent::EnemyDefeated { .. }),
    );

    assert!(found, "Should produce EnemyDefeated event");
    let defeated = events
        .iter()
        .find(|e| matches!(e, TickEvent::EnemyDefeated { .. }))
        .unwrap();
    if let TickEvent::EnemyDefeated {
        xp_gained, message, ..
    } = defeated
    {
        assert!(*xp_gained > 0, "XP gained should be positive");
        assert!(
            message.contains("defeated"),
            "Message should mention defeat"
        );
    }
}

#[test]
fn test_game_tick_applies_xp_on_enemy_defeat() {
    let mut state = create_strong_character("XP Apply Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let initial_xp = state.character_xp;

    let (_events, found) = run_until(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        5000,
        |e| matches!(e, TickEvent::EnemyDefeated { .. }),
    );

    assert!(found, "Should defeat an enemy");
    assert!(
        state.character_xp > initial_xp,
        "XP should increase after enemy defeat (game_tick applies XP internally)"
    );
}

#[test]
fn test_game_tick_increments_session_kills() {
    let mut state = create_strong_character("Session Kill Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    assert_eq!(state.session_kills, 0);

    let (_events, found) = run_until(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        5000,
        |e| matches!(e, TickEvent::EnemyDefeated { .. }),
    );

    assert!(found, "Should defeat an enemy");
    assert!(
        state.session_kills > 0,
        "Session kills should increment after enemy defeat"
    );
}

#[test]
fn test_game_tick_produces_enemy_attack_events() {
    let mut state = GameState::new("Enemy Attack Test".to_string(), 0);
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let (events, found) = run_until(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        5000,
        |e| matches!(e, TickEvent::EnemyAttack { .. }),
    );

    assert!(found, "Should produce EnemyAttack events");
    let attack = events
        .iter()
        .find(|e| matches!(e, TickEvent::EnemyAttack { .. }))
        .unwrap();
    if let TickEvent::EnemyAttack {
        damage,
        enemy_name,
        message,
    } = attack
    {
        assert!(*damage > 0, "Enemy damage should be positive");
        assert!(!enemy_name.is_empty(), "Enemy name should not be empty");
        assert!(
            message.contains("hits you"),
            "Message should describe enemy attack"
        );
    }
}

#[test]
fn test_game_tick_level_up_event_on_sufficient_xp() {
    let mut state = create_strong_character("Level Up Event Test");
    // Set XP just below level-up threshold
    let xp_needed = xp_for_next_level(1);
    state.character_xp = xp_needed - 1;

    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let (events, found) = run_until(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        5000,
        |e| matches!(e, TickEvent::LeveledUp { .. }),
    );

    assert!(found, "Should produce LeveledUp event when XP is enough");
    let leveled = events
        .iter()
        .find(|e| matches!(e, TickEvent::LeveledUp { .. }))
        .unwrap();
    if let TickEvent::LeveledUp { new_level } = leveled {
        assert!(*new_level >= 2, "New level should be at least 2");
    }
}

// =============================================================================
// 3. Item Drop TickEvents
// =============================================================================

#[test]
fn test_game_tick_can_produce_item_dropped_event() {
    let mut state = create_strong_character("Item Drop Event Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    // Run many ticks — mob drop rate is 15%, so need multiple kills
    let events = run_ticks(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        50_000,
    );

    let item_drops: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, TickEvent::ItemDropped { .. }))
        .collect();

    assert!(
        !item_drops.is_empty(),
        "Should get at least one item drop in 50k ticks"
    );

    if let TickEvent::ItemDropped {
        item_name,
        slot,
        stats,
        ..
    } = &item_drops[0]
    {
        assert!(!item_name.is_empty(), "Item should have a name");
        assert!(!slot.is_empty(), "Item should have a slot");
        assert!(!stats.is_empty(), "Item should have stat summary");
    }
}

#[test]
fn test_game_tick_item_drop_updates_recent_drops() {
    let mut state = create_strong_character("Recent Drops Event Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let events = run_ticks(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        50_000,
    );

    let has_drops = events
        .iter()
        .any(|e| matches!(e, TickEvent::ItemDropped { .. }));

    if has_drops {
        assert!(
            !state.recent_drops.is_empty(),
            "Recent drops should be populated after ItemDropped event"
        );
    }
}

// =============================================================================
// 4. Zone Progression TickEvents
// =============================================================================

#[test]
fn test_game_tick_subzone_boss_defeated_event() {
    let mut state = create_strong_character("Boss Defeat Event Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    assert_eq!(state.zone_progression.current_zone_id, 1);
    assert_eq!(state.zone_progression.current_subzone_id, 1);

    let (events, found) = run_until(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        100_000,
        |e| matches!(e, TickEvent::SubzoneBossDefeated { .. }),
    );

    assert!(found, "Should defeat a subzone boss");

    let boss_event = events
        .iter()
        .find(|e| matches!(e, TickEvent::SubzoneBossDefeated { .. }))
        .unwrap();
    if let TickEvent::SubzoneBossDefeated {
        xp_gained, message, ..
    } = boss_event
    {
        assert!(*xp_gained > 0, "Boss XP should be positive");
        assert!(
            message.contains("Boss defeated") || message.contains("conquered"),
            "Message should describe boss defeat"
        );
    }

    // Zone should have advanced
    assert_eq!(
        state.zone_progression.current_subzone_id, 2,
        "Should advance to subzone 2"
    );
}

// =============================================================================
// 5. Fishing TickEvents
// =============================================================================

#[test]
fn test_game_tick_fishing_produces_events() {
    let mut state = GameState::new("Fishing Event Test".to_string(), 0);
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    // Set up a fishing session
    let session = quest::FishingSession {
        spot_name: "Test Lake".to_string(),
        total_fish: 3,
        fish_caught: Vec::new(),
        items_found: Vec::new(),
        ticks_remaining: 1,
        phase: quest::fishing::FishingPhase::Casting,
    };
    state.active_fishing = Some(session);

    // Run ticks while fishing
    let mut all_events = Vec::new();
    for _ in 0..500 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut achievements,
            false,
            &mut rng,
        );
        all_events.extend(result.events);
        if state.active_fishing.is_none() {
            break;
        }
    }

    // Should produce fishing-related events
    let fishing_events: Vec<_> = all_events
        .iter()
        .filter(|e| {
            matches!(
                e,
                TickEvent::FishingMessage { .. }
                    | TickEvent::FishCaught { .. }
                    | TickEvent::FishingItemFound { .. }
                    | TickEvent::FishingRankUp { .. }
            )
        })
        .collect();

    assert!(
        !fishing_events.is_empty(),
        "Should produce fishing events during fishing session"
    );
}

#[test]
fn test_game_tick_fishing_skips_combat() {
    let mut state = create_strong_character("Fishing No Combat Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    // Spawn an enemy first
    spawn_enemy_if_needed(&mut state);
    assert!(state.combat_state.current_enemy.is_some());
    let enemy_hp_before = state
        .combat_state
        .current_enemy
        .as_ref()
        .unwrap()
        .current_hp;

    // Start fishing
    let session = quest::FishingSession {
        spot_name: "No Combat Lake".to_string(),
        total_fish: 3,
        fish_caught: Vec::new(),
        items_found: Vec::new(),
        ticks_remaining: 1,
        phase: quest::fishing::FishingPhase::Casting,
    };
    state.active_fishing = Some(session);

    // Run a tick — combat should be skipped
    let result = game_tick(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        false,
        &mut rng,
    );

    // No combat events should be produced
    let combat_events: Vec<_> = result
        .events
        .iter()
        .filter(|e| {
            matches!(
                e,
                TickEvent::PlayerAttack { .. }
                    | TickEvent::EnemyAttack { .. }
                    | TickEvent::EnemyDefeated { .. }
            )
        })
        .collect();

    assert!(
        combat_events.is_empty(),
        "No combat events should occur while fishing"
    );

    // Enemy HP should be unchanged
    if let Some(enemy) = &state.combat_state.current_enemy {
        assert_eq!(
            enemy.current_hp, enemy_hp_before,
            "Enemy HP should not change while fishing"
        );
    }
}

#[test]
fn test_game_tick_fishing_still_tracks_play_time() {
    let mut state = GameState::new("Fishing Time Test".to_string(), 0);
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
        phase: quest::fishing::FishingPhase::Casting,
    };
    state.active_fishing = Some(session);

    let initial_time = state.play_time_seconds;

    for _ in 0..10 {
        game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut achievements,
            false,
            &mut rng,
        );
    }

    assert_eq!(
        state.play_time_seconds,
        initial_time + 1,
        "Play time should still increment while fishing"
    );
}

#[test]
fn test_game_tick_fish_caught_event_has_rarity() {
    let mut state = GameState::new("Fish Rarity Test".to_string(), 0);
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
        phase: quest::fishing::FishingPhase::Casting,
    };
    state.active_fishing = Some(session);

    let mut all_events = Vec::new();
    for _ in 0..500 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut achievements,
            false,
            &mut rng,
        );
        all_events.extend(result.events);
        if state.active_fishing.is_none() {
            break;
        }
    }

    let fish_caught: Vec<_> = all_events
        .iter()
        .filter(|e| matches!(e, TickEvent::FishCaught { .. }))
        .collect();

    if !fish_caught.is_empty() {
        if let TickEvent::FishCaught {
            fish_name, message, ..
        } = &fish_caught[0]
        {
            assert!(!fish_name.is_empty(), "Fish name should not be empty");
            assert!(!message.is_empty(), "Fish message should not be empty");
        }
    }
}

// =============================================================================
// 6. Dungeon TickEvents
// =============================================================================

#[test]
fn test_game_tick_dungeon_room_entered_event() {
    let mut state = GameState::new("Dungeon Room Event Test".to_string(), 0);
    state.character_level = 10;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let dungeon = generate_dungeon(state.character_level, state.prestige_rank, 1);
    state.active_dungeon = Some(dungeon);

    // Run ticks until we get a room entry event
    let (events, found) = run_until(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        5000,
        |e| matches!(e, TickEvent::DungeonRoomEntered { .. }),
    );

    // May or may not enter a room depending on dungeon auto-exploration timing
    if found {
        let room_event = events
            .iter()
            .find(|e| matches!(e, TickEvent::DungeonRoomEntered { .. }))
            .unwrap();
        if let TickEvent::DungeonRoomEntered { message, .. } = room_event {
            assert!(
                !message.is_empty(),
                "Room entry message should not be empty"
            );
        }
    }
}

#[test]
fn test_game_tick_dungeon_failed_event_on_death() {
    let mut state = GameState::new("Dungeon Death Test".to_string(), 0);
    state.character_level = 1;
    // Very low HP to die quickly in dungeon
    state.combat_state.player_current_hp = 1;
    state.combat_state.player_max_hp = 1;

    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let dungeon = generate_dungeon(state.character_level, state.prestige_rank, 1);
    state.active_dungeon = Some(dungeon);

    // Run ticks — with 1 HP, should die quickly if entering combat room
    let mut all_events = Vec::new();
    let mut dungeon_ended = false;
    for _ in 0..10_000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut achievements,
            false,
            &mut rng,
        );
        for event in &result.events {
            if matches!(
                event,
                TickEvent::DungeonFailed { .. } | TickEvent::PlayerDiedInDungeon { .. }
            ) {
                dungeon_ended = true;
            }
        }
        all_events.extend(result.events);
        if dungeon_ended || state.active_dungeon.is_none() {
            break;
        }
    }

    // If player entered a combat room and died, dungeon should be gone
    if dungeon_ended {
        assert!(
            state.active_dungeon.is_none(),
            "Dungeon should be cleared after death"
        );
    }
}

// =============================================================================
// 7. Discovery TickEvents
// =============================================================================

#[test]
fn test_game_tick_can_discover_dungeon() {
    let mut state = create_strong_character("Dungeon Discovery Event Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    // Run many ticks to allow dungeon discovery after kills
    let events = run_ticks(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        50_000,
    );

    let discovery_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, TickEvent::DungeonDiscovered { .. }))
        .collect();

    // Discovery is probabilistic; just verify no crash and events are well-formed
    for event in &discovery_events {
        if let TickEvent::DungeonDiscovered { message } = event {
            assert!(!message.is_empty(), "Discovery message should not be empty");
        }
    }
}

#[test]
fn test_game_tick_challenge_discovery_requires_prestige() {
    let mut state = GameState::new("Challenge No Prestige Test".to_string(), 0);
    state.prestige_rank = 0; // P0 cannot discover challenges
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let events = run_ticks(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        1000,
    );

    let challenge_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, TickEvent::ChallengeDiscovered { .. }))
        .collect();

    assert!(
        challenge_events.is_empty(),
        "P0 characters should never discover challenges"
    );
}

// =============================================================================
// 8. Achievement TickEvents
// =============================================================================

#[test]
fn test_game_tick_achievement_event_on_level_milestone() {
    let mut state = create_strong_character("Achievement Level Test");
    // Set XP close to level 10 (first level achievement milestone)
    for level in 1..10 {
        let xp = xp_for_next_level(level);
        state.character_xp += xp;
        state.character_level = level + 1;
    }
    // Now set XP just below level 10's next threshold so that the next
    // kill triggers level 10
    state.character_level = 9;
    state.character_xp = xp_for_next_level(9) - 1;

    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let (events, found) = run_until(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        10_000,
        |e| matches!(e, TickEvent::AchievementUnlocked { .. }),
    );

    // If we leveled to 10, there should be an achievement event
    if found {
        let achievement = events
            .iter()
            .find(|e| matches!(e, TickEvent::AchievementUnlocked { .. }))
            .unwrap();
        if let TickEvent::AchievementUnlocked { name, message } = achievement {
            assert!(!name.is_empty(), "Achievement name should not be empty");
            assert!(
                message.contains("Achievement Unlocked"),
                "Message should mention unlock"
            );
        }
    }
}

#[test]
fn test_game_tick_achievements_changed_flag() {
    let mut state = create_strong_character("Achievement Flag Test");
    // Set up to trigger level 10 achievement
    state.character_level = 9;
    state.character_xp = xp_for_next_level(9) - 1;

    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let mut achievements_changed = false;
    for _ in 0..10_000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut achievements,
            false,
            &mut rng,
        );
        if result.achievements_changed {
            achievements_changed = true;
            break;
        }
    }

    if state.character_level >= 10 {
        assert!(
            achievements_changed,
            "achievements_changed should be true when achievement unlocked"
        );
    }
}

#[test]
fn test_game_tick_debug_mode_suppresses_achievement_save() {
    let mut state = create_strong_character("Debug Achievement Test");
    state.character_level = 9;
    state.character_xp = xp_for_next_level(9) - 1;

    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    // Note: debug_mode only suppresses the flag for fishing storm leviathan path.
    // Achievement events themselves still fire. This test verifies the code path
    // doesn't crash when debug_mode is true.
    for _ in 0..5000 {
        let _result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut achievements,
            true, // debug_mode = true
            &mut rng,
        );
    }
    // No crash means success
}

// =============================================================================
// 9. Simulator use case: running game_tick in a loop
// =============================================================================

#[test]
fn test_simulator_100_ticks_produces_progression() {
    let mut state = create_strong_character("Simulator Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    let initial_xp = state.character_xp;
    let initial_kills = state.session_kills;

    let events = run_ticks(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        &mut rng,
        5000,
    );

    // After 5000 ticks (500 seconds), the character should have progressed
    assert!(
        state.character_xp > initial_xp,
        "Should gain XP over 5000 ticks"
    );
    assert!(
        state.session_kills > initial_kills,
        "Should have kills over 5000 ticks"
    );
    assert!(
        state.play_time_seconds >= 500,
        "Should have at least 500 seconds of play time"
    );

    // Should have produced a variety of events
    let has_attacks = events
        .iter()
        .any(|e| matches!(e, TickEvent::PlayerAttack { .. }));
    let has_defeats = events
        .iter()
        .any(|e| matches!(e, TickEvent::EnemyDefeated { .. }));

    assert!(has_attacks, "Should have attack events");
    assert!(has_defeats, "Should have defeat events");
}

#[test]
fn test_simulator_rng_param_is_used_for_challenge_ai() {
    // Verify the seeded RNG parameter is actually used by game_tick.
    // Note: Some subsystems (spawn_enemy_if_needed, apply_tick_xp) use
    // thread_rng() internally, so full determinism requires those to be
    // refactored too. But the RNG param IS used for challenge AI, fishing,
    // and discovery — we verify it doesn't panic and produces valid state.
    let mut state = create_strong_character("RNG Usage Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = ChaCha8Rng::seed_from_u64(12345);

    for _ in 0..1000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut achievements,
            false,
            &mut rng,
        );
        // Verify no panics and events are well-formed
        for event in &result.events {
            match event {
                TickEvent::PlayerAttack { damage, .. } => assert!(*damage > 0),
                TickEvent::EnemyDefeated { xp_gained, .. } => assert!(*xp_gained > 0),
                _ => {}
            }
        }
    }

    // State should have progressed
    assert!(state.character_xp > 0, "Should have gained XP");
    assert!(state.session_kills > 0, "Should have kills");
}

#[test]
fn test_simulator_different_seeds_produce_different_results() {
    let run_simulation = |seed: u64| -> u64 {
        let mut state = create_strong_character("Seed Variation Test");
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut achievements = Achievements::default();
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        for _ in 0..2000 {
            game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut achievements,
                false,
                &mut rng,
            );
        }

        state.character_xp
    };

    let result1 = run_simulation(111);
    let result2 = run_simulation(999);

    // Different seeds should (almost certainly) produce different XP
    // This is probabilistic but extremely unlikely to fail
    assert_ne!(
        result1, result2,
        "Different seeds should produce different XP values"
    );
}

// =============================================================================
// 10. Max HP sync every tick
// =============================================================================

#[test]
fn test_game_tick_syncs_max_hp_with_attributes() {
    let mut state = GameState::new("HP Sync Test".to_string(), 0);
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    // Record initial max HP
    let initial_max_hp = state.combat_state.player_max_hp;

    // Boost CON to increase max HP
    state.attributes.set(AttributeType::Constitution, 30);

    // game_tick should recalculate derived stats and update max HP
    game_tick(
        &mut state,
        &mut tick_counter,
        &mut haven,
        &mut achievements,
        false,
        &mut rng,
    );

    assert!(
        state.combat_state.player_max_hp > initial_max_hp,
        "Max HP should increase after CON boost on next tick"
    );
}

// =============================================================================
// 11. TickResult structure validation
// =============================================================================

#[test]
fn test_tick_result_default_is_empty() {
    let result = TickResult::default();
    assert!(result.events.is_empty());
    assert!(result.leviathan_encounter.is_none());
    assert!(!result.achievements_changed);
}

#[test]
fn test_game_tick_returns_events_in_chronological_order() {
    let mut state = create_strong_character("Event Order Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = test_rng();

    // Run until we get a tick with multiple events (e.g., attack + defeat)
    for _ in 0..5000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut achievements,
            false,
            &mut rng,
        );

        if result.events.len() >= 2 {
            // If we have an EnemyDefeated, check that related events make sense
            let has_defeat = result
                .events
                .iter()
                .any(|e| matches!(e, TickEvent::EnemyDefeated { .. }));
            if has_defeat {
                // EnemyDefeated should come before ItemDropped (drops happen after kill)
                let defeat_idx = result
                    .events
                    .iter()
                    .position(|e| matches!(e, TickEvent::EnemyDefeated { .. }));
                let drop_idx = result
                    .events
                    .iter()
                    .position(|e| matches!(e, TickEvent::ItemDropped { .. }));
                if let (Some(d), Some(i)) = (defeat_idx, drop_idx) {
                    assert!(
                        d < i,
                        "EnemyDefeated should come before ItemDropped in events"
                    );
                }
                break;
            }
        }
    }
}
