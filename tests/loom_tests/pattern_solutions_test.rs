//! Integration tests verifying that each of the 28 Woven Patterns is solvable
//! with a specific shuttle configuration and extractor level setup.
//!
//! Each test:
//! 1. Sets up extractor levels and shuttles for a known solution
//! 2. Runs 250 ticks to stabilize rolling-window rate trackers
//! 3. Verifies all pattern requirements are met within 5% tolerance

use quest::loom::discovery::complete_discovery;
use quest::loom::logic::{node_level_multiplier, tick_base_production, tick_shuttle_pull};
use quest::loom::types::*;
use std::collections::HashMap;

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Unlock all 6 extractor nodes (complete_discovery only unlocks EmberSpindle).
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
    // Buffer capacity = base_rate * level_multiplier * BUFFER_HOURS (10)
    node.buffer_capacity = node.base_rate * node_level_multiplier(level) * 10.0;
    // Start full so shuttles can pull immediately
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

/// Run 250 ticks of production and push results to rate trackers (mirroring tick_stages.rs).
fn run_ticks(loom: &mut LoomState, tick_count: usize) {
    let dt = 0.1; // 100ms tick
    for _ in 0..tick_count {
        let base_produced = tick_base_production(loom, dt);
        let shuttle_produced = tick_shuttle_pull(loom, dt);

        // Merge produced maps
        let mut produced: HashMap<Resource, f64> = base_produced;
        for (resource, amount) in shuttle_produced {
            *produced.entry(resource).or_insert(0.0) += amount;
        }

        // Push to rate trackers (same as tick_stages.rs)
        for (resource, &amount) in &produced {
            loom.rate_trackers
                .entry(*resource)
                .or_default()
                .push(amount);
        }
        // Push 0.0 for non-produced resources
        for resource in &ALL_RESOURCES {
            if !produced.contains_key(resource) {
                loom.rate_trackers.entry(*resource).or_default().push(0.0);
            }
        }
    }
}

/// Read rates from trackers and verify all pattern requirements are met.
fn verify_pattern(loom: &LoomState, pattern_index: usize) {
    let rates: HashMap<Resource, f64> = loom
        .rate_trackers
        .iter()
        .map(|(r, t)| (*r, t.rate_per_hour()))
        .collect();

    let pattern = &loom.persistent.patterns[pattern_index];

    // Verify shuttle count within limit
    assert!(
        loom.persistent.shuttles.len() <= loom.persistent.max_shuttles(),
        "Pattern {} '{}' uses {} shuttles but only {} slots available",
        pattern_index,
        pattern.name,
        loom.persistent.shuttles.len(),
        loom.persistent.max_shuttles()
    );

    for req in &pattern.requirements {
        let actual = rates.get(&req.resource).copied().unwrap_or(0.0);
        assert!(
            actual >= req.required_rate * 0.95, // 5% tolerance for rolling window
            "Pattern {} '{}' requirement {:?} at {:.1}/hr not met (got {:.1}/hr)",
            pattern_index,
            pattern.name,
            req.resource,
            req.required_rate,
            actual
        );
    }
}

/// Common setup: discover loom, unlock all nodes, mark previous patterns complete.
fn setup_loom(pattern_index: usize) -> LoomState {
    let mut loom = LoomState::new();
    complete_discovery(&mut loom);
    unlock_all_nodes(&mut loom);

    // Mark previous patterns as complete (for shuttle slot unlocks)
    for i in 0..pattern_index {
        loom.persistent.patterns[i].completed = true;
    }
    loom.persistent.active_pattern = pattern_index;

    loom
}

// ── Chapter I: The Awakening (Patterns 0-7) ─────────────────────────────────

#[test]
fn test_pattern_0_first_thread_is_solvable() {
    let mut loom = setup_loom(0);
    // No shuttles needed. Ember L1 = 25/hr >= 25.
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 0);
}

#[test]
fn test_pattern_1_still_waters_is_solvable() {
    let mut loom = setup_loom(1);
    // No shuttles needed. Silence L1 = 25/hr >= 25.
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 1);
}

#[test]
fn test_pattern_2_echoing_halls_is_solvable() {
    let mut loom = setup_loom(2);
    // No shuttles needed. Memory L1 = 25/hr >= 25.
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 2);
}

