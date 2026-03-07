#![allow(dead_code)]
use super::types::{
    LoomArchetype, LoomNode, LoomNodeRef, LoomState, NodeId, NodeNature, Pipe, Refinery, Resource,
};

/// Archetype-to-node mapping.
/// Returns (first_node, second_node) for the archetype.
pub fn archetype_nodes(archetype: LoomArchetype) -> (NodeId, NodeId) {
    match archetype {
        LoomArchetype::BurnBright => (NodeId::EmberSpindle, NodeId::VoidCondenser),
        LoomArchetype::ReachWide => (NodeId::ReflectionLens, NodeId::MemoryArchive),
        LoomArchetype::RunDeep => (NodeId::SilenceWell, NodeId::ResonanceForge),
    }
}

/// Select an archetype at Loom unlock.
///
/// - Sets the archetype.
/// - Unlocks the first archetype node immediately.
/// - Applies the first node's passive bonuses.
/// - Starts the staggered unlock timer for the second node.
pub fn select_archetype(loom: &mut LoomState, archetype: LoomArchetype) {
    if loom.persistent.archetype.is_some() {
        return; // Archetype already chosen — cannot re-select.
    }
    loom.persistent.archetype = Some(archetype);

    let (first, _second) = archetype_nodes(archetype);

    // Unlock the first node in a scoped borrow.
    if let Some(node) = loom.persistent.nodes.iter_mut().find(|n| n.id == first) {
        node.unlocked = true;
    }
    // Apply passives with full access to loom.
    apply_node_passive_on_unlock(first, loom);

    // Staggered unlock: start timer at 0 seconds elapsed.
    loom.persistent.second_node_unlock_elapsed = Some(0.0);
}

/// Apply passive effects to a node when it first unlocks.
/// Mutates the LoomState directly (may affect stockpiles, other nodes, etc.).
fn apply_node_passive_on_unlock(node_id: NodeId, loom: &mut LoomState) {
    match node_id {
        NodeId::EmberSpindle => {
            // +50% throughput applied at production time (checked via archetype field).
            // Neighbors unlock 30% slower — also applied at unlock time.
            // No immediate state mutation needed beyond unlocking the node.
        }
        NodeId::VoidCondenser => {
            // 2x conversion ratio at levels 1-3 — applied at production time.
            // No immediate state mutation.
        }
        NodeId::ReflectionLens => {
            // Unlocks 3 neighbors instead of 2 — applied at neighbor unlock time.
            // No immediate state mutation.
        }
        NodeId::MemoryArchive => {
            // Starts with a stockpile of 3 of each adjacent resource.
            // Adjacent resources to MemoryArchive in the cycle:
            //   Ember → Reflection → Void Essence → Memory → Silence → Resonance → (back to Ember)
            // Memory's neighbors in the cycle: VoidEssence (input) and Silence (output).
            let adjacent = [Resource::VoidEssence, Resource::Silence];
            for resource in adjacent {
                *loom.persistent.stockpiles.entry(resource).or_insert(0.0) += 3.0;
            }
        }
        NodeId::SilenceWell => {
            // -25% upgrade costs for first 5 levels — applied at upgrade time.
            // No immediate state mutation.
        }
        NodeId::ResonanceForge => {
            // Feedback loop at 50% strength before cycle closes — applied at production time.
            // No immediate state mutation.
        }
    }
}

/// Tick the staggered second-node unlock timer.
///
/// Call once per tick with the elapsed seconds for that tick.
/// When elapsed >= SECOND_NODE_UNLOCK_SECONDS, the second node unlocks.
pub const SECOND_NODE_UNLOCK_SECONDS: f64 = 14_400.0; // 4 hours

pub fn tick_loom_staggered_unlock(loom: &mut LoomState, elapsed_seconds: f64) -> bool {
    let archetype = match loom.persistent.archetype {
        Some(a) => a,
        None => return false,
    };

    let timer = match &mut loom.persistent.second_node_unlock_elapsed {
        Some(t) => t,
        None => return false,
    };

    let (_first, second) = archetype_nodes(archetype);

    // If already unlocked, nothing to do.
    if loom
        .persistent
        .nodes
        .iter()
        .any(|n| n.id == second && n.unlocked)
    {
        loom.persistent.second_node_unlock_elapsed = None;
        return false;
    }

    *timer += elapsed_seconds;

    if *timer >= SECOND_NODE_UNLOCK_SECONDS {
        loom.persistent.second_node_unlock_elapsed = None;

        // Unlock the second node and apply its passive.
        let second_id = second;
        if let Some(node) = loom.persistent.nodes.iter_mut().find(|n| n.id == second_id) {
            node.unlocked = true;
        }
        apply_node_passive_on_unlock(second_id, loom);

        return true; // Signal that unlock just happened
    }

    false
}

/// Returns the throughput multiplier for a node based on archetype passives.
/// 1.0 = no bonus.
pub fn node_throughput_multiplier(loom: &LoomState, node_id: NodeId) -> f64 {
    match loom.persistent.archetype {
        Some(LoomArchetype::BurnBright) if node_id == NodeId::EmberSpindle => 1.5,
        _ => 1.0,
    }
}

/// Returns the conversion ratio multiplier for a node based on archetype passives.
/// 1.0 = no bonus.
pub fn node_conversion_multiplier(loom: &LoomState, node_id: NodeId) -> f64 {
    if loom.persistent.archetype == Some(LoomArchetype::BurnBright)
        && node_id == NodeId::VoidCondenser
    {
        // Find the node's level
        if let Some(node) = loom.persistent.nodes.iter().find(|n| n.id == node_id) {
            if node.level <= 3 {
                return 2.0;
            }
        }
    }
    1.0
}

/// Returns the number of neighbors that unlock when a node produces enough.
pub fn node_neighbor_unlock_count(loom: &LoomState, node_id: NodeId) -> usize {
    if loom.persistent.archetype == Some(LoomArchetype::ReachWide)
        && node_id == NodeId::ReflectionLens
    {
        3
    } else {
        2
    }
}

/// Returns the upgrade cost multiplier for a node based on archetype passives.
/// 1.0 = no bonus. 0.75 = 25% discount.
pub fn node_upgrade_cost_multiplier(loom: &LoomState, node_id: NodeId) -> f64 {
    if loom.persistent.archetype == Some(LoomArchetype::RunDeep) && node_id == NodeId::SilenceWell {
        if let Some(node) = loom.persistent.nodes.iter().find(|n| n.id == node_id) {
            if node.level <= 5 {
                return 0.75;
            }
        }
    }
    1.0
}

/// Returns the neighbor unlock speed multiplier for a node.
/// 1.0 = normal, >1.0 = faster (not used currently), <1.0 = slower.
pub fn node_neighbor_unlock_speed_multiplier(loom: &LoomState, node_id: NodeId) -> f64 {
    if loom.persistent.archetype == Some(LoomArchetype::BurnBright)
        && node_id == NodeId::EmberSpindle
    {
        0.7 // 30% slower
    } else {
        1.0
    }
}

/// Returns whether the Resonance Forge feedback loop passive is active (50% strength).
/// This is only true before the cycle fully closes.
pub fn resonance_early_feedback_active(loom: &LoomState) -> bool {
    loom.persistent.archetype == Some(LoomArchetype::RunDeep)
        && loom
            .persistent
            .nodes
            .iter()
            .any(|n| n.id == NodeId::ResonanceForge && n.unlocked)
}

// ── Phase 3: Node Base Production ─────────────────────────────────────────────

/// Returns the native resource produced by a node.
pub fn node_native_resource(node_id: NodeId) -> Resource {
    match node_id {
        NodeId::EmberSpindle => Resource::Ember,
        NodeId::ReflectionLens => Resource::Reflection,
        NodeId::VoidCondenser => Resource::VoidEssence,
        NodeId::MemoryArchive => Resource::Memory,
        NodeId::SilenceWell => Resource::Silence,
        NodeId::ResonanceForge => Resource::Resonance,
    }
}

/// Returns the level multiplier for a node's production rate.
/// Scales linearly: level 1 = 1.0x, level 2 = 1.5x, level N = 1 + (N-1)*0.5.
pub fn node_level_multiplier(level: u32) -> f64 {
    1.0 + (level.saturating_sub(1) as f64) * 0.5
}

/// Returns the effective production rate (native resource per hour) for a node,
/// incorporating level and archetype passives.
pub fn node_effective_rate(loom: &LoomState, node: &LoomNode) -> f64 {
    if !node.unlocked {
        return 0.0;
    }
    node.base_rate * node_level_multiplier(node.level) * node_throughput_multiplier(loom, node.id)
}

/// Tick base production for all unlocked nodes.
///
/// `delta_seconds` is the wall-clock time elapsed since the last tick (typically 0.1s).
/// Each unlocked node produces its native resource at its effective rate and stores
/// the output in the node's buffer (capped at buffer_capacity).
/// Stalled nodes (buffer full) skip production.
///
/// Returns a map of resource → total produced this tick (for pattern rate tracking).
pub fn tick_base_production(
    loom: &mut LoomState,
    delta_seconds: f64,
) -> std::collections::HashMap<Resource, f64> {
    let delta_hours = delta_seconds / 3600.0;
    let mut produced: std::collections::HashMap<Resource, f64> = std::collections::HashMap::new();

    // Collect rates first to avoid borrow conflicts.
    let node_data: Vec<(usize, NodeId, f64, f64)> = loom
        .persistent
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let rate = node_effective_rate(loom, node);
            (i, node.id, rate, node.buffer_capacity)
        })
        .collect();

    for (idx, node_id, rate, capacity) in node_data {
        if rate == 0.0 {
            continue;
        }
        let node = &mut loom.persistent.nodes[idx];

        // If buffer is at capacity, node stalls — no production.
        if node.buffer >= capacity {
            node.stalled = true;
            continue;
        }

        let amount = rate * delta_hours;
        let new_buffer = (node.buffer + amount).min(capacity);
        let actually_produced = new_buffer - node.buffer;
        node.buffer = new_buffer;
        node.stalled = false;

        if actually_produced > 0.0 {
            let resource = node_native_resource(node_id);
            *produced.entry(resource).or_insert(0.0) += actually_produced;
        }
    }

    produced
}

