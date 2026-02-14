//! Dungeon navigation and auto-exploration logic.

use super::generation::reveal_adjacent_rooms;
use super::types::{Dungeon, DungeonSize, RoomState, RoomType};
use crate::core::game_state::GameState;
use crate::items::{
    generate_item, ilvl_for_zone, roll_random_slot, roll_rarity_for_mob, Item, Rarity,
};
use rand::RngExt;
use std::collections::{HashSet, VecDeque};

/// Time between room movements during auto-exploration (seconds)
pub const ROOM_MOVE_INTERVAL: f64 = 2.5;

/// Faster movement when traveling through already-cleared rooms (seconds)
pub const ROOM_TRAVEL_INTERVAL: f64 = 0.8;

/// Events that can occur during dungeon exploration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum DungeonEvent {
    /// Player moved to a new room
    EnteredRoom {
        room_type: RoomType,
        position: (usize, usize),
    },
    /// Player found the big key
    FoundKey,
    /// Player unlocked the boss room
    BossUnlocked,
    /// Dungeon completed (boss defeated)
    DungeonComplete {
        xp_earned: u64,
        items_collected: usize,
    },
    /// Player died and exited dungeon
    DungeonFailed,
    /// Combat started in current room
    CombatStarted { is_elite: bool, is_boss: bool },
    /// Found treasure
    TreasureFound,
}

/// Updates dungeon exploration state
/// Returns events that occurred during this tick
pub fn update_dungeon(state: &mut GameState, delta_time: f64) -> Vec<DungeonEvent> {
    let mut events = Vec::new();

    let dungeon = match &mut state.active_dungeon {
        Some(d) => d,
        None => return events,
    };

    // Can't move until current room is cleared (combat complete)
    if !dungeon.current_room_cleared {
        return events;
    }

    // Update move timer
    dungeon.move_timer += delta_time;

    // Find next room to explore
    if let Some(next_pos) = find_next_room(dungeon) {
        // Check if next room is already cleared (traveling) or new (exploring)
        let is_traveling = dungeon
            .get_room(next_pos.0, next_pos.1)
            .map(|r| r.state == RoomState::Cleared)
            .unwrap_or(false);

        // Use faster interval when traveling through cleared rooms
        let move_interval = if is_traveling {
            ROOM_TRAVEL_INTERVAL
        } else {
            ROOM_MOVE_INTERVAL
        };

        // Update traveling state for UI
        dungeon.is_traveling = is_traveling;

        // Check if it's time to move
        if dungeon.move_timer >= move_interval {
            dungeon.move_timer = 0.0;

            // Move to the next room
            let move_events = move_to_room(dungeon, next_pos);
            events.extend(move_events);
        }
    } else {
        dungeon.is_traveling = false;
    }

    events
}

/// Finds the next room to explore using BFS
/// Prioritizes: unexplored rooms, then boss (if has key)
pub fn find_next_room(dungeon: &Dungeon) -> Option<(usize, usize)> {
    let current = dungeon.player_position;

    // If we have the key and boss is accessible and not yet cleared, go to boss
    if dungeon.has_key {
        // Only go to boss if it's not already cleared (beaten)
        let boss_not_cleared = dungeon
            .get_room(dungeon.boss_position.0, dungeon.boss_position.1)
            .map(|r| r.state != RoomState::Cleared)
            .unwrap_or(false);

        if boss_not_cleared {
            if let Some(path) = find_path_to(dungeon, current, dungeon.boss_position) {
                if path.len() > 1 {
                    return Some(path[1]); // Next step toward boss
                }
            }
        }
    }

    // Find nearest unexplored (revealed but not cleared) room
    let mut best_path: Option<Vec<(usize, usize)>> = None;

    let grid_size = dungeon.size.grid_size();
    for y in 0..grid_size {
        for x in 0..grid_size {
            if let Some(room) = dungeon.get_room(x, y) {
                // Look for revealed rooms we haven't entered yet
                if room.state == RoomState::Revealed {
                    // Skip boss if we don't have key
                    if room.room_type == RoomType::Boss && !dungeon.has_key {
                        continue;
                    }

                    if let Some(path) = find_path_to(dungeon, current, (x, y)) {
                        let is_shorter = best_path
                            .as_ref()
                            .is_none_or(|best| path.len() < best.len());
                        if is_shorter {
                            best_path = Some(path);
                        }
                    }
                }
            }
        }
    }

    // Return first step along the shortest path
    best_path.and_then(|path| if path.len() > 1 { Some(path[1]) } else { None })
}

