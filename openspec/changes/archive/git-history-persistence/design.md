> Backported design record. Sources: docs/plans/2026-02-22-git-history-persistence-design.md.

## 2026-02-22-git-history-persistence-design.md

# Git-Based Save History

## Overview

Add git-based versioning to the existing `~/.quest/` save directory so players can restore to previous save points. Significant game events trigger git commits with descriptive messages. Players browse and restore from a two-panel in-game Timeline Browser.

**Key constraint:** Zero changes to existing save files or the save/load pipeline. Git is purely additive — a `.git/` directory managed on top of the existing JSON persistence.

## Architecture

### New Module: `src/history/`

```
src/history/
├── mod.rs          # Public API: init, commit, restore, switch_branch, list_*
├── types.rs        # SaveEvent enum, TimelineInfo, CommitInfo structs
├── git.rs          # git2 operations (init, stage, commit, checkout, branch)
└── persistence.rs  # Parse commit messages back into CommitInfo for UI
```

### New Dependency

`git2` crate (Rust bindings to libgit2). Native, no runtime dependency on git CLI.

## SaveEvent Enum

Commits are triggered by significant events, never by autosaves.

```rust
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
```

## Commit Message Format

```
{event description} | Lv{level} P{prestige} Z{zone}-{subzone} {hours}h{minutes}m
```

Examples:
```
Defeated Dark Forest boss | Lv18 P0 Z2-3 2h15m
Prestige to rank 5 | Lv50 P5 Z1-1 11h05m
Built Armory in Haven | Lv50 P12 Z6-1 18h42m
```

The suffix is derived from `GameState` at commit time. Date/time comes from the git commit's author timestamp.

## Git Repository & Branch Model

- **Location:** `~/.quest/` (the existing save directory)
- **Initialization:** On first save event, `git2::Repository::init()` if `.git/` doesn't exist
- **Default branch:** `main`
- **On restore:** Create `timeline-{N}` branch from chosen commit, checkout it
- **On branch switch:** Auto-commit current state, then checkout target branch HEAD
- **Cleanup:** No automatic pruning (repo is tiny — a few small JSON files)

## Public API

```rust
pub fn init_repo(quest_dir: &Path) -> Result<Repository>
pub fn commit(repo: &Repository, event: &SaveEvent, state: &GameState) -> Result<Oid>
pub fn list_branches(repo: &Repository) -> Result<Vec<TimelineInfo>>
pub fn list_commits(repo: &Repository, branch: &str) -> Result<Vec<CommitInfo>>
pub fn restore_to(repo: &Repository, commit_id: Oid) -> Result<String>
pub fn switch_branch(repo: &Repository, branch: &str) -> Result<()>
```

## Data Types

```rust
pub struct CommitInfo {
    pub id: git2::Oid,
    pub message: String,        // "Defeated Dark Forest boss"
    pub timestamp: DateTime,
    pub level: u32,
    pub prestige: u32,
    pub zone: String,
    pub playtime: String,
}

pub struct TimelineInfo {
    pub name: String,           // "main", "timeline-1"
    pub is_active: bool,
    pub head_commit: CommitInfo,
}
```

## Save Flow

```
game_tick() produces TickEvent (e.g., BossKilled)
    → Caller maps to SaveEvent::ZoneBossDefeated("Dark Forest")
    → save_all(state, accounts, Some(save_event))
        → Write all JSON files (existing behavior, unchanged)
        → history::commit(repo, save_event, state)
            → git add -A
            → git commit with formatted message
```

Autosaves pass `None` and skip the commit entirely.

## Restore Flow

```
Player opens Timeline Browser
    → UI shows branches (left) and commits (right)
    → Player selects a commit, confirms
    → history::restore_to(repo, commit_id)
        → Create "timeline-{N}" branch from chosen commit
        → Checkout the new branch (JSON files on disk change)
    → Game reloads all state from disk via existing load_*() functions
```

## Branch Switch Flow

```
Player selects a different branch in Timeline Browser
    → history::switch_branch(repo, branch_name)
        → Auto-commit current state to current branch
        → Checkout target branch HEAD
    → Game reloads all state from disk
```

## Timeline Browser UI

Two-panel overlay (follows Haven/Achievement browser pattern):

```
╔══════════════════════════════════════════════════════════════════╗
║                     ⚡ TIMELINE BROWSER ⚡                      ║
╠═══════════════════╦══════════════════════════════════════════════╣
║   TIMELINES       ║  main                                      ║
║                   ║──────────────────────────────────────────────║
║ ● main            ║  ┌─────────────────────────────────────────┐║
║   timeline-1      ║  │ Defeated Dark Forest boss               │║
║   timeline-2      ║  │ Feb 22, 2026  3:42 PM                   │║
║                   ║  │ Lv18 · P0 · Zone 2-3 · 2h 15m          │║
║                   ║  └─────────────────────────────────────────┘║
║                   ║  ┌─────────────────────────────────────────┐║
║                   ║  │ Prestige to rank 1                      │║
║                   ║  │ Feb 22, 2026  2:10 PM                   │║
║                   ║  │ Lv50 · P1 · Zone 1-1 · 1h 48m          │║
║                   ║  └─────────────────────────────────────────┘║
║                   ║  ┌─────────────────────────────────────────┐║
║                   ║  │ Unlocked Dark Forest                    │║
║                   ║  │ Feb 22, 2026  1:05 PM                   │║
║                   ║  │ Lv30 · P0 · Zone 2-1 · 0h 55m          │║
║                   ║  └─────────────────────────────────────────┘║
║                   ║                                              ║
║                   ║  [Enter] Restore  [←/→] Branch  [Esc] Close ║
╚═══════════════════╩══════════════════════════════════════════════╝
```

Each commit card shows three lines: event description, date/time, and status summary.

**Navigation:** `←/→` switches branches, `↑/↓` scrolls commits, `Enter` restores, `Esc` closes.

## Integration Points

| File | Change |
|------|--------|
| `Cargo.toml` | Add `git2` dependency |
| `main.rs` | Call `history::init_repo()` at startup, store `Repository` handle |
| `main_helpers/persistence.rs` | `save_all()` gains `Option<SaveEvent>` param, calls `history::commit()` when `Some` |
| `core/tick.rs` / callers | Map relevant `TickEvent`s to `SaveEvent`s |
| New: `src/history/` | Git operations module |
| New: `src/ui/timeline_scene.rs` | Two-panel overlay |
| New: `src/input/timeline_input.rs` | Timeline Browser input handling |

## Error Handling

All git failures are non-fatal. Same philosophy as existing persistence (`.ok()` on errors).

| Scenario | Behavior |
|----------|----------|
| git2 init fails | Log warning, disable history for session |
| Commit fails | Log warning, skip. Next event retries |
| Restore fails | Show error in UI, stay on current branch |
| Corrupt `.git/` | Re-init on next startup |
| Player deletes `.git/` | Re-init on next startup, clean slate |
| Branch switch with unsaved changes | Auto-commit before switching |
