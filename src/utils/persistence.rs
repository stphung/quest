//! Generic JSON persistence helpers for ~/.quest/ save files.
//!
//! Replaces duplicated save/load boilerplate across character/manager.rs,
//! haven/logic.rs, and achievements/persistence.rs.

use std::fs;
use std::io;
use std::path::PathBuf;

/// Get the ~/.quest/ directory path, creating it if needed.
pub fn quest_dir() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })?;
    let dir = home_dir.join(".quest");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Get the full path for a save file in ~/.quest/.
pub fn save_path(filename: &str) -> io::Result<PathBuf> {
    Ok(quest_dir()?.join(filename))
}

/// Load a JSON file from ~/.quest/, returning `T::default()` if missing or invalid.
pub fn load_json_or_default<T: Default + serde::de::DeserializeOwned>(filename: &str) -> T {
    let path = match save_path(filename) {
        Ok(p) => p,
        Err(_) => return T::default(),
    };
    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => T::default(),
    }
}

/// Save a value as pretty-printed JSON to ~/.quest/.
pub fn save_json<T: serde::Serialize>(filename: &str, data: &T) -> io::Result<()> {
    let path = save_path(filename)?;
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_dir_exists() {
        let dir = quest_dir().expect("quest_dir should succeed");
        assert!(dir.exists());
        assert!(dir.ends_with(".quest"));
    }

    #[test]
    fn test_save_path_format() {
        let path = save_path("test.json").expect("save_path should succeed");
        assert!(path.to_string_lossy().ends_with(".quest/test.json"));
    }

    #[test]
    fn test_load_missing_returns_default() {
        let val: Vec<String> = load_json_or_default("nonexistent_test_file_12345.json");
        assert!(val.is_empty());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let data = vec!["hello".to_string(), "world".to_string()];
        save_json("persistence_test.json", &data).expect("save should succeed");

        let loaded: Vec<String> = load_json_or_default("persistence_test.json");
        assert_eq!(loaded, data);

        // Cleanup
        let path = save_path("persistence_test.json").unwrap();
        fs::remove_file(path).ok();
    }
}
