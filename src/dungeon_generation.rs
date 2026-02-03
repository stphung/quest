//! Dungeon procedural generation algorithms.
//!
//! These functions will be used when dungeon discovery is integrated.

#![allow(dead_code)]

use crate::dungeon::{
    Dungeon, DungeonSize, Room, RoomState, RoomType, DIR_DOWN, DIR_LEFT, DIR_OFFSETS, DIR_RIGHT,
    DIR_UP,
};
use rand::seq::SliceRandom;
use rand::Rng;

/// Generates a complete dungeon with rooms and connections
pub fn generate_dungeon(level: u32, prestige_rank: u32) -> Dungeon {
    let size = DungeonSize::roll_from_progression(level, prestige_rank);
    let mut dungeon = Dungeon::new(size);
    let mut rng = rand::thread_rng();

    // Generate maze structure (without extra connections yet)
    generate_maze(&mut dungeon);

    // Place special rooms first (boss needs to be in a dead end)
    place_special_rooms(&mut dungeon);

    // Add extra connections for variety, but skip boss room to keep it as dead end
    add_extra_connections(&mut dungeon, &mut rng);

    // Set initial player position and reveal entrance
    let entrance_pos = dungeon.entrance_position;
    dungeon.player_position = entrance_pos;
    if let Some(room) = dungeon.get_room_mut(entrance_pos.0, entrance_pos.1) {
        room.state = RoomState::Current;
    }

    // Reveal adjacent rooms to entrance
    reveal_adjacent_rooms(&mut dungeon, entrance_pos.0, entrance_pos.1);

    dungeon
}

/// Generates maze using randomized depth-first search (recursive backtracker)
fn generate_maze(dungeon: &mut Dungeon) {
    let grid_size = dungeon.size.grid_size();
    let (min_rooms, max_rooms) = dungeon.size.room_count_range();
    let mut rng = rand::thread_rng();

    // Target room count
    let target_rooms = rng.gen_range(min_rooms..=max_rooms);

    // Start from center of grid
    let start_x = grid_size / 2;
    let start_y = grid_size / 2;

    // Track visited cells and the path
    let mut visited = vec![vec![false; grid_size]; grid_size];
    let mut stack: Vec<(usize, usize)> = Vec::new();

    // Create first room
    dungeon.grid[start_y][start_x] = Some(Room::new(RoomType::Combat, (start_x, start_y)));
    visited[start_y][start_x] = true;
    stack.push((start_x, start_y));

    let mut room_count = 1;

    // DFS maze generation
    while !stack.is_empty() && room_count < target_rooms {
        let (cx, cy) = *stack.last().unwrap();

        // Get unvisited neighbors
        let mut neighbors: Vec<(usize, usize, usize)> = Vec::new();

        for (dir_idx, &(dx, dy)) in DIR_OFFSETS.iter().enumerate() {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;

            if nx >= 0 && ny >= 0 && (nx as usize) < grid_size && (ny as usize) < grid_size {
                let nx = nx as usize;
                let ny = ny as usize;

                if !visited[ny][nx] {
                    neighbors.push((nx, ny, dir_idx));
                }
            }
        }

        if neighbors.is_empty() {
            // Backtrack
            stack.pop();
        } else {
            // Choose random neighbor
            let &(nx, ny, dir_idx) = neighbors.choose(&mut rng).unwrap();

            // Create room at neighbor
            dungeon.grid[ny][nx] = Some(Room::new(RoomType::Combat, (nx, ny)));
            visited[ny][nx] = true;
            room_count += 1;

            // Connect rooms
            let opposite_dir = opposite_direction(dir_idx);

            if let Some(current_room) = dungeon.get_room_mut(cx, cy) {
                current_room.connections[dir_idx] = true;
            }
            if let Some(new_room) = dungeon.get_room_mut(nx, ny) {
                new_room.connections[opposite_dir] = true;
            }

            stack.push((nx, ny));
        }
    }
}

/// Returns the opposite direction index
fn opposite_direction(dir: usize) -> usize {
    match dir {
        DIR_UP => DIR_DOWN,
        DIR_RIGHT => DIR_LEFT,
        DIR_DOWN => DIR_UP,
        DIR_LEFT => DIR_RIGHT,
        _ => dir,
    }
}

