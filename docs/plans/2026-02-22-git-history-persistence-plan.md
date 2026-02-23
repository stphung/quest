# Git-Based Save History Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add git-based versioning to `~/.quest/` so players can restore to previous save points via an in-game Timeline Browser.

**Architecture:** New `src/history/` module wraps `git2` operations. `save_all()` gains an `Option<SaveEvent>` parameter — `None` for autosaves (no commit), `Some(event)` for significant events (commit). New two-panel Timeline Browser overlay for restore/branch switching.

**Tech Stack:** Rust, git2 crate, Ratatui (existing)

**Design Doc:** `docs/plans/2026-02-22-git-history-persistence-design.md`

---

### Task 1: Add git2 dependency

**Files:**
- Modify: `Cargo.toml:7-21`

**Step 1: Add git2 to dependencies**

Add `git2` to `Cargo.toml` dependencies:

```toml
git2 = "0.20"
```

Add it alphabetically after `flate2`.

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully with new dependency

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add git2 dependency for save history"
```

---

### Task 2: Create history types module

**Files:**
- Create: `src/history/mod.rs`
- Create: `src/history/types.rs`
- Modify: `src/lib.rs:1-15` (add `pub mod history;`)

**Step 1: Write the failing test**

Create `tests/history_types_test.rs`:

```rust
use quest::history::types::{SaveEvent, CommitInfo, TimelineInfo};

#[test]
fn save_event_commit_message_level_up() {
    let event = SaveEvent::LevelUp(15);
    assert_eq!(event.description(), "Level up to 15");
}

#[test]
fn save_event_commit_message_prestige() {
    let event = SaveEvent::PrestigeRank(5);
    assert_eq!(event.description(), "Prestige to rank 5");
}

#[test]
fn save_event_commit_message_zone_boss() {
    let event = SaveEvent::ZoneBossDefeated("Dark Forest".to_string());
    assert_eq!(event.description(), "Defeated Dark Forest boss");
}

#[test]
fn save_event_commit_message_zone_unlocked() {
    let event = SaveEvent::ZoneUnlocked("Mountain Pass".to_string());
    assert_eq!(event.description(), "Unlocked Mountain Pass");
}

#[test]
fn save_event_commit_message_dungeon_completed() {
    let event = SaveEvent::DungeonCompleted("Medium".to_string());
    assert_eq!(event.description(), "Completed Medium dungeon");
}

#[test]
fn save_event_commit_message_fishing_rank() {
    let event = SaveEvent::FishingRankUp(12);
    assert_eq!(event.description(), "Fishing rank up to 12");
}

#[test]
fn save_event_commit_message_storm_leviathan() {
    let event = SaveEvent::StormLeviathanCaught;
    assert_eq!(event.description(), "Caught the Storm Leviathan");
}

#[test]
fn save_event_commit_message_haven_room() {
    let event = SaveEvent::HavenRoomBuilt("Armory".to_string());
    assert_eq!(event.description(), "Built Armory in Haven");
}

#[test]
fn save_event_commit_message_haven_upgrade() {
    let event = SaveEvent::HavenRoomUpgraded("Armory".to_string(), 2);
    assert_eq!(event.description(), "Upgraded Armory to T2");
}

#[test]
fn save_event_commit_message_soulforge() {
    let event = SaveEvent::SoulforgeEnhanced("Weapon".to_string(), 7);
    assert_eq!(event.description(), "Enhanced Weapon to +7");
}

#[test]
fn save_event_commit_message_challenge_won() {
    let event = SaveEvent::ChallengeWon("Chess".to_string(), "Master".to_string());
    assert_eq!(event.description(), "Won Chess at Master");
}

#[test]
fn save_event_commit_message_god_item() {
    let event = SaveEvent::GodItemForged("Asprika".to_string());
    assert_eq!(event.description(), "Forged Asprika");
}

#[test]
fn save_event_commit_message_character_created() {
    let event = SaveEvent::CharacterCreated("Odin".to_string());
    assert_eq!(event.description(), "Created character Odin");
}

#[test]
fn save_event_commit_message_character_deleted() {
    let event = SaveEvent::CharacterDeleted("Loki".to_string());
    assert_eq!(event.description(), "Deleted character Loki");
}

#[test]
fn save_event_commit_message_equipment() {
    let event = SaveEvent::EquipmentUpgrade("Legendary Sword".to_string());
    assert_eq!(event.description(), "Equipped Legendary Sword");
}

#[test]
fn save_event_commit_message_sigil() {
    let event = SaveEvent::StormSigilActivated("Battle Fury".to_string());
    assert_eq!(event.description(), "Activated Storm Sigil: Battle Fury");
}

#[test]
fn save_event_commit_message_manual() {
    let event = SaveEvent::ManualSave;
    assert_eq!(event.description(), "Manual save");
}

#[test]
fn save_event_format_suffix() {
    let suffix = SaveEvent::format_suffix(18, 0, 2, 3, 135);
    assert_eq!(suffix, "Lv18 P0 Z2-3 2h15m");
}

#[test]
fn save_event_format_suffix_large_playtime() {
    let suffix = SaveEvent::format_suffix(50, 20, 8, 4, 162600);
    assert_eq!(suffix, "Lv50 P20 Z8-4 45h10m");
}

