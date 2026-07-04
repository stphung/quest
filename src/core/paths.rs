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

#[cfg(test)]
mod tests {
    use super::*;

    /// Env vars are process-global, so override, empty, and fallback cases
    /// share one test to keep set/restore ordering deterministic.
    #[test]
    fn test_quest_dir_env_override() {
        // Pin a known prior value ourselves (rather than trusting whatever the
        // ambient environment happens to have) so `saved` is deterministically
        // `Some(..)` below, exercising the "restore a prior value" branch
        // regardless of whether QUEST_DIR was set before this test ran. This
        // matches the save/restore pattern used elsewhere for this same
        // process-global env var (e.g. haven/logic.rs, character/manager.rs):
        // each guarded test restores to whatever it captured as `saved`, not
        // necessarily the absolute pre-suite value.
        std::env::set_var(QUEST_DIR_ENV, "/tmp/quest-test-prior-marker");
        let saved = std::env::var(QUEST_DIR_ENV).ok();
        assert_eq!(saved.as_deref(), Some("/tmp/quest-test-prior-marker"));

        std::env::set_var(QUEST_DIR_ENV, "/tmp/quest-test-override");
        assert_eq!(
            get_quest_dir().unwrap(),
            PathBuf::from("/tmp/quest-test-override")
        );

        // Empty and whitespace-only values fall back to the default.
        std::env::set_var(QUEST_DIR_ENV, "");
        let dir = get_quest_dir().unwrap();
        assert!(dir.ends_with(".quest"), "got {dir:?}");
        std::env::set_var(QUEST_DIR_ENV, "   ");
        assert!(get_quest_dir().unwrap().ends_with(".quest"));

        std::env::remove_var(QUEST_DIR_ENV);
        assert!(get_quest_dir().unwrap().ends_with(".quest"));

        if let Some(v) = saved {
            std::env::set_var(QUEST_DIR_ENV, v);
        }
    }
}
