use crate::constants::SAVE_VERSION_MAGIC;
use crate::game_state::GameState;
use bincode;
use directories::ProjectDirs;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

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
        hasher.update(&SAVE_VERSION_MAGIC.to_le_bytes());
        hasher.update(&data_len.to_le_bytes());
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
    /// Returns an error if:
    /// - The file doesn't exist
    /// - The version magic is incorrect
    /// - The checksum verification fails
    /// - The data cannot be deserialized
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
        hasher.update(&version_bytes);
        hasher.update(&length_bytes);
        hasher.update(&data);
        let computed_checksum = hasher.finalize();

        if stored_checksum != computed_checksum.as_slice() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Checksum verification failed"
            ));
        }

        // Deserialize game state
        let state = bincode::deserialize(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(state)
    }

    /// Checks if a save file exists
    pub fn save_exists(&self) -> bool {
        self.save_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        original_state.stats[0].level = 25;
        original_state.stats[0].current_xp = 5000;

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
        assert_eq!(loaded_state.stats[0].level, original_state.stats[0].level);
        assert_eq!(loaded_state.stats[0].current_xp, original_state.stats[0].current_xp);

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
}
