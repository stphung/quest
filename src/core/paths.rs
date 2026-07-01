//! Centralized save path resolution for the quest save directory.

use std::io;
use std::path::PathBuf;

/// Environment variable that overrides the save directory location.
/// Used by test harnesses and fixture tooling to run the game against
/// an isolated state directory instead of the player's real saves.
pub const QUEST_DIR_ENV: &str = "QUEST_DIR";

/// Returns the path to the quest save directory.
///
/// Defaults to `~/.quest`. Set the `QUEST_DIR` environment variable to
/// point at a different directory (must be non-empty).
///
/// All persistence modules must use this function instead of independently
/// calling `dirs::home_dir()` and constructing the path themselves.
pub fn get_quest_dir() -> io::Result<PathBuf> {
    if let Ok(dir) = std::env::var(QUEST_DIR_ENV) {
        if !dir.trim().is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }
    let home_dir = dirs::home_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })?;
    Ok(home_dir.join(".quest"))
}
