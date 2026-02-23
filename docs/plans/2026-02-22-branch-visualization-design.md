# Branch Visualization in Time Vault — Design

## Overview

Add two new view modes to the Time Vault overlay: a full-width commit Graph view and a side-by-side Compare view. The existing two-panel Browse view remains unchanged.

## View Modes

Three tabbed modes along the top of the Time Vault overlay, switched with `B`, `G`, `C` hotkeys:

```
┌─ TIME VAULT ─── [B]rowse  [G]raph  [C]ompare ────────┐
```

- **Browse** — existing two-panel layout (branches + snapshots), unchanged
- **Graph** — full-width commit graph showing all branches spatially
- **Compare** — side-by-side comparison of two selected branches

## Graph View

Full-width view with each branch as a vertical column. Commits are nodes connected by vertical lines, with horizontal fork-point connectors where branches diverge.

```
  main              alt-run           magic-build
  ─────             ───────           ───────────
   ○ Lv52 P3 Z8
   │
   ○ Lv48 P3 Z7     ○ Lv45 P2 Z6
   │                 │
   ○ Lv42 P2 Z6     ○ Lv40 P2 Z5
   │                 │
   ├─────────────────┘  ← fork point
   │
   ○ Lv38 P2 Z5     ○ Lv30 P1 Z4    ○ Lv28 P1 Z3
   │                 │                │
   │                 ├────────────────┘
   │                 │
   ○ Lv20 P1 Z2     │
   │                 │
   ○═════════════════╛  (initial)
```

### Rendering Rules

- **Columns**: Each branch gets a fixed-width column (~18 chars). `main` always leftmost. Others ordered by fork time.
- **Nodes**: `○` for each commit, `│` for vertical connectors between commits.
- **Fork lines**: `├───┘` horizontal connector at the commit where a branch forked.
- **Active branch**: Green column header, `●` for its head commit.
- **Selected node**: Highlighted background. Cursor moves with `↑↓` within a column and `←→` between columns.
- **Node labels**: Compact `Lv{N} P{N} Z{N}` next to each node.
- **Scrolling**: Vertical scroll when history exceeds viewport. Keeps selected node visible.

### Graph Layout

Rows are sorted by timestamp descending (newest at top). Columns are ordered by fork time (main first). The graph is built by walking all branches, finding shared ancestor commits, and laying them out on a unified vertical timeline.

### Controls

```
[↑↓] Navigate  [Enter] Detail  [←→] Branch  [F] Fork  [C] Compare  [Esc] Close
```

- `↑↓` — navigate within column
- `←→` — switch between branch columns
- `Enter` — detail popup for selected commit
- `F` — fork from selected commit (reuses existing NamingFork mode)
- `C` — pre-selects current branch and enters Compare view

## Compare View

Side-by-side comparison of two branches. If entering from Graph view, the branch of the selected node is pre-selected; player picks the second branch.

```
  main                          vs           alt-run
  ─────────────────────────────────────────  ───────────
  Level        52                            45
  Prestige      3                             2
  Zone          8 (Sunken Kingdom)            6 (Frozen Tundra)
  Playtime     12h 34m                        9h 10m

  ── Divergence ──────────────────────────────────────
  Forked at:   Lv38 P2 Z5  (3h 24m ago)
  Since fork:  +14 levels, +1 prestige       +7 levels

  ── Timeline ────────────────────────────────────────
  ○ Lv52 Defeated Frost Giant    │
  ○ Lv48 Prestige rank 3        │  ○ Lv45 Zone 6 cleared
  ○ Lv42 Zone 6 cleared         │  ○ Lv40 Haven room built
  ├──────────────────────────────┘
  ○ Lv38 (fork point)
```

### Three Sections

1. **Stats comparison** — Key metrics side by side (level, prestige, zone, playtime). Data pulled from each branch's head commit suffix.
2. **Divergence summary** — Where they forked, how long ago, what changed since the fork point.
3. **Interleaved timeline** — Commits from both branches shown chronologically below the fork point, with shared history merged into a single column.

### Controls

```
[←→] Switch side  [↑↓] Scroll  [B] Browse  [G] Graph  [Esc] Close
```

## Data Layer

### New HistoryRepo Queries

- `find_fork_point(branch_a, branch_b) -> Option<CommitInfo>` — walk both branches' commit histories to find their first shared commit ID.
- `all_commits_graph() -> Vec<(String, Vec<CommitInfo>)>` — all branches with full commit histories, used to build the unified graph layout.

Both are built on existing `list_branches()` and `list_commits()` — no new git operations needed.

### Graph Layout Engine

New pure-logic module that converts raw branch+commit data into a positioned grid:

- Each node has: column index, row index, commit info, branch name.
- Connector metadata: which rows have fork lines and between which columns.
- Sorting: rows by timestamp descending, columns by fork order (main first).

### Performance

All data loading happens on overlay open. With typical save frequencies and < 10 branches, we're dealing with hundreds of commits at most — trivially fast.

## State Changes

### New Types

- `ViewMode` enum: `Browse`, `Graph`, `Compare`
- `GraphState`: selected column, selected row, scroll offset, computed layout
- `CompareState`: left branch, right branch, scroll offset, selection phase

### TimeVaultState Additions

- `view_mode: ViewMode`
- `graph: GraphState`
- `compare: CompareState`

### Input Routing

- Top-level: `B`, `G`, `C` switch `view_mode` from any mode.
- Graph mode: `↑↓` navigate within column, `←→` switch columns, `Enter` detail popup, `F` fork, `C` compare.
- Compare mode: `←→` switch highlighted side, `↑↓` scroll timeline.
- Existing `ConfirmRestore` and `NamingFork` modes work from Graph view unchanged — they operate on commit IDs.
- Compare branch selection: temporary mode showing branch list for picking the second branch.
- Reopening Time Vault remembers last used view mode for the session.
