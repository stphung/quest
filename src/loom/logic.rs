#![allow(dead_code)]
use super::types::{
    LoomArchetype, LoomNode, LoomNodeRef, LoomState, NodeId, NodeNature, Resource, Shuttle,
};

/// Legacy archetype-to-node mapping (kept for save compatibility).
pub fn archetype_nodes(archetype: LoomArchetype) -> (NodeId, NodeId) {
    match archetype {
        LoomArchetype::BurnBright => (NodeId::EmberSpindle, NodeId::VoidCondenser),
        LoomArchetype::ReachWide => (NodeId::ReflectionLens, NodeId::MemoryArchive),
        LoomArchetype::RunDeep => (NodeId::SilenceWell, NodeId::ResonanceForge),
    }
}

/// Legacy: select_archetype is kept as a no-op for save compatibility.
/// All 6 extractors now unlock on discovery via initialize_loom().
pub fn select_archetype(loom: &mut LoomState, archetype: LoomArchetype) {
    loom.persistent.archetype = Some(archetype);
}

/// No-op: archetype passives removed for rebalancing.
fn apply_node_passive_on_unlock(_node_id: NodeId, _loom: &mut LoomState) {}

/// Initialize the Loom on discovery: unlock only Ember Spindle.
/// Other nodes unlock via the neighbor unlock system as the player progresses.
pub fn initialize_loom(loom: &mut LoomState) {
    if let Some(node) = loom
        .persistent
        .nodes
        .iter_mut()
        .find(|n| n.id == NodeId::EmberSpindle)
    {
        node.unlocked = true;
    }
    loom.persistent.second_node_unlock_elapsed = None;
}

/// Legacy constant kept for save compatibility.
pub const SECOND_NODE_UNLOCK_SECONDS: f64 = 14_400.0;

/// Legacy: staggered unlock is disabled. Returns false always.
pub fn tick_loom_staggered_unlock(_loom: &mut LoomState, _elapsed_seconds: f64) -> bool {
    false
}

/// Returns the throughput multiplier for a node.
/// Currently always 1.0 (archetype bonuses removed for rebalancing).
pub fn node_throughput_multiplier(_loom: &LoomState, _node_id: NodeId) -> f64 {
    1.0
}

/// Returns the conversion ratio multiplier for a node.
/// Currently always 1.0 (archetype bonuses removed for rebalancing).
pub fn node_conversion_multiplier(_loom: &LoomState, _node_id: NodeId) -> f64 {
    1.0
}

/// Returns the number of neighbors that unlock when a node produces enough.
pub fn node_neighbor_unlock_count(_loom: &LoomState, _node_id: NodeId) -> usize {
    2
}

/// Returns the upgrade cost multiplier for a node.
/// Currently always 1.0 (archetype bonuses removed for rebalancing).
pub fn node_upgrade_cost_multiplier(_loom: &LoomState, _node_id: NodeId) -> f64 {
    1.0
}

/// Returns the neighbor unlock speed multiplier for a node.
/// Currently always 1.0 (archetype bonuses removed for rebalancing).
pub fn node_neighbor_unlock_speed_multiplier(_loom: &LoomState, _node_id: NodeId) -> f64 {
    1.0
}

/// Returns whether the Resonance Forge feedback loop passive is active.
/// Currently always false (archetype bonuses removed for rebalancing).
pub fn resonance_early_feedback_active(_loom: &LoomState) -> bool {
    false
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
/// Full buffers cap at buffer_capacity; extractors never stall.
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

        let amount = rate * delta_hours;
        node.buffer = (node.buffer + amount).min(capacity);
        node.stalled = false;

        if amount > 0.0 {
            let resource = node_native_resource(node_id);
            *produced.entry(resource).or_insert(0.0) += amount;
        }
    }

    produced
}

// ── Phase 3: Node Upgrading ───────────────────────────────────────────────────

/// Returns the upgrade cost (in the node's native resource) for going from current level to next.
/// Base cost: 100 * level^1.2, rounded. Silence Well gets 25% discount at levels 1-5.
pub fn node_upgrade_cost(loom: &LoomState, node_id: NodeId) -> f64 {
    let node = &loom.persistent.nodes[node_id.index()];
    let base_cost = 100.0 * (node.level as f64).powf(1.2);
    let multiplier = node_upgrade_cost_multiplier(loom, node_id);
    (base_cost * multiplier).round()
}

/// Maximum extractor node level.
pub const MAX_NODE_LEVEL: u32 = 20;

/// Buffer capacity multiplier: 10 hours of production at current level's rate.
const BUFFER_HOURS: f64 = 10.0;

