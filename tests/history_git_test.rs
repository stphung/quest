//! Integration tests for `HistoryRepo` git operations.

use std::fs;

use quest::history::{HistoryError, HistoryRepo, SaveEvent};
use tempfile::TempDir;

/// Create a temp dir with a dummy JSON save file so git has something to commit.
fn setup_quest_dir() -> TempDir {
    let dir = TempDir::new().expect("create temp dir");
    fs::write(
        dir.path().join("save.json"),
        r#"{"level":1,"prestige":0}"#,
    )
    .expect("write save file");
    dir
}

// ── init ────────────────────────────────────────────────────────────────

#[test]
fn init_creates_git_repo() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).expect("init");

    // .git directory should exist.
    assert!(dir.path().join(".git").exists());

    // Should have exactly one branch named "main".
    let branches = repo.list_branches().expect("list_branches");
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");
    assert!(branches[0].is_active);
}

#[test]
fn init_idempotent() {
    let dir = setup_quest_dir();
    let _repo1 = HistoryRepo::init(dir.path()).expect("first init");
    let repo2 = HistoryRepo::init(dir.path()).expect("second init");

    // Still one branch, one commit.
    let branches = repo2.list_branches().expect("list_branches");
    assert_eq!(branches.len(), 1);

    let commits = repo2.list_commits("main").expect("list_commits");
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].message, "Initialize save history");
}

// ── commit ──────────────────────────────────────────────────────────────

#[test]
fn commit_adds_entry() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).expect("init");

    // Modify a file so there is something to commit.
    fs::write(
        dir.path().join("save.json"),
        r#"{"level":5,"prestige":0}"#,
    )
    .expect("write");

    repo.commit(&SaveEvent::LevelUp(5), 5, 0, 1, 1, 600)
        .expect("commit");

    let commits = repo.list_commits("main").expect("list_commits");
    assert_eq!(commits.len(), 2);
    assert!(commits[0].message.starts_with("Level up to 5"));
}

#[test]
fn commit_skipped_when_no_changes() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).expect("init");

    // No file changes => should return NothingToCommit.
    let result = repo.commit(&SaveEvent::ManualSave, 1, 0, 1, 1, 0);
    assert!(matches!(result, Err(HistoryError::NothingToCommit)));

    // Still just the initial commit.
    let commits = repo.list_commits("main").expect("list_commits");
    assert_eq!(commits.len(), 1);
}

// ── list_commits ────────────────────────────────────────────────────────

#[test]
fn list_commits_newest_first() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).expect("init");

    // First change.
    fs::write(dir.path().join("save.json"), r#"{"level":2}"#).expect("write");
    repo.commit(&SaveEvent::LevelUp(2), 2, 0, 1, 1, 300)
        .expect("commit 1");

    // Second change.
    fs::write(dir.path().join("save.json"), r#"{"level":3}"#).expect("write");
    repo.commit(&SaveEvent::LevelUp(3), 3, 0, 1, 1, 600)
        .expect("commit 2");

    let commits = repo.list_commits("main").expect("list_commits");
    assert_eq!(commits.len(), 3); // init + 2 commits

    // Newest first.
    assert!(commits[0].message.contains("Level up to 3"));
    assert!(commits[1].message.contains("Level up to 2"));
    assert!(commits[2].message.contains("Initialize save history"));
}

// ── restore_to ──────────────────────────────────────────────────────────

#[test]
fn restore_resets_branch_to_commit() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).expect("init");

    // Create a second commit with different file content.
    fs::write(dir.path().join("save.json"), r#"{"level":10}"#).expect("write");
    repo.commit(&SaveEvent::LevelUp(10), 10, 0, 2, 1, 3600)
        .expect("commit");

    // Get the initial commit id.
    let commits = repo.list_commits("main").expect("list_commits");
    assert_eq!(commits.len(), 2);
    let initial_commit_id = &commits[1].id;

    // Restore to the initial commit (resets main).
    repo.restore_to(initial_commit_id).expect("restore_to");

    // File should have original content (from the initial commit).
    let content = fs::read_to_string(dir.path().join("save.json")).expect("read");
    assert_eq!(content, r#"{"level":1,"prestige":0}"#);

    // Still only one branch (main), no new branches created.
    let branches = repo.list_branches().expect("list_branches");
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");

    // main should now have only the initial commit.
    let commits_after = repo.list_commits("main").expect("list_commits");
    assert_eq!(commits_after.len(), 1);
    assert_eq!(commits_after[0].message, "Initialize save history");
}
