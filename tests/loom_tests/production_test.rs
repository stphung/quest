//! Production chain tests: base production, contention splitting, rate tracking,
//! stall detection, upgrades, and multi-tier pipeline verification.

use super::helpers::*;
use quest::loom::logic::{
    node_effective_rate, tick_base_production, tick_neighbor_unlocking, tick_node_upgrades,
    tick_shuttle_pull, tick_shuttle_stall_detection, try_upgrade_node, MAX_NODE_LEVEL,
};
use quest::loom::types::*;

// ── Base production ──────────────────────────────────────────────────────────

#[test]
fn test_zero_production_during_upgrade() {
    let mut loom = setup_loom();
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1);

    loom.persistent.nodes[NodeId::EmberSpindle.index()].upgrading = true;
    loom.persistent.nodes[NodeId::EmberSpindle.index()].upgrade_remaining_secs = 7200.0;

    let produced = tick_base_production(&mut loom, 0.1);
    let ember_produced = produced.get(&Resource::Ember).copied().unwrap_or(0.0);
    assert!(
        ember_produced.abs() < 1e-9,
        "Upgrading node should produce nothing, produced {ember_produced}"
    );
}

#[test]
fn test_base_production_negative_delta_produces_nothing() {
    let mut loom = setup_loom();
    let initial_buffer = loom.persistent.nodes[0].buffer;
    let produced = tick_base_production(&mut loom, -1.0);
    assert!(produced.is_empty(), "Negative delta should produce nothing");
    assert!(
        (loom.persistent.nodes[0].buffer - initial_buffer).abs() < 0.001,
        "Buffer should not change with negative delta"
    );
}

#[test]
fn test_base_production_zero_delta_produces_nothing() {
    let mut loom = setup_loom();
    let produced = tick_base_production(&mut loom, 0.0);
    assert!(produced.is_empty(), "Zero delta should produce nothing");
}

// ── No intake cap contract ───────────────────────────────────────────────────

#[test]
fn test_shuttle_output_exceeds_old_intake_cap() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3); // 50/hr
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 3); // 50/hr

    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));

    run_ticks(&mut loom, 250);

    let rate = loom.persistent.shuttles[0]
        .output_rate_tracker
        .rate_per_hour();
    assert!(
        rate > 20.0,
        "ForgedLight output ({rate:.1}/hr) should exceed old 20/hr intake cap",
    );
    assert_rate_approx(rate, 50.0, 1.0, "ForgedLight T1 with L3 extractors");
}

// ── Contention splitting ─────────────────────────────────────────────────────

#[test]
fn test_two_shuttles_sharing_extractor_split_evenly() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 1);

    for _ in 0..2 {
        loom.persistent.shuttles.push(make_t1_shuttle(
            Resource::Ember,
            NodeId::EmberSpindle,
            Resource::VoidEssence,
            NodeId::VoidCondenser,
            NodeNature::Heat,
            Resource::ForgedLight,
        ));
    }

    run_ticks(&mut loom, 250);

    let rate_0 = loom.persistent.shuttles[0]
        .output_rate_tracker
        .rate_per_hour();
    let rate_1 = loom.persistent.shuttles[1]
        .output_rate_tracker
        .rate_per_hour();
    assert_rate_approx(rate_0, 12.5, 1.0, "Shuttle 0 with 2-way split");
    assert_rate_approx(rate_1, 12.5, 1.0, "Shuttle 1 with 2-way split");
}

#[test]
fn test_three_shuttles_sharing_extractor_split_three_ways() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 1);

    for _ in 0..3 {
        loom.persistent.shuttles.push(make_t1_shuttle(
            Resource::Ember,
            NodeId::EmberSpindle,
            Resource::VoidEssence,
            NodeId::VoidCondenser,
            NodeNature::Heat,
            Resource::ForgedLight,
        ));
    }

    run_ticks(&mut loom, 250);

    for i in 0..3 {
        let rate = loom.persistent.shuttles[i]
            .output_rate_tracker
            .rate_per_hour();
        assert_rate_approx(
            rate,
            25.0 / 3.0,
            1.0,
            &format!("Shuttle {i} with 3-way split"),
        );
    }
}