#[test]
fn test_pattern_3_harmonic_pulse_is_solvable() {
    let mut loom = setup_loom(3);
    // No shuttles needed. Resonance L1 = 25/hr >= 25.
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 3);
}

#[test]
fn test_pattern_4_mirror_and_void_is_solvable() {
    let mut loom = setup_loom(4);
    // No shuttles. Reflection L2 = 37.5 >= 35, VoidEssence L2 = 37.5 >= 35.
    set_extractor_level(&mut loom, NodeId::ReflectionLens, 2);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 2);
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 4);
}

#[test]
fn test_pattern_5_full_circle_is_solvable() {
    let mut loom = setup_loom(5);
    // No shuttles. All extractors L2 = 37.5 >= 30.
    for &node_id in &NodeId::ALL {
        set_extractor_level(&mut loom, node_id, 2);
    }
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 5);
}

#[test]
fn test_pattern_6_the_catalyst_is_solvable() {
    let mut loom = setup_loom(6);
    // ForgedLight T1: Ember + VoidEssence -> ForgedLight.
    // Ember L2 = 37.5, VoidEssence L2 = 37.5. FL = min(37.5, 37.5) = 37.5 >= 22.
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
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 6);
}

#[test]
fn test_pattern_7_echo_of_flame_is_solvable() {
    let mut loom = setup_loom(7);
    // EchoGlass T1 + StillbornSong T1.
    // Silence L3 = 50 (shared by EG and SS -> contention: 50/2 = 25 each).
    // Memory L2 = 37.5. Resonance L2 = 37.5.
    // EG = min(37.5, 25) = 25 >= 20. SS = min(25, 37.5) = 25 >= 18.
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 2);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 3);
    set_extractor_level(&mut loom, NodeId::ResonanceForge, 2);
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    ));
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Silence,
        NodeId::SilenceWell,
        Resource::Resonance,
        NodeId::ResonanceForge,
        NodeNature::Stillness,
        Resource::StillbornSong,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 7);
}

// ── Chapter II: The Deepening (Patterns 8-15) ───────────────────────────────

#[test]
fn test_pattern_8_forged_in_fire_is_solvable() {
    let mut loom = setup_loom(8);
    // ForgedLight T1. Ember L2 = 37.5, VoidEssence L2 = 37.5. FL = 37.5 >= 35.
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
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 8);
}

#[test]
fn test_pattern_9_glass_resonance_is_solvable() {
    let mut loom = setup_loom(9);
    // ForgedLight T1 + EchoGlass T1.
    // Ember L2 = 37.5, VoidEssence L2 = 37.5 (no contention). FL = 37.5 >= 25.
    // Memory L2 = 37.5, Silence L3 = 50 (no contention on Silence). EG = min(37.5, 50) = 37.5 >= 30.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 2);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 2);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 2);
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
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 9);
}

#[test]
fn test_pattern_10_the_unsung_is_solvable() {
    let mut loom = setup_loom(10);
    // StillbornSong T1. Silence L3 = 50, Resonance L3 = 50. SS = min(50, 50) = 50 >= 35.
    // Also needs Resonance base at 50 >= 50.
    set_extractor_level(&mut loom, NodeId::SilenceWell, 3);
    set_extractor_level(&mut loom, NodeId::ResonanceForge, 3);
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Silence,
        NodeId::SilenceWell,
        Resource::Resonance,
        NodeId::ResonanceForge,
        NodeNature::Stillness,
        Resource::StillbornSong,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 10);
}

#[test]
fn test_pattern_11_void_distillation_is_solvable() {
    let mut loom = setup_loom(11);
    // EchoGlass T1 + PurifiedVoid T2.
    // Memory L2 = 37.5, Silence L3 = 50. EG = min(37.5, 50) = 37.5.
    // PV T2: EchoGlass(shuttle 0) + VoidEssence. PV = min(37.5, VoidL2=37.5) = 37.5 >= 20.
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 2);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 3);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 2);
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    ));
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(0),
        Resource::VoidEssence,
        LoomNodeRef::Extractor(NodeId::VoidCondenser),
        NodeNature::Void,
        Resource::PurifiedVoid,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 11);
}

