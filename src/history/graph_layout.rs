//! Graph layout engine for the Time Vault.
//!
//! Converts raw branch+commit data into a positioned grid of nodes, rows,
//! and fork connectors for rendering the graph view.

use std::collections::{HashMap, HashSet};

use super::types::CommitInfo;

/// A column in the graph (one per branch).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GraphColumn {
    pub branch_name: String,
    pub commit_ids: Vec<String>,
}

/// A row in the graph (one per unique timestamp group).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GraphRow {
    pub timestamp: i64,
    pub nodes: Vec<Option<GraphNode>>,
}

/// A single node in the graph grid.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GraphNode {
    pub commit: CommitInfo,
    pub branch_name: String,
    pub is_head: bool,
}

/// A fork connector between two columns at a specific row.
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct GraphLayout {
    pub columns: Vec<GraphColumn>,
    pub rows: Vec<GraphRow>,
    pub fork_connectors: Vec<ForkConnector>,
}

/// Build a graph layout from branch data.
///
/// Input: list of `(branch_name, commits_newest_first)` pairs.
/// Output: a `GraphLayout` with columns, rows, and fork connectors.
#[allow(dead_code)]
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

    for (i, (name, _)) in branches.iter().enumerate() {
        if name == "main" {
            branch_order.push(i);
            break;
        }
    }
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

    // 2. Collect all unique commits by ID.
    let mut commit_map: HashMap<String, CommitInfo> = HashMap::new();
    let mut all_ids: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
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

    // 3. Sort by timestamp descending.
    all_ids.sort_by(|a, b| {
        let ta = commit_map[a].timestamp;
        let tb = commit_map[b].timestamp;
        tb.cmp(&ta)
    });

    // 4. Build rows.
    let num_cols = columns.len();
    let mut rows: Vec<GraphRow> = Vec::new();

    for commit_id in &all_ids {
        let commit = &commit_map[commit_id];
        let branch_cols = &commit_branches[commit_id];
        let mut nodes: Vec<Option<GraphNode>> = vec![None; num_cols];
        for &col in branch_cols {
            let is_head =
                columns[col].commit_ids.first().map(|s| s.as_str()) == Some(commit_id.as_str());
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
    let mut fork_connectors: Vec<ForkConnector> = Vec::new();
    for (col_idx, column) in columns.iter().enumerate().skip(1) {
        for commit_id in &column.commit_ids {
            let branch_cols = &commit_branches[commit_id];
            let parent_col = branch_cols.iter().filter(|&&c| c < col_idx).min().copied();
            if let Some(from_col) = parent_col {
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
