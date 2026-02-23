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

    // Restore to level 10 commit (resets main back)
    let level10_id = &commits[1].id;
    repo.restore_to(level10_id).unwrap();

    // Verify file contents match level 10
    let content = fs::read_to_string(dir.path().join("char.json")).unwrap();
    assert!(content.contains("\"level\":10"));

    // Still only one branch (main) — no new branches created
    let branches = repo.list_branches().unwrap();
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");

    // main now has 3 commits (init + created + level up), prestige commit is gone
    let commits_after = repo.list_commits("main").unwrap();
    assert_eq!(commits_after.len(), 3);
}

#[test]
fn restore_then_continue() {
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

    // Restore to v1 (resets main)
    let commits = repo.list_commits("main").unwrap();
    let v1_id = &commits[2].id; // third newest = LevelUp(10)
    repo.restore_to(v1_id).unwrap();

    // Verify main was reset to 2 commits (init + v1)
    let commits_after = repo.list_commits("main").unwrap();
    assert_eq!(commits_after.len(), 2);

    // Continue playing from restored state — new commits go on main
    fs::write(dir.path().join("save.json"), "v1-alt").unwrap();
    repo.commit(&SaveEvent::PrestigeRank(1), 50, 1, 1, 1, 400)
        .unwrap();

    // main now has 3 commits (init + v1 + prestige)
    let final_commits = repo.list_commits("main").unwrap();
    assert_eq!(final_commits.len(), 3);
    assert!(final_commits[0].message.contains("Prestige to rank 1"));

    // Still only one branch
    let branches = repo.list_branches().unwrap();
    assert_eq!(branches.len(), 1);
}