#[test]
fn test_pattern_12_crossed_streams_is_solvable() {
    let mut loom = setup_loom(12);
    // ForgedLight T1 + EchoGlass T1 + PurifiedVoid T2.
    // VoidEssence L4 = 62.5, shared by FL and PV -> 62.5/2 = 31.25 each.
    // FL = min(Ember L2=37.5, 31.25) = 31.25 >= 30.
    // EG = min(Memory L2=37.5, Silence L3=50) = 37.5.
    // PV = min(EG shuttle=37.5, 31.25) = 31.25 >= 18.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 2);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 4);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 2);
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
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(1),
        Resource::VoidEssence,
        LoomNodeRef::Extractor(NodeId::VoidCondenser),
        NodeNature::Void,
        Resource::PurifiedVoid,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 12);
}

#[test]
fn test_pattern_13_the_asymmetry_is_solvable() {
    let mut loom = setup_loom(13);
    // ForgedLight T1 + StillbornSong T1 + CondensedEmber T2 + EmberEcho T2.
    // Ember L3 = 50, shared by FL and CE T2 -> but FL uses Ember extractor,
    // and CE T2 uses SS shuttle + Ember extractor. So Ember has 2 consumers: FL + CE.
    // Ember 50/2 = 25 each.
    // FL = min(25, Void L4=62.5) = 25.
    // SS = min(Silence L3=50, Resonance L3=50) = 50.
    // CE T2 = min(SS=50, Ember=25) = 25 >= 20.
    // EE T2 = min(FL shuttle=25, Memory L2=37.5) = 25 >= 18.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 4);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 3);
    set_extractor_level(&mut loom, NodeId::ResonanceForge, 3);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 2);
    // Shuttle 0: ForgedLight T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    // Shuttle 1: StillbornSong T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Silence,
        NodeId::SilenceWell,
        Resource::Resonance,
        NodeId::ResonanceForge,
        NodeNature::Stillness,
        Resource::StillbornSong,
    ));
    // Shuttle 2: CondensedEmber T2 (SS shuttle + Ember extractor)
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::StillbornSong,
        LoomNodeRef::Shuttle(1),
        Resource::Ember,
        LoomNodeRef::Extractor(NodeId::EmberSpindle),
        NodeNature::Heat,
        Resource::CondensedEmber,
    ));
    // Shuttle 3: EmberEcho T2 (FL shuttle + Memory extractor)
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::ForgedLight,
        LoomNodeRef::Shuttle(0),
        Resource::Memory,
        LoomNodeRef::Extractor(NodeId::MemoryArchive),
        NodeNature::Form,
        Resource::EmberEcho,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 13);
}

#[test]
fn test_pattern_14_pressure_test_is_solvable() {
    let mut loom = setup_loom(14);
    // ForgedLight T1 + EchoGlass T1.
    // FL = min(Ember L3=50, Void L4=62.5) = 50 >= 45.
    // EG = min(Memory L2=37.5, Silence L3=50) = 37.5 >= 35.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 4);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 2);
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
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 14);
}

#[test]
fn test_pattern_15_three_confluences_is_solvable() {
    let mut loom = setup_loom(15);
    // ForgedLight T1 + EchoGlass T1 + StillbornSong T1.
    // Silence L6 = 87.5, shared by EG and SS -> 87.5/2 = 43.75 each.
    // Memory L3 = 50.
    // EG = min(50, 43.75) = 43.75 >= 40.
    // SS = min(43.75, Resonance L3=50) = 43.75 >= 40.
    // FL = min(Ember L3=50, Void L4=62.5) = 50 >= 40.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 3);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 4);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 3);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 6);
    set_extractor_level(&mut loom, NodeId::ResonanceForge, 3);
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
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Silence,
        NodeId::SilenceWell,
        Resource::Resonance,
        NodeId::ResonanceForge,
        NodeNature::Stillness,
        Resource::StillbornSong,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 15);
}

// ── Chapter III: The Unraveling (Patterns 16-27) ────────────────────────────

#[test]
fn test_pattern_16_the_amplifier_is_solvable() {
    let mut loom = setup_loom(16);
    // ForgedLight T1. Ember L4 = 62.5, Void L4 = 62.5. FL = 62.5 >= 60.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 4);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 4);
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 16);
}