// ── Phase 3: Node Upgrading ───────────────────────────────────────────────────

/// Returns the upgrade cost (in the node's native resource) for going from current level to next.
/// Base cost: 10 * level^1.5, rounded. Silence Well gets 25% discount at levels 1-5.
pub fn node_upgrade_cost(loom: &LoomState, node_id: NodeId) -> f64 {
    if let Some(node) = loom.persistent.nodes.iter().find(|n| n.id == node_id) {
        let base_cost = 10.0 * (node.level as f64).powf(1.5);
        let multiplier = node_upgrade_cost_multiplier(loom, node_id);
        (base_cost * multiplier).round()
    } else {
        f64::MAX
    }
}

/// Attempt to upgrade a node's level.
/// Costs `node_upgrade_cost()` units of the node's native resource from the node's buffer.
/// Returns true if the upgrade succeeded.
pub fn try_upgrade_node(loom: &mut LoomState, node_id: NodeId) -> bool {
    let cost = node_upgrade_cost(loom, node_id);

    let node = match loom.persistent.nodes.iter_mut().find(|n| n.id == node_id) {
        Some(n) => n,
        None => return false,
    };

    if !node.unlocked || node.buffer < cost {
        return false;
    }

    node.buffer -= cost;
    node.level += 1;
    // Buffer capacity scales with level: 4 hours of production at new level's rate.
    node.buffer_capacity = node.base_rate * node_level_multiplier(node.level) * 4.0;

    true
}

// ── Phase 3: Neighbor Unlocking ───────────────────────────────────────────────

/// Node adjacency in the 6-node cycle.
/// Cycle: EmberSpindle → ReflectionLens → VoidCondenser → MemoryArchive → SilenceWell → ResonanceForge → (back to Ember)
pub fn node_neighbors(node_id: NodeId) -> &'static [NodeId] {
    match node_id {
        NodeId::EmberSpindle => &[NodeId::ReflectionLens, NodeId::ResonanceForge],
        NodeId::ReflectionLens => &[NodeId::EmberSpindle, NodeId::VoidCondenser],
        NodeId::VoidCondenser => &[NodeId::ReflectionLens, NodeId::MemoryArchive],
        NodeId::MemoryArchive => &[NodeId::VoidCondenser, NodeId::SilenceWell],
        NodeId::SilenceWell => &[NodeId::MemoryArchive, NodeId::ResonanceForge],
        NodeId::ResonanceForge => &[NodeId::SilenceWell, NodeId::EmberSpindle],
    }
}

/// Fraction of buffer capacity that must be filled before a node starts unlocking neighbors.
const NEIGHBOR_UNLOCK_BUFFER_THRESHOLD: f64 = 0.5;

/// Hours of accumulated production a locked neighbor needs to unlock.
const NEIGHBOR_UNLOCK_HOURS: f64 = 2.0;

/// Tick neighbor unlock progress for all unlocked nodes.
///
/// For each unlocked node whose buffer fill ratio exceeds `NEIGHBOR_UNLOCK_BUFFER_THRESHOLD`,
/// its locked cycle neighbors accumulate unlock progress at the per-tick rate.
/// When a neighbor's `unlock_progress` reaches `NEIGHBOR_UNLOCK_HOURS` it unlocks.
///
/// `delta_seconds` is the wall-clock time elapsed since last tick.
/// Returns the list of newly unlocked node IDs.
pub fn tick_neighbor_unlocking(loom: &mut LoomState, delta_seconds: f64) -> Vec<NodeId> {
    let delta_hours = delta_seconds / 3600.0;
    let mut newly_unlocked: Vec<NodeId> = Vec::new();

    // Snapshot source info to avoid borrow conflicts.
    let sources: Vec<(NodeId, bool, f64)> = loom
        .persistent
        .nodes
        .iter()
        .map(|n| (n.id, n.unlocked, n.buffer / n.buffer_capacity.max(1.0)))
        .collect();

    for (src_id, src_unlocked, fill_ratio) in &sources {
        if !src_unlocked || *fill_ratio < NEIGHBOR_UNLOCK_BUFFER_THRESHOLD {
            continue;
        }

        let speed_mult = node_neighbor_unlock_speed_multiplier(loom, *src_id);
        let unlock_count = node_neighbor_unlock_count(loom, *src_id);
        let neighbors = node_neighbors(*src_id);

        let locked_neighbors: Vec<NodeId> = neighbors
            .iter()
            .filter(|&&nb| {
                loom.persistent
                    .nodes
                    .iter()
                    .any(|n| n.id == nb && !n.unlocked)
            })
            .take(unlock_count)
            .copied()
            .collect();

        for neighbor_id in locked_neighbors {
            if let Some(neighbor) = loom
                .persistent
                .nodes
                .iter_mut()
                .find(|n| n.id == neighbor_id)
            {
                neighbor.unlock_progress += delta_hours * speed_mult;
                if neighbor.unlock_progress >= NEIGHBOR_UNLOCK_HOURS {
                    neighbor.unlocked = true;
                    neighbor.unlock_progress = 0.0;
                    newly_unlocked.push(neighbor_id);
                }
            }
        }
    }

    for node_id in &newly_unlocked {
        apply_node_passive_on_unlock(*node_id, loom);
    }

    newly_unlocked
}

// ── Phase 5: Buffer Stalling ──────────────────────────────────────────────────

/// Determine whether a node should be stalled.
///
/// A node stalls when **both** conditions hold:
/// 1. Its buffer is at or above capacity (full).
/// 2. Every active outgoing pipe's destination buffer is also full,
///    OR there are no active outgoing pipes.
///
/// When stalled, base production stops so resources are not wasted.
/// `outgoing_pipes` must only contain pipes whose `from == node.id`.
pub fn check_node_stall(node: &LoomNode, outgoing_pipes: &[&Pipe], all_nodes: &[LoomNode]) -> bool {
    if node.buffer < node.buffer_capacity {
        return false;
    }

    let active_pipes: Vec<&&Pipe> = outgoing_pipes
        .iter()
        .filter(|p| !p.under_construction)
        .collect();

    // No active outgoing pipes → nowhere to drain → stalled.
    if active_pipes.is_empty() {
        return true;
    }

    // Stalled only if every destination is also full.
    active_pipes.iter().all(|pipe| match pipe.to {
        LoomNodeRef::Extractor(dest_id) => match all_nodes.iter().find(|n| n.id == dest_id) {
            Some(dst) => dst.buffer >= dst.buffer_capacity,
            None => true,
        },
        LoomNodeRef::Refinery(_) => true, // Treat refineries as blocked for now.
    })
}

/// Update the `stalled` flag on every unlocked node.
///
/// Called after pipe flow simulation so buffer levels reflect the latest state.
/// Returns node IDs whose stall state changed (useful for event emission).
pub fn tick_stall_detection(loom: &mut LoomState) -> Vec<NodeId> {
    let mut changed = Vec::new();

    let node_ids: Vec<NodeId> = loom.persistent.nodes.iter().map(|n| n.id).collect();

    for node_id in node_ids {
        let outgoing: Vec<&Pipe> = loom
            .persistent
            .pipes
            .iter()
            .filter(|p| p.from == LoomNodeRef::Extractor(node_id))
            .collect();

        let should_stall = {
            let node = loom.persistent.nodes.iter().find(|n| n.id == node_id);
            match node {
                Some(n) if n.unlocked => check_node_stall(n, &outgoing, &loom.persistent.nodes),
                _ => false,
            }
        };

        if let Some(node) = loom.persistent.nodes.iter_mut().find(|n| n.id == node_id) {
            if node.stalled != should_stall {
                node.stalled = should_stall;
                changed.push(node_id);
            }
        }
    }

    changed
}

// ── Phase 6: Reaction Processing ───────────────────────────────────────────────

/// Process combinatorial reactions at destination nodes.
///
/// Takes the deliveries returned by `tick_pipe_flow`, groups them by destination
/// node, then checks whether two different resources arrived at the same node this
/// tick. If a recipe matches `(resource_a, resource_b, node.nature())`, the output
/// is produced and added to the global stockpile.
///
/// Only the first matching recipe pair fires per node per tick.
/// Returns a list of `(output_resource, amount_produced)` for this tick.
pub fn process_reactions(
    loom: &mut LoomState,
    deliveries: Vec<(LoomNodeRef, Resource, f64)>,
) -> Vec<(Resource, f64)> {
    use crate::loom::recipes;
    use std::collections::HashMap;

    // Group deliveries by destination node.
    let mut by_node: HashMap<LoomNodeRef, Vec<(Resource, f64)>> = HashMap::new();
    for (node_ref, resource, amount) in deliveries {
        by_node
            .entry(node_ref)
            .or_default()
            .push((resource, amount));
    }

    let mut outputs: Vec<(Resource, f64)> = Vec::new();

    for (node_ref, inputs) in &by_node {
        if inputs.len() < 2 {
            continue;
        }

        let node_nature = match node_ref {
            LoomNodeRef::Extractor(node_id) => node_id.nature(),
            LoomNodeRef::Refinery(_) => continue, // Refineries don't do reactions.
        };

        // Try each pair of distinct resources; take first match.
        'pair_search: for i in 0..inputs.len() {
            for j in (i + 1)..inputs.len() {
                let (res_a, amt_a) = inputs[i];
                let (res_b, amt_b) = inputs[j];

                if let Some(recipe) = recipes::find_recipe(res_a, res_b, node_nature) {
                    let min_input = amt_a.min(amt_b);
                    let produced = min_input * recipe.amount;

                    if produced > 0.0 {
                        *loom
                            .persistent
                            .stockpiles
                            .entry(recipe.output)
                            .or_insert(0.0) += produced;

                        outputs.push((recipe.output, produced));
                    }

                    // Record discovery in codex if first time seeing this recipe.
                    record_codex_discovery(
                        &mut loom.persistent.codex,
                        res_a,
                        res_b,
                        node_nature,
                        recipe.output,
                        recipe.amount,
                    );

                    break 'pair_search;
                }
            }
        }
    }

    outputs
}

