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
pub(super) const ACCOUNT_FILES: &[&str] = &["haven.json", "achievements.json", ".cloud.json"];

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
            stormglass: 0,
            stormglass_discovered: false,
            storm_sigils: crate::stormglass::sigils::StormSigils::new(),
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
}