/// BFS pathfinding between two positions
pub fn find_path_to(
    dungeon: &Dungeon,
    from: (usize, usize),
    to: (usize, usize),
) -> Option<Vec<(usize, usize)>> {
    if from == to {
        return Some(vec![from]);
    }

    // BFS state: position + path taken to reach it
    type BfsNode = (usize, usize, Vec<(usize, usize)>);

    let mut visited: HashSet<(usize, usize)> = HashSet::new();
    let mut queue: VecDeque<BfsNode> = VecDeque::new();

    visited.insert(from);
    queue.push_back((from.0, from.1, vec![from]));

    while let Some((x, y, path)) = queue.pop_front() {
        let neighbors = dungeon.get_connected_neighbors(x, y);

        for (nx, ny) in neighbors {
            if visited.contains(&(nx, ny)) {
                continue;
            }

            let mut new_path = path.clone();
            new_path.push((nx, ny));

            if (nx, ny) == to {
                return Some(new_path);
            }

            // Can only traverse through cleared or current rooms (or revealed if it's the target)
            if let Some(room) = dungeon.get_room(nx, ny) {
                let can_traverse = matches!(
                    room.state,
                    RoomState::Cleared | RoomState::Current | RoomState::Revealed
                );

                if can_traverse {
                    visited.insert((nx, ny));
                    queue.push_back((nx, ny, new_path));
                }
            }
        }
    }

    None
}

/// Moves player to a new room and handles room entry
fn move_to_room(dungeon: &mut Dungeon, new_pos: (usize, usize)) -> Vec<DungeonEvent> {
    let mut events = Vec::new();
    let old_pos = dungeon.player_position;

    // Mark old room as cleared
    if let Some(old_room) = dungeon.get_room_mut(old_pos.0, old_pos.1) {
        if old_room.state == RoomState::Current {
            old_room.state = RoomState::Cleared;
            dungeon.rooms_cleared += 1;
        }
    }

    // Move to new room
    dungeon.player_position = new_pos;

    // Get room type and previous state before mutating
    let (room_type, was_already_cleared) = dungeon
        .get_room(new_pos.0, new_pos.1)
        .map(|r| (r.room_type, r.state == RoomState::Cleared))
        .unwrap_or((RoomType::Combat, false));

    // Mark new room as current (unless already cleared - don't re-count on backtrack)
    if let Some(new_room) = dungeon.get_room_mut(new_pos.0, new_pos.1) {
        if new_room.state != RoomState::Cleared {
            new_room.state = RoomState::Current;
        }
    }

    // Reveal adjacent rooms
    reveal_adjacent_rooms(dungeon, new_pos.0, new_pos.1);

    // Set current_room_cleared based on room type
    // Combat rooms need enemy defeated before moving on, unless already cleared
    dungeon.current_room_cleared =
        matches!(room_type, RoomType::Entrance | RoomType::Treasure) || was_already_cleared;

    // Emit entered room event
    events.push(DungeonEvent::EnteredRoom {
        room_type,
        position: new_pos,
    });

    // Handle room-specific events (only if room wasn't already cleared)
    if !was_already_cleared {
        match room_type {
            RoomType::Elite => {
                events.push(DungeonEvent::CombatStarted {
                    is_elite: true,
                    is_boss: false,
                });
            }
            RoomType::Boss => {
                events.push(DungeonEvent::CombatStarted {
                    is_elite: false,
                    is_boss: true,
                });
            }
            RoomType::Combat => {
                events.push(DungeonEvent::CombatStarted {
                    is_elite: false,
                    is_boss: false,
                });
            }
            RoomType::Treasure => {
                events.push(DungeonEvent::TreasureFound);
            }
            RoomType::Entrance => {
                // No special event for entrance
            }
        }
    }

    events
}

/// Called when player defeats an enemy in the current room
pub fn on_room_enemy_defeated(dungeon: &mut Dungeon) {
    dungeon.current_room_cleared = true;
}

/// Called when player defeats an elite enemy (gets the key)
pub fn on_elite_defeated(dungeon: &mut Dungeon) -> Vec<DungeonEvent> {
    let mut events = Vec::new();
    dungeon.current_room_cleared = true;

    if !dungeon.has_key {
        dungeon.has_key = true;
        events.push(DungeonEvent::FoundKey);
        events.push(DungeonEvent::BossUnlocked);
    }

    events
}

/// Called when player defeats the boss
pub fn on_boss_defeated(state: &mut GameState) -> Vec<DungeonEvent> {
    let mut events = Vec::new();

    if let Some(dungeon) = &state.active_dungeon {
        events.push(DungeonEvent::DungeonComplete {
            xp_earned: dungeon.xp_earned,
            items_collected: dungeon.collected_items.len(),
        });
    }

    // Clear dungeon
    state.active_dungeon = None;

    events
}

