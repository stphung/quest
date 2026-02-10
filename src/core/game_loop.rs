//! Shared game loop trait for game and simulator.
//!
//! This trait provides a unified interface for executing game ticks,
//! allowing both the interactive game and simulator to share core logic.

use crate::core::game_state::GameState;
use crate::fishing::types::{CaughtFish, FishRarity};
use crate::items::Item;
use rand::Rng;

/// Represents a fish catch result for simulation tracking.
#[derive(Debug, Clone)]
pub struct FishCatch {
    pub fish_name: String,
    pub rarity: FishRarity,
    pub xp_gained: u64,
    pub item_dropped: Option<Item>,
}

/// Result of one game tick - captures everything that happened.
///
/// This struct is used to communicate game events to the UI layer,
/// allowing separation between game logic and visual presentation.
#[derive(Debug, Clone, Default)]
pub struct TickResult {
    /// Combat happened this tick
    pub had_combat: bool,
    /// Player won the fight (enemy killed)
    pub player_won: bool,
    /// Player died (respawns)
    pub player_died: bool,
    /// Was fighting a boss
    pub was_boss: bool,
    /// Damage dealt by player this tick
    pub damage_dealt: u32,
    /// Whether the attack was a critical hit
    pub was_crit: bool,
    /// Damage taken by player this tick
    pub damage_taken: u32,
    /// Name of the enemy fought/killed
    pub enemy_name: Option<String>,
    /// XP gained this tick
    pub xp_gained: u64,
    /// Player leveled up
    pub leveled_up: bool,
    /// New level (if leveled up)
    pub new_level: u32,
    /// Advanced to new zone
    pub zone_advanced: bool,
    /// New zone (if advanced)
    pub new_zone: u32,
    /// Item dropped
    pub loot_dropped: Option<Item>,
    /// Item was equipped (upgrade)
    pub loot_equipped: bool,
    /// Can prestige now
    pub can_prestige: bool,
    /// At max zone for current prestige
    pub at_prestige_wall: bool,

    // ── Fishing/Dungeon tracking ──────────────────────────────────────────
    /// Discovered a dungeon this tick (2% per kill)
    pub discovered_dungeon: bool,
    /// Discovered a fishing spot this tick (5% per tick when idle)
    pub discovered_fishing: bool,
    /// Fish caught this tick (if any)
    pub fishing_catch: Option<FishCatch>,
    /// Cleared a dungeon room this tick
    pub dungeon_room_cleared: bool,
    /// Completed a dungeon (boss defeated)
    pub dungeon_completed: bool,
    /// XP earned from dungeon this tick
    pub dungeon_xp_gained: u64,
    /// Items found in dungeon this tick
    pub dungeon_items_found: Vec<Item>,
}

/// Core game loop trait - implemented by game engine.
///
/// This trait abstracts the game tick execution, allowing:
/// - Real game to run with UI updates and visual effects
/// - Simulator to run thousands of ticks for testing/balancing
pub trait GameLoop {
    /// Execute one game tick. Returns what happened.
    fn tick(&mut self, rng: &mut impl Rng) -> TickResult;

    /// Perform prestige reset.
    fn prestige(&mut self);

    /// Get current game state (read-only).
    fn state(&self) -> &GameState;

    /// Get current game state (mutable).
    fn state_mut(&mut self) -> &mut GameState;

    /// Check if can prestige.
    fn can_prestige(&self) -> bool;

    /// Check if at max zone for prestige rank.
    fn at_prestige_wall(&self) -> bool;
}
