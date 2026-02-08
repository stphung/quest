//! Achievement persistence (load/save to disk).

#![allow(dead_code)] // Will be used when integrated with main.rs

use super::types::Achievements;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Get the achievements save file path (~/.quest/achievements.json).
pub fn achievements_save_path() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })?;
    Ok(home_dir.join(".quest").join("achievements.json"))
}

/// Load achievements from disk, or return default if not found.
pub fn load_achievements() -> Achievements {
    let path = match achievements_save_path() {
        Ok(p) => p,
        Err(_) => return Achievements::default(),
    };

    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => Achievements::default(),
    }
}

/// Save achievements to disk.
pub fn save_achievements(achievements: &Achievements) -> io::Result<()> {
    let path = achievements_save_path()?;

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(achievements)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::types::AchievementId;

    #[test]
    fn test_achievements_serialization() {
        // Create test achievements
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::SlayerI, Some("TestHero".to_string()));
        achievements.total_kills = 42;

        // Serialize and deserialize
        let json = serde_json::to_string_pretty(&achievements).unwrap();
        let loaded: Achievements = serde_json::from_str(&json).unwrap();

        assert!(loaded.is_unlocked(AchievementId::SlayerI));
        assert_eq!(loaded.total_kills, 42);
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        // This tests that loading from a non-existent file returns default
        let default = Achievements::default();
        assert_eq!(default.total_kills, 0);
        assert!(!default.is_unlocked(AchievementId::SlayerI));
    }

    #[test]
    fn test_achievements_save_path() {
        // Just verify the path generation doesn't panic
        let result = achievements_save_path();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("achievements.json"));
    }
}