/// Adds extra connections between adjacent rooms for variety
/// Skips the boss room to ensure it remains a dead end
fn add_extra_connections(dungeon: &mut Dungeon, rng: &mut impl Rng) {
    let grid_size = dungeon.size.grid_size();
    let extra_connection_chance = 0.15; // 15% chance per possible connection
    let boss_pos = dungeon.boss_position;

    for y in 0..grid_size {
        for x in 0..grid_size {
            if dungeon.get_room(x, y).is_none() {
                continue;
            }

            // Skip boss room - it must remain a dead end
            if (x, y) == boss_pos {
                continue;
            }

            // Check right and down neighbors only (to avoid double-checking)
            for &(dir_idx, dx, dy) in &[(DIR_RIGHT, 1i32, 0i32), (DIR_DOWN, 0i32, 1i32)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && ny >= 0 && (nx as usize) < grid_size && (ny as usize) < grid_size {
                    let nx = nx as usize;
                    let ny = ny as usize;

                    // Skip if neighbor is boss room
                    if (nx, ny) == boss_pos {
                        continue;
                    }

                    // If neighbor room exists and not already connected
                    if dungeon.get_room(nx, ny).is_some() {
                        let already_connected = dungeon
                            .get_room(x, y)
                            .map(|r| r.connections[dir_idx])
                            .unwrap_or(false);

                        if !already_connected && rng.gen::<f64>() < extra_connection_chance {
                            let opposite_dir = opposite_direction(dir_idx);

                            if let Some(room) = dungeon.get_room_mut(x, y) {
                                room.connections[dir_idx] = true;
                            }
                            if let Some(room) = dungeon.get_room_mut(nx, ny) {
                                room.connections[opposite_dir] = true;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Counts the number of connections a room has
fn connection_count(dungeon: &Dungeon, x: usize, y: usize) -> usize {
    dungeon
        .get_room(x, y)
        .map(|r| r.connections.iter().filter(|&&c| c).count())
        .unwrap_or(0)
}

/// Finds all "dead end" rooms (rooms with only one connection)
fn find_dead_ends(dungeon: &Dungeon) -> Vec<(usize, usize)> {
    let grid_size = dungeon.size.grid_size();
    let mut dead_ends = Vec::new();

    for y in 0..grid_size {
        for x in 0..grid_size {
            if dungeon.get_room(x, y).is_some() && connection_count(dungeon, x, y) == 1 {
                dead_ends.push((x, y));
            }
        }
    }

    dead_ends
}

/// Places special rooms (entrance, boss, elite, treasure)
fn place_special_rooms(dungeon: &mut Dungeon) {
    let mut rng = rand::thread_rng();

    // Collect all room positions
    let mut room_positions: Vec<(usize, usize)> = Vec::new();
    let grid_size = dungeon.size.grid_size();

    for y in 0..grid_size {
        for x in 0..grid_size {
            if dungeon.get_room(x, y).is_some() {
                room_positions.push((x, y));
            }
        }
    }

    if room_positions.is_empty() {
        return;
    }

    // Find dead ends for boss placement (boss should never block path to key)
    let mut dead_ends = find_dead_ends(dungeon);

    // Sort dead ends by distance from center (furthest first for entrance)
    let center = (grid_size / 2, grid_size / 2);
    dead_ends.sort_by(|a, b| {
        let dist_a = distance_squared(*a, center);
        let dist_b = distance_squared(*b, center);
        dist_b.cmp(&dist_a)
    });

    // Place entrance at a dead end if possible, otherwise edge room
    let entrance_pos = if !dead_ends.is_empty() {
        dead_ends.remove(0)
    } else {
        room_positions.sort_by(|a, b| {
            let dist_a = distance_squared(*a, center);
            let dist_b = distance_squared(*b, center);
            dist_b.cmp(&dist_a)
        });
        room_positions.pop().unwrap()
    };

    dungeon.entrance_position = entrance_pos;
    if let Some(room) = dungeon.get_room_mut(entrance_pos.0, entrance_pos.1) {
        room.room_type = RoomType::Entrance;
    }
    room_positions.retain(|&p| p != entrance_pos);

    // Place boss at a dead end (furthest from entrance) - MUST be a dead end
    // This ensures boss never blocks the path to the key
    dead_ends.retain(|&p| p != entrance_pos);
    dead_ends.sort_by(|a, b| {
        let dist_a = distance_squared(*a, entrance_pos);
        let dist_b = distance_squared(*b, entrance_pos);
        dist_b.cmp(&dist_a)
    });

    let boss_pos = if !dead_ends.is_empty() {
        dead_ends.remove(0)
    } else {
        // Fallback: find room furthest from entrance (shouldn't happen with proper maze)
        room_positions.sort_by(|a, b| {
            let dist_a = distance_squared(*a, entrance_pos);
            let dist_b = distance_squared(*b, entrance_pos);
            dist_b.cmp(&dist_a)
        });
        room_positions.remove(0)
    };

    dungeon.boss_position = boss_pos;
    if let Some(room) = dungeon.get_room_mut(boss_pos.0, boss_pos.1) {
        room.room_type = RoomType::Boss;
    }
    room_positions.retain(|&p| p != boss_pos);
    dead_ends.retain(|&p| p != boss_pos);

    // Place elite room (key guardian) in a dead end alcove that requires exploration
    // Strategy: Find a dead end that's far from entrance (requires exploration)
    // but not adjacent to boss (you have to seek it out)
    dead_ends.sort_by(|a, b| {
        // Score = distance from entrance (want high) + distance from boss (want moderate)
        // This places the key in a hidden corner requiring exploration
        let dist_entrance_a = distance_squared(*a, entrance_pos);
        let dist_entrance_b = distance_squared(*b, entrance_pos);
        // Prefer rooms far from entrance
        dist_entrance_b.cmp(&dist_entrance_a)
    });

    // Filter out dead ends too close to entrance (within 2 steps)
    let min_distance_from_entrance = 4; // squared distance of 2
    let viable_dead_ends: Vec<_> = dead_ends
        .iter()
        .filter(|&&pos| distance_squared(pos, entrance_pos) >= min_distance_from_entrance)
        .cloned()
        .collect();

    // Minimum squared distance from entrance (2 = not adjacent, not diagonal)
    let min_elite_distance = 2;

    let elite_pos = if !viable_dead_ends.is_empty() {
        // Pick the furthest viable dead end from entrance
        viable_dead_ends[0]
    } else if !dead_ends.is_empty() {
        // Fallback: pick dead end furthest from entrance that meets minimum distance
        dead_ends
            .iter()
            .find(|&&pos| distance_squared(pos, entrance_pos) > min_elite_distance)
            .cloned()
            .unwrap_or(dead_ends[0])
    } else {
        // Fallback: furthest room from entrance that meets minimum distance
        room_positions.sort_by(|a, b| {
            let dist_a = distance_squared(*a, entrance_pos);
            let dist_b = distance_squared(*b, entrance_pos);
            dist_b.cmp(&dist_a)
        });
        room_positions
            .iter()
            .find(|&&pos| distance_squared(pos, entrance_pos) > min_elite_distance)
            .cloned()
            .unwrap_or(room_positions[0])
    };

    if let Some(room) = dungeon.get_room_mut(elite_pos.0, elite_pos.1) {
        room.room_type = RoomType::Elite;
    }
    room_positions.retain(|&p| p != elite_pos);

    // Shuffle remaining rooms for random placement
    room_positions.shuffle(&mut rng);

    // Place treasure rooms based on dungeon size
    let treasure_count = dungeon.size.treasure_room_count().min(room_positions.len());
    for _ in 0..treasure_count {
        if let Some(pos) = room_positions.pop() {
            if let Some(room) = dungeon.get_room_mut(pos.0, pos.1) {
                room.room_type = RoomType::Treasure;
            }
        }
    }

    // Remaining rooms stay as Combat (default)
}

/// Calculate squared distance between two points
fn distance_squared(a: (usize, usize), b: (usize, usize)) -> usize {
    let dx = (a.0 as i32 - b.0 as i32).unsigned_abs() as usize;
    let dy = (a.1 as i32 - b.1 as i32).unsigned_abs() as usize;
    dx * dx + dy * dy
}

/// Reveals rooms adjacent to the given position
pub fn reveal_adjacent_rooms(dungeon: &mut Dungeon, x: usize, y: usize) {
    let neighbors = dungeon.get_connected_neighbors(x, y);

    for (nx, ny) in neighbors {
        if let Some(room) = dungeon.get_room_mut(nx, ny) {
            if room.state == RoomState::Hidden {
                room.state = RoomState::Revealed;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_dungeon_low_level() {
        // Low level can roll Small or Medium (±1 from expected Small)
        let dungeon = generate_dungeon(10, 0);

        // Should have rooms within valid range for rolled size
        let (min, max) = dungeon.size.room_count_range();
        let room_count = dungeon.room_count();
        assert!(
            (min..=max).contains(&room_count),
            "Room count {} not in range {:?} for size {:?}",
            room_count,
            (min, max),
            dungeon.size
        );

        // Should have entrance and boss
        assert!(dungeon
            .get_room(dungeon.entrance_position.0, dungeon.entrance_position.1)
            .is_some());
        assert!(dungeon
            .get_room(dungeon.boss_position.0, dungeon.boss_position.1)
            .is_some());

        // Entrance should be marked as entrance
        let entrance = dungeon
            .get_room(dungeon.entrance_position.0, dungeon.entrance_position.1)
            .unwrap();
        assert_eq!(entrance.room_type, RoomType::Entrance);

        // Boss should be marked as boss
        let boss = dungeon
            .get_room(dungeon.boss_position.0, dungeon.boss_position.1)
            .unwrap();
        assert_eq!(boss.room_type, RoomType::Boss);
    }

    #[test]
    fn test_generate_dungeon_mid_level() {
        // Mid level can roll Small, Medium, or Large (±1 from expected Medium)
        let dungeon = generate_dungeon(50, 0);

        let (min, max) = dungeon.size.room_count_range();
        let room_count = dungeon.room_count();
        assert!(
            (min..=max).contains(&room_count),
            "Room count {} not in range {:?} for size {:?}",
            room_count,
            (min, max),
            dungeon.size
        );
    }

    #[test]
    fn test_generate_dungeon_high_level() {
        // High level can roll Medium, Large, or Epic (±1 from expected Large)
        let dungeon = generate_dungeon(100, 0);

        let (min, max) = dungeon.size.room_count_range();
        let room_count = dungeon.room_count();
        assert!(
            (min..=max).contains(&room_count),
            "Room count {} not in range {:?} for size {:?}",
            room_count,
            (min, max),
            dungeon.size
        );
    }

    #[test]
    fn test_player_starts_at_entrance() {
        let dungeon = generate_dungeon(10, 0);
        assert_eq!(dungeon.player_position, dungeon.entrance_position);
    }

    #[test]
    fn test_entrance_room_state() {
        let dungeon = generate_dungeon(10, 0);
        let entrance = dungeon.current_room().unwrap();
        assert_eq!(entrance.state, RoomState::Current);
    }

    #[test]
    fn test_adjacent_rooms_revealed() {
        let dungeon = generate_dungeon(10, 0);
        let entrance_pos = dungeon.entrance_position;
        let neighbors = dungeon.get_connected_neighbors(entrance_pos.0, entrance_pos.1);

        // All connected neighbors should be revealed
        for (nx, ny) in neighbors {
            let room = dungeon.get_room(nx, ny).unwrap();
            assert_eq!(room.state, RoomState::Revealed);
        }
    }

    #[test]
    fn test_has_elite_room() {
        let dungeon = generate_dungeon(10, 0);

        let has_elite = dungeon.grid.iter().any(|row| {
            row.iter().any(|r| {
                r.as_ref()
                    .map(|room| room.room_type == RoomType::Elite)
                    .unwrap_or(false)
            })
        });

        assert!(has_elite, "Dungeon should have an elite room with the key");
    }

    #[test]
    fn test_has_treasure_rooms() {
        let dungeon = generate_dungeon(50, 0); // Medium dungeon for more rooms

        let treasure_count = dungeon.grid.iter().fold(0, |acc, row| {
            acc + row
                .iter()
                .filter(|r| {
                    r.as_ref()
                        .map(|room| room.room_type == RoomType::Treasure)
                        .unwrap_or(false)
                })
                .count()
        });

        assert!(
            treasure_count >= 1,
            "Dungeon should have at least one treasure room"
        );
    }

    #[test]
    fn test_boss_is_dead_end() {
        // Test multiple dungeons to ensure boss is always in a dead end
        for _ in 0..10 {
            let dungeon = generate_dungeon(50, 0);
            let boss_pos = dungeon.boss_position;
            let connections = connection_count(&dungeon, boss_pos.0, boss_pos.1);

            assert_eq!(
                connections, 1,
                "Boss room should be a dead end (1 connection), but has {}",
                connections
            );
        }
    }

    #[test]
    fn test_elite_not_adjacent_to_entrance() {
        // Test that elite (key) room requires some exploration
        for _ in 0..10 {
            let dungeon = generate_dungeon(50, 0);

            // Find elite room position
            let mut elite_pos = None;
            let grid_size = dungeon.size.grid_size();
            for y in 0..grid_size {
                for x in 0..grid_size {
                    if let Some(room) = dungeon.get_room(x, y) {
                        if room.room_type == RoomType::Elite {
                            elite_pos = Some((x, y));
                        }
                    }
                }
            }

            if let Some(elite) = elite_pos {
                let entrance = dungeon.entrance_position;
                let distance = distance_squared(elite, entrance);
                // Elite should not be immediately adjacent to entrance (distance > 1)
                assert!(
                    distance > 1,
                    "Elite room should not be adjacent to entrance, distance: {}",
                    distance
                );
            }
        }
    }

    // =========================================================================
    // DUNGEON REACHABILITY TESTS
    // =========================================================================

    /// BFS to find all reachable rooms from a starting position
    fn find_reachable_rooms(dungeon: &Dungeon, start: (usize, usize)) -> Vec<(usize, usize)> {
        use std::collections::{HashSet, VecDeque};

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut reachable = Vec::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(pos) = queue.pop_front() {
            reachable.push(pos);

            // Get connected neighbors
            let neighbors = dungeon.get_connected_neighbors(pos.0, pos.1);
            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }

        reachable
    }

    #[test]
    fn test_all_rooms_reachable_from_entrance() {
        // Test multiple dungeons of various sizes
        for _ in 0..20 {
            let dungeon = generate_dungeon(50, 0);

            // Count total rooms
            let grid_size = dungeon.size.grid_size();
            let mut total_rooms = 0;
            for y in 0..grid_size {
                for x in 0..grid_size {
                    if dungeon.get_room(x, y).is_some() {
                        total_rooms += 1;
                    }
                }
            }

            // Find all reachable rooms from entrance
            let reachable = find_reachable_rooms(&dungeon, dungeon.entrance_position);

            assert_eq!(
                reachable.len(),
                total_rooms,
                "All {} rooms should be reachable from entrance, but only {} are reachable",
                total_rooms,
                reachable.len()
            );
        }
    }

    #[test]
    fn test_boss_reachable_from_entrance() {
        for _ in 0..20 {
            let dungeon = generate_dungeon(50, 0);
            let reachable = find_reachable_rooms(&dungeon, dungeon.entrance_position);

            assert!(
                reachable.contains(&dungeon.boss_position),
                "Boss room at {:?} should be reachable from entrance at {:?}",
                dungeon.boss_position,
                dungeon.entrance_position
            );
        }
    }

    #[test]
    fn test_elite_reachable_from_entrance() {
        for _ in 0..20 {
            let dungeon = generate_dungeon(50, 0);

            // Find elite room
            let grid_size = dungeon.size.grid_size();
            let mut elite_pos = None;
            for y in 0..grid_size {
                for x in 0..grid_size {
                    if let Some(room) = dungeon.get_room(x, y) {
                        if room.room_type == RoomType::Elite {
                            elite_pos = Some((x, y));
                            break;
                        }
                    }
                }
            }

            if let Some(elite) = elite_pos {
                let reachable = find_reachable_rooms(&dungeon, dungeon.entrance_position);
                assert!(
                    reachable.contains(&elite),
                    "Elite room (key guardian) at {:?} should be reachable from entrance",
                    elite
                );
            }
        }
    }

    #[test]
    fn test_all_dungeon_sizes_produce_valid_dungeons() {
        // Test each dungeon size explicitly
        let test_cases = [
            (10, 0),  // Should produce Small
            (30, 0),  // Should produce Small/Medium
            (50, 0),  // Should produce Medium
            (75, 0),  // Should produce Medium/Large
            (100, 0), // Should produce Large
            (50, 5),  // With prestige - could produce larger
        ];

        for (level, prestige) in test_cases {
            for _ in 0..5 {
                let dungeon = generate_dungeon(level, prestige);

                // Verify basic structure
                assert!(
                    dungeon.room_count() >= 6,
                    "Dungeon should have at least 6 rooms"
                );

                // Verify all rooms reachable
                let reachable = find_reachable_rooms(&dungeon, dungeon.entrance_position);
                assert_eq!(
                    reachable.len(),
                    dungeon.room_count(),
                    "Level {}, Prestige {}: All rooms should be reachable",
                    level,
                    prestige
                );

                // Verify special rooms exist and are reachable
                assert!(
                    reachable.contains(&dungeon.boss_position),
                    "Boss should be reachable"
                );
            }
        }
    }
}
