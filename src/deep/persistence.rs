use super::types::DeepState;
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn deep_save_path() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })?;
    Ok(home_dir.join(".quest").join("deep.json"))
}

pub fn load_deep() -> DeepState {
    let path = match deep_save_path() {
        Ok(p) => p,
        Err(_) => return DeepState::new(),
    };
    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
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