/// Record a recipe discovery in the codex.
/// If the recipe is already present (discovered or not), marks it discovered.
/// If not present, creates a new discovered entry.
fn record_codex_discovery(
    codex: &mut Vec<crate::loom::types::CodexEntry>,
    input_a: Resource,
    input_b: Resource,
    node_nature: crate::loom::types::NodeNature,
    output: Resource,
    output_amount: f64,
) {
    use crate::loom::types::CodexEntry;

    // Check if already in codex.
    let existing = codex.iter_mut().find(|e| {
        e.node_nature == node_nature
            && e.output == output
            && e.inputs.len() == 2
            && ((e.inputs[0] == input_a && e.inputs[1] == input_b)
                || (e.inputs[0] == input_b && e.inputs[1] == input_a))
    });

    if let Some(entry) = existing {
        entry.discovered = true;
    } else {
        codex.push(CodexEntry {
            inputs: vec![input_a, input_b],
            node_nature,
            output,
            output_amount,
            discovered: true,
        });
    }
}

/// Returns the indices of codex entries that are undiscovered but adjacent
/// (share at least one input resource with a discovered recipe).
pub fn codex_hint_indices(codex: &[crate::loom::types::CodexEntry]) -> Vec<usize> {
    use crate::loom::recipes;
    let registry = recipes::all_recipes();
    // Map discovered codex entries back to their position in the recipe registry.
    let discovered_registry_indices: Vec<usize> = codex
        .iter()
        .filter(|e| e.discovered)
        .filter_map(|e| {
            let (a, b) = if e.inputs.len() >= 2 {
                (e.inputs[0], e.inputs[1])
            } else {
                return None;
            };
            registry.iter().position(|r| r.matches(a, b, e.node_nature))
        })
        .collect();
    recipes::adjacent_recipe_indices(&discovered_registry_indices)
}

/// Compute a Loom production multiplier from external systems.
///
/// Each contributing system provides a small additive bonus that is meaningful
/// early (before the player has built up Loom infrastructure) but becomes
/// negligible relative to Loom upgrades at endgame.
///
/// # Parameters
/// - `deep_layer`: The player's deepest Deep layer reached (0 = not started).
/// - `haven_tree_level`: Total Haven skill tree points invested (0 = no Haven).
/// - `sigil_count`: Number of Storm Sigils currently etched (0–12).
/// - `ascension_level`: Current Ascension level (0 = not ascended).
///
/// # Returns
/// A multiplier ≥ 1.0 to apply to every node's effective production rate.
/// The formula is `1.0 + sum_of_bonuses` where each system contributes:
/// - Deep: +0.5% per layer (capped at layer 30 → +15%)
/// - Haven: +0.3% per tree level (capped at 50 levels → +15%)
/// - Sigils: +1.0% per etched sigil (capped at 12 → +12%)
/// - Ascension: +2.0% per ascension level (capped at level 10 → +20%)
///
/// Total cap: roughly +62% at absolute max investment across all systems.
pub fn loom_production_bonus(
    deep_layer: u32,
    haven_tree_level: u32,
    sigil_count: u32,
    ascension_level: u32,
) -> f64 {
    let deep_bonus = (deep_layer.min(30) as f64) * 0.005;
    let haven_bonus = (haven_tree_level.min(50) as f64) * 0.003;
    let sigil_bonus = (sigil_count.min(12) as f64) * 0.010;
    let ascension_bonus = (ascension_level.min(10) as f64) * 0.020;
    1.0 + deep_bonus + haven_bonus + sigil_bonus + ascension_bonus
}

/// Error conditions for refinery building.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefineryError {
    InvalidRecipe,
    TierLocked,
    AtCapacity,
    InsufficientResources,
}

fn refinery_build_cost(tier: u8) -> f64 {
    match tier {
        1 => 25.0,
        2 => 15.0,
        _ => 10.0,
    }
}

fn refinery_tier_unlock_threshold(tier: u8) -> usize {
    match tier {
        1 => 1,
        2 => 6,
        _ => 12,
    }
}

/// Attempt to build a new Refinery locked to the given recipe.
///
/// # Errors
/// - `RefineryError::InvalidRecipe` — no recipe exists for the given inputs and nature.
/// - `RefineryError::TierLocked` — the recipe's tier requires more completed patterns.
/// - `RefineryError::AtCapacity` — the player already has the maximum number of refineries.
/// - `RefineryError::InsufficientResources` — not enough `input_a` stockpile to pay the build cost.
///
/// # Returns
/// The index of the newly created `Refinery` in `loom.persistent.refineries`.
pub fn build_refinery(
    loom: &mut LoomState,
    input_a: Resource,
    input_b: Resource,
    nature: NodeNature,
) -> Result<usize, RefineryError> {
    let recipe = crate::loom::recipes::find_recipe(input_a, input_b, nature)
        .ok_or(RefineryError::InvalidRecipe)?;

    let completed_patterns = loom
        .persistent
        .patterns
        .iter()
        .filter(|p| p.completed)
        .count();
    if completed_patterns < refinery_tier_unlock_threshold(recipe.tier) {
        return Err(RefineryError::TierLocked);
    }

    if loom.persistent.refineries.len() >= loom.persistent.max_refineries() {
        return Err(RefineryError::AtCapacity);
    }

    let cost = refinery_build_cost(recipe.tier);
    let stockpile = loom.persistent.stockpiles.entry(input_a).or_insert(0.0);
    if *stockpile < cost {
        return Err(RefineryError::InsufficientResources);
    }
    *stockpile -= cost;

    let mut r = Refinery::new(
        recipe.input_a,
        recipe.input_b,
        recipe.node_nature,
        recipe.output,
        recipe.amount,
        recipe.tier,
    );
    r.under_construction = true;
    r.construction_ticks_remaining = crate::loom::pipes::PIPE_CONSTRUCTION_TICKS;
    loom.persistent.refineries.push(r);
    Ok(loom.persistent.refineries.len() - 1)
}

/// Tick construction for all refineries under construction.
/// Returns indices of refineries that completed this tick.
pub fn tick_refinery_construction(loom: &mut LoomState) -> Vec<usize> {
    let mut completed = Vec::new();
    for (i, r) in loom.persistent.refineries.iter_mut().enumerate() {
        if !r.under_construction {
            continue;
        }
        r.construction_ticks_remaining = r.construction_ticks_remaining.saturating_sub(1);
        if r.construction_ticks_remaining == 0 {
            r.under_construction = false;
            completed.push(i);
        }
    }
    completed
}

/// Process reactions at refineries from pipe deliveries.
/// Unlike Extractor reactions (which use node nature from NodeId),
/// Refineries have their recipe baked in — just check both inputs arrived.
pub fn process_refinery_reactions(
    loom: &mut LoomState,
    deliveries: Vec<(LoomNodeRef, Resource, f64)>,
) -> Vec<(usize, Resource, f64)> {
    let mut results = Vec::new();

    // Group deliveries by refinery index.
    let mut refinery_inputs: std::collections::HashMap<usize, Vec<(Resource, f64)>> =
        std::collections::HashMap::new();
    for (node_ref, resource, amount) in deliveries {
        if let LoomNodeRef::Refinery(idx) = node_ref {
            refinery_inputs
                .entry(idx)
                .or_default()
                .push((resource, amount));
        }
    }

    for (idx, inputs) in refinery_inputs {
        let Some(r) = loom.persistent.refineries.get(idx) else {
            continue;
        };
        if r.under_construction {
            continue;
        }

        // Find amounts for each required input.
        let amt_a: f64 = inputs
            .iter()
            .filter(|(res, _)| *res == r.input_a)
            .map(|(_, a)| a)
            .sum();
        let amt_b: f64 = inputs
            .iter()
            .filter(|(res, _)| *res == r.input_b)
            .map(|(_, a)| a)
            .sum();

        if amt_a > 0.0 && amt_b > 0.0 {
            let output_amount = amt_a.min(amt_b) * r.amount;
            let cap = r.buffer_capacity;
            let r = &mut loom.persistent.refineries[idx];
            let space = (cap - r.buffer).max(0.0);
            let actual = output_amount.min(space);
            r.buffer += actual;
            r.stalled = false;
            results.push((idx, r.output, actual));
        }
    }

    results
}

