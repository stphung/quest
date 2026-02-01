use crate::attributes::Attributes;
use crate::combat::CombatState;
use crate::constants::SAVE_VERSION_MAGIC;
use crate::game_state::GameState;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

const _SAVE_VERSION: u32 = 2;

/// Old stat structure from version 1
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OldStat {
    stat_type: OldStatType,
    level: u32,
    current_xp: u64,
}

/// Old stat types from version 1
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum OldStatType {
    Strength,
    Magic,
    Wisdom,
    Vitality,
}

/// Old game state structure from version 1
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OldGameState {
    stats: Vec<OldStat>,
    prestige_rank: u32,
    total_prestige_count: u64,
    last_save_time: i64,
    play_time_seconds: u64,
    #[serde(default)]
    combat_state: OldCombatState,
}

/// Old combat state from version 1
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct OldCombatState {
    current_enemy: Option<String>,
    enemy_spawn_timer: f64,
    attack_animation_timer: f64,
}

/// Manages saving and loading game state with checksummed binary format
pub struct SaveManager {
    save_path: PathBuf,
}

impl SaveManager {
    /// Creates a new SaveManager instance
    ///
    /// Sets up the save directory at the appropriate location for the platform
    /// using the `directories` crate.
    pub fn new() -> io::Result<Self> {
        let project_dirs = ProjectDirs::from("", "", "idle-rpg")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not determine config directory"))?;

        let config_dir = project_dirs.config_dir();
        fs::create_dir_all(config_dir)?;

        let save_path = config_dir.join("save.dat");

        Ok(Self { save_path })
    }

    /// Saves the game state to disk with checksum verification
    ///
    /// File format:
    /// - Version magic (8 bytes)
    /// - Data length (4 bytes)
    /// - Serialized game state (variable length)
    /// - SHA256 checksum (32 bytes)
    pub fn save(&self, state: &GameState) -> io::Result<()> {
        // Serialize the game state
        let data = bincode::serialize(state)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let data_len = data.len() as u32;

        // Compute checksum over version + length + data
        let mut hasher = Sha256::new();
        hasher.update(SAVE_VERSION_MAGIC.to_le_bytes());
        hasher.update(data_len.to_le_bytes());
        hasher.update(&data);
        let checksum = hasher.finalize();

        // Write to file
        let mut file = fs::File::create(&self.save_path)?;
        file.write_all(&SAVE_VERSION_MAGIC.to_le_bytes())?;
        file.write_all(&data_len.to_le_bytes())?;
        file.write_all(&data)?;
        file.write_all(&checksum)?;

        Ok(())
    }

