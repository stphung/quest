//! Shuttle management tests: build errors, demolish/reindex, capacity limits,
//! graph_dirty triggers, and milestone curve.

use super::helpers::*;
use quest::loom::discovery::complete_discovery;
use quest::loom::logic::{
    build_shuttle, demolish_shuttle, eligible_sources_for_tier, node_neighbors, node_upgrade_cost,
    node_upgrade_duration, shuttle_construction_secs, tick_node_upgrades,
    tick_shuttle_construction, try_upgrade_node, unlocked_tiers, upgrade_shuttle,
    valid_source_for_tier, ShuttleError, ShuttleUpgradeError,
};
use quest::loom::types::*;

// ── Build error paths ────────────────────────────────────────────────────────

#[test]
fn test_build_shuttle_tier_locked_error() {
    let mut loom = setup_loom();
    // 0 patterns — T1 needs 1
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 3);

    let result = build_shuttle(
        &mut loom,
        Resource::Ember,
        Resource::VoidEssence,
        NodeNature::Heat,
        vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
        vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
    );
    assert!(matches!(result, Err(ShuttleError::TierLocked)));
}

#[test]
fn test_build_shuttle_t2_locked_before_8_patterns() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 7);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 3);

    let result = build_shuttle(
        &mut loom,
        Resource::ForgedLight,
        Resource::Memory,
        NodeNature::Form,
        vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
        vec![LoomNodeRef::Extractor(NodeId::MemoryArchive)],
    );
    assert!(matches!(result, Err(ShuttleError::TierLocked)));
}

#[test]
fn test_build_shuttle_at_capacity_returns_error() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 15);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 3);

    for _ in 0..5 {
        loom.persistent.shuttles.push(make_t1_shuttle(
            Resource::Ember,
            NodeId::EmberSpindle,
            Resource::VoidEssence,
            NodeId::VoidCondenser,
            NodeNature::Heat,
            Resource::ForgedLight,
        ));
    }

    let result = build_shuttle(
        &mut loom,
        Resource::Ember,
        Resource::VoidEssence,
        NodeNature::Heat,
        vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
        vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
    );
    assert!(matches!(result, Err(ShuttleError::AtCapacity)));
}

// ── Shuttle capacity milestone curve ─────────────────────────────────────────

#[test]
fn test_max_shuttles_milestone_curve() {
    let test_cases: Vec<(usize, usize)> = vec![
        (0, 0),
        (1, 1),
        (3, 1),
        (4, 2),
        (7, 2),
        (8, 3),
        (11, 3),
        (12, 4),
        (14, 4),
        (15, 5),
        (28, 5),
    ];

    for (patterns, expected_slots) in test_cases {
        let mut loom = LoomState::new();
        complete_discovery(&mut loom);
        complete_n_patterns(&mut loom, patterns);
        let actual = loom.persistent.max_shuttles();
        assert_eq!(
            actual, expected_slots,
            "With {patterns} patterns, expected {expected_slots} slots but got {actual}"
        );
    }
}

// ── Demolish and reindex ─────────────────────────────────────────────────────

#[test]
fn test_demolish_shuttle_middle_element() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 8);

    let s0 = make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    );
    let s1 = make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    );
    let s2 = make_t1_shuttle(
        Resource::Silence,
        NodeId::SilenceWell,
        Resource::Resonance,
        NodeId::ResonanceForge,
        NodeNature::Stillness,
        Resource::StillbornSong,
    );
    loom.persistent.shuttles = vec![s0, s1, s2];

    // T2 shuttle pulling from Shuttle(2)
    let mut s3 = make_t2_shuttle(
        Resource::StillbornSong,
        LoomNodeRef::Shuttle(2),
        Resource::Ember,
        LoomNodeRef::Extractor(NodeId::EmberSpindle),
        NodeNature::Heat,
        Resource::CondensedEmber,
    );
    s3.under_construction = false;
    loom.persistent.shuttles.push(s3);

    demolish_shuttle(&mut loom, 1);

    assert_eq!(loom.persistent.shuttles.len(), 3);
    let t2_shuttle = &loom.persistent.shuttles[2];
    assert_eq!(
        t2_shuttle.sources_a,
        vec![LoomNodeRef::Shuttle(1)],
        "Source reference should be reindexed from Shuttle(2) to Shuttle(1)"
    );
}

