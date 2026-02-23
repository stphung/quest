//! Tests for the graph layout engine.

use quest::history::graph_layout::build_graph_layout;
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
    let branches = vec![(
        "main".to_string(),
        vec![
            commit("aaa", 300, 30, 2, 5),
            commit("bbb", 200, 20, 1, 3),
            commit("ccc", 100, 10, 0, 1),
        ],
    )];
    let layout = build_graph_layout(&branches);
    assert_eq!(layout.columns.len(), 1);
    assert_eq!(layout.columns[0].branch_name, "main");
    assert_eq!(layout.rows.len(), 3);
    assert_eq!(layout.rows[0].timestamp, 300);
}

#[test]
fn two_branch_fork_layout() {
    let branches = vec![
        (
            "main".to_string(),
            vec![
                commit("c3", 300, 30, 2, 5),
                commit("c2", 200, 20, 1, 3),
                commit("c1", 100, 10, 0, 1),
            ],
        ),
        (
            "fork".to_string(),
            vec![
                commit("c4", 250, 25, 1, 4),
                commit("c2", 200, 20, 1, 3),
                commit("c1", 100, 10, 0, 1),
            ],
        ),
    ];
    let layout = build_graph_layout(&branches);
    assert_eq!(layout.columns.len(), 2);
    assert_eq!(layout.columns[0].branch_name, "main");
    assert_eq!(layout.columns[1].branch_name, "fork");
    assert_eq!(layout.rows.len(), 4);
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