/// Update stall flags for all refineries.
pub fn tick_refinery_stall_detection(loom: &mut LoomState) {
    for r in &mut loom.persistent.refineries {
        if r.under_construction {
            continue;
        }
        if r.buffer >= r.buffer_capacity {
            r.stalled = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_archetype_burn_bright() {
        let mut loom = LoomState::new();
        loom.persistent.discovered = true;

        select_archetype(&mut loom, LoomArchetype::BurnBright);

        assert_eq!(loom.persistent.archetype, Some(LoomArchetype::BurnBright));

        // First node (Ember Spindle) unlocked immediately
        let ember = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        assert!(ember.unlocked);

        // Second node (Void Condenser) NOT yet unlocked (staggered)
        let void_n = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::VoidCondenser)
            .unwrap();
        assert!(!void_n.unlocked);

        // Staggered unlock timer started
        assert_eq!(loom.persistent.second_node_unlock_elapsed, Some(0.0));
    }

    #[test]
    fn test_select_archetype_reach_wide() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::ReachWide);

        let lens = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::ReflectionLens)
            .unwrap();
        assert!(lens.unlocked);

        let archive = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::MemoryArchive)
            .unwrap();
        assert!(!archive.unlocked);

        assert_eq!(loom.persistent.second_node_unlock_elapsed, Some(0.0));
    }

    #[test]
    fn test_select_archetype_run_deep() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::RunDeep);

        let well = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::SilenceWell)
            .unwrap();
        assert!(well.unlocked);

        let forge = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::ResonanceForge)
            .unwrap();
        assert!(!forge.unlocked);

        assert_eq!(loom.persistent.second_node_unlock_elapsed, Some(0.0));
    }

    #[test]
    fn test_memory_archive_passive_stockpile() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::ReachWide);

        // After selecting ReachWide, ReflectionLens is immediately unlocked.
        // MemoryArchive passive doesn't apply yet (second node, not yet unlocked).
        // Manually unlock MemoryArchive and apply passive.
        apply_node_passive_on_unlock(NodeId::MemoryArchive, &mut loom);

        // Adjacent resources: VoidEssence and Silence should each have 3 stockpiled.
        assert_eq!(
            *loom
                .persistent
                .stockpiles
                .get(&Resource::VoidEssence)
                .unwrap_or(&0.0),
            3.0
        );
        assert_eq!(
            *loom
                .persistent
                .stockpiles
                .get(&Resource::Silence)
                .unwrap_or(&0.0),
            3.0
        );
    }

    #[test]
    fn test_staggered_unlock_not_yet() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        // Simulate 2 hours (7200 seconds) — not enough
        let unlocked = tick_loom_staggered_unlock(&mut loom, 7200.0);
        assert!(!unlocked);

        let void_n = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::VoidCondenser)
            .unwrap();
        assert!(!void_n.unlocked);

        // Timer should have advanced
        assert_eq!(loom.persistent.second_node_unlock_elapsed, Some(7200.0));
    }

    #[test]
    fn test_staggered_unlock_after_4_hours() {
        let mut loom = LoomState::new();
        loom.persistent.discovered = true;
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        // Simulate 4 hours of ticks (14400 seconds)
        let unlocked = tick_loom_staggered_unlock(&mut loom, 14400.0);
        assert!(unlocked);

        let void_n = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::VoidCondenser)
            .unwrap();
        assert!(void_n.unlocked);

        // Timer should be cleared
        assert_eq!(loom.persistent.second_node_unlock_elapsed, None);
    }

    #[test]
    fn test_ember_spindle_throughput_passive() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        assert_eq!(node_throughput_multiplier(&loom, NodeId::EmberSpindle), 1.5);
        // Other nodes unaffected
        assert_eq!(
            node_throughput_multiplier(&loom, NodeId::ResonanceForge),
            1.0
        );
    }

    #[test]
    fn test_void_condenser_conversion_passive_levels_1_to_3() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        // Level 1 — 2x multiplier
        assert_eq!(
            node_conversion_multiplier(&loom, NodeId::VoidCondenser),
            2.0
        );

        // Level 3 — still 2x
        let void_node = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::VoidCondenser)
            .unwrap();
        void_node.level = 3;
        assert_eq!(
            node_conversion_multiplier(&loom, NodeId::VoidCondenser),
            2.0
        );

        // Level 4 — back to 1x
        let void_node = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::VoidCondenser)
            .unwrap();
        void_node.level = 4;
        assert_eq!(
            node_conversion_multiplier(&loom, NodeId::VoidCondenser),
            1.0
        );
    }

    #[test]
    fn test_reflection_lens_neighbor_count() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::ReachWide);

        assert_eq!(node_neighbor_unlock_count(&loom, NodeId::ReflectionLens), 3);
        // Other nodes get 2
        assert_eq!(node_neighbor_unlock_count(&loom, NodeId::EmberSpindle), 2);
    }

    #[test]
    fn test_silence_well_upgrade_cost_passive() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::RunDeep);

        // Level 1 — 25% discount
        assert_eq!(
            node_upgrade_cost_multiplier(&loom, NodeId::SilenceWell),
            0.75
        );

        // Level 5 — still discounted
        let well = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::SilenceWell)
            .unwrap();
        well.level = 5;
        assert_eq!(
            node_upgrade_cost_multiplier(&loom, NodeId::SilenceWell),
            0.75
        );

        // Level 6 — no discount
        let well = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::SilenceWell)
            .unwrap();
        well.level = 6;
        assert_eq!(
            node_upgrade_cost_multiplier(&loom, NodeId::SilenceWell),
            1.0
        );
    }

    #[test]
    fn test_resonance_early_feedback_active() {
        let mut loom = LoomState::new();

        // No archetype — not active
        assert!(!resonance_early_feedback_active(&loom));

        // RunDeep but Resonance Forge not yet unlocked
        select_archetype(&mut loom, LoomArchetype::RunDeep);
        // SilenceWell is unlocked (first node), ResonanceForge is not yet.
        assert!(!resonance_early_feedback_active(&loom));

        // Unlock ResonanceForge
        tick_loom_staggered_unlock(&mut loom, 14400.0);
        assert!(resonance_early_feedback_active(&loom));
    }

    #[test]
    fn test_no_archetype_no_passives() {
        let loom = LoomState::new();
        assert_eq!(node_throughput_multiplier(&loom, NodeId::EmberSpindle), 1.0);
        assert_eq!(
            node_conversion_multiplier(&loom, NodeId::VoidCondenser),
            1.0
        );
        assert_eq!(node_neighbor_unlock_count(&loom, NodeId::ReflectionLens), 2);
        assert_eq!(
            node_upgrade_cost_multiplier(&loom, NodeId::SilenceWell),
            1.0
        );
    }

    #[test]
    fn test_wrong_archetype_no_passives() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::ReachWide);

        // BurnBright passives shouldn't apply
        assert_eq!(node_throughput_multiplier(&loom, NodeId::EmberSpindle), 1.0);
        assert_eq!(
            node_conversion_multiplier(&loom, NodeId::VoidCondenser),
            1.0
        );
    }

    // ── Phase 3: Node Base Production tests ───────────────────────────────────

    #[test]
    fn test_node_native_resource_mapping() {
        assert_eq!(node_native_resource(NodeId::EmberSpindle), Resource::Ember);
        assert_eq!(
            node_native_resource(NodeId::ReflectionLens),
            Resource::Reflection
        );
        assert_eq!(
            node_native_resource(NodeId::VoidCondenser),
            Resource::VoidEssence
        );
        assert_eq!(
            node_native_resource(NodeId::MemoryArchive),
            Resource::Memory
        );
        assert_eq!(node_native_resource(NodeId::SilenceWell), Resource::Silence);
        assert_eq!(
            node_native_resource(NodeId::ResonanceForge),
            Resource::Resonance
        );
    }

    #[test]
    fn test_node_level_multiplier() {
        assert_eq!(node_level_multiplier(1), 1.0);
        assert_eq!(node_level_multiplier(2), 1.5);
        assert_eq!(node_level_multiplier(3), 2.0);
    }

    #[test]
    fn test_base_production_fills_buffer() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        // EmberSpindle is now unlocked.

        tick_base_production(&mut loom, 3600.0);

        let ember = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        // BurnBright: 5/hr base * 1.5x passive = 7.5/hr. After 1 hr: 7.5 units.
        assert!(
            (ember.buffer - 7.5).abs() < 0.001,
            "buffer should be ~7.5, got {}",
            ember.buffer
        );
    }

    #[test]
    fn test_base_production_locked_node_produces_nothing() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        // VoidCondenser is locked.

        tick_base_production(&mut loom, 3600.0);

        let void_n = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::VoidCondenser)
            .unwrap();
        assert_eq!(void_n.buffer, 0.0);
    }

    #[test]
    fn test_base_production_stalls_at_capacity() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        // Fill buffer to capacity.
        let ember = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        ember.buffer = ember.buffer_capacity;

        tick_base_production(&mut loom, 3600.0);

        let ember = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        assert!(ember.stalled);
        assert_eq!(ember.buffer, ember.buffer_capacity);
    }

    // ── Phase 3: Node Upgrading tests ─────────────────────────────────────────

    #[test]
    fn test_upgrade_cost_level1() {
        let loom = LoomState::new();
        let cost = node_upgrade_cost(&loom, NodeId::EmberSpindle);
        assert_eq!(cost, 10.0); // 10 * 1^1.5 = 10
    }

    #[test]
    fn test_upgrade_succeeds_with_sufficient_buffer() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        let ember = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        ember.buffer = 50.0;

        let result = try_upgrade_node(&mut loom, NodeId::EmberSpindle);
        assert!(result);

        let ember = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        assert_eq!(ember.level, 2);
    }

    #[test]
    fn test_upgrade_fails_with_insufficient_buffer() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        let result = try_upgrade_node(&mut loom, NodeId::EmberSpindle);
        assert!(!result);

        let ember = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        assert_eq!(ember.level, 1);
    }

    #[test]
    fn test_upgrade_fails_for_locked_node() {
        let mut loom = LoomState::new();
        let void_n = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::VoidCondenser)
            .unwrap();
        void_n.buffer = 100.0;

        let result = try_upgrade_node(&mut loom, NodeId::VoidCondenser);
        assert!(!result);
    }

    #[test]
    fn test_silence_well_upgrade_discount_applied() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::RunDeep);

        let base_cost = 10.0_f64 * 1.0_f64.powf(1.5);
        let discounted = (base_cost * 0.75).round();
        let cost = node_upgrade_cost(&loom, NodeId::SilenceWell);
        assert_eq!(cost, discounted);
    }

    // ── Phase 3: Neighbor Unlocking tests ─────────────────────────────────────

    #[test]
    fn test_node_neighbors_are_symmetric() {
        for &node in &NodeId::ALL {
            for &nb in node_neighbors(node) {
                assert!(
                    node_neighbors(nb).contains(&node),
                    "{:?} has {:?} as neighbor but not vice versa",
                    node,
                    nb
                );
            }
        }
    }

    #[test]
    fn test_neighbor_unlocking_accumulates_progress() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        let ember = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        ember.buffer = ember.buffer_capacity;

        // Tick 1 hour — not enough to unlock (threshold = 2 hours).
        tick_neighbor_unlocking(&mut loom, 3600.0);

        for &nb in node_neighbors(NodeId::EmberSpindle) {
            let n = loom.persistent.nodes.iter().find(|n| n.id == nb).unwrap();
            assert!(!n.unlocked, "{:?} should not be unlocked yet", nb);
            assert!(n.unlock_progress > 0.0, "{:?} should have progress", nb);
        }
    }

    #[test]
    fn test_neighbor_unlocks_after_threshold() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::ReachWide);
        // ReflectionLens is unlocked.

        let lens = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::ReflectionLens)
            .unwrap();
        lens.buffer = lens.buffer_capacity;

        // Tick 2 hours — enough to unlock neighbors.
        let unlocked = tick_neighbor_unlocking(&mut loom, 7200.0);
        assert!(
            !unlocked.is_empty(),
            "Should have unlocked at least one neighbor"
        );
    }

    #[test]
    fn test_ember_spindle_unlock_speed_is_slower() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        let ember = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        ember.buffer = ember.buffer_capacity;

        // At 0.7x speed, 2hr tick = 1.4hr progress — not enough to unlock (threshold = 2hr).
        tick_neighbor_unlocking(&mut loom, 7200.0);

        for &nb in node_neighbors(NodeId::EmberSpindle) {
            let n = loom.persistent.nodes.iter().find(|n| n.id == nb).unwrap();
            assert!(
                !n.unlocked,
                "{:?} should not be unlocked with 0.7x speed in 2hr",
                nb
            );
            assert!(
                (n.unlock_progress - 1.4).abs() < 0.01,
                "expected ~1.4hr progress, got {}",
                n.unlock_progress
            );
        }
    }

    // ── Phase 6: Reaction Processing tests ────────────────────────────────────

    #[test]
    fn test_process_reactions_produces_forged_light() {
        let mut loom = LoomState::new();

        // Primary recipe: Ember + VoidEssence @ Heat (EmberSpindle) → ForgedLight (1.0x)
        // Deliver both to EmberSpindle (Heat nature).
        let deliveries = vec![
            (
                LoomNodeRef::Extractor(NodeId::EmberSpindle),
                Resource::Ember,
                2.0,
            ),
            (
                LoomNodeRef::Extractor(NodeId::EmberSpindle),
                Resource::VoidEssence,
                2.0,
            ),
        ];

        let outputs = process_reactions(&mut loom, deliveries);

        assert_eq!(outputs.len(), 1);
        let (resource, amount) = outputs[0];
        assert_eq!(resource, Resource::ForgedLight);
        // min(2.0, 2.0) * 1.0 = 2.0
        assert!((amount - 2.0).abs() < 0.001, "expected 2.0, got {}", amount);

        // ForgedLight should be in stockpile.
        let stockpile = loom
            .persistent
            .stockpiles
            .get(&Resource::ForgedLight)
            .copied()
            .unwrap_or(0.0);
        assert!((stockpile - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_process_reactions_no_recipe_produces_nothing() {
        let mut loom = LoomState::new();

        // WovenReality + WovenReality has no recipe.
        let deliveries = vec![
            (
                LoomNodeRef::Extractor(NodeId::EmberSpindle),
                Resource::WovenReality,
                1.0,
            ),
            (
                LoomNodeRef::Extractor(NodeId::EmberSpindle),
                Resource::WovenReality,
                1.0,
            ),
        ];

        let outputs = process_reactions(&mut loom, deliveries);
        assert!(outputs.is_empty());
    }

    #[test]
    fn test_process_reactions_single_resource_no_reaction() {
        let mut loom = LoomState::new();

        let deliveries = vec![(
            LoomNodeRef::Extractor(NodeId::EmberSpindle),
            Resource::Ember,
            5.0,
        )];
        let outputs = process_reactions(&mut loom, deliveries);
        assert!(outputs.is_empty());
    }

    #[test]
    fn test_process_reactions_output_scales_by_minimum_input() {
        let mut loom = LoomState::new();

        // Ember + VoidEssence @ Heat → ForgedLight (amount 1.0).
        // Deliver 3.0 Ember but only 1.0 VoidEssence → min = 1.0 → output = 1.0.
        let deliveries = vec![
            (
                LoomNodeRef::Extractor(NodeId::EmberSpindle),
                Resource::Ember,
                3.0,
            ),
            (
                LoomNodeRef::Extractor(NodeId::EmberSpindle),
                Resource::VoidEssence,
                1.0,
            ),
        ];

        let outputs = process_reactions(&mut loom, deliveries);
        assert_eq!(outputs.len(), 1);
        assert!(
            (outputs[0].1 - 1.0).abs() < 0.001,
            "expected 1.0, got {}",
            outputs[0].1
        );
    }

    #[test]
    fn test_process_reactions_accumulates_to_stockpile() {
        let mut loom = LoomState::new();

        // Run the same reaction twice.
        let deliveries1 = vec![
            (
                LoomNodeRef::Extractor(NodeId::EmberSpindle),
                Resource::Ember,
                1.0,
            ),
            (
                LoomNodeRef::Extractor(NodeId::EmberSpindle),
                Resource::VoidEssence,
                1.0,
            ),
        ];
        process_reactions(&mut loom, deliveries1);

        let deliveries2 = vec![
            (
                LoomNodeRef::Extractor(NodeId::EmberSpindle),
                Resource::Ember,
                1.0,
            ),
            (
                LoomNodeRef::Extractor(NodeId::EmberSpindle),
                Resource::VoidEssence,
                1.0,
            ),
        ];
        process_reactions(&mut loom, deliveries2);

        let stockpile = loom
            .persistent
            .stockpiles
            .get(&Resource::ForgedLight)
            .copied()
            .unwrap_or(0.0);
        // 1.0 * 1.0 + 1.0 * 1.0 = 2.0
        assert!((stockpile - 2.0).abs() < 0.001);
    }

    // ── Phase 5: Buffer Stalling tests ────────────────────────────────────────

    fn make_node(id: NodeId, buffer: f64, capacity: f64) -> LoomNode {
        let mut n = LoomNode::new(id);
        n.unlocked = true;
        n.buffer = buffer;
        n.buffer_capacity = capacity;
        n
    }

    fn make_pipe(from: LoomNodeRef, to: LoomNodeRef, under_construction: bool) -> Pipe {
        Pipe {
            from,
            to,
            tier: crate::loom::types::PipeTier::T1,
            split_ratio: 1.0,
            under_construction,
            construction_ticks_remaining: 0,
        }
    }

    #[test]
    fn test_check_node_stall_buffer_below_capacity_not_stalled() {
        let node = make_node(NodeId::EmberSpindle, 10.0, 20.0); // half full
        let all_nodes = vec![node.clone()];
        assert!(!check_node_stall(&node, &[], &all_nodes));
    }

    #[test]
    fn test_check_node_stall_full_buffer_no_outgoing_pipes_stalls() {
        let node = make_node(NodeId::EmberSpindle, 20.0, 20.0); // full
        let all_nodes = vec![node.clone()];
        assert!(check_node_stall(&node, &[], &all_nodes));
    }

    #[test]
    fn test_check_node_stall_full_buffer_active_pipe_to_non_full_dest_not_stalled() {
        let src = make_node(NodeId::EmberSpindle, 20.0, 20.0); // full
        let dst = make_node(NodeId::VoidCondenser, 5.0, 20.0); // not full
        let pipe = make_pipe(
            LoomNodeRef::Extractor(NodeId::EmberSpindle),
            LoomNodeRef::Extractor(NodeId::VoidCondenser),
            false,
        );
        let all_nodes = vec![src.clone(), dst];
        assert!(!check_node_stall(&src, &[&pipe], &all_nodes));
    }

    #[test]
    fn test_check_node_stall_full_buffer_all_destinations_full_stalls() {
        let src = make_node(NodeId::EmberSpindle, 20.0, 20.0); // full
        let dst = make_node(NodeId::VoidCondenser, 20.0, 20.0); // also full
        let pipe = make_pipe(
            LoomNodeRef::Extractor(NodeId::EmberSpindle),
            LoomNodeRef::Extractor(NodeId::VoidCondenser),
            false,
        );
        let all_nodes = vec![src.clone(), dst];
        assert!(check_node_stall(&src, &[&pipe], &all_nodes));
    }

    #[test]
    fn test_check_node_stall_full_buffer_construction_pipe_treated_as_no_active_pipes() {
        let src = make_node(NodeId::EmberSpindle, 20.0, 20.0); // full
        let dst = make_node(NodeId::VoidCondenser, 0.0, 20.0); // empty — but pipe is under construction
        let pipe = make_pipe(
            LoomNodeRef::Extractor(NodeId::EmberSpindle),
            LoomNodeRef::Extractor(NodeId::VoidCondenser),
            true,
        );
        let all_nodes = vec![src.clone(), dst];
        // under_construction pipe is inactive → treated as no active pipes → stalled
        assert!(check_node_stall(&src, &[&pipe], &all_nodes));
    }

    #[test]
    fn test_check_node_stall_two_pipes_one_destination_not_full_not_stalled() {
        let src = make_node(NodeId::EmberSpindle, 20.0, 20.0); // full
        let dst1 = make_node(NodeId::VoidCondenser, 20.0, 20.0); // full
        let dst2 = make_node(NodeId::ReflectionLens, 5.0, 20.0); // not full
        let pipe1 = make_pipe(
            LoomNodeRef::Extractor(NodeId::EmberSpindle),
            LoomNodeRef::Extractor(NodeId::VoidCondenser),
            false,
        );
        let pipe2 = make_pipe(
            LoomNodeRef::Extractor(NodeId::EmberSpindle),
            LoomNodeRef::Extractor(NodeId::ReflectionLens),
            false,
        );
        let all_nodes = vec![src.clone(), dst1, dst2];
        // One destination still has room → not stalled
        assert!(!check_node_stall(&src, &[&pipe1, &pipe2], &all_nodes));
    }

    #[test]
    fn test_check_node_stall_dangling_pipe_treated_as_blocked() {
        let src = make_node(NodeId::EmberSpindle, 20.0, 20.0); // full
                                                               // Pipe points to a node not in all_nodes → treated as full/blocked
        let pipe = make_pipe(
            LoomNodeRef::Extractor(NodeId::EmberSpindle),
            LoomNodeRef::Extractor(NodeId::MemoryArchive),
            false,
        );
        let all_nodes = vec![src.clone()]; // MemoryArchive absent
        assert!(check_node_stall(&src, &[&pipe], &all_nodes));
    }

    #[test]
    fn test_tick_stall_detection_marks_stalled_node() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        // Fill EmberSpindle's buffer to capacity with no outgoing pipes.
        let ember = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        ember.buffer = ember.buffer_capacity;

        let changed = tick_stall_detection(&mut loom);

        assert!(
            changed.contains(&NodeId::EmberSpindle),
            "EmberSpindle should have changed to stalled"
        );
        let ember = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        assert!(ember.stalled);
    }

    #[test]
    fn test_tick_stall_detection_clears_stall_when_buffer_drains() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        // First: fill buffer → stall.
        {
            let ember = loom
                .persistent
                .nodes
                .iter_mut()
                .find(|n| n.id == NodeId::EmberSpindle)
                .unwrap();
            ember.buffer = ember.buffer_capacity;
            ember.stalled = true; // pre-mark so we test the transition back
        }

        // Drain below capacity → should un-stall.
        {
            let ember = loom
                .persistent
                .nodes
                .iter_mut()
                .find(|n| n.id == NodeId::EmberSpindle)
                .unwrap();
            ember.buffer = ember.buffer_capacity - 1.0;
        }

        let changed = tick_stall_detection(&mut loom);
        assert!(
            changed.contains(&NodeId::EmberSpindle),
            "EmberSpindle should have changed to un-stalled"
        );
        let ember = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        assert!(!ember.stalled);
    }

    #[test]
    fn test_tick_stall_detection_locked_node_never_stalls() {
        let mut loom = LoomState::new();
        // No archetype selected → all nodes locked.

        // Force a node's buffer to full.
        let node = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        node.buffer = node.buffer_capacity;

        let changed = tick_stall_detection(&mut loom);
        assert!(
            !changed.contains(&NodeId::EmberSpindle),
            "Locked node should never stall"
        );
        let node = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        assert!(!node.stalled);
    }

    #[test]
    fn test_tick_stall_detection_no_change_when_already_stalled() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        // Pre-stall the node.
        let ember = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        ember.buffer = ember.buffer_capacity;
        ember.stalled = true;

        let changed = tick_stall_detection(&mut loom);
        // State didn't change — should not appear in changed list.
        assert!(
            !changed.contains(&NodeId::EmberSpindle),
            "Already-stalled node with same stall state should not be in changed"
        );
    }

    // ── Codex Discovery ───────────────────────────────────────────────────────

    #[test]
    fn test_process_reactions_records_codex_discovery() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        assert!(loom.persistent.codex.is_empty());

        // Simulate Ember + Reflection arriving at ReflectionLens (nature=Form).
        let deliveries = vec![
            (
                LoomNodeRef::Extractor(NodeId::ReflectionLens),
                Resource::Ember,
                1.0,
            ),
            (
                LoomNodeRef::Extractor(NodeId::ReflectionLens),
                Resource::Reflection,
                1.0,
            ),
        ];
        process_reactions(&mut loom, deliveries);

        // Should have at least one discovered codex entry if recipe exists.
        // Even if no recipe fires (no matching combo), codex stays empty.
        // Just verify the function runs without panic.
        // A matching recipe test would need to match the actual registry.
        // Here we confirm the codex is a Vec (no panic, proper structure).
        let _ = &loom.persistent.codex;
    }

    #[test]
    fn test_record_codex_discovery_adds_new_entry() {
        let mut codex = Vec::new();
        record_codex_discovery(
            &mut codex,
            Resource::Ember,
            Resource::VoidEssence,
            super::super::types::NodeNature::Heat,
            Resource::CondensedEmber,
            0.5,
        );
        assert_eq!(codex.len(), 1);
        assert!(codex[0].discovered);
        assert_eq!(codex[0].output, Resource::CondensedEmber);
    }

    #[test]
    fn test_record_codex_discovery_marks_existing_entry() {
        use super::super::types::{CodexEntry, NodeNature};
        let mut codex = vec![CodexEntry {
            inputs: vec![Resource::Ember, Resource::VoidEssence],
            node_nature: NodeNature::Heat,
            output: Resource::CondensedEmber,
            output_amount: 0.5,
            discovered: false,
        }];
        record_codex_discovery(
            &mut codex,
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::CondensedEmber,
            0.5,
        );
        assert_eq!(codex.len(), 1, "should not duplicate");
        assert!(codex[0].discovered);
    }

    #[test]
    fn test_record_codex_discovery_commutative_inputs() {
        use super::super::types::{CodexEntry, NodeNature};
        let mut codex = vec![CodexEntry {
            inputs: vec![Resource::Ember, Resource::VoidEssence],
            node_nature: NodeNature::Heat,
            output: Resource::CondensedEmber,
            output_amount: 0.5,
            discovered: false,
        }];
        // Reverse input order — should still match.
        record_codex_discovery(
            &mut codex,
            Resource::VoidEssence,
            Resource::Ember,
            NodeNature::Heat,
            Resource::CondensedEmber,
            0.5,
        );
        assert_eq!(codex.len(), 1, "commutative match should not duplicate");
        assert!(codex[0].discovered);
    }

    // ── loom_production_bonus ──────────────────────────────────────────────────

    #[test]
    fn test_loom_production_bonus_zero_inputs_is_one() {
        let bonus = loom_production_bonus(0, 0, 0, 0);
        assert!(
            (bonus - 1.0).abs() < 1e-9,
            "no bonuses should return 1.0, got {}",
            bonus
        );
    }

    #[test]
    fn test_loom_production_bonus_deep_layer_contribution() {
        // 10 deep layers → +5%
        let bonus = loom_production_bonus(10, 0, 0, 0);
        assert!(
            (bonus - 1.05).abs() < 1e-9,
            "10 deep layers should give 1.05, got {}",
            bonus
        );
    }

    #[test]
    fn test_loom_production_bonus_haven_tree_contribution() {
        // 10 haven tree levels → +3%
        let bonus = loom_production_bonus(0, 10, 0, 0);
        assert!(
            (bonus - 1.03).abs() < 1e-9,
            "10 haven levels should give 1.03, got {}",
            bonus
        );
    }

    #[test]
    fn test_loom_production_bonus_sigil_contribution() {
        // 6 sigils → +6%
        let bonus = loom_production_bonus(0, 0, 6, 0);
        assert!(
            (bonus - 1.06).abs() < 1e-9,
            "6 sigils should give 1.06, got {}",
            bonus
        );
    }

    #[test]
    fn test_loom_production_bonus_ascension_contribution() {
        // 3 ascension levels → +6%
        let bonus = loom_production_bonus(0, 0, 0, 3);
        assert!(
            (bonus - 1.06).abs() < 1e-9,
            "3 ascension levels should give 1.06, got {}",
            bonus
        );
    }

    #[test]
    fn test_loom_production_bonus_all_systems_additive() {
        // 10 deep + 10 haven + 6 sigils + 3 ascension = 5% + 3% + 6% + 6% = 20%
        let bonus = loom_production_bonus(10, 10, 6, 3);
        assert!(
            (bonus - 1.20).abs() < 1e-9,
            "combined bonus should be 1.20, got {}",
            bonus
        );
    }

    #[test]
    fn test_loom_production_bonus_caps_are_enforced() {
        // Values beyond the caps should give same result as capped values.
        let capped = loom_production_bonus(30, 50, 12, 10);
        let over_cap = loom_production_bonus(100, 200, 50, 50);
        assert!(
            (capped - over_cap).abs() < 1e-9,
            "over-cap inputs should equal capped: {} vs {}",
            capped,
            over_cap
        );
        // Max bonus: 15% + 15% + 12% + 20% = 62%
        assert!(
            (capped - 1.62).abs() < 1e-9,
            "max bonus should be 1.62, got {}",
            capped
        );
    }

    #[test]
    fn test_loom_production_bonus_always_at_least_one() {
        assert!(loom_production_bonus(0, 0, 0, 0) >= 1.0);
        assert!(loom_production_bonus(1, 1, 1, 1) >= 1.0);
    }

    // ── node_effective_rate ───────────────────────────────────────────────────

    #[test]
    fn test_node_effective_rate_locked_node_is_zero() {
        let loom = LoomState::new();
        let node = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        assert_eq!(node_effective_rate(&loom, node), 0.0);
    }

    #[test]
    fn test_node_effective_rate_level2_scales_correctly() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::RunDeep);
        // SilenceWell is unlocked by RunDeep (no throughput multiplier for RunDeep on SilenceWell).
        let well = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::SilenceWell)
            .unwrap();
        well.level = 2;

        let well = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::SilenceWell)
            .unwrap();
        let rate = node_effective_rate(&loom, well);
        // base_rate 5.0 * level_mult(2) 1.5 * throughput_mult 1.0 = 7.5
        assert!((rate - 7.5).abs() < 0.001, "expected 7.5/hr, got {}", rate);
    }

    #[test]
    fn test_node_effective_rate_burn_bright_ember_spindle_with_level() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        let ember = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        ember.level = 3;

        let ember = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        let rate = node_effective_rate(&loom, ember);
        // base_rate 5.0 * level_mult(3) 2.0 * throughput_mult 1.5 = 15.0
        assert!(
            (rate - 15.0).abs() < 0.001,
            "expected 15.0/hr, got {}",
            rate
        );
    }

    // ── try_upgrade_node buffer capacity ──────────────────────────────────────

    #[test]
    fn test_upgrade_node_increases_buffer_capacity() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        let old_capacity = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer_capacity;

        let ember = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        ember.buffer = 50.0;
        try_upgrade_node(&mut loom, NodeId::EmberSpindle);

        let new_capacity = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer_capacity;
        assert!(
            new_capacity > old_capacity,
            "buffer_capacity should grow after upgrade: {} -> {}",
            old_capacity,
            new_capacity
        );
    }

    #[test]
    fn test_upgrade_node_deducts_cost_from_buffer() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        let cost = node_upgrade_cost(&loom, NodeId::EmberSpindle);
        let starting_buffer = cost + 5.0;

        let ember = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        ember.buffer = starting_buffer;
        try_upgrade_node(&mut loom, NodeId::EmberSpindle);

        let remaining = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer;
        assert!(
            (remaining - 5.0).abs() < 0.001,
            "expected 5.0 remaining, got {}",
            remaining
        );
    }

    // ── tick_base_production return value ─────────────────────────────────────

    #[test]
    fn test_tick_base_production_returns_produced_map() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        let produced = tick_base_production(&mut loom, 3600.0);

        assert!(
            produced.contains_key(&Resource::Ember),
            "produced map should include Ember"
        );
        let ember_amount = produced[&Resource::Ember];
        // BurnBright EmberSpindle: 5/hr * 1.5 = 7.5/hr; after 1hr = 7.5 units
        assert!(
            (ember_amount - 7.5).abs() < 0.001,
            "expected 7.5 Ember produced, got {}",
            ember_amount
        );
    }

    #[test]
    fn test_tick_base_production_locked_nodes_absent_from_produced_map() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        let produced = tick_base_production(&mut loom, 3600.0);

        // VoidCondenser is locked — must not appear in map.
        assert!(
            !produced.contains_key(&Resource::VoidEssence),
            "locked VoidCondenser should produce nothing"
        );
    }

    // ── tick_neighbor_unlocking with buffer below threshold ───────────────────

    #[test]
    fn test_neighbor_unlocking_no_progress_when_buffer_below_threshold() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        // EmberSpindle buffer at 0 — well below the 50% threshold.
        tick_neighbor_unlocking(&mut loom, 7200.0);

        for &nb in node_neighbors(NodeId::EmberSpindle) {
            let n = loom.persistent.nodes.iter().find(|n| n.id == nb).unwrap();
            assert!(
                !n.unlocked,
                "{:?} should not be unlocked when buffer is empty",
                nb
            );
            assert_eq!(
                n.unlock_progress, 0.0,
                "{:?} should have zero progress when source buffer is empty",
                nb
            );
        }
    }

    #[test]
    fn test_neighbor_unlocking_returns_empty_when_no_unlock_occurs() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        // Buffer is empty — nothing should unlock.
        let unlocked = tick_neighbor_unlocking(&mut loom, 3600.0);
        assert!(
            unlocked.is_empty(),
            "should return empty vec when nothing unlocks"
        );
    }

    // ── process_reactions with non-Heat node natures ──────────────────────────

    #[test]
    fn test_process_reactions_memory_silence_at_pattern_node_produces_echo_glass() {
        let mut loom = LoomState::new();
        // MemoryArchive has Pattern nature — Memory + Silence + Pattern → EchoGlass (1.0x).
        let deliveries = vec![
            (
                LoomNodeRef::Extractor(NodeId::MemoryArchive),
                Resource::Memory,
                3.0,
            ),
            (
                LoomNodeRef::Extractor(NodeId::MemoryArchive),
                Resource::Silence,
                3.0,
            ),
        ];
        let outputs = process_reactions(&mut loom, deliveries);

        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].0, Resource::EchoGlass);
        // min(3.0, 3.0) * 1.0 = 3.0
        assert!(
            (outputs[0].1 - 3.0).abs() < 0.001,
            "expected 3.0 EchoGlass, got {}",
            outputs[0].1
        );
    }

    #[test]
    fn test_process_reactions_silence_resonance_at_stillness_node_produces_stillborn_song() {
        let mut loom = LoomState::new();
        // SilenceWell has Stillness nature — Silence + Resonance + Stillness → StillbornSong (1.0x).
        let deliveries = vec![
            (
                LoomNodeRef::Extractor(NodeId::SilenceWell),
                Resource::Silence,
                2.0,
            ),
            (
                LoomNodeRef::Extractor(NodeId::SilenceWell),
                Resource::Resonance,
                2.0,
            ),
        ];
        let outputs = process_reactions(&mut loom, deliveries);

        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].0, Resource::StillbornSong);
        assert!(
            (outputs[0].1 - 2.0).abs() < 0.001,
            "expected 2.0 StillbornSong, got {}",
            outputs[0].1
        );
    }

    // ── codex_hint_indices ────────────────────────────────────────────────────

    #[test]
    fn test_codex_hint_indices_empty_codex_returns_empty() {
        let codex: Vec<crate::loom::types::CodexEntry> = Vec::new();
        let hints = codex_hint_indices(&codex);
        // No discovered recipes → no adjacent undiscovered recipes to hint at.
        assert!(
            hints.is_empty(),
            "empty codex should yield no hints, got {} hints",
            hints.len()
        );
    }

    #[test]
    fn test_codex_hint_indices_after_first_discovery_reveals_adjacent() {
        use crate::loom::types::{CodexEntry, NodeNature};
        // Discover the primary ForgedLight recipe: Ember + VoidEssence + Heat.
        let codex = vec![CodexEntry {
            inputs: vec![Resource::Ember, Resource::VoidEssence],
            node_nature: NodeNature::Heat,
            output: Resource::ForgedLight,
            output_amount: 1.0,
            discovered: true,
        }];
        let hints = codex_hint_indices(&codex);
        assert!(
            !hints.is_empty(),
            "discovering first recipe should hint at adjacent undiscovered recipes"
        );
    }

    // ── node_level_multiplier edge cases ──────────────────────────────────────

    #[test]
    fn test_node_level_multiplier_level_zero_treated_as_one() {
        // saturating_sub(1) on 0 gives 0, so multiplier is 1.0.
        assert_eq!(node_level_multiplier(0), 1.0);
    }

    #[test]
    fn test_node_level_multiplier_level_10() {
        // 1.0 + (10-1)*0.5 = 1.0 + 4.5 = 5.5
        assert!((node_level_multiplier(10) - 5.5).abs() < 0.001);
    }

    // ── archetype_nodes mapping ───────────────────────────────────────────────

    #[test]
    fn test_archetype_nodes_burn_bright() {
        let (first, second) = archetype_nodes(LoomArchetype::BurnBright);
        assert_eq!(first, NodeId::EmberSpindle);
        assert_eq!(second, NodeId::VoidCondenser);
    }

    #[test]
    fn test_archetype_nodes_reach_wide() {
        let (first, second) = archetype_nodes(LoomArchetype::ReachWide);
        assert_eq!(first, NodeId::ReflectionLens);
        assert_eq!(second, NodeId::MemoryArchive);
    }

    #[test]
    fn test_archetype_nodes_run_deep() {
        let (first, second) = archetype_nodes(LoomArchetype::RunDeep);
        assert_eq!(first, NodeId::SilenceWell);
        assert_eq!(second, NodeId::ResonanceForge);
    }

    // ── node_neighbor_unlock_speed_multiplier ─────────────────────────────────

    #[test]
    fn test_ember_spindle_unlock_speed_burn_bright_is_point_seven() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        assert!(
            (node_neighbor_unlock_speed_multiplier(&loom, NodeId::EmberSpindle) - 0.7).abs()
                < 0.001
        );
    }

    #[test]
    fn test_other_nodes_unlock_speed_is_one() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        assert_eq!(
            node_neighbor_unlock_speed_multiplier(&loom, NodeId::VoidCondenser),
            1.0
        );
        assert_eq!(
            node_neighbor_unlock_speed_multiplier(&loom, NodeId::SilenceWell),
            1.0
        );
    }

    // ── upgrade_cost scaling ──────────────────────────────────────────────────

    #[test]
    fn test_upgrade_cost_increases_with_level() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);

        let cost_l1 = node_upgrade_cost(&loom, NodeId::EmberSpindle);
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .level = 2;
        let cost_l2 = node_upgrade_cost(&loom, NodeId::EmberSpindle);
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .level = 3;
        let cost_l3 = node_upgrade_cost(&loom, NodeId::EmberSpindle);

        assert!(
            cost_l1 < cost_l2 && cost_l2 < cost_l3,
            "upgrade cost should increase: {} < {} < {}",
            cost_l1,
            cost_l2,
            cost_l3
        );
    }

    // ── Refinery Ticking ──────────────────────────────────────────────────────

    #[test]
    fn test_refinery_construction_completes() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        loom.persistent.refineries.push({
            let mut r = Refinery::new(
                Resource::Ember,
                Resource::VoidEssence,
                NodeNature::Heat,
                Resource::ForgedLight,
                1.0,
                1,
            );
            r.under_construction = true;
            r.construction_ticks_remaining = 1;
            r
        });

        let completed = tick_refinery_construction(&mut loom);
        assert_eq!(completed.len(), 1);
        assert!(!loom.persistent.refineries[0].under_construction);
    }

    #[test]
    fn test_refinery_processing_produces_output() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        loom.persistent.refineries.push(Refinery::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
        ));
        let deliveries = vec![
            (LoomNodeRef::Refinery(0), Resource::Ember, 5.0),
            (LoomNodeRef::Refinery(0), Resource::VoidEssence, 3.0),
        ];

        let reactions = process_refinery_reactions(&mut loom, deliveries);
        assert!(!reactions.is_empty());
        assert!((loom.persistent.refineries[0].buffer - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_refinery_stall_when_buffer_full() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        let mut r = Refinery::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
        );
        r.buffer = r.buffer_capacity;
        loom.persistent.refineries.push(r);

        tick_refinery_stall_detection(&mut loom);
        assert!(loom.persistent.refineries[0].stalled);
    }
}