#[test]
fn test_demolish_shuttle_last_element() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    loom.persistent.shuttles = vec![
        make_t1_shuttle(
            Resource::Ember,
            NodeId::EmberSpindle,
            Resource::VoidEssence,
            NodeId::VoidCondenser,
            NodeNature::Heat,
            Resource::ForgedLight,
        ),
        make_t1_shuttle(
            Resource::Memory,
            NodeId::MemoryArchive,
            Resource::Silence,
            NodeId::SilenceWell,
            NodeNature::Pattern,
            Resource::EchoGlass,
        ),
    ];

    demolish_shuttle(&mut loom, 1);

    assert_eq!(loom.persistent.shuttles.len(), 1);
    assert_eq!(loom.persistent.shuttles[0].output, Resource::ForgedLight);
}

#[test]
fn test_demolish_shuttle_out_of_bounds() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));

    demolish_shuttle(&mut loom, 99);
    assert_eq!(
        loom.persistent.shuttles.len(),
        1,
        "Out-of-bounds demolish should be no-op"
    );
}

// ── graph_dirty triggers ─────────────────────────────────────────────────────

#[test]
fn test_graph_dirty_set_on_build_shuttle() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 3);
    assert!(!loom.graph_dirty);

    let result = build_shuttle(
        &mut loom,
        Resource::Ember,
        Resource::VoidEssence,
        NodeNature::Heat,
        vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
        vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
    );
    assert!(result.is_ok());
    assert!(loom.graph_dirty, "graph_dirty should be true after build");
}

#[test]
fn test_graph_dirty_set_on_demolish_shuttle() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    loom.graph_dirty = false;

    demolish_shuttle(&mut loom, 0);
    assert!(
        loom.graph_dirty,
        "graph_dirty should be true after demolish"
    );
}

#[test]
fn test_graph_dirty_set_on_upgrade_start() {
    let mut loom = setup_loom();
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1);

    loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer =
        loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer_capacity;
    loom.graph_dirty = false;

    let started = try_upgrade_node(&mut loom, NodeId::EmberSpindle);
    assert!(started);
    assert!(
        loom.graph_dirty,
        "graph_dirty should be true after upgrade start"
    );
}

#[test]
fn test_graph_dirty_set_on_upgrade_completion() {
    let mut loom = setup_loom();
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1);

    loom.persistent.nodes[NodeId::EmberSpindle.index()].upgrading = true;
    loom.persistent.nodes[NodeId::EmberSpindle.index()].upgrade_remaining_secs = 0.05;
    loom.graph_dirty = false;

    tick_node_upgrades(&mut loom, 0.1);

    assert!(!loom.persistent.nodes[NodeId::EmberSpindle.index()].upgrading);
    assert!(
        loom.graph_dirty,
        "graph_dirty should be true after upgrade completion"
    );
}

// ── valid_source_for_tier ────────────────────────────────────────────────────

#[test]
fn test_valid_source_extractor_always_valid() {
    let shuttles = vec![];
    assert!(valid_source_for_tier(
        LoomNodeRef::Extractor(NodeId::EmberSpindle),
        1,
        &shuttles
    ));
    assert!(valid_source_for_tier(
        LoomNodeRef::Extractor(NodeId::EmberSpindle),
        3,
        &shuttles
    ));
}

