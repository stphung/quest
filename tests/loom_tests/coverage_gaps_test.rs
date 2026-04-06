//! Coverage gap tests for the Loom system.
//!
//! Tests verifying contention splitting, sharing, upgrades, milestones,
//! graph_dirty triggers, under-construction exclusion, simultaneous requirements,
//! and build error paths.

use quest::loom::discovery::complete_discovery;
use quest::loom::logic::{
    build_shuttle, demolish_shuttle, node_level_multiplier, tick_base_production,
    tick_node_upgrades, tick_shuttle_pull, try_upgrade_node, ShuttleError,
};
use quest::loom::types::*;
use std::collections::HashMap;

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Unlock all 6 extractor nodes.
fn unlock_all_nodes(loom: &mut LoomState) {
    for node in &mut loom.persistent.nodes {
        node.unlocked = true;
    }
}

/// Set an extractor to a specific level with correct buffer capacity.
fn set_extractor_level(loom: &mut LoomState, node_id: NodeId, level: u32) {
    let node = &mut loom.persistent.nodes[node_id.index()];
    node.level = level;
    node.unlocked = true;
    node.buffer_capacity = node.base_rate * node_level_multiplier(level) * 10.0;
    node.buffer = node.buffer_capacity;
}

/// Create a T1 shuttle (base + base -> confluence).
fn make_t1_shuttle(
    input_a: Resource,
    extractor_a: NodeId,
    input_b: Resource,
    extractor_b: NodeId,
    nature: NodeNature,
    output: Resource,
) -> Shuttle {
    let mut s = Shuttle::new(
        input_a,
        input_b,
        nature,
        output,
        1.0,
        1,
        vec![LoomNodeRef::Extractor(extractor_a)],
        vec![LoomNodeRef::Extractor(extractor_b)],
    );
    s.under_construction = false;
    s.construction_secs_remaining = 0.0;
    s
}

/// Create a T2 shuttle with explicit source references.
fn make_t2_shuttle(
    input_a: Resource,
    source_a: LoomNodeRef,
    input_b: Resource,
    source_b: LoomNodeRef,
    nature: NodeNature,
    output: Resource,
) -> Shuttle {
    let mut s = Shuttle::new(
        input_a,
        input_b,
        nature,
        output,
        1.0,
        2,
        vec![source_a],
        vec![source_b],
    );
    s.under_construction = false;
    s.construction_secs_remaining = 0.0;
    s
}

/// Create a T3 shuttle with explicit source references.
fn make_t3_shuttle(
    input_a: Resource,
    source_a: LoomNodeRef,
    input_b: Resource,
    source_b: LoomNodeRef,
    nature: NodeNature,
    output: Resource,
) -> Shuttle {
    let mut s = Shuttle::new(
        input_a,
        input_b,
        nature,
        output,
        1.0,
        3,
        vec![source_a],
        vec![source_b],
    );
    s.under_construction = false;
    s.construction_secs_remaining = 0.0;
    s
}

/// All 13 loom resources for zero-pushing.
const ALL_RESOURCES: [Resource; 13] = [
    Resource::Ember,
    Resource::Reflection,
    Resource::VoidEssence,
    Resource::Memory,
    Resource::Silence,
    Resource::Resonance,
    Resource::ForgedLight,
    Resource::EchoGlass,
    Resource::StillbornSong,
    Resource::CondensedEmber,
    Resource::EmberEcho,
    Resource::PurifiedVoid,
    Resource::WovenReality,
];

/// Run N ticks of production and push results to rate trackers.
fn run_ticks(loom: &mut LoomState, tick_count: usize) {
    let dt = 0.1; // 100ms tick
    for _ in 0..tick_count {
        let base_produced = tick_base_production(loom, dt);
        let shuttle_produced = tick_shuttle_pull(loom, dt);

        let mut produced: HashMap<Resource, f64> = base_produced;
        for (resource, amount) in shuttle_produced {
            *produced.entry(resource).or_insert(0.0) += amount;
        }

        for (resource, &amount) in &produced {
            loom.rate_trackers
                .entry(*resource)
                .or_default()
                .push(amount);
        }
        for resource in &ALL_RESOURCES {
            if !produced.contains_key(resource) {
                loom.rate_trackers.entry(*resource).or_default().push(0.0);
            }
        }
    }
}

