use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
    checksum: String,
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
        // Create save data without checksum
        let mut save_data = CharacterSaveData {
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
            checksum: String::new(),
        };

        // Serialize without checksum to compute hash
        let json_without_checksum = serde_json::to_string_pretty(&save_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Compute checksum
        let mut hasher = Sha256::new();
        hasher.update(json_without_checksum.as_bytes());
        let checksum = format!("{:x}", hasher.finalize());

        // Add checksum and serialize final version
        save_data.checksum = checksum;
        let json_with_checksum = serde_json::to_string_pretty(&save_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Write to file
        let filename = format!("{}.json", sanitize_name(&state.character_name));
        let filepath = self.quest_dir.join(filename);
        fs::write(filepath, json_with_checksum)?;

        Ok(())
    }

    pub fn load_character(&self, filename: &str) -> io::Result<crate::game_state::GameState> {
        let filepath = self.quest_dir.join(filename);
        let json_content = fs::read_to_string(filepath)?;

        // Parse JSON
        let save_data: CharacterSaveData = serde_json::from_str(&json_content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Store checksum, then zero it out for verification
        let stored_checksum = save_data.checksum.clone();
        let mut save_data_for_check = save_data.clone();
        save_data_for_check.checksum = String::new();

        // Recompute checksum
        let json_without_checksum = serde_json::to_string_pretty(&save_data_for_check)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut hasher = Sha256::new();
        hasher.update(json_without_checksum.as_bytes());
        let computed_checksum = format!("{:x}", hasher.finalize());

        // Verify checksum
        if stored_checksum != computed_checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Checksum verification failed - file may be corrupted or tampered",
            ));
        }

        // Convert to GameState
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
        })
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
}