#[test]
fn test_valid_source_shuttle_must_be_lower_tier() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 15);

    let t1 = make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    );
    loom.persistent.shuttles = vec![t1];

    // T1 shuttle is valid source for T2
    assert!(valid_source_for_tier(
        LoomNodeRef::Shuttle(0),
        2,
        &loom.persistent.shuttles
    ));
    // T1 shuttle is NOT valid source for T1 (same tier)
    assert!(!valid_source_for_tier(
        LoomNodeRef::Shuttle(0),
        1,
        &loom.persistent.shuttles
    ));
}

#[test]
fn test_valid_source_shuttle_out_of_bounds() {
    let shuttles = vec![];
    assert!(
        !valid_source_for_tier(LoomNodeRef::Shuttle(99), 2, &shuttles),
        "Out-of-bounds shuttle index should be invalid"
    );
}

// ── unlocked_tiers ───────────────────────────────────────────────────────────

#[test]
fn test_unlocked_tiers_boundaries() {
    let mut loom = setup_loom();

    // 0 patterns: no tiers
    assert!(unlocked_tiers(&loom).is_empty());

    // 1 pattern: T1 only
    complete_n_patterns(&mut loom, 1);
    assert_eq!(unlocked_tiers(&loom), vec![1]);

    // 7 patterns: still T1 only
    complete_n_patterns(&mut loom, 7);
    assert_eq!(unlocked_tiers(&loom), vec![1]);

    // 8 patterns: T1 + T2
    complete_n_patterns(&mut loom, 8);
    assert_eq!(unlocked_tiers(&loom), vec![1, 2]);

    // 14 patterns: still T1 + T2
    complete_n_patterns(&mut loom, 14);
    assert_eq!(unlocked_tiers(&loom), vec![1, 2]);

    // 15 patterns: T1 + T2 + T3
    complete_n_patterns(&mut loom, 15);
    assert_eq!(unlocked_tiers(&loom), vec![1, 2, 3]);
}

// ── eligible_sources_for_tier ────────────────────────────────────────────────

#[test]
fn test_eligible_sources_includes_unlocked_extractors() {
    let loom = setup_loom(); // All nodes unlocked
    let sources = eligible_sources_for_tier(&loom, 1, Resource::Ember);
    assert!(
        sources.contains(&LoomNodeRef::Extractor(NodeId::EmberSpindle)),
        "EmberSpindle should be eligible source for Ember"
    );
    // VoidCondenser does not produce Ember
    assert!(
        !sources.contains(&LoomNodeRef::Extractor(NodeId::VoidCondenser)),
        "VoidCondenser should not be eligible for Ember"
    );
}

#[test]
fn test_eligible_sources_excludes_locked_extractors() {
    let mut loom = LoomState::new();
    quest::loom::discovery::complete_discovery(&mut loom);
    // Only EmberSpindle is unlocked after discovery

    let sources = eligible_sources_for_tier(&loom, 1, Resource::Ember);
    assert!(
        sources.contains(&LoomNodeRef::Extractor(NodeId::EmberSpindle)),
        "Unlocked EmberSpindle should be eligible"
    );

    // Silence is produced by SilenceWell which is still locked
    let silence_sources = eligible_sources_for_tier(&loom, 1, Resource::Silence);
    assert!(
        silence_sources.is_empty(),
        "Locked SilenceWell should not be eligible"
    );
}

#[test]
fn test_eligible_sources_includes_lower_tier_shuttles() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 8);

    let t1 = make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    );
    loom.persistent.shuttles = vec![t1];

    // T2 should see T1 shuttle as eligible source for ForgedLight
    let sources = eligible_sources_for_tier(&loom, 2, Resource::ForgedLight);
    assert!(
        sources.contains(&LoomNodeRef::Shuttle(0)),
        "T1 shuttle should be eligible source for T2"
    );

    // T1 should NOT see T1 shuttle (same tier)
    let t1_sources = eligible_sources_for_tier(&loom, 1, Resource::ForgedLight);
    assert!(
        !t1_sources.contains(&LoomNodeRef::Shuttle(0)),
        "T1 shuttle should not be eligible source for another T1"
    );
}