/// Set up a LoomState with discovery complete and all nodes unlocked.
fn setup_loom() -> LoomState {
    let mut loom = LoomState::new();
    complete_discovery(&mut loom);
    unlock_all_nodes(&mut loom);
    loom
}

/// Mark the first N patterns as completed.
fn complete_n_patterns(loom: &mut LoomState, n: usize) {
    for i in 0..n.min(loom.persistent.patterns.len()) {
        loom.persistent.patterns[i].completed = true;
        for req in &mut loom.persistent.patterns[i].requirements {
            req.completed = true;
        }
    }
}

/// Assert a rate is within tolerance of an expected value.
fn assert_rate_approx(actual: f64, expected: f64, tolerance: f64, msg: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{}: expected ~{:.1}/hr, got {:.1}/hr (tolerance {:.1})",
        msg,
        expected,
        actual,
        tolerance,
    );
}

// ── 1. No intake cap contract ────────────────────────────────────────────────

#[test]
fn test_shuttle_output_exceeds_old_intake_cap() {
    // Build a ForgedLight T1 shuttle with L3 extractors (50/hr each).
    // Old cap was 20/hr. Verify output > 20/hr (should be 50/hr).
    // This locks in the cap removal as a permanent contract.
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1); // Need 1 pattern for T1

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3); // 25 * 2.0 = 50/hr
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 3); // 25 * 2.0 = 50/hr

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
        "ForgedLight output ({:.1}/hr) should exceed old 20/hr intake cap",
        rate
    );
    assert_rate_approx(rate, 50.0, 1.0, "ForgedLight T1 with L3 extractors");
}

// ── 2. Contention splitting — exact math ─────────────────────────────────────

#[test]
fn test_two_shuttles_sharing_extractor_split_evenly() {
    // Two ForgedLight T1 shuttles sharing Ember and VoidEssence.
    // At L1 (25/hr), each should get 12.5/hr → output 12.5/hr each.
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1); // 25/hr
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 1); // 25/hr

    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));

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
    // Three T1 shuttles sharing an extractor.
    // At L1 (25/hr), each gets 8.33/hr.
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
            &format!("Shuttle {} with 3-way split", i),
        );
    }
}

#[test]
fn test_demolish_consumer_restores_full_throughput() {
    // Two shuttles sharing, demolish one, verify remaining gets full rate.
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 1);

    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));

    // Run briefly to establish split rates
    run_ticks(&mut loom, 250);
    let rate_before = loom.persistent.shuttles[0]
        .output_rate_tracker
        .rate_per_hour();
    assert_rate_approx(rate_before, 12.5, 1.0, "Before demolish (split)");

    // Demolish shuttle 1
    demolish_shuttle(&mut loom, 1);

    // Reset rate tracker for shuttle 0 to measure fresh
    loom.persistent.shuttles[0].output_rate_tracker = RateTracker::new();

    run_ticks(&mut loom, 250);
    let rate_after = loom.persistent.shuttles[0]
        .output_rate_tracker
        .rate_per_hour();
    assert_rate_approx(rate_after, 25.0, 1.0, "After demolish (full rate)");
}

// ── 3. T1 sharing — one T1 feeding two higher-tier shuttles ──────────────────

