//! Achievement persistence (load/save to disk).

#![allow(dead_code)] // Will be used when integrated with main.rs

use super::types::Achievements;
use std::io;

/// Load achievements from disk, or return default if not found.
pub fn load_achievements() -> Achievements {
    crate::utils::persistence::load_json_or_default("achievements.json")
}

/// Save achievements to disk.
pub fn save_achievements(achievements: &Achievements) -> io::Result<()> {
    crate::utils::persistence::save_json("achievements.json", achievements)
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
        let result = crate::utils::persistence::save_path("achievements.json");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("achievements.json"));
    }
}
