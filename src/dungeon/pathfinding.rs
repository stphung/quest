//! Dungeon pathfinding: BFS-based navigation between rooms.

use super::generation::reveal_adjacent_rooms;
use super::logic::DungeonEvent;
use super::types::{Dungeon, RoomState, RoomType};
use std::collections::{HashMap, HashSet, VecDeque};

/// Time between room movements during auto-exploration (seconds)
pub const ROOM_MOVE_INTERVAL: f64 = 2.5;

/// Faster movement when traveling through already-cleared rooms (seconds)
pub const ROOM_TRAVEL_INTERVAL: f64 = 0.8;

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

    let mut visited: HashSet<(usize, usize)> = HashSet::new();
    let mut parents: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    visited.insert(from);
    queue.push_back(from);

    while let Some((x, y)) = queue.pop_front() {
        let neighbors = dungeon.get_connected_neighbors(x, y);

        for (nx, ny) in neighbors {
            if visited.contains(&(nx, ny)) {
                continue;
            }

            parents.insert((nx, ny), (x, y));

            if (nx, ny) == to {
                // Reconstruct path from parents
                let mut path = vec![to];
                let mut current = to;
                while current != from {
                    current = parents[&current];
                    path.push(current);
                }
                path.reverse();
                return Some(path);
            }

            // Can only traverse through cleared or current rooms (or revealed if it's the target)
            if let Some(room) = dungeon.get_room(nx, ny) {
                let can_traverse = matches!(
                    room.state,
                    RoomState::Cleared | RoomState::Current | RoomState::Revealed
                );

                if can_traverse {
                    visited.insert((nx, ny));
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    None
}

/// Moves player to a new room and handles room entry
pub(crate) fn move_to_room(dungeon: &mut Dungeon, new_pos: (usize, usize)) -> Vec<DungeonEvent> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::types::{DungeonSize, DIR_DOWN, DIR_LEFT, DIR_RIGHT, DIR_UP};

    /// Place a room of the given type at (x, y), starting Hidden.
    fn place(dungeon: &mut Dungeon, x: usize, y: usize, room_type: RoomType) {
        dungeon.grid[y][x] = Some(super::super::types::Room::new(room_type, (x, y)));
    }

    /// Bidirectionally connect two orthogonally-adjacent rooms.
    fn connect(dungeon: &mut Dungeon, a: (usize, usize), b: (usize, usize)) {
        let (ax, ay) = a;
        let (bx, by) = b;
        let dx = bx as i32 - ax as i32;
        let dy = by as i32 - ay as i32;
        let (dir_a, dir_b) = match (dx, dy) {
            (1, 0) => (DIR_RIGHT, DIR_LEFT),
            (-1, 0) => (DIR_LEFT, DIR_RIGHT),
            (0, 1) => (DIR_DOWN, DIR_UP),
            (0, -1) => (DIR_UP, DIR_DOWN),
            _ => panic!("connect() requires orthogonally adjacent rooms"),
        };
        dungeon.get_room_mut(ax, ay).unwrap().connections[dir_a] = true;
        dungeon.get_room_mut(bx, by).unwrap().connections[dir_b] = true;
    }

    fn reveal(dungeon: &mut Dungeon, x: usize, y: usize) {
        dungeon.get_room_mut(x, y).unwrap().state = RoomState::Revealed;
    }

    // ── find_path_to ──────────────────────────────────────────────────────────

    #[test]
    fn test_find_path_to_same_position_returns_single_element_path() {
        let dungeon = Dungeon::new(DungeonSize::Small);
        let path = find_path_to(&dungeon, (2, 2), (2, 2));
        assert_eq!(path, Some(vec![(2, 2)]));
    }

    #[test]
    fn test_find_path_to_no_connection_returns_none() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        place(&mut dungeon, 0, 0, RoomType::Entrance);
        place(&mut dungeon, 1, 0, RoomType::Combat);
        // Not connected.
        let path = find_path_to(&dungeon, (0, 0), (1, 0));
        assert_eq!(path, None);
    }

    #[test]
    fn test_find_path_to_finds_path_through_connected_rooms() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        place(&mut dungeon, 0, 0, RoomType::Entrance);
        place(&mut dungeon, 1, 0, RoomType::Combat);
        place(&mut dungeon, 2, 0, RoomType::Boss);
        connect(&mut dungeon, (0, 0), (1, 0));
        connect(&mut dungeon, (1, 0), (2, 0));
        reveal(&mut dungeon, 1, 0);
        reveal(&mut dungeon, 2, 0);

        let path = find_path_to(&dungeon, (0, 0), (2, 0)).expect("path should exist");
        assert_eq!(path, vec![(0, 0), (1, 0), (2, 0)]);
    }

    #[test]
    fn test_find_path_to_does_not_traverse_hidden_rooms() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        place(&mut dungeon, 0, 0, RoomType::Entrance);
        place(&mut dungeon, 1, 0, RoomType::Combat);
        place(&mut dungeon, 2, 0, RoomType::Boss);
        connect(&mut dungeon, (0, 0), (1, 0));
        connect(&mut dungeon, (1, 0), (2, 0));
        // (1,0) stays Hidden (default), so the path through it should be blocked.
        let path = find_path_to(&dungeon, (0, 0), (2, 0));
        assert_eq!(path, None);
    }

    // ── find_next_room ───────────────────────────────────────────────────────

    #[test]
    fn test_find_next_room_heads_to_boss_when_key_held_and_not_cleared() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        place(&mut dungeon, 0, 0, RoomType::Entrance);
        place(&mut dungeon, 1, 0, RoomType::Boss);
        connect(&mut dungeon, (0, 0), (1, 0));
        dungeon.get_room_mut(1, 0).unwrap().state = RoomState::Revealed;
        dungeon.player_position = (0, 0);
        dungeon.boss_position = (1, 0);
        dungeon.has_key = true;

        let next = find_next_room(&dungeon);
        assert_eq!(next, Some((1, 0)));
    }

    #[test]
    fn test_find_next_room_ignores_cleared_boss_even_with_key() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        place(&mut dungeon, 0, 0, RoomType::Entrance);
        place(&mut dungeon, 1, 0, RoomType::Boss);
        place(&mut dungeon, 0, 1, RoomType::Combat);
        connect(&mut dungeon, (0, 0), (1, 0));
        connect(&mut dungeon, (0, 0), (0, 1));
        dungeon.get_room_mut(1, 0).unwrap().state = RoomState::Cleared;
        reveal(&mut dungeon, 0, 1);
        dungeon.player_position = (0, 0);
        dungeon.boss_position = (1, 0);
        dungeon.has_key = true;

        // Boss is already cleared, so exploration should fall through to the
        // other revealed (unexplored) room instead.
        let next = find_next_room(&dungeon);
        assert_eq!(next, Some((0, 1)));
    }

    #[test]
    fn test_find_next_room_skips_boss_room_without_key() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        place(&mut dungeon, 0, 0, RoomType::Entrance);
        place(&mut dungeon, 1, 0, RoomType::Boss);
        connect(&mut dungeon, (0, 0), (1, 0));
        reveal(&mut dungeon, 1, 0);
        dungeon.player_position = (0, 0);
        dungeon.boss_position = (1, 0);
        dungeon.has_key = false;

        // The only revealed room is the boss room, but without the key it must
        // be skipped entirely, leaving no destination.
        let next = find_next_room(&dungeon);
        assert_eq!(next, None);
    }

    #[test]
    fn test_find_next_room_picks_shortest_path_among_revealed_rooms() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        place(&mut dungeon, 2, 2, RoomType::Entrance);
        place(&mut dungeon, 3, 2, RoomType::Combat); // 1 step away
        place(&mut dungeon, 4, 2, RoomType::Combat);
        place(&mut dungeon, 2, 3, RoomType::Combat);
        place(&mut dungeon, 2, 4, RoomType::Combat); // 2 steps away
        connect(&mut dungeon, (2, 2), (3, 2));
        connect(&mut dungeon, (3, 2), (4, 2));
        connect(&mut dungeon, (2, 2), (2, 3));
        connect(&mut dungeon, (2, 3), (2, 4));
        reveal(&mut dungeon, 3, 2);
        reveal(&mut dungeon, 4, 2);
        reveal(&mut dungeon, 2, 3);
        reveal(&mut dungeon, 2, 4);
        dungeon.player_position = (2, 2);

        // Nearest revealed room is (3,2), one step away.
        let next = find_next_room(&dungeon);
        assert_eq!(next, Some((3, 2)));
    }

    #[test]
    fn test_find_next_room_returns_none_when_nothing_revealed() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        place(&mut dungeon, 0, 0, RoomType::Entrance);
        dungeon.player_position = (0, 0);
        assert_eq!(find_next_room(&dungeon), None);
    }

    // ── move_to_room ─────────────────────────────────────────────────────────

    #[test]
    fn test_move_to_room_clears_old_current_room_and_counts_it() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        place(&mut dungeon, 0, 0, RoomType::Entrance);
        place(&mut dungeon, 1, 0, RoomType::Combat);
        connect(&mut dungeon, (0, 0), (1, 0));
        reveal(&mut dungeon, 1, 0);
        dungeon.get_room_mut(0, 0).unwrap().state = RoomState::Current;
        dungeon.player_position = (0, 0);
        dungeon.rooms_cleared = 0;

        let events = move_to_room(&mut dungeon, (1, 0));

        assert_eq!(
            dungeon.get_room(0, 0).unwrap().state,
            RoomState::Cleared,
            "old room should be marked cleared"
        );
        assert_eq!(dungeon.rooms_cleared, 1);
        assert_eq!(dungeon.player_position, (1, 0));
        assert!(events.iter().any(|e| matches!(
            e,
            DungeonEvent::EnteredRoom {
                room_type: RoomType::Combat,
                position: (1, 0)
            }
        )));
        assert!(events.iter().any(|e| matches!(
            e,
            DungeonEvent::CombatStarted {
                is_elite: false,
                is_boss: false
            }
        )));
    }

    #[test]
    fn test_move_to_room_elite_emits_elite_combat_started() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        place(&mut dungeon, 0, 0, RoomType::Entrance);
        place(&mut dungeon, 1, 0, RoomType::Elite);
        connect(&mut dungeon, (0, 0), (1, 0));
        dungeon.player_position = (0, 0);

        let events = move_to_room(&mut dungeon, (1, 0));
        assert!(events.iter().any(|e| matches!(
            e,
            DungeonEvent::CombatStarted {
                is_elite: true,
                is_boss: false
            }
        )));
        assert!(!dungeon.current_room_cleared);
    }

    #[test]
    fn test_move_to_room_boss_emits_boss_combat_started() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        place(&mut dungeon, 0, 0, RoomType::Entrance);
        place(&mut dungeon, 1, 0, RoomType::Boss);
        connect(&mut dungeon, (0, 0), (1, 0));
        dungeon.player_position = (0, 0);

        let events = move_to_room(&mut dungeon, (1, 0));
        assert!(events.iter().any(|e| matches!(
            e,
            DungeonEvent::CombatStarted {
                is_elite: false,
                is_boss: true
            }
        )));
    }

    #[test]
    fn test_move_to_room_treasure_emits_treasure_found_and_is_cleared() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        place(&mut dungeon, 0, 0, RoomType::Entrance);
        place(&mut dungeon, 1, 0, RoomType::Treasure);
        connect(&mut dungeon, (0, 0), (1, 0));
        dungeon.player_position = (0, 0);

        let events = move_to_room(&mut dungeon, (1, 0));
        assert!(events
            .iter()
            .any(|e| matches!(e, DungeonEvent::TreasureFound)));
        assert!(
            dungeon.current_room_cleared,
            "Treasure rooms auto-clear (no combat gate)"
        );
    }

    #[test]
    fn test_move_to_room_entrance_emits_no_extra_event() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        place(&mut dungeon, 0, 0, RoomType::Combat);
        place(&mut dungeon, 1, 0, RoomType::Entrance);
        connect(&mut dungeon, (0, 0), (1, 0));
        dungeon.player_position = (0, 0);

        let events = move_to_room(&mut dungeon, (1, 0));
        // Only the EnteredRoom event — no CombatStarted/TreasureFound for Entrance.
        assert_eq!(events.len(), 1);
        assert!(dungeon.current_room_cleared);
    }

    #[test]
    fn test_move_to_room_backtrack_to_cleared_room_skips_room_events() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        place(&mut dungeon, 0, 0, RoomType::Combat);
        place(&mut dungeon, 1, 0, RoomType::Combat);
        connect(&mut dungeon, (0, 0), (1, 0));
        dungeon.get_room_mut(1, 0).unwrap().state = RoomState::Cleared;
        dungeon.player_position = (0, 0);
        dungeon.rooms_cleared = 5;

        let events = move_to_room(&mut dungeon, (1, 0));

        // Backtracking into an already-cleared room should not re-trigger combat,
        // should not re-increment rooms_cleared, and current_room_cleared should
        // be set true via the `was_already_cleared` fallback.
        assert_eq!(dungeon.rooms_cleared, 5);
        assert!(dungeon.current_room_cleared);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DungeonEvent::EnteredRoom { .. }));
    }
}
