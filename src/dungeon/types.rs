//! Dungeon exploration data structures.
//!
//! These types define the dungeon system for procedural exploration.
//! Functions will be used when dungeon generation/navigation is integrated.

#![allow(dead_code)]

use crate::core::constants::{
    DUNGEON_LEVEL_TIER_LARGE, DUNGEON_LEVEL_TIER_MEDIUM, DUNGEON_PRESTIGE_PER_SIZE_TIER,
    DUNGEON_SIZE_VARIATION_DOWN, DUNGEON_SIZE_VARIATION_UP,
};
use serde::{Deserialize, Serialize};

/// Type of room in the dungeon
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomType {
    /// Starting room - no enemies
    Entrance,
    /// Standard combat encounter (60% spawn rate)
    Combat,
    /// Guaranteed item drop, no combat (20% spawn rate)
    Treasure,
    /// Tough enemy guarding the big key (15% spawn rate)
    Elite,
    /// Final encounter, requires big key to unlock (5% spawn rate)
    Boss,
}

impl RoomType {
    /// Returns the display character for this room type
    pub fn icon(&self) -> char {
        match self {
            RoomType::Entrance => 'E',
            RoomType::Combat => 'C',
            RoomType::Treasure => 'T',
            RoomType::Elite => 'K', // Key guardian
            RoomType::Boss => 'B',
        }
    }

    /// Returns the display character when room is cleared
    pub fn cleared_icon(&self) -> char {
        '.'
    }
}

/// State of a room in the dungeon
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomState {
    /// Not yet visible (fog of war)
    Hidden,
    /// Visible but not entered
    Revealed,
    /// Currently in this room
    Current,
    /// Completed/cleared
    Cleared,
}

/// A single room in the dungeon grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub room_type: RoomType,
    pub state: RoomState,
    /// Grid position (x, y)
    pub position: (usize, usize),
    /// Connected room positions (up, right, down, left)
    pub connections: [bool; 4],
}

impl Room {
    pub fn new(room_type: RoomType, position: (usize, usize)) -> Self {
        Self {
            room_type,
            state: RoomState::Hidden,
            position,
            connections: [false; 4],
        }
    }

    pub fn is_accessible(&self) -> bool {
        matches!(self.state, RoomState::Revealed | RoomState::Cleared)
    }
}

/// Direction indices for room connections
pub const DIR_UP: usize = 0;
pub const DIR_RIGHT: usize = 1;
pub const DIR_DOWN: usize = 2;
pub const DIR_LEFT: usize = 3;

/// Direction offsets: (dx, dy) for each direction
pub const DIR_OFFSETS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// Size tier for dungeons based on player progression
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DungeonSize {
    /// 5x5 grid, 8-12 rooms
    Small,
    /// 7x7 grid, 15-20 rooms
    Medium,
    /// 9x9 grid, 25-30 rooms
    Large,
    /// 11x11 grid, 35-45 rooms (Prestige 4+)
    Epic,
    /// 13x13 grid, 50-65 rooms (Prestige 6+)
    Legendary,
}

impl DungeonSize {
    /// Returns the grid dimensions for this size
    pub fn grid_size(&self) -> usize {
        match self {
            DungeonSize::Small => 5,
            DungeonSize::Medium => 7,
            DungeonSize::Large => 9,
            DungeonSize::Epic => 11,
            DungeonSize::Legendary => 13,
        }
    }

    /// Returns the target room count range (min, max)
    pub fn room_count_range(&self) -> (usize, usize) {
        match self {
            DungeonSize::Small => (8, 12),
            DungeonSize::Medium => (15, 20),
            DungeonSize::Large => (25, 30),
            DungeonSize::Epic => (35, 45),
            DungeonSize::Legendary => (50, 65),
        }
    }

