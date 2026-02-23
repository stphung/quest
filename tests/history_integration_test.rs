use quest::history::git::HistoryRepo;
use quest::history::types::SaveEvent;
use std::fs;
use tempfile::TempDir;

#[test]
fn full_save_restore_round_trip() {
    let dir = TempDir::new().unwrap();
    let repo = HistoryRepo::init(dir.path()).unwrap();

    // Simulate: initial save
    fs::write(dir.path().join("char.json"), r#"{"level":1,"zone":1}"#).unwrap();
    repo.commit(
        &SaveEvent::CharacterCreated("Hero".to_string()),
        1,
        0,
        1,
        1,
        0,
    )
    .unwrap();

    // Simulate: level up
    fs::write(dir.path().join("char.json"), r#"{"level":10,"zone":1}"#).unwrap();
    repo.commit(&SaveEvent::LevelUp(10), 10, 0, 1, 3, 600)
        .unwrap();

    // Simulate: prestige
    fs::write(dir.path().join("char.json"), r#"{"level":50,"zone":3}"#).unwrap();
    repo.commit(&SaveEvent::PrestigeRank(1), 50, 1, 3, 2, 3600)
        .unwrap();

    // Verify 4 commits (init + 3)
    let commits = repo.list_commits("main").unwrap();
    assert_eq!(commits.len(), 4);
    assert!(commits[0].message.contains("Prestige to rank 1"));

    // Restore to level 10 commit (newest-first: [0]=Prestige, [1]=LevelUp, [2]=Created, [3]=Init)
    let level10_id = &commits[1].id;
    let new_branch = repo.restore_to(level10_id).unwrap();
    assert_eq!(new_branch, "timeline-1");

    // Verify file contents match level 10
    let content = fs::read_to_string(dir.path().join("char.json")).unwrap();
    assert!(content.contains("\"level\":10"));

    // Switch back to main
    repo.switch_branch("main").unwrap();
    let content = fs::read_to_string(dir.path().join("char.json")).unwrap();
    assert!(content.contains("\"level\":50"));

    // Verify 2 branches
    let branches = repo.list_branches().unwrap();
    assert_eq!(branches.len(), 2);
}

#[test]
fn multiple_timelines() {
    let dir = TempDir::new().unwrap();
    let repo = HistoryRepo::init(dir.path()).unwrap();

    fs::write(dir.path().join("save.json"), "v1").unwrap();
    repo.commit(&SaveEvent::LevelUp(10), 10, 0, 1, 1, 100)
        .unwrap();

    fs::write(dir.path().join("save.json"), "v2").unwrap();
    repo.commit(&SaveEvent::LevelUp(20), 20, 0, 2, 1, 200)
        .unwrap();

    fs::write(dir.path().join("save.json"), "v3").unwrap();
    repo.commit(&SaveEvent::LevelUp(30), 30, 0, 3, 1, 300)
        .unwrap();

    // Restore to v1 -> creates timeline-1
    let commits = repo.list_commits("main").unwrap();
    let v1_id = &commits[2].id; // third newest = LevelUp(10)
    repo.restore_to(v1_id).unwrap();

    // Make progress on timeline-1
    fs::write(dir.path().join("save.json"), "t1-v2").unwrap();
    repo.commit(&SaveEvent::PrestigeRank(1), 50, 1, 1, 1, 400)
        .unwrap();

    // Restore again from timeline-1 -> creates timeline-2
    let t1_commits = repo.list_commits("timeline-1").unwrap();
    let t1_first = &t1_commits[1].id; // the v1 commit
    repo.restore_to(t1_first).unwrap();

    // Should now have 3 branches
    let branches = repo.list_branches().unwrap();
    assert_eq!(branches.len(), 3);
    assert!(branches.iter().any(|b| b.name == "main"));
    assert!(branches.iter().any(|b| b.name == "timeline-1"));
    assert!(branches.iter().any(|b| b.name == "timeline-2"));
}
