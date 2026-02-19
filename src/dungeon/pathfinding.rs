//! Dungeon pathfinding: BFS-based navigation between rooms.

use super::generation::reveal_adjacent_rooms;
use super::logic::DungeonEvent;
use super::types::{Dungeon, RoomState, RoomType};
use std::collections::{HashSet, VecDeque};

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