    /// Returns the XP reward range for boss kill
    pub fn boss_xp_range(&self) -> (u64, u64) {
        match self {
            DungeonSize::Small => (1000, 1500),
            DungeonSize::Medium => (2000, 3000),
            DungeonSize::Large => (4000, 6000),
            DungeonSize::Epic => (8000, 12000),
            DungeonSize::Legendary => (15000, 25000),
        }
    }

    /// Returns the number of treasure rooms for this size
    pub fn treasure_room_count(&self) -> usize {
        match self {
            DungeonSize::Small => 1,
            DungeonSize::Medium => 2,
            DungeonSize::Large => 3,
            DungeonSize::Epic => 5,
            DungeonSize::Legendary => 8,
        }
    }

    /// Returns the rarity boost for treasure items (+1 to +3)
    pub fn treasure_rarity_boost(&self) -> u32 {
        match self {
            DungeonSize::Small | DungeonSize::Medium | DungeonSize::Large => 1,
            DungeonSize::Epic => 2,
            DungeonSize::Legendary => 3,
        }
    }

    /// Determines dungeon size based on level and prestige
    /// Returns the "expected" tier - actual size is rolled in a range
    fn base_tier(level: u32, prestige_rank: u32) -> u32 {
        // Base size from level
        let level_tier: u32 = if level >= DUNGEON_LEVEL_TIER_LARGE {
            2 // Large
        } else if level >= DUNGEON_LEVEL_TIER_MEDIUM {
            1 // Medium
        } else {
            0 // Small
        };

        // Prestige adds tiers
        let prestige_bonus = prestige_rank / DUNGEON_PRESTIGE_PER_SIZE_TIER;
        level_tier + prestige_bonus
    }

    /// Rolls a dungeon size based on progression
    /// Size can vary ±1 from the expected tier for variety
    pub fn roll_from_progression(level: u32, prestige_rank: u32) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let expected_tier = Self::base_tier(level, prestige_rank);

        // Roll variation: 20% chance -1, 60% chance same, 20% chance +1
        let roll: f64 = rng.gen();
        let variation: i32 = if roll < DUNGEON_SIZE_VARIATION_DOWN {
            -1 // Smaller dungeon
        } else if roll < DUNGEON_SIZE_VARIATION_UP {
            0 // Expected size
        } else {
            1 // Larger dungeon (lucky!)
        };

        let final_tier = (expected_tier as i32 + variation).max(0) as u32;

        match final_tier {
            0 => DungeonSize::Small,
            1 => DungeonSize::Medium,
            2 => DungeonSize::Large,
            3 => DungeonSize::Epic,
            _ => DungeonSize::Legendary,
        }
    }

    /// Determines dungeon size (deterministic, for testing)
    pub fn from_progression(level: u32, prestige_rank: u32) -> Self {
        let tier = Self::base_tier(level, prestige_rank);
        match tier {
            0 => DungeonSize::Small,
            1 => DungeonSize::Medium,
            2 => DungeonSize::Large,
            3 => DungeonSize::Epic,
            _ => DungeonSize::Legendary,
        }
    }
}

/// Main dungeon state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dungeon {
    pub size: DungeonSize,
    /// 2D grid of rooms (None = no room at this position)
    pub grid: Vec<Vec<Option<Room>>>,
    /// Current player position (x, y)
    pub player_position: (usize, usize),
    /// Position of the entrance
    pub entrance_position: (usize, usize),
    /// Position of the boss room
    pub boss_position: (usize, usize),
    /// Whether player has the big key
    pub has_key: bool,
    /// Timer for auto-exploration movement (seconds)
    pub move_timer: f64,
    /// Items collected during this dungeon run
    pub collected_items: Vec<crate::items::Item>,
    /// Total XP earned in this dungeon
    pub xp_earned: u64,
    /// Number of rooms cleared
    pub rooms_cleared: u32,
    /// Whether the current room's combat has been completed
    #[serde(default)]
    pub current_room_cleared: bool,
    /// Whether currently traveling through cleared rooms (for UI display)
    #[serde(skip)]
    pub is_traveling: bool,
    /// Zone where dungeon was discovered (for enemy scaling)
    #[serde(default = "default_dungeon_zone")]
    pub zone_id: u32,
}