#[test]
fn save_event_full_commit_message() {
    let event = SaveEvent::ZoneBossDefeated("Dark Forest".to_string());
    let msg = event.commit_message(18, 0, 2, 3, 135);
    assert_eq!(msg, "Defeated Dark Forest boss | Lv18 P0 Z2-3 2h15m");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test history_types_test`
Expected: FAIL — module `history` not found

**Step 3: Write minimal implementation**

Create `src/history/types.rs`:

```rust
//! Types for the git-based save history system.

/// Events that trigger a git commit of the save state.
/// Each variant produces a human-readable commit message.
#[derive(Debug, Clone)]
pub enum SaveEvent {
    // Milestone progression
    LevelUp(u32),
    PrestigeRank(u32),
    ZoneBossDefeated(String),
    ZoneUnlocked(String),
    DungeonCompleted(String),
    FishingRankUp(u32),
    StormLeviathanCaught,

    // State-changing actions
    HavenRoomBuilt(String),
    HavenRoomUpgraded(String, u8),
    SoulforgeEnhanced(String, u8),
    ChallengeWon(String, String),
    GodItemForged(String),
    CharacterCreated(String),
    CharacterDeleted(String),
    EquipmentUpgrade(String),
    StormSigilActivated(String),

    // Manual
    ManualSave,
}

impl SaveEvent {
    /// Human-readable description of the event (first part of commit message).
    pub fn description(&self) -> String {
        match self {
            SaveEvent::LevelUp(level) => format!("Level up to {level}"),
            SaveEvent::PrestigeRank(rank) => format!("Prestige to rank {rank}"),
            SaveEvent::ZoneBossDefeated(zone) => format!("Defeated {zone} boss"),
            SaveEvent::ZoneUnlocked(zone) => format!("Unlocked {zone}"),
            SaveEvent::DungeonCompleted(size) => format!("Completed {size} dungeon"),
            SaveEvent::FishingRankUp(rank) => format!("Fishing rank up to {rank}"),
            SaveEvent::StormLeviathanCaught => "Caught the Storm Leviathan".to_string(),
            SaveEvent::HavenRoomBuilt(room) => format!("Built {room} in Haven"),
            SaveEvent::HavenRoomUpgraded(room, tier) => format!("Upgraded {room} to T{tier}"),
            SaveEvent::SoulforgeEnhanced(slot, level) => format!("Enhanced {slot} to +{level}"),
            SaveEvent::ChallengeWon(game, difficulty) => format!("Won {game} at {difficulty}"),
            SaveEvent::GodItemForged(name) => format!("Forged {name}"),
            SaveEvent::CharacterCreated(name) => format!("Created character {name}"),
            SaveEvent::CharacterDeleted(name) => format!("Deleted character {name}"),
            SaveEvent::EquipmentUpgrade(name) => format!("Equipped {name}"),
            SaveEvent::StormSigilActivated(name) => format!("Activated Storm Sigil: {name}"),
            SaveEvent::ManualSave => "Manual save".to_string(),
        }
    }

    /// Format the status suffix: "Lv{level} P{prestige} Z{zone}-{subzone} {h}h{m}m"
    pub fn format_suffix(
        level: u32,
        prestige: u32,
        zone_id: u32,
        subzone_id: u32,
        play_time_seconds: u64,
    ) -> String {
        let hours = play_time_seconds / 3600;
        let minutes = (play_time_seconds % 3600) / 60;
        format!("Lv{level} P{prestige} Z{zone_id}-{subzone_id} {hours}h{minutes:02}m")
    }

    /// Full commit message: "{description} | {suffix}"
    pub fn commit_message(
        &self,
        level: u32,
        prestige: u32,
        zone_id: u32,
        subzone_id: u32,
        play_time_seconds: u64,
    ) -> String {
        let desc = self.description();
        let suffix = Self::format_suffix(level, prestige, zone_id, subzone_id, play_time_seconds);
        format!("{desc} | {suffix}")
    }
}

/// Information about a single commit in the save history.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Git object ID.
    pub id: String,
    /// Event description (first part of commit message, before " | ").
    pub message: String,
    /// Commit timestamp as Unix seconds.
    pub timestamp: i64,
    /// Parsed from suffix.
    pub level: u32,
    /// Parsed from suffix.
    pub prestige: u32,
    /// Parsed zone string (e.g., "2-3").
    pub zone: String,
    /// Parsed playtime string (e.g., "2h15m").
    pub playtime: String,
}

/// Information about a timeline (git branch).
#[derive(Debug, Clone)]
pub struct TimelineInfo {
    /// Branch name (e.g., "main", "timeline-1").
    pub name: String,
    /// Whether this is the currently checked-out branch.
    pub is_active: bool,
    /// The most recent commit on this branch.
    pub head_commit: Option<CommitInfo>,
}
```

Create `src/history/mod.rs`:

```rust
//! Git-based save history for checkpoint and restore.

pub mod types;

pub use types::{CommitInfo, SaveEvent, TimelineInfo};
```

Add to `src/lib.rs` (after `pub mod haven;`, before `pub mod items;`):

```rust
pub mod history;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test history_types_test`
Expected: All 20 tests PASS

**Step 5: Commit**

```bash
git add src/history/ src/lib.rs tests/history_types_test.rs
git commit -m "feat(history): add SaveEvent types with commit message formatting"
```

---

### Task 3: Create history git operations module

**Files:**
- Create: `src/history/git.rs`
- Modify: `src/history/mod.rs`

**Step 1: Write the failing test**

Create `tests/history_git_test.rs`:

```rust
use quest::history::git::{HistoryRepo, HistoryError};
use quest::history::types::SaveEvent;
use std::fs;
use tempfile::TempDir;

fn setup_quest_dir() -> TempDir {
    let dir = TempDir::new().unwrap();
    // Create a dummy save file so there's something to commit
    fs::write(dir.path().join("TestChar.json"), r#"{"level": 1}"#).unwrap();
    dir
}

#[test]
fn init_creates_git_repo() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).unwrap();
    assert!(dir.path().join(".git").exists());
    // Should have an initial commit
    let branches = repo.list_branches().unwrap();
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");
    assert!(branches[0].is_active);
}

#[test]
fn init_idempotent_on_existing_repo() {
    let dir = setup_quest_dir();
    let _repo1 = HistoryRepo::init(dir.path()).unwrap();
    let repo2 = HistoryRepo::init(dir.path()).unwrap();
    // Should not error, should have same single branch
    let branches = repo2.list_branches().unwrap();
    assert_eq!(branches.len(), 1);
}

#[test]
fn commit_adds_entry_to_log() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).unwrap();

    // Modify a file to create a diff
    fs::write(dir.path().join("TestChar.json"), r#"{"level": 5}"#).unwrap();

    let event = SaveEvent::LevelUp(5);
    repo.commit(&event, 5, 0, 1, 1, 300).unwrap();

    let commits = repo.list_commits("main").unwrap();
    assert_eq!(commits.len(), 2); // init + our commit
    assert_eq!(commits[0].message, "Level up to 5");
    assert!(commits[0].playtime == "0h05m");
}

#[test]
fn commit_skipped_when_no_changes() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).unwrap();

    // No file changes since init
    let event = SaveEvent::ManualSave;
    let result = repo.commit(&event, 1, 0, 1, 1, 0);

    // Should succeed but not create a new commit (nothing to commit)
    assert!(result.is_ok());
    let commits = repo.list_commits("main").unwrap();
    assert_eq!(commits.len(), 1); // only init
}