#[test]
fn test_pattern_17_purified_cascade_is_solvable() {
    let mut loom = setup_loom(17);
    // ForgedLight T1 + EchoGlass T1 + PurifiedVoid T2.
    // VoidEssence L5 = 75, shared by FL and PV -> 75/2 = 37.5 each.
    // FL = min(Ember L4=62.5, 37.5) = 37.5 >= 35.
    // EG = min(Memory L3=50, Silence L6=87.5) = 50.
    // PV = min(EG shuttle=50, Void=37.5) = 37.5 >= 35.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 4);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 5);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 3);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 6);
    // Shuttle 0: ForgedLight T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    // Shuttle 1: EchoGlass T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    ));
    // Shuttle 2: PurifiedVoid T2 (EG shuttle + Void extractor)
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(1),
        Resource::VoidEssence,
        LoomNodeRef::Extractor(NodeId::VoidCondenser),
        NodeNature::Void,
        Resource::PurifiedVoid,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 17);
}

#[test]
fn test_pattern_18_resonance_cascade_is_solvable() {
    let mut loom = setup_loom(18);
    // StillbornSong T1. Silence L6 = 87.5, Resonance L5 = 75.
    // SS = min(87.5, 75) = 75 >= 50. Resonance base = 75 >= 75.
    set_extractor_level(&mut loom, NodeId::SilenceWell, 6);
    set_extractor_level(&mut loom, NodeId::ResonanceForge, 5);
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Silence,
        NodeId::SilenceWell,
        Resource::Resonance,
        NodeId::ResonanceForge,
        NodeNature::Stillness,
        Resource::StillbornSong,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 18);
}

#[test]
fn test_pattern_19_first_weave_is_solvable() {
    let mut loom = setup_loom(19);
    // ForgedLight T1 + EchoGlass T1 + WovenReality T3.
    // FL = min(Ember L4=62.5, Void L5=75) = 62.5.
    // EG = min(Memory L3=50, Silence L6=87.5) = 50.
    // WR T3 = min(FL shuttle=62.5, EG shuttle=50) = 50 >= 15.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 4);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 5);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 3);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 6);
    // Shuttle 0: ForgedLight T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    // Shuttle 1: EchoGlass T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    ));
    // Shuttle 2: WovenReality T3 (FL shuttle + EG shuttle)
    loom.persistent.shuttles.push(make_t3_shuttle(
        Resource::ForgedLight,
        LoomNodeRef::Shuttle(0),
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(1),
        NodeNature::Vibration,
        Resource::WovenReality,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 19);
}

#[test]
fn test_pattern_20_the_unraveling_is_solvable() {
    let mut loom = setup_loom(20);
    // ForgedLight T1 + EchoGlass T1 (shared->WR+PV) + WovenReality T3 + PurifiedVoid T2.
    // Memory L4 = 62.5.
    // EG = min(62.5, Silence L6=87.5) = 62.5, shared by WR and PV -> 62.5/2 = 31.25 each.
    // Void L5 = 75, shared by FL and PV -> 75/2 = 37.5 each.
    // FL = min(Ember L4=62.5, 37.5) = 37.5.
    // WR = min(FL shuttle=37.5, EG share=31.25) = 31.25 >= 30.
    // PV = min(EG share=31.25, Void share=37.5) = 31.25 >= 25.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 4);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 5);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 4);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 6);
    // Shuttle 0: ForgedLight T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    // Shuttle 1: EchoGlass T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    ));
    // Shuttle 2: WovenReality T3 (FL shuttle + EG shuttle)
    loom.persistent.shuttles.push(make_t3_shuttle(
        Resource::ForgedLight,
        LoomNodeRef::Shuttle(0),
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(1),
        NodeNature::Vibration,
        Resource::WovenReality,
    ));
    // Shuttle 3: PurifiedVoid T2 (EG shuttle + Void extractor)
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(1),
        Resource::VoidEssence,
        LoomNodeRef::Extractor(NodeId::VoidCondenser),
        NodeNature::Void,
        Resource::PurifiedVoid,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 20);
}

