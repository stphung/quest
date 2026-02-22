use super::types::TheDeepState;
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn deep_save_path() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home_dir.join(".quest").join("deep.json")
}

pub fn load_deep() -> TheDeepState {
    let path = deep_save_path();
    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => TheDeepState::new(),
    }
}

pub fn save_deep(state: &TheDeepState) -> io::Result<()> {
    let path = deep_save_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)?;
    Ok(())
}