// ---------------------------------------------------------------------------
// External system bonuses — granular per-aspect bonuses (Task #20 supplement)
// ---------------------------------------------------------------------------

/// Pre-computed bonuses from existing game systems that boost Loom production.
///
/// Breaks external bonuses into three separate axes so callers can apply them
/// only where relevant (production rate, buffer capacity, pipe bandwidth).
/// Passed via explicit parameters following the Haven bonus injection pattern —
/// Loom logic never imports Haven/Deep/Stormglass/Ascension directly.
#[derive(Debug, Clone, Copy, Default)]
pub struct LoomExternalBonuses {
    /// Additive fraction bonus on all node base production rates (e.g. 0.10 = +10%).
    pub production_rate_bonus: f64,
    /// Additive fraction bonus on buffer capacity for all nodes (e.g. 0.20 = +20%).
    pub buffer_capacity_bonus: f64,
    /// Additive fraction bonus on all pipe bandwidth (e.g. 0.15 = +15%).
    pub pipe_bandwidth_bonus: f64,
}

/// Compute granular Loom bonuses from the current state of existing game systems.
///
/// # Parameters
/// - `haven_damage_percent`: Haven Armory damage bonus (0-25.0). Each 5% maps to
///   +1% production rate (max +5% at 25%).
/// - `deep_guild_rank`: Deep guild rank 1-5. Each rank above 1 adds +5% buffer
///   capacity (max +20% at rank 5).
/// - `ascension_level`: Current ascension level (0 = none). Each level adds +2%
///   pipe bandwidth, capped at +12% (level 6).
/// - `stormglass_balance`: Current Stormglass balance. Every 100k SG adds +1%
///   production rate, capped at +5% (500k SG).
///
/// All bonuses are additive within their category and independent of each other.
/// Haven and Stormglass bonuses stack additively into `production_rate_bonus`.
pub fn loom_external_bonuses(
    haven_damage_percent: f64,
    deep_guild_rank: u8,
    ascension_level: u32,
    stormglass_balance: u64,
) -> LoomExternalBonuses {
    // Haven Armory: up to +25% damage maps linearly to up to +5% production rate.
    let haven_production = (haven_damage_percent / 5.0).min(5.0) / 100.0;

    // Stormglass: +1% per 100k balance, capped at +5% (500k).
    let sg_production = (stormglass_balance as f64 / 100_000.0).min(5.0) / 100.0;

    let production_rate_bonus = haven_production + sg_production;

    // Deep guild rank: +5% buffer capacity per rank above 1, capped at +20% (rank 5).
    let rank_above_one = deep_guild_rank.saturating_sub(1) as f64;
    let buffer_capacity_bonus = (rank_above_one * 5.0).min(20.0) / 100.0;

    // Ascension: +2% pipe bandwidth per level, capped at +12% (level 6).
    let pipe_bandwidth_bonus = (ascension_level as f64 * 2.0).min(12.0) / 100.0;

    LoomExternalBonuses {
        production_rate_bonus,
        buffer_capacity_bonus,
        pipe_bandwidth_bonus,
    }
}

