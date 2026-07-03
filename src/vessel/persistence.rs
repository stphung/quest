//! Voyage persistence — `voyage.json` in the quest dir (Deep/Loom pattern),
//! keyed by character id so a different character never inherits a crossing.

use super::voyage::VoyageState;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn voyage_save_path() -> io::Result<PathBuf> {
    Ok(crate::core::paths::get_quest_dir()?.join("voyage.json"))
}

/// Load the voyage for `character_id`, if one is underway. A missing file,
/// unreadable JSON, or another character's voyage all return `None` — the
/// caller treats that as "no crossing started".
pub fn load_voyage(character_id: &str) -> Option<VoyageState> {
    let path = voyage_save_path().ok()?;
    load_voyage_from_path(&path, character_id)
}

pub fn load_voyage_from_path(path: &Path, character_id: &str) -> Option<VoyageState> {
    let json = fs::read_to_string(path).ok()?;
    let state: VoyageState = serde_json::from_str(&json).ok()?;
    (state.character_id == character_id).then_some(state)
}

pub fn save_voyage(voyage: &VoyageState) -> io::Result<()> {
    let path = voyage_save_path()?;
    save_voyage_to_path(voyage, &path)
}

pub fn save_voyage_to_path(voyage: &VoyageState, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(voyage)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vessel::route::{self, ROUTE_START};
    use crate::vessel::voyage::{Trim, VoyagePhase};
    use chrono::{DateTime, Utc};

    fn t0() -> DateTime<Utc> {
        "2026-07-03T12:00:00Z".parse().unwrap()
    }

    #[test]
    fn roundtrip_preserves_the_crossing_exactly() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("voyage.json");

        let mut v = VoyageState::begin("char-1".to_string(), 42, t0());
        v.play_arrival_scene();
        v.set_trim(Trim::Quiet);
        v.depart(route::roads_from(ROUTE_START).next().unwrap().id)
            .unwrap();
        v.tick(t0() + chrono::Duration::hours(9));

        save_voyage_to_path(&v, &path).unwrap();
        let loaded = load_voyage_from_path(&path, "char-1").unwrap();

        assert_eq!(loaded.phase, v.phase);
        assert_eq!(loaded.trim, v.trim);
        // Bitwise: offline equivalence depends on exact f64 roundtrips.
        assert_eq!(loaded.provisions.to_bits(), v.provisions.to_bits());
        assert_eq!(loaded.processed_minutes, v.processed_minutes);
        assert_eq!(loaded.voyage_seed, v.voyage_seed);
        assert_eq!(loaded.visited, v.visited);
    }

    #[test]
    fn another_characters_voyage_does_not_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("voyage.json");
        let v = VoyageState::begin("char-1".to_string(), 42, t0());
        save_voyage_to_path(&v, &path).unwrap();
        assert!(load_voyage_from_path(&path, "char-2").is_none());
    }

    #[test]
    fn missing_or_corrupt_file_means_no_crossing() {
        assert!(load_voyage_from_path(Path::new("/nonexistent/voyage.json"), "c").is_none());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("voyage.json");
        fs::write(&path, "{{not json").unwrap();
        assert!(load_voyage_from_path(&path, "c").is_none());
    }

    #[test]
    fn loaded_voyage_keeps_ticking_from_where_it_stood() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("voyage.json");

        let mut v = VoyageState::begin("char-1".to_string(), 42, t0());
        v.play_arrival_scene();
        v.depart(route::roads_from(ROUTE_START).next().unwrap().id)
            .unwrap();
        v.tick(t0() + chrono::Duration::hours(6));
        save_voyage_to_path(&v, &path).unwrap();

        // "Come back the next day": load and lazy-tick across the gap.
        let mut loaded = load_voyage_from_path(&path, "char-1").unwrap();
        loaded.tick(t0() + chrono::Duration::hours(30));
        assert!(
            matches!(loaded.phase, VoyagePhase::HoldingStation { .. }),
            "the 1-day leg completed offline"
        );
    }
}