#[test]
fn test_t1_shuttle_shared_between_t2_and_t3() {
    // ForgedLight T1 feeds both EmberEcho T2 and WovenReality T3.
    // At L3 Ember+Void (50/hr each), ForgedLight = 50/hr.
    // Each consumer (EE T2 and WR T3) gets 25/hr from ForgedLight.
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 15); // Unlock T1, T2, T3

    // L3 extractors for ForgedLight
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3); // 50/hr
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 3); // 50/hr
                                                              // Memory extractor for EmberEcho T2 (ForgedLight + Memory -> EmberEcho)
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 3); // 50/hr
                                                              // EchoGlass needs Memory + Silence
    set_extractor_level(&mut loom, NodeId::SilenceWell, 3); // 50/hr

    // Shuttle 0: ForgedLight T1 (Ember + VoidEssence -> ForgedLight)
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));

    // Shuttle 1: EchoGlass T1 (Memory + Silence -> EchoGlass)
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    ));

    // Shuttle 2: EmberEcho T2 (ForgedLight + Memory -> EmberEcho)
    // Sources: ForgedLight from Shuttle(0), Memory from MemoryArchive
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::ForgedLight,
        LoomNodeRef::Shuttle(0),
        Resource::Memory,
        LoomNodeRef::Extractor(NodeId::MemoryArchive),
        NodeNature::Form,
        Resource::EmberEcho,
    ));

    // Shuttle 3: WovenReality T3 (ForgedLight + EchoGlass -> WovenReality)
    // Sources: ForgedLight from Shuttle(0), EchoGlass from Shuttle(1)
    loom.persistent.shuttles.push(make_t3_shuttle(
        Resource::ForgedLight,
        LoomNodeRef::Shuttle(0),
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(1),
        NodeNature::Vibration,
        Resource::WovenReality,
    ));

    run_ticks(&mut loom, 250);

    // ForgedLight shuttle produces 50/hr (uncapped)
    let fl_rate = loom.persistent.shuttles[0]
        .output_rate_tracker
        .rate_per_hour();
    assert_rate_approx(fl_rate, 50.0, 1.0, "ForgedLight T1 output");

    // EmberEcho T2 and WovenReality T3 share ForgedLight (2 consumers).
    // Each gets 25/hr from ForgedLight.
    // EmberEcho: min(25 FL, 50 Memory * share) = 25/hr
    // Memory has 2 consumers (EmberEcho T2 + EchoGlass T1 via MemoryArchive),
    // but EmberEcho pulls from Shuttle(0) for FL and Extractor(MemoryArchive) for Memory.
    // MemoryArchive is shared by EchoGlass T1 (sources_a) and EmberEcho T2 (sources_b).
    let ee_rate = loom.persistent.shuttles[2]
        .output_rate_tracker
        .rate_per_hour();
    // EmberEcho limited by ForgedLight share (25/hr) since Memory share is also 25/hr
    assert_rate_approx(ee_rate, 25.0, 1.0, "EmberEcho T2 (shared ForgedLight)");

    // WovenReality: min(25 FL, EchoGlass output share)
    let wr_rate = loom.persistent.shuttles[3]
        .output_rate_tracker
        .rate_per_hour();
    // WovenReality limited by the lower of ForgedLight share (25) and EchoGlass output (25)
    assert_rate_approx(wr_rate, 25.0, 1.0, "WovenReality T3 (shared ForgedLight)");
}

// ── 4. Rate tracker cleared on upgrade ───────────────────────────────────────

#[test]
fn test_rate_tracker_cleared_on_upgrade_start() {
    // Fill a rate tracker with positive history.
    // Start upgrade on that extractor.
    // Verify rate_per_hour() returns 0.0 immediately.
    let mut loom = setup_loom();
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1);

    // Run ticks to build up rate tracker history
    run_ticks(&mut loom, 250);
    let rate_before = loom
        .rate_trackers
        .get(&Resource::Ember)
        .map(|t| t.rate_per_hour())
        .unwrap_or(0.0);
    assert!(
        rate_before > 10.0,
        "Rate should be positive before upgrade, was {}",
        rate_before
    );

    // Start upgrade (needs buffer >= 50% capacity)
    let started = try_upgrade_node(&mut loom, NodeId::EmberSpindle);
    assert!(started, "Upgrade should start");

    let rate_after = loom
        .rate_trackers
        .get(&Resource::Ember)
        .map(|t| t.rate_per_hour())
        .unwrap_or(0.0);
    assert!(
        rate_after.abs() < 0.01,
        "Rate should be 0.0 after upgrade start, was {}",
        rate_after
    );
}

