# Branch Visualization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Graph and Compare view modes to the Time Vault overlay, enabling players to visualize branch relationships, navigate a 2D commit graph, and compare branch stats side-by-side.

**Architecture:** Three tabbed views (Browse/Graph/Compare) sharing the same `TimeVaultState`. A new `graph_layout` module computes column/row positions from raw branch+commit data. The existing Browse view is unchanged. Graph and Compare views are new paint functions and input handlers.

**Tech Stack:** Rust, Ratatui, git2 (existing), chrono (existing)

---

### Task 1: Add ViewMode enum and wire tab switching

**Files:**
- Modify: `src/ui/time_vault_scene.rs:30-62` (add ViewMode, extend TimeVaultState)
- Modify: `src/input/time_vault_input.rs:36-44` (route by view mode)
- Test: `tests/time_vault_view_mode_test.rs`

**Step 1: Write the failing test**

Create `tests/time_vault_view_mode_test.rs`:

```rust
//! Tests for Time Vault view mode switching.

use quest::history::types::{CommitInfo, TimelineInfo};

// We need to access TimeVaultState and ViewMode from the UI module.
// Since the UI module is private in lib.rs, we test through input handling.
// For now, test the data types exist and ViewMode default is Browse.

#[test]
fn view_mode_default_is_browse() {
    // This test validates that ViewMode exists and TimeVaultState starts in Browse.
    // Implementation will make this pass.
    use quest::ui::time_vault_scene::{TimeVaultState, ViewMode};
    let state = TimeVaultState::new(vec![], vec![]);
    assert!(matches!(state.view_mode, ViewMode::Browse));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test time_vault_view_mode_test -- --nocapture`
Expected: FAIL — `ViewMode` doesn't exist yet, ui module is private.

**Step 3: Write minimal implementation**

In `src/ui/time_vault_scene.rs`, add the `ViewMode` enum after `PanelFocus`:

```rust
/// Which view mode the Time Vault is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Existing two-panel branch/snapshot browser.
    Browse,
    /// Full-width commit graph showing all branches.
    Graph,
    /// Side-by-side comparison of two branches.
    Compare,
}
```

Add `view_mode: ViewMode` field to `TimeVaultState`:

```rust
pub view_mode: ViewMode,
```

Initialize in `TimeVaultState::new()`:

```rust
view_mode: ViewMode::Browse,
```

Make `ViewMode` and `TimeVaultState` accessible: in `src/ui/mod.rs`, ensure `time_vault_scene` is `pub`.