#[test]
fn test_pattern_21_grand_harmony_is_solvable() {
    let mut loom = setup_loom(21);
    // ForgedLight T1 + EchoGlass T1 (no sharing needed for confluence outputs).
    // All extractors L4 = 62.5 >= 62.5 for base resource requirements.
    // Reflection L4 = 62.5 >= 62.5.
    // FL = min(Ember L4=62.5, Void L4=62.5) = 62.5 >= 25.
    // EG = min(Memory L4=62.5, Silence L4=62.5) = 62.5 >= 25.
    // But Ember is consumed by FL shuttle AND needed as base rate.
    // The base rate tracks extractor production (buffer filling), not consumption.
    // Contention only applies to shuttles pulling from extractors.
    // Base production rate is always the full extractor rate regardless of shuttle pulls.
    // So Ember rate tracker = 62.5/hr, even though FL pulls from the buffer.
    for &node_id in &NodeId::ALL {
        set_extractor_level(&mut loom, node_id, 4);
    }
    // Shuttle 0: ForgedLight T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    // Shuttle 1: EchoGlass T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 21);
}

#[test]
fn test_pattern_22_the_knot_is_solvable() {
    let mut loom = setup_loom(22);
    // SS T1 + CE T2 + FL T1 + EG T1 + WR T3.
    // Ember L4 = 62.5, shared by FL and CE -> 62.5/2 = 31.25 each.
    // Silence L6 = 87.5, shared by EG and SS -> 87.5/2 = 43.75 each.
    // EG = min(Memory L4=62.5, 43.75) = 43.75.
    // SS = min(43.75, Resonance L5=75) = 43.75.
    // FL = min(31.25, Void L5=75) = 31.25.
    // CE T2 = min(SS=43.75, Ember=31.25) = 31.25 >= 30.
    // WR T3 = min(FL=31.25, EG=43.75) = 31.25 >= 15.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 4);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 5);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 4);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 6);
    set_extractor_level(&mut loom, NodeId::ResonanceForge, 5);
    // Shuttle 0: ForgedLight T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    // Shuttle 1: EchoGlass T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    ));
    // Shuttle 2: StillbornSong T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Silence,
        NodeId::SilenceWell,
        Resource::Resonance,
        NodeId::ResonanceForge,
        NodeNature::Stillness,
        Resource::StillbornSong,
    ));
    // Shuttle 3: CondensedEmber T2 (SS shuttle + Ember extractor)
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::StillbornSong,
        LoomNodeRef::Shuttle(2),
        Resource::Ember,
        LoomNodeRef::Extractor(NodeId::EmberSpindle),
        NodeNature::Heat,
        Resource::CondensedEmber,
    ));
    // Shuttle 4: WovenReality T3 (FL shuttle + EG shuttle)
    loom.persistent.shuttles.push(make_t3_shuttle(
        Resource::ForgedLight,
        LoomNodeRef::Shuttle(0),
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(1),
        NodeNature::Vibration,
        Resource::WovenReality,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 22);
}

#[test]
fn test_pattern_23_strange_alchemy_is_solvable() {
    let mut loom = setup_loom(23);
    // FL T1 + EG T1 + SS T1 + EE T2 + PV T2.
    // Memory L4 = 62.5, shared by EG and EE -> 62.5/2 = 31.25 each.
    // Void L5 = 75, shared by FL and PV -> 75/2 = 37.5 each.
    // Silence L6 = 87.5, shared by EG and SS -> 87.5/2 = 43.75 each.
    // FL = min(Ember L4=62.5, 37.5) = 37.5.
    // EG = min(31.25, 43.75) = 31.25.
    // SS = min(43.75, Resonance L5=75) = 43.75 >= 35.
    // EE T2 = min(FL=37.5, Memory=31.25) = 31.25 >= 30.
    // PV T2 = min(EG=31.25, Void=37.5) = 31.25 >= 30.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 4);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 5);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 4);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 6);
    set_extractor_level(&mut loom, NodeId::ResonanceForge, 5);
    // Shuttle 0: ForgedLight T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    // Shuttle 1: EchoGlass T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    ));
    // Shuttle 2: StillbornSong T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Silence,
        NodeId::SilenceWell,
        Resource::Resonance,
        NodeId::ResonanceForge,
        NodeNature::Stillness,
        Resource::StillbornSong,
    ));
    // Shuttle 3: EmberEcho T2 (FL shuttle + Memory extractor)
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::ForgedLight,
        LoomNodeRef::Shuttle(0),
        Resource::Memory,
        LoomNodeRef::Extractor(NodeId::MemoryArchive),
        NodeNature::Form,
        Resource::EmberEcho,
    ));
    // Shuttle 4: PurifiedVoid T2 (EG shuttle + Void extractor)
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(1),
        Resource::VoidEssence,
        LoomNodeRef::Extractor(NodeId::VoidCondenser),
        NodeNature::Void,
        Resource::PurifiedVoid,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 23);
}

