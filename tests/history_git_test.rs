//! Integration tests for `HistoryRepo` git operations.

use std::fs;

use quest::history::{HistoryError, HistoryRepo, SaveEvent};
use tempfile::TempDir;

/// Create a temp dir with a dummy JSON save file so git has something to commit.
fn setup_quest_dir() -> TempDir {
    let dir = TempDir::new().expect("create temp dir");
    fs::write(dir.path().join("save.json"), r#"{"level":1,"prestige":0}"#)
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
    fs::write(dir.path().join("save.json"), r#"{"level":5,"prestige":0}"#).expect("write");

    repo.commit(&SaveEvent::LevelUp(5), 5, 0, 1, 1, 600, "Hero")
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
    let result = repo.commit(&SaveEvent::ManualSave, 1, 0, 1, 1, 0, "Hero");
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
    repo.commit(&SaveEvent::LevelUp(2), 2, 0, 1, 1, 300, "Hero")
        .expect("commit 1");

    // Second change.
    fs::write(dir.path().join("save.json"), r#"{"level":3}"#).expect("write");
    repo.commit(&SaveEvent::LevelUp(3), 3, 0, 1, 1, 600, "Hero")
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
    repo.commit(&SaveEvent::LevelUp(10), 10, 0, 2, 1, 3600, "Hero")
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

// ── fork_timeline ──────────────────────────────────────────────────────

#[test]
fn fork_creates_branch_at_commit() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).expect("init");

    fs::write(dir.path().join("save.json"), r#"{"level":5}"#).expect("write");
    repo.commit(&SaveEvent::LevelUp(5), 5, 0, 1, 1, 600, "Hero")
        .expect("commit");

    fs::write(dir.path().join("save.json"), r#"{"level":10}"#).expect("write");
    repo.commit(&SaveEvent::LevelUp(10), 10, 0, 2, 1, 1200, "Hero")
        .expect("commit");

    // Fork at the first commit (oldest).
    let commits = repo.list_commits("main").expect("list_commits");
    let oldest_id = &commits.last().unwrap().id;

    repo.fork_timeline("alt", oldest_id).expect("fork");

    let branches = repo.list_branches().expect("list_branches");
    let names: Vec<&str> = branches.iter().map(|b| b.name.as_str()).collect();
    assert!(names.contains(&"alt"));
    assert!(names.contains(&"main"));

    // The new branch should be active.
    let active = branches.iter().find(|b| b.is_active).unwrap();
    assert_eq!(active.name, "alt");
}

#[test]
fn fork_rejects_duplicate_name() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).expect("init");

    fs::write(dir.path().join("save.json"), r#"{"level":5}"#).expect("write");
    repo.commit(&SaveEvent::LevelUp(5), 5, 0, 1, 1, 600, "Hero")
        .expect("commit");

    let commits = repo.list_commits("main").expect("list_commits");
    let commit_id = &commits[0].id;

    repo.fork_timeline("branch-a", commit_id).expect("fork");
    repo.switch_timeline("main").expect("switch back");

    let result = repo.fork_timeline("branch-a", commit_id);
    assert!(result.is_err());
}

#[test]
fn fork_rejects_invalid_names() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).expect("init");

    fs::write(dir.path().join("save.json"), r#"{"level":5}"#).expect("write");
    repo.commit(&SaveEvent::LevelUp(5), 5, 0, 1, 1, 600, "Hero")
        .expect("commit");

    let commits = repo.list_commits("main").expect("list_commits");
    let commit_id = &commits[0].id;

    assert!(repo.fork_timeline("main", commit_id).is_err());
    assert!(repo.fork_timeline("", commit_id).is_err());
    assert!(repo.fork_timeline("-bad", commit_id).is_err());
    assert!(repo.fork_timeline("UPPER", commit_id).is_err());
    assert!(repo.fork_timeline("has spaces", commit_id).is_err());
}

// ── switch_timeline ────────────────────────────────────────────────────

#[test]
fn switch_changes_active_branch() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).expect("init");

    fs::write(dir.path().join("save.json"), r#"{"level":5}"#).expect("write");
    repo.commit(&SaveEvent::LevelUp(5), 5, 0, 1, 1, 600, "Hero")
        .expect("commit");

    let commits = repo.list_commits("main").expect("list_commits");
    let commit_id = &commits[0].id;

    repo.fork_timeline("alt", commit_id).expect("fork");

    // Now on alt, switch back to main.
    repo.switch_timeline("main").expect("switch");

    let branches = repo.list_branches().expect("list_branches");
    let active = branches.iter().find(|b| b.is_active).unwrap();
    assert_eq!(active.name, "main");
}

#[test]
fn switch_fails_on_nonexistent() {
    let dir = setup_quest_dir();
    let _repo = HistoryRepo::init(dir.path()).expect("init");
    let result = _repo.switch_timeline("nope");
    assert!(result.is_err());
}

// ── delete_timeline ────────────────────────────────────────────────────

#[test]
fn delete_removes_branch() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).expect("init");

    fs::write(dir.path().join("save.json"), r#"{"level":5}"#).expect("write");
    repo.commit(&SaveEvent::LevelUp(5), 5, 0, 1, 1, 600, "Hero")
        .expect("commit");

    let commits = repo.list_commits("main").expect("list_commits");
    let commit_id = &commits[0].id;

    repo.fork_timeline("to-delete", commit_id).expect("fork");
    repo.switch_timeline("main").expect("switch back");

    repo.delete_timeline("to-delete").expect("delete");

    let branches = repo.list_branches().expect("list_branches");
    let names: Vec<&str> = branches.iter().map(|b| b.name.as_str()).collect();
    assert!(!names.contains(&"to-delete"));
}

