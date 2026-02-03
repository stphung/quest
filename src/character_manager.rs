use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[allow(dead_code)]
struct CharacterSaveData {
    version: u32,
    character_id: String,
    character_name: String,
    character_level: u32,
    character_xp: u64,
    attributes: crate::attributes::Attributes,
    prestige_rank: u32,
    total_prestige_count: u64,
    last_save_time: i64,
    play_time_seconds: u64,
    combat_state: crate::combat::CombatState,
    equipment: crate::equipment::Equipment,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    active_dungeon: Option<crate::dungeon::Dungeon>,
    #[serde(default)]
    fishing: crate::fishing::FishingState,
    #[serde(default)]
    zone_progression: crate::zones::ZoneProgression,
    #[serde(default)]
    chess_stats: crate::chess::ChessStats,
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
    pub attributes: crate::attributes::Attributes,
    pub equipment: crate::equipment::Equipment,
    pub is_corrupted: bool,
}

#[allow(dead_code)]
pub struct CharacterManager {
    quest_dir: PathBuf,
}

#[allow(dead_code)]
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

    pub fn save_character(&self, state: &crate::game_state::GameState) -> io::Result<()> {
        let save_data = CharacterSaveData {
            version: 2,
            character_id: state.character_id.clone(),
            character_name: state.character_name.clone(),
            character_level: state.character_level,
            character_xp: state.character_xp,
            attributes: state.attributes,
            prestige_rank: state.prestige_rank,
            total_prestige_count: state.total_prestige_count,
            last_save_time: state.last_save_time,
            play_time_seconds: state.play_time_seconds,
            combat_state: state.combat_state.clone(),
            equipment: state.equipment.clone(),
            active_dungeon: state.active_dungeon.clone(),
            fishing: state.fishing.clone(),
            zone_progression: state.zone_progression.clone(),
            chess_stats: state.chess_stats.clone(),
        };

        let json = serde_json::to_string_pretty(&save_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let filename = format!("{}.json", sanitize_name(&state.character_name));
        let filepath = self.quest_dir.join(filename);
        fs::write(filepath, json)?;

        Ok(())
    }

    pub fn load_character(&self, filename: &str) -> io::Result<crate::game_state::GameState> {
        let filepath = self.quest_dir.join(filename);
        let json_content = fs::read_to_string(filepath)?;

        let save_data: CharacterSaveData = serde_json::from_str(&json_content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(crate::game_state::GameState {
            character_id: save_data.character_id,
            character_name: save_data.character_name,
            character_level: save_data.character_level,
            character_xp: save_data.character_xp,
            attributes: save_data.attributes,
            prestige_rank: save_data.prestige_rank,
            total_prestige_count: save_data.total_prestige_count,
            last_save_time: save_data.last_save_time,
            play_time_seconds: save_data.play_time_seconds,
            combat_state: save_data.combat_state,
            equipment: save_data.equipment,
            active_dungeon: save_data.active_dungeon,
            fishing: save_data.fishing,
            active_fishing: None,
            zone_progression: save_data.zone_progression,
            challenge_menu: crate::challenge_menu::ChallengeMenu::new(),
            chess_stats: save_data.chess_stats,
            active_chess: None,
        })
    }

    pub fn list_characters(&self) -> io::Result<Vec<CharacterInfo>> {
        let mut characters = Vec::new();

        // Read directory entries
        let entries = fs::read_dir(&self.quest_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Only process .json files
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            // Try to load character
            match self.load_character(&filename) {
                Ok(state) => {
                    characters.push(CharacterInfo {
                        character_id: state.character_id,
                        character_name: state.character_name,
                        filename,
                        character_level: state.character_level,
                        prestige_rank: state.prestige_rank,
                        play_time_seconds: state.play_time_seconds,
                        last_save_time: state.last_save_time,
                        attributes: state.attributes,
                        equipment: state.equipment,
                        is_corrupted: false,
                    });
                }
                Err(_) => {
                    // Mark as corrupted but include in list
                    characters.push(CharacterInfo {
                        character_id: String::new(),
                        character_name: "[CORRUPTED]".to_string(),
                        filename,
                        character_level: 0,
                        prestige_rank: 0,
                        play_time_seconds: 0,
                        last_save_time: 0,
                        attributes: crate::attributes::Attributes::new(),
                        equipment: crate::equipment::Equipment::new(),
                        is_corrupted: true,
                    });
                }
            }
        }

        // Sort by last_save_time (most recent first)
        characters.sort_by(|a, b| b.last_save_time.cmp(&a.last_save_time));

        Ok(characters)
    }

    pub fn delete_character(&self, filename: &str) -> io::Result<()> {
        let filepath = self.quest_dir.join(filename);
        fs::remove_file(filepath)?;
        Ok(())
    }

    pub fn rename_character(&self, old_filename: &str, new_name: String) -> io::Result<()> {
        // Validate new name
        validate_name(&new_name).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        // Load existing character
        let mut state = self.load_character(old_filename)?;

        // Update character name
        state.character_name = new_name.clone();

        // Save with new name
        self.save_character(&state)?;

        // Delete old file
        let old_filepath = self.quest_dir.join(old_filename);
        fs::remove_file(old_filepath)?;

        Ok(())
    }
}

#[allow(dead_code)]
pub fn validate_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if trimmed.len() > 16 {
        return Err("Name must be 16 characters or less".to_string());
    }

    let valid_chars = trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_');

    if !valid_chars {
        return Err(
            "Name can only contain letters, numbers, spaces, hyphens, and underscores".to_string(),
        );
    }

    Ok(())
}