#[test]
fn test_under_construction_shuttle_excluded_from_contention() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 1);

    // Shuttle 0: operational
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));

    // Shuttle 1: under construction
    let mut s = make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    );
    s.under_construction = true;
    s.construction_secs_remaining = 50.0;
    loom.persistent.shuttles.push(s);

    run_ticks(&mut loom, 250);

    let rate = loom.persistent.shuttles[0]
        .output_rate_tracker
        .rate_per_hour();
    assert_rate_approx(
        rate,
        25.0,
        1.0,
        "Operational shuttle with under-construction neighbor",
    );
}

// ── Multi-tier pipeline ──────────────────────────────────────────────────────

#[test]
fn test_t1_shuttle_shared_between_t2_and_t3() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 15);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 3);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 3);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 3);

    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    ));
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::ForgedLight,
        LoomNodeRef::Shuttle(0),
        Resource::Memory,
        LoomNodeRef::Extractor(NodeId::MemoryArchive),
        NodeNature::Form,
        Resource::EmberEcho,
    ));
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(1),
        Resource::VoidEssence,
        LoomNodeRef::Extractor(NodeId::VoidCondenser),
        NodeNature::Void,
        Resource::PurifiedVoid,
    ));
    loom.persistent.shuttles.push(make_t3_shuttle(
        Resource::PurifiedVoid,
        LoomNodeRef::Shuttle(3),
        Resource::EmberEcho,
        LoomNodeRef::Shuttle(2),
        NodeNature::Vibration,
        Resource::WovenReality,
    ));

    run_ticks(&mut loom, 250);

    let fl_rate = loom.persistent.shuttles[0]
        .output_rate_tracker
        .rate_per_hour();
    assert_rate_approx(fl_rate, 25.0, 1.0, "ForgedLight T1 output (shared Void)");

    let wr_rate = loom.persistent.shuttles[4]
        .output_rate_tracker
        .rate_per_hour();
    assert_rate_approx(wr_rate, 25.0, 1.0, "WovenReality T3 output");
}

#[test]
fn test_shuttle_with_empty_sources_produces_zero() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    let mut shuttle = Shuttle::new(
        Resource::Ember,
        Resource::VoidEssence,
        NodeNature::Heat,
        Resource::ForgedLight,
        1.0,
        1,
        vec![],
        vec![],
    );
    shuttle.under_construction = false;
    shuttle.construction_secs_remaining = 0.0;
    loom.persistent.shuttles.push(shuttle);

    let produced = tick_shuttle_pull(&mut loom, 0.1);
    let forged_light = produced.get(&Resource::ForgedLight).copied().unwrap_or(0.0);
    assert!(
        forged_light.abs() < 0.001,
        "Shuttle with empty sources should produce zero, got {forged_light}"
    );
}

#[test]
fn test_shuttle_pulls_zero_from_upgrading_extractor() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 2);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 2);

    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));

    run_ticks(&mut loom, 100);
    let rate_before = loom.persistent.shuttles[0]
        .output_rate_tracker
        .rate_per_hour();
    assert!(rate_before > 0.0, "Shuttle should produce before upgrade");

    loom.persistent.nodes[NodeId::EmberSpindle.index()].upgrading = true;
    loom.persistent.shuttles[0].output_rate_tracker = RateTracker::new();

    run_ticks(&mut loom, 100);
    let rate_after = loom.persistent.shuttles[0]
        .output_rate_tracker
        .rate_per_hour();
    assert!(
        rate_after < 0.01,
        "Shuttle should produce ~0 while Ember is upgrading, got {rate_after}"
    );
}

// ── Stall detection ──────────────────────────────────────────────────────────

#[test]
fn test_shuttle_stall_marks_when_buffer_full() {
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
    shuttle.buffer = shuttle.buffer_capacity;
    loom.persistent.shuttles.push(shuttle);

    tick_shuttle_stall_detection(&mut loom);
    assert!(
        loom.persistent.shuttles[0].stalled,
        "Shuttle should be stalled when buffer is full"
    );
}

#[test]
fn test_shuttle_stall_clears_when_buffer_drains() {
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
    shuttle.buffer = shuttle.buffer_capacity;
    shuttle.stalled = true;
    loom.persistent.shuttles.push(shuttle);

    loom.persistent.shuttles[0].buffer = 0.0;
    tick_shuttle_stall_detection(&mut loom);
    assert!(
        !loom.persistent.shuttles[0].stalled,
        "Shuttle should unstall when buffer is drained"
    );
}

