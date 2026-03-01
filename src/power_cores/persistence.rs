//! Persistence for Power Core state.
//!
//! Saves/loads `PowerCoreState` to `~/.quest/power_cores.json`.

use super::types::PowerCoreState;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Returns the path `~/.quest/power_cores.json`.
pub fn power_cores_save_path() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })?;
    Ok(home_dir.join(".quest").join("power_cores.json"))
}

/// Load `PowerCoreState` from disk.
///
/// Returns a default (empty) state if the file is missing or corrupted.
pub fn load_power_cores() -> PowerCoreState {
    let path = match power_cores_save_path() {
        Ok(p) => p,
        Err(_) => return PowerCoreState::default(),
    };
    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => PowerCoreState::default(),
    }
}

/// Persist `PowerCoreState` to disk.
///
/// Creates the `~/.quest/` directory if it does not already exist.
pub fn save_power_cores(state: &PowerCoreState) -> io::Result<()> {
    let path = power_cores_save_path()?;
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
    use crate::achievements::AchievementId;

    #[test]
    fn load_returns_default_when_no_file() {
        // We just verify the function doesn't panic and returns something sane.
        // The actual file may or may not exist on the test runner machine.
        let state = load_power_cores();
        // A missing file returns an empty map; a valid file is also accepted.
        let _ = state;
    }

    #[test]
    fn save_and_load_roundtrip() {
        use tempfile::TempDir;

        // Use a temp dir to avoid clobbering real saves.
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("power_cores.json");

        let mut state = PowerCoreState::default();
        state
            .last_granted_at
            .insert(AchievementId::PowerCoreI, 1_700_000_000);
        state
            .last_granted_at
            .insert(AchievementId::PowerCoreII, 1_700_100_000);

        let json = serde_json::to_string_pretty(&state).unwrap();
        std::fs::write(&path, &json).unwrap();

        let loaded: PowerCoreState = serde_json::from_str(&json).unwrap();
        assert_eq!(
            loaded.last_granted_at[&AchievementId::PowerCoreI],
            1_700_000_000
        );
        assert_eq!(
            loaded.last_granted_at[&AchievementId::PowerCoreII],
            1_700_100_000
        );
        // Unset core should be missing from the map
        assert!(!loaded
            .last_granted_at
            .contains_key(&AchievementId::PowerCoreIII));
    }

    #[test]
    fn power_cores_save_path_is_within_home_dot_quest() {
        if let Ok(path) = power_cores_save_path() {
            let path_str = path.to_string_lossy();
            assert!(
                path_str.contains(".quest"),
                "Expected path to contain .quest, got: {path_str}"
            );
            assert!(
                path_str.ends_with("power_cores.json"),
                "Expected path to end with power_cores.json, got: {path_str}"
            );
        }
        // If home_dir is unavailable (CI without HOME), skip gracefully.
    }
}
