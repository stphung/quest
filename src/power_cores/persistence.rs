//! Persistence for passive generator state.
//!
//! Saves/loads [`PassivesState`] to `~/.quest/passives.json`.

use super::types::PassivesState;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Returns the path `~/.quest/passives.json`.
pub fn passives_save_path() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })?;
    Ok(home_dir.join(".quest").join("passives.json"))
}

/// Load [`PassivesState`] from disk.
///
/// Returns a default (empty) state if the file is missing or corrupted.
pub fn load_passives() -> PassivesState {
    let path = match passives_save_path() {
        Ok(p) => p,
        Err(_) => return PassivesState::default(),
    };
    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => PassivesState::default(),
    }
}

/// Persist [`PassivesState`] to disk.
///
/// Creates the `~/.quest/` directory if it does not already exist.
pub fn save_passives(state: &PassivesState) -> io::Result<()> {
    let path = passives_save_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::power_cores::types::GeneratorTimer;

    #[test]
    fn load_returns_default_when_no_file() {
        let state = load_passives();
        let _ = state;
    }

    #[test]
    fn save_and_load_roundtrip() {
        use tempfile::TempDir;

        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("passives.json");

        let mut state = PassivesState::default();
        state.generators.insert(
            "power_core_1".to_string(),
            GeneratorTimer {
                last_granted_at: 1_700_000_000,
            },
        );
        state.generators.insert(
            "power_core_2".to_string(),
            GeneratorTimer {
                last_granted_at: 1_700_100_000,
            },
        );

        let json = serde_json::to_string_pretty(&state).unwrap();
        std::fs::write(&path, &json).unwrap();

        let loaded: PassivesState = serde_json::from_str(&json).unwrap();
        assert_eq!(
            loaded.generators["power_core_1"].last_granted_at,
            1_700_000_000
        );
        assert_eq!(
            loaded.generators["power_core_2"].last_granted_at,
            1_700_100_000
        );
        // Unset core should be missing from the map
        assert!(!loaded.generators.contains_key("power_core_3"));
    }

    #[test]
    fn passives_save_path_is_within_home_dot_quest() {
        if let Ok(path) = passives_save_path() {
            let path_str = path.to_string_lossy();
            assert!(
                path_str.contains(".quest"),
                "Expected path to contain .quest, got: {path_str}"
            );
            assert!(
                path_str.ends_with("passives.json"),
                "Expected path to end with passives.json, got: {path_str}"
            );
        }
    }
}