#[test]
fn test_zero_production_during_upgrade() {
    // Set a node to upgrading.
    // Call tick_base_production.
    // Assert the upgrading node's resource is absent from produced map.
    let mut loom = setup_loom();
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1);

    // Manually set upgrading
    loom.persistent.nodes[NodeId::EmberSpindle.index()].upgrading = true;
    loom.persistent.nodes[NodeId::EmberSpindle.index()].upgrade_remaining_secs = 7200.0;

    let produced = tick_base_production(&mut loom, 0.1);
    let ember_produced = produced.get(&Resource::Ember).copied().unwrap_or(0.0);
    assert!(
        ember_produced.abs() < 1e-9,
        "Upgrading node should produce nothing, produced {}",
        ember_produced
    );
}

// ── 5. MAX_SHUTTLES milestone curve ──────────────────────────────────────────

#[test]
fn test_max_shuttles_milestone_curve() {
    // Verify exact slot counts at each milestone.
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
            "With {} patterns, expected {} slots but got {}",
            patterns, expected_slots, actual
        );
    }
}

#[test]
fn test_build_shuttle_at_capacity_returns_error() {
    // Fill all 5 slots, try to build one more, assert AtCapacity error.
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 15); // Unlock all tiers, max 5 slots

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 3);

    // Fill 5 slots manually
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

    assert_eq!(loom.persistent.shuttles.len(), 5);
    assert_eq!(loom.persistent.max_shuttles(), 5);

    // Try to build via the official API
    let result = build_shuttle(
        &mut loom,
        Resource::Ember,
        Resource::VoidEssence,
        NodeNature::Heat,
        vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
        vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
    );
    assert!(
        matches!(result, Err(ShuttleError::AtCapacity)),
        "Expected AtCapacity error, got {:?}",
        result.as_ref().err()
    );
}

// ── 6. graph_dirty triggers ──────────────────────────────────────────────────

#[test]
fn test_graph_dirty_set_on_build_shuttle() {
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 3);

    assert!(!loom.graph_dirty, "graph_dirty should be false initially");

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

    // Fill buffer so upgrade can start
    loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer =
        loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer_capacity;
    loom.graph_dirty = false;

    let started = try_upgrade_node(&mut loom, NodeId::EmberSpindle);
    assert!(started, "Upgrade should start");
    assert!(
        loom.graph_dirty,
        "graph_dirty should be true after upgrade start"
    );
}

#[test]
fn test_graph_dirty_set_on_upgrade_completion() {
    let mut loom = setup_loom();
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1);

    // Manually set upgrading with very short remaining time
    loom.persistent.nodes[NodeId::EmberSpindle.index()].upgrading = true;
    loom.persistent.nodes[NodeId::EmberSpindle.index()].upgrade_remaining_secs = 0.05;
    loom.graph_dirty = false;

    // Tick enough to complete the upgrade
    tick_node_upgrades(&mut loom, 0.1);

    assert!(
        !loom.persistent.nodes[NodeId::EmberSpindle.index()].upgrading,
        "Upgrade should be complete"
    );
    assert!(
        loom.graph_dirty,
        "graph_dirty should be true after upgrade completion"
    );
}

// ── 7. Under-construction excluded from consumer count ───────────────────────

#[test]
fn test_under_construction_shuttle_excluded_from_contention() {
    // One operational shuttle + one under-construction shuttle sharing Ember.
    // Operational shuttle should get full Ember rate, not half.
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 1);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 1); // 25/hr
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 1); // 25/hr

    // Shuttle 0: operational
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));

    // Shuttle 1: under construction (same sources)
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
    // Operational shuttle should get full rate (25/hr), not half
    assert_rate_approx(
        rate,
        25.0,
        1.0,
        "Operational shuttle with under-construction neighbor",
    );
}