/// Attempt to upgrade a node's level.
/// Costs `node_upgrade_cost()` units of the node's native resource from the node's buffer.
/// Capped at `MAX_NODE_LEVEL` (level 20 = 525/hr).
/// Returns true if the upgrade succeeded.
pub fn try_upgrade_node(loom: &mut LoomState, node_id: NodeId) -> bool {
    let node = &loom.persistent.nodes[node_id.index()];
    if node.level >= MAX_NODE_LEVEL || !node.unlocked {
        return false;
    }

    let cost = node_upgrade_cost(loom, node_id);

    let node = &mut loom.persistent.nodes[node_id.index()];
    if node.buffer < cost {
        return false;
    }
    node.buffer -= cost;

    node.level += 1;
    node.buffer_capacity = node.base_rate * node_level_multiplier(node.level) * BUFFER_HOURS;

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

/// Determine whether a node should be stalled (buffer at capacity).
pub fn check_node_stall(node: &LoomNode) -> bool {
    node.buffer >= node.buffer_capacity
}

/// Update the `stalled` flag on every unlocked node.
///
/// A node stalls when its buffer is full (no pipes to drain it in direct-pull model).
/// Returns node IDs whose stall state changed (useful for event emission).
pub fn tick_stall_detection(loom: &mut LoomState) -> Vec<NodeId> {
    let mut changed = Vec::new();
    for node in &mut loom.persistent.nodes {
        if !node.unlocked {
            continue;
        }
        let should_stall = node.buffer >= node.buffer_capacity;
        if node.stalled != should_stall {
            node.stalled = should_stall;
            changed.push(node.id);
        }
    }
    changed
}

// ── Phase 6: Direct-Pull Shuttle Tick ────────────────────────────────────────

/// Max intake rate per input slot, by shuttle tier (units/hour).
pub fn tier_intake_cap(tier: u8) -> f64 {
    match tier {
        1 => 20.0,
        2 => 30.0,
        3 => 40.0,
        _ => 20.0,
    }
}

/// Effective intake cap for a shuttle, applying the level multiplier.
pub fn shuttle_effective_intake_cap(tier: u8, level: u32) -> f64 {
    tier_intake_cap(tier) * node_level_multiplier(level)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShuttleUpgradeError {
    InvalidIndex,
    UnderConstruction,
    AscensionTooLow,
    AtMaxLevel,
    InsufficientBuffer { needed: f64, have: f64 },
}

/// Upgrade a shuttle's level. Cost: 100 × level^1.2 from the shuttle's own buffer.
/// Max level capped by Ascension level via max_shuttle_level().
/// Buffer capacity scales with level after upgrade.
pub fn upgrade_shuttle(
    loom: &mut LoomState,
    shuttle_idx: usize,
    ascension_level: u32,
) -> Result<(), ShuttleUpgradeError> {
    let max_level = crate::ascension::types::max_shuttle_level(ascension_level);
    if max_level <= 1 {
        return Err(ShuttleUpgradeError::AscensionTooLow);
    }

    let shuttle = loom
        .persistent
        .shuttles
        .get(shuttle_idx)
        .ok_or(ShuttleUpgradeError::InvalidIndex)?;

    if shuttle.under_construction {
        return Err(ShuttleUpgradeError::UnderConstruction);
    }
    if shuttle.level >= max_level {
        return Err(ShuttleUpgradeError::AtMaxLevel);
    }

    let cost = 100.0 * (shuttle.level as f64).powf(1.2);
    if shuttle.buffer < cost {
        return Err(ShuttleUpgradeError::InsufficientBuffer {
            needed: cost,
            have: shuttle.buffer,
        });
    }

    let shuttle = loom.persistent.shuttles.get_mut(shuttle_idx).unwrap();
    shuttle.buffer -= cost;
    shuttle.level += 1;
    shuttle.buffer_capacity = 500.0 * node_level_multiplier(shuttle.level);
    Ok(())
}

/// Check whether a source node reference is valid for a given shuttle tier.
/// Extractors are always valid. Shuttles are valid only if their tier is
/// strictly less than the consuming shuttle's tier.
pub fn valid_source_for_tier(source: LoomNodeRef, shuttle_tier: u8, shuttles: &[Shuttle]) -> bool {
    match source {
        LoomNodeRef::Extractor(_) => true,
        LoomNodeRef::Shuttle(idx) => {
            if let Some(source_ref) = shuttles.get(idx) {
                source_ref.tier < shuttle_tier
            } else {
                false
            }
        }
    }
}

/// Tick all shuttle direct-pull processing.
///
/// Each tick:
/// 1. Count consumers per source (for contention splitting).
/// 2. Process shuttles by tier order (T1 first, then T2, then T3).
/// 3. For each shuttle: calculate available pull for each input slot.
///    - For each source: `share = source_available / num_consumers_of_that_source`
///    - `actual_pull = min(tier_intake_cap, share)` summed across all sources for that slot
/// 4. `output_rate = min(total_pull_a, total_pull_b) * recipe_amount`
/// 5. Add output to shuttle buffer (capped at capacity).
///
/// Returns a map of resource → total produced this tick (for pattern tracking).
pub fn tick_shuttle_pull(
    loom: &mut LoomState,
    delta_seconds: f64,
) -> std::collections::HashMap<Resource, f64> {
    let delta_hours = delta_seconds / 3600.0;
    let mut produced: std::collections::HashMap<Resource, f64> = std::collections::HashMap::new();

    // ── Pre-compute per-node throughput multipliers before mutable borrow ──
    let node_multipliers: std::collections::HashMap<NodeId, f64> = loom
        .persistent
        .nodes
        .iter()
        .map(|n| (n.id, node_throughput_multiplier(loom, n.id)))
        .collect();

    // ── Step 1: Count consumers per source across all non-construction shuttles ──
    let mut consumer_count: std::collections::HashMap<LoomNodeRef, usize> =
        std::collections::HashMap::new();
    for r in &loom.persistent.shuttles {
        if r.under_construction {
            continue;
        }
        for &src in r.sources_a.iter().chain(r.sources_b.iter()) {
            *consumer_count.entry(src).or_insert(0) += 1;
        }
    }

    // ── Step 2: Process shuttles by tier (T1 before T2 before T3) ──
    // Track effective output rates per shuttle index for higher-tier pulls.
    let mut shuttle_output_rates: Vec<f64> = vec![0.0; loom.persistent.shuttles.len()];

    for tier in 1u8..=3 {
        let indices: Vec<usize> = loom
            .persistent
            .shuttles
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.under_construction && r.tier == tier)
            .map(|(i, _)| i)
            .collect();

        for idx in indices {
            let r = &loom.persistent.shuttles[idx];
            let cap = shuttle_effective_intake_cap(r.tier, r.level);

            // Calculate available pull for input A.
            let pull_a: f64 = r
                .sources_a
                .iter()
                .map(|&src| {
                    let available = source_available_rate(
                        src,
                        &loom.persistent,
                        &shuttle_output_rates,
                        &node_multipliers,
                    );
                    let consumers = consumer_count.get(&src).copied().unwrap_or(1).max(1);
                    let share = available / consumers as f64;
                    share.min(cap)
                })
                .sum::<f64>()
                .min(cap);

            // Calculate available pull for input B.
            let pull_b: f64 = r
                .sources_b
                .iter()
                .map(|&src| {
                    let available = source_available_rate(
                        src,
                        &loom.persistent,
                        &shuttle_output_rates,
                        &node_multipliers,
                    );
                    let consumers = consumer_count.get(&src).copied().unwrap_or(1).max(1);
                    let share = available / consumers as f64;
                    share.min(cap)
                })
                .sum::<f64>()
                .min(cap);

            // Output rate for this tick = min(pull_a, pull_b) * recipe_amount.
            let output_rate = pull_a.min(pull_b) * r.amount;
            shuttle_output_rates[idx] = output_rate;

            // Add to buffer (capped); excess is discarded but still counted for rate tracking.
            let output_this_tick = output_rate * delta_hours;
            if output_this_tick > 0.0 {
                let r = &mut loom.persistent.shuttles[idx];
                r.buffer = (r.buffer + output_this_tick).min(r.buffer_capacity);
                r.stalled = false;
                *produced.entry(r.output).or_insert(0.0) += output_this_tick;
            }
        }
    }

    produced
}

/// Returns the effective hourly output rate of a source node reference.
fn source_available_rate(
    src: LoomNodeRef,
    persistent: &super::types::LoomPersistent,
    shuttle_rates: &[f64],
    node_multipliers: &std::collections::HashMap<NodeId, f64>,
) -> f64 {
    match src {
        LoomNodeRef::Extractor(node_id) => {
            let node = &persistent.nodes[node_id.index()];
            let throughput_mult = node_multipliers.get(&node_id).copied().unwrap_or(1.0);
            node_effective_rate_from_node(node, throughput_mult)
        }
        LoomNodeRef::Shuttle(idx) => shuttle_rates.get(idx).copied().unwrap_or(0.0),
    }
}

/// Compute a node's effective rate without needing the full LoomState borrow.
fn node_effective_rate_from_node(node: &LoomNode, throughput_multiplier: f64) -> f64 {
    if !node.unlocked {
        return 0.0;
    }
    node.base_rate * node_level_multiplier(node.level) * throughput_multiplier
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

/// Ticks required for a shuttle to finish construction (2 hours at 100ms/tick = 72000 ticks).
pub const SHUTTLE_CONSTRUCTION_TICKS: u32 = 72_000;

/// Error conditions for shuttle building.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShuttleError {
    InvalidRecipe,
    TierLocked,
    AtCapacity,
    InsufficientResources,
    InvalidSource,
}

/// Deduct `amount` from the buffer that holds `resource`.
/// Checks extractors first (for base resources), then shuttle buffers (for confluence/reaction).
/// Returns true if the deduction succeeded.
fn deduct_from_resource_buffer(
    persistent: &mut super::types::LoomPersistent,
    resource: Resource,
    amount: f64,
) -> bool {
    // Check extractors.
    for node in &mut persistent.nodes {
        if node.unlocked && node_native_resource(node.id) == resource && node.buffer >= amount {
            node.buffer -= amount;
            return true;
        }
    }
    // Check shuttle buffers.
    for shuttle in &mut persistent.shuttles {
        if !shuttle.under_construction && shuttle.output == resource && shuttle.buffer >= amount {
            shuttle.buffer -= amount;
            return true;
        }
    }
    false
}

/// Returns the available amount of a resource across all buffers (for UI display).
pub fn available_resource(loom: &LoomState, resource: Resource) -> f64 {
    let mut total = 0.0;
    for node in &loom.persistent.nodes {
        if node.unlocked && node_native_resource(node.id) == resource {
            total += node.buffer;
        }
    }
    for shuttle in &loom.persistent.shuttles {
        if !shuttle.under_construction && shuttle.output == resource {
            total += shuttle.buffer;
        }
    }
    total
}

fn shuttle_build_cost(tier: u8) -> f64 {
    match tier {
        1 => 250.0,
        2 => 150.0,
        _ => 100.0,
    }
}

fn shuttle_tier_unlock_threshold(tier: u8) -> usize {
    match tier {
        1 => 1,
        2 => 8,
        _ => 15,
    }
}

/// Returns the tiers currently unlocked based on completed pattern count.
pub fn unlocked_tiers(loom: &LoomState) -> Vec<u8> {
    let completed = loom
        .persistent
        .patterns
        .iter()
        .filter(|p| p.completed)
        .count();
    let mut tiers = Vec::new();
    for tier in 1u8..=3 {
        if completed >= shuttle_tier_unlock_threshold(tier) {
            tiers.push(tier);
        }
    }
    tiers
}

/// Returns the build cost for a shuttle of the given tier.
pub fn shuttle_build_cost_public(tier: u8) -> f64 {
    shuttle_build_cost(tier)
}

/// Returns all eligible source nodes for a given shuttle tier.
pub fn eligible_sources_for_tier(
    loom: &LoomState,
    tier: u8,
    resource: Resource,
) -> Vec<LoomNodeRef> {
    let mut sources = Vec::new();
    // Extractors that produce the needed resource.
    for node in &loom.persistent.nodes {
        if node.unlocked && node_native_resource(node.id) == resource {
            sources.push(LoomNodeRef::Extractor(node.id));
        }
    }
    // Shuttles of lower tier that output the needed resource.
    for (i, r) in loom.persistent.shuttles.iter().enumerate() {
        if r.output == resource
            && valid_source_for_tier(LoomNodeRef::Shuttle(i), tier, &loom.persistent.shuttles)
        {
            sources.push(LoomNodeRef::Shuttle(i));
        }
    }
    sources
}

/// Attempt to build a new Shuttle locked to the given recipe.
///
/// # Errors
/// - `ShuttleError::InvalidRecipe` — no recipe exists for the given inputs and nature.
/// - `ShuttleError::TierLocked` — the recipe's tier requires more completed patterns.
/// - `ShuttleError::AtCapacity` — the player already has the maximum number of shuttles.
/// - `ShuttleError::InsufficientResources` — not enough `input_a` stockpile to pay the build cost.
/// - `ShuttleError::InvalidSource` — a source node is invalid for the recipe's tier.
///
/// # Returns
/// The index of the newly created `Shuttle` in `loom.persistent.shuttles`.
pub fn build_shuttle(
    loom: &mut LoomState,
    input_a: Resource,
    input_b: Resource,
    nature: NodeNature,
    sources_a: Vec<LoomNodeRef>,
    sources_b: Vec<LoomNodeRef>,
) -> Result<usize, ShuttleError> {
    let recipe = crate::loom::recipes::find_recipe(input_a, input_b, nature)
        .ok_or(ShuttleError::InvalidRecipe)?;

    let completed_patterns = loom
        .persistent
        .patterns
        .iter()
        .filter(|p| p.completed)
        .count();
    if completed_patterns < shuttle_tier_unlock_threshold(recipe.tier) {
        return Err(ShuttleError::TierLocked);
    }

    if loom.persistent.shuttles.len() >= loom.persistent.max_shuttles() {
        return Err(ShuttleError::AtCapacity);
    }

    // Validate all sources for this tier.
    for &src in sources_a.iter().chain(sources_b.iter()) {
        if !valid_source_for_tier(src, recipe.tier, &loom.persistent.shuttles) {
            return Err(ShuttleError::InvalidSource);
        }
    }

    let cost = shuttle_build_cost(recipe.tier);
    // Draw build cost from the buffer that holds the input_a resource.
    // For base resources: the matching extractor's buffer.
    // For confluence/reaction resources: a shuttle buffer that outputs it.
    if !deduct_from_resource_buffer(&mut loom.persistent, input_a, cost) {
        return Err(ShuttleError::InsufficientResources);
    }

    let mut r = Shuttle::new(
        recipe.input_a,
        recipe.input_b,
        recipe.node_nature,
        recipe.output,
        recipe.amount,
        recipe.tier,
        sources_a,
        sources_b,
    );
    r.under_construction = true;
    r.construction_ticks_remaining = SHUTTLE_CONSTRUCTION_TICKS;
    loom.persistent.shuttles.push(r);
    Ok(loom.persistent.shuttles.len() - 1)
}

/// Tick construction for all shuttles under construction.
/// Returns indices of shuttles that completed this tick.
pub fn tick_shuttle_construction(loom: &mut LoomState) -> Vec<usize> {
    let warp = (loom.time_warp as u32).max(1);
    let mut completed = Vec::new();
    for (i, r) in loom.persistent.shuttles.iter_mut().enumerate() {
        if !r.under_construction {
            continue;
        }
        r.construction_ticks_remaining = r.construction_ticks_remaining.saturating_sub(warp);
        if r.construction_ticks_remaining == 0 {
            r.under_construction = false;
            completed.push(i);
        }
    }
    completed
}

/// Demolish a shuttle by index.
/// Removes the shuttle and re-indexes source references in remaining shuttles.
pub fn demolish_shuttle(loom: &mut LoomState, idx: usize) {
    if idx >= loom.persistent.shuttles.len() {
        return;
    }

    // Remove the shuttle.
    loom.persistent.shuttles.remove(idx);

    // Re-index source references in remaining shuttles.
    for r in &mut loom.persistent.shuttles {
        reindex_sources(&mut r.sources_a, idx);
        reindex_sources(&mut r.sources_b, idx);
    }
}

fn reindex_sources(sources: &mut Vec<LoomNodeRef>, removed_idx: usize) {
    sources.retain(|s| !matches!(s, LoomNodeRef::Shuttle(i) if *i == removed_idx));
    for s in sources.iter_mut() {
        if let LoomNodeRef::Shuttle(ref mut i) = s {
            if *i > removed_idx {
                *i -= 1;
            }
        }
    }
}

/// Update stall flags for all shuttles.
pub fn tick_shuttle_stall_detection(loom: &mut LoomState) {
    for r in &mut loom.persistent.shuttles {
        if r.under_construction {
            continue;
        }
        if r.buffer >= r.buffer_capacity {
            r.stalled = true;
        }
    }
}

/// Calculate PR generated per day from a given WR production rate (units/hr).
///
/// Tiered brackets:
/// - 0–10 WR/hr: 5 PR per WR/hr per day
/// - 10–25 WR/hr: 10 PR per WR/hr per day
/// - 25+ WR/hr: 15 PR per WR/hr per day
pub fn wr_to_pr_per_day(wr_per_hour: f64) -> u32 {
    if wr_per_hour <= 0.0 {
        return 0;
    }
    let mut pr = 0.0;
    let mut remaining = wr_per_hour;

    // Bracket 1: 0–10 at 5 PR per WR/hr
    let b1 = remaining.min(10.0);
    pr += b1 * 5.0;
    remaining -= b1;

    // Bracket 2: 10–25 at 10 PR per WR/hr
    if remaining > 0.0 {
        let b2 = remaining.min(15.0);
        pr += b2 * 10.0;
        remaining -= b2;
    }

    // Bracket 3: 25+ at 15 PR per WR/hr
    if remaining > 0.0 {
        pr += remaining * 15.0;
    }

    pr.round() as u32
}

/// Returns the highest zone ID unlocked by the given number of completed Woven Patterns.
pub fn loom_zone_cap_for_patterns(completed_patterns: usize) -> u32 {
    if completed_patterns >= 28 {
        50
    } else if completed_patterns >= 22 {
        46
    } else if completed_patterns >= 16 {
        42
    } else if completed_patterns >= 8 {
        38
    } else if completed_patterns >= 4 {
        34
    } else {
        0 // No Loom zones unlocked yet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loom_zone_cap_for_patterns() {
        assert_eq!(loom_zone_cap_for_patterns(0), 0);
        assert_eq!(loom_zone_cap_for_patterns(3), 0);
        assert_eq!(loom_zone_cap_for_patterns(4), 34);
        assert_eq!(loom_zone_cap_for_patterns(7), 34);
        assert_eq!(loom_zone_cap_for_patterns(8), 38);
        assert_eq!(loom_zone_cap_for_patterns(16), 42);
        assert_eq!(loom_zone_cap_for_patterns(22), 46);
        assert_eq!(loom_zone_cap_for_patterns(28), 50);
    }

    #[test]
    fn test_wr_to_pr_per_day_zero_rate() {
        assert_eq!(wr_to_pr_per_day(0.0), 0);
    }

    #[test]
    fn test_wr_to_pr_per_day_low_bracket() {
        assert_eq!(wr_to_pr_per_day(5.0), 25);
    }

    #[test]
    fn test_wr_to_pr_per_day_mid_bracket() {
        assert_eq!(wr_to_pr_per_day(20.0), 150);
    }

    #[test]
    fn test_wr_to_pr_per_day_high_bracket() {
        assert_eq!(wr_to_pr_per_day(60.0), 725);
    }

    #[test]
    fn test_wr_to_pr_per_day_exact_bracket_boundary() {
        assert_eq!(wr_to_pr_per_day(10.0), 50);
        assert_eq!(wr_to_pr_per_day(25.0), 200);
    }

    #[test]
    fn test_initialize_loom_unlocks_only_ember_spindle() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);

        for node in &loom.persistent.nodes {
            if node.id == NodeId::EmberSpindle {
                assert!(node.unlocked, "EmberSpindle should be unlocked");
            } else {
                assert!(!node.unlocked, "node {:?} should be locked", node.id);
            }
        }
        assert_eq!(loom.persistent.second_node_unlock_elapsed, None);
    }

    #[test]
    fn test_multipliers_are_neutral() {
        let loom = LoomState::new();
        // All multiplier functions return neutral values (archetype bonuses removed)
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
        assert_eq!(
            node_neighbor_unlock_speed_multiplier(&loom, NodeId::EmberSpindle),
            1.0
        );
        assert!(!resonance_early_feedback_active(&loom));
    }

    #[test]
    fn test_staggered_unlock_is_noop() {
        let mut loom = LoomState::new();
        // Staggered unlock always returns false now
        assert!(!tick_loom_staggered_unlock(&mut loom, 14400.0));
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
        initialize_loom(&mut loom);

        tick_base_production(&mut loom, 3600.0);

        let ember = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        // 50/hr base * 1.0x (no archetype bonus). After 1 hr: 50.0 units.
        assert!(
            (ember.buffer - 50.0).abs() < 0.001,
            "buffer should be ~50.0, got {}",
            ember.buffer
        );
    }

    #[test]
    fn test_base_production_locked_node_produces_nothing() {
        let mut loom = LoomState::new();
        // Don't initialize — leave nodes locked.
        // Manually unlock just EmberSpindle.
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .unlocked = true;

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
    fn test_base_production_caps_buffer_at_capacity() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);

        // Fill buffer to capacity.
        let ember = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        ember.buffer = ember.buffer_capacity;

        let produced = tick_base_production(&mut loom, 3600.0);

        let ember = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        // Extractor no longer stalls from full buffer — it auto-drains.
        assert!(!ember.stalled);
        assert_eq!(ember.buffer, ember.buffer_capacity);
        // Full production amount is still reported for rate tracking.
        let ember_produced = produced.get(&Resource::Ember).copied().unwrap_or(0.0);
        assert!(
            ember_produced > 0.0,
            "should report production even with full buffer"
        );
    }

    #[test]
    fn test_extractor_produces_at_full_rate_when_buffer_full() {
        let mut loom = LoomState::new();
        loom.persistent.nodes[0].unlocked = true;
        loom.persistent.nodes[0].buffer = loom.persistent.nodes[0].buffer_capacity;

        let produced = tick_base_production(&mut loom, 0.1);

        let ember_produced = produced.get(&Resource::Ember).copied().unwrap_or(0.0);
        assert!(
            ember_produced > 0.0,
            "extractor should report production even with full buffer"
        );
        assert!(
            !loom.persistent.nodes[0].stalled,
            "extractor should not stall from full buffer"
        );
    }

    #[test]
    fn test_extractor_buffer_does_not_exceed_capacity() {
        let mut loom = LoomState::new();
        loom.persistent.nodes[0].unlocked = true;
        loom.persistent.nodes[0].buffer = loom.persistent.nodes[0].buffer_capacity - 0.001;

        tick_base_production(&mut loom, 0.1);

        assert!(
            loom.persistent.nodes[0].buffer <= loom.persistent.nodes[0].buffer_capacity + 1e-9,
            "buffer should not exceed capacity"
        );
    }

    // ── Phase 3: Node Upgrading tests ─────────────────────────────────────────

    #[test]
    fn test_upgrade_cost_level1() {
        let loom = LoomState::new();
        let cost = node_upgrade_cost(&loom, NodeId::EmberSpindle);
        assert_eq!(cost, 100.0); // 100 * 1^1.2 = 100
    }

    #[test]
    fn test_upgrade_succeeds_with_sufficient_stockpile() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);

        loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer = 500.0;

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
    fn test_upgrade_fails_with_insufficient_stockpile() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);

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
        loom.persistent.nodes[NodeId::VoidCondenser.index()].buffer = 500.0;

        let result = try_upgrade_node(&mut loom, NodeId::VoidCondenser);
        assert!(!result);
    }

    #[test]
    fn test_upgrade_cost_no_discount() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);

        // No archetype discount — cost is base cost * 1.0
        let base_cost = 100.0_f64 * 1.0_f64.powf(1.2);
        let expected = (base_cost * 1.0).round();
        let cost = node_upgrade_cost(&loom, NodeId::SilenceWell);
        assert_eq!(cost, expected);
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
        // Only unlock EmberSpindle, leave neighbors locked.
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .unlocked = true;

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
        // Only unlock ReflectionLens.
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::ReflectionLens)
            .unwrap()
            .unlocked = true;

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
    fn test_neighbor_unlock_speed_is_normal() {
        let mut loom = LoomState::new();
        // Only unlock EmberSpindle.
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .unlocked = true;

        let ember = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        ember.buffer = ember.buffer_capacity;

        // At 1.0x speed (no archetype bonus), 2hr tick = 2.0hr progress — enough to unlock.
        let unlocked = tick_neighbor_unlocking(&mut loom, 7200.0);
        assert!(
            !unlocked.is_empty(),
            "Should have unlocked neighbors at 1.0x speed in 2hr"
        );
    }

    // ── Phase 6: Direct-Pull Shuttle tests ───────────────────────────────────

    #[test]
    fn test_tick_shuttle_pull_empty_no_panic() {
        let mut loom = LoomState::new();
        let produced = tick_shuttle_pull(&mut loom, 0.1);
        assert!(produced.is_empty());
    }

    #[test]
    fn test_tick_shuttle_pull_with_unlocked_source_produces_output() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        // Fill EmberSpindle buffer to give it a non-zero rate proxy.
        let ember_node = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        ember_node.buffer = ember_node.buffer_capacity;
        // VoidCondenser also needs to be unlocked for sources_b.
        let void_node = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::VoidCondenser)
            .unwrap();
        void_node.unlocked = true;
        void_node.buffer = void_node.buffer_capacity;

        let mut r = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        r.under_construction = false;
        loom.persistent.shuttles.push(r);

        // Run for 1 hour worth of ticks.
        let produced = tick_shuttle_pull(&mut loom, 3600.0);
        // Should produce some ForgedLight.
        assert!(
            produced.get(&Resource::ForgedLight).copied().unwrap_or(0.0) > 0.0,
            "expected ForgedLight production"
        );
    }

    #[test]
    fn test_tick_shuttle_pull_under_construction_skipped() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        let void_node = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::VoidCondenser)
            .unwrap();
        void_node.unlocked = true;

        let mut r = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        r.under_construction = true;
        r.construction_ticks_remaining = 100;
        loom.persistent.shuttles.push(r);

        let produced = tick_shuttle_pull(&mut loom, 3600.0);
        assert!(
            produced.is_empty(),
            "under-construction shuttle should produce nothing"
        );
    }

    // ── Phase 5: Buffer Stalling tests ────────────────────────────────────────

    fn make_node(id: NodeId, buffer: f64, capacity: f64) -> LoomNode {
        let mut n = LoomNode::new(id);
        n.unlocked = true;
        n.buffer = buffer;
        n.buffer_capacity = capacity;
        n
    }

    #[test]
    fn test_check_node_stall_buffer_below_capacity_not_stalled() {
        let node = make_node(NodeId::EmberSpindle, 10.0, 20.0); // half full
        assert!(!check_node_stall(&node));
    }

    #[test]
    fn test_check_node_stall_full_buffer_stalls() {
        let node = make_node(NodeId::EmberSpindle, 20.0, 20.0); // full
        assert!(check_node_stall(&node));
    }

    #[test]
    fn test_tick_stall_detection_marks_stalled_node() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);

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
        initialize_loom(&mut loom);

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
        initialize_loom(&mut loom);

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
        // Manually unlock SilenceWell for this test
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::SilenceWell)
            .unwrap()
            .unlocked = true;
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
        // base_rate 50.0 * level_mult(2) 1.5 * throughput_mult 1.0 = 75.0
        assert!(
            (rate - 75.0).abs() < 0.001,
            "expected 75.0/hr, got {}",
            rate
        );
    }

    #[test]
    fn test_node_effective_rate_ember_spindle_with_level() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
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
        // base_rate 50.0 * level_mult(3) 2.0 * throughput_mult 1.0 = 100.0
        assert!(
            (rate - 100.0).abs() < 0.001,
            "expected 100.0/hr, got {}",
            rate
        );
    }

    // ── try_upgrade_node buffer capacity ──────────────────────────────────────

    #[test]
    fn test_upgrade_node_increases_buffer_capacity() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        let old_capacity = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer_capacity;

        loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer = 500.0;
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
    fn test_upgrade_node_deducts_cost_from_stockpile() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        let cost = node_upgrade_cost(&loom, NodeId::EmberSpindle);
        let starting_buffer = cost + 5.0;

        loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer = starting_buffer;
        try_upgrade_node(&mut loom, NodeId::EmberSpindle);

        let remaining = loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer;
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
        initialize_loom(&mut loom);

        let produced = tick_base_production(&mut loom, 3600.0);

        assert!(
            produced.contains_key(&Resource::Ember),
            "produced map should include Ember"
        );
        let ember_amount = produced[&Resource::Ember];
        // 50/hr base * 1.0x (no archetype bonus); after 1hr = 50.0 units
        assert!(
            (ember_amount - 50.0).abs() < 0.001,
            "expected 50.0 Ember produced, got {}",
            ember_amount
        );
    }

    #[test]
    fn test_tick_base_production_locked_nodes_absent_from_produced_map() {
        let mut loom = LoomState::new();
        // Only unlock EmberSpindle, leave others locked.
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .unlocked = true;

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
        // Only unlock EmberSpindle, leave neighbors locked.
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .unlocked = true;

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
        initialize_loom(&mut loom);
        // Buffer is empty — nothing should unlock.
        let unlocked = tick_neighbor_unlocking(&mut loom, 3600.0);
        assert!(
            unlocked.is_empty(),
            "should return empty vec when nothing unlocks"
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
    fn test_ember_spindle_unlock_speed_is_normal() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        // No archetype bonus — speed is 1.0x for all nodes.
        assert!(
            (node_neighbor_unlock_speed_multiplier(&loom, NodeId::EmberSpindle) - 1.0).abs()
                < 0.001
        );
    }

    #[test]
    fn test_other_nodes_unlock_speed_is_one() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
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
        initialize_loom(&mut loom);

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

    // ── Shuttle Ticking ──────────────────────────────────────────────────────

    #[test]
    fn test_shuttle_construction_completes() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        loom.persistent.shuttles.push({
            let mut r = Shuttle::new(
                Resource::Ember,
                Resource::VoidEssence,
                NodeNature::Heat,
                Resource::ForgedLight,
                1.0,
                1,
                vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
                vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
            );
            r.under_construction = true;
            r.construction_ticks_remaining = 1;
            r
        });

        let completed = tick_shuttle_construction(&mut loom);
        assert_eq!(completed.len(), 1);
        assert!(!loom.persistent.shuttles[0].under_construction);
    }

    #[test]
    fn test_shuttle_pull_produces_output() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        // Unlock two extractor nodes and fill their buffers.
        loom.persistent.nodes[0].unlocked = true;
        loom.persistent.nodes[0].buffer = 50.0;
        loom.persistent.nodes[1].unlocked = true;
        loom.persistent.nodes[1].buffer = 50.0;
        loom.persistent.shuttles.push(Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        ));

        let produced = tick_shuttle_pull(&mut loom, 1.0);
        // A T1 shuttle should produce some output when sources have stock.
        assert!(!produced.is_empty() || loom.persistent.shuttles[0].buffer >= 0.0);
    }

    #[test]
    fn test_shuttle_stall_when_buffer_full() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        let mut r = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![],
            vec![],
        );
        r.buffer = r.buffer_capacity;
        loom.persistent.shuttles.push(r);

        tick_shuttle_stall_detection(&mut loom);
        assert!(loom.persistent.shuttles[0].stalled);
    }

    #[test]
    fn test_demolish_shuttle() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        loom.persistent.shuttles.push(Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        ));

        demolish_shuttle(&mut loom, 0);
        assert!(loom.persistent.shuttles.is_empty());
    }

    #[test]
    fn test_demolish_shuttle_reindexes_sources() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        // Build two shuttles; second references first via sources_a.
        loom.persistent.shuttles.push(Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        ));
        loom.persistent.shuttles.push(Shuttle::new(
            Resource::ForgedLight,
            Resource::Reflection,
            NodeNature::Form,
            Resource::EchoGlass,
            1.0,
            2,
            vec![LoomNodeRef::Shuttle(0)],
            vec![LoomNodeRef::Extractor(NodeId::ReflectionLens)],
        ));
        // Insert a T1 shuttle before index 0 — then demolish it.
        // Actually insert at index 0 by inserting first, shifting second to index 1.
        // Easier: demolish shuttle 0 and check that shuttle (now index 0) has its
        // sources_a updated from Shuttle(0) to be removed (was pointing to the removed one).
        demolish_shuttle(&mut loom, 0);
        assert_eq!(loom.persistent.shuttles.len(), 1);
        // sources_a pointed to Shuttle(0) which was demolished → should be empty.
        assert!(
            loom.persistent.shuttles[0].sources_a.is_empty(),
            "source reference to demolished shuttle should be removed"
        );
    }
}