// ── available_resource ───────────────────────────────────────────────────────

#[test]
fn test_available_resource_sums_extractors() {
    let mut loom = setup_loom();
    loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer = 100.0;
    let available = quest::loom::logic::available_resource(&loom, Resource::Ember);
    assert!(
        (available - 100.0).abs() < 0.01,
        "Should sum Ember buffer from EmberSpindle, got {available}"
    );
}

#[test]
fn test_available_resource_excludes_locked_extractors() {
    let mut loom = LoomState::new();
    quest::loom::discovery::complete_discovery(&mut loom);
    // SilenceWell is locked after discovery
    loom.persistent.nodes[NodeId::SilenceWell.index()].buffer = 999.0;
    let available = quest::loom::logic::available_resource(&loom, Resource::Silence);
    assert!(
        available < 0.01,
        "Locked extractor buffer should not count, got {available}"
    );
}

#[test]
fn test_available_resource_includes_active_shuttle_buffers() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    let mut shuttle = make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    );
    shuttle.buffer = 50.0;
    loom.persistent.shuttles.push(shuttle);

    let available = quest::loom::logic::available_resource(&loom, Resource::ForgedLight);
    assert!(
        (available - 50.0).abs() < 0.01,
        "Active shuttle buffer should be included, got {available}"
    );
}

#[test]
fn test_available_resource_excludes_under_construction_shuttles() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    let mut shuttle = make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    );
    shuttle.buffer = 50.0;
    shuttle.under_construction = true;
    loom.persistent.shuttles.push(shuttle);

    let available = quest::loom::logic::available_resource(&loom, Resource::ForgedLight);
    assert!(
        available < 0.01,
        "Under-construction shuttle buffer should not count, got {available}"
    );
}

// ── upgrade_shuttle error paths ──────────────────────────────────────────────

#[test]
fn test_upgrade_shuttle_invalid_index() {
    let mut loom = setup_loom();
    let result = upgrade_shuttle(&mut loom, 99, 7);
    assert!(matches!(result, Err(ShuttleUpgradeError::InvalidIndex)));
}

#[test]
fn test_upgrade_shuttle_under_construction() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    let mut shuttle = make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    );
    shuttle.under_construction = true;
    loom.persistent.shuttles.push(shuttle);

    let result = upgrade_shuttle(&mut loom, 0, 7);
    assert!(matches!(
        result,
        Err(ShuttleUpgradeError::UnderConstruction)
    ));
}

#[test]
fn test_upgrade_shuttle_insufficient_buffer() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    let mut shuttle = make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    );
    shuttle.buffer = 0.0; // Cost is 100 * 1^1.2 = 100
    loom.persistent.shuttles.push(shuttle);

    let result = upgrade_shuttle(&mut loom, 0, 7);
    assert!(matches!(
        result,
        Err(ShuttleUpgradeError::InsufficientBuffer { .. })
    ));
}

// ── tick_shuttle_construction ─────────────────────────────────────────────────

#[test]
fn test_shuttle_construction_completes() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    let mut shuttle = make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    );
    shuttle.under_construction = true;
    shuttle.construction_secs_remaining = 10.0;
    loom.persistent.shuttles.push(shuttle);

    let completed = tick_shuttle_construction(&mut loom, 10.0);
    assert_eq!(completed, vec![0], "Shuttle 0 should complete");
    assert!(!loom.persistent.shuttles[0].under_construction);
    assert_eq!(loom.persistent.shuttles[0].construction_secs_remaining, 0.0);
}

#[test]
fn test_shuttle_construction_partial_progress() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    let mut shuttle = make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    );
    shuttle.under_construction = true;
    shuttle.construction_secs_remaining = 100.0;
    loom.persistent.shuttles.push(shuttle);

    let completed = tick_shuttle_construction(&mut loom, 50.0);
    assert!(completed.is_empty(), "Should not complete yet");
    assert!(loom.persistent.shuttles[0].under_construction);
    assert!(
        (loom.persistent.shuttles[0].construction_secs_remaining - 50.0).abs() < 0.01,
        "Should have 50s remaining"
    );
}

