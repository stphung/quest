use crate::challenges::chess::ChessStats;
use crate::challenges::menu::ChallengeMenu;
use crate::challenges::ActiveMinigame;
use crate::challenges::MinigameWinInfo;
use crate::character::attributes::Attributes;
use crate::combat::types::CombatState;
use crate::dungeon::types::Dungeon;
use crate::fishing::types::{FishingSession, FishingState};
use crate::items::equipment::Equipment;
use crate::items::types::Rarity;
use crate::zones::ZoneProgression;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A recently gained item or fish for display in the Loot panel
#[derive(Debug, Clone)]
pub struct RecentDrop {
    pub name: String,
    pub rarity: Rarity,
    pub equipped: bool,
    pub icon: &'static str,
    /// Equipment slot name (e.g. "Weapon", "Armor"), empty for non-equipment
    pub slot: String,
    /// Stat summary (e.g. "+8 STR +3 DEX +Crit"), empty for non-equipment
    pub stats: String,
}

/// Max number of recent drops to track
const MAX_RECENT_DROPS: usize = 10;

impl GameState {
    /// Record a recent gain (item drop, fish catch, etc.)
    pub fn add_recent_drop(
        &mut self,
        name: String,
        rarity: Rarity,
        equipped: bool,
        icon: &'static str,
        slot: String,
        stats: String,
    ) {
        if self.recent_drops.len() >= MAX_RECENT_DROPS {
            self.recent_drops.pop_back();
        }
        self.recent_drops.push_front(RecentDrop {
            name,
            rarity,
            equipped,
            icon,
            slot,
            stats,
        });
    }
}

/// Main game state containing all player progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub character_id: String,
    pub character_name: String,
    pub character_level: u32,
    pub character_xp: u64,
    pub attributes: Attributes,
    pub prestige_rank: u32,
    pub total_prestige_count: u64,
    pub last_save_time: i64,
    pub play_time_seconds: u64,
    pub combat_state: CombatState,
    pub equipment: Equipment,
    /// Active dungeon exploration (None when not in a dungeon)
    #[serde(default)]
    pub active_dungeon: Option<Dungeon>,
    /// Persistent fishing progression state
    #[serde(default)]
    pub fishing: FishingState,
    /// Active fishing session (transient, not saved)
    #[serde(skip)]
    #[allow(dead_code)]
    pub active_fishing: Option<FishingSession>,
    /// Zone progression state
    #[serde(default)]
    pub zone_progression: ZoneProgression,
    /// Generic challenge menu (transient, not saved)
    #[serde(skip)]
    pub challenge_menu: ChallengeMenu,
    /// Persistent chess stats (survives prestige, saved to disk)
    #[serde(default)]
    pub chess_stats: ChessStats,
    /// Active challenge minigame (transient, not saved)
    #[serde(skip)]
    pub active_minigame: Option<ActiveMinigame>,
    /// Session kill count (transient, not saved)
    #[serde(skip)]
    pub session_kills: u64,
    /// Recent item drops for display (transient, not saved)
    #[serde(skip)]
    pub recent_drops: VecDeque<RecentDrop>,
    /// Last minigame win info for achievement tracking (transient, not saved)
    #[serde(skip)]
    pub last_minigame_win: Option<MinigameWinInfo>,
}

impl GameState {
    /// Creates a new game state with default values
    pub fn new(character_name: String, current_time: i64) -> Self {
        use uuid::Uuid;

        let attributes = Attributes::new();
        let combat_state = CombatState::new(50); // Base HP
        let equipment = Equipment::new();

        Self {
            character_id: Uuid::new_v4().to_string(),
            character_name,
            character_level: 1,
            character_xp: 0,
            attributes,
            prestige_rank: 0,
            total_prestige_count: 0,
            last_save_time: current_time,
            play_time_seconds: 0,
            combat_state,
            equipment,
            active_dungeon: None,
            fishing: FishingState::default(),
            active_fishing: None,
            zone_progression: ZoneProgression::new(),
            challenge_menu: ChallengeMenu::new(),
            chess_stats: ChessStats::default(),
            active_minigame: None,
            session_kills: 0,
            recent_drops: VecDeque::with_capacity(5),
            last_minigame_win: None,
        }
    }

