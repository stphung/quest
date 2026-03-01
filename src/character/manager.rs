use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// Re-export for backward API compatibility
pub use super::name_validation::{sanitize_name, validate_name};

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct CharacterSaveData {
    pub(super) version: u32,
    pub(super) character_id: String,
    pub(super) character_name: String,
    pub(super) character_level: u32,
    pub(super) character_xp: u64,
    pub(super) attributes: super::attributes::Attributes,
    pub(super) prestige_rank: u32,
    pub(super) total_prestige_count: u64,
    pub(super) last_save_time: i64,
    pub(super) play_time_seconds: u64,
    pub(super) combat_state: crate::combat::CombatState,
    pub(super) equipment: crate::items::Equipment,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) active_dungeon: Option<crate::dungeon::Dungeon>,
    #[serde(default)]
    pub(super) fishing: crate::fishing::FishingState,
    #[serde(default)]
    pub(super) zone_progression: crate::zones::ZoneProgression,
    #[serde(default)]
    pub(super) stormglass: u64,
    #[serde(default)]
    pub(super) stormglass_discovered: bool,
    #[serde(default)]
    pub(super) storm_sigils: crate::stormglass::sigils::StormSigils,
    #[serde(default)]
    pub(super) ascension_level: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CharacterInfo {
    pub character_id: String,
    pub character_name: String,
    pub filename: String,
    pub character_level: u32,
    pub prestige_rank: u32,
    pub play_time_seconds: u64,
    pub last_save_time: i64,
    pub attributes: super::attributes::Attributes,
    pub equipment: crate::items::Equipment,
    pub is_corrupted: bool,
}

pub struct CharacterManager {
    pub(super) quest_dir: PathBuf,
}

impl CharacterManager {
    pub fn new() -> io::Result<Self> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Could not determine home directory",
            )
        })?;

        let quest_dir = home_dir.join(".quest");
        fs::create_dir_all(&quest_dir)?;

        Ok(Self { quest_dir })
    }

    /// Creates a CharacterManager with a custom directory path.
    /// Useful for testing with isolated temp directories.
    #[cfg(test)]
    pub fn with_dir(quest_dir: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&quest_dir)?;
        Ok(Self { quest_dir })
    }
}