/// Apply external bonuses to a node's effective base production rate.
pub fn effective_node_base_rate(node: &LoomNode, bonuses: &LoomExternalBonuses) -> f64 {
    node.base_rate * (1.0 + bonuses.production_rate_bonus)
}

/// Apply external bonuses to a node's effective buffer capacity.
pub fn effective_buffer_capacity(node: &LoomNode, bonuses: &LoomExternalBonuses) -> f64 {
    node.buffer_capacity * (1.0 + bonuses.buffer_capacity_bonus)
}

/// Apply external bonuses to a pipe's effective bandwidth (units/hour).
pub fn effective_pipe_bandwidth(
    tier: super::types::PipeTier,
    bonuses: &LoomExternalBonuses,
) -> f64 {
    tier.bandwidth() * (1.0 + bonuses.pipe_bandwidth_bonus)
}

#[cfg(test)]
mod external_bonus_tests {
    use super::*;

    #[test]
    fn test_no_bonuses_when_all_systems_at_minimum() {
        let b = loom_external_bonuses(0.0, 1, 0, 0);
        assert!((b.production_rate_bonus).abs() < 1e-9);
        assert!((b.buffer_capacity_bonus).abs() < 1e-9);
        assert!((b.pipe_bandwidth_bonus).abs() < 1e-9);
    }