#[test]
fn delete_fails_on_main() {
    let dir = setup_quest_dir();
    let _repo = HistoryRepo::init(dir.path()).expect("init");
    let result = _repo.delete_timeline("main");
    assert!(result.is_err());
}

#[test]
fn delete_fails_on_active_branch() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).expect("init");

    fs::write(dir.path().join("save.json"), r#"{"level":5}"#).expect("write");
    repo.commit(&SaveEvent::LevelUp(5), 5, 0, 1, 1, 600, "Hero")
        .expect("commit");

    let commits = repo.list_commits("main").expect("list_commits");
    let commit_id = &commits[0].id;

    repo.fork_timeline("active-one", commit_id).expect("fork");

    // Currently on "active-one" — should not be deletable.
    let result = repo.delete_timeline("active-one");
    assert!(result.is_err());
}

#[test]
fn delete_fails_on_nonexistent() {
    let dir = setup_quest_dir();
    let _repo = HistoryRepo::init(dir.path()).expect("init");
    let result = _repo.delete_timeline("ghost");
    assert!(result.is_err());
}

// ── fork + switch + delete lifecycle ───────────────────────────────────

#[test]
fn fork_switch_delete_lifecycle() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).expect("init");

    // Add some history on main.
    fs::write(dir.path().join("save.json"), r#"{"level":5}"#).expect("write");
    repo.commit(&SaveEvent::LevelUp(5), 5, 0, 1, 1, 600, "Hero")
        .expect("commit");

    fs::write(dir.path().join("save.json"), r#"{"level":10}"#).expect("write");
    repo.commit(&SaveEvent::LevelUp(10), 10, 0, 2, 1, 1200, "Hero")
        .expect("commit");

    // Fork at level 5 commit.
    let commits = repo.list_commits("main").expect("list_commits");
    let level_5_id = &commits[1].id; // second newest = level 5 commit

    repo.fork_timeline("speedrun", level_5_id).expect("fork");

    // We're now on speedrun. Add a commit there.
    fs::write(dir.path().join("save.json"), r#"{"level":6}"#).expect("write");
    repo.commit(&SaveEvent::LevelUp(6), 6, 0, 1, 2, 700, "Hero")
        .expect("commit on fork");

    // Switch back to main.
    repo.switch_timeline("main").expect("switch to main");

    // Verify main still has its commits.
    let main_commits = repo.list_commits("main").expect("list main commits");
    assert_eq!(main_commits.len(), 3); // init + lv5 + lv10

    // Verify speedrun has its own history.
    let speedrun_commits = repo.list_commits("speedrun").expect("list speedrun commits");
    // Should have: init + lv5 (forked from main) + lv6 (added on fork)
    assert_eq!(speedrun_commits.len(), 3);

    // Delete speedrun while on main.
    repo.delete_timeline("speedrun").expect("delete speedrun");

    let branches = repo.list_branches().expect("list_branches");
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");
}