#[test]
fn list_commits_returns_newest_first() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).unwrap();

    fs::write(dir.path().join("TestChar.json"), r#"{"level": 5}"#).unwrap();
    repo.commit(&SaveEvent::LevelUp(5), 5, 0, 1, 1, 300).unwrap();

    fs::write(dir.path().join("TestChar.json"), r#"{"level": 10}"#).unwrap();
    repo.commit(&SaveEvent::LevelUp(10), 10, 0, 1, 2, 600).unwrap();

    let commits = repo.list_commits("main").unwrap();
    assert_eq!(commits.len(), 3); // init + 2
    assert_eq!(commits[0].message, "Level up to 10");
    assert_eq!(commits[1].message, "Level up to 5");
}

#[test]
fn restore_creates_new_branch() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).unwrap();

    fs::write(dir.path().join("TestChar.json"), r#"{"level": 5}"#).unwrap();
    repo.commit(&SaveEvent::LevelUp(5), 5, 0, 1, 1, 300).unwrap();

    fs::write(dir.path().join("TestChar.json"), r#"{"level": 10}"#).unwrap();
    repo.commit(&SaveEvent::LevelUp(10), 10, 0, 1, 2, 600).unwrap();

    // Restore to the level-5 commit
    let commits = repo.list_commits("main").unwrap();
    let level5_id = &commits[1].id; // second newest = level 5

    let new_branch = repo.restore_to(level5_id).unwrap();
    assert_eq!(new_branch, "timeline-1");

    // File should be back to level 5
    let content = fs::read_to_string(dir.path().join("TestChar.json")).unwrap();
    assert!(content.contains("\"level\": 5"));

    // Should now have 2 branches
    let branches = repo.list_branches().unwrap();
    assert_eq!(branches.len(), 2);
}

#[test]
fn switch_branch_changes_files() {
    let dir = setup_quest_dir();
    let repo = HistoryRepo::init(dir.path()).unwrap();

    // Make a commit on main
    fs::write(dir.path().join("TestChar.json"), r#"{"level": 10}"#).unwrap();
    repo.commit(&SaveEvent::LevelUp(10), 10, 0, 1, 2, 600).unwrap();

    // Restore to init (creates timeline-1)
    let commits = repo.list_commits("main").unwrap();
    let init_id = &commits[1].id;
    repo.restore_to(init_id).unwrap();

    // Make a commit on timeline-1
    fs::write(dir.path().join("TestChar.json"), r#"{"level": 3}"#).unwrap();
    repo.commit(&SaveEvent::LevelUp(3), 3, 0, 1, 1, 100).unwrap();

    // Switch back to main
    repo.switch_branch("main").unwrap();
    let content = fs::read_to_string(dir.path().join("TestChar.json")).unwrap();
    assert!(content.contains("\"level\": 10"));

    // Switch to timeline-1
    repo.switch_branch("timeline-1").unwrap();
    let content = fs::read_to_string(dir.path().join("TestChar.json")).unwrap();
    assert!(content.contains("\"level\": 3"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test history_git_test`
Expected: FAIL — module `git` not found in `history`

**Step 3: Write minimal implementation**

Add `tempfile` as a dev-dependency in `Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3"
```

Create `src/history/git.rs`:

```rust
//! Git operations for save history using libgit2.

use git2::{
    BranchType, Commit, IndexAddOption, ObjectType, Repository, Signature, StatusOptions,
};
use std::path::Path;

use super::types::{CommitInfo, SaveEvent, TimelineInfo};

/// Error type for history operations.
#[derive(Debug)]
pub enum HistoryError {
    Git(git2::Error),
    NothingToCommit,
    BranchNotFound(String),
    CommitNotFound(String),
}

impl From<git2::Error> for HistoryError {
    fn from(e: git2::Error) -> Self {
        HistoryError::Git(e)
    }
}

impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryError::Git(e) => write!(f, "git error: {e}"),
            HistoryError::NothingToCommit => write!(f, "nothing to commit"),
            HistoryError::BranchNotFound(name) => write!(f, "branch not found: {name}"),
            HistoryError::CommitNotFound(id) => write!(f, "commit not found: {id}"),
        }
    }
}

/// Wraps a git2::Repository for save history operations.
pub struct HistoryRepo {
    repo: Repository,
}

impl HistoryRepo {
    /// Initialize or open a git repository at the given path.
    /// Creates an initial commit if the repo is new.
    pub fn init(quest_dir: &Path) -> Result<Self, HistoryError> {
        let repo = if quest_dir.join(".git").exists() {
            Repository::open(quest_dir)?
        } else {
            let repo = Repository::init(quest_dir)?;

            // Create initial commit with all existing files
            let sig = Self::signature();
            let mut index = repo.index()?;
            index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
            index.write()?;
            let tree_id = index.write_tree()?;
            let tree = repo.find_tree(tree_id)?;

            repo.commit(Some("HEAD"), &sig, &sig, "Initialize save history", &tree, &[])?;

            // Rename default branch to "main" if needed
            if let Ok(mut head_ref) = repo.find_branch("master", BranchType::Local) {
                head_ref.rename("main", false)?;
            }

            repo
        };

        Ok(Self { repo })
    }

    /// Commit all changes with a save event message.
    /// Returns Ok(()) even if there's nothing to commit (no-op).
    pub fn commit(
        &self,
        event: &SaveEvent,
        level: u32,
        prestige: u32,
        zone_id: u32,
        subzone_id: u32,
        play_time_seconds: u64,
    ) -> Result<(), HistoryError> {
        // Check if there are any changes to commit
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);
        let statuses = self.repo.statuses(Some(&mut status_opts))?;
        if statuses.is_empty() {
            return Ok(()); // Nothing to commit
        }

        let sig = Self::signature();
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        let head = self.repo.head()?.peel_to_commit()?;
        let message = event.commit_message(level, prestige, zone_id, subzone_id, play_time_seconds);

        self.repo
            .commit(Some("HEAD"), &sig, &sig, &message, &tree, &[&head])?;

        Ok(())
    }

    /// List all branches with their head commit info.
    pub fn list_branches(&self) -> Result<Vec<TimelineInfo>, HistoryError> {
        let mut branches = Vec::new();
        let head = self.repo.head().ok();
        let head_name = head.as_ref().and_then(|h| h.shorthand().map(String::from));

        for branch_result in self.repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch_result?;
            let name = branch.name()?.unwrap_or("unknown").to_string();
            let is_active = head_name.as_deref() == Some(&name);

            let head_commit = if let Ok(commit) = branch.get().peel_to_commit() {
                Some(Self::parse_commit(&commit))
            } else {
                None
            };

            branches.push(TimelineInfo {
                name,
                is_active,
                head_commit,
            });
        }

        // Sort: active branch first, then alphabetically
        branches.sort_by(|a, b| b.is_active.cmp(&a.is_active).then(a.name.cmp(&b.name)));

        Ok(branches)
    }

    /// List commits on a branch, newest first.
    pub fn list_commits(&self, branch_name: &str) -> Result<Vec<CommitInfo>, HistoryError> {
        let branch = self
            .repo
            .find_branch(branch_name, BranchType::Local)
            .map_err(|_| HistoryError::BranchNotFound(branch_name.to_string()))?;

        let commit = branch.get().peel_to_commit()?;
        let mut commits = Vec::new();
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push(commit.id())?;

        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            commits.push(Self::parse_commit(&commit));
        }

        Ok(commits)
    }

    /// Restore to a specific commit by creating a new timeline branch.
    /// Returns the name of the new branch.
    pub fn restore_to(&self, commit_id: &str) -> Result<String, HistoryError> {
        let oid = git2::Oid::from_str(commit_id)
            .map_err(|_| HistoryError::CommitNotFound(commit_id.to_string()))?;
        let commit = self
            .repo
            .find_commit(oid)
            .map_err(|_| HistoryError::CommitNotFound(commit_id.to_string()))?;

        // Find next timeline number
        let branch_name = self.next_timeline_name()?;

        // Create new branch at the target commit
        self.repo.branch(&branch_name, &commit, false)?;

        // Checkout the new branch
        self.checkout_branch(&branch_name)?;

        Ok(branch_name)
    }

    /// Switch to an existing branch.
    pub fn switch_branch(&self, branch_name: &str) -> Result<(), HistoryError> {
        self.checkout_branch(branch_name)
    }

    // -- Private helpers --

    fn signature() -> Signature<'static> {
        Signature::now("Quest", "quest@localhost").expect("valid signature")
    }

    fn parse_commit(commit: &Commit<'_>) -> CommitInfo {
        let full_message = commit.message().unwrap_or("").to_string();
        let timestamp = commit.time().seconds();

        // Split on " | " to separate description from suffix
        let (message, suffix) = if let Some(idx) = full_message.find(" | ") {
            (full_message[..idx].to_string(), &full_message[idx + 3..])
        } else {
            (full_message.clone(), "")
        };

        // Parse suffix: "Lv18 P0 Z2-3 2h15m"
        let (level, prestige, zone, playtime) = Self::parse_suffix(suffix);

        CommitInfo {
            id: commit.id().to_string(),
            message,
            timestamp,
            level,
            prestige,
            zone,
            playtime,
        }
    }

    fn parse_suffix(suffix: &str) -> (u32, u32, String, String) {
        let parts: Vec<&str> = suffix.split_whitespace().collect();
        let level = parts
            .iter()
            .find(|p| p.starts_with("Lv"))
            .and_then(|p| p[2..].parse().ok())
            .unwrap_or(0);
        let prestige = parts
            .iter()
            .find(|p| p.starts_with('P') && p[1..].chars().next().map_or(false, |c| c.is_ascii_digit()))
            .and_then(|p| p[1..].parse().ok())
            .unwrap_or(0);
        let zone = parts
            .iter()
            .find(|p| p.starts_with('Z'))
            .map(|p| p[1..].to_string())
            .unwrap_or_default();
        let playtime = parts
            .iter()
            .find(|p| p.ends_with('m') && p.contains('h'))
            .map(|p| p.to_string())
            .unwrap_or_default();

        (level, prestige, zone, playtime)
    }

    fn next_timeline_name(&self) -> Result<String, HistoryError> {
        let mut max_n = 0u32;
        for branch_result in self.repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch_result?;
            if let Some(name) = branch.name()? {
                if let Some(n_str) = name.strip_prefix("timeline-") {
                    if let Ok(n) = n_str.parse::<u32>() {
                        max_n = max_n.max(n);
                    }
                }
            }
        }
        Ok(format!("timeline-{}", max_n + 1))
    }

    fn checkout_branch(&self, branch_name: &str) -> Result<(), HistoryError> {
        let refname = format!("refs/heads/{branch_name}");
        let obj = self
            .repo
            .revparse_single(&refname)
            .map_err(|_| HistoryError::BranchNotFound(branch_name.to_string()))?;

        self.repo.checkout_tree(
            &obj,
            Some(git2::build::CheckoutBuilder::new().force()),
        )?;
        self.repo.set_head(&refname)?;

        Ok(())
    }
}
```

Update `src/history/mod.rs`:

```rust
//! Git-based save history for checkpoint and restore.