#[allow(dead_code)]
pub fn sanitize_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .replace(' ', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("Hero").is_ok());
        assert!(validate_name("Test 123").is_ok());
        assert!(validate_name("Warrior-2").is_ok());
        assert!(validate_name("under_score").is_ok());
    }

    #[test]
    fn test_validate_name_too_short() {
        assert!(validate_name("").is_err());
        assert!(validate_name("   ").is_err());
    }

    #[test]
    fn test_validate_name_too_long() {
        assert!(validate_name("12345678901234567").is_err()); // 17 chars
    }

    #[test]
    fn test_validate_name_invalid_chars() {
        assert!(validate_name("test@123").is_err());
        assert!(validate_name("hello!world").is_err());
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Hero"), "hero");
        assert_eq!(sanitize_name("Mage the Great"), "mage_the_great");
        assert_eq!(sanitize_name("Warrior-2"), "warrior-2");
        assert_eq!(sanitize_name("Test!!!"), "test");
        assert_eq!(sanitize_name("   Spaces   "), "spaces");
        assert_eq!(sanitize_name("MixedCase"), "mixedcase");
    }

    #[test]
    fn test_character_manager_new() {
        let manager = CharacterManager::new().expect("Failed to create CharacterManager");
        assert!(manager.quest_dir.ends_with(".quest"));
        assert!(manager.quest_dir.exists());
    }

    #[test]
    fn test_save_and_load_character() {
        use crate::attributes::Attributes;
        use crate::combat::CombatState;
        use crate::equipment::Equipment;
        use crate::game_state::GameState;
        use chrono::Utc;

        let manager = CharacterManager::new().unwrap();

        let state = GameState {
            character_id: "test-id".to_string(),
            character_name: "TestHero".to_string(),
            character_level: 10,
            character_xp: 5000,
            attributes: Attributes::new(),
            prestige_rank: 2,
            total_prestige_count: 2,
            last_save_time: Utc::now().timestamp(),
            play_time_seconds: 3600,
            combat_state: CombatState::new(100),
            equipment: Equipment::new(),
            active_dungeon: None,
            fishing: crate::fishing::FishingState::default(),
            active_fishing: None,
            zone_progression: crate::zones::ZoneProgression::default(),
            challenge_menu: crate::challenge_menu::ChallengeMenu::new(),
            chess_stats: crate::chess::ChessStats::default(),
            active_chess: None,
        };

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

        // Cleanup
        fs::remove_file(filepath).ok();
    }

    #[test]
    fn test_list_characters() {
        use crate::attributes::Attributes;
        use crate::combat::CombatState;
        use crate::equipment::Equipment;
        use crate::game_state::GameState;

        let manager = CharacterManager::new().unwrap();

        // Clean up only our test files (isolation)
        fs::remove_file(manager.quest_dir.join("listtest1.json")).ok();
        fs::remove_file(manager.quest_dir.join("listtest2.json")).ok();

        // Create test characters with unique names to avoid conflicts with other tests
        let char1 = GameState {
            character_id: "id1".to_string(),
            character_name: "ListTest1".to_string(),
            character_level: 10,
            character_xp: 5000,
            attributes: Attributes::new(),
            prestige_rank: 2,
            total_prestige_count: 2,
            last_save_time: 1000,
            play_time_seconds: 3600,
            combat_state: CombatState::new(100),
            equipment: Equipment::new(),
            active_dungeon: None,
            fishing: crate::fishing::FishingState::default(),
            active_fishing: None,
            zone_progression: crate::zones::ZoneProgression::default(),
            challenge_menu: crate::challenge_menu::ChallengeMenu::new(),
            chess_stats: crate::chess::ChessStats::default(),
            active_chess: None,
        };

        let char2 = GameState {
            character_id: "id2".to_string(),
            character_name: "ListTest2".to_string(),
            character_level: 15,
            character_xp: 8000,
            attributes: Attributes::new(),
            prestige_rank: 3,
            total_prestige_count: 3,
            last_save_time: 2000,
            play_time_seconds: 7200,
            combat_state: CombatState::new(100),
            equipment: Equipment::new(),
            active_dungeon: None,
            fishing: crate::fishing::FishingState::default(),
            active_fishing: None,
            zone_progression: crate::zones::ZoneProgression::default(),
            challenge_menu: crate::challenge_menu::ChallengeMenu::new(),
            chess_stats: crate::chess::ChessStats::default(),
            active_chess: None,
        };

        manager.save_character(&char1).unwrap();
        manager.save_character(&char2).unwrap();

        // List characters and filter to only our test characters (for parallel test isolation)
        let list = manager.list_characters().expect("Failed to list");
        let test_chars: Vec<_> = list
            .iter()
            .filter(|c| c.character_name == "ListTest1" || c.character_name == "ListTest2")
            .collect();
        assert_eq!(test_chars.len(), 2);

        // Verify sorted by last_played (most recent first)
        assert_eq!(test_chars[0].character_name, "ListTest2"); // last_save_time = 2000
        assert_eq!(test_chars[1].character_name, "ListTest1"); // last_save_time = 1000

        // Cleanup
        fs::remove_file(manager.quest_dir.join("listtest1.json")).ok();
        fs::remove_file(manager.quest_dir.join("listtest2.json")).ok();
    }

    #[test]
    fn test_delete_character() {
        use crate::attributes::Attributes;
        use crate::combat::CombatState;
        use crate::equipment::Equipment;
        use crate::game_state::GameState;
        use chrono::Utc;

        let manager = CharacterManager::new().unwrap();

        let state = GameState {
            character_id: "test-id".to_string(),
            character_name: "ToDelete".to_string(),
            character_level: 5,
            character_xp: 1000,
            attributes: Attributes::new(),
            prestige_rank: 0,
            total_prestige_count: 0,
            last_save_time: Utc::now().timestamp(),
            play_time_seconds: 100,
            combat_state: CombatState::new(50),
            equipment: Equipment::new(),
            active_dungeon: None,
            fishing: crate::fishing::FishingState::default(),
            active_fishing: None,
            zone_progression: crate::zones::ZoneProgression::default(),
            challenge_menu: crate::challenge_menu::ChallengeMenu::new(),
            chess_stats: crate::chess::ChessStats::default(),
            active_chess: None,
        };

        manager.save_character(&state).unwrap();

        let filename = "todelete.json";
        assert!(manager.quest_dir.join(filename).exists());

        manager.delete_character(filename).expect("Delete failed");
        assert!(!manager.quest_dir.join(filename).exists());
    }

    #[test]
    fn test_rename_character() {
        use crate::attributes::Attributes;
        use crate::combat::CombatState;
        use crate::equipment::Equipment;
        use crate::game_state::GameState;
        use chrono::Utc;

        let manager = CharacterManager::new().unwrap();

        let state = GameState {
            character_id: "test-id".to_string(),
            character_name: "OldName".to_string(),
            character_level: 8,
            character_xp: 3000,
            attributes: Attributes::new(),
            prestige_rank: 1,
            total_prestige_count: 1,
            last_save_time: Utc::now().timestamp(),
            play_time_seconds: 500,
            combat_state: CombatState::new(75),
            equipment: Equipment::new(),
            active_dungeon: None,
            fishing: crate::fishing::FishingState::default(),
            active_fishing: None,
            zone_progression: crate::zones::ZoneProgression::default(),
            challenge_menu: crate::challenge_menu::ChallengeMenu::new(),
            chess_stats: crate::chess::ChessStats::default(),
            active_chess: None,
        };

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

        // Cleanup
        fs::remove_file(manager.quest_dir.join("newname.json")).ok();
    }

    #[test]
    fn test_load_nonexistent_character() {
        let manager = CharacterManager::new().unwrap();

        let result = manager.load_character("nonexistent_character_12345.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_nonexistent_character() {
        let manager = CharacterManager::new().unwrap();

        let result = manager.delete_character("nonexistent_delete_test.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_with_invalid_name() {
        use crate::attributes::Attributes;
        use crate::combat::CombatState;
        use crate::equipment::Equipment;
        use crate::game_state::GameState;
        use chrono::Utc;

        let manager = CharacterManager::new().unwrap();

        let state = GameState {
            character_id: "test-id".to_string(),
            character_name: "RenameTest".to_string(),
            character_level: 1,
            character_xp: 0,
            attributes: Attributes::new(),
            prestige_rank: 0,
            total_prestige_count: 0,
            last_save_time: Utc::now().timestamp(),
            play_time_seconds: 0,
            combat_state: CombatState::new(50),
            equipment: Equipment::new(),
            active_dungeon: None,
            fishing: crate::fishing::FishingState::default(),
            active_fishing: None,
            zone_progression: crate::zones::ZoneProgression::default(),
            challenge_menu: crate::challenge_menu::ChallengeMenu::new(),
            chess_stats: crate::chess::ChessStats::default(),
            active_chess: None,
        };

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

        // Cleanup
        fs::remove_file(manager.quest_dir.join("renametest.json")).ok();
    }

    #[test]
    fn test_corrupted_file_handling() {
        let manager = CharacterManager::new().unwrap();

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

        // Cleanup
        fs::remove_file(filepath).ok();
    }

    #[test]
    fn test_sanitize_name_special_cases() {
        // Unicode alphanumeric characters are preserved
        assert_eq!(sanitize_name("Hérö"), "hérö");

        // Multiple spaces become underscores
        assert_eq!(sanitize_name("My   Hero"), "my___hero");

        // Empty after sanitization (only special chars)
        assert_eq!(sanitize_name("!!!"), "");

        // Numbers preserved
        assert_eq!(sanitize_name("Hero123"), "hero123");
    }

    #[test]
    fn test_validate_name_boundary_lengths() {
        // Exactly 16 characters should be valid
        assert!(validate_name("1234567890123456").is_ok());

        // 17 characters should fail
        assert!(validate_name("12345678901234567").is_err());

        // 1 character should be valid
        assert!(validate_name("A").is_ok());
    }

    #[test]
    fn test_character_data_integrity() {
        use crate::attributes::{AttributeType, Attributes};
        use crate::combat::CombatState;
        use crate::equipment::Equipment;
        use crate::game_state::GameState;
        use chrono::Utc;

        let manager = CharacterManager::new().unwrap();

        // Create a character with specific values
        let mut attributes = Attributes::new();
        attributes.set(AttributeType::Strength, 15);
        attributes.set(AttributeType::Dexterity, 18);

        let state = GameState {
            character_id: "integrity-test-id".to_string(),
            character_name: "IntegrityTest".to_string(),
            character_level: 25,
            character_xp: 12345,
            attributes,
            prestige_rank: 3,
            total_prestige_count: 5,
            last_save_time: Utc::now().timestamp(),
            play_time_seconds: 9999,
            combat_state: CombatState::new(100),
            equipment: Equipment::new(),
            active_dungeon: None,
            fishing: crate::fishing::FishingState::default(),
            active_fishing: None,
            zone_progression: crate::zones::ZoneProgression::default(),
            challenge_menu: crate::challenge_menu::ChallengeMenu::new(),
            chess_stats: crate::chess::ChessStats::default(),
            active_chess: None,
        };

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

        // Cleanup
        fs::remove_file(manager.quest_dir.join("integritytest.json")).ok();
    }

    #[test]
    fn test_rename_nonexistent_character() {
        let manager = CharacterManager::new().unwrap();

        let result = manager.rename_character("does_not_exist.json", "NewName".to_string());
        assert!(result.is_err());
    }
}