    /// Returns true if the player is currently in a dungeon
    #[allow(dead_code)]
    pub fn is_in_dungeon(&self) -> bool {
        self.active_dungeon.is_some()
    }

    pub fn get_attribute_cap(&self) -> u32 {
        super::balance::attribute_cap(self.prestige_rank)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::attributes::AttributeType;

    #[test]
    fn test_new_game_state() {
        let current_time = 1234567890;
        let game_state = GameState::new("Test Hero".to_string(), current_time);

        assert_eq!(game_state.character_level, 1);
        assert_eq!(game_state.character_xp, 0);
        assert_eq!(game_state.prestige_rank, 0);
        assert_eq!(game_state.total_prestige_count, 0);
        assert_eq!(game_state.last_save_time, current_time);
        assert_eq!(game_state.play_time_seconds, 0);

        // Verify all attributes start at 10
        for attr in AttributeType::all() {
            assert_eq!(game_state.attributes.get(attr), 10);
        }
    }

    #[test]
    fn test_attribute_cap() {
        let mut game_state = GameState::new("Test Hero".to_string(), 0);

        // Prestige 0: cap 20
        assert_eq!(game_state.get_attribute_cap(), 20);

        // Prestige 1: cap 25
        game_state.prestige_rank = 1;
        assert_eq!(game_state.get_attribute_cap(), 25);

        // Prestige 2: cap 30
        game_state.prestige_rank = 2;
        assert_eq!(game_state.get_attribute_cap(), 30);
    }

    #[test]
    fn test_character_id_uniqueness() {
        let state1 = GameState::new("Hero1".to_string(), 0);
        let state2 = GameState::new("Hero2".to_string(), 0);

        // Each character should have a unique ID
        assert_ne!(state1.character_id, state2.character_id);
        // IDs should be valid UUIDs (36 chars with hyphens)
        assert_eq!(state1.character_id.len(), 36);
        assert_eq!(state2.character_id.len(), 36);
    }

    #[test]
    fn test_is_in_dungeon() {
        let mut game_state = GameState::new("Test Hero".to_string(), 0);

        // Initially not in a dungeon
        assert!(!game_state.is_in_dungeon());

        // Set an active dungeon
        game_state.active_dungeon = Some(crate::dungeon::generation::generate_dungeon(1, 0));

        assert!(game_state.is_in_dungeon());
    }

    #[test]
    fn test_character_name_stored() {
        let game_state = GameState::new("My Hero Name".to_string(), 0);
        assert_eq!(game_state.character_name, "My Hero Name");
    }

    #[test]
    fn test_combat_state_initialized() {
        let game_state = GameState::new("Test Hero".to_string(), 0);

        // Combat state should be initialized with base HP
        assert_eq!(game_state.combat_state.player_max_hp, 50);
        assert_eq!(game_state.combat_state.player_current_hp, 50);
        assert!(game_state.combat_state.current_enemy.is_none());
        assert!(!game_state.combat_state.is_regenerating);
    }

    #[test]
    fn test_equipment_starts_empty() {
        let game_state = GameState::new("Test Hero".to_string(), 0);

        assert!(game_state.equipment.weapon.is_none());
        assert!(game_state.equipment.armor.is_none());
        assert!(game_state.equipment.helmet.is_none());
        assert!(game_state.equipment.gloves.is_none());
        assert!(game_state.equipment.boots.is_none());
        assert!(game_state.equipment.amulet.is_none());
        assert!(game_state.equipment.ring.is_none());
    }

    #[test]
    fn test_zone_progression_starts_at_zone_1() {
        let game_state = GameState::new("Test Hero".to_string(), 0);

        assert_eq!(game_state.zone_progression.current_zone_id, 1);
        assert_eq!(game_state.zone_progression.current_subzone_id, 1);
        assert!(!game_state.zone_progression.fighting_boss);
    }

    #[test]
    fn test_fishing_state_initialized() {
        let game_state = GameState::new("Test Hero".to_string(), 0);

        assert_eq!(game_state.fishing.rank, 1); // Fishing starts at rank 1
        assert_eq!(game_state.fishing.total_fish_caught, 0);
        assert!(game_state.active_fishing.is_none());
    }

    #[test]
    fn test_attribute_cap_high_prestige() {
        let mut game_state = GameState::new("Test Hero".to_string(), 0);

        // Prestige 10: cap should be 20 + (10 * 5) = 70
        game_state.prestige_rank = 10;
        assert_eq!(game_state.get_attribute_cap(), 70);

        // Prestige 20: cap should be 20 + (20 * 5) = 120
        game_state.prestige_rank = 20;
        assert_eq!(game_state.get_attribute_cap(), 120);
    }

    #[test]
    fn test_add_recent_drop_single() {
        let mut gs = GameState::new("Hero".to_string(), 0);
        assert!(gs.recent_drops.is_empty());

        gs.add_recent_drop(
            "Iron Sword".to_string(),
            Rarity::Common,
            true,
            "⚔",
            "Weapon".to_string(),
            "+2 STR".to_string(),
        );

        assert_eq!(gs.recent_drops.len(), 1);
        assert_eq!(gs.recent_drops[0].name, "Iron Sword");
        assert_eq!(gs.recent_drops[0].rarity, Rarity::Common);
        assert!(gs.recent_drops[0].equipped);
        assert_eq!(gs.recent_drops[0].slot, "Weapon");
        assert_eq!(gs.recent_drops[0].stats, "+2 STR");
    }

    #[test]
    fn test_add_recent_drop_fifo_order() {
        let mut gs = GameState::new("Hero".to_string(), 0);

        gs.add_recent_drop(
            "First".to_string(),
            Rarity::Common,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );
        gs.add_recent_drop(
            "Second".to_string(),
            Rarity::Rare,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );
        gs.add_recent_drop(
            "Third".to_string(),
            Rarity::Epic,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );

        // Most recent should be at front
        assert_eq!(gs.recent_drops.len(), 3);
        assert_eq!(gs.recent_drops[0].name, "Third");
        assert_eq!(gs.recent_drops[1].name, "Second");
        assert_eq!(gs.recent_drops[2].name, "First");
    }

    #[test]
    fn test_add_recent_drop_caps_at_max() {
        let mut gs = GameState::new("Hero".to_string(), 0);

        // Fill to the cap (MAX_RECENT_DROPS = 10)
        for i in 0..10 {
            gs.add_recent_drop(
                format!("Item {i}"),
                Rarity::Common,
                false,
                "",
                "".to_string(),
                "".to_string(),
            );
        }
        assert_eq!(gs.recent_drops.len(), 10);

        // Adding one more should evict the oldest
        gs.add_recent_drop(
            "Overflow".to_string(),
            Rarity::Legendary,
            true,
            "",
            "".to_string(),
            "".to_string(),
        );
        assert_eq!(gs.recent_drops.len(), 10);
        assert_eq!(gs.recent_drops[0].name, "Overflow");
        // "Item 0" (the oldest) should have been evicted
        assert!(gs.recent_drops.iter().all(|d| d.name != "Item 0"));
        // "Item 1" should still be present as the last element
        assert_eq!(gs.recent_drops[9].name, "Item 1");
    }

    #[test]
    fn test_add_recent_drop_at_exact_cap_boundary() {
        let mut gs = GameState::new("Hero".to_string(), 0);

        // Add exactly MAX_RECENT_DROPS items
        for i in 0..10 {
            gs.add_recent_drop(
                format!("Item {i}"),
                Rarity::Common,
                false,
                "",
                "".to_string(),
                "".to_string(),
            );
        }
        assert_eq!(gs.recent_drops.len(), 10);

        // Add two more, should still be capped at 10
        gs.add_recent_drop(
            "Extra1".to_string(),
            Rarity::Common,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );
        gs.add_recent_drop(
            "Extra2".to_string(),
            Rarity::Common,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );
        assert_eq!(gs.recent_drops.len(), 10);
        assert_eq!(gs.recent_drops[0].name, "Extra2");
        assert_eq!(gs.recent_drops[1].name, "Extra1");
    }

    #[test]
    fn test_serialization_round_trip_preserves_persistent_fields() {
        let mut gs = GameState::new("Serde Hero".to_string(), 42);
        gs.character_level = 15;
        gs.character_xp = 5000;
        gs.prestige_rank = 3;
        gs.total_prestige_count = 5;
        gs.play_time_seconds = 3600;
        gs.attributes.set(AttributeType::Strength, 18);

        let json = serde_json::to_string(&gs).unwrap();
        let loaded: GameState = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.character_name, "Serde Hero");
        assert_eq!(loaded.character_level, 15);
        assert_eq!(loaded.character_xp, 5000);
        assert_eq!(loaded.prestige_rank, 3);
        assert_eq!(loaded.total_prestige_count, 5);
        assert_eq!(loaded.play_time_seconds, 3600);
        assert_eq!(loaded.last_save_time, 42);
        assert_eq!(loaded.attributes.get(AttributeType::Strength), 18);
    }