#[test]
fn test_pattern_24_refined_purpose_is_solvable() {
    let mut loom = setup_loom(24);
    // FL T1 (shared->WR+EE) + EG T1 + WR T3 + EE T2.
    // Memory L4 = 62.5, shared by EG and EE -> 62.5/2 = 31.25 each.
    // FL = min(Ember L4=62.5, Void L5=75) = 62.5, shared by WR and EE -> 62.5/2 = 31.25 each.
    // EG = min(31.25, Silence L6=87.5) = 31.25.
    // WR = min(FL share=31.25, EG=31.25) = 31.25 >= 30.
    // EE = min(FL share=31.25, Memory share=31.25) = 31.25 >= 20.
    // Reflection L5 = 75 >= 75.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 4);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 5);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 4);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 6);
    set_extractor_level(&mut loom, NodeId::ReflectionLens, 5);
    // Shuttle 0: ForgedLight T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    // Shuttle 1: EchoGlass T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    ));
    // Shuttle 2: WovenReality T3 (FL shuttle + EG shuttle)
    loom.persistent.shuttles.push(make_t3_shuttle(
        Resource::ForgedLight,
        LoomNodeRef::Shuttle(0),
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(1),
        NodeNature::Vibration,
        Resource::WovenReality,
    ));
    // Shuttle 3: EmberEcho T2 (FL shuttle + Memory extractor)
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::ForgedLight,
        LoomNodeRef::Shuttle(0),
        Resource::Memory,
        LoomNodeRef::Extractor(NodeId::MemoryArchive),
        NodeNature::Form,
        Resource::EmberEcho,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 24);
}

#[test]
fn test_pattern_25_the_flood_is_solvable() {
    let mut loom = setup_loom(25);
    // ForgedLight T1 + EchoGlass T1 + WovenReality T3.
    // Ember L7 = 100. FL = min(100, Void L5=75) = 75.
    // EG = min(Memory L4=62.5, Silence L6=87.5) = 62.5.
    // WR = min(FL=75, EG=62.5) = 62.5 >= 45.
    // Ember base rate = 100 >= 100.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 7);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 5);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 4);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 6);
    // Shuttle 0: ForgedLight T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    // Shuttle 1: EchoGlass T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    ));
    // Shuttle 2: WovenReality T3 (FL shuttle + EG shuttle)
    loom.persistent.shuttles.push(make_t3_shuttle(
        Resource::ForgedLight,
        LoomNodeRef::Shuttle(0),
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(1),
        NodeNature::Vibration,
        Resource::WovenReality,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 25);
}

#[test]
fn test_pattern_26_everything_flows_is_solvable() {
    let mut loom = setup_loom(26);
    // 5 slots: FL T1 + EG T1 + SS T1 + CE T2 + PV T2.
    // Ember L9 = 125, shared by FL and CE -> 125/2 = 62.5 each.
    // Void L9 = 125, shared by FL and PV -> 125/2 = 62.5 each.
    // Silence L8 = 112.5, shared by EG and SS -> 112.5/2 = 56.25 each.
    // Memory L5 = 75.
    // FL = min(Ember=62.5, Void=62.5) = 62.5 >= 60.
    // EG = min(75, 56.25) = 56.25 >= 55.
    // SS = min(56.25, Resonance L5=75) = 56.25 >= 50.
    // CE T2 = min(SS=56.25, Ember=62.5) = 56.25 >= 25.
    // PV T2 = min(EG=56.25, Void=62.5) = 56.25 >= 20.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 9);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 9);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 5);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 8);
    set_extractor_level(&mut loom, NodeId::ResonanceForge, 5);
    // Shuttle 0: ForgedLight T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    // Shuttle 1: EchoGlass T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    ));
    // Shuttle 2: StillbornSong T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Silence,
        NodeId::SilenceWell,
        Resource::Resonance,
        NodeId::ResonanceForge,
        NodeNature::Stillness,
        Resource::StillbornSong,
    ));
    // Shuttle 3: CondensedEmber T2 (SS shuttle + Ember extractor)
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::StillbornSong,
        LoomNodeRef::Shuttle(2),
        Resource::Ember,
        LoomNodeRef::Extractor(NodeId::EmberSpindle),
        NodeNature::Heat,
        Resource::CondensedEmber,
    ));
    // Shuttle 4: PurifiedVoid T2 (EG shuttle + Void extractor)
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(1),
        Resource::VoidEssence,
        LoomNodeRef::Extractor(NodeId::VoidCondenser),
        NodeNature::Void,
        Resource::PurifiedVoid,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 26);
}

