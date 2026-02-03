//! Integration test: Complete dungeon run
//!
//! Tests the full dungeon flow: enter → explore → elite (key) → boss → rewards

use quest::dungeon::{Dungeon, DungeonSize, RoomState, RoomType};
use quest::dungeon_generation::generate_dungeon;
use quest::dungeon_logic::{
    add_dungeon_xp, collect_dungeon_item, find_next_room, get_enemy_stat_multiplier,
    on_boss_defeated, on_elite_defeated, on_player_died_in_dungeon, on_room_enemy_defeated,
    on_treasure_room_entered, update_dungeon, DungeonEvent, ROOM_MOVE_INTERVAL,
};
use quest::game_state::GameState;
use quest::item_generation::generate_item;
use quest::items::{EquipmentSlot, Rarity};

/// Helper to find a room of a specific type in the dungeon
fn find_room_of_type(dungeon: &Dungeon, room_type: RoomType) -> Option<(usize, usize)> {
    let grid_size = dungeon.size.grid_size();
    for y in 0..grid_size {
        for x in 0..grid_size {
            if let Some(room) = dungeon.get_room(x, y) {
                if room.room_type == room_type {
                    return Some((x, y));
                }
            }
        }
    }
    None
}

/// Helper to count rooms of a specific type
fn count_rooms_of_type(dungeon: &Dungeon, room_type: RoomType) -> usize {
    let grid_size = dungeon.size.grid_size();
    let mut count = 0;
    for y in 0..grid_size {
        for x in 0..grid_size {
            if let Some(room) = dungeon.get_room(x, y) {
                if room.room_type == room_type {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Test complete dungeon exploration and boss defeat
#[test]
fn test_complete_dungeon_run() {
    let mut state = GameState::new("Dungeon Explorer".to_string(), 0);
    state.character_level = 10;

    // Generate a dungeon
    let dungeon = generate_dungeon(state.character_level, state.prestige_rank);
    state.active_dungeon = Some(dungeon);

    // Verify dungeon structure
    let dungeon = state.active_dungeon.as_ref().unwrap();
    assert!(find_room_of_type(dungeon, RoomType::Entrance).is_some());
    assert!(find_room_of_type(dungeon, RoomType::Elite).is_some());
    assert!(find_room_of_type(dungeon, RoomType::Boss).is_some());

    // Start at entrance
    assert_eq!(
        state.active_dungeon.as_ref().unwrap().player_position,
        state.active_dungeon.as_ref().unwrap().entrance_position
    );

    // Track progress
    let mut rooms_explored = 0;
    let mut combats_fought = 0;
    let mut elite_defeated = false;
    let mut boss_defeated = false;
    let mut xp_earned = 0u64;

    // Simulate dungeon exploration
    let max_iterations = 500; // Prevent infinite loop
    let mut iterations = 0;

    while state.active_dungeon.is_some() && iterations < max_iterations {
        iterations += 1;

        let dungeon = state.active_dungeon.as_mut().unwrap();

        // If current room needs combat, simulate defeating enemy
        if !dungeon.current_room_cleared {
            if let Some(room) = dungeon.current_room() {
                match room.room_type {
                    RoomType::Combat => {
                        combats_fought += 1;
                        let xp = 100; // Simulated XP
                        xp_earned += xp;
                        add_dungeon_xp(&mut state, xp);
                        on_room_enemy_defeated(state.active_dungeon.as_mut().unwrap());
                    }
                    RoomType::Elite => {
                        combats_fought += 1;
                        let xp = 250;
                        xp_earned += xp;
                        add_dungeon_xp(&mut state, xp);

                        let events = on_elite_defeated(state.active_dungeon.as_mut().unwrap());
                        elite_defeated = events.iter().any(|e| matches!(e, DungeonEvent::FoundKey));
                    }
                    RoomType::Boss => {
                        combats_fought += 1;
                        let xp = 500;
                        xp_earned += xp;
                        add_dungeon_xp(&mut state, xp);

                        // Defeat boss and complete dungeon
                        let events = on_boss_defeated(&mut state);
                        boss_defeated = events
                            .iter()
                            .any(|e| matches!(e, DungeonEvent::DungeonComplete { .. }));

                        // Dungeon is now cleared
                        break;
                    }
                    RoomType::Treasure => {
                        // Auto-cleared, collect item
                        let result = on_treasure_room_entered(&mut state);
                        assert!(result.is_some());
                    }
                    RoomType::Entrance => {
                        // No action needed
                    }
                }
            }
            continue;
        }

        // Try to move to next room
        let events = update_dungeon(&mut state, ROOM_MOVE_INTERVAL + 0.1);

        for event in &events {
            if matches!(event, DungeonEvent::EnteredRoom { .. }) {
                rooms_explored += 1;
            }
        }

        // If no movement possible and room is cleared, we might be stuck
        // This shouldn't happen in a well-generated dungeon
        if events.is_empty() {
            if let Some(dungeon) = &state.active_dungeon {
                if dungeon.current_room_cleared {
                    // Check if there's a next room
                    if find_next_room(dungeon).is_none() {
                        // No more rooms to explore - this is expected when dungeon is complete
                        // or we need the key
                        if !dungeon.has_key {
                            // Shouldn't happen - elite should have been found
                            panic!("Stuck without key at iteration {}", iterations);
                        }
                    }
                }
            }
        }
    }

    // Verify completion
    assert!(boss_defeated, "Boss should have been defeated");
    assert!(elite_defeated, "Elite should have been defeated (for key)");
    assert!(
        state.active_dungeon.is_none(),
        "Dungeon should be cleared after boss"
    );
    assert!(
        combats_fought >= 3,
        "Should have fought at least 3 combats (combat + elite + boss)"
    );
    assert!(rooms_explored > 0, "Should have explored rooms");

    println!(
        "Dungeon complete: {} rooms explored, {} combats, {} XP in {} iterations",
        rooms_explored, combats_fought, xp_earned, iterations
    );
}

/// Test dungeon stat multipliers for different room types
#[test]
fn test_dungeon_enemy_stat_multipliers() {
    let mut state = GameState::new("Test Hero".to_string(), 0);
    state.active_dungeon = Some(generate_dungeon(10, 0));

    let dungeon = state.active_dungeon.as_mut().unwrap();

    // Test entrance room (no combat)
    assert_eq!(dungeon.player_position, dungeon.entrance_position);
    let entrance_mult = get_enemy_stat_multiplier(dungeon);
    assert!((entrance_mult - 1.0).abs() < 0.01);

    // Find and test elite room
    if let Some(elite_pos) = find_room_of_type(dungeon, RoomType::Elite) {
        if let Some(room) = dungeon.get_room_mut(elite_pos.0, elite_pos.1) {
            room.state = RoomState::Current;
        }
        dungeon.player_position = elite_pos;

        let elite_mult = get_enemy_stat_multiplier(dungeon);
        assert!(
            (elite_mult - 1.5).abs() < 0.01,
            "Elite should have 1.5x multiplier"
        );
    }

    // Test boss room
    let boss_pos = dungeon.boss_position;
    if let Some(room) = dungeon.get_room_mut(boss_pos.0, boss_pos.1) {
        room.state = RoomState::Current;
    }
    dungeon.player_position = boss_pos;

    let boss_mult = get_enemy_stat_multiplier(dungeon);
    assert!(
        (boss_mult - 2.0).abs() < 0.01,
        "Boss should have 2.0x multiplier"
    );
}

/// Test dungeon key acquisition from elite
#[test]
fn test_elite_grants_key_for_boss() {
    let mut state = GameState::new("Key Seeker".to_string(), 0);
    state.active_dungeon = Some(generate_dungeon(10, 0));

    let dungeon = state.active_dungeon.as_mut().unwrap();

    // Initially no key
    assert!(!dungeon.has_key);

    // Defeat elite
    let events = on_elite_defeated(dungeon);

    // Should have key now
    assert!(dungeon.has_key);
    assert!(events.iter().any(|e| matches!(e, DungeonEvent::FoundKey)));
    assert!(events
        .iter()
        .any(|e| matches!(e, DungeonEvent::BossUnlocked)));

    // Second elite defeat shouldn't give another key event
    let events2 = on_elite_defeated(dungeon);
    assert!(!events2.iter().any(|e| matches!(e, DungeonEvent::FoundKey)));
}

/// Test dungeon death exits without prestige loss
#[test]
fn test_dungeon_death_safe_exit() {
    let mut state = GameState::new("Doomed Hero".to_string(), 0);
    state.prestige_rank = 5; // Should be preserved

    state.active_dungeon = Some(generate_dungeon(10, 0));
    assert!(state.active_dungeon.is_some());

    // Simulate death
    let events = on_player_died_in_dungeon(&mut state);

    // Verify safe exit
    assert!(
        state.active_dungeon.is_none(),
        "Dungeon should be cleared on death"
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, DungeonEvent::DungeonFailed)));
    assert_eq!(
        state.prestige_rank, 5,
        "Prestige should be preserved on dungeon death"
    );
}

/// Test dungeon sizes scale with prestige
#[test]
fn test_dungeon_size_scaling() {
    // Dungeon size formula:
    // - Level < 25: base tier 0 (Small)
    // - Level 25-74: base tier 1 (Medium)
    // - Level 75+: base tier 2 (Large)
    // - Prestige adds (rank / 2) tiers
    // - Random variation: ±1 tier

    // Low prestige, low level - expect Small with possible Medium (20% chance)
    let dungeon_p0 = generate_dungeon(10, 0);
    assert!(
        dungeon_p0.size == DungeonSize::Small || dungeon_p0.size == DungeonSize::Medium,
        "Prestige 0, level 10 should get Small or Medium (got {:?})",
        dungeon_p0.size
    );

    // Prestige 5 at level 10: base 0 + 5/2 = tier 2 (Large), ±1 = Medium/Large/Epic
    // Actually generated several dungeons to verify scaling
    let mut found_larger = false;
    for _ in 0..10 {
        let dungeon = generate_dungeon(10, 5);
        if dungeon.size != DungeonSize::Small {
            found_larger = true;
            break;
        }
    }
    assert!(
        found_larger,
        "Prestige 5 should sometimes get larger dungeons than Small"
    );
}

/// Test treasure room item collection
#[test]
fn test_treasure_room_rewards() {
    let mut state = GameState::new("Treasure Hunter".to_string(), 0);
    state.character_level = 20;
    state.prestige_rank = 3;

    state.active_dungeon = Some(generate_dungeon(20, 3));

    // Simulate entering treasure room
    let result = on_treasure_room_entered(&mut state);

    assert!(result.is_some());
    let (item, was_equipped) = result.unwrap();

    // Item should be valid
    assert!(!item.display_name.is_empty());
    assert!(item.attributes.total() > 0);

    // Item should be tracked in dungeon
    let collected = &state.active_dungeon.as_ref().unwrap().collected_items;
    assert_eq!(collected.len(), 1);

    println!(
        "Found treasure: {} ({:?}), equipped: {}",
        item.display_name, item.rarity, was_equipped
    );
}

/// Test dungeon room types are correctly distributed
#[test]
fn test_dungeon_room_distribution() {
    // Generate several dungeons and verify structure
    for _ in 0..10 {
        let dungeon = generate_dungeon(10, 0);

        // Must have exactly one entrance
        assert_eq!(count_rooms_of_type(&dungeon, RoomType::Entrance), 1);

        // Must have exactly one boss
        assert_eq!(count_rooms_of_type(&dungeon, RoomType::Boss), 1);

        // Must have at least one elite (for key)
        assert!(count_rooms_of_type(&dungeon, RoomType::Elite) >= 1);

        // Should have some combat rooms
        assert!(count_rooms_of_type(&dungeon, RoomType::Combat) >= 1);

        // May have treasure rooms
        let treasure_count = count_rooms_of_type(&dungeon, RoomType::Treasure);
        assert!(
            treasure_count <= 5,
            "Shouldn't have too many treasure rooms"
        );
    }
}

/// Test dungeon navigation doesn't get stuck
#[test]
fn test_dungeon_pathfinding_completeness() {
    // Test that dungeon structure allows reaching all rooms
    // Note: pathfinding only works on Revealed/Current/Cleared rooms
    // So we test by revealing all rooms first

    for _ in 0..5 {
        let mut dungeon = generate_dungeon(10, 0);

        // Reveal all rooms for testing reachability
        let grid_size = dungeon.size.grid_size();
        for y in 0..grid_size {
            for x in 0..grid_size {
                if let Some(room) = dungeon.get_room_mut(x, y) {
                    if room.state == RoomState::Hidden {
                        room.state = RoomState::Revealed;
                    }
                }
            }
        }

        // From entrance, should be able to reach elite
        if let Some(elite_pos) = find_room_of_type(&dungeon, RoomType::Elite) {
            let path =
                quest::dungeon_logic::find_path_to(&dungeon, dungeon.entrance_position, elite_pos);
            assert!(
                path.is_some(),
                "Should be able to reach elite from entrance (with all rooms revealed)"
            );
        }

        // From entrance, should be able to reach boss
        let boss_pos = dungeon.boss_position;
        let path =
            quest::dungeon_logic::find_path_to(&dungeon, dungeon.entrance_position, boss_pos);
        assert!(
            path.is_some(),
            "Should be able to reach boss from entrance (with all rooms revealed)"
        );
    }
}

/// Test dungeon XP accumulation
#[test]
fn test_dungeon_xp_tracking() {
    let mut state = GameState::new("XP Tracker".to_string(), 0);
    state.active_dungeon = Some(generate_dungeon(10, 0));

    // Add XP from various sources
    add_dungeon_xp(&mut state, 100);
    add_dungeon_xp(&mut state, 200);
    add_dungeon_xp(&mut state, 150);

    let dungeon = state.active_dungeon.as_ref().unwrap();
    assert_eq!(dungeon.xp_earned, 450);
}

/// Test dungeon item collection tracking
#[test]
fn test_dungeon_item_collection() {
    let mut state = GameState::new("Collector".to_string(), 0);
    state.active_dungeon = Some(generate_dungeon(10, 0));

    // Collect some items
    let item1 = generate_item(EquipmentSlot::Weapon, Rarity::Rare, 10);
    let item2 = generate_item(EquipmentSlot::Armor, Rarity::Magic, 10);

    collect_dungeon_item(&mut state, item1);
    collect_dungeon_item(&mut state, item2);

    let dungeon = state.active_dungeon.as_ref().unwrap();
    assert_eq!(dungeon.collected_items.len(), 2);
}

/// Test boss defeat reports correct stats
#[test]
fn test_boss_defeat_completion_report() {
    let mut state = GameState::new("Boss Slayer".to_string(), 0);
    state.active_dungeon = Some(generate_dungeon(10, 0));

    // Simulate dungeon progress
    add_dungeon_xp(&mut state, 1500);

    let item = generate_item(EquipmentSlot::Weapon, Rarity::Epic, 10);
    collect_dungeon_item(&mut state, item);

    // Defeat boss
    let events = on_boss_defeated(&mut state);

    // Verify completion event
    let completion = events.iter().find_map(|e| match e {
        DungeonEvent::DungeonComplete {
            xp_earned,
            items_collected,
        } => Some((*xp_earned, *items_collected)),
        _ => None,
    });

    assert!(completion.is_some());
    let (xp, items) = completion.unwrap();
    assert_eq!(xp, 1500);
    assert_eq!(items, 1);

    // Dungeon should be cleared
    assert!(state.active_dungeon.is_none());
}
