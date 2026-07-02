use super::types::DeepState;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn deep_save_path() -> io::Result<PathBuf> {
    Ok(crate::core::paths::get_quest_dir()?.join("deep.json"))
}

pub fn load_deep() -> DeepState {
    match deep_save_path() {
        Ok(path) => load_deep_from(&path),
        Err(_) => DeepState::new(),
    }
}

fn load_deep_from(path: &Path) -> DeepState {
    match fs::read_to_string(path) {
        Ok(json) => match serde_json::from_str(&json) {
            Ok(state) => state,
            Err(_) => {
                // Unparseable save: preserve a copy before falling back to a
                // fresh state, so a schema bug can't silently destroy progress.
                let _ = fs::copy(path, path.with_extension("json.bak"));
                DeepState::new()
            }
        },
        Err(_) => DeepState::new(),
    }
}

pub fn save_deep(deep: &DeepState) -> io::Result<()> {
    let path = deep_save_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(deep)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_deep_from_missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let state = load_deep_from(&dir.path().join("deep.json"));
        assert!(!state.persistent.discovered);
    }

    #[test]
    fn test_load_deep_from_valid_file_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("deep.json");
        let mut state = DeepState::new();
        state.persistent.discovered = true;
        state.session.warband_marks = 777;
        fs::write(&path, serde_json::to_string(&state).unwrap()).unwrap();

        let loaded = load_deep_from(&path);
        assert!(loaded.persistent.discovered);
        assert_eq!(loaded.session.warband_marks, 777);
    }

    #[test]
    fn test_load_deep_from_corrupt_file_backs_up_before_reset() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("deep.json");
        fs::write(&path, "{ not valid json").unwrap();

        let state = load_deep_from(&path);
        assert!(!state.persistent.discovered);
        let backup = fs::read_to_string(dir.path().join("deep.json.bak")).unwrap();
        assert_eq!(backup, "{ not valid json");
    }
}