#[test]
fn test_pattern_27_mended_loom_is_solvable() {
    let mut loom = setup_loom(27);
    // 5 slots: FL T1(shared->WR+EE) + EG T1(shared->WR+PV) + WR T3 + EE T2 + PV T2.
    // Ember L9 = 125 (no contention on Ember, only FL uses it).
    // Void L15 = 200, shared by FL and PV -> 200/2 = 100 each.
    // Memory L15 = 200, shared by EG and EE -> 200/2 = 100 each.
    // Silence L9 = 125 (no contention on Silence, only EG uses it).
    // FL = min(125, 100) = 100, shared by WR and EE -> 100/2 = 50 each.
    // EG = min(100, 125) = 100, shared by WR and PV -> 100/2 = 50 each.
    // WR = min(FL share=50, EG share=50) = 50 >= 50.
    // EE = min(FL share=50, Memory share=100) = 50 >= 30.
    // PV = min(EG share=50, Void share=100) = 50 >= 30.
    // Reflection L9 = 125 >= 125.
    set_extractor_level(&mut loom, NodeId::EmberSpindle, 9);
    set_extractor_level(&mut loom, NodeId::VoidCondenser, 15);
    set_extractor_level(&mut loom, NodeId::MemoryArchive, 15);
    set_extractor_level(&mut loom, NodeId::SilenceWell, 9);
    set_extractor_level(&mut loom, NodeId::ReflectionLens, 9);
    // Shuttle 0: ForgedLight T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Ember,
        NodeId::EmberSpindle,
        Resource::VoidEssence,
        NodeId::VoidCondenser,
        NodeNature::Heat,
        Resource::ForgedLight,
    ));
    // Shuttle 1: EchoGlass T1
    loom.persistent.shuttles.push(make_t1_shuttle(
        Resource::Memory,
        NodeId::MemoryArchive,
        Resource::Silence,
        NodeId::SilenceWell,
        NodeNature::Pattern,
        Resource::EchoGlass,
    ));
    // Shuttle 2: WovenReality T3 (FL shuttle + EG shuttle)
    loom.persistent.shuttles.push(make_t3_shuttle(
        Resource::ForgedLight,
        LoomNodeRef::Shuttle(0),
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(1),
        NodeNature::Vibration,
        Resource::WovenReality,
    ));
    // Shuttle 3: EmberEcho T2 (FL shuttle + Memory extractor)
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::ForgedLight,
        LoomNodeRef::Shuttle(0),
        Resource::Memory,
        LoomNodeRef::Extractor(NodeId::MemoryArchive),
        NodeNature::Form,
        Resource::EmberEcho,
    ));
    // Shuttle 4: PurifiedVoid T2 (EG shuttle + Void extractor)
    loom.persistent.shuttles.push(make_t2_shuttle(
        Resource::EchoGlass,
        LoomNodeRef::Shuttle(1),
        Resource::VoidEssence,
        LoomNodeRef::Extractor(NodeId::VoidCondenser),
        NodeNature::Void,
        Resource::PurifiedVoid,
    ));
    run_ticks(&mut loom, 250);
    verify_pattern(&loom, 27);
}
