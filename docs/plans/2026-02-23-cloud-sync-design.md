# Cloud Sync Design: GitHub Remote Storage for Time Vault

## Overview

Enable players to back up Time Vault saves to GitHub and restore them on another machine. Uses the existing `git2`-based save history system with a private GitHub repository as the remote.

## Architecture

### Module

`src/history/cloud.rs` — evolved from existing commit `d397416`.

Reuses: PAT validation, GitHub API repo creation, `git2` push/fetch, `.cloud.json` persistence.

### Authentication

- Personal Access Token (PAT) entered by player
- Stored in `~/.quest/.cloud.json` (not tracked by git)
- Authenticated URL: `https://x-access-token:TOKEN@github.com/user/quest-saves.git`

### Remote Repository

- Name: `quest-saves` (private, auto-created via GitHub API)
- All local branches pushed as-is

## Flows

### Game Launch

```
1. ~/.quest/ exists?
   no  → create dir, git init
   yes → continue

2. .cloud.json exists?
   no  → first-launch prompt (if no characters) OR play offline
   yes → load config, background fetch

3. Fetch result (checked before entering game loop):
   in sync     → continue
   behind      → fast-forward all behind branches, reload from disk
   diverged    → show resolution dialog (K/C/B/Esc)
   fetch fail  → warn, continue offline
```

### Linking GitHub

Three entry points:

1. **New machine, remote has data** — first-launch prompt → enter PAT → validate → find `quest-saves` → add remote → fetch → create local branches → checkout main
2. **New machine, remote empty** — first-launch prompt → enter PAT → validate → create `quest-saves` → add remote → push init commit
3. **Existing machine** — `[C]` in Time Vault → enter PAT → validate → find/create `quest-saves` → add remote → push all local branches

All machines always `git init` locally first. No `git clone` — single codepath.

### Push (Background, On Commit)

```
game_tick() → TickEvent → save_all() → git commit
                                  ↓
                            if cloud linked
                            && not already pushing:
                              spawn background push (all branches)
```

No timer or debounce needed — commit frequency (significant game events only) is naturally low.

If a push is already in flight when the next commit happens, skip it. The next commit after the in-flight push completes will push everything.

Push results:
- Success → status = linked
- Network error → status = error, continue playing, retry on next commit
- Non-fast-forward → status = out of sync

### Pull (Manual, from Time Vault)

Triggered by `[V]` in Time Vault. Checks divergence per branch before applying:

```
For each local branch that also exists on remote:
  (ahead, behind) = git2::graph_ahead_behind(local_head, remote_head)

  ahead=0, behind=0  → in sync, skip
  ahead=0, behind>0  → safe to fast-forward (reset to remote)
  ahead>0, behind=0  → local is ahead, skip (push handles it)
  ahead>0, behind>0  → DIVERGED, block with warning
```

Branches only on remote → create local branch at remote head.
Branches only on local → leave alone (push sends them up).

### Divergence Resolution

Shown on launch (auto-fetch) or manual pull when branches have diverged:

```
┌─────────────────────────────────────────────┐
│  Saves have diverged from cloud             │
│                                             │
│  Local: Lv42 · P5 · 3h 20m                 │
│  Cloud: Lv38 · P5 · 2h 45m                 │
│                                             │
│  [K] Keep local, push to cloud              │
│  [C] Use cloud, discard local               │
│  [B] Keep both (branch local as 'backup')   │
│  [Esc] Decide later                         │
└─────────────────────────────────────────────┘
```

- **Keep local [K]** → force-push local to cloud
- **Use cloud [C]** → hard reset to cloud version
- **Keep both [B]** → rename local branch to `main-backup-<date>`, reset `main` to cloud
- **Decide later [Esc]** → play on local saves, cloud stays out of sync

### Two-Computer Workflow

```
Computer A                          Cloud                         Computer B
──────────                          ─────                         ──────────
Play, commits trigger pushes
Quit game                  ──────►  Branches updated
                                                                  Launch game
                                                                  Auto-fetch on launch
                                                                  Local behind → fast-fwd
                                                                  Reload from disk
                                                                  Play, commits trigger pushes
                                                          ──────► Branches updated
                                                                  Quit game
Launch game
Auto-fetch on launch
Local behind → fast-fwd   ◄──────
Reload from disk
Seamless continuation
```