    /// Loads the game state from disk with checksum verification
    ///
    /// Attempts to load in new format first. If that fails, tries to migrate
    /// from old format (version 1).
    ///
    /// Returns an error if:
    /// - The file doesn't exist
    /// - The version magic is incorrect
    /// - The checksum verification fails
    /// - The data cannot be deserialized or migrated
    pub fn load(&self) -> io::Result<GameState> {
        let mut file = fs::File::open(&self.save_path)?;

        // Read and verify version magic
        let mut version_bytes = [0u8; 8];
        file.read_exact(&mut version_bytes)?;
        let version = u64::from_le_bytes(version_bytes);

        if version != SAVE_VERSION_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid save version: expected 0x{:016X}, got 0x{:016X}", SAVE_VERSION_MAGIC, version)
            ));
        }

        // Read data length
        let mut length_bytes = [0u8; 4];
        file.read_exact(&mut length_bytes)?;
        let data_len = u32::from_le_bytes(length_bytes);

        // Read data
        let mut data = vec![0u8; data_len as usize];
        file.read_exact(&mut data)?;

        // Read checksum
        let mut stored_checksum = [0u8; 32];
        file.read_exact(&mut stored_checksum)?;

        // Verify checksum
        let mut hasher = Sha256::new();
        hasher.update(version_bytes);
        hasher.update(length_bytes);
        hasher.update(&data);
        let computed_checksum = hasher.finalize();

        if stored_checksum != computed_checksum.as_slice() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Checksum verification failed"
            ));
        }

        // Try to deserialize as new GameState (version 2)
        match bincode::deserialize::<GameState>(&data) {
            Ok(state) => Ok(state),
            Err(_) => {
                // Fall back to old format (version 1) and migrate
                let old_state = bincode::deserialize::<OldGameState>(&data)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
                        format!("Failed to deserialize as both new and old formats: {}", e)))?;

                Ok(self.migrate_old_save(old_state))
            }
        }
    }

    /// Checks if a save file exists
    pub fn save_exists(&self) -> bool {
        self.save_path.exists()
    }

    /// Migrates an old save file (version 1) to the new format (version 2)
    fn migrate_old_save(&self, old_state: OldGameState) -> GameState {
        use crate::attributes::AttributeType;

        // Calculate average level from old stats
        let total_level: u32 = old_state.stats.iter().map(|s| s.level).sum();
        let avg_level = total_level / old_state.stats.len().max(1) as u32;

        // Convert old stats to attributes using a simple mapping
        // Each stat level contributes to an attribute (beyond base 10)
        let mut attributes = Attributes::new();

        for old_stat in &old_state.stats {
            let bonus = (old_stat.level.saturating_sub(1)) / 3; // Each 3 levels = 1 attribute point
            match old_stat.stat_type {
                OldStatType::Strength => {
                    attributes.set(AttributeType::Strength, 10 + bonus);
                }
                OldStatType::Magic => {
                    attributes.set(AttributeType::Intelligence, 10 + bonus);
                }
                OldStatType::Wisdom => {
                    attributes.set(AttributeType::Wisdom, 10 + bonus);
                }
                OldStatType::Vitality => {
                    attributes.set(AttributeType::Constitution, 10 + bonus);
                }
            }
        }

        // Calculate approximate XP based on average level
        let character_level = avg_level.max(1);
        let mut _total_xp = 0u64;
        for level in 1..character_level {
            _total_xp += crate::game_logic::xp_for_next_level(level);
        }

        // Add partial XP from current level (average of old stats' current_xp)
        let avg_xp: u64 = old_state.stats.iter().map(|s| s.current_xp).sum::<u64>()
            / old_state.stats.len().max(1) as u64;

        // Create new combat state with proper HP
        use crate::derived_stats::DerivedStats;
        let derived = DerivedStats::from_attributes(&attributes);
        let combat_state = CombatState::new(derived.max_hp);

        GameState {
            character_level,
            character_xp: avg_xp,
            attributes,
            prestige_rank: old_state.prestige_rank,
            total_prestige_count: old_state.total_prestige_count,
            last_save_time: old_state.last_save_time,
            play_time_seconds: old_state.play_time_seconds,
            combat_state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::AttributeType;
    use std::fs;

    #[test]
    fn test_save_and_load() {
        let manager = SaveManager::new().expect("Failed to create SaveManager");

        // Clean up any existing save file
        if manager.save_exists() {
            fs::remove_file(&manager.save_path).expect("Failed to remove existing save file");
        }

        // Create a game state with some non-default values
        let mut original_state = GameState::new(1234567890);
        original_state.prestige_rank = 5;
        original_state.total_prestige_count = 10;
        original_state.play_time_seconds = 3600;
        original_state.character_level = 25;
        original_state.character_xp = 5000;
        original_state.attributes.set(AttributeType::Strength, 15);

        // Save the state
        manager.save(&original_state).expect("Failed to save game state");

        // Verify the file exists
        assert!(manager.save_exists());

        // Load the state
        let loaded_state = manager.load().expect("Failed to load game state");

        // Verify the loaded state matches the original
        assert_eq!(loaded_state.prestige_rank, original_state.prestige_rank);
        assert_eq!(loaded_state.total_prestige_count, original_state.total_prestige_count);
        assert_eq!(loaded_state.last_save_time, original_state.last_save_time);
        assert_eq!(loaded_state.play_time_seconds, original_state.play_time_seconds);
        assert_eq!(loaded_state.character_level, original_state.character_level);
        assert_eq!(loaded_state.character_xp, original_state.character_xp);
        assert_eq!(
            loaded_state.attributes.get(AttributeType::Strength),
            original_state.attributes.get(AttributeType::Strength)
        );

        // Clean up
        fs::remove_file(&manager.save_path).expect("Failed to remove save file");
    }

    #[test]
    fn test_load_nonexistent() {
        let manager = SaveManager::new().expect("Failed to create SaveManager");

        // Ensure no save file exists
        if manager.save_exists() {
            fs::remove_file(&manager.save_path).expect("Failed to remove existing save file");
        }

        // Attempt to load should fail
        let result = manager.load();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn test_migrate_old_save() {
        let manager = SaveManager::new().expect("Failed to create SaveManager");

        // Create an old-format game state
        let old_state = OldGameState {
            stats: vec![
                OldStat {
                    stat_type: OldStatType::Strength,
                    level: 10,
                    current_xp: 1000,
                },
                OldStat {
                    stat_type: OldStatType::Magic,
                    level: 15,
                    current_xp: 2000,
                },
                OldStat {
                    stat_type: OldStatType::Wisdom,
                    level: 20,
                    current_xp: 3000,
                },
                OldStat {
                    stat_type: OldStatType::Vitality,
                    level: 5,
                    current_xp: 500,
                },
            ],
            prestige_rank: 2,
            total_prestige_count: 5,
            last_save_time: 1234567890,
            play_time_seconds: 7200,
            combat_state: OldCombatState::default(),
        };

        // Migrate the old save
        let new_state = manager.migrate_old_save(old_state);

        // Verify migration results
        assert_eq!(new_state.prestige_rank, 2);
        assert_eq!(new_state.total_prestige_count, 5);
        assert_eq!(new_state.last_save_time, 1234567890);
        assert_eq!(new_state.play_time_seconds, 7200);

        // Character level should be average of old stats: (10+15+20+5)/4 = 12.5 -> 12
        assert_eq!(new_state.character_level, 12);

        // Attributes should be converted
        assert!(new_state.attributes.get(AttributeType::Strength) >= 10);
        assert!(new_state.attributes.get(AttributeType::Intelligence) >= 10);
        assert!(new_state.attributes.get(AttributeType::Wisdom) >= 10);
        assert!(new_state.attributes.get(AttributeType::Constitution) >= 10);
    }
}