pub mod git;
pub mod types;

pub use git::HistoryRepo;
pub use types::{CommitInfo, SaveEvent, TimelineInfo};
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test history_git_test`
Expected: All 7 tests PASS

**Step 5: Commit**

```bash
git add src/history/git.rs src/history/mod.rs Cargo.toml Cargo.lock tests/history_git_test.rs
git commit -m "feat(history): add git operations with init, commit, restore, and branch switching"
```

---

### Task 4: Create commit message parsing module

**Files:**
- Create: `src/history/persistence.rs`
- Modify: `src/history/mod.rs`

**Step 1: Write the failing test**

Add to `tests/history_types_test.rs`:

```rust
use quest::history::persistence::parse_commit_message;

#[test]
fn parse_commit_message_full() {
    let (msg, level, prestige, zone, playtime) =
        parse_commit_message("Defeated Dark Forest boss | Lv18 P0 Z2-3 2h15m");
    assert_eq!(msg, "Defeated Dark Forest boss");
    assert_eq!(level, 18);
    assert_eq!(prestige, 0);
    assert_eq!(zone, "2-3");
    assert_eq!(playtime, "2h15m");
}

#[test]
fn parse_commit_message_no_suffix() {
    let (msg, level, prestige, zone, playtime) =
        parse_commit_message("Initialize save history");
    assert_eq!(msg, "Initialize save history");
    assert_eq!(level, 0);
    assert_eq!(prestige, 0);
    assert_eq!(zone, "");
    assert_eq!(playtime, "");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test history_types_test parse_commit_message`
Expected: FAIL — module `persistence` not found

**Step 3: Write minimal implementation**

Create `src/history/persistence.rs`:

```rust
//! Parsing commit messages back into structured data for the UI.

/// Parse a commit message into (description, level, prestige, zone, playtime).
pub fn parse_commit_message(full_message: &str) -> (String, u32, u32, String, String) {
    let (message, suffix) = if let Some(idx) = full_message.find(" | ") {
        (full_message[..idx].to_string(), &full_message[idx + 3..])
    } else {
        return (full_message.to_string(), 0, 0, String::new(), String::new());
    };

    let parts: Vec<&str> = suffix.split_whitespace().collect();

    let level = parts
        .iter()
        .find(|p| p.starts_with("Lv"))
        .and_then(|p| p[2..].parse().ok())
        .unwrap_or(0);

    let prestige = parts
        .iter()
        .find(|p| {
            p.starts_with('P')
                && p[1..]
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_ascii_digit())
        })
        .and_then(|p| p[1..].parse().ok())
        .unwrap_or(0);

    let zone = parts
        .iter()
        .find(|p| p.starts_with('Z'))
        .map(|p| p[1..].to_string())
        .unwrap_or_default();

    let playtime = parts
        .iter()
        .find(|p| p.ends_with('m') && p.contains('h'))
        .map(|p| p.to_string())
        .unwrap_or_default();

    (message, level, prestige, zone, playtime)
}
```

Update `src/history/mod.rs` to add:

```rust
pub mod persistence;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test history_types_test parse_commit_message`
Expected: PASS

**Step 5: Refactor git.rs to use shared parsing**

Refactor `HistoryRepo::parse_suffix` and `parse_commit` in `git.rs` to call `persistence::parse_commit_message` instead of duplicating the logic.

**Step 6: Run all tests**

Run: `cargo test --test history_types_test && cargo test --test history_git_test`
Expected: All PASS

**Step 7: Commit**

```bash
git add src/history/ tests/history_types_test.rs
git commit -m "feat(history): add commit message parsing module, deduplicate suffix parsing"
```

---

### Task 5: Integrate history commits into save_all()

**Files:**
- Modify: `src/main_helpers/persistence.rs:10-25`
- Modify: `src/main.rs` (all `save_all()` call sites)
- Modify: `src/main_helpers/input_routing.rs:52-64`

**Step 1: Modify save_all() signature**

Update `src/main_helpers/persistence.rs` to accept an optional `SaveEvent` and `HistoryRepo`:

```rust
//! Game state persistence (save all).