// ── Rate tracker and upgrades ────────────────────────────────────────────────

#[test]
fn test_rate_tracker_cleared_on_upgrade_start() {
    let mut loom = setup_loom();
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1);

    run_ticks(&mut loom, 250);
    let rate_before = loom
        .rate_trackers
        .get(&Resource::Ember)
        .map(|t| t.rate_per_hour())
        .unwrap_or(0.0);
    assert!(rate_before > 10.0, "Rate should be positive before upgrade");

    let started = try_upgrade_node(&mut loom, NodeId::EmberSpindle);
    assert!(started, "Upgrade should start");

    let rate_after = loom
        .rate_trackers
        .get(&Resource::Ember)
        .map(|t| t.rate_per_hour())
        .unwrap_or(0.0);
    assert!(
        rate_after.abs() < 0.01,
        "Rate should be 0.0 after upgrade start, was {rate_after}"
    );
}

#[test]
fn test_node_upgrade_capped_at_max_level() {
    let mut loom = setup_loom();
    let node = &mut loom.persistent.nodes[NodeId::EmberSpindle.index()];
    node.level = MAX_NODE_LEVEL - 1;
    node.upgrading = true;
    node.upgrade_remaining_secs = 0.01;

    tick_node_upgrades(&mut loom, 1.0);

    let node = &loom.persistent.nodes[NodeId::EmberSpindle.index()];
    assert_eq!(
        node.level, MAX_NODE_LEVEL,
        "Should upgrade to MAX_NODE_LEVEL"
    );
    assert!(!node.upgrading, "Should no longer be upgrading");
}

#[test]
fn test_try_upgrade_node_rejects_at_max_level() {
    let mut loom = setup_loom();
    set_extractor_level(&mut loom, NodeId::EmberSpindle, MAX_NODE_LEVEL);

    let result = try_upgrade_node(&mut loom, NodeId::EmberSpindle);
    assert!(!result, "Should reject upgrade at MAX_NODE_LEVEL");
}

#[test]
fn test_demolish_consumer_restores_full_throughput() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 1);

    for _ in 0..2 {
        loom.persistent.shuttles.push(make_t1_shuttle(
            Resource::Ember,
            NodeId::EmberSpindle,
            Resource::VoidEssence,
            NodeId::VoidCondenser,
            NodeNature::Heat,
            Resource::ForgedLight,
        ));
    }

    run_ticks(&mut loom, 250);
    let rate_before = loom.persistent.shuttles[0]
        .output_rate_tracker
        .rate_per_hour();
    assert_rate_approx(rate_before, 12.5, 1.0, "Before demolish (split)");

    quest::loom::logic::demolish_shuttle(&mut loom, 1);
    loom.persistent.shuttles[0].output_rate_tracker = RateTracker::new();

    run_ticks(&mut loom, 250);
    let rate_after = loom.persistent.shuttles[0]
        .output_rate_tracker
        .rate_per_hour();
    assert_rate_approx(rate_after, 25.0, 1.0, "After demolish (full rate)");
}

// ── node_effective_rate ──────────────────────────────────────────────────────

#[test]
fn test_effective_rate_locked_node_is_zero() {
    let loom = LoomState::new(); // All nodes start locked
    let node = &loom.persistent.nodes[NodeId::EmberSpindle.index()];
    assert!(!node.unlocked);
    assert_eq!(node_effective_rate(&loom, node), 0.0);
}

#[test]
fn test_effective_rate_upgrading_node_is_zero() {
    let mut loom = setup_loom();
    loom.persistent.nodes[NodeId::EmberSpindle.index()].upgrading = true;
    let node = &loom.persistent.nodes[NodeId::EmberSpindle.index()];
    assert_eq!(node_effective_rate(&loom, node), 0.0);
}

#[test]
fn test_effective_rate_unlocked_level1() {
    let loom = setup_loom();
    let node = &loom.persistent.nodes[NodeId::EmberSpindle.index()];
    // base_rate=25, level=1, multiplier=1.0, throughput=1.0
    assert!((node_effective_rate(&loom, node) - 25.0).abs() < 0.01);
}

