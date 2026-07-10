//! Shared helpers for history tests.

use quest::history::types::CommitMetadata;

/// Build a CommitMetadata for tests with sensible defaults.
pub fn meta(
    level: u32,
    prestige: u32,
    zone_id: u32,
    subzone_id: u32,
    play_time_seconds: u64,
    character_name: &str,
) -> CommitMetadata {
    CommitMetadata {
        level,
        prestige,
        zone_id,
        subzone_id,
        play_time_seconds,
        character_name: character_name.to_string(),
    }
}