use crate::achievements;
use crate::character::manager::CharacterManager;
use crate::core::game_state::GameState;
use crate::enhancement;
use crate::haven;
use crate::history::{HistoryRepo, SaveEvent};

/// Save all game state files (character, achievements, haven, enhancement).
/// If a `SaveEvent` is provided and `history_repo` is available, also creates a git commit.
pub fn save_all(
    character_manager: &CharacterManager,
    state: &GameState,
    global_achievements: &achievements::Achievements,
    haven: &haven::Haven,
    enhancement: &enhancement::EnhancementProgress,
    save_event: Option<&SaveEvent>,
    history_repo: Option<&HistoryRepo>,
) {
    let _ = character_manager.save_character(state);
    achievements::save_achievements(global_achievements).ok();
    if haven.discovered {
        haven::save_haven(haven).ok();
    }
    if enhancement.discovered {
        enhancement::save_enhancement(enhancement).ok();
    }

    // Git commit for significant events only
    if let (Some(event), Some(repo)) = (save_event, history_repo) {
        let _ = repo.commit(
            event,
            state.character_level,
            state.prestige_rank,
            state.zone_progression.current_zone_id,
            state.zone_progression.current_subzone_id,
            state.play_time_seconds,
        );
    }
}
```

**Step 2: Update all call sites in main.rs**

Every existing `save_all(...)` call must add `None, None` (or `None, history_repo.as_ref()`) for the two new parameters. Autosave calls pass `None` for the event. Event-triggered saves will pass `Some(&event)` — but this task just adds the plumbing, not the event mapping.

Update `src/main_helpers/input_routing.rs` similarly — `save_all()` calls gain the two new params.

For now, all call sites pass `save_event: None` and `history_repo: history_repo.as_ref()`. The repo handle is stored in main.rs (initialized at startup, see Task 6).

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles with the new signature

**Step 4: Run full test suite**

Run: `cargo test`
Expected: All existing tests PASS (no behavior change, just plumbing)

**Step 5: Commit**

```bash
git add src/main_helpers/persistence.rs src/main_helpers/input_routing.rs src/main.rs
git commit -m "refactor(persistence): add SaveEvent and HistoryRepo params to save_all()"
```

---

### Task 6: Initialize HistoryRepo at startup in main.rs

**Files:**
- Modify: `src/main.rs` (startup section, around line 335)

**Step 1: Add repo initialization**

After the `quest_dir` / `character_manager` setup, initialize the history repo:

```rust
// Initialize save history (git-based)
let history_repo = match history::HistoryRepo::init(&quest_dir) {
    Ok(repo) => Some(repo),
    Err(e) => {
        eprintln!("Warning: save history unavailable: {e}");
        None
    }
};
```

Pass `history_repo.as_ref()` to all `save_all()` calls.

**Step 2: Verify it compiles and runs**

Run: `cargo check`
Expected: Compiles

Run: `cargo run -- --debug` (briefly launch and quit)
Expected: `~/.quest/.git/` is created, game runs normally

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat(history): initialize git repo at startup"
```

---

### Task 7: Map TickEvents to SaveEvents in the game loop

**Files:**
- Modify: `src/main.rs` (tick processing section, around lines 754-815)

**Step 1: Create the event mapping function**

Add a helper in `src/main_helpers/` or directly in `main.rs`:

