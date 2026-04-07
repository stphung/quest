//! Shared helpers for Loom integration tests.

use quest::loom::discovery::complete_discovery;
use quest::loom::logic::{node_level_multiplier, tick_base_production, tick_shuttle_pull};
use quest::loom::types::*;
use std::collections::HashMap;

/// Unlock all 6 extractor nodes.
pub fn unlock_all_nodes(loom: &mut LoomState) {
    for node in &mut loom.persistent.nodes {
        node.unlocked = true;
    }
}

/// Set an extractor to a specific level with correct buffer capacity.
pub fn set_extractor_level(loom: &mut LoomState, node_id: NodeId, level: u32) {
    let node = &mut loom.persistent.nodes[node_id.index()];
    node.level = level;
    node.unlocked = true;
    node.buffer_capacity = node.base_rate * node_level_multiplier(level) * 10.0;
    node.buffer = node.buffer_capacity;
}

/// Create a T1 shuttle (base + base -> confluence).
pub fn make_t1_shuttle(
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
pub fn make_t2_shuttle(
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
pub fn make_t3_shuttle(
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
pub const ALL_RESOURCES: [Resource; 13] = [
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
pub fn run_ticks(loom: &mut LoomState, tick_count: usize) {
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
pub fn setup_loom() -> LoomState {
    let mut loom = LoomState::new();
    complete_discovery(&mut loom);
    unlock_all_nodes(&mut loom);
    loom
}

/// Mark the first N non-eternal patterns as completed.
pub fn complete_n_patterns(loom: &mut LoomState, n: usize) {
    let mut completed = 0;
    for i in 0..loom.persistent.patterns.len() {
        if completed >= n {
            break;
        }
        if loom.persistent.patterns[i].eternal {
            continue;
        }
        loom.persistent.patterns[i].completed = true;
        for req in &mut loom.persistent.patterns[i].requirements {
            req.completed = true;
        }
        completed += 1;
    }
}

/// Assert a rate is within tolerance of an expected value.
pub fn assert_rate_approx(actual: f64, expected: f64, tolerance: f64, msg: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{}: expected ~{:.1}/hr, got {:.1}/hr (tolerance {:.1})",
        msg,
        expected,
        actual,
        tolerance,
    );
}