In `src/lib.rs`, add `pub mod ui;` if not already present (it's currently private — check and expose only `time_vault_scene` types via re-export if needed).

**Step 4: Run test to verify it passes**

Run: `cargo test --test time_vault_view_mode_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/time_vault_scene.rs src/lib.rs tests/time_vault_view_mode_test.rs
git commit -m "feat: add ViewMode enum to TimeVaultState"
```

---

### Task 2: Add tab-switching input (B/G/C keys)

**Files:**
- Modify: `src/input/time_vault_input.rs:46-151` (add B/G/C in browse handler)
- Modify: `src/input/time_vault_input.rs:36-44` (dispatch by view_mode)
- Test: `tests/time_vault_view_mode_test.rs` (add tab-switch tests)

**Step 1: Write the failing tests**

Add to `tests/time_vault_view_mode_test.rs`:

```rust
use quest::input::time_vault_input::handle_time_vault_input;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

#[test]
fn pressing_g_switches_to_graph_view() {
    let mut state = TimeVaultState::new(vec![], vec![]);
    assert!(matches!(state.view_mode, ViewMode::Browse));
    let _ = handle_time_vault_input(key(KeyCode::Char('g')), &mut state);
    assert!(matches!(state.view_mode, ViewMode::Graph));
}

#[test]
fn pressing_c_switches_to_compare_view() {
    let mut state = TimeVaultState::new(vec![], vec![]);
    let _ = handle_time_vault_input(key(KeyCode::Char('c')), &mut state);
    assert!(matches!(state.view_mode, ViewMode::Compare));
}

#[test]
fn pressing_b_switches_to_browse_view() {
    let mut state = TimeVaultState::new(vec![], vec![]);
    state.view_mode = ViewMode::Graph;
    let _ = handle_time_vault_input(key(KeyCode::Char('b')), &mut state);
    assert!(matches!(state.view_mode, ViewMode::Browse));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test time_vault_view_mode_test -- --nocapture`
Expected: FAIL — B/G/C keys not handled yet.

**Step 3: Write minimal implementation**

In `src/input/time_vault_input.rs`, modify `handle_time_vault_input` to check for B/G/C before dispatching by mode. These keys work from any view mode when in Browse browser-mode (not in a confirmation dialog):

```rust
pub fn handle_time_vault_input(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    // Global tab switching (only when not in a dialog).
    if state.mode == BrowserMode::Browse {
        match key.code {
            KeyCode::Char('b') | KeyCode::Char('B') => {
                state.view_mode = ViewMode::Browse;
                return TimeVaultAction::Continue;
            }
            KeyCode::Char('g') | KeyCode::Char('G') => {
                state.view_mode = ViewMode::Graph;
                return TimeVaultAction::Continue;
            }
            _ => {}
        }
    }

    match state.view_mode {
        ViewMode::Browse => match &state.mode {
            BrowserMode::Browse => handle_browse(key, state),
            BrowserMode::ConfirmRestore => handle_confirm_restore(key, state),
            BrowserMode::ConfirmSwitch => handle_confirm_switch(key, state),
            BrowserMode::ConfirmDelete => handle_confirm_delete(key, state),
            BrowserMode::NamingFork { .. } => handle_naming_fork(key, state),
        },
        ViewMode::Graph => handle_graph_input(key, state),
        ViewMode::Compare => handle_compare_input(key, state),
    }
}
```

Add stub handlers:

```rust
fn handle_graph_input(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match key.code {
        KeyCode::Esc => TimeVaultAction::Close,
        _ => TimeVaultAction::Continue,
    }
}

fn handle_compare_input(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match key.code {
        KeyCode::Esc => TimeVaultAction::Close,
        _ => TimeVaultAction::Continue,
    }
}
```

Note: The `C` key currently does nothing in browse mode (it's not used for any existing action), so we can safely add it. However, `handle_browse` has `KeyCode::Char('d')` handling — the `C` key for Compare needs to be handled at the top level before the browse handler. This is already done in the code above.

For the Compare tab, we need to handle the `C` key carefully: in Browse mode with Right panel focus, `C` is not used. In Browse mode with Left panel focus, `C` is not used. So adding `C` at the global level is safe.

**Step 4: Run tests to verify they pass**

Run: `cargo test --test time_vault_view_mode_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/input/time_vault_input.rs tests/time_vault_view_mode_test.rs
git commit -m "feat: add B/G/C tab switching in Time Vault"
```

---

### Task 3: Add GraphState and graph layout data types

**Files:**
- Create: `src/history/graph_layout.rs`
- Modify: `src/history/mod.rs` (add `pub mod graph_layout`)
- Modify: `src/ui/time_vault_scene.rs` (add GraphState to TimeVaultState)
- Test: `tests/graph_layout_test.rs`

**Step 1: Write the failing test**

Create `tests/graph_layout_test.rs`:

```rust
//! Tests for the graph layout engine.

use quest::history::graph_layout::{build_graph_layout, GraphLayout, GraphNode};
use quest::history::types::CommitInfo;

fn commit(id: &str, ts: i64, level: u32, prestige: u32, zone: u32) -> CommitInfo {
    CommitInfo {
        id: id.to_string(),
        message: format!("Level up | Lv{level} P{prestige} Z{zone}-1 0h00m @test"),
        timestamp: ts,
        level,
        prestige,
        zone,
        playtime: 0,
    }
}

#[test]
fn single_branch_layout() {
    let branches = vec![("main".to_string(), vec![
        commit("aaa", 300, 30, 2, 5),
        commit("bbb", 200, 20, 1, 3),
        commit("ccc", 100, 10, 0, 1),
    ])];

    let layout = build_graph_layout(&branches);

    assert_eq!(layout.columns.len(), 1);
    assert_eq!(layout.columns[0].branch_name, "main");
    assert_eq!(layout.rows.len(), 3);
    // Newest commit first (timestamp 300)
    assert_eq!(layout.rows[0].timestamp, 300);
}

#[test]
fn two_branch_fork_layout() {
    // main: c1(100) -> c2(200) -> c3(300)
    // fork: c1(100) -> c2(200) -> c4(250)
    // Shared: c1, c2. Fork point = c2.
    let branches = vec![
        ("main".to_string(), vec![
            commit("c3", 300, 30, 2, 5),
            commit("c2", 200, 20, 1, 3),
            commit("c1", 100, 10, 0, 1),
        ]),
        ("fork".to_string(), vec![
            commit("c4", 250, 25, 1, 4),
            commit("c2", 200, 20, 1, 3),
            commit("c1", 100, 10, 0, 1),
        ]),
    ];

    let layout = build_graph_layout(&branches);

    assert_eq!(layout.columns.len(), 2);
    assert_eq!(layout.columns[0].branch_name, "main");
    assert_eq!(layout.columns[1].branch_name, "fork");
    // 4 unique commits: c1, c2, c3, c4
    assert_eq!(layout.rows.len(), 4);
    // Fork connectors should exist at the fork point (c2)
    assert!(!layout.fork_connectors.is_empty());
}

#[test]
fn main_is_always_first_column() {
    let branches = vec![
        ("zebra".to_string(), vec![commit("z1", 100, 10, 0, 1)]),
        ("main".to_string(), vec![commit("m1", 100, 10, 0, 1)]),
    ];

    let layout = build_graph_layout(&branches);
    assert_eq!(layout.columns[0].branch_name, "main");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test graph_layout_test -- --nocapture`
Expected: FAIL — module doesn't exist.

**Step 3: Write minimal implementation**

Create `src/history/graph_layout.rs`:

```rust
//! Graph layout engine for the Time Vault.
//!
//! Converts raw branch+commit data into a positioned grid of nodes, rows,
//! and fork connectors for rendering the graph view.

use std::collections::{HashMap, HashSet};

use super::types::CommitInfo;

/// A column in the graph (one per branch).
#[derive(Debug, Clone)]
pub struct GraphColumn {
    pub branch_name: String,
    pub commit_ids: Vec<String>,
}

/// A row in the graph (one per unique timestamp group).
#[derive(Debug, Clone)]
pub struct GraphRow {
    pub timestamp: i64,
    pub nodes: Vec<Option<GraphNode>>,
}

/// A single node in the graph grid.
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub commit: CommitInfo,
    pub branch_name: String,
    pub is_head: bool,
}

/// A fork connector between two columns at a specific row.
#[derive(Debug, Clone)]
pub struct ForkConnector {
    /// Row index where the fork happened.
    pub row: usize,
    /// Column of the parent branch.
    pub from_col: usize,
    /// Column of the forked branch.
    pub to_col: usize,
}

/// The complete graph layout.
#[derive(Debug, Clone)]
pub struct GraphLayout {
    pub columns: Vec<GraphColumn>,
    pub rows: Vec<GraphRow>,
    pub fork_connectors: Vec<ForkConnector>,
}

/// Build a graph layout from branch data.
///
/// Input: list of `(branch_name, commits_newest_first)` pairs.
/// Output: a `GraphLayout` with columns, rows, and fork connectors.
pub fn build_graph_layout(branches: &[(String, Vec<CommitInfo>)]) -> GraphLayout {
    if branches.is_empty() {
        return GraphLayout {
            columns: vec![],
            rows: vec![],
            fork_connectors: vec![],
        };
    }

    // 1. Build columns, main first.
    let mut columns: Vec<GraphColumn> = Vec::new();
    let mut branch_order: Vec<usize> = Vec::new();

    // Find main and put it first.
    for (i, (name, _)) in branches.iter().enumerate() {
        if name == "main" {
            branch_order.push(i);
            break;
        }
    }
    // Then others in input order.
    for (i, (name, _)) in branches.iter().enumerate() {
        if name != "main" {
            branch_order.push(i);
        }
    }

    for &bi in &branch_order {
        let (name, commits) = &branches[bi];
        columns.push(GraphColumn {
            branch_name: name.clone(),
            commit_ids: commits.iter().map(|c| c.id.clone()).collect(),
        });
    }

    // 2. Collect all unique commits by ID, build a commit map.
    let mut commit_map: HashMap<String, CommitInfo> = HashMap::new();
    let mut all_ids: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    // Determine which commits belong exclusively to which branch.
    let mut commit_branches: HashMap<String, Vec<usize>> = HashMap::new();
    for (col_idx, &bi) in branch_order.iter().enumerate() {
        let (_, commits) = &branches[bi];
        for c in commits {
            commit_branches
                .entry(c.id.clone())
                .or_default()
                .push(col_idx);
            if !seen.contains(&c.id) {
                seen.insert(c.id.clone());
                all_ids.push(c.id.clone());
                commit_map.insert(c.id.clone(), c.clone());
            }
        }
    }

    // 3. Sort all commits by timestamp descending (newest first).
    all_ids.sort_by(|a, b| {
        let ta = commit_map[a].timestamp;
        let tb = commit_map[b].timestamp;
        tb.cmp(&ta)
    });

    // 4. Build rows — one row per unique commit.
    let num_cols = columns.len();
    let mut rows: Vec<GraphRow> = Vec::new();

    for commit_id in &all_ids {
        let commit = &commit_map[commit_id];
        let branch_cols = &commit_branches[commit_id];

        let mut nodes: Vec<Option<GraphNode>> = vec![None; num_cols];
        for &col in branch_cols {
            let is_head = columns[col].commit_ids.first().map(|s| s.as_str())
                == Some(commit_id.as_str());
            nodes[col] = Some(GraphNode {
                commit: commit.clone(),
                branch_name: columns[col].branch_name.clone(),
                is_head,
            });
        }

        rows.push(GraphRow {
            timestamp: commit.timestamp,
            nodes,
        });
    }

    // 5. Find fork connectors.
    // A fork point is the first shared commit between a non-main branch and its parent.
    let mut fork_connectors: Vec<ForkConnector> = Vec::new();

    for (col_idx, column) in columns.iter().enumerate().skip(1) {
        // Walk this branch's commits to find the first one shared with another column.
        for commit_id in &column.commit_ids {
            let branch_cols = &commit_branches[commit_id];
            // Find the leftmost column that also has this commit (that's the parent).
            let parent_col = branch_cols.iter().filter(|&&c| c < col_idx).min().copied();
            if let Some(from_col) = parent_col {
                // Find which row this commit is in.
                if let Some(row_idx) = rows.iter().position(|r| {
                    r.nodes[from_col]
                        .as_ref()
                        .is_some_and(|n| n.commit.id == *commit_id)
                }) {
                    fork_connectors.push(ForkConnector {
                        row: row_idx,
                        from_col,
                        to_col: col_idx,
                    });
                }
                break;
            }
        }
    }

    GraphLayout {
        columns,
        rows,
        fork_connectors,
    }
}
```

Update `src/history/mod.rs`:

```rust
pub mod graph_layout;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test graph_layout_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/history/graph_layout.rs src/history/mod.rs tests/graph_layout_test.rs
git commit -m "feat: add graph layout engine for Time Vault"
```

---

### Task 4: Add GraphState to TimeVaultState

**Files:**
- Modify: `src/ui/time_vault_scene.rs` (add GraphState struct and field)
- Test: `tests/time_vault_view_mode_test.rs` (add graph state tests)

**Step 1: Write the failing test**

Add to `tests/time_vault_view_mode_test.rs`:

```rust
#[test]
fn graph_state_default_selection() {
    let state = TimeVaultState::new(vec![], vec![]);
    assert_eq!(state.graph.selected_col, 0);
    assert_eq!(state.graph.selected_row, 0);
    assert_eq!(state.graph.scroll_offset, 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test time_vault_view_mode_test::graph_state_default_selection -- --nocapture`
Expected: FAIL — `graph` field doesn't exist.

**Step 3: Write minimal implementation**

In `src/ui/time_vault_scene.rs`, add after `ViewMode`:

```rust
/// State for the Graph view.
#[derive(Debug, Clone)]
pub struct GraphState {
    pub selected_col: usize,
    pub selected_row: usize,
    pub scroll_offset: usize,
    pub layout: Option<GraphLayout>,
}

impl Default for GraphState {
    fn default() -> Self {
        Self {
            selected_col: 0,
            selected_row: 0,
            scroll_offset: 0,
            layout: None,
        }
    }
}
```

Add the import at the top:

```rust
use crate::history::graph_layout::GraphLayout;
```

Add `graph: GraphState` to `TimeVaultState`:

```rust
pub graph: GraphState,
```

Initialize in `new()`:

```rust
graph: GraphState::default(),
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test time_vault_view_mode_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/time_vault_scene.rs tests/time_vault_view_mode_test.rs
git commit -m "feat: add GraphState to TimeVaultState"
```

---

### Task 5: Add CompareState to TimeVaultState

**Files:**
- Modify: `src/ui/time_vault_scene.rs` (add CompareState struct and field)
- Test: `tests/time_vault_view_mode_test.rs`

**Step 1: Write the failing test**

Add to `tests/time_vault_view_mode_test.rs`:

```rust
#[test]
fn compare_state_defaults() {
    let state = TimeVaultState::new(vec![], vec![]);
    assert!(state.compare.left_branch.is_none());
    assert!(state.compare.right_branch.is_none());
    assert_eq!(state.compare.scroll_offset, 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test time_vault_view_mode_test::compare_state_defaults -- --nocapture`
Expected: FAIL — `compare` field doesn't exist.

**Step 3: Write minimal implementation**

In `src/ui/time_vault_scene.rs`, add after `GraphState`:

```rust
/// Which phase of comparison the user is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparePhase {
    /// Picking the first branch.
    SelectLeft,
    /// Picking the second branch.
    SelectRight,
    /// Viewing the comparison.
    Viewing,
}

/// State for the Compare view.
#[derive(Debug, Clone)]
pub struct CompareState {
    pub left_branch: Option<String>,
    pub right_branch: Option<String>,
    pub scroll_offset: usize,
    pub phase: ComparePhase,
    pub branch_cursor: usize,
}

impl Default for CompareState {
    fn default() -> Self {
        Self {
            left_branch: None,
            right_branch: None,
            scroll_offset: 0,
            phase: ComparePhase::SelectLeft,
            branch_cursor: 0,
        }
    }
}
```

Add `compare: CompareState` to `TimeVaultState`:

```rust
pub compare: CompareState,
```

Initialize in `new()`:

```rust
compare: CompareState::default(),
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test time_vault_view_mode_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/time_vault_scene.rs tests/time_vault_view_mode_test.rs
git commit -m "feat: add CompareState to TimeVaultState"
```

---

### Task 6: Add `find_fork_point` and `all_commits_graph` to HistoryRepo

**Files:**
- Modify: `src/history/git.rs:181-241` (add new query methods)
- Test: `tests/history_git_test.rs` (add fork point tests)

**Step 1: Write the failing test**

Add to `tests/history_git_test.rs` (or create new test file `tests/history_graph_queries_test.rs`):

```rust
// These tests require a temp git repo so they belong in the integration test
// that already sets up HistoryRepo. Add to existing tests/history_git_test.rs.

#[test]
fn find_fork_point_shared_ancestor() {
    let dir = tempfile::tempdir().unwrap();
    let repo = HistoryRepo::init(dir.path()).unwrap();

    // Create a file and commit on main.
    std::fs::write(dir.path().join("data.txt"), "v1").unwrap();
    repo.commit_raw("commit 1").unwrap();

    std::fs::write(dir.path().join("data.txt"), "v2").unwrap();
    repo.commit_raw("commit 2").unwrap();

    // Fork from current commit.
    let main_commits = repo.list_commits("main").unwrap();
    let head_id = &main_commits[0].id;
    repo.fork_timeline("branch-a", head_id).unwrap();

    // Add commits on the fork.
    std::fs::write(dir.path().join("data.txt"), "v3-fork").unwrap();
    repo.commit_raw("fork commit 1").unwrap();

    // Switch back to main and add commits.
    repo.switch_timeline("main").unwrap();
    std::fs::write(dir.path().join("data.txt"), "v3-main").unwrap();
    repo.commit_raw("main commit 3").unwrap();

    // Find fork point.
    let fork_point = repo.find_fork_point("main", "branch-a").unwrap();
    assert!(fork_point.is_some());
    // Fork point should be "commit 2" (the shared head before divergence).
    assert_eq!(fork_point.unwrap().id, *head_id);
}

#[test]
fn all_commits_graph_returns_all_branches() {
    let dir = tempfile::tempdir().unwrap();
    let repo = HistoryRepo::init(dir.path()).unwrap();

    std::fs::write(dir.path().join("data.txt"), "v1").unwrap();
    repo.commit_raw("commit 1").unwrap();

    let commits = repo.list_commits("main").unwrap();
    repo.fork_timeline("alt", &commits[0].id).unwrap();

    let graph = repo.all_commits_graph().unwrap();
    assert_eq!(graph.len(), 2);
    // main should be first.
    assert_eq!(graph[0].0, "main");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test find_fork_point_shared_ancestor all_commits_graph_returns_all_branches -- --nocapture`
Expected: FAIL — methods don't exist.

**Step 3: Write minimal implementation**

Add to `src/history/git.rs` inside `impl HistoryRepo`, after `list_commits`:

```rust
    /// Find the fork point (common ancestor) of two branches.
    ///
    /// Returns the most recent commit shared by both branches, or None if
    /// the branches share no common history.
    pub fn find_fork_point(
        &self,
        branch_a: &str,
        branch_b: &str,
    ) -> Result<Option<CommitInfo>, HistoryError> {
        let commits_a = self.list_commits(branch_a)?;
        let commits_b = self.list_commits(branch_b)?;

        let ids_b: std::collections::HashSet<String> =
            commits_b.iter().map(|c| c.id.clone()).collect();

        // Walk branch_a's commits (newest first) to find first shared commit.
        for commit in &commits_a {
            if ids_b.contains(&commit.id) {
                return Ok(Some(commit.clone()));
            }
        }

        Ok(None)
    }

    /// Get all branches with their full commit histories for graph rendering.
    ///
    /// Returns a list of `(branch_name, commits_newest_first)` pairs.
    /// "main" is always listed first.
    pub fn all_commits_graph(&self) -> Result<Vec<(String, Vec<CommitInfo>)>, HistoryError> {
        let branches = self.list_branches()?;
        let mut result: Vec<(String, Vec<CommitInfo>)> = Vec::new();

        // Main first.
        for branch in &branches {
            if branch.name == "main" {
                let commits = self.list_commits(&branch.name)?;
                result.push((branch.name.clone(), commits));
                break;
            }
        }

        // Then others.
        for branch in &branches {
            if branch.name != "main" {
                let commits = self.list_commits(&branch.name)?;
                result.push((branch.name.clone(), commits));
            }
        }

        Ok(result)
    }
```

**Step 4: Run tests to verify they pass**

Run: `cargo test find_fork_point_shared_ancestor all_commits_graph_returns_all_branches -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/history/git.rs tests/history_git_test.rs
git commit -m "feat: add find_fork_point and all_commits_graph to HistoryRepo"
```

---

### Task 7: Render the Graph view tab header

**Files:**
- Modify: `src/ui/time_vault_scene.rs:189-258` (add tab bar and dispatch by view_mode)

**Step 1: No separate test needed** — this is pure rendering. We'll verify visually and through existing tests still passing.

**Step 2: Modify `draw_time_vault` to render tab header and dispatch by view mode**

In `draw_time_vault`, after painting the backdrop, add a tab bar row at the top of the buffer, then dispatch rendering by `view_mode`:

```rust
// Paint tab bar at row 0
paint_tab_bar(&mut buffer, state);

// Dispatch by view mode
match state.view_mode {
    ViewMode::Browse => {
        // Existing branch panel + snapshot panel code (shift down by 1 row for tab bar)
        let content_buffer = &mut buffer[1..]; // offset for tab bar
        paint_branch_panel(content_buffer, state, branch_width);
        paint_snapshot_panel(content_buffer, state, snap_x, snap_w);
        if state.mode != BrowserMode::Browse {
            paint_confirm_dialog(content_buffer, state);
        }
    }
    ViewMode::Graph => {
        paint_graph_view(&mut buffer[1..], state);
    }
    ViewMode::Compare => {
        paint_compare_view(&mut buffer[1..], state);
    }
}
```

Add the tab bar painter:

```rust
fn paint_tab_bar(buffer: &mut [Vec<SceneCell>], state: &TimeVaultState) {
    if buffer.is_empty() {
        return;
    }

    let tabs = [
        ("B", "rowse", ViewMode::Browse),
        ("G", "raph", ViewMode::Graph),
        ("C", "ompare", ViewMode::Compare),
    ];

    let mut col = 2i32;
    for (hotkey, label, mode) in &tabs {
        let is_active = state.view_mode == *mode;
        let key_color = Color::Cyan;
        let label_color = if is_active {
            Color::White
        } else {
            Color::DarkGray
        };

        put_text(buffer, 0, col, "[", Color::DarkGray);
        col += 1;
        put_text(buffer, 0, col, hotkey, key_color);
        col += hotkey.len() as i32;
        put_text(buffer, 0, col, "]", Color::DarkGray);
        col += 1;
        put_text(buffer, 0, col, label, label_color);
        col += label.len() as i32 + 2;
    }
}
```

Add stubs for graph and compare views:

```rust
fn paint_graph_view(buffer: &mut [Vec<SceneCell>], state: &TimeVaultState) {
    if buffer.is_empty() {
        return;
    }
    put_text(buffer, 2, 4, "Graph view — coming soon", Color::DarkGray);
}

fn paint_compare_view(buffer: &mut [Vec<SceneCell>], state: &TimeVaultState) {
    if buffer.is_empty() {
        return;
    }
    put_text(buffer, 2, 4, "Compare view — coming soon", Color::DarkGray);
}
```

**Step 3: Run all tests to ensure nothing broke**

Run: `cargo test -- --nocapture`
Expected: All existing tests pass.

**Step 4: Commit**

```bash
git add src/ui/time_vault_scene.rs
git commit -m "feat: add tab bar and view mode dispatch to Time Vault"
```

---

### Task 8: Render the Graph view — column headers and nodes

**Files:**
- Modify: `src/ui/time_vault_scene.rs` (implement `paint_graph_view`)
- Modify: `src/input/time_vault_input.rs` (load graph layout on mode switch)

**Step 1: Implement the graph painting function**

Replace the `paint_graph_view` stub with the full implementation:

```rust
fn paint_graph_view(buffer: &mut [Vec<SceneCell>], state: &TimeVaultState) {
    let height = buffer.len();
    if height < 4 {
        return;
    }
    let width = if buffer[0].is_empty() { return } else { buffer[0].len() };

    let layout = match &state.graph.layout {
        Some(l) => l,
        None => {
            put_text(buffer, 2, 4, "No graph data loaded", Color::DarkGray);
            return;
        }
    };

    if layout.columns.is_empty() {
        put_text(buffer, 2, 4, "No branches", Color::DarkGray);
        return;
    }

    let col_width = 20usize; // chars per column
    let max_cols = (width / col_width).max(1);

    // Column headers (row 0-1)
    for (ci, column) in layout.columns.iter().enumerate().take(max_cols) {
        let x = (ci * col_width + 2) as i32;
        let is_selected_col = ci == state.graph.selected_col;
        let color = if is_selected_col {
            Color::Cyan
        } else {
            Color::DarkGray
        };
        put_text(buffer, 0, x, &column.branch_name, color);

        // Separator
        let sep: String = "\u{2500}".repeat(col_width.saturating_sub(2));
        put_text(buffer, 1, x, &sep, Color::Rgb(30, 50, 80));
    }

    // Commit rows
    let content_start = 2usize;
    let available_rows = height.saturating_sub(content_start);
    let rows_per_commit = 2usize; // node line + connector
    let visible_commits = available_rows / rows_per_commit;

    let scroll = state.graph.scroll_offset;

    for (vi, row) in layout.rows.iter().enumerate().skip(scroll) {
        let screen_row = content_start + (vi - scroll) * rows_per_commit;
        if screen_row + 1 >= height {
            break;
        }

        for (ci, node_opt) in row.nodes.iter().enumerate().take(max_cols) {
            let x = (ci * col_width + 2) as i32;

            if let Some(node) = node_opt {
                let is_selected = ci == state.graph.selected_col
                    && vi == state.graph.selected_row;

                // Node marker
                let marker = if node.is_head { "\u{25cf}" } else { "\u{25cb}" }; // ● or ○
                let node_color = if is_selected {
                    Color::Yellow
                } else if node.is_head {
                    Color::Green
                } else {
                    Color::Cyan
                };
                put_text(buffer, screen_row as i32, x, marker, node_color);

                // Compact label: Lv{N} P{N} Z{N}
                let label = format!(
                    "Lv{} P{} Z{}",
                    node.commit.level, node.commit.prestige, node.commit.zone
                );
                let label_color = if is_selected {
                    Color::Yellow
                } else {
                    Color::White
                };
                put_text(buffer, screen_row as i32, x + 2, &label, label_color);

                // Highlight background for selected
                if is_selected {
                    let r = screen_row;
                    if r < height {
                        let highlight_bg = Color::Rgb(25, 40, 80);
                        for col in (ci * col_width)..(ci * col_width + col_width).min(width) {
                            if col < buffer[r].len() {
                                buffer[r][col].bg = highlight_bg;
                            }
                        }
                    }
                }
            }

            // Vertical connector (if not last visible row)
            if screen_row + 1 < height {
                let has_node_above = node_opt.is_some();
                // Check if there's a node below in this column
                let has_node_below = layout.rows.get(vi + 1)
                    .and_then(|r| r.nodes.get(ci))
                    .and_then(|n| n.as_ref())
                    .is_some();

                if has_node_above || has_node_below {
                    // Check if the CURRENT column has more commits below
                    let show_connector = layout.rows.iter().skip(vi + 1).any(|r| {
                        r.nodes.get(ci).and_then(|n| n.as_ref()).is_some()
                    });
                    if show_connector {
                        put_text(
                            buffer,
                            (screen_row + 1) as i32,
                            x,
                            "\u{2502}",
                            TIMELINE_DIM,
                        );
                    }
                }
            }
        }
    }

    // Fork connectors
    for fc in &layout.fork_connectors {
        let vi = fc.row;
        if vi < scroll {
            continue;
        }
        let screen_row = content_start + (vi - scroll) * rows_per_commit;
        if screen_row >= height {
            continue;
        }

        let from_x = (fc.from_col * col_width + 2) as i32;
        let to_x = (fc.to_col * col_width + 2) as i32;

        // Draw horizontal connector from parent to child column
        put_text(buffer, screen_row as i32, from_x, "\u{251c}", TIMELINE_DIM); // ├
        for cx in (from_x + 1)..to_x {
            put_text(buffer, screen_row as i32, cx, "\u{2500}", TIMELINE_DIM); // ─
        }
        put_text(buffer, screen_row as i32, to_x, "\u{2518}", TIMELINE_DIM); // ┘
    }
}
```

**Step 2: Wire up graph layout building when switching to Graph view**

In `src/input/time_vault_input.rs`, add a new action variant to `TimeVaultAction`:

```rust
/// Build the graph layout (main loop fetches data and populates state.graph.layout).
BuildGraph,
```

In the `KeyCode::Char('g')` handler:

```rust
KeyCode::Char('g') | KeyCode::Char('G') => {
    state.view_mode = ViewMode::Graph;
    return TimeVaultAction::BuildGraph;
}
```

In `src/main.rs`, handle `TimeVaultAction::BuildGraph`:

```rust
TimeVaultAction::BuildGraph => {
    if let Some(repo) = history_repo.as_ref() {
        if let Ok(branch_data) = repo.all_commits_graph() {
            let layout = quest::history::graph_layout::build_graph_layout(&branch_data);
            state.graph.layout = Some(layout);
            state.graph.selected_col = 0;
            state.graph.selected_row = 0;
            state.graph.scroll_offset = 0;
        }
    }
}
```

**Step 3: Run all tests**

Run: `cargo test -- --nocapture`
Expected: All pass. Run `cargo clippy` to fix any warnings.

**Step 4: Commit**

```bash
git add src/ui/time_vault_scene.rs src/input/time_vault_input.rs src/main.rs
git commit -m "feat: render graph view with columns, nodes, and fork connectors"
```

---

### Task 9: Graph view navigation (arrow keys, Enter, Fork)

**Files:**
- Modify: `src/input/time_vault_input.rs` (implement `handle_graph_input`)
- Test: `tests/time_vault_view_mode_test.rs`

**Step 1: Write the failing tests**

Add to `tests/time_vault_view_mode_test.rs`:

```rust
use quest::history::graph_layout::{build_graph_layout, GraphLayout};

fn make_graph_state_with_layout() -> TimeVaultState {
    let commits = vec![
        CommitInfo { id: "c3".into(), message: "Lv30".into(), timestamp: 300, level: 30, prestige: 2, zone: 5, playtime: 0 },
        CommitInfo { id: "c2".into(), message: "Lv20".into(), timestamp: 200, level: 20, prestige: 1, zone: 3, playtime: 0 },
        CommitInfo { id: "c1".into(), message: "Lv10".into(), timestamp: 100, level: 10, prestige: 0, zone: 1, playtime: 0 },
    ];
    let branches = vec![("main".to_string(), commits)];
    let layout = build_graph_layout(&branches);

    let mut state = TimeVaultState::new(vec![], vec![]);
    state.view_mode = ViewMode::Graph;
    state.graph.layout = Some(layout);
    state
}

#[test]
fn graph_down_moves_selection() {
    let mut state = make_graph_state_with_layout();
    assert_eq!(state.graph.selected_row, 0);
    let _ = handle_time_vault_input(key(KeyCode::Down), &mut state);
    assert_eq!(state.graph.selected_row, 1);
}

#[test]
fn graph_up_at_top_stays() {
    let mut state = make_graph_state_with_layout();
    assert_eq!(state.graph.selected_row, 0);
    let _ = handle_time_vault_input(key(KeyCode::Up), &mut state);
    assert_eq!(state.graph.selected_row, 0);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test time_vault_view_mode_test graph_ -- --nocapture`
Expected: FAIL — graph input not implemented.

**Step 3: Implement graph navigation**

Replace the `handle_graph_input` stub:

```rust
fn handle_graph_input(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    // B/G/C handled at top level already.
    match key.code {
        KeyCode::Esc => TimeVaultAction::Close,
        KeyCode::Up => {
            if state.graph.selected_row > 0 {
                state.graph.selected_row -= 1;
                // Adjust scroll
                if state.graph.selected_row < state.graph.scroll_offset {
                    state.graph.scroll_offset = state.graph.selected_row;
                }
            }
            TimeVaultAction::Continue
        }
        KeyCode::Down => {
            let max_row = state
                .graph
                .layout
                .as_ref()
                .map(|l| l.rows.len().saturating_sub(1))
                .unwrap_or(0);
            if state.graph.selected_row < max_row {
                state.graph.selected_row += 1;
            }
            TimeVaultAction::Continue
        }
        KeyCode::Left => {
            if state.graph.selected_col > 0 {
                state.graph.selected_col -= 1;
            }
            TimeVaultAction::Continue
        }
        KeyCode::Right => {
            let max_col = state
                .graph
                .layout
                .as_ref()
                .map(|l| l.columns.len().saturating_sub(1))
                .unwrap_or(0);
            if state.graph.selected_col < max_col {
                state.graph.selected_col += 1;
            }
            TimeVaultAction::Continue
        }
        KeyCode::Enter => {
            // Get the selected node's commit ID for restore.
            if let Some(commit_id) = get_selected_graph_commit_id(state) {
                state.mode = BrowserMode::ConfirmRestore;
                // Temporarily store the commit ID so the confirm dialog can use it.
                // We'll set selected_commit to match in the commits list.
                TimeVaultAction::Continue
            } else {
                TimeVaultAction::Continue
            }
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            if let Some(commit_id) = get_selected_graph_commit_id(state) {
                state.mode = BrowserMode::NamingFork { commit_id };
                state.fork_name_input.clear();
                state.fork_name_error = None;
            }
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}

fn get_selected_graph_commit_id(state: &TimeVaultState) -> Option<String> {
    state
        .graph
        .layout
        .as_ref()
        .and_then(|l| l.rows.get(state.graph.selected_row))
        .and_then(|row| row.nodes.get(state.graph.selected_col))
        .and_then(|node| node.as_ref())
        .map(|n| n.commit.id.clone())
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --test time_vault_view_mode_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/input/time_vault_input.rs tests/time_vault_view_mode_test.rs
git commit -m "feat: add graph view navigation (arrows, Enter, Fork)"
```

---

### Task 10: Render the Compare view — branch picker and stats

**Files:**
- Modify: `src/ui/time_vault_scene.rs` (implement `paint_compare_view`)
- Modify: `src/input/time_vault_input.rs` (implement `handle_compare_input`)

**Step 1: Implement the compare view painting**

Replace `paint_compare_view` stub:

```rust
fn paint_compare_view(buffer: &mut [Vec<SceneCell>], state: &TimeVaultState) {
    let height = buffer.len();
    if height < 4 {
        return;
    }
    let width = if buffer[0].is_empty() { return } else { buffer[0].len() };

    match state.compare.phase {
        ComparePhase::SelectLeft | ComparePhase::SelectRight => {
            paint_compare_branch_picker(buffer, state);
        }
        ComparePhase::Viewing => {
            paint_compare_stats(buffer, state, width);
        }
    }
}

fn paint_compare_branch_picker(buffer: &mut [Vec<SceneCell>], state: &TimeVaultState) {
    let prompt = match state.compare.phase {
        ComparePhase::SelectLeft => "Select first branch to compare:",
        ComparePhase::SelectRight => "Select second branch to compare:",
        _ => return,
    };
    put_text(buffer, 1, 4, prompt, Color::White);

    for (i, branch) in state.branches.iter().enumerate() {
        let row = 3 + i as i32;
        if row as usize >= buffer.len() {
            break;
        }

        let is_selected = i == state.compare.branch_cursor;
        let marker = if is_selected { "\u{25b6}" } else { " " }; // ▶ or space
        let color = if is_selected {
            Color::Yellow
        } else {
            Color::White
        };

        put_text(buffer, row, 4, marker, Color::Cyan);
        put_text(buffer, row, 6, &branch.name, color);

        // Show if this is already picked as left
        if state.compare.left_branch.as_deref() == Some(&branch.name) {
            put_text(buffer, row, 6 + branch.name.len() as i32 + 2, "(left)", Color::DarkGray);
        }
    }
}

fn paint_compare_stats(
    buffer: &mut [Vec<SceneCell>],
    state: &TimeVaultState,
    width: usize,
) {
    let left_name = state.compare.left_branch.as_deref().unwrap_or("?");
    let right_name = state.compare.right_branch.as_deref().unwrap_or("?");

    // Headers
    let mid = width / 2;
    put_text(buffer, 0, 4, left_name, Color::Cyan);
    put_text(buffer, 0, mid as i32 + 2, "vs", Color::DarkGray);
    put_text(buffer, 0, mid as i32 + 6, right_name, Color::Cyan);

    // Find head commits for each branch
    let left_head = state.branches.iter()
        .find(|b| b.name == left_name)
        .and_then(|b| b.head_commit.as_ref());
    let right_head = state.branches.iter()
        .find(|b| b.name == right_name)
        .and_then(|b| b.head_commit.as_ref());

    let sep: String = "\u{2500}".repeat(width.saturating_sub(4));
    put_text(buffer, 1, 2, &sep, Color::Rgb(30, 50, 80));

    // Stats rows
    let labels = ["Level", "Prestige", "Zone", "Playtime"];
    for (i, label) in labels.iter().enumerate() {
        let row = 3 + i as i32;
        if row as usize >= buffer.len() {
            break;
        }

        put_text(buffer, row, 4, label, Color::DarkGray);

        if let Some(commit) = left_head {
            let val = match *label {
                "Level" => format!("{}", commit.level),
                "Prestige" => format!("{}", commit.prestige),
                "Zone" => format!("{}", commit.zone),
                "Playtime" => {
                    let h = commit.playtime / 3600;
                    let m = (commit.playtime % 3600) / 60;
                    format!("{}h {:02}m", h, m)
                }
                _ => String::new(),
            };
            put_text(buffer, row, 16, &val, Color::White);
        }

        if let Some(commit) = right_head {
            let val = match *label {
                "Level" => format!("{}", commit.level),
                "Prestige" => format!("{}", commit.prestige),
                "Zone" => format!("{}", commit.zone),
                "Playtime" => {
                    let h = commit.playtime / 3600;
                    let m = (commit.playtime % 3600) / 60;
                    format!("{}h {:02}m", h, m)
                }
                _ => String::new(),
            };
            put_text(buffer, row, mid as i32 + 6, &val, Color::White);
        }
    }
}
```

**Step 2: Implement compare input handling**

Replace `handle_compare_input` stub:

```rust
fn handle_compare_input(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match state.compare.phase {
        ComparePhase::SelectLeft | ComparePhase::SelectRight => {
            match key.code {
                KeyCode::Esc => {
                    // If selecting right, go back to left. If selecting left, close.
                    if state.compare.phase == ComparePhase::SelectRight {
                        state.compare.phase = ComparePhase::SelectLeft;
                        state.compare.right_branch = None;
                    } else {
                        TimeVaultAction::Close;
                    }
                    TimeVaultAction::Continue
                }
                KeyCode::Up => {
                    if state.compare.branch_cursor > 0 {
                        state.compare.branch_cursor -= 1;
                    }
                    TimeVaultAction::Continue
                }
                KeyCode::Down => {
                    if !state.branches.is_empty()
                        && state.compare.branch_cursor < state.branches.len() - 1
                    {
                        state.compare.branch_cursor += 1;
                    }
                    TimeVaultAction::Continue
                }
                KeyCode::Enter => {
                    if let Some(branch) = state.branches.get(state.compare.branch_cursor) {
                        let name = branch.name.clone();
                        match state.compare.phase {
                            ComparePhase::SelectLeft => {
                                state.compare.left_branch = Some(name);
                                state.compare.phase = ComparePhase::SelectRight;
                                state.compare.branch_cursor = 0;
                            }
                            ComparePhase::SelectRight => {
                                state.compare.right_branch = Some(name);
                                state.compare.phase = ComparePhase::Viewing;
                            }
                            _ => {}
                        }
                    }
                    TimeVaultAction::Continue
                }
                _ => TimeVaultAction::Continue,
            }
        }
        ComparePhase::Viewing => {
            match key.code {
                KeyCode::Esc => {
                    // Reset compare state.
                    state.compare = CompareState::default();
                    TimeVaultAction::Close
                }
                KeyCode::Up => {
                    if state.compare.scroll_offset > 0 {
                        state.compare.scroll_offset -= 1;
                    }
                    TimeVaultAction::Continue
                }
                KeyCode::Down => {
                    state.compare.scroll_offset += 1;
                    TimeVaultAction::Continue
                }
                _ => TimeVaultAction::Continue,
            }
        }
    }
}
```

**Step 3: Run all tests**

Run: `cargo test -- --nocapture`
Expected: All pass.

**Step 4: Commit**

```bash
git add src/ui/time_vault_scene.rs src/input/time_vault_input.rs
git commit -m "feat: add Compare view with branch picker and stats display"
```

---

### Task 11: Add divergence section to Compare view

**Files:**
- Modify: `src/ui/time_vault_scene.rs` (extend `paint_compare_stats`)
- Modify: `src/input/time_vault_input.rs` (add `LoadForkPoint` action)

**Step 1: Add fork point data to CompareState**

In `src/ui/time_vault_scene.rs`, add to `CompareState`:

```rust
pub fork_point: Option<CommitInfo>,
pub left_commits: Vec<CommitInfo>,
pub right_commits: Vec<CommitInfo>,
```

Initialize in `Default`:

```rust
fork_point: None,
left_commits: vec![],
right_commits: vec![],
```

**Step 2: Add a new action for loading compare data**

In `TimeVaultAction`:

```rust
/// Load comparison data (fork point, commit histories) for two branches.
LoadCompareData {
    left_branch: String,
    right_branch: String,
},
```

When the user picks the second branch (in `handle_compare_input`, `ComparePhase::SelectRight` → `Enter`), return this action:

```rust
ComparePhase::SelectRight => {
    state.compare.right_branch = Some(name.clone());
    state.compare.phase = ComparePhase::Viewing;
    return TimeVaultAction::LoadCompareData {
        left_branch: state.compare.left_branch.clone().unwrap_or_default(),
        right_branch: name,
    };
}
```

**Step 3: Handle in main.rs**

```rust
TimeVaultAction::LoadCompareData { left_branch, right_branch } => {
    if let Some(repo) = history_repo.as_ref() {
        if let Ok(fork_point) = repo.find_fork_point(&left_branch, &right_branch) {
            state.compare.fork_point = fork_point;
        }
        if let Ok(commits) = repo.list_commits(&left_branch) {
            state.compare.left_commits = commits;
        }
        if let Ok(commits) = repo.list_commits(&right_branch) {
            state.compare.right_commits = commits;
        }
    }
}
```

**Step 4: Render divergence section in `paint_compare_stats`**

After the stats rows, add:

```rust
    // Divergence section
    let div_start = 8i32;
    let div_sep: String = "\u{2500}".repeat(width.saturating_sub(4));
    put_text(buffer, div_start, 2, &div_sep, Color::Rgb(30, 50, 80));
    put_text(buffer, div_start + 1, 4, "Divergence", Color::Cyan);

    if let Some(fork) = &state.compare.fork_point {
        let fork_label = format!(
            "Forked at: Lv{} P{} Z{}",
            fork.level, fork.prestige, fork.zone
        );
        put_text(buffer, div_start + 2, 4, &fork_label, Color::DarkGray);

        // Count commits since fork on each side
        let left_since = state.compare.left_commits.iter()
            .take_while(|c| c.id != fork.id)
            .count();
        let right_since = state.compare.right_commits.iter()
            .take_while(|c| c.id != fork.id)
            .count();
        let since_label = format!(
            "Since fork: {} snapshots (left) vs {} snapshots (right)",
            left_since, right_since
        );
        put_text(buffer, div_start + 3, 4, &since_label, Color::DarkGray);
    } else {
        put_text(buffer, div_start + 2, 4, "No common ancestor found", Color::DarkGray);
    }
```

**Step 5: Run all tests and clippy**

Run: `cargo test && cargo clippy --all-targets -- -D warnings`
Expected: All pass.

**Step 6: Commit**

```bash
git add src/ui/time_vault_scene.rs src/input/time_vault_input.rs src/main.rs
git commit -m "feat: add divergence section to Compare view"
```

---

### Task 12: Add interleaved timeline to Compare view

**Files:**
- Modify: `src/ui/time_vault_scene.rs` (extend `paint_compare_stats` with timeline section)

**Step 1: Add timeline rendering after divergence section**

In `paint_compare_stats`, after the divergence section:

```rust
    // Interleaved timeline section
    let tl_start = div_start + 5;
    let tl_sep: String = "\u{2500}".repeat(width.saturating_sub(4));
    put_text(buffer, tl_start, 2, &tl_sep, Color::Rgb(30, 50, 80));
    put_text(buffer, tl_start + 1, 4, "Timeline", Color::Cyan);

    // Collect commits unique to each side (before fork point)
    let fork_id = state.compare.fork_point.as_ref().map(|f| f.id.as_str());

    let left_unique: Vec<&CommitInfo> = state.compare.left_commits.iter()
        .take_while(|c| fork_id.map_or(true, |fid| c.id != fid))
        .collect();
    let right_unique: Vec<&CommitInfo> = state.compare.right_commits.iter()
        .take_while(|c| fork_id.map_or(true, |fid| c.id != fid))
        .collect();

    // Interleave by timestamp (newest first)
    let mut merged: Vec<(&CommitInfo, bool)> = Vec::new(); // (commit, is_left)
    merged.extend(left_unique.iter().map(|c| (*c, true)));
    merged.extend(right_unique.iter().map(|c| (*c, false)));
    merged.sort_by(|a, b| b.0.timestamp.cmp(&a.0.timestamp));

    let scroll = state.compare.scroll_offset;
    let mut row = tl_start + 2;

    for (i, (commit, is_left)) in merged.iter().enumerate().skip(scroll) {
        if row as usize >= buffer.len().saturating_sub(1) {
            break;
        }

        let side_x = if *is_left { 4i32 } else { mid as i32 + 6 };
        let side_color = if *is_left { Color::Cyan } else { Color::Yellow };

        let (icon, icon_color) = event_icon_color(&commit.message);
        let desc = commit.message.split(" | ").next().unwrap_or(&commit.message);
        let label = format!("Lv{}", commit.level);

        put_text(buffer, row, side_x, "\u{25cb}", side_color); // ○
        put_text(buffer, row, side_x + 2, icon, icon_color);
        let iw = super::scene_fx::display_width(icon);
        let desc_trunc: String = desc.chars().take(25).collect();
        put_text(buffer, row, side_x + 2 + iw as i32 + 1, &desc_trunc, Color::White);

        row += 1;
    }

    // Fork point marker
    if row < buffer.len() as i32 && state.compare.fork_point.is_some() {
        let fork_marker = format!(
            "\u{251c}{}\u{2518}  (fork point)",
            "\u{2500}".repeat(mid.saturating_sub(8))
        );
        put_text(buffer, row, 4, &fork_marker, TIMELINE_DIM);
    }
```

**Step 2: Run all tests**

Run: `cargo test && cargo clippy --all-targets -- -D warnings`
Expected: All pass.

**Step 3: Commit**

```bash
git add src/ui/time_vault_scene.rs
git commit -m "feat: add interleaved timeline to Compare view"
```

---

### Task 13: Update controls bar for Graph and Compare views

**Files:**
- Modify: `src/ui/time_vault_scene.rs:628-684` (extend `draw_controls`)

**Step 1: Add view-mode-aware controls**

In `draw_controls`, before the existing `BrowserMode::Browse` match, add view-mode dispatch:

```rust
fn draw_controls(frame: &mut Frame, area: Rect, state: &TimeVaultState) {
    let controls = match state.view_mode {
        ViewMode::Graph => {
            let dot = Span::styled("  \u{00b7}  ", Style::default().fg(Color::Rgb(40, 80, 120)));
            Line::from(vec![
                Span::styled(" [\u{2190}\u{2191}\u{2192}\u{2193}] ", Style::default().fg(Color::Cyan)),
                Span::styled("Navigate", Style::default().fg(Color::DarkGray)),
                dot.clone(),
                Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
                Span::styled("Detail", Style::default().fg(Color::DarkGray)),
                dot.clone(),
                Span::styled("[F] ", Style::default().fg(Color::Cyan)),
                Span::styled("Fork", Style::default().fg(Color::DarkGray)),
                dot.clone(),
                Span::styled("[C] ", Style::default().fg(Color::Cyan)),
                Span::styled("Compare", Style::default().fg(Color::DarkGray)),
                dot,
                Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
                Span::styled("Close", Style::default().fg(Color::DarkGray)),
            ])
        }
        ViewMode::Compare => {
            let dot = Span::styled("  \u{00b7}  ", Style::default().fg(Color::Rgb(40, 80, 120)));
            match state.compare.phase {
                ComparePhase::SelectLeft | ComparePhase::SelectRight => {
                    Line::from(vec![
                        Span::styled(" [\u{2191}\u{2193}] ", Style::default().fg(Color::Cyan)),
                        Span::styled("Select", Style::default().fg(Color::DarkGray)),
                        dot.clone(),
                        Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
                        Span::styled("Pick", Style::default().fg(Color::DarkGray)),
                        dot,
                        Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
                        Span::styled("Back", Style::default().fg(Color::DarkGray)),
                    ])
                }
                ComparePhase::Viewing => {
                    Line::from(vec![
                        Span::styled(" [\u{2191}\u{2193}] ", Style::default().fg(Color::Cyan)),
                        Span::styled("Scroll", Style::default().fg(Color::DarkGray)),
                        dot.clone(),
                        Span::styled("[B] ", Style::default().fg(Color::Cyan)),
                        Span::styled("Browse", Style::default().fg(Color::DarkGray)),
                        dot.clone(),
                        Span::styled("[G] ", Style::default().fg(Color::Cyan)),
                        Span::styled("Graph", Style::default().fg(Color::DarkGray)),
                        dot,
                        Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
                        Span::styled("Close", Style::default().fg(Color::DarkGray)),
                    ])
                }
            }
        }
        ViewMode::Browse => {
            // ... existing browse controls (unchanged) ...
        }
    };

    let paragraph = Paragraph::new(controls).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}
```

**Step 2: Run all tests**

Run: `cargo test && cargo clippy --all-targets -- -D warnings`
Expected: All pass.

**Step 3: Commit**

```bash
git add src/ui/time_vault_scene.rs
git commit -m "feat: add context-sensitive controls for Graph and Compare views"
```

---

### Task 14: Final integration — wire Graph view from Compare (C key in Graph)

**Files:**
- Modify: `src/input/time_vault_input.rs` (C key in graph preselects branch)

**Step 1: In `handle_graph_input`, handle C key**

Add to the graph input handler:

```rust
KeyCode::Char('c') | KeyCode::Char('C') => {
    // Pre-select the current graph column's branch as the left side.
    if let Some(layout) = &state.graph.layout {
        if let Some(column) = layout.columns.get(state.graph.selected_col) {
            state.compare.left_branch = Some(column.branch_name.clone());
            state.compare.phase = ComparePhase::SelectRight;
            state.compare.branch_cursor = 0;
            state.view_mode = ViewMode::Compare;
        }
    }
    TimeVaultAction::Continue
}
```

**Step 2: Run all tests and full CI checks**

Run: `make check`
Expected: All pass (format, clippy, tests, build).

**Step 3: Commit**

```bash
git add src/input/time_vault_input.rs
git commit -m "feat: C key in Graph view pre-selects branch for comparison"
```

---

### Task 15: Update CLAUDE.md and help overlay

**Files:**
- Modify: `CLAUDE.md` (update Time Vault section with new view modes)
- Modify: `src/ui/help_overlay.rs` (add Graph/Compare keybindings)

**Step 1: Update CLAUDE.md**

Add to the history module description:

```markdown
- `graph_layout.rs` — Graph layout engine: builds positioned grid from branch+commit data
```

Update the Time Vault description to mention the three view modes (Browse, Graph, Compare).

**Step 2: Update help overlay**

Add the new keybindings to the help overlay for Time Vault:
- `G` — Graph view
- `C` — Compare view
- `B` — Browse view
- `←→` — Switch columns (Graph)

**Step 3: Run all tests**

Run: `make check`
Expected: All pass.

**Step 4: Commit**

```bash
git add CLAUDE.md src/ui/help_overlay.rs
git commit -m "docs: update CLAUDE.md and help overlay for branch visualization"
```
