use super::types::EnhancementProgress;
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn enhancement_save_path() -> io::Result<PathBuf> {
    Ok(crate::core::paths::get_quest_dir()?.join("enhancement.json"))
}

pub fn load_enhancement() -> EnhancementProgress {
    let path = match enhancement_save_path() {
        Ok(p) => p,
        Err(_) => return EnhancementProgress::new(),
    };
    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => EnhancementProgress::new(),
    }
}

pub fn save_enhancement(enhancement: &EnhancementProgress) -> io::Result<()> {
    let path = enhancement_save_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(enhancement)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)?;
    Ok(())
}