// ── 8. Simultaneous requirements — mid-progress regression ───────────────────

#[test]
fn test_simultaneous_requirements_no_advance_when_one_drops() {
    // Two requirements, both at 50% progress.
    // Supply rate for only one. Assert neither advances.
    let mut loom = setup_loom();

    // Manually create a pattern with two requirements
    loom.persistent.patterns[0] = WovenPattern {
        index: 0,
        name: "Test Pattern".to_string(),
        requirements: vec![
            PatternRequirement {
                resource: Resource::Ember,
                required_rate: 25.0,
                sustain_duration_secs: 100.0,
                sustained_secs: 50.0,
                completed: false,
                amount: 0.0,
                accumulated: 0.0,
            },
            PatternRequirement {
                resource: Resource::VoidEssence,
                required_rate: 25.0,
                sustain_duration_secs: 100.0,
                sustained_secs: 50.0,
                completed: false,
                amount: 0.0,
                accumulated: 0.0,
            },
        ],
        completed: false,
        flavor: String::new(),
    };
    loom.persistent.active_pattern = 0;

    // Supply only Ember at sufficient rate, VoidEssence at 0
    let mut rates = HashMap::new();
    rates.insert(Resource::Ember, 30.0); // Above 25/hr threshold
    rates.insert(Resource::VoidEssence, 0.0); // Below threshold

    quest::loom::patterns::tick_pattern_sustain(&mut loom.persistent, &rates, 1.0);

    // Neither should advance because not ALL requirements are met simultaneously
    let req_ember = &loom.persistent.patterns[0].requirements[0];
    let req_void = &loom.persistent.patterns[0].requirements[1];
    assert!(
        (req_ember.sustained_secs - 50.0).abs() < 0.01,
        "Ember requirement should not advance (was {}, expected 50.0)",
        req_ember.sustained_secs
    );
    assert!(
        (req_void.sustained_secs - 50.0).abs() < 0.01,
        "VoidEssence requirement should not advance (was {}, expected 50.0)",
        req_void.sustained_secs
    );
}

// ── 9. Build shuttle error paths ─────────────────────────────────────────────

#[test]
fn test_build_shuttle_tier_locked_error() {
    // 0 completed patterns, try T1 (needs 1). Assert TierLocked.
    let mut loom = setup_loom();
    // 0 patterns completed — T1 needs 1

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
    assert!(
        matches!(result, Err(ShuttleError::TierLocked)),
        "Expected TierLocked error with 0 patterns, got {:?}",
        result.as_ref().err()
    );
}

#[test]
fn test_build_shuttle_t2_locked_before_8_patterns() {
    // 7 completed patterns, try T2 (needs 8). Assert TierLocked.
    let mut loom = setup_loom();
    complete_n_patterns(&mut loom, 7);

    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 3);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 3);
    set_extractor_level(&mut loom, NodeId::ResonanceForge, 3);

    // First build a T1 shuttle to produce StillbornSong (needed as T2 input)
    // T2 recipe: StillbornSong + Ember -> CondensedEmber (tier 2, nature Heat)
    // But we can just try to build directly and check the tier gate
    // Use ForgedLight + Memory -> EmberEcho (T2, nature Form)
    let result = build_shuttle(
        &mut loom,
        Resource::ForgedLight,
        Resource::Memory,
        NodeNature::Form,
        vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)], // Invalid source but tier check comes first
        vec![LoomNodeRef::Extractor(NodeId::MemoryArchive)],
    );
    assert!(
        matches!(result, Err(ShuttleError::TierLocked)),
        "Expected TierLocked error with 7 patterns for T2, got {:?}",
        result.as_ref().err()
    );
}