fn default_dungeon_zone() -> u32 {
    1
}

impl Dungeon {
    pub fn new(size: DungeonSize) -> Self {
        let grid_size = size.grid_size();
        Self {
            size,
            grid: vec![vec![None; grid_size]; grid_size],
            player_position: (0, 0),
            entrance_position: (0, 0),
            boss_position: (0, 0),
            has_key: false,
            move_timer: 0.0,
            collected_items: Vec::new(),
            xp_earned: 0,
            rooms_cleared: 0,
            current_room_cleared: true, // Entrance starts cleared
            is_traveling: false,
            zone_id: 1,
        }
    }

    /// Get a room at position, if it exists
    pub fn get_room(&self, x: usize, y: usize) -> Option<&Room> {
        self.grid.get(y)?.get(x)?.as_ref()
    }

    /// Get a mutable room at position, if it exists
    pub fn get_room_mut(&mut self, x: usize, y: usize) -> Option<&mut Room> {
        self.grid.get_mut(y)?.get_mut(x)?.as_mut()
    }

    /// Get the current room the player is in
    pub fn current_room(&self) -> Option<&Room> {
        let (x, y) = self.player_position;
        self.get_room(x, y)
    }

    /// Get the current room mutably
    pub fn current_room_mut(&mut self) -> Option<&mut Room> {
        let (x, y) = self.player_position;
        self.get_room_mut(x, y)
    }

