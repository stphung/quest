use super::types::LoomState;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn loom_save_path() -> io::Result<PathBuf> {
    Ok(crate::core::paths::get_quest_dir()?.join("loom.json"))
}

pub fn load_loom() -> LoomState {
    let path = match loom_save_path() {
        Ok(p) => p,
        Err(_) => return LoomState::new(),
    };
    load_loom_from_path(&path)
}

pub fn load_loom_from_path(path: &Path) -> LoomState {
    match fs::read_to_string(path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => LoomState::new(),
    }
}

pub fn save_loom(loom: &LoomState) -> io::Result<()> {
    let path = loom_save_path()?;
    save_loom_to_path(loom, &path)
}

pub fn save_loom_to_path(loom: &LoomState, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(loom)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loom::types::LoomArchetype;

    #[test]
    fn test_loom_save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("loom.json");

        let mut state = LoomState::new();
        state.persistent.discovered = true;
        state.persistent.archetype = Some(LoomArchetype::BurnBright);

        save_loom_to_path(&state, &path).unwrap();
        let loaded = load_loom_from_path(&path);

        assert!(loaded.persistent.discovered);
        assert_eq!(loaded.persistent.archetype, Some(LoomArchetype::BurnBright));
        assert_eq!(loaded.persistent.nodes.len(), 6);
    }

    #[test]
    fn test_loom_load_missing_file_returns_default() {
        let loaded = load_loom_from_path(Path::new("/nonexistent/loom.json"));
        assert!(!loaded.persistent.discovered);
    }
}