```rust
/// Extract save-worthy events from a tick's events.
fn extract_save_events(events: &[core::tick_types::TickEvent]) -> Vec<history::SaveEvent> {
    use core::tick_types::TickEvent;
    use history::SaveEvent;

    let mut save_events = Vec::new();
    for event in events {
        match event {
            TickEvent::LeveledUp { new_level } => {
                // Only commit on milestone levels (every 10)
                if new_level % 10 == 0 {
                    save_events.push(SaveEvent::LevelUp(*new_level));
                }
            }
            TickEvent::SubzoneBossDefeated { result, .. } => {
                use crate::zones::BossDefeatResult;
                match result {
                    BossDefeatResult::ZoneComplete { next_zone_name, .. } => {
                        save_events.push(SaveEvent::ZoneUnlocked(next_zone_name.clone()));
                    }
                    BossDefeatResult::StormsEnd { .. } => {
                        save_events.push(SaveEvent::ZoneBossDefeated("Storm Citadel".to_string()));
                    }
                    _ => {}
                }
            }
            TickEvent::DungeonCompleted { size, .. } => {
                save_events.push(SaveEvent::DungeonCompleted(size.clone()));
            }
            TickEvent::FishingRankUp { new_rank, .. } => {
                save_events.push(SaveEvent::FishingRankUp(*new_rank));
            }
            TickEvent::StormLeviathanCaught { .. } => {
                save_events.push(SaveEvent::StormLeviathanCaught);
            }
            _ => {}
        }
    }
    save_events
}
```

**Note:** The exact TickEvent variant fields need to be verified against `src/core/tick_types.rs` — the implementer should read the file and match the actual field names. The function above is a template; adjust field names to match the real enum.

**Step 2: Wire it into the game loop**

In the tick processing section (around line 786 where `save_all` is called for tick-based changes), add:

```rust
// Check for save-worthy events
let save_events = extract_save_events(&tick_result.events);
if let Some(first_event) = save_events.first() {
    save_all(
        &character_manager,
        &state,
        &global_achievements,
        &haven,
        &enhancement,
        Some(first_event),
        history_repo.as_ref(),
    );
}
```

**Step 3: Map input-driven events**

For events triggered by player input (not ticks):
- **Prestige**: In prestige input handler, when prestige executes, the `InputResult::NeedsSave` path should carry a `SaveEvent::PrestigeRank(new_rank)`. This requires either enriching `InputResult` with an optional `SaveEvent` or mapping it in the routing layer.
- **Haven build/upgrade**: Similarly, `InputResult::NeedsSaveAll` after Haven changes should carry the event.
- **Soulforge**: The soulforge animation result section (main.rs ~line 901) already calls `save_all` — add the event there.
- **Challenge wins**: When `apply_game_result` produces a win, emit a `SaveEvent::ChallengeWon`.

The simplest approach: add `Option<SaveEvent>` to `InputResult::NeedsSave` and `NeedsSaveAll` variants.

**Step 4: Verify it compiles and run a quick test**

Run: `cargo check`
Expected: Compiles

Run: `cargo run -- --debug`, use debug menu to trigger events, check `cd ~/.quest && git log --oneline`
Expected: Commits appear for significant events

**Step 5: Commit**

```bash
git add src/main.rs src/main_helpers/ src/input/
git commit -m "feat(history): map TickEvents and input events to SaveEvents for git commits"
```

---

### Task 8: Add Timeline Browser UI overlay

**Files:**
- Create: `src/ui/timeline_scene.rs`
- Modify: `src/ui/mod.rs` (add module)
- Modify: `src/input/types.rs` (add `GameOverlay::Timeline` variant)

**Step 1: Add the overlay variant**

In `src/input/types.rs`, add to the `GameOverlay` enum:

```rust
/// Timeline browser for save history restore/branch switching.
Timeline {
    browser: crate::ui::timeline_scene::TimelineBrowserState,
},
```

**Step 2: Create TimelineBrowserState and render function**

Create `src/ui/timeline_scene.rs` with:

```rust
//! Timeline browser overlay for viewing and restoring save history.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::history::{CommitInfo, TimelineInfo};

/// State for the Timeline Browser overlay.
pub struct TimelineBrowserState {
    /// All branches.
    pub branches: Vec<TimelineInfo>,
    /// Currently selected branch index.
    pub selected_branch: usize,
    /// Commits for the selected branch.
    pub commits: Vec<CommitInfo>,
    /// Currently selected commit index.
    pub selected_commit: usize,
    /// Whether a restore confirmation is pending.
    pub confirm_pending: bool,
}

impl TimelineBrowserState {
    pub fn new(branches: Vec<TimelineInfo>, commits: Vec<CommitInfo>) -> Self {
        Self {
            branches,
            selected_branch: 0,
            commits,
            selected_commit: 0,
            confirm_pending: false,
        }
    }
}

/// Render the two-panel timeline browser overlay.
pub fn draw_timeline_browser(
    frame: &mut Frame,
    area: Rect,
    state: &TimelineBrowserState,
) {
    // Clear background
    frame.render_widget(Clear, area);

    // Outer block
    let outer = Block::default()
        .title(" ⚡ TIMELINE BROWSER ⚡ ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::default().bg(Color::Black));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    // Split into left (branches) and right (commits)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(40)])
        .split(inner);

    // -- Left panel: branches --
    let branch_items: Vec<ListItem> = state
        .branches
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let prefix = if b.is_active { "● " } else { "  " };
            let style = if i == state.selected_branch {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if b.is_active {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!("{prefix}{}", b.name)).style(style)
        })
        .collect();

    let branch_list = List::new(branch_items)
        .block(
            Block::default()
                .title(" TIMELINES ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
    frame.render_widget(branch_list, chunks[0]);

    // -- Right panel: commits --
    let branch_name = state
        .branches
        .get(state.selected_branch)
        .map(|b| b.name.as_str())
        .unwrap_or("unknown");

    let mut commit_lines: Vec<Line> = vec![
        Line::from(Span::styled(
            format!("  {branch_name}"),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from("─".repeat(chunks[1].width.saturating_sub(2) as usize)),
    ];

    for (i, commit) in state.commits.iter().enumerate() {
        let is_selected = i == state.selected_commit;
        let highlight = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let dim = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Format timestamp
        let datetime = chrono::DateTime::from_timestamp(commit.timestamp, 0)
            .map(|dt| dt.with_timezone(&chrono::Local).format("%b %d, %Y  %l:%M %p").to_string())
            .unwrap_or_else(|| "Unknown date".to_string());

        // Status line
        let status = if !commit.zone.is_empty() {
            format!("Lv{} · P{} · Zone {} · {}", commit.level, commit.prestige, commit.zone, commit.playtime)
        } else {
            String::new()
        };

        let prefix = if is_selected { "▸ " } else { "  " };

        commit_lines.push(Line::from(Span::styled(
            format!("  ┌{'─'.repeat(40)}┐"),
            dim,
        )));
        commit_lines.push(Line::from(Span::styled(
            format!("  │ {prefix}{:<38}│", commit.message),
            highlight,
        )));
        commit_lines.push(Line::from(Span::styled(
            format!("  │  {:<39}│", datetime),
            dim,
        )));
        if !status.is_empty() {
            commit_lines.push(Line::from(Span::styled(
                format!("  │  {:<39}│", status),
                dim,
            )));
        }
        commit_lines.push(Line::from(Span::styled(
            format!("  └{'─'.repeat(40)}┘"),
            dim,
        )));
    }

    // Controls
    commit_lines.push(Line::from(""));
    if state.confirm_pending {
        commit_lines.push(Line::from(Span::styled(
            "  Restore to this point? [Enter] Confirm  [Esc] Cancel",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
    } else {
        commit_lines.push(Line::from(Span::styled(
            "  [Enter] Restore  [←/→] Branch  [↑/↓] Scroll  [Esc] Close",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let commit_widget = Paragraph::new(commit_lines)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
    frame.render_widget(commit_widget, chunks[1]);
}
```