#[test]
fn test_shuttle_construction_skips_operational() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    // One operational, one under construction
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));

    let mut s2 = make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    );
    s2.under_construction = true;
    s2.construction_secs_remaining = 5.0;
    loom.persistent.shuttles.push(s2);

    let completed = tick_shuttle_construction(&mut loom, 10.0);
    assert_eq!(completed, vec![1], "Only shuttle 1 should complete");
}

// ── node_upgrade_cost ────────────────────────────────────────────────────────

#[test]
fn test_node_upgrade_cost_scales_with_level() {
    let mut loom = setup_loom();

    // Level 1: cost = 100 * 1^1.2 = 100
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1);
    let cost1 = node_upgrade_cost(&loom, NodeId::EmberSpindle);
    assert!(
        (cost1 - 100.0).abs() < 1.0,
        "Level 1 cost should be ~100, got {cost1}"
    );

    // Level 5: cost = round(100 * 5^1.2) = 690
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 5);
    let cost5 = node_upgrade_cost(&loom, NodeId::EmberSpindle);
    assert!(cost5 > cost1, "Level 5 cost should exceed level 1 cost");
    assert!(
        (cost5 - 690.0).abs() < 1.0,
        "Level 5 cost should be 690, got {cost5}"
    );
}

// ── node_upgrade_duration ────────────────────────────────────────────────────

#[test]
fn test_node_upgrade_duration_linear() {
    // Level × 2h (7200s per level)
    assert!(
        (node_upgrade_duration(1) - 7200.0).abs() < 0.01,
        "L1→L2 = 2h"
    );
    assert!(
        (node_upgrade_duration(2) - 14400.0).abs() < 0.01,
        "L2→L3 = 4h"
    );
    assert!(
        (node_upgrade_duration(5) - 36000.0).abs() < 0.01,
        "L5→L6 = 10h"
    );
    assert!(
        (node_upgrade_duration(20) - 144000.0).abs() < 0.01,
        "L20 = 40h"
    );
}

// ── shuttle_construction_secs ────────────────────────────────────────────────

#[test]
fn test_shuttle_construction_secs_by_tier() {
    assert!(
        (shuttle_construction_secs(1) - 7200.0).abs() < 0.01,
        "T1 = 2h"
    );
    assert!(
        (shuttle_construction_secs(2) - 14400.0).abs() < 0.01,
        "T2 = 4h"
    );
    assert!(
        (shuttle_construction_secs(3) - 21600.0).abs() < 0.01,
        "T3 = 6h"
    );
    // Invalid tier defaults to T1
    assert!(
        (shuttle_construction_secs(99) - 7200.0).abs() < 0.01,
        "Invalid = T1 default"
    );
}

// ── node_neighbors ───────────────────────────────────────────────────────────

#[test]
fn test_node_neighbors_form_cycle() {
    // 6-node cycle: Ember → Reflection → Void → Memory → Silence → Resonance → Ember
    let all_nodes = [
        NodeId::EmberSpindle,
        NodeId::ReflectionLens,
        NodeId::VoidCondenser,
        NodeId::MemoryArchive,
        NodeId::SilenceWell,
        NodeId::ResonanceForge,
    ];

    for &node in &all_nodes {
        let neighbors = node_neighbors(node);
        assert_eq!(
            neighbors.len(),
            2,
            "Node {:?} should have exactly 2 neighbors",
            node
        );
        // Each neighbor should also list this node as its neighbor (symmetry)
        for &nb in neighbors {
            let nb_neighbors = node_neighbors(nb);
            assert!(
                nb_neighbors.contains(&node),
                "Neighbor {:?} of {:?} should list it back",
                nb,
                node
            );
        }
    }
}