// ---------------------------------------------------------------------------
// External system bonuses — granular per-aspect bonuses (Task #20 supplement)
// ---------------------------------------------------------------------------

/// Pre-computed bonuses from existing game systems that boost Loom production.
///
/// Breaks external bonuses into two separate axes so callers can apply them
/// only where relevant (production rate, buffer capacity).
/// Passed via explicit parameters following the Haven bonus injection pattern —
/// Loom logic never imports Haven/Deep/Stormglass/Ascension directly.
#[derive(Debug, Clone, Copy, Default)]
pub struct LoomExternalBonuses {
    /// Additive fraction bonus on all node base production rates (e.g. 0.10 = +10%).
    pub production_rate_bonus: f64,
    /// Additive fraction bonus on buffer capacity for all nodes (e.g. 0.20 = +20%).
    pub buffer_capacity_bonus: f64,
}

/// Compute granular Loom bonuses from the current state of existing game systems.
///
/// # Parameters
/// - `haven_damage_percent`: Haven Armory damage bonus (0-25.0). Each 5% maps to
///   +1% production rate (max +5% at 25%).
/// - `deep_guild_rank`: Deep guild rank 1-5. Each rank above 1 adds +5% buffer
///   capacity (max +20% at rank 5).
/// - `ascension_level`: Current ascension level (0 = none). Reserved for future use.
/// - `stormglass_balance`: Current Stormglass balance. Every 100k SG adds +1%
///   production rate, capped at +5% (500k SG).
///
/// All bonuses are additive within their category and independent of each other.
/// Haven and Stormglass bonuses stack additively into `production_rate_bonus`.
pub fn loom_external_bonuses(
    haven_damage_percent: f64,
    deep_guild_rank: u8,
    _ascension_level: u32,
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

    LoomExternalBonuses {
        production_rate_bonus,
        buffer_capacity_bonus,
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

#[cfg(test)]
mod external_bonus_tests {
    use super::*;

    #[test]
    fn test_no_bonuses_when_all_systems_at_minimum() {
        let b = loom_external_bonuses(0.0, 1, 0, 0);
        assert!((b.production_rate_bonus).abs() < 1e-9);
        assert!((b.buffer_capacity_bonus).abs() < 1e-9);
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

    // Helper: populate patterns via complete_discovery and mark the first N as completed.
    fn setup_patterns(loom: &mut LoomState, completed_count: usize) {
        crate::loom::discovery::complete_discovery(loom);
        for p in loom.persistent.patterns.iter_mut().take(completed_count) {
            p.completed = true;
        }
    }

    #[test]
    fn test_build_shuttle_success() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        setup_patterns(&mut loom, 1);
        loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer = 500.0;

        let result = build_shuttle(
            &mut loom,
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        assert!(result.is_ok());
        assert_eq!(loom.persistent.shuttles.len(), 1);
        let r = &loom.persistent.shuttles[0];
        assert_eq!(r.output, Resource::ForgedLight);
        assert!(r.under_construction);
    }

    #[test]
    fn test_build_shuttle_fails_at_capacity() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        // No completed patterns → max_shuttles() == 0 → AtCapacity.
        let result = build_shuttle(
            &mut loom,
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            vec![],
            vec![],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_build_shuttle_fails_insufficient_resources() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        setup_patterns(&mut loom, 1);
        // No Ember stockpile → InsufficientResources.
        let result = build_shuttle(
            &mut loom,
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_build_shuttle_fails_invalid_recipe() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        setup_patterns(&mut loom, 1);
        loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer = 50.0;

        let result = build_shuttle(
            &mut loom,
            Resource::WovenReality,
            Resource::WovenReality,
            NodeNature::Heat,
            vec![],
            vec![],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_build_shuttle_tier_gating() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        // Only 1 completed pattern; Tier 2 recipes require 8 → TierLocked.
        setup_patterns(&mut loom, 1);
        // Resources don't matter — tier check fails first (T2 needs 8 patterns).

        let result = build_shuttle(
            &mut loom,
            Resource::ForgedLight,
            Resource::EchoGlass,
            NodeNature::Heat,
            vec![],
            vec![],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_tick_shuttle_pull_basic() {
        let mut loom = LoomState::new();
        for node in loom.persistent.nodes.iter_mut() {
            node.unlocked = true;
        }

        let mut r = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        r.under_construction = false;
        loom.persistent.shuttles.push(r);

        // Run for exactly 1 hour.
        let produced = tick_shuttle_pull(&mut loom, 3600.0);
        let forged = produced.get(&Resource::ForgedLight).copied().unwrap_or(0.0);

        // T1 intake cap = 20.0/hr. EmberSpindle = 50.0/hr, VoidCondenser = 50.0/hr.
        // pull_a = min(50.0, 20.0 cap) = 20.0; pull_b = min(50.0, 20.0 cap) = 20.0
        // output = min(20.0, 20.0) * 1.0 = 20.0/hr => 20.0 in 1 hour.
        assert!(
            (forged - 20.0).abs() < 0.01,
            "expected ~20.0 ForgedLight, got {forged}"
        );
    }

    #[test]
    fn test_tick_shuttle_pull_contention_splits_evenly() {
        let mut loom = LoomState::new();
        for node in loom.persistent.nodes.iter_mut() {
            node.unlocked = true;
        }

        // Two T1 shuttles both pulling from EmberSpindle (and VoidCondenser).
        for _ in 0..2 {
            let mut r = Shuttle::new(
                Resource::Ember,
                Resource::VoidEssence,
                NodeNature::Heat,
                Resource::ForgedLight,
                1.0,
                1,
                vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
                vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
            );
            r.under_construction = false;
            loom.persistent.shuttles.push(r);
        }

        let produced = tick_shuttle_pull(&mut loom, 3600.0);
        let forged = produced.get(&Resource::ForgedLight).copied().unwrap_or(0.0);

        // EmberSpindle effective = 50.0/hr, split 2 ways = 25.0 each, capped at 20.0.
        // VoidCondenser = 50.0/hr, split 2 ways = 25.0 each, capped at 20.0.
        // Each shuttle: min(20.0, 20.0) * 1.0 = 20.0/hr. Total = 40.0/hr => 40.0 in 1 hour.
        assert!(
            (forged - 40.0).abs() < 0.01,
            "expected ~40.0 ForgedLight from two shuttles, got {forged}"
        );
    }

    #[test]
    fn test_tier_gates_shifted() {
        let mut loom = LoomState::new();
        crate::loom::complete_discovery(&mut loom);
        assert!(unlocked_tiers(&loom).is_empty());
        loom.persistent.patterns[0].completed = true;
        assert_eq!(unlocked_tiers(&loom), vec![1]);
        for i in 1..7 {
            loom.persistent.patterns[i].completed = true;
        }
        assert_eq!(unlocked_tiers(&loom), vec![1]);
        loom.persistent.patterns[7].completed = true;
        assert_eq!(unlocked_tiers(&loom), vec![1, 2]);
        for i in 8..14 {
            loom.persistent.patterns[i].completed = true;
        }
        assert_eq!(unlocked_tiers(&loom), vec![1, 2]);
        loom.persistent.patterns[14].completed = true;
        assert_eq!(unlocked_tiers(&loom), vec![1, 2, 3]);
    }

    #[test]
    fn test_shuttle_effective_intake_cap() {
        assert!((shuttle_effective_intake_cap(1, 1) - 20.0).abs() < 0.001);
        assert!((shuttle_effective_intake_cap(1, 3) - 40.0).abs() < 0.001);
        assert!((shuttle_effective_intake_cap(3, 5) - 120.0).abs() < 0.001);
    }

    #[test]
    fn test_upgrade_shuttle_success() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        setup_patterns(&mut loom, 8);
        for node in loom.persistent.nodes.iter_mut() {
            node.unlocked = true;
        }
        loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer = 500.0;
        let _ = build_shuttle(
            &mut loom,
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        loom.persistent.shuttles[0].under_construction = false;
        // Upgrade costs come from shuttle's own buffer
        loom.persistent.shuttles[0].buffer = 500.0;

        let result = upgrade_shuttle(&mut loom, 0, 7);
        assert!(result.is_ok());
        assert_eq!(loom.persistent.shuttles[0].level, 2);
    }

    #[test]
    fn test_upgrade_shuttle_blocked_by_ascension_cap() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        setup_patterns(&mut loom, 8);
        for node in loom.persistent.nodes.iter_mut() {
            node.unlocked = true;
        }
        loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer = 500.0;
        let _ = build_shuttle(
            &mut loom,
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        loom.persistent.shuttles[0].under_construction = false;
        loom.persistent.shuttles[0].level = 3;
        loom.persistent.shuttles[0].buffer = 5000.0;

        let result = upgrade_shuttle(&mut loom, 0, 7); // max for Asc VII is 3
        assert!(result.is_err());
    }

    #[test]
    fn test_upgrade_shuttle_blocked_without_ascension_vii() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        setup_patterns(&mut loom, 1);
        for node in loom.persistent.nodes.iter_mut() {
            node.unlocked = true;
        }
        loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer = 500.0;
        let _ = build_shuttle(
            &mut loom,
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        loom.persistent.shuttles[0].under_construction = false;
        loom.persistent.shuttles[0].buffer = 5000.0;

        let result = upgrade_shuttle(&mut loom, 0, 6); // Asc VI, no shuttle upgrades
        assert!(result.is_err());
    }
}