**Note:** The exact Ratatui API calls (frame, widgets, styles) should be verified against the project's Ratatui 0.30 usage patterns. Check existing overlay renderers like `haven_scene.rs` for conventions and adapt as needed.

**Step 3: Register the module**

Add `pub mod timeline_scene;` to `src/ui/mod.rs`.

**Step 4: Verify it compiles**

Run: `cargo check`
Expected: Compiles (the UI isn't wired to input yet)

**Step 5: Commit**

```bash
git add src/ui/timeline_scene.rs src/ui/mod.rs src/input/types.rs
git commit -m "feat(history): add Timeline Browser UI overlay"
```

---

### Task 9: Add Timeline Browser input handling

**Files:**
- Create: `src/input/timeline_input.rs`
- Modify: `src/input/mod.rs` (add keybind 'T' and overlay dispatch)

**Step 1: Create input handler**

Create `src/input/timeline_input.rs`:

```rust
//! Input handling for the Timeline Browser overlay.

use crossterm::event::{KeyCode, KeyEvent};

use crate::input::types::GameOverlay;
use crate::input::InputResult;
use crate::ui::timeline_scene::TimelineBrowserState;

/// Result of a timeline browser input action.
pub enum TimelineAction {
    /// Close the overlay.
    Close,
    /// Continue (stay in overlay).
    Continue,
    /// Player confirmed a restore. Carries the commit ID and is for a branch switch (not restore).
    Restore { commit_id: String },
    /// Player wants to switch to a different branch (select branch, not individual commit).
    SwitchBranch { branch_name: String },
}

/// Handle input for the timeline browser.
pub fn handle_timeline_input(
    key: KeyEvent,
    state: &mut TimelineBrowserState,
) -> TimelineAction {
    match key.code {
        KeyCode::Esc => {
            if state.confirm_pending {
                state.confirm_pending = false;
                TimelineAction::Continue
            } else {
                TimelineAction::Close
            }
        }
        KeyCode::Up => {
            if state.selected_commit > 0 {
                state.selected_commit -= 1;
                state.confirm_pending = false;
            }
            TimelineAction::Continue
        }
        KeyCode::Down => {
            if state.selected_commit + 1 < state.commits.len() {
                state.selected_commit += 1;
                state.confirm_pending = false;
            }
            TimelineAction::Continue
        }
        KeyCode::Left => {
            if state.selected_branch > 0 {
                state.selected_branch -= 1;
                state.selected_commit = 0;
                state.confirm_pending = false;
                // Signal that commits need to be reloaded for this branch
                let branch = &state.branches[state.selected_branch];
                TimelineAction::SwitchBranch {
                    branch_name: branch.name.clone(),
                }
            } else {
                TimelineAction::Continue
            }
        }
        KeyCode::Right => {
            if state.selected_branch + 1 < state.branches.len() {
                state.selected_branch += 1;
                state.selected_commit = 0;
                state.confirm_pending = false;
                let branch = &state.branches[state.selected_branch];
                TimelineAction::SwitchBranch {
                    branch_name: branch.name.clone(),
                }
            } else {
                TimelineAction::Continue
            }
        }
        KeyCode::Enter => {
            if state.confirm_pending {
                // Confirmed restore
                if let Some(commit) = state.commits.get(state.selected_commit) {
                    TimelineAction::Restore {
                        commit_id: commit.id.clone(),
                    }
                } else {
                    TimelineAction::Continue
                }
            } else {
                state.confirm_pending = true;
                TimelineAction::Continue
            }
        }
        _ => TimelineAction::Continue,
    }
}
```

**Step 2: Add keybind and overlay dispatch**

In `src/input/mod.rs`:

- Add `pub mod timeline_input;`
- Add keybind `KeyCode::Char('t') | KeyCode::Char('T')` in the global keybinds section (after achievements, around line 354) that opens the Timeline overlay
- Add a match arm for `GameOverlay::Timeline { browser }` in the overlay priority chain that delegates to `timeline_input::handle_timeline_input()`

**Step 3: Wire restore action**

When `TimelineAction::Restore` is returned, the main loop should:
1. Call `history_repo.restore_to(commit_id)` to create a new timeline branch
2. Reload all game state from disk
3. Close the overlay

When `TimelineAction::SwitchBranch` is returned (from `←/→`), reload commits for the newly selected branch into the browser state (call `history_repo.list_commits(branch_name)`).

**Step 4: Add rendering dispatch**

In `src/main_helpers/scene.rs` or wherever overlays are drawn, add a match for `GameOverlay::Timeline` that calls `draw_timeline_browser()`.

**Step 5: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 6: Manual test**

Run: `cargo run -- --debug`
- Play until some events are committed
- Press `T` to open Timeline Browser
- Navigate with arrow keys
- Press `Enter` to restore, confirm
- Verify game state matches the restored commit

**Step 7: Commit**

```bash
git add src/input/timeline_input.rs src/input/mod.rs src/main.rs src/main_helpers/
git commit -m "feat(history): add Timeline Browser keybind (T) and input handling with restore/switch"
```

---

### Task 10: Add overlay rendering to the draw loop

**Files:**
- Modify: `src/main_helpers/overlay.rs` or scene rendering dispatch
- Modify: `src/main_helpers/scene.rs` (if scene_kind needs updating)

**Step 1: Add rendering**

In the overlay drawing function (wherever `GameOverlay::Achievements`, `GameOverlay::Help`, etc. are matched for rendering), add:

```rust
GameOverlay::Timeline { ref browser } => {
    ui::timeline_scene::draw_timeline_browser(frame, area, browser);
}
```

**Step 2: Verify the overlay renders**

Run: `cargo run -- --debug`, press `T`
Expected: Timeline Browser overlay appears with the correct two-panel layout

**Step 3: Commit**

```bash
git add src/main_helpers/ src/ui/
git commit -m "feat(history): wire Timeline Browser rendering into overlay draw loop"
```

---

### Task 11: Handle game state reload after restore

**Files:**
- Modify: `src/main.rs` (game loop, where restore/switch actions are processed)

**Step 1: Implement reload logic**

When `TimelineAction::Restore` or branch switch is confirmed:

```rust
// After history_repo.restore_to() or switch_branch() succeeds:

// Reload character
if let Some(loaded) = character_manager.load_character(&state.character_name) {
    state = loaded;
}

// Reload account-level state
global_achievements = achievements::load_achievements();
haven = haven::load_haven();
enhancement = enhancement::load_enhancement();
// deep state if applicable

// Close overlay
overlay = GameOverlay::None;

// Add a log entry so the player knows what happened
state.combat_state.add_log_entry(
    format!("⏳ Restored to: {}", commit_message),
    false,
    true,
);
```

**Step 2: Test restore round-trip**

Run: `cargo run -- --debug`
1. Play to generate some history (trigger events via debug menu)
2. Note current level/zone
3. Open Timeline Browser (`T`)
4. Restore to an earlier commit
5. Verify level/zone match the earlier state
6. Switch back to `main` branch
7. Verify original state is restored

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat(history): reload game state after timeline restore or branch switch"
```

---

### Task 12: Add deep state to save_all and final integration test

**Files:**
- Modify: `src/main_helpers/persistence.rs` (add deep state to save_all if not already there)
- Create: `tests/history_integration_test.rs`

**Step 1: Write integration test**

```rust
//! Integration test for the full save-history round trip.

use quest::history::git::HistoryRepo;
use quest::history::types::SaveEvent;
use std::fs;
use tempfile::TempDir;

#[test]
fn full_save_restore_round_trip() {
    let dir = TempDir::new().unwrap();
    let repo = HistoryRepo::init(dir.path()).unwrap();

    // Simulate: initial save
    fs::write(
        dir.path().join("char.json"),
        r#"{"level":1,"zone":1,"subzone":1}"#,
    )
    .unwrap();
    repo.commit(&SaveEvent::CharacterCreated("Hero".to_string()), 1, 0, 1, 1, 0)
        .unwrap();

    // Simulate: level up
    fs::write(
        dir.path().join("char.json"),
        r#"{"level":10,"zone":1,"subzone":3}"#,
    )
    .unwrap();
    repo.commit(&SaveEvent::LevelUp(10), 10, 0, 1, 3, 600)
        .unwrap();

    // Simulate: prestige
    fs::write(
        dir.path().join("char.json"),
        r#"{"level":50,"zone":3,"subzone":2}"#,
    )
    .unwrap();
    repo.commit(&SaveEvent::PrestigeRank(1), 50, 1, 3, 2, 3600)
        .unwrap();

    // Verify 4 commits (init + 3 events)
    let commits = repo.list_commits("main").unwrap();
    assert_eq!(commits.len(), 4);
    assert_eq!(commits[0].message, "Prestige to rank 1");
    assert_eq!(commits[0].level, 50);
    assert_eq!(commits[0].prestige, 1);

    // Restore to level 10 commit
    let level10_id = &commits[2].id;
    let new_branch = repo.restore_to(level10_id).unwrap();
    assert_eq!(new_branch, "timeline-1");

    // Verify file is back to level 10
    let content = fs::read_to_string(dir.path().join("char.json")).unwrap();
    assert!(content.contains("\"level\":10"));

    // Switch back to main
    repo.switch_branch("main").unwrap();
    let content = fs::read_to_string(dir.path().join("char.json")).unwrap();
    assert!(content.contains("\"level\":50"));

    // Verify branches
    let branches = repo.list_branches().unwrap();
    assert_eq!(branches.len(), 2);
    assert!(branches.iter().any(|b| b.name == "main" && b.is_active));
    assert!(branches.iter().any(|b| b.name == "timeline-1" && !b.is_active));
}
```

**Step 2: Run integration test**

Run: `cargo test --test history_integration_test`
Expected: PASS

**Step 3: Run full test suite**

Run: `cargo test`
Expected: All existing + new tests PASS

**Step 4: Run make check**

Run: `make check`
Expected: All CI checks pass (format, clippy, tests, build, audit)

**Step 5: Commit**

```bash
git add tests/history_integration_test.rs src/main_helpers/persistence.rs
git commit -m "feat(history): add integration test for full save-restore round trip"
```

---

### Task 13: Add help text for Timeline Browser

**Files:**
- Modify: `src/ui/help_overlay.rs` (add 'T' keybind to the help screen)

**Step 1: Add keybind entry**

Find where other keybinds are listed (H for Haven, S for Soulforge, etc.) and add:

```rust
("T", "Timeline Browser (save history)"),
```

**Step 2: Verify**

Run: `cargo run`, press `?` to open help
Expected: 'T' keybind appears in the controls list

**Step 3: Commit**

```bash
git add src/ui/help_overlay.rs
git commit -m "docs: add Timeline Browser keybind to help overlay"
```

---

### Task 14: Final cleanup and make check

**Step 1: Run make check**

Run: `make check`
Expected: All CI checks pass

**Step 2: Fix any clippy/fmt issues**

Run: `make fmt` if needed

**Step 3: Final commit**

```bash
git commit -m "chore: final cleanup for git-based save history feature"
```
