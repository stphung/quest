//! Dungeon navigation and auto-exploration logic.

#![allow(dead_code)]

use crate::dungeon::{Dungeon, DungeonSize, RoomState, RoomType};
use crate::dungeon_generation::reveal_adjacent_rooms;
use crate::game_state::GameState;
use crate::item_drops::{roll_random_slot, roll_rarity};
use crate::item_generation::generate_item;
use crate::items::{Item, Rarity};
use rand::Rng;
use std::collections::{HashSet, VecDeque};

/// Time between room movements during auto-exploration (seconds)
pub const ROOM_MOVE_INTERVAL: f64 = 2.5;

/// Events that can occur during dungeon exploration
#[derive(Debug, Clone)]
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

    // Check if it's time to move
    if dungeon.move_timer >= ROOM_MOVE_INTERVAL {
        dungeon.move_timer = 0.0;

        // Find next room to explore
        if let Some(next_pos) = find_next_room(dungeon) {
            // Move to the next room
            let move_events = move_to_room(dungeon, next_pos);
            events.extend(move_events);
        }
    }

    events
}

/// Finds the next room to explore using BFS
/// Prioritizes: unexplored rooms, then boss (if has key)
fn find_next_room(dungeon: &Dungeon) -> Option<(usize, usize)> {
    let current = dungeon.player_position;

    // If we have the key and boss is accessible, go to boss
    if dungeon.has_key {
        if let Some(path) = find_path_to(dungeon, current, dungeon.boss_position) {
            if path.len() > 1 {
                return Some(path[1]); // Next step toward boss
            }
        }
    }

    // Find nearest unexplored (revealed but not cleared) room
    let mut best_target: Option<(usize, usize)> = None;
    let mut best_distance = usize::MAX;

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
                        if path.len() < best_distance {
                            best_distance = path.len();
                            best_target = Some((x, y));
                        }
                    }
                }
            }
        }
    }

    // Return first step toward target
    if let Some(target) = best_target {
        if let Some(path) = find_path_to(dungeon, current, target) {
            if path.len() > 1 {
                return Some(path[1]);
            }
        }
    }

    None
}

/// BFS pathfinding between two positions
fn find_path_to(
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

    // Get room type before mutating
    let room_type = dungeon
        .get_room(new_pos.0, new_pos.1)
        .map(|r| r.room_type)
        .unwrap_or(RoomType::Combat);

    // Mark new room as current
    if let Some(new_room) = dungeon.get_room_mut(new_pos.0, new_pos.1) {
        new_room.state = RoomState::Current;
    }

    // Reveal adjacent rooms
    reveal_adjacent_rooms(dungeon, new_pos.0, new_pos.1);

    // Set current_room_cleared based on room type
    // Combat rooms need enemy defeated before moving on
    dungeon.current_room_cleared = matches!(room_type, RoomType::Entrance | RoomType::Treasure);

    // Emit entered room event
    events.push(DungeonEvent::EnteredRoom {
        room_type,
        position: new_pos,
    });

    // Handle room-specific events
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
pub fn on_player_died_in_dungeon(state: &mut GameState) -> Vec<DungeonEvent> {
    let events = vec![DungeonEvent::DungeonFailed];

    // Clear dungeon (keep collected items via different mechanism if desired)
    state.active_dungeon = None;

    events
}

/// Calculates the XP reward for defeating a dungeon boss
pub fn calculate_boss_xp_reward(size: DungeonSize) -> u64 {
    let mut rng = rand::thread_rng();
    let (min_xp, max_xp) = size.boss_xp_range();
    rng.gen_range(min_xp..=max_xp)
}

/// Generates a treasure room item with rarity boost based on dungeon size
pub fn generate_treasure_item(prestige_rank: u32, player_level: u32, rarity_boost: u32) -> Item {
    let mut rng = rand::thread_rng();

    // Roll a random slot
    let slot = roll_random_slot(&mut rng);

    // Roll rarity with boost based on dungeon tier
    let base_rarity = roll_rarity(prestige_rank, &mut rng);
    let boosted_rarity = boost_rarity(base_rarity, rarity_boost);

    generate_item(slot, boosted_rarity, player_level)
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

    let item = generate_treasure_item(state.prestige_rank, state.character_level, rarity_boost);

    // Auto-equip if better
    let item_clone = item.clone();
    let equipped = crate::item_scoring::auto_equip_if_better(item, state);

    // Collect in dungeon tally (whether equipped or not, for completion summary)
    if let Some(dungeon) = &mut state.active_dungeon {
        dungeon.collected_items.push(item_clone.clone());
    }

    Some((item_clone, equipped))
}

/// Checks if the current room needs combat resolution
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
    use super::*;
    use crate::dungeon_generation::generate_dungeon;

    #[test]
    fn test_find_path_same_position() {
        let dungeon = generate_dungeon(10, 0);
        let pos = dungeon.player_position;
        let path = find_path_to(&dungeon, pos, pos);
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 1);
    }

    #[test]
    fn test_find_path_to_adjacent() {
        let dungeon = generate_dungeon(10, 0);
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
        let dungeon = generate_dungeon(10, 0);
        // From entrance, there should be a next room to explore
        let next = find_next_room(&dungeon);
        assert!(next.is_some());
    }

    #[test]
    fn test_move_to_room_updates_state() {
        let mut dungeon = generate_dungeon(10, 0);
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
        let mut dungeon = generate_dungeon(10, 0);
        assert!(!dungeon.has_key);

        let events = on_elite_defeated(&mut dungeon);

        assert!(dungeon.has_key);
        assert!(events.iter().any(|e| matches!(e, DungeonEvent::FoundKey)));
    }

    #[test]
    fn test_get_enemy_stat_multiplier() {
        let dungeon = generate_dungeon(10, 0);
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
        let item = generate_treasure_item(0, 10, 1);
        assert!(!item.display_name.is_empty());
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
}