    #[test]
    fn test_serialization_skips_transient_fields() {
        let mut gs = GameState::new("Hero".to_string(), 0);
        gs.session_kills = 999;
        gs.add_recent_drop(
            "Sword".to_string(),
            Rarity::Rare,
            true,
            "",
            "".to_string(),
            "".to_string(),
        );

        let json = serde_json::to_string(&gs).unwrap();
        let loaded: GameState = serde_json::from_str(&json).unwrap();

        // Transient fields should be at default values after deserialization
        assert_eq!(loaded.session_kills, 0);
        assert!(loaded.recent_drops.is_empty());
        assert!(loaded.active_fishing.is_none());
        assert!(loaded.active_minigame.is_none());
        assert!(loaded.last_minigame_win.is_none());
    }

    #[test]
    fn test_serialization_default_fields_from_old_json() {
        // Simulate loading from an older save that lacks optional fields
        let minimal_json = serde_json::json!({
            "character_id": "test-id",
            "character_name": "Old Hero",
            "character_level": 5,
            "character_xp": 100,
            "attributes": { "values": [10, 10, 10, 10, 10, 10] },
            "prestige_rank": 0,
            "total_prestige_count": 0,
            "last_save_time": 0,
            "play_time_seconds": 0,
            "combat_state": {
                "player_max_hp": 50,
                "player_current_hp": 50,
                "current_enemy": null,
                "is_regenerating": false,
                "regen_timer": 0.0,
                "attack_timer": 0.0,
                "kills_in_subzone": 0,
                "fighting_boss": false,
                "total_kills": 0,
                "combat_log": []
            },
            "equipment": {
                "weapon": null,
                "armor": null,
                "helmet": null,
                "gloves": null,
                "boots": null,
                "amulet": null,
                "ring": null
            }
        });

        let loaded: GameState = serde_json::from_value(minimal_json).unwrap();

        // #[serde(default)] fields should get their defaults
        assert!(loaded.active_dungeon.is_none());
        assert_eq!(loaded.fishing.rank, 1);
        assert_eq!(loaded.zone_progression.current_zone_id, 1);
        assert_eq!(loaded.chess_stats.games_played, 0);
    }
}
