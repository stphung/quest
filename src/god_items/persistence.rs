//! God item progress persistence (load/save to disk).

use super::types::GodItemProgress;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Get the god items save file path (~/.quest/god_items.json).
pub fn god_items_save_path() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })?;
    Ok(home_dir.join(".quest").join("god_items.json"))
}

/// Load god item progress from disk, or return default if not found.
pub fn load_god_item_progress() -> GodItemProgress {
    let path = match god_items_save_path() {
        Ok(p) => p,
        Err(_) => return GodItemProgress::default(),
    };

    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => GodItemProgress::default(),
    }
}

/// Save god item progress to disk.
pub fn save_god_item_progress(progress: &GodItemProgress) -> io::Result<()> {
    let path = god_items_save_path()?;

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(progress)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_god_item_progress_serialization_roundtrip() {
        let progress = GodItemProgress::default();
        let json = serde_json::to_string_pretty(&progress).unwrap();
        let loaded: GodItemProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.asprika_state, progress.asprika_state);
    }

    #[test]
    fn test_god_items_save_path() {
        let result = god_items_save_path();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("god_items.json"));
    }
}