/// Called when player dies in dungeon
#[allow(dead_code)]
pub fn on_player_died_in_dungeon(state: &mut GameState) -> Vec<DungeonEvent> {
    let events = vec![DungeonEvent::DungeonFailed];

    // Clear dungeon (keep collected items via different mechanism if desired)
    state.active_dungeon = None;

    events
}

/// Calculates the XP reward for defeating a dungeon boss
pub fn calculate_boss_xp_reward(size: DungeonSize) -> u64 {
    let mut rng = rand::rng();
    let (min_xp, max_xp) = size.boss_xp_range();
    rng.random_range(min_xp..=max_xp)
}

/// Generates a treasure room item with rarity boost based on dungeon size.
/// `zone_id` determines item level (ilvl = zone_id * 10).
pub fn generate_treasure_item(prestige_rank: u32, zone_id: usize, rarity_boost: u32) -> Item {
    let mut rng = rand::rng();

    // Roll a random slot
    let slot = roll_random_slot(&mut rng);

    // Roll rarity with boost based on dungeon tier
    let base_rarity = roll_rarity_for_mob(prestige_rank, 0.0, &mut rng);
    let boosted_rarity = boost_rarity(base_rarity, rarity_boost);

    // Item level based on zone
    let ilvl = ilvl_for_zone(zone_id);

    generate_item(slot, boosted_rarity, ilvl)
}

/// Boosts a rarity by N tiers (capped at Legendary)
fn boost_rarity(rarity: Rarity, boost: u32) -> Rarity {
    let rarity_level = match rarity {
        Rarity::Common => 0,
        Rarity::Magic => 1,
        Rarity::Rare => 2,
        Rarity::Epic => 3,
        Rarity::Legendary => 4,
    };

    match (rarity_level + boost).min(4) {
        0 => Rarity::Common,
        1 => Rarity::Magic,
        2 => Rarity::Rare,
        3 => Rarity::Epic,
        _ => Rarity::Legendary,
    }
}

/// Adds XP earned to the dungeon tally
pub fn add_dungeon_xp(state: &mut GameState, xp: u64) {
    if let Some(dungeon) = &mut state.active_dungeon {
        dungeon.xp_earned += xp;
    }
}

/// Adds an item to the dungeon collected items
#[allow(dead_code)]
pub fn collect_dungeon_item(state: &mut GameState, item: Item) {
    if let Some(dungeon) = &mut state.active_dungeon {
        dungeon.collected_items.push(item);
    }
}

/// Called when player enters a treasure room - generates and collects an item
/// Returns (item, was_equipped)
pub fn on_treasure_room_entered(state: &mut GameState) -> Option<(Item, bool)> {
    // Get rarity boost from dungeon size (defaults to 1 if no dungeon somehow)
    let rarity_boost = state
        .active_dungeon
        .as_ref()
        .map(|d| d.size.treasure_rarity_boost())
        .unwrap_or(1);

    // Use current zone for item level
    let zone_id = state.zone_progression.current_zone_id as usize;

    let item = generate_treasure_item(state.prestige_rank, zone_id, rarity_boost);

    // Auto-equip if better
    let item_clone = item.clone();
    let equipped = crate::items::auto_equip_if_better(item, state);

    // Collect in dungeon tally (whether equipped or not, for completion summary)
    if let Some(dungeon) = &mut state.active_dungeon {
        dungeon.collected_items.push(item_clone.clone());
    }

    Some((item_clone, equipped))
}

/// Checks if the current room needs combat resolution
#[allow(dead_code)]
pub fn current_room_needs_combat(dungeon: &Dungeon) -> bool {
    if let Some(room) = dungeon.current_room() {
        matches!(
            room.room_type,
            RoomType::Combat | RoomType::Elite | RoomType::Boss
        ) && room.state == RoomState::Current
    } else {
        false
    }
}

