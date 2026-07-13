//! Act 2 ↔ Time Vault interplay (release-polish spec, "Act 2 State
//! Participates In Time Vault Timelines"): the vault repo is the whole
//! quest dir, so `voyage.json`/`colony.json` are snapshotted with every
//! commit and rewound by every restore — the hero and the era rewind
//! together. This pins that semantic in both directions.

use quest::history::git::HistoryRepo;
use quest::history::types::{CommitMetadata, SaveEvent};
use quest::vessel::colony::ColonyState;
use quest::vessel::persistence::{
    load_colony_from_path, load_voyage_from_path, save_colony_to_path, save_voyage_to_path,
};
use quest::vessel::voyage::VoyageState;

fn meta(name: &str) -> CommitMetadata {
    CommitMetadata {
        level: 80,
        prestige: 250_000,
        zone_id: 50,
        subzone_id: 5,
        play_time_seconds: 3_600,
        character_name: name.to_string(),
    }
}

#[test]
fn the_vault_rewinds_the_hero_and_the_era_together() {
    let dir = tempfile::tempdir().unwrap();
    let quest_dir = dir.path();
    let t0 = "2026-07-03T12:00:00Z".parse().unwrap();

    // Commit A: pre-launch — a character save exists, no vessel files.
    // (init commits the empty dir; writing the save afterwards gives the
    // ManualSave commit something to record.)
    let repo = HistoryRepo::init(quest_dir).expect("init");
    std::fs::write(quest_dir.join("hero.json"), r#"{"pre":"launch"}"#).unwrap();
    repo.commit(&SaveEvent::ManualSave, &meta("Hero")).unwrap();

    // Commit B: post-launch — a crossing underway, a colony founded.
    let voyage = VoyageState::begin("hero".to_string(), 7, t0);
    save_voyage_to_path(&voyage, &quest_dir.join("voyage.json")).unwrap();
    let colony = ColonyState::found("hero".to_string());
    save_colony_to_path(&colony, &quest_dir.join("colony.json")).unwrap();
    repo.commit(&SaveEvent::VesselLaunched, &meta("Hero"))
        .unwrap();

    let commits = repo.list_commits("main").expect("list");
    // Newest first: [B (VesselLaunched), A (ManualSave), init].
    let commit_b = commits[0].id.clone();
    let commit_a = commits[1].id.clone();

    // Restore to A: the era rewinds with the hero — the vessel files are gone.
    repo.restore_to(&commit_a).expect("restore to pre-launch");
    assert!(
        !quest_dir.join("voyage.json").exists(),
        "a pre-launch restore removes the in-progress crossing"
    );
    assert!(
        !quest_dir.join("colony.json").exists(),
        "a pre-launch restore removes the colony"
    );
    // A launch from here would begin a FRESH crossing: nothing to load.
    assert!(load_voyage_from_path(&quest_dir.join("voyage.json"), "hero").is_none());

    // Restore forward to B (the commit id survives): the crossing returns
    // intact and loads through the real paths.
    repo.restore_to(&commit_b).expect("restore forward");
    let restored = load_voyage_from_path(&quest_dir.join("voyage.json"), "hero")
        .expect("the crossing came back with the timeline");
    assert_eq!(restored.character_id, "hero");
    assert_eq!(restored.crossing_number, voyage.crossing_number);
    assert!(
        load_colony_from_path(&quest_dir.join("colony.json"), "hero").is_some(),
        "the colony came back with the timeline"
    );
}
