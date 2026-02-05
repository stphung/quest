use crate::challenges::chess::ChessStats;
use crate::challenges::menu::ChallengeMenu;
use crate::challenges::ActiveMinigame;
use crate::character::attributes::Attributes;
use crate::combat::types::CombatState;
use crate::dungeon::types::Dungeon;
use crate::fishing::types::{FishingSession, FishingState};
use crate::items::equipment::Equipment;
use crate::zones::ZoneProgression;
use serde::{Deserialize, Serialize};

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
        }
    }

    /// Returns true if the player is currently in a dungeon
    #[allow(dead_code)]
    pub fn is_in_dungeon(&self) -> bool {
        self.active_dungeon.is_some()
    }

    pub fn get_attribute_cap(&self) -> u32 {
        20 + (self.prestige_rank * 5)
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
}