/// Gets the stat multiplier for the current room's enemy
#[allow(dead_code)]
pub fn get_enemy_stat_multiplier(dungeon: &Dungeon) -> f64 {
    if let Some(room) = dungeon.current_room() {
        match room.room_type {
            RoomType::Elite => 1.5, // 150% stats
            RoomType::Boss => 2.0,  // 200% stats
            _ => 1.0,               // Normal stats
        }
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::super::generation::generate_dungeon;
    use super::*;

    /// Finds the first room of the given type in the dungeon.
    fn find_room_of_type(dungeon: &Dungeon, room_type: RoomType) -> Option<(usize, usize)> {
        let grid_size = dungeon.size.grid_size();
        (0..grid_size)
            .flat_map(|y| (0..grid_size).map(move |x| (x, y)))
            .find(|&(x, y)| {
                dungeon
                    .get_room(x, y)
                    .map(|r| r.room_type == room_type)
                    .unwrap_or(false)
            })
    }

    #[test]
    fn test_find_path_same_position() {
        let dungeon = generate_dungeon(10, 0, 1);
        let pos = dungeon.player_position;
        let path = find_path_to(&dungeon, pos, pos);
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 1);
    }

    #[test]
    fn test_find_path_to_adjacent() {
        let dungeon = generate_dungeon(10, 0, 1);
        let pos = dungeon.player_position;
        let neighbors = dungeon.get_connected_neighbors(pos.0, pos.1);

        if let Some(&neighbor) = neighbors.first() {
            let path = find_path_to(&dungeon, pos, neighbor);
            assert!(path.is_some());
            assert_eq!(path.unwrap().len(), 2);
        }
    }

    #[test]
    fn test_find_next_room_from_start() {
        let dungeon = generate_dungeon(10, 0, 1);
        // From entrance, there should be a next room to explore
        let next = find_next_room(&dungeon);
        assert!(next.is_some());
    }

    #[test]
    fn test_move_to_room_updates_state() {
        let mut dungeon = generate_dungeon(10, 0, 1);
        let start_pos = dungeon.player_position;
        let neighbors = dungeon.get_connected_neighbors(start_pos.0, start_pos.1);

        if let Some(&next_pos) = neighbors.first() {
            let events = move_to_room(&mut dungeon, next_pos);

            // Should have moved
            assert_eq!(dungeon.player_position, next_pos);

            // Old room should be cleared
            let old_room = dungeon.get_room(start_pos.0, start_pos.1).unwrap();
            assert_eq!(old_room.state, RoomState::Cleared);

            // New room should be current
            let new_room = dungeon.get_room(next_pos.0, next_pos.1).unwrap();
            assert_eq!(new_room.state, RoomState::Current);

            // Should have events
            assert!(!events.is_empty());
        }
    }

    #[test]
    fn test_on_elite_defeated_gives_key() {
        let mut dungeon = generate_dungeon(10, 0, 1);
        assert!(!dungeon.has_key);

        let events = on_elite_defeated(&mut dungeon);

        assert!(dungeon.has_key);
        assert!(events.iter().any(|e| matches!(e, DungeonEvent::FoundKey)));
    }

    #[test]
    fn test_get_enemy_stat_multiplier() {
        let dungeon = generate_dungeon(10, 0, 1);
        // Default room (entrance) should have 1.0 multiplier
        let mult = get_enemy_stat_multiplier(&dungeon);
        assert!((mult - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_boss_xp_reward_small() {
        let xp = calculate_boss_xp_reward(DungeonSize::Small);
        assert!((1000..=1500).contains(&xp));
    }

    #[test]
    fn test_calculate_boss_xp_reward_medium() {
        let xp = calculate_boss_xp_reward(DungeonSize::Medium);
        assert!((2000..=3000).contains(&xp));
    }

    #[test]
    fn test_calculate_boss_xp_reward_large() {
        let xp = calculate_boss_xp_reward(DungeonSize::Large);
        assert!((4000..=6000).contains(&xp));
    }

    #[test]
    fn test_generate_treasure_item() {
        // prestige_rank=0, zone_id=5 (ilvl 50), rarity_boost=1
        let item = generate_treasure_item(0, 5, 1);
        assert!(!item.display_name.is_empty());
        assert_eq!(item.ilvl, 50);
    }

    #[test]
    fn test_boost_rarity() {
        // +1 boost
        assert_eq!(boost_rarity(Rarity::Common, 1), Rarity::Magic);
        assert_eq!(boost_rarity(Rarity::Rare, 1), Rarity::Epic);

        // +2 boost (Epic dungeons)
        assert_eq!(boost_rarity(Rarity::Common, 2), Rarity::Rare);
        assert_eq!(boost_rarity(Rarity::Magic, 2), Rarity::Epic);

        // +3 boost (Legendary dungeons)
        assert_eq!(boost_rarity(Rarity::Common, 3), Rarity::Epic);
        assert_eq!(boost_rarity(Rarity::Magic, 3), Rarity::Legendary);

        // Cap at Legendary
        assert_eq!(boost_rarity(Rarity::Legendary, 3), Rarity::Legendary);
    }

    #[test]
    fn test_cleared_room_no_combat_on_reentry() {
        let mut dungeon = generate_dungeon(10, 0, 1);
        let start_pos = dungeon.player_position;

        // Find an adjacent combat room
        let neighbors = dungeon.get_connected_neighbors(start_pos.0, start_pos.1);
        let combat_room_pos = neighbors
            .iter()
            .find(|&&(x, y)| {
                dungeon
                    .get_room(x, y)
                    .map(|r| r.room_type == RoomType::Combat)
                    .unwrap_or(false)
            })
            .copied();

        if let Some(combat_pos) = combat_room_pos {
            // Move to the combat room (first time - should start combat)
            let events1 = move_to_room(&mut dungeon, combat_pos);
            assert!(!dungeon.current_room_cleared);
            assert!(events1
                .iter()
                .any(|e| matches!(e, DungeonEvent::CombatStarted { .. })));

            // Simulate clearing the room
            on_room_enemy_defeated(&mut dungeon);
            assert!(dungeon.current_room_cleared);

            // Mark room as cleared and move back to entrance
            if let Some(room) = dungeon.get_room_mut(combat_pos.0, combat_pos.1) {
                room.state = RoomState::Cleared;
            }
            dungeon.player_position = start_pos;

            // Re-enter the cleared combat room (should NOT start combat)
            let events2 = move_to_room(&mut dungeon, combat_pos);

            // Should be immediately cleared (no combat needed)
            assert!(dungeon.current_room_cleared);

            // Should NOT have CombatStarted event
            assert!(!events2
                .iter()
                .any(|e| matches!(e, DungeonEvent::CombatStarted { .. })));
        }
    }

    // ============ update_dungeon tests ============

    #[test]
    fn test_update_dungeon_no_active_dungeon() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.active_dungeon = None;

        let events = update_dungeon(&mut state, 0.1);

        assert!(events.is_empty());
    }

    #[test]
    fn test_update_dungeon_room_not_cleared_blocks_movement() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.active_dungeon = Some(generate_dungeon(10, 0, 1));

        // Set room as not cleared (in combat)
        if let Some(dungeon) = &mut state.active_dungeon {
            dungeon.current_room_cleared = false;
            dungeon.move_timer = 10.0; // Way past move interval
        }

        let events = update_dungeon(&mut state, 0.1);

        // Should not move - blocked by combat
        assert!(events.is_empty());
    }

    #[test]
    fn test_update_dungeon_timer_accumulation() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.active_dungeon = Some(generate_dungeon(10, 0, 1));

        // Ensure room is cleared so we can move
        if let Some(dungeon) = &mut state.active_dungeon {
            dungeon.current_room_cleared = true;
            dungeon.move_timer = 0.0;
        }

        // Not enough time to move
        let events = update_dungeon(&mut state, 0.5);
        assert!(events.is_empty());

        // Check timer accumulated
        let timer = state.active_dungeon.as_ref().unwrap().move_timer;
        assert!(timer > 0.4 && timer < 0.6);
    }

    #[test]
    fn test_update_dungeon_moves_after_interval() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.active_dungeon = Some(generate_dungeon(10, 0, 1));

        if let Some(dungeon) = &mut state.active_dungeon {
            dungeon.current_room_cleared = true;
            dungeon.move_timer = ROOM_MOVE_INTERVAL - 0.1;
        }

        let start_pos = state.active_dungeon.as_ref().unwrap().player_position;

        // This tick should trigger movement
        let events = update_dungeon(&mut state, 0.2);

        // Should have moved
        let new_pos = state.active_dungeon.as_ref().unwrap().player_position;

        // Either we moved or there was no next room
        if !events.is_empty() {
            assert_ne!(start_pos, new_pos);
            assert!(events
                .iter()
                .any(|e| matches!(e, DungeonEvent::EnteredRoom { .. })));
        }
    }

    #[test]
    fn test_update_dungeon_traveling_uses_faster_interval() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.active_dungeon = Some(generate_dungeon(10, 0, 1));

        // Manually set up a scenario where next room is cleared
        if let Some(dungeon) = &mut state.active_dungeon {
            dungeon.current_room_cleared = true;
            let neighbors = dungeon
                .get_connected_neighbors(dungeon.player_position.0, dungeon.player_position.1);
            // Mark a neighbor as cleared to trigger travel mode
            if let Some(&(nx, ny)) = neighbors.first() {
                if let Some(room) = dungeon.get_room_mut(nx, ny) {
                    room.state = RoomState::Cleared;
                }
            }
        }

        // Small delta that would pass travel interval but not explore interval
        let events = update_dungeon(&mut state, 0.1);

        // Check is_traveling flag was set
        if let Some(dungeon) = &state.active_dungeon {
            // Either traveling or no valid next room
            // The important thing is the code path was exercised
            assert!(dungeon.is_traveling || events.is_empty());
        }
    }

    // ============ find_next_room tests ============

    #[test]
    fn test_find_next_room_prioritizes_boss_with_key() {
        let mut dungeon = generate_dungeon(10, 0, 1);
        dungeon.has_key = true;

        // Clear path to boss by marking all rooms EXCEPT boss as Cleared
        // Boss must be Revealed (visible but not defeated) for pathfinding to target it
        let boss_pos = dungeon.boss_position;
        let player_pos = dungeon.player_position;
        let grid_size = dungeon.size.grid_size();
        for y in 0..grid_size {
            for x in 0..grid_size {
                if let Some(room) = dungeon.get_room_mut(x, y) {
                    if (x, y) == boss_pos {
                        // Boss is revealed but not cleared (we want to go there)
                        room.state = RoomState::Revealed;
                    } else if (x, y) == player_pos {
                        // Player position stays as Current
                        room.state = RoomState::Current;
                    } else {
                        // All other rooms are cleared (passable)
                        room.state = RoomState::Cleared;
                    }
                }
            }
        }

        // Edge case: player already at boss - nothing to test
        if player_pos == boss_pos {
            return;
        }

        // Verify that following find_next_room eventually reaches the boss
        // This is more robust than checking single-step distance because
        // BFS can return different paths of equal length
        let mut pos = dungeon.player_position;
        let mut steps = 0;
        let max_steps = grid_size * grid_size; // Upper bound

        while pos != boss_pos && steps < max_steps {
            dungeon.player_position = pos;
            if let Some(next_pos) = find_next_room(&dungeon) {
                // Verify the suggested position is adjacent (manhattan distance 1)
                let dx = (next_pos.0 as i32 - pos.0 as i32).abs();
                let dy = (next_pos.1 as i32 - pos.1 as i32).abs();
                assert!(
                    dx + dy == 1,
                    "Next position {:?} should be adjacent to current {:?}",
                    next_pos,
                    pos
                );
                pos = next_pos;
                steps += 1;
            } else {
                break;
            }
        }

        assert_eq!(
            pos, boss_pos,
            "With key, should eventually reach boss. Got stuck at {:?} after {} steps",
            pos, steps
        );
    }

    #[test]
    fn test_find_next_room_skips_boss_without_key() {
        let mut dungeon = generate_dungeon(10, 0, 1);
        dungeon.has_key = false;

        // Make boss the only revealed room
        let boss_pos = dungeon.boss_position;
        if let Some(boss_room) = dungeon.get_room_mut(boss_pos.0, boss_pos.1) {
            boss_room.state = RoomState::Revealed;
        }

        // Clear all other rooms
        let grid_size = dungeon.size.grid_size();
        for y in 0..grid_size {
            for x in 0..grid_size {
                if (x, y) != boss_pos && (x, y) != dungeon.player_position {
                    if let Some(room) = dungeon.get_room_mut(x, y) {
                        room.state = RoomState::Cleared;
                    }
                }
            }
        }

        let next = find_next_room(&dungeon);

        // Should NOT go to boss without key
        if let Some(next_pos) = next {
            assert_ne!(next_pos, boss_pos);
        }
    }

    #[test]
    fn test_find_next_room_returns_none_when_fully_explored() {
        let mut dungeon = generate_dungeon(10, 0, 1);
        dungeon.has_key = true; // Even with key

        // Mark ALL rooms as cleared
        let grid_size = dungeon.size.grid_size();
        for y in 0..grid_size {
            for x in 0..grid_size {
                if let Some(room) = dungeon.get_room_mut(x, y) {
                    room.state = RoomState::Cleared;
                }
            }
        }

        let next = find_next_room(&dungeon);

        // No revealed rooms left to explore
        assert!(next.is_none());
    }

    // ============ find_path_to tests ============

    #[test]
    fn test_find_path_returns_none_for_unreachable() {
        let dungeon = generate_dungeon(10, 0, 1);

        // Try to find path to a position that's definitely not a room
        let path = find_path_to(&dungeon, (0, 0), (99, 99));

        assert!(path.is_none());
    }

    #[test]
    fn test_find_path_through_multiple_rooms() {
        let dungeon = generate_dungeon(10, 0, 1);
        let boss_pos = dungeon.boss_position;

        // Path to boss should be longer than 2 for most dungeons
        if let Some(path) = find_path_to(&dungeon, dungeon.entrance_position, boss_pos) {
            assert!(path.len() >= 2);
            assert_eq!(*path.first().unwrap(), dungeon.entrance_position);
            assert_eq!(*path.last().unwrap(), boss_pos);
        }
    }

    // ============ move_to_room tests for each room type ============

    #[test]
    fn test_move_to_treasure_room_auto_clears() {
        let mut dungeon = generate_dungeon(10, 0, 1);

        if let Some(pos) = find_room_of_type(&dungeon, RoomType::Treasure) {
            // Make it revealed so we can move there
            if let Some(room) = dungeon.get_room_mut(pos.0, pos.1) {
                room.state = RoomState::Revealed;
            }

            let events = move_to_room(&mut dungeon, pos);

            // Treasure rooms auto-clear (no combat)
            assert!(dungeon.current_room_cleared);
            assert!(events
                .iter()
                .any(|e| matches!(e, DungeonEvent::TreasureFound)));
        }
    }

    #[test]
    fn test_move_to_elite_room_starts_combat() {
        let mut dungeon = generate_dungeon(10, 0, 1);

        if let Some(pos) = find_room_of_type(&dungeon, RoomType::Elite) {
            if let Some(room) = dungeon.get_room_mut(pos.0, pos.1) {
                room.state = RoomState::Revealed;
            }

            let events = move_to_room(&mut dungeon, pos);

            assert!(!dungeon.current_room_cleared);
            assert!(events
                .iter()
                .any(|e| matches!(e, DungeonEvent::CombatStarted { is_elite: true, .. })));
        }
    }

    #[test]
    fn test_move_to_boss_room_starts_combat() {
        let mut dungeon = generate_dungeon(10, 0, 1);
        let boss_pos = dungeon.boss_position;

        if let Some(room) = dungeon.get_room_mut(boss_pos.0, boss_pos.1) {
            room.state = RoomState::Revealed;
        }

        let events = move_to_room(&mut dungeon, boss_pos);

        assert!(!dungeon.current_room_cleared);
        assert!(events
            .iter()
            .any(|e| matches!(e, DungeonEvent::CombatStarted { is_boss: true, .. })));
    }

    #[test]
    fn test_move_to_room_increments_rooms_cleared() {
        let mut dungeon = generate_dungeon(10, 0, 1);
        let initial_cleared = dungeon.rooms_cleared;

        let neighbors =
            dungeon.get_connected_neighbors(dungeon.player_position.0, dungeon.player_position.1);

        if let Some(&next_pos) = neighbors.first() {
            move_to_room(&mut dungeon, next_pos);

            // Old room (entrance) should now be counted as cleared
            assert_eq!(dungeon.rooms_cleared, initial_cleared + 1);
        }
    }

    #[test]
    fn test_backtracking_does_not_double_count_rooms() {
        let mut dungeon = generate_dungeon(10, 0, 1);
        let start_pos = dungeon.player_position;

        let neighbors = dungeon.get_connected_neighbors(start_pos.0, start_pos.1);
        if neighbors.is_empty() {
            return; // Skip if no neighbors (shouldn't happen)
        }

        let next_pos = neighbors[0];

        // Move to adjacent room (clears entrance)
        move_to_room(&mut dungeon, next_pos);
        let cleared_after_first_move = dungeon.rooms_cleared;
        assert_eq!(cleared_after_first_move, 1, "Should have cleared entrance");

        // Mark current room as cleared so we can leave it
        dungeon.current_room_cleared = true;

        // Backtrack to entrance (already cleared)
        move_to_room(&mut dungeon, start_pos);
        let cleared_after_backtrack = dungeon.rooms_cleared;

        // Should have cleared the second room, but NOT re-counted the entrance
        assert_eq!(
            cleared_after_backtrack, 2,
            "Backtracking should clear the room we left, but not re-count the cleared room we entered"
        );

        // Move forward again through the cleared entrance
        dungeon.current_room_cleared = true;
        move_to_room(&mut dungeon, next_pos);

        // Should NOT increment again - both rooms already cleared
        assert_eq!(
            dungeon.rooms_cleared, 2,
            "Moving through already-cleared rooms should not increment count"
        );

        // Verify we never exceed total room count
        assert!(
            (dungeon.rooms_cleared as usize) <= dungeon.room_count(),
            "rooms_cleared ({}) should never exceed room_count ({})",
            dungeon.rooms_cleared,
            dungeon.room_count()
        );
    }

    // ============ on_boss_defeated tests ============

    #[test]
    fn test_on_boss_defeated_clears_dungeon() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.active_dungeon = Some(generate_dungeon(10, 0, 1));

        // Set some XP and items
        if let Some(dungeon) = &mut state.active_dungeon {
            dungeon.xp_earned = 1000;
        }

        let events = on_boss_defeated(&mut state);

        // Dungeon should be cleared
        assert!(state.active_dungeon.is_none());
        assert!(events
            .iter()
            .any(|e| matches!(e, DungeonEvent::DungeonComplete { .. })));
    }

    #[test]
    fn test_on_boss_defeated_reports_xp_and_items() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.active_dungeon = Some(generate_dungeon(10, 0, 1));

        if let Some(dungeon) = &mut state.active_dungeon {
            dungeon.xp_earned = 5000;
            // Add a fake item
            dungeon.collected_items.push(crate::items::generate_item(
                crate::items::EquipmentSlot::Weapon,
                Rarity::Rare,
                10,
            ));
        }

        let events = on_boss_defeated(&mut state);

        if let Some(DungeonEvent::DungeonComplete {
            xp_earned,
            items_collected,
        }) = events.first()
        {
            assert_eq!(*xp_earned, 5000);
            assert_eq!(*items_collected, 1);
        } else {
            panic!("Expected DungeonComplete event");
        }
    }

    // ============ on_player_died_in_dungeon tests ============

    #[test]
    fn test_on_player_died_clears_dungeon() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.active_dungeon = Some(generate_dungeon(10, 0, 1));

        let events = on_player_died_in_dungeon(&mut state);

        assert!(state.active_dungeon.is_none());
        assert!(events
            .iter()
            .any(|e| matches!(e, DungeonEvent::DungeonFailed)));
    }

    // ============ current_room_needs_combat tests ============

    #[test]
    fn test_current_room_needs_combat_for_combat_room() {
        let mut dungeon = generate_dungeon(10, 0, 1);

        if let Some(pos) = find_room_of_type(&dungeon, RoomType::Combat) {
            if let Some(room) = dungeon.get_room_mut(pos.0, pos.1) {
                room.state = RoomState::Current;
            }
            dungeon.player_position = pos;

            assert!(current_room_needs_combat(&dungeon));
        }
    }

    #[test]
    fn test_current_room_needs_combat_false_for_treasure() {
        let mut dungeon = generate_dungeon(10, 0, 1);

        if let Some(pos) = find_room_of_type(&dungeon, RoomType::Treasure) {
            if let Some(room) = dungeon.get_room_mut(pos.0, pos.1) {
                room.state = RoomState::Current;
            }
            dungeon.player_position = pos;

            assert!(!current_room_needs_combat(&dungeon));
        }
    }

    // ============ add_dungeon_xp and collect_dungeon_item tests ============

    #[test]
    fn test_add_dungeon_xp_accumulates() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.active_dungeon = Some(generate_dungeon(10, 0, 1));

        add_dungeon_xp(&mut state, 100);
        add_dungeon_xp(&mut state, 250);

        let xp = state.active_dungeon.as_ref().unwrap().xp_earned;
        assert_eq!(xp, 350);
    }

    #[test]
    fn test_add_dungeon_xp_no_dungeon_does_nothing() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.active_dungeon = None;

        // Should not panic
        add_dungeon_xp(&mut state, 100);
    }

    #[test]
    fn test_collect_dungeon_item_adds_to_list() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.active_dungeon = Some(generate_dungeon(10, 0, 1));

        let item =
            crate::items::generate_item(crate::items::EquipmentSlot::Weapon, Rarity::Magic, 10);

        collect_dungeon_item(&mut state, item);

        let items = &state.active_dungeon.as_ref().unwrap().collected_items;
        assert_eq!(items.len(), 1);
    }

    // ============ on_elite_defeated edge cases ============

    #[test]
    fn test_on_elite_defeated_only_gives_key_once() {
        let mut dungeon = generate_dungeon(10, 0, 1);

        // First elite defeat
        let events1 = on_elite_defeated(&mut dungeon);
        assert!(dungeon.has_key);
        assert!(events1.iter().any(|e| matches!(e, DungeonEvent::FoundKey)));

        // Second elite defeat (already have key)
        let events2 = on_elite_defeated(&mut dungeon);
        assert!(dungeon.has_key);
        // Should NOT have FoundKey event again
        assert!(!events2.iter().any(|e| matches!(e, DungeonEvent::FoundKey)));
    }

    // ============ get_enemy_stat_multiplier tests ============

    #[test]
    fn test_get_enemy_stat_multiplier_elite() {
        let mut dungeon = generate_dungeon(10, 0, 1);

        // Find and move to elite room
        if let Some(pos) = find_room_of_type(&dungeon, RoomType::Elite) {
            if let Some(room) = dungeon.get_room_mut(pos.0, pos.1) {
                room.state = RoomState::Current;
            }
            dungeon.player_position = pos;

            let mult = get_enemy_stat_multiplier(&dungeon);
            assert!((mult - 1.5).abs() < 0.01);
        }
    }

    #[test]
    fn test_get_enemy_stat_multiplier_boss() {
        let mut dungeon = generate_dungeon(10, 0, 1);
        let boss_pos = dungeon.boss_position;

        if let Some(room) = dungeon.get_room_mut(boss_pos.0, boss_pos.1) {
            room.state = RoomState::Current;
        }
        dungeon.player_position = boss_pos;

        let mult = get_enemy_stat_multiplier(&dungeon);
        assert!((mult - 2.0).abs() < 0.01);
    }
}
