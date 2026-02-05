use super::attributes::Attributes;
use crate::combat::CombatState;
use crate::core::constants::SAVE_VERSION_MAGIC;
use crate::core::game_state::GameState;
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
        let project_dirs = ProjectDirs::from("", "", "idle-rpg").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Could not determine config directory",
            )
        })?;

        let config_dir = project_dirs.config_dir();
        fs::create_dir_all(config_dir)?;

        let save_path = config_dir.join("save.dat");

        Ok(Self { save_path })
    }

    /// Creates a SaveManager for testing with a unique temporary directory
    #[cfg(test)]
    fn new_for_test() -> io::Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join(format!("quest-test-{}", test_id));
        fs::create_dir_all(&temp_dir)?;

        let save_path = temp_dir.join("save.dat");
        Ok(Self { save_path })
    }

    /// Saves the game state to disk with checksum verification
    ///
    /// File format:
    /// - Version magic (8 bytes)
    /// - Data length (4 bytes)
    /// - Serialized game state (variable length)
    /// - SHA256 checksum (32 bytes)
    #[allow(dead_code)]
    pub fn save(&self, state: &GameState) -> io::Result<()> {
        // Serialize the game state
        let data =
            bincode::serialize(state).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

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
                format!(
                    "Invalid save version: expected 0x{:016X}, got 0x{:016X}",
                    SAVE_VERSION_MAGIC, version
                ),
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
                "Checksum verification failed",
            ));
        }

        // Try to deserialize as new GameState (version 2)
        match bincode::deserialize::<GameState>(&data) {
            Ok(state) => Ok(state),
            Err(_) => {
                // Fall back to old format (version 1) and migrate
                let old_state = bincode::deserialize::<OldGameState>(&data).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to deserialize as both new and old formats: {}", e),
                    )
                })?;

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
        use super::attributes::AttributeType;

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
            _total_xp += crate::core::game_logic::xp_for_next_level(level);
        }

        // Add partial XP from current level (average of old stats' current_xp)
        let avg_xp: u64 = old_state.stats.iter().map(|s| s.current_xp).sum::<u64>()
            / old_state.stats.len().max(1) as u64;

        // Create new combat state with proper HP
        use super::derived_stats::DerivedStats;
        use crate::items::Equipment;
        let derived = DerivedStats::calculate_derived_stats(&attributes, &Equipment::new());
        let combat_state = CombatState::new(derived.max_hp);

        GameState {
            character_id: uuid::Uuid::new_v4().to_string(),
            character_name: "Imported Character".to_string(),
            character_level,
            character_xp: avg_xp,
            attributes,
            prestige_rank: old_state.prestige_rank,
            total_prestige_count: old_state.total_prestige_count,
            last_save_time: old_state.last_save_time,
            play_time_seconds: old_state.play_time_seconds,
            combat_state,
            equipment: crate::items::Equipment::new(),
            active_dungeon: None,
            fishing: crate::fishing::FishingState::default(),
            active_fishing: None,
            zone_progression: crate::zones::ZoneProgression::default(),
            challenge_menu: crate::challenges::menu::ChallengeMenu::new(),
            chess_stats: crate::challenges::chess::ChessStats::default(),
            active_minigame: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::attributes::AttributeType;
    use std::fs;

    #[test]
    fn test_save_and_load() {
        let manager = SaveManager::new_for_test().expect("Failed to create SaveManager");

        // Clean up any existing save file
        if manager.save_exists() {
            fs::remove_file(&manager.save_path).expect("Failed to remove existing save file");
        }

        // Create a game state with some non-default values
        let mut original_state = GameState::new("Test Hero".to_string(), 1234567890);
        original_state.prestige_rank = 5;
        original_state.total_prestige_count = 10;
        original_state.play_time_seconds = 3600;
        original_state.character_level = 25;
        original_state.character_xp = 5000;
        original_state.attributes.set(AttributeType::Strength, 15);

        // Save the state
        manager
            .save(&original_state)
            .expect("Failed to save game state");

        // Verify the file exists
        assert!(manager.save_exists());

        // Load the state
        let loaded_state = manager.load().expect("Failed to load game state");

        // Verify the loaded state matches the original
        assert_eq!(loaded_state.prestige_rank, original_state.prestige_rank);
        assert_eq!(
            loaded_state.total_prestige_count,
            original_state.total_prestige_count
        );
        assert_eq!(loaded_state.last_save_time, original_state.last_save_time);
        assert_eq!(
            loaded_state.play_time_seconds,
            original_state.play_time_seconds
        );
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
        let manager = SaveManager::new_for_test().expect("Failed to create SaveManager");

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
        let manager = SaveManager::new_for_test().expect("Failed to create SaveManager");

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

    #[test]
    fn test_save_load_with_equipment() {
        use crate::items::{AttributeBonuses, EquipmentSlot, Item, Rarity};

        let save_mgr = SaveManager::new_for_test().unwrap();

        let mut game_state = crate::core::game_state::GameState::new(
            "Test Hero".to_string(),
            chrono::Utc::now().timestamp(),
        );

        // Equip items
        let weapon = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Legendary,
            base_name: "Greatsword".to_string(),
            display_name: "Flaming Greatsword".to_string(),
            attributes: AttributeBonuses {
                str: 12,
                dex: 5,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
        };
        game_state
            .equipment
            .set(EquipmentSlot::Weapon, Some(weapon));

        // Save
        save_mgr.save(&game_state).unwrap();

        // Load
        let loaded = save_mgr.load().unwrap();

        // Verify equipment loaded correctly
        assert!(loaded.equipment.get(EquipmentSlot::Weapon).is_some());
        let loaded_weapon = loaded
            .equipment
            .get(EquipmentSlot::Weapon)
            .as_ref()
            .unwrap();
        assert_eq!(loaded_weapon.display_name, "Flaming Greatsword");
        assert_eq!(loaded_weapon.attributes.str, 12);
        assert_eq!(loaded_weapon.rarity, Rarity::Legendary);
    }

    #[test]
    fn test_load_corrupted_file_random_bytes() {
        let manager = SaveManager::new_for_test().expect("Failed to create SaveManager");

        // Write random garbage to the save file
        fs::write(&manager.save_path, b"random garbage data that is not valid").unwrap();

        // Attempt to load should fail with InvalidData
        let result = manager.load();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_load_truncated_file() {
        let manager = SaveManager::new_for_test().expect("Failed to create SaveManager");

        // Write just the version magic (incomplete file)
        fs::write(&manager.save_path, SAVE_VERSION_MAGIC.to_le_bytes()).unwrap();

        // Attempt to load should fail (can't read length)
        let result = manager.load();
        assert!(result.is_err());
    }

    #[test]
    fn test_load_wrong_version_magic() {
        let manager = SaveManager::new_for_test().expect("Failed to create SaveManager");

        // Write a file with wrong version magic
        let wrong_magic: u64 = 0xDEADBEEF;
        let mut data = Vec::new();
        data.extend_from_slice(&wrong_magic.to_le_bytes());
        data.extend_from_slice(&[0u8; 100]); // Pad with zeros
        fs::write(&manager.save_path, &data).unwrap();

        // Attempt to load should fail with InvalidData mentioning version
        let result = manager.load();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("version"));
    }

    #[test]
    fn test_load_bad_checksum() {
        let manager = SaveManager::new_for_test().expect("Failed to create SaveManager");

        // First save a valid state
        let state = GameState::new("Test".to_string(), 0);
        manager.save(&state).unwrap();

        // Read the file and corrupt the checksum (last 32 bytes)
        let mut data = fs::read(&manager.save_path).unwrap();
        let len = data.len();
        // Flip some bits in the checksum
        data[len - 1] ^= 0xFF;
        data[len - 2] ^= 0xFF;
        fs::write(&manager.save_path, &data).unwrap();

        // Attempt to load should fail with checksum error
        let result = manager.load();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("Checksum"));
    }

    #[test]
    fn test_load_bad_checksum_corrupted_data() {
        let manager = SaveManager::new_for_test().expect("Failed to create SaveManager");

        // Save a valid state
        let state = GameState::new("Test".to_string(), 0);
        manager.save(&state).unwrap();

        // Read the file and corrupt the data (not the checksum)
        let mut data = fs::read(&manager.save_path).unwrap();
        // Corrupt byte in the middle of the data (after header: 8 + 4 = 12 bytes)
        if data.len() > 20 {
            data[15] ^= 0xFF;
            data[16] ^= 0xFF;
        }
        fs::write(&manager.save_path, &data).unwrap();

        // Attempt to load should fail with checksum error
        let result = manager.load();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_save_load_with_deprecated_affixes() {
        use crate::items::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};

        let manager = SaveManager::new_for_test().unwrap();

        let mut state = GameState::new("Test".to_string(), 0);

        // Create item with deprecated affixes (these no longer drop but should still load)
        let item_with_deprecated = Item {
            slot: EquipmentSlot::Ring,
            rarity: Rarity::Epic,
            base_name: "Ring".to_string(),
            display_name: "Legacy Ring".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![
                Affix {
                    affix_type: AffixType::DropRate,
                    value: 25.0,
                },
                Affix {
                    affix_type: AffixType::PrestigeBonus,
                    value: 30.0,
                },
                Affix {
                    affix_type: AffixType::OfflineRate,
                    value: 20.0,
                },
            ],
        };

        state
            .equipment
            .set(EquipmentSlot::Ring, Some(item_with_deprecated));

        // Save and load
        manager.save(&state).unwrap();
        let loaded = manager.load().unwrap();

        // Verify deprecated affixes are preserved
        let ring = loaded.equipment.get(EquipmentSlot::Ring).as_ref().unwrap();
        assert_eq!(ring.affixes.len(), 3);
        assert!(ring
            .affixes
            .iter()
            .any(|a| a.affix_type == AffixType::DropRate));
        assert!(ring
            .affixes
            .iter()
            .any(|a| a.affix_type == AffixType::PrestigeBonus));
        assert!(ring
            .affixes
            .iter()
            .any(|a| a.affix_type == AffixType::OfflineRate));
    }

    #[test]
    fn test_save_load_with_new_affixes() {
        use crate::items::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};

        let manager = SaveManager::new_for_test().unwrap();

        let mut state = GameState::new("Test".to_string(), 0);

        // Create item with the newly implemented affixes
        let item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Legendary,
            base_name: "Sword".to_string(),
            display_name: "Epic Sword".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![
                Affix {
                    affix_type: AffixType::CritMultiplier,
                    value: 50.0,
                },
                Affix {
                    affix_type: AffixType::AttackSpeed,
                    value: 25.0,
                },
                Affix {
                    affix_type: AffixType::HPRegen,
                    value: 40.0,
                },
                Affix {
                    affix_type: AffixType::DamageReflection,
                    value: 15.0,
                },
            ],
        };

        state.equipment.set(EquipmentSlot::Weapon, Some(item));

        // Save and load
        manager.save(&state).unwrap();
        let loaded = manager.load().unwrap();

        // Verify all affixes loaded with correct values
        let weapon = loaded
            .equipment
            .get(EquipmentSlot::Weapon)
            .as_ref()
            .unwrap();
        assert_eq!(weapon.affixes.len(), 4);

        let crit_mult = weapon
            .affixes
            .iter()
            .find(|a| a.affix_type == AffixType::CritMultiplier)
            .unwrap();
        assert!((crit_mult.value - 50.0).abs() < f64::EPSILON);

        let attack_speed = weapon
            .affixes
            .iter()
            .find(|a| a.affix_type == AffixType::AttackSpeed)
            .unwrap();
        assert!((attack_speed.value - 25.0).abs() < f64::EPSILON);

        let hp_regen = weapon
            .affixes
            .iter()
            .find(|a| a.affix_type == AffixType::HPRegen)
            .unwrap();
        assert!((hp_regen.value - 40.0).abs() < f64::EPSILON);

        let reflect = weapon
            .affixes
            .iter()
            .find(|a| a.affix_type == AffixType::DamageReflection)
            .unwrap();
        assert!((reflect.value - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_save_exists_false_initially() {
        let manager = SaveManager::new_for_test().expect("Failed to create SaveManager");

        // Clean up any existing file
        if manager.save_exists() {
            fs::remove_file(&manager.save_path).unwrap();
        }

        assert!(!manager.save_exists());
    }

    #[test]
    fn test_save_creates_file() {
        let manager = SaveManager::new_for_test().expect("Failed to create SaveManager");

        // Clean up any existing file
        if manager.save_exists() {
            fs::remove_file(&manager.save_path).unwrap();
        }

        let state = GameState::new("Test".to_string(), 0);
        manager.save(&state).unwrap();

        assert!(manager.save_exists());

        // Clean up
        fs::remove_file(&manager.save_path).unwrap();
    }

    #[test]
    fn test_save_overwrites_existing() {
        let manager = SaveManager::new_for_test().expect("Failed to create SaveManager");

        // Save first state
        let mut state1 = GameState::new("Hero1".to_string(), 0);
        state1.character_level = 10;
        manager.save(&state1).unwrap();

        // Save second state (should overwrite)
        let mut state2 = GameState::new("Hero2".to_string(), 0);
        state2.character_level = 50;
        manager.save(&state2).unwrap();

        // Load should return second state
        let loaded = manager.load().unwrap();
        assert_eq!(loaded.character_name, "Hero2");
        assert_eq!(loaded.character_level, 50);

        // Clean up
        fs::remove_file(&manager.save_path).unwrap();
    }
}