    /// Count total rooms in the dungeon
    pub fn room_count(&self) -> usize {
        self.grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|r| r.is_some())
            .count()
    }

    /// Check if the boss room is unlocked (player has key)
    pub fn is_boss_unlocked(&self) -> bool {
        self.has_key
    }

    /// Get adjacent room positions that exist and are connected
    pub fn get_connected_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();

        if let Some(room) = self.get_room(x, y) {
            for (dir_idx, &connected) in room.connections.iter().enumerate() {
                if connected {
                    let (dx, dy) = DIR_OFFSETS[dir_idx];
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && ny >= 0 {
                        let nx = nx as usize;
                        let ny = ny as usize;
                        if self.get_room(nx, ny).is_some() {
                            neighbors.push((nx, ny));
                        }
                    }
                }
            }
        }

        neighbors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dungeon_size_grid() {
        assert_eq!(DungeonSize::Small.grid_size(), 5);
        assert_eq!(DungeonSize::Medium.grid_size(), 7);
        assert_eq!(DungeonSize::Large.grid_size(), 9);
        assert_eq!(DungeonSize::Epic.grid_size(), 11);
        assert_eq!(DungeonSize::Legendary.grid_size(), 13);
    }

    #[test]
    fn test_dungeon_size_from_progression() {
        // Low level, no prestige = Small
        assert_eq!(DungeonSize::from_progression(10, 0), DungeonSize::Small);

        // Mid level, no prestige = Medium
        assert_eq!(DungeonSize::from_progression(50, 0), DungeonSize::Medium);

        // High level, no prestige = Large
        assert_eq!(DungeonSize::from_progression(100, 0), DungeonSize::Large);

        // Low level + prestige upgrades size
        assert_eq!(DungeonSize::from_progression(10, 2), DungeonSize::Medium);
        assert_eq!(DungeonSize::from_progression(10, 4), DungeonSize::Large);
        assert_eq!(DungeonSize::from_progression(10, 6), DungeonSize::Epic);
        assert_eq!(DungeonSize::from_progression(10, 8), DungeonSize::Legendary);

        // High level + high prestige = Legendary
        assert_eq!(
            DungeonSize::from_progression(100, 4),
            DungeonSize::Legendary
        );
    }

    #[test]
    fn test_dungeon_treasure_scaling() {
        assert_eq!(DungeonSize::Small.treasure_room_count(), 1);
        assert_eq!(DungeonSize::Legendary.treasure_room_count(), 8);

        assert_eq!(DungeonSize::Small.treasure_rarity_boost(), 1);
        assert_eq!(DungeonSize::Epic.treasure_rarity_boost(), 2);
        assert_eq!(DungeonSize::Legendary.treasure_rarity_boost(), 3);
    }

    #[test]
    fn test_dungeon_creation() {
        let dungeon = Dungeon::new(DungeonSize::Small);
        assert_eq!(dungeon.grid.len(), 5);
        assert_eq!(dungeon.grid[0].len(), 5);
        assert!(!dungeon.has_key);
        assert_eq!(dungeon.rooms_cleared, 0);
    }

    #[test]
    fn test_room_type_icons() {
        assert_eq!(RoomType::Entrance.icon(), 'E');
        assert_eq!(RoomType::Combat.icon(), 'C');
        assert_eq!(RoomType::Treasure.icon(), 'T');
        assert_eq!(RoomType::Elite.icon(), 'K');
        assert_eq!(RoomType::Boss.icon(), 'B');
    }

    #[test]
    fn test_cleared_icon_always_dot() {
        let types = [
            RoomType::Entrance,
            RoomType::Combat,
            RoomType::Treasure,
            RoomType::Elite,
            RoomType::Boss,
        ];
        for rt in types {
            assert_eq!(rt.cleared_icon(), '.');
        }
    }

    #[test]
    fn test_room_starts_hidden() {
        let room = Room::new(RoomType::Combat, (3, 4));
        assert_eq!(room.state, RoomState::Hidden);
        assert_eq!(room.position, (3, 4));
        assert_eq!(room.room_type, RoomType::Combat);
        assert_eq!(room.connections, [false; 4]);
    }

    #[test]
    fn test_room_is_accessible() {
        let mut room = Room::new(RoomType::Combat, (0, 0));

        // Hidden is not accessible
        assert!(!room.is_accessible());

        // Revealed is accessible
        room.state = RoomState::Revealed;
        assert!(room.is_accessible());

        // Current is not accessible (player is already there)
        room.state = RoomState::Current;
        assert!(!room.is_accessible());

        // Cleared is accessible (can revisit)
        room.state = RoomState::Cleared;
        assert!(room.is_accessible());
    }

    #[test]
    fn test_dungeon_size_room_count_ranges() {
        // All ranges should have min < max
        let sizes = [
            DungeonSize::Small,
            DungeonSize::Medium,
            DungeonSize::Large,
            DungeonSize::Epic,
            DungeonSize::Legendary,
        ];
        for size in sizes {
            let (min, max) = size.room_count_range();
            assert!(
                min < max,
                "{:?} room count range: min ({min}) >= max ({max})",
                size
            );
            // Room count should fit within grid
            let grid = size.grid_size();
            assert!(
                max <= grid * grid,
                "{:?} max rooms ({max}) exceeds grid capacity ({})",
                size,
                grid * grid
            );
        }
    }

    #[test]
    fn test_dungeon_size_boss_xp_ranges() {
        let sizes = [
            DungeonSize::Small,
            DungeonSize::Medium,
            DungeonSize::Large,
            DungeonSize::Epic,
            DungeonSize::Legendary,
        ];
        let mut prev_max = 0;
        for size in sizes {
            let (min, max) = size.boss_xp_range();
            assert!(min < max, "{:?} XP range: min ({min}) >= max ({max})", size);
            // Each tier should give more XP than the last
            assert!(
                min > prev_max,
                "{:?} XP min ({min}) should exceed previous tier max ({prev_max})",
                size
            );
            prev_max = max;
        }
    }

    #[test]
    fn test_dungeon_size_treasure_rooms_increase() {
        let sizes = [
            DungeonSize::Small,
            DungeonSize::Medium,
            DungeonSize::Large,
            DungeonSize::Epic,
            DungeonSize::Legendary,
        ];
        let mut prev = 0;
        for size in sizes {
            let count = size.treasure_room_count();
            assert!(
                count >= prev,
                "{:?} treasure count ({count}) < previous ({prev})",
                size
            );
            prev = count;
        }
    }

    #[test]
    fn test_dungeon_get_room_out_of_bounds() {
        let dungeon = Dungeon::new(DungeonSize::Small);
        // Grid is 5x5, so (5,5) is out of bounds
        assert!(dungeon.get_room(5, 5).is_none());
        assert!(dungeon.get_room(100, 100).is_none());
        // (0,0) exists in grid but no room placed yet
        assert!(dungeon.get_room(0, 0).is_none());
    }

    #[test]
    fn test_dungeon_room_count_empty() {
        let dungeon = Dungeon::new(DungeonSize::Small);
        assert_eq!(dungeon.room_count(), 0);
    }

    #[test]
    fn test_dungeon_room_count_with_rooms() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        dungeon.grid[0][0] = Some(Room::new(RoomType::Entrance, (0, 0)));
        dungeon.grid[1][1] = Some(Room::new(RoomType::Combat, (1, 1)));
        assert_eq!(dungeon.room_count(), 2);
    }

    #[test]
    fn test_dungeon_is_boss_unlocked() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        assert!(!dungeon.is_boss_unlocked());

        dungeon.has_key = true;
        assert!(dungeon.is_boss_unlocked());
    }

    #[test]
    fn test_dungeon_current_room() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        dungeon.player_position = (2, 3);

        // No room placed yet
        assert!(dungeon.current_room().is_none());

        // Place a room at player position
        dungeon.grid[3][2] = Some(Room::new(RoomType::Combat, (2, 3)));
        let room = dungeon.current_room().unwrap();
        assert_eq!(room.room_type, RoomType::Combat);
        assert_eq!(room.position, (2, 3));
    }

    #[test]
    fn test_dungeon_current_room_mut() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);
        dungeon.player_position = (1, 1);
        dungeon.grid[1][1] = Some(Room::new(RoomType::Treasure, (1, 1)));

        // Mutate the room state
        dungeon.current_room_mut().unwrap().state = RoomState::Cleared;
        assert_eq!(dungeon.current_room().unwrap().state, RoomState::Cleared);
    }

    #[test]
    fn test_dungeon_get_connected_neighbors() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);

        // Place rooms at (1,1) and (2,1), connect them (right/left)
        let mut room_a = Room::new(RoomType::Entrance, (1, 1));
        room_a.connections[DIR_RIGHT] = true;
        dungeon.grid[1][1] = Some(room_a);

        let mut room_b = Room::new(RoomType::Combat, (2, 1));
        room_b.connections[DIR_LEFT] = true;
        dungeon.grid[1][2] = Some(room_b);

        let neighbors = dungeon.get_connected_neighbors(1, 1);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0], (2, 1));
    }

    #[test]
    fn test_dungeon_get_connected_neighbors_no_room() {
        let dungeon = Dungeon::new(DungeonSize::Small);
        // No room at (0,0), should return empty
        let neighbors = dungeon.get_connected_neighbors(0, 0);
        assert!(neighbors.is_empty());
    }

    #[test]
    fn test_dungeon_get_connected_neighbors_edge() {
        let mut dungeon = Dungeon::new(DungeonSize::Small);

        // Room at (0,0) with a connection pointing up (out of bounds)
        let mut room = Room::new(RoomType::Entrance, (0, 0));
        room.connections[DIR_UP] = true; // Points to (0, -1) — out of bounds
        room.connections[DIR_LEFT] = true; // Points to (-1, 0) — out of bounds
        dungeon.grid[0][0] = Some(room);

        let neighbors = dungeon.get_connected_neighbors(0, 0);
        assert!(neighbors.is_empty());
    }

    #[test]
    fn test_dungeon_size_from_progression_tier_clamping() {
        // Very high prestige should cap at Legendary, not panic
        assert_eq!(
            DungeonSize::from_progression(100, 100),
            DungeonSize::Legendary
        );
    }
}