### Simultaneous Play (Edge Case)

If both computers play at the same time:
- First to push succeeds
- Second push fails (non-fast-forward) → status shows "out of sync"
- Player continues playing, no interruption
- On next launch of either machine, divergence dialog appears
- Player resolves with K/C/B options — no data loss possible with [B]

## UI Integration

### Cloud Status Indicator (Time Vault top-right)

| State | Display | Color |
|-------|---------|-------|
| Not linked | `⊘ offline` | Dark gray |
| Linked, idle | `☁ stphung` | Cyan |
| Syncing | `☁ pushing...` | Cyan |
| Out of sync | `☁ ⚠ out of sync` | Yellow |
| Error | `☁ ✗ push failed` | Red |

### Time Vault Controls (left panel focused)

```
Not linked:
  [Enter] Switch · [B] Branch · [D] Delete · [C] Link · [Tab] Saves · [Esc] Close

Linked:
  [Enter] Switch · [B] Branch · [D] Delete · [C] Push · [V] Pull · [X] Unlink · [Tab] Saves · [Esc] Close
```

### Dialogs

| Dialog | Trigger | Content |
|--------|---------|---------|
| Link | `[C]` when not linked | PAT text input (same pattern as fork naming) |
| Push confirm | `[C]` when linked | "Push all branches to cloud?" |
| Pull confirm | `[V]` when linked | "Pull latest from cloud?" |
| Unlink confirm | `[X]` when linked | Type-to-confirm |
| Divergence | Auto on launch / pull | Local vs cloud stats, K/C/B/Esc options |

### First-Launch Prompt (Character Select Screen)

Shown when `~/.quest/` was just created and no characters exist:

```
┌─────────────────────────────────────────┐
│  Restore saves from GitHub?             │
│                                         │
│  Enter a Personal Access Token to       │
│  download your saves from the cloud.    │
│                                         │
│  Token: ________________________________│
│                                         │
│  [Enter] Link    [Esc] Skip             │
└─────────────────────────────────────────┘
```

## Background Threading

Same pattern as existing `updater.rs`:

```
Main thread                          Background thread
───────────                          ─────────────────
Set status = Syncing
Spawn thread ──────────────────────► push_all_branches() / fetch()
                                     ... network I/O ...
Poll mpsc channel each tick          ◄── Send result
Receive result
Update status
```

Single `mpsc` channel. Only one background op at a time — if already syncing, skip.

Startup fetch runs in background while character select screen loads. Result checked before entering game loop.

## Config Persistence

File: `~/.quest/.cloud.json`

```json
{
  "token": "ghp_...",
  "username": "stphung",
  "repo_url": "https://github.com/stphung/quest-saves.git"
}
```

Added to `.gitignore` (not tracked in save history).

## Existing Code to Reuse (from commit d397416)

- `CloudConfig` struct and serialization
- `CloudStatus` / `CloudOpResult` enums
- `github_get_username()` — PAT validation
- `github_ensure_repo()` — create/find private repo
- `authenticated_url()` — inject token into HTTPS URL
- `make_callbacks()` — git2 credential callback
- `push_branch()` / `push_all_branches()` — git2 push logic
- `link_github()` — full link flow
- `unlink()` — remove remote and config

## New Code Needed

- `fetch_all()` — fetch all remote branches
- `check_divergence()` — per-branch ahead/behind check using `git2::graph_ahead_behind`
- `fast_forward_branch()` — reset local branch to remote head
- `create_local_from_remote()` — create local branch for remote-only branches
- `force_push_branch()` — force-push for divergence resolution [K]
- `backup_and_reset()` — rename branch + reset for divergence resolution [B]
- Push-on-commit integration in main loop
- Auto-fetch on launch integration
- All UI dialogs (link, push, pull, unlink, divergence, first-launch)
- Cloud status indicator rendering
