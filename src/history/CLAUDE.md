# History Module (Time Vault)

Git-based save versioning system. Every meaningful game event creates a git commit containing the full save state. Players browse, restore, and fork save branches through the Time Vault overlay.

## Files

| File | Purpose |
|------|---------|
| `mod.rs` | Public re-exports: `HistoryRepo`, `HistoryError`, `validate_branch_name`, `SaveEvent`, `CommitInfo`, `TimelineInfo` |
| `types.rs` | `SaveEvent` enum (21 variants), `CommitInfo` struct, `TimelineInfo` struct, commit message formatting and suffix parsing |
| `git.rs` | `HistoryRepo` struct wrapping `git2::Repository`; all local git operations (init, commit, list branches/commits, restore, fork, switch, delete) |
| `cloud.rs` | GitHub cloud sync: `CloudConfig`, `CloudStatus`, `CloudOpResult`; PAT validation, repo creation, push/pull, divergence detection and resolution |

## Key Types

- **`SaveEvent`**: Describes why a commit was made. 21 variants covering milestone progression (LevelUp, PrestigeRank, ZoneBossDefeated, ZoneUnlocked, DungeonCompleted, FishingRankUp, StormLeviathanCaught, AchievementUnlocked), state-changing actions (HavenRoomBuilt, HavenRoomUpgraded, SoulforgeEnhanced, SoulforgeFailed, ChallengeWon, GodItemForged, CharacterCreated, CharacterDeleted, EquipmentUpgrade, StormSigilActivated), ChronoSurge, and manual/auto saves (ManualSave, AutoSave). Each variant produces a human-readable `description()` and a full `commit_message()` with encoded metadata suffix.
- **`CommitInfo`**: Metadata extracted from a single history commit -- short SHA (`id`), full `message`, Unix `timestamp`, and parsed snapshot fields (`level`, `prestige`, `zone`, `playtime`).
- **`TimelineInfo`**: Summary of a git branch -- `name`, `is_active` flag, and optional `head_commit`.
- **`HistoryRepo`**: Wrapper around `git2::Repository` providing high-level save versioning operations.
- **`HistoryError`**: Error enum with variants: `Git`, `NothingToCommit`, `BranchNotFound`, `CommitNotFound`, `InvalidBranchName`, `BranchAlreadyExists`.
- **`CloudConfig`**: Persisted cloud configuration (token, username, repo URL) stored in `~/.quest/.cloud.json` (git-ignored).
- **`CloudStatus`**: State machine: `Offline` -> `Linked` -> `Syncing` -> `Linked` (success) / `OutOfSync` (diverged) / `TokenExpired` (auth failure) / `Error`.

## How It Works

### Local Git Operations (`git.rs`)

`HistoryRepo::init(quest_dir)` initializes or opens a git repo at `~/.quest/`. On first run, it stages all files, creates an initial commit, and renames master to main. It also ensures `.cloud.json` is in `.gitignore`.

**Commit flow**: `commit()` checks for working tree changes, stages everything with `index.add_all(["*"])`, writes the tree, and creates a commit with the formatted message. The message format is `"{event description} | Lv{N} P{N} Z{N}-{N} {N}h{NN}m @{name}"`. `commit_raw()` is a simpler variant for auto-saves.

**Branch operations**:
- `list_branches()` -- lists all local branches sorted by active-first, then alphabetical
- `list_commits(branch)` -- walks the commit graph newest-first using `revwalk`
- `restore_to(commit_id)` -- hard-resets the current branch to a target commit
- `fork_timeline(name, commit_id)` -- creates a new branch at a commit and checks it out
- `switch_timeline(name)` -- switches HEAD to an existing branch with hard checkout
- `delete_timeline(name)` -- deletes a branch (rejects "main" and active branch)

**Branch name validation**: Lowercase letters, digits, hyphens, underscores only; max 16 chars; cannot be empty, "main", or start with a hyphen.

**Commit suffix parsing**: `parse_commit_suffix()` extracts `(level, prestige, zone, playtime_seconds)` from the metadata suffix embedded after " | " in commit messages.

### Cloud Sync (`cloud.rs`)

Uses `ureq` for GitHub API calls and `git2` for remote operations. Cloud operations run in background threads with results delivered via `mpsc` channels (managed in `main.rs`).

**Key operations**:
- `validate_token()` -- validates a GitHub PAT, returns username and repo list
- `link_github()` -- validates PAT, ensures remote repo exists (creates if needed), adds git remote, fetches, saves config
- `push_all_branches()` -- pushes all local branches to the cloud remote
- `fetch_all()` / `fast_forward_all()` -- fetches from remote and fast-forwards local branches
- `check_divergence()` -- detects branches that are both ahead AND behind remote
- `reset_to_remote()` -- discards local changes, resets to cloud state
- `backup_and_reset()` -- renames diverged local branches as backups, then resets to cloud
- `force_push_branch()` -- force-pushes local state to cloud (local wins)
- `update_token()` -- replaces expired PAT while preserving repo link
- `is_auth_error()` -- detects HTTP 401/403 (excludes rate-limiting)
- `sanitize_cloud_error()` -- converts raw API errors to user-friendly messages

**Cloud status flow**: Offline (no config) -> Linked (config exists, idle) -> Syncing (operation in progress) -> Linked (success) / OutOfSync (diverged) / TokenExpired (auth failure) / Error (other).

## Integration Points

- **Called from**: `main_helpers/persistence.rs` (commit after save), `main_helpers/input_routing.rs` (save-with-event routing), `main.rs` game loop (Time Vault actions, cloud operations)
- **UI**: `ui/time_vault_scene.rs` renders the Time Vault overlay; `input/time_vault_input.rs` handles keyboard input and emits `TimeVaultAction` / `InputResult` variants
- **Dependencies**: `git2` (vendored-openssl), `ureq` (HTTP client), `serde` (config persistence)

## Key Constants

- Commit signature: `"Quest" <quest@localhost>`
- Default cloud repo name: `"quest-saves"`
- Git remote name: `"cloud"`
- GitHub API timeout: 15 seconds
- Branch name max length: 16 characters