/// Account-level JSON files that are not character saves
pub(super) const ACCOUNT_FILES: &[&str] = &[
    "haven.json",
    "achievements.json",
    "enhancement.json",
    "deep.json",
    ".cloud.json",
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Creates a test GameState with the given name for persistence tests.
    fn make_test_state(name: &str) -> crate::core::game_state::GameState {
        use crate::character::attributes::Attributes;
        use crate::combat::CombatState;
        use crate::core::game_state::GameState;
        use crate::items::Equipment;

        let character_id = format!("test-{}", sanitize_name(name));
        let player = crate::core::player_identity::PlayerIdentity {
            character_id: character_id.clone(),
            character_name: name.to_string(),
            character_level: 1,
            character_xp: 0,
            attributes: Attributes::new(),
            prestige_rank: 0,
            total_prestige_count: 0,
        };

        GameState {
            character_id,
            character_name: name.to_string(),
            character_level: 1,
            character_xp: 0,
            attributes: Attributes::new(),
            prestige_rank: 0,
            total_prestige_count: 0,
            last_save_time: 0,
            play_time_seconds: 0,
            combat_state: CombatState::new(50),
            equipment: Equipment::new(),
            active_dungeon: None,
            fishing: crate::fishing::FishingState::default(),
            active_fishing: None,
            zone_progression: crate::zones::ZoneProgression::default(),
            challenge_menu: crate::challenges::menu::ChallengeMenu::new(),
            chess_stats: crate::challenges::chess::ChessStats::default(),
            active_minigame: None,
            session_kills: 0,
            consecutive_deaths: 0,
            recent_drops: std::collections::VecDeque::new(),
            ticker: crate::core::game_state::Ticker::new(),
            last_minigame_win: None,
            cached_derived_stats: Default::default(),
            cached_prestige_bonuses: Default::default(),
            derived_stats_dirty: true,
            xp_rate_samples: std::collections::VecDeque::new(),
            xp_this_second: 0,
            combat_seconds_this_tick: false,
            game_over_shown_at: None,
            cached_power_rating: 0.0,
            cached_fracture_zone_cap: 0,
            stormglass: 0,
            stormglass_discovered: false,
            storm_sigils: crate::stormglass::sigils::StormSigils::new(),
            ascension_level: 0,
            chrono_surge_active: false,
            debug_force_overcharge: false,
            player,
            combat_ctx: crate::core::combat_context::CombatContext {
                combat_state: CombatState::new(50),
                equipment: Equipment::new(),
                zone_progression: crate::zones::ZoneProgression::new(),
                active_dungeon: None,
                session_kills: 0,
                consecutive_deaths: 0,
            },
            prog: crate::core::progression_state::ProgressionState {
                fishing: crate::fishing::types::FishingState::default(),
                active_fishing: None,
                stormglass: 0,
                stormglass_discovered: false,
                storm_sigils: crate::stormglass::sigils::StormSigils::new(),
                challenge_menu: crate::challenges::menu::ChallengeMenu::new(),
                chess_stats: crate::challenges::chess::ChessStats::default(),
                active_minigame: None,
                last_minigame_win: None,
            },
            sess: crate::core::session_state::SessionState::default(),
        }
    }

    /// Creates a CharacterManager backed by an isolated temp directory.
    fn temp_manager() -> (CharacterManager, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let manager =
            CharacterManager::with_dir(dir.path().to_path_buf()).expect("Failed to create manager");
        (manager, dir)
    }

    #[test]
    fn test_character_manager_new() {
        let (manager, _dir) = temp_manager();
        assert!(manager.quest_dir.exists());
    }

    #[test]
    fn test_save_and_load_character() {
        let (manager, _dir) = temp_manager();

        let mut state = make_test_state("TestHero");
        state.character_level = 10;
        state.character_xp = 5000;
        state.prestige_rank = 2;
        state.total_prestige_count = 2;

        // Save character
        manager.save_character(&state).expect("Failed to save");

        // Verify file exists
        let filename = format!("{}.json", sanitize_name(&state.character_name));
        let filepath = manager.quest_dir.join(&filename);
        assert!(filepath.exists());

        // Load character
        let loaded = manager.load_character(&filename).expect("Failed to load");
        assert_eq!(loaded.character_name, "TestHero");
        assert_eq!(loaded.character_level, 10);
    }

    #[test]
    fn test_list_characters() {
        let (manager, dir) = temp_manager();

        let mut char1 = make_test_state("ListTest1");
        char1.character_level = 10;

        let mut char2 = make_test_state("ListTest2");
        char2.character_level = 15;

        manager.save_character(&char1).unwrap();
        manager.save_character(&char2).unwrap();

        // Set deterministic timestamps to avoid timing-dependent ordering.
        // save_character() uses Utc::now() which can produce equal timestamps
        // when both saves happen within the same second.
        let path1 = dir.path().join("listtest1.json");
        let mut json1: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path1).unwrap()).unwrap();
        json1["last_save_time"] = serde_json::json!(1000);
        std::fs::write(&path1, serde_json::to_string_pretty(&json1).unwrap()).unwrap();

        let path2 = dir.path().join("listtest2.json");
        let mut json2: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path2).unwrap()).unwrap();
        json2["last_save_time"] = serde_json::json!(2000);
        std::fs::write(&path2, serde_json::to_string_pretty(&json2).unwrap()).unwrap();

        // Isolated temp dir means only our test files are present
        let list = manager.list_characters().expect("Failed to list");
        assert_eq!(list.len(), 2);

        // Verify sorted by last_played (most recent first)
        assert_eq!(list[0].character_name, "ListTest2");
        assert_eq!(list[1].character_name, "ListTest1");
    }

    #[test]
    fn test_delete_character() {
        let (manager, _dir) = temp_manager();
        let state = make_test_state("ToDelete");

        manager.save_character(&state).unwrap();

        let filename = "todelete.json";
        assert!(manager.quest_dir.join(filename).exists());

        manager.delete_character(filename).expect("Delete failed");
        assert!(!manager.quest_dir.join(filename).exists());
    }

    #[test]
    fn test_rename_character() {
        let (manager, _dir) = temp_manager();
        let state = make_test_state("OldName");

        manager.save_character(&state).unwrap();

        manager
            .rename_character("oldname.json", "NewName".to_string())
            .expect("Rename failed");

        // Old file should not exist
        assert!(!manager.quest_dir.join("oldname.json").exists());

        // New file should exist
        assert!(manager.quest_dir.join("newname.json").exists());

        // Load and verify name updated
        let loaded = manager.load_character("newname.json").unwrap();
        assert_eq!(loaded.character_name, "NewName");
    }

    #[test]
    fn test_load_nonexistent_character() {
        let (manager, _dir) = temp_manager();

        let result = manager.load_character("nonexistent_character_12345.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_nonexistent_character() {
        let (manager, _dir) = temp_manager();

        let result = manager.delete_character("nonexistent_delete_test.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_with_invalid_name() {
        let (manager, _dir) = temp_manager();
        let state = make_test_state("RenameTest");

        manager.save_character(&state).unwrap();

        // Try to rename with invalid characters
        let result = manager.rename_character("renametest.json", "Invalid@Name!".to_string());
        assert!(result.is_err());

        // Try to rename with empty name
        let result = manager.rename_character("renametest.json", "".to_string());
        assert!(result.is_err());

        // Try to rename with too long name
        let result = manager.rename_character(
            "renametest.json",
            "ThisNameIsWayTooLongForTheLimit".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_corrupted_file_handling() {
        let (manager, _dir) = temp_manager();

        // Write invalid JSON to a file
        let filepath = manager.quest_dir.join("corrupted_test.json");
        fs::write(&filepath, "{ invalid json }").unwrap();

        // Load should fail
        let result = manager.load_character("corrupted_test.json");
        assert!(result.is_err());

        // List should show as corrupted
        let list = manager.list_characters().unwrap();
        let corrupted = list.iter().find(|c| c.filename == "corrupted_test.json");
        assert!(corrupted.is_some());
        assert!(corrupted.unwrap().is_corrupted);
    }

    #[test]
    fn test_character_data_integrity() {
        use crate::character::attributes::AttributeType;

        let (manager, _dir) = temp_manager();

        // Create a character with specific values
        let mut state = make_test_state("IntegrityTest");
        state.character_id = "integrity-test-id".to_string();
        state.character_level = 25;
        state.character_xp = 12345;
        state.prestige_rank = 3;
        state.total_prestige_count = 5;
        state.play_time_seconds = 9999;
        state.attributes.set(AttributeType::Strength, 15);
        state.attributes.set(AttributeType::Dexterity, 18);

        manager.save_character(&state).unwrap();

        // Load and verify all values preserved
        let loaded = manager.load_character("integritytest.json").unwrap();

        assert_eq!(loaded.character_id, "integrity-test-id");
        assert_eq!(loaded.character_name, "IntegrityTest");
        assert_eq!(loaded.character_level, 25);
        assert_eq!(loaded.character_xp, 12345);
        assert_eq!(loaded.prestige_rank, 3);
        assert_eq!(loaded.total_prestige_count, 5);
        assert_eq!(loaded.play_time_seconds, 9999);
        assert_eq!(loaded.attributes.get(AttributeType::Strength), 15);
        assert_eq!(loaded.attributes.get(AttributeType::Dexterity), 18);
    }

    #[test]
    fn test_rename_nonexistent_character() {
        let (manager, _dir) = temp_manager();

        let result = manager.rename_character("does_not_exist.json", "NewName".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_json_with_extra_fields_backward_compat() {
        let (manager, _dir) = temp_manager();

        let json_with_extra = r#"{
            "version": 2,
            "character_id": "test-id",
            "character_name": "BackwardCompat",
            "character_level": 10,
            "character_xp": 5000,
            "attributes": {"values": [10, 10, 10, 10, 10, 10]},
            "prestige_rank": 0,
            "total_prestige_count": 0,
            "last_save_time": 1234567890,
            "play_time_seconds": 100,
            "combat_state": {
                "current_enemy": null,
                "player_current_hp": 50,
                "player_max_hp": 50,
                "attack_timer": 0.0,
                "regen_timer": 0.0,
                "is_regenerating": false
            },
            "equipment": {
                "weapon": null,
                "armor": null,
                "helmet": null,
                "gloves": null,
                "boots": null,
                "amulet": null,
                "ring": null
            },
            "fishing": {
                "rank": 1,
                "total_fish_caught": 0,
                "fish_toward_next_rank": 0,
                "legendary_catches": 0
            },
            "zone_progression": {
                "current_zone_id": 1,
                "current_subzone_id": 1,
                "defeated_bosses": [],
                "unlocked_zones": [1, 2]
            },
            "future_field_that_doesnt_exist": "should be ignored",
            "another_future_field": 12345
        }"#;

        let filepath = manager.quest_dir.join("backward_compat_test.json");
        fs::write(&filepath, json_with_extra).unwrap();

        let result = manager.load_character("backward_compat_test.json");
        assert!(
            result.is_ok(),
            "Should ignore extra fields: {:?}",
            result.err()
        );

        let loaded = result.unwrap();
        assert_eq!(loaded.character_name, "BackwardCompat");
        assert_eq!(loaded.character_level, 10);
    }

    #[test]
    fn test_load_json_missing_optional_fields_forward_compat() {
        let (manager, _dir) = temp_manager();

        let minimal_json = r#"{
            "version": 2,
            "character_id": "old-save-id",
            "character_name": "OldSave",
            "character_level": 5,
            "character_xp": 1000,
            "attributes": {"values": [12, 10, 10, 10, 10, 10]},
            "prestige_rank": 1,
            "total_prestige_count": 1,
            "last_save_time": 1000000000,
            "play_time_seconds": 500,
            "combat_state": {
                "current_enemy": null,
                "player_current_hp": 60,
                "player_max_hp": 60,
                "attack_timer": 0.0,
                "regen_timer": 0.0,
                "is_regenerating": false
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
        }"#;

        let filepath = manager.quest_dir.join("forward_compat_test.json");
        fs::write(&filepath, minimal_json).unwrap();

        let result = manager.load_character("forward_compat_test.json");
        assert!(
            result.is_ok(),
            "Should use defaults for missing optional fields: {:?}",
            result.err()
        );

        let loaded = result.unwrap();
        assert_eq!(loaded.character_name, "OldSave");
        assert_eq!(loaded.character_level, 5);
        assert_eq!(loaded.fishing.rank, 1);
        assert_eq!(loaded.zone_progression.current_zone_id, 1);
    }

    #[test]
    fn test_load_json_missing_nested_optional_fields() {
        let (manager, _dir) = temp_manager();

        let json = r#"{
            "version": 2,
            "character_id": "nested-test",
            "character_name": "NestedTest",
            "character_level": 1,
            "character_xp": 0,
            "attributes": {"values": [10, 10, 10, 10, 10, 10]},
            "prestige_rank": 0,
            "total_prestige_count": 0,
            "last_save_time": 0,
            "play_time_seconds": 0,
            "combat_state": {
                "current_enemy": null,
                "player_current_hp": 50,
                "player_max_hp": 50,
                "attack_timer": 0.0,
                "regen_timer": 0.0,
                "is_regenerating": false
            },
            "equipment": {
                "weapon": null,
                "armor": null,
                "helmet": null,
                "gloves": null,
                "boots": null,
                "amulet": null,
                "ring": null
            },
            "zone_progression": {
                "current_zone_id": 3,
                "current_subzone_id": 2,
                "defeated_bosses": [[1,1], [1,2]],
                "unlocked_zones": [1, 2, 3]
            }
        }"#;

        let filepath = manager.quest_dir.join("nested_compat_test.json");
        fs::write(&filepath, json).unwrap();

        let result = manager.load_character("nested_compat_test.json");
        assert!(
            result.is_ok(),
            "Should handle missing nested optional fields: {:?}",
            result.err()
        );

        let loaded = result.unwrap();
        assert_eq!(loaded.zone_progression.current_zone_id, 3);
        assert_eq!(loaded.zone_progression.kills_in_subzone, 0);
        assert!(!loaded.zone_progression.fighting_boss);
        assert!(!loaded.zone_progression.has_stormbreaker);
    }

    #[test]
    fn test_minimal_v2_save_still_loads() {
        let (manager, _dir) = temp_manager();

        let minimal_v2_json = r#"{
            "version": 2,
            "character_id": "minimal-v2",
            "character_name": "MinimalV2",
            "character_level": 1,
            "character_xp": 0,
            "attributes": {"values": [10, 10, 10, 10, 10, 10]},
            "prestige_rank": 0,
            "total_prestige_count": 0,
            "last_save_time": 0,
            "play_time_seconds": 0,
            "combat_state": {
                "current_enemy": null,
                "player_current_hp": 50,
                "player_max_hp": 50,
                "attack_timer": 0.0,
                "regen_timer": 0.0,
                "is_regenerating": false
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
        }"#;

        let filepath = manager.quest_dir.join("minimal_v2_test.json");
        fs::write(&filepath, minimal_v2_json).unwrap();

        let result = manager.load_character("minimal_v2_test.json");
        assert!(
            result.is_ok(),
            "BACKWARD COMPATIBILITY BROKEN! Minimal v2 save failed to load. \
             If you added a new field, add #[serde(default)] to it. Error: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_default_impls_exist_for_save_structs() {
        use crate::combat::CombatState;
        use crate::fishing::FishingState;
        use crate::items::Equipment;
        use crate::zones::ZoneProgression;

        let combat = CombatState::default();
        assert_eq!(combat.player_max_hp, 50);
        assert!(combat.current_enemy.is_none());

        let equipment = Equipment::default();
        assert!(equipment.weapon.is_none());

        let fishing = FishingState::default();
        assert_eq!(fishing.rank, 1);

        let zones = ZoneProgression::default();
        assert_eq!(zones.current_zone_id, 1);
    }

    // =========================================================================
    // SAVE STRUCT COMPATIBILITY REGISTRY
    // =========================================================================

    #[test]
    fn test_combat_state_minimal_json() {
        use crate::combat::CombatState;

        let minimal_json = r#"{
            "current_enemy": null,
            "player_current_hp": 50,
            "player_max_hp": 50,
            "attack_timer": 0.0,
            "regen_timer": 0.0,
            "is_regenerating": false
        }"#;

        let result: Result<CombatState, _> = serde_json::from_str(minimal_json);
        assert!(
            result.is_ok(),
            "CombatState minimal JSON failed to deserialize. \
             If you added a new field, add #[serde(default)] or #[serde(skip)]. Error: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_equipment_minimal_json() {
        use crate::items::Equipment;

        let minimal_json = r#"{
            "weapon": null,
            "armor": null,
            "helmet": null,
            "gloves": null,
            "boots": null,
            "amulet": null,
            "ring": null
        }"#;

        let result: Result<Equipment, _> = serde_json::from_str(minimal_json);
        assert!(
            result.is_ok(),
            "Equipment minimal JSON failed to deserialize. \
             If you added a new slot, add #[serde(default)]. Error: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_attributes_minimal_json() {
        use crate::character::attributes::Attributes;

        let minimal_json = r#"{"values": [10, 10, 10, 10, 10, 10]}"#;

        let result: Result<Attributes, _> = serde_json::from_str(minimal_json);
        assert!(
            result.is_ok(),
            "Attributes minimal JSON failed to deserialize. Error: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_fishing_state_minimal_json() {
        use crate::fishing::FishingState;

        let minimal_json = r#"{
            "rank": 1,
            "total_fish_caught": 0,
            "fish_toward_next_rank": 0,
            "legendary_catches": 0
        }"#;

        let result: Result<FishingState, _> = serde_json::from_str(minimal_json);
        assert!(
            result.is_ok(),
            "FishingState minimal JSON failed to deserialize. \
             If you added a new field, add #[serde(default)]. Error: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_zone_progression_minimal_json() {
        use crate::zones::ZoneProgression;

        let minimal_json = r#"{
            "current_zone_id": 1,
            "current_subzone_id": 1,
            "defeated_bosses": [],
            "unlocked_zones": [1]
        }"#;

        let result: Result<ZoneProgression, _> = serde_json::from_str(minimal_json);
        assert!(
            result.is_ok(),
            "ZoneProgression minimal JSON failed to deserialize. \
             If you added a new field, add #[serde(default)]. Error: {:?}",
            result.err()
        );

        let zones = result.unwrap();
        assert_eq!(zones.kills_in_subzone, 0);
        assert!(!zones.fighting_boss);
        assert!(!zones.has_stormbreaker);
    }

    #[test]
    fn test_enemy_minimal_json() {
        use crate::combat::Enemy;

        let minimal_json = r#"{
            "name": "Test Enemy",
            "max_hp": 100,
            "current_hp": 100,
            "damage": 10
        }"#;

        let result: Result<Enemy, _> = serde_json::from_str(minimal_json);
        assert!(
            result.is_ok(),
            "Enemy minimal JSON failed to deserialize. \
             If you added a new field, add #[serde(default)]. Error: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_item_minimal_json() {
        use crate::items::Item;

        let minimal_json = r#"{
            "slot": "Weapon",
            "rarity": "Common",
            "base_name": "Sword",
            "display_name": "Iron Sword",
            "attributes": {"str": 1, "dex": 0, "con": 0, "int": 0, "wis": 0, "cha": 0},
            "affixes": []
        }"#;

        let result: Result<Item, _> = serde_json::from_str(minimal_json);
        assert!(
            result.is_ok(),
            "Item minimal JSON failed to deserialize. \
             If you added a new field, add #[serde(default)]. Error: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_dungeon_minimal_json() {
        use crate::dungeon::Dungeon;

        let minimal_json = r#"{
            "size": "Small",
            "grid": [[{
                "room_type": "Entrance",
                "state": "Cleared",
                "position": [0, 0],
                "connections": [false, false, false, false]
            }]],
            "player_position": [0, 0],
            "entrance_position": [0, 0],
            "boss_position": [0, 0],
            "has_key": false,
            "move_timer": 0.0,
            "collected_items": [],
            "xp_earned": 0,
            "rooms_cleared": 0
        }"#;

        let result: Result<Dungeon, _> = serde_json::from_str(minimal_json);
        assert!(
            result.is_ok(),
            "Dungeon minimal JSON failed to deserialize. \
             If you added a new field, add #[serde(default)] or #[serde(skip)]. Error: {:?}",
            result.err()
        );

        let dungeon = result.unwrap();
        assert!(!dungeon.current_room_cleared);
    }

    #[test]
    fn test_save_struct_registry_roundtrip() {
        use crate::character::attributes::Attributes;
        use crate::combat::{CombatState, Enemy};
        use crate::fishing::FishingState;
        use crate::items::Equipment;
        use crate::items::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};
        use crate::zones::ZoneProgression;

        let attrs = Attributes::default();
        let json = serde_json::to_string(&attrs).expect("Attributes should serialize");
        let _: Attributes = serde_json::from_str(&json).expect("Attributes should roundtrip");

        let combat = CombatState::default();
        let json = serde_json::to_string(&combat).expect("CombatState should serialize");
        let _: CombatState = serde_json::from_str(&json).expect("CombatState should roundtrip");

        let equipment = Equipment::default();
        let json = serde_json::to_string(&equipment).expect("Equipment should serialize");
        let _: Equipment = serde_json::from_str(&json).expect("Equipment should roundtrip");

        let fishing = FishingState::default();
        let json = serde_json::to_string(&fishing).expect("FishingState should serialize");
        let _: FishingState = serde_json::from_str(&json).expect("FishingState should roundtrip");

        let zones = ZoneProgression::default();
        let json = serde_json::to_string(&zones).expect("ZoneProgression should serialize");
        let _: ZoneProgression =
            serde_json::from_str(&json).expect("ZoneProgression should roundtrip");

        let enemy = Enemy::new("Test".to_string(), 100, 10);
        let json = serde_json::to_string(&enemy).expect("Enemy should serialize");
        let _: Enemy = serde_json::from_str(&json).expect("Enemy should roundtrip");

        let item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Common,
            ilvl: 10,
            tier: 5,
            base_name: "Sword".to_string(),
            display_name: "Test Sword".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::DamagePercent,
                value: 5.0,
            }],
            god_item_id: None,
        };
        let json = serde_json::to_string(&item).expect("Item should serialize");
        let _: Item = serde_json::from_str(&json).expect("Item should roundtrip");

        let bonuses = AttributeBonuses {
            str: 5,
            dex: 3,
            con: 2,
            int: 1,
            wis: 0,
            cha: 0,
        };
        let json = serde_json::to_string(&bonuses).expect("AttributeBonuses should serialize");
        let _: AttributeBonuses =
            serde_json::from_str(&json).expect("AttributeBonuses should roundtrip");

        let affix = Affix {
            affix_type: AffixType::CritChance,
            value: 10.0,
        };
        let json = serde_json::to_string(&affix).expect("Affix should serialize");
        let _: Affix = serde_json::from_str(&json).expect("Affix should roundtrip");
    }

    // =========================================================================
    // NEW FIELD ROUND-TRIP TESTS (PR #428 — Fracture Expansion)
    // =========================================================================
    // These tests verify that every new field added in PR #428 persists correctly
    // through a full save→load cycle via CharacterManager, and that transient
    // (non-saved) fields are reset to their defaults on load.

    // ─── ascension_level ────────────────────────────────────────────────────

    #[test]
    fn test_ascension_level_nonzero_roundtrip() {
        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("AscRtNonzero");
        state.ascension_level = 3;

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("ascrtnonzero.json")
            .expect("load failed");

        assert_eq!(
            loaded.ascension_level, 3,
            "ascension_level=3 should persist through save→load"
        );
    }

    #[test]
    fn test_ascension_level_zero_roundtrip() {
        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("AscRtZero");
        state.ascension_level = 0;

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("ascrtzero.json")
            .expect("load failed");

        assert_eq!(
            loaded.ascension_level, 0,
            "ascension_level=0 (default) should persist through save→load"
        );
    }

    #[test]
    fn test_ascension_level_vi_roundtrip() {
        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("AscRtVI");
        state.ascension_level = 6; // Ascension VI — documented cap

        manager.save_character(&state).expect("save failed");
        let loaded = manager.load_character("ascrtvi.json").expect("load failed");

        assert_eq!(
            loaded.ascension_level, 6,
            "ascension_level=6 (Ascension VI, documented cap) should persist"
        );
    }

    #[test]
    fn test_ascension_level_all_valid_levels_roundtrip() {
        // Test every documented Ascension level I-VI (values 1-6)
        for level in 1..=6_u32 {
            let (manager, _dir) = temp_manager();
            let name = format!("AscLvl{}", level);
            let mut state = make_test_state(&name);
            state.ascension_level = level;

            let filename = format!("{}.json", sanitize_name(&state.character_name));
            manager.save_character(&state).expect("save failed");
            let loaded = manager.load_character(&filename).expect("load failed");

            assert_eq!(
                loaded.ascension_level, level,
                "ascension_level={} should round-trip correctly",
                level
            );
        }
    }

    // ─── cached_fracture_zone_cap (transient, not saved) ────────────────────

    #[test]
    fn test_cached_fracture_zone_cap_is_transient() {
        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("FracCapRt");
        // Set a non-zero value before saving — should NOT appear after load
        state.cached_fracture_zone_cap = 17;

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("fraccaprt.json")
            .expect("load failed");

        assert_eq!(
            loaded.cached_fracture_zone_cap, 0,
            "cached_fracture_zone_cap is transient and must be 0 after load (was 17 before save)"
        );
    }

    // ─── cached_power_rating (transient, not saved) ──────────────────────────

    #[test]
    fn test_cached_power_rating_is_transient() {
        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("PwrRatingRt");
        // Set a non-zero value before saving — should NOT appear after load
        state.cached_power_rating = 12345.678;

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("pwrratingrt.json")
            .expect("load failed");

        assert!(
            loaded.cached_power_rating == 0.0,
            "cached_power_rating is transient and must be 0.0 after load (was {} before save)",
            state.cached_power_rating
        );
    }

    // ─── stormglass + stormglass_discovered ──────────────────────────────────

    #[test]
    fn test_stormglass_balance_roundtrip() {
        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("SGlassBalRt");
        state.stormglass = 75_000;
        state.stormglass_discovered = true;

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("sglassbalrt.json")
            .expect("load failed");

        assert_eq!(
            loaded.stormglass, 75_000,
            "stormglass currency should persist through save→load"
        );
        assert!(
            loaded.stormglass_discovered,
            "stormglass_discovered should persist through save→load"
        );
    }

    #[test]
    fn test_stormglass_zero_balance_roundtrip() {
        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("SGlassZeroRt");
        state.stormglass = 0;
        state.stormglass_discovered = false;

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("sglasszerort.json")
            .expect("load failed");

        assert_eq!(
            loaded.stormglass, 0,
            "zero stormglass balance should persist"
        );
        assert!(
            !loaded.stormglass_discovered,
            "stormglass_discovered=false should persist"
        );
    }

    #[test]
    fn test_stormglass_large_balance_roundtrip() {
        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("SGlassLgRt");
        state.stormglass = 1_000_000; // large but plausible balance

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("sglasslgrt.json")
            .expect("load failed");

        assert_eq!(
            loaded.stormglass, 1_000_000,
            "large stormglass balance should persist"
        );
    }

    // ─── storm_sigils ─────────────────────────────────────────────────────────

    #[test]
    fn test_storm_sigils_empty_roundtrip() {
        use crate::stormglass::sigils::StormSigils;

        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("SigilsEmptyRt");
        state.storm_sigils = StormSigils::new(); // all slots locked, no sigils

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("sigilsemptyrt.json")
            .expect("load failed");

        assert_eq!(
            loaded.storm_sigils.slots_unlocked, 0,
            "empty storm_sigils slots_unlocked should be 0 after load"
        );
        assert_eq!(
            loaded.storm_sigils.etched_count(),
            0,
            "empty storm_sigils should have 0 etched sigils after load"
        );
    }

    #[test]
    fn test_storm_sigils_with_etched_sigils_roundtrip() {
        use crate::stormglass::sigils::{Sigil, SigilEffectType, SigilGrade};

        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("SigilsEtchRt");

        // Unlock 3 slots and etch 2 sigils
        state.storm_sigils.slots_unlocked = 3;
        state.storm_sigils.sigils[0] = Some(Sigil {
            effect: SigilEffectType::XpPercent,
            value: 18.5,
            grade: SigilGrade::A,
        });
        state.storm_sigils.sigils[1] = Some(Sigil {
            effect: SigilEffectType::DamagePercent,
            value: 7.2,
            grade: SigilGrade::C,
        });
        // Slot 2 is unlocked but empty

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("sigilsetchrt.json")
            .expect("load failed");

        assert_eq!(
            loaded.storm_sigils.slots_unlocked, 3,
            "slots_unlocked should persist through save→load"
        );
        assert_eq!(
            loaded.storm_sigils.etched_count(),
            2,
            "etched_count should be 2 after load"
        );

        let s0 = loaded.storm_sigils.sigils[0]
            .as_ref()
            .expect("sigil slot 0 should be Some after load");
        assert_eq!(s0.effect, SigilEffectType::XpPercent, "slot 0 effect type");
        assert!(
            (s0.value - 18.5).abs() < 1e-10,
            "slot 0 value {} should be 18.5",
            s0.value
        );
        assert_eq!(s0.grade, SigilGrade::A, "slot 0 grade");

        let s1 = loaded.storm_sigils.sigils[1]
            .as_ref()
            .expect("sigil slot 1 should be Some after load");
        assert_eq!(
            s1.effect,
            SigilEffectType::DamagePercent,
            "slot 1 effect type"
        );
        assert!(
            (s1.value - 7.2).abs() < 1e-10,
            "slot 1 value {} should be 7.2",
            s1.value
        );
        assert_eq!(s1.grade, SigilGrade::C, "slot 1 grade");

        assert!(
            loaded.storm_sigils.sigils[2].is_none(),
            "slot 2 should be None (empty unlocked slot)"
        );
    }

    #[test]
    fn test_storm_sigils_extreme_grades_roundtrip() {
        use crate::stormglass::sigils::{Sigil, SigilEffectType, SigilGrade};

        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("SigilsGradeRt");
        state.storm_sigils.slots_unlocked = 5;

        // Etch one sigil with each extreme grade
        state.storm_sigils.sigils[0] = Some(Sigil {
            effect: SigilEffectType::CritChancePercent,
            value: 2.0, // min
            grade: SigilGrade::FMinus,
        });
        state.storm_sigils.sigils[1] = Some(Sigil {
            effect: SigilEffectType::CritChancePercent,
            value: 8.0, // max
            grade: SigilGrade::SPlus,
        });

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("sigilsgradert.json")
            .expect("load failed");

        let s0 = loaded.storm_sigils.sigils[0].as_ref().unwrap();
        assert_eq!(s0.grade, SigilGrade::FMinus, "FMinus grade should survive");

        let s1 = loaded.storm_sigils.sigils[1].as_ref().unwrap();
        assert_eq!(s1.grade, SigilGrade::SPlus, "SPlus grade should survive");
    }

    // ─── zone_progression with fracture zones ─────────────────────────────────

    #[test]
    fn test_zone_progression_with_unlocked_fracture_zones_roundtrip() {
        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("FracZoneRt");

        // Manually unlock fracture zones 12-14 (Red Fault chapter)
        state.zone_progression.current_zone_id = 12;
        state.zone_progression.current_subzone_id = 2;
        for zone_id in 1..=14_u32 {
            state.zone_progression.unlock_zone(zone_id);
        }
        state.zone_progression.defeat_boss(11, 4); // Expanse final boss
        state.zone_progression.defeat_boss(12, 3); // Zone 12 subzone 3 boss

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("fraczonert.json")
            .expect("load failed");

        assert_eq!(
            loaded.zone_progression.current_zone_id, 12,
            "current_zone_id should persist"
        );
        assert_eq!(
            loaded.zone_progression.current_subzone_id, 2,
            "current_subzone_id should persist"
        );

        for zone_id in 1..=14_u32 {
            assert!(
                loaded.zone_progression.is_zone_unlocked(zone_id),
                "Zone {} should remain unlocked after load",
                zone_id
            );
        }
        assert!(
            !loaded.zone_progression.is_zone_unlocked(15),
            "Zone 15 should NOT be unlocked (cap was 14)"
        );
        assert!(
            loaded.zone_progression.is_boss_defeated(11, 4),
            "Zone 11 subzone 4 boss defeat should persist"
        );
        assert!(
            loaded.zone_progression.is_boss_defeated(12, 3),
            "Zone 12 subzone 3 boss defeat should persist"
        );
    }

    #[test]
    fn test_zone_progression_all_fracture_zones_roundtrip() {
        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("AllFracRt");

        // Unlock all 30 zones and position in zone 30
        for zone_id in 1..=30_u32 {
            state.zone_progression.unlock_zone(zone_id);
        }
        state.zone_progression.current_zone_id = 30;
        state.zone_progression.current_subzone_id = 5;
        state.zone_progression.has_stormbreaker = true;

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("allfracrt.json")
            .expect("load failed");

        assert_eq!(loaded.zone_progression.current_zone_id, 30);
        assert_eq!(loaded.zone_progression.current_subzone_id, 5);
        assert!(loaded.zone_progression.has_stormbreaker);

        for zone_id in 1..=30_u32 {
            assert!(
                loaded.zone_progression.is_zone_unlocked(zone_id),
                "Zone {} should remain unlocked",
                zone_id
            );
        }
    }

    #[test]
    fn test_zone_progression_kills_in_subzone_roundtrip() {
        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("FracKillsRt");

        // Set up zone in middle of fracture zone kill tracking
        state.zone_progression.current_zone_id = 14;
        state.zone_progression.current_subzone_id = 3;
        state.zone_progression.unlock_zone(14);
        // Simulate 7 kills toward boss (boss spawns at 10)
        for _ in 0..7 {
            state.zone_progression.record_kill();
        }
        assert_eq!(state.zone_progression.kills_in_subzone, 7);
        assert!(!state.zone_progression.fighting_boss);

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("frackillsrt.json")
            .expect("load failed");

        assert_eq!(loaded.zone_progression.current_zone_id, 14);
        assert_eq!(loaded.zone_progression.current_subzone_id, 3);
        // kills_in_subzone has #[serde(default)] but is NOT #[serde(skip)], so it serializes
        assert_eq!(
            loaded.zone_progression.kills_in_subzone, 7,
            "kills_in_subzone should persist through save→load"
        );
        assert!(
            !loaded.zone_progression.fighting_boss,
            "fighting_boss=false should persist"
        );
    }

    // ─── Backward compat: loading old saves without new fields ────────────────

    #[test]
    fn test_backward_compat_save_without_ascension_level() {
        let (manager, _dir) = temp_manager();

        // Write a save file that omits ascension_level (old format pre-PR-428)
        let json_without_ascension = r#"{
            "version": 2,
            "character_id": "old-no-asc-id",
            "character_name": "OldNoAsc",
            "character_level": 15,
            "character_xp": 50000,
            "attributes": {"values": [12, 14, 11, 10, 10, 10]},
            "prestige_rank": 5,
            "total_prestige_count": 5,
            "last_save_time": 1700000000,
            "play_time_seconds": 3600,
            "combat_state": {
                "current_enemy": null,
                "player_current_hp": 100,
                "player_max_hp": 100,
                "attack_timer": 0.0,
                "regen_timer": 0.0,
                "is_regenerating": false
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
        }"#;

        let filepath = manager.quest_dir.join("oldnoasc.json");
        fs::write(&filepath, json_without_ascension).expect("Failed to write old save");

        let result = manager.load_character("oldnoasc.json");
        assert!(
            result.is_ok(),
            "Should load old save missing ascension_level via serde default. Error: {:?}",
            result.err()
        );

        let loaded = result.unwrap();
        assert_eq!(loaded.character_name, "OldNoAsc");
        assert_eq!(
            loaded.ascension_level, 0,
            "ascension_level should default to 0 when missing from save (serde default)"
        );
        assert_eq!(loaded.character_level, 15);
        assert_eq!(loaded.prestige_rank, 5);
    }

    #[test]
    fn test_backward_compat_save_without_stormglass_fields() {
        let (manager, _dir) = temp_manager();

        // Write a save file missing all stormglass-era fields
        let json_without_sg = r#"{
            "version": 2,
            "character_id": "old-no-sg-id",
            "character_name": "OldNoSG",
            "character_level": 10,
            "character_xp": 10000,
            "attributes": {"values": [10, 10, 10, 10, 10, 10]},
            "prestige_rank": 3,
            "total_prestige_count": 3,
            "last_save_time": 1699000000,
            "play_time_seconds": 1800,
            "combat_state": {
                "current_enemy": null,
                "player_current_hp": 80,
                "player_max_hp": 80,
                "attack_timer": 0.0,
                "regen_timer": 0.0,
                "is_regenerating": false
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
        }"#;

        let filepath = manager.quest_dir.join("oldnosg.json");
        fs::write(&filepath, json_without_sg).expect("Failed to write old save");

        let result = manager.load_character("oldnosg.json");
        assert!(
            result.is_ok(),
            "Should load save missing stormglass/storm_sigils/ascension_level. Error: {:?}",
            result.err()
        );

        let loaded = result.unwrap();
        assert_eq!(loaded.stormglass, 0, "stormglass should default to 0");
        assert!(
            !loaded.stormglass_discovered,
            "stormglass_discovered should default to false"
        );
        assert_eq!(
            loaded.storm_sigils.slots_unlocked, 0,
            "storm_sigils should default to empty"
        );
        assert_eq!(
            loaded.ascension_level, 0,
            "ascension_level should default to 0"
        );
    }

    // ─── Transient fields reset to defaults on load ───────────────────────────

    #[test]
    fn test_transient_fields_reset_on_load() {
        use crate::items::Rarity;

        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("TransientRt");

        // Set transient fields to non-default values before saving
        state.session_kills = 999;
        state.consecutive_deaths = 5;
        state.chrono_surge_active = true;
        state.debug_force_overcharge = true;
        state.xp_this_second = 12345;
        state.combat_seconds_this_tick = true;
        state.derived_stats_dirty = false; // should be true after load
        state.cached_power_rating = 9999.0;
        state.cached_fracture_zone_cap = 20;
        state.add_recent_drop(
            "Test Item".to_string(),
            Rarity::Rare,
            true,
            "",
            "Weapon".to_string(),
            "+5 STR".to_string(),
        );

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("transientrt.json")
            .expect("load failed");

        // All transient fields should be at defaults after load
        assert_eq!(loaded.session_kills, 0, "session_kills is transient");
        assert_eq!(
            loaded.consecutive_deaths, 0,
            "consecutive_deaths is transient"
        );
        assert!(
            !loaded.chrono_surge_active,
            "chrono_surge_active is transient"
        );
        assert!(
            !loaded.debug_force_overcharge,
            "debug_force_overcharge is transient"
        );
        assert_eq!(loaded.xp_this_second, 0, "xp_this_second is transient");
        assert!(
            !loaded.combat_seconds_this_tick,
            "combat_seconds_this_tick is transient"
        );
        assert!(
            loaded.derived_stats_dirty,
            "derived_stats_dirty should be true after load (recalculation needed)"
        );
        assert_eq!(
            loaded.cached_power_rating, 0.0,
            "cached_power_rating is transient"
        );
        assert_eq!(
            loaded.cached_fracture_zone_cap, 0,
            "cached_fracture_zone_cap is transient"
        );
        assert!(loaded.recent_drops.is_empty(), "recent_drops is transient");
        assert!(
            loaded.active_fishing.is_none(),
            "active_fishing is transient"
        );
        assert!(
            loaded.active_minigame.is_none(),
            "active_minigame is transient"
        );
        assert!(
            loaded.last_minigame_win.is_none(),
            "last_minigame_win is transient"
        );
        assert!(
            loaded.xp_rate_samples.is_empty(),
            "xp_rate_samples is transient"
        );
        assert!(
            loaded.game_over_shown_at.is_none(),
            "game_over_shown_at is transient"
        );
    }

    // ─── Multiple save→load cycles ────────────────────────────────────────────

    #[test]
    fn test_multiple_save_load_cycles_preserve_values() {
        use crate::stormglass::sigils::{Sigil, SigilEffectType, SigilGrade};

        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("MultiCycleRt");

        // Cycle 1: Set initial values
        state.ascension_level = 2;
        state.stormglass = 10_000;
        state.stormglass_discovered = true;
        state.storm_sigils.slots_unlocked = 1;
        state.storm_sigils.sigils[0] = Some(Sigil {
            effect: SigilEffectType::MaxHpPercent,
            value: 9.0,
            grade: SigilGrade::B,
        });

        manager.save_character(&state).expect("first save failed");
        let loaded1 = manager
            .load_character("multicyclert.json")
            .expect("first load failed");
        assert_eq!(loaded1.ascension_level, 2);
        assert_eq!(loaded1.stormglass, 10_000);

        // Cycle 2: Mutate and save again
        let mut state2 = loaded1;
        state2.ascension_level = 4;
        state2.stormglass = 50_000;

        manager.save_character(&state2).expect("second save failed");
        let loaded2 = manager
            .load_character("multicyclert.json")
            .expect("second load failed");

        assert_eq!(
            loaded2.ascension_level, 4,
            "ascension_level should update after second cycle"
        );
        assert_eq!(
            loaded2.stormglass, 50_000,
            "stormglass should update after second cycle"
        );
        // Sigil should still be intact through both cycles
        assert_eq!(loaded2.storm_sigils.slots_unlocked, 1);
        assert!(loaded2.storm_sigils.sigils[0].is_some());
        let s0 = loaded2.storm_sigils.sigils[0].as_ref().unwrap();
        assert_eq!(s0.effect, SigilEffectType::MaxHpPercent);
        assert!((s0.value - 9.0).abs() < 1e-10);
    }

    // ─── Combined: all new fields in one character ─────────────────────────────

    #[test]
    fn test_all_new_pr428_fields_combined_roundtrip() {
        use crate::stormglass::sigils::{Sigil, SigilEffectType, SigilGrade};

        let (manager, _dir) = temp_manager();
        let mut state = make_test_state("CombinedRt");

        // Set all new persistent fields to non-default values
        state.ascension_level = 5;
        state.stormglass = 250_000;
        state.stormglass_discovered = true;
        state.storm_sigils.slots_unlocked = 4;
        state.storm_sigils.sigils[0] = Some(Sigil {
            effect: SigilEffectType::XpPercent,
            value: 22.0,
            grade: SigilGrade::S,
        });
        state.storm_sigils.sigils[1] = Some(Sigil {
            effect: SigilEffectType::DamagePercent,
            value: 12.5,
            grade: SigilGrade::APlus,
        });
        state.storm_sigils.sigils[2] = Some(Sigil {
            effect: SigilEffectType::CritChancePercent,
            value: 6.0,
            grade: SigilGrade::B,
        });
        // slot 3 is unlocked but empty

        // Set fracture zone state
        for zone_id in 1..=20_u32 {
            state.zone_progression.unlock_zone(zone_id);
        }
        state.zone_progression.current_zone_id = 18;
        state.zone_progression.current_subzone_id = 4;

        // Transient fields — must NOT persist
        state.cached_fracture_zone_cap = 20;
        state.cached_power_rating = 5000.0;
        state.session_kills = 500;

        manager.save_character(&state).expect("save failed");
        let loaded = manager
            .load_character("combinedrt.json")
            .expect("load failed");

        // Persistent fields must survive
        assert_eq!(loaded.ascension_level, 5, "ascension_level");
        assert_eq!(loaded.stormglass, 250_000, "stormglass");
        assert!(loaded.stormglass_discovered, "stormglass_discovered");
        assert_eq!(loaded.storm_sigils.slots_unlocked, 4, "slots_unlocked");
        assert_eq!(loaded.storm_sigils.etched_count(), 3, "etched_count");

        let s0 = loaded.storm_sigils.sigils[0].as_ref().unwrap();
        assert_eq!(s0.effect, SigilEffectType::XpPercent);
        assert!((s0.value - 22.0).abs() < 1e-10);
        assert_eq!(s0.grade, SigilGrade::S);

        assert_eq!(loaded.zone_progression.current_zone_id, 18);
        assert_eq!(loaded.zone_progression.current_subzone_id, 4);
        for zone_id in 1..=20_u32 {
            assert!(
                loaded.zone_progression.is_zone_unlocked(zone_id),
                "Zone {} should be unlocked",
                zone_id
            );
        }

        // Transient fields must be reset
        assert_eq!(loaded.cached_fracture_zone_cap, 0, "transient cap reset");
        assert_eq!(
            loaded.cached_power_rating, 0.0,
            "transient power rating reset"
        );
        assert_eq!(loaded.session_kills, 0, "transient session_kills reset");
    }
}
