//! Centralized save path resolution for the quest save directory.

use std::io;
use std::path::PathBuf;

/// Returns the path to the quest save directory (`~/.quest`).
///
/// All persistence modules must use this function instead of independently
/// calling `dirs::home_dir()` and constructing the path themselves.
pub fn get_quest_dir() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })?;
    Ok(home_dir.join(".quest"))
}