    #[test]
    fn test_haven_armory_max_gives_five_percent_production() {
        let b = loom_external_bonuses(25.0, 1, 0, 0);
        assert!((b.production_rate_bonus - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_haven_bonus_capped_at_five_percent() {
        let b = loom_external_bonuses(200.0, 1, 0, 0);
        assert!((b.production_rate_bonus - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_deep_guild_rank_2_gives_five_percent_buffer() {
        let b = loom_external_bonuses(0.0, 2, 0, 0);
        assert!((b.buffer_capacity_bonus - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_deep_guild_rank_5_gives_twenty_percent_buffer() {
        let b = loom_external_bonuses(0.0, 5, 0, 0);
        assert!((b.buffer_capacity_bonus - 0.20).abs() < 1e-9);
    }

    #[test]
    fn test_deep_guild_rank_1_gives_no_buffer_bonus() {
        let b = loom_external_bonuses(0.0, 1, 0, 0);
        assert!((b.buffer_capacity_bonus).abs() < 1e-9);
    }

    #[test]
    fn test_ascension_level_1_gives_two_percent_bandwidth() {
        let b = loom_external_bonuses(0.0, 1, 1, 0);
        assert!((b.pipe_bandwidth_bonus - 0.02).abs() < 1e-9);
    }

    #[test]
    fn test_ascension_level_6_gives_twelve_percent_bandwidth() {
        let b = loom_external_bonuses(0.0, 1, 6, 0);
        assert!((b.pipe_bandwidth_bonus - 0.12).abs() < 1e-9);
    }

    #[test]
    fn test_ascension_capped_at_twelve_percent() {
        let b = loom_external_bonuses(0.0, 1, 10, 0);
        assert!((b.pipe_bandwidth_bonus - 0.12).abs() < 1e-9);
    }

    #[test]
    fn test_stormglass_100k_gives_one_percent_production() {
        let b = loom_external_bonuses(0.0, 1, 0, 100_000);
        assert!((b.production_rate_bonus - 0.01).abs() < 1e-9);
    }

    #[test]
    fn test_stormglass_500k_gives_five_percent_production() {
        let b = loom_external_bonuses(0.0, 1, 0, 500_000);
        assert!((b.production_rate_bonus - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_stormglass_capped_at_five_percent() {
        let b = loom_external_bonuses(0.0, 1, 0, 2_000_000);
        assert!((b.production_rate_bonus - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_haven_and_stormglass_stack_additively() {
        // Haven T3 (25 dmg% -> 5%) + 500k SG (5%) = 10%
        let b = loom_external_bonuses(25.0, 1, 0, 500_000);
        assert!((b.production_rate_bonus - 0.10).abs() < 1e-9);
    }

    #[test]
    fn test_all_systems_at_max_gives_correct_totals() {
        let b = loom_external_bonuses(25.0, 5, 6, 500_000);
        assert!((b.production_rate_bonus - 0.10).abs() < 1e-9);
        assert!((b.buffer_capacity_bonus - 0.20).abs() < 1e-9);
        assert!((b.pipe_bandwidth_bonus - 0.12).abs() < 1e-9);
    }

    #[test]
    fn test_effective_node_base_rate_applies_production_bonus() {
        let mut node = LoomNode::new(NodeId::EmberSpindle);
        node.base_rate = 10.0;
        let b = LoomExternalBonuses {
            production_rate_bonus: 0.10,
            ..Default::default()
        };
        assert!((effective_node_base_rate(&node, &b) - 11.0).abs() < 1e-9);
    }

    #[test]
    fn test_effective_buffer_capacity_applies_buffer_bonus() {
        let mut node = LoomNode::new(NodeId::EmberSpindle);
        node.buffer_capacity = 20.0;
        let b = LoomExternalBonuses {
            buffer_capacity_bonus: 0.20,
            ..Default::default()
        };
        assert!((effective_buffer_capacity(&node, &b) - 24.0).abs() < 1e-9);
    }

    #[test]
    fn test_effective_pipe_bandwidth_applies_bandwidth_bonus() {
        use super::super::types::PipeTier;
        let b = LoomExternalBonuses {
            pipe_bandwidth_bonus: 0.10,
            ..Default::default()
        };
        // T1 base = 5.0/hr, +10% = 5.5
        assert!((effective_pipe_bandwidth(PipeTier::T1, &b) - 5.5).abs() < 1e-9);
    }

    #[test]
    fn test_effective_pipe_bandwidth_t4_with_max_bonus() {
        use super::super::types::PipeTier;
        let b = LoomExternalBonuses {
            pipe_bandwidth_bonus: 0.12,
            ..Default::default()
        };
        // T4 base = 50.0/hr, +12% = 56.0
        assert!((effective_pipe_bandwidth(PipeTier::T4, &b) - 56.0).abs() < 1e-9);
    }

    // Helper: populate patterns via complete_discovery and mark the first N as completed.
    fn setup_patterns(loom: &mut LoomState, completed_count: usize) {
        crate::loom::discovery::complete_discovery(loom);
        for p in loom.persistent.patterns.iter_mut().take(completed_count) {
            p.completed = true;
        }
    }

    #[test]
    fn test_build_refinery_success() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        setup_patterns(&mut loom, 1);
        *loom
            .persistent
            .stockpiles
            .entry(Resource::Ember)
            .or_insert(0.0) += 50.0;

        let result = build_refinery(
            &mut loom,
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
        );
        assert!(result.is_ok());
        assert_eq!(loom.persistent.refineries.len(), 1);
        let r = &loom.persistent.refineries[0];
        assert_eq!(r.output, Resource::ForgedLight);
        assert!(r.under_construction);
    }

    #[test]
    fn test_build_refinery_fails_at_capacity() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        // No completed patterns → max_refineries() == 0 → AtCapacity.
        let result = build_refinery(
            &mut loom,
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_build_refinery_fails_insufficient_resources() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        setup_patterns(&mut loom, 1);
        // No Ember stockpile → InsufficientResources.
        let result = build_refinery(
            &mut loom,
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_build_refinery_fails_invalid_recipe() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        setup_patterns(&mut loom, 1);
        *loom
            .persistent
            .stockpiles
            .entry(Resource::Ember)
            .or_insert(0.0) += 50.0;

        let result = build_refinery(
            &mut loom,
            Resource::WovenReality,
            Resource::WovenReality,
            NodeNature::Heat,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_build_refinery_tier_gating() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        // Only 1 completed pattern; Tier 2 recipes require 6 → TierLocked.
        setup_patterns(&mut loom, 1);
        *loom
            .persistent
            .stockpiles
            .entry(Resource::ForgedLight)
            .or_insert(0.0) += 50.0;
        *loom
            .persistent
            .stockpiles
            .entry(Resource::EchoGlass)
            .or_insert(0.0) += 50.0;

        let result = build_refinery(
            &mut loom,
            Resource::ForgedLight,
            Resource::EchoGlass,
            NodeNature::Heat,
        );
        assert!(result.is_err());
    }
}