#[test]
fn test_effective_rate_unlocked_level3() {
    let mut loom = setup_loom();
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3);
    let node = &loom.persistent.nodes[NodeId::EmberSpindle.index()];
    // base_rate=25, level=3, multiplier=2.0 -> 50.0
    assert!((node_effective_rate(&loom, node) - 50.0).abs() < 0.01);
}

// ── tick_neighbor_unlocking ──────────────────────────────────────────────────

#[test]
fn test_neighbor_unlock_no_progress_below_threshold() {
    let mut loom = LoomState::new();
    quest::loom::discovery::complete_discovery(&mut loom);
    // Only EmberSpindle is unlocked initially. Set buffer to 0 (below 50% threshold).
    loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer = 0.0;

    let unlocked = tick_neighbor_unlocking(&mut loom, 3600.0); // 1 hour
    assert!(
        unlocked.is_empty(),
        "Should not unlock neighbors when buffer < 50%"
    );
}

#[test]
fn test_neighbor_unlock_progresses_above_threshold() {
    let mut loom = LoomState::new();
    quest::loom::discovery::complete_discovery(&mut loom);
    // Fill EmberSpindle buffer above 50%
    let node = &mut loom.persistent.nodes[NodeId::EmberSpindle.index()];
    node.buffer = node.buffer_capacity; // 100%

    // 1 hour of delta should give 1.0 hours of progress (need 2.0 to unlock)
    let unlocked = tick_neighbor_unlocking(&mut loom, 3600.0);
    assert!(unlocked.is_empty(), "Should not unlock after only 1 hour");

    // Check that neighbor has progress
    let neighbors = quest::loom::logic::node_neighbors(NodeId::EmberSpindle);
    let neighbor_node = loom
        .persistent
        .nodes
        .iter()
        .find(|n| n.id == neighbors[0])
        .unwrap();
    assert!(
        neighbor_node.unlock_progress > 0.9,
        "Neighbor should have ~1h progress, got {}",
        neighbor_node.unlock_progress
    );
}

#[test]
fn test_neighbor_unlock_completes_at_2_hours() {
    let mut loom = LoomState::new();
    quest::loom::discovery::complete_discovery(&mut loom);
    let node = &mut loom.persistent.nodes[NodeId::EmberSpindle.index()];
    node.buffer = node.buffer_capacity;

    // 2 hours should complete the unlock. EmberSpindle has 2 neighbors and
    // unlock_count=2, so both neighbors unlock simultaneously.
    let unlocked = tick_neighbor_unlocking(&mut loom, 7200.0);
    assert!(
        !unlocked.is_empty(),
        "Should unlock at least 1 neighbor after 2 hours"
    );

    // All returned neighbors should be unlocked with reset progress
    for &id in &unlocked {
        let neighbor = loom.persistent.nodes.iter().find(|n| n.id == id).unwrap();
        assert!(neighbor.unlocked, "Neighbor {:?} should be unlocked", id);
        assert_eq!(
            neighbor.unlock_progress, 0.0,
            "Progress should reset after unlock"
        );
    }
}

#[test]
fn test_neighbor_unlock_returns_newly_unlocked_ids() {
    let mut loom = LoomState::new();
    quest::loom::discovery::complete_discovery(&mut loom);
    let node = &mut loom.persistent.nodes[NodeId::EmberSpindle.index()];
    node.buffer = node.buffer_capacity;

    // 4 hours — enough for 2 neighbors to unlock
    let unlocked = tick_neighbor_unlocking(&mut loom, 14400.0);
    assert!(
        !unlocked.is_empty(),
        "Should unlock at least 1 neighbor after 4 hours"
    );
    // Verify returned IDs are actually unlocked now
    for id in &unlocked {
        let node = loom.persistent.nodes.iter().find(|n| n.id == *id).unwrap();
        assert!(node.unlocked, "Returned node {:?} should be unlocked", id);
    }
}

#[test]
fn test_neighbor_unlock_all_unlocked_returns_empty() {
    let mut loom = setup_loom(); // All nodes already unlocked
    let node = &mut loom.persistent.nodes[NodeId::EmberSpindle.index()];
    node.buffer = node.buffer_capacity;

    let unlocked = tick_neighbor_unlocking(&mut loom, 7200.0);
    assert!(
        unlocked.is_empty(),
        "No neighbors to unlock when all are already unlocked"
    );
}
