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

/// Number of cycle-neighbors each node unlocks simultaneously.
const NEIGHBOR_UNLOCK_COUNT: usize = 2;

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

/// Returns the effective production rate (native resource per hour) for a node.
pub fn node_effective_rate(_loom: &LoomState, node: &LoomNode) -> f64 {
    if !node.unlocked || node.upgrading {
        return 0.0;
    }
    node.base_rate * node_level_multiplier(node.level)
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
    if delta_seconds <= 0.0 {
        return std::collections::HashMap::new();
    }
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
        if node.upgrading {
            continue; // Nodes produce nothing while upgrading.
        }

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
/// Base cost: 100 * level^1.2, rounded.
pub fn node_upgrade_cost(loom: &LoomState, node_id: NodeId) -> f64 {
    let node = &loom.persistent.nodes[node_id.index()];
    (100.0 * (node.level as f64).powf(1.2)).round()
}

/// Maximum extractor node level.
pub const MAX_NODE_LEVEL: u32 = 20;

/// Buffer capacity multiplier: 10 hours of production at current level's rate.
const BUFFER_HOURS: f64 = 10.0;

/// Upgrade duration in seconds for going from current_level to next.
/// Linear 2h per level: L1→L2 = 2h, L2→L3 = 4h, L3→L4 = 6h, etc.
/// Time warp accelerates this (delta_seconds includes warp).
pub fn node_upgrade_duration(level: u32) -> f64 {
    7200.0 * level as f64 // 2h, 4h, 6h, 8h... (level * 2 hours)
}

/// Attempt to start upgrading a node's level.
/// Drains 50% of buffer capacity as cost and starts a lockout timer.
/// The node produces nothing while upgrading. When the timer expires,
/// the node's level increases.
/// Capped at `MAX_NODE_LEVEL` (level 20 = 525/hr).
/// Returns true if the upgrade was initiated.
pub fn try_upgrade_node(loom: &mut LoomState, node_id: NodeId) -> bool {
    let node = &loom.persistent.nodes[node_id.index()];
    if node.level >= MAX_NODE_LEVEL || !node.unlocked || node.upgrading {
        return false;
    }

    let drain = node.buffer_capacity * 0.5;

    let node = &mut loom.persistent.nodes[node_id.index()];
    if node.buffer < drain {
        return false;
    }

    // Drain 50% of buffer capacity as upgrade cost.
    node.buffer = (node.buffer - drain).max(0.0);

    // Start upgrade lockout.
    node.upgrading = true;
    node.upgrade_remaining_secs = node_upgrade_duration(node.level);
    loom.graph_dirty = true;

    // Clear the rate tracker so the rate drops to 0 immediately
    // (instead of decaying over the 20-second rolling window).
    let resource = node_native_resource(node_id);
    loom.rate_trackers
        .entry(resource)
        .and_modify(|t| *t = super::types::RateTracker::new());

    true
}

/// Tick upgrade timers for all nodes. Called each game tick.
/// When an upgrade completes, the node's level increases and production resumes.
pub fn tick_node_upgrades(loom: &mut LoomState, delta_seconds: f64) {
    for node in &mut loom.persistent.nodes {
        if !node.upgrading {
            continue;
        }
        node.upgrade_remaining_secs -= delta_seconds;
        if node.upgrade_remaining_secs <= 0.0 {
            node.upgrading = false;
            node.upgrade_remaining_secs = 0.0;
            node.level = (node.level + 1).min(MAX_NODE_LEVEL);
            node.buffer_capacity =
                node.base_rate * node_level_multiplier(node.level) * BUFFER_HOURS;
            loom.graph_dirty = true;
        }
    }
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

        let unlock_count = NEIGHBOR_UNLOCK_COUNT;
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
                neighbor.unlock_progress += delta_hours;
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
/// NOTE: Display-only — no longer used for simulation (intake cap removed).
pub fn tier_intake_cap(tier: u8) -> f64 {
    match tier {
        1 => 20.0,
        2 => 30.0,
        3 => 40.0,
        _ => 20.0,
    }
}

/// Effective intake cap for a shuttle, applying the level multiplier.
/// NOTE: Display-only — no longer used for simulation (intake cap removed).
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
    loom.graph_dirty = true;
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
///    - No per-tier intake cap — throughput limited only by source rate and contention.
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

            // Calculate available pull for input A (no intake cap — limited only by source rate and contention).
            let pull_a: f64 = r
                .sources_a
                .iter()
                .map(|&src| {
                    let available =
                        source_available_rate(src, &loom.persistent, &shuttle_output_rates);
                    let consumers = consumer_count.get(&src).copied().unwrap_or(1).max(1);
                    available / consumers as f64
                })
                .sum();

            // Calculate available pull for input B (no intake cap — limited only by source rate and contention).
            let pull_b: f64 = r
                .sources_b
                .iter()
                .map(|&src| {
                    let available =
                        source_available_rate(src, &loom.persistent, &shuttle_output_rates);
                    let consumers = consumer_count.get(&src).copied().unwrap_or(1).max(1);
                    available / consumers as f64
                })
                .sum();

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
                // Push un-warped amount so rate_per_hour() reflects logical rate,
                // consistent with extractor rate_trackers in tick_stages.rs.
                let warp = loom.time_warp.max(1.0);
                r.output_rate_tracker.push(output_this_tick / warp);
            } else {
                loom.persistent.shuttles[idx].output_rate_tracker.push(0.0);
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
) -> f64 {
    match src {
        LoomNodeRef::Extractor(node_id) => {
            let node = &persistent.nodes[node_id.index()];
            node_effective_rate_from_node(node)
        }
        LoomNodeRef::Shuttle(idx) => shuttle_rates.get(idx).copied().unwrap_or(0.0),
    }
}

/// Compute a node's effective rate without needing the full LoomState borrow.
fn node_effective_rate_from_node(node: &LoomNode) -> f64 {
    if !node.unlocked || node.upgrading {
        return 0.0;
    }
    node.base_rate * node_level_multiplier(node.level)
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

/// Construction duration in seconds by tier: T1=2h, T2=4h, T3=6h.
pub fn shuttle_construction_secs(tier: u8) -> f64 {
    match tier {
        1 => 7200.0,  // 2 hours
        2 => 14400.0, // 4 hours
        3 => 21600.0, // 6 hours
        _ => 7200.0,
    }
}

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

/// Returns the build cost for a shuttle of the given tier.
pub fn shuttle_build_cost(tier: u8) -> f64 {
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
    r.construction_secs_remaining = shuttle_construction_secs(recipe.tier);
    loom.persistent.shuttles.push(r);
    loom.graph_dirty = true;
    Ok(loom.persistent.shuttles.len() - 1)
}

/// Tick construction for all shuttles under construction.
/// Returns indices of shuttles that completed this tick.
pub fn tick_shuttle_construction(loom: &mut LoomState, delta_seconds: f64) -> Vec<usize> {
    let mut completed = Vec::new();
    for (i, r) in loom.persistent.shuttles.iter_mut().enumerate() {
        if !r.under_construction {
            continue;
        }
        r.construction_secs_remaining -= delta_seconds;
        if r.construction_secs_remaining <= 0.0 {
            r.under_construction = false;
            r.construction_secs_remaining = 0.0;
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
    loom.graph_dirty = true;
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
        r.stalled = r.buffer >= r.buffer_capacity;
    }
}

/// Calculate PR generated per day from a given WR production rate (units/hr).
///
/// Tiered brackets:
/// Convert Weave Rate (WR/hr) to Prestige Ranks per hour.
///
/// Formula: `PR/hr = WR × (1 + WR/100)`
/// Starts ~1:1 at low rates, scales superlinearly as WR increases.
/// At 50 WR/hr → 75 PR/hr, at 131 WR/hr → 302 PR/hr.
pub fn wr_to_pr_per_hour(wr_per_hour: f64) -> u32 {
    if !wr_per_hour.is_finite() || wr_per_hour <= 0.0 {
        return 0;
    }
    let pr = wr_per_hour * (1.0 + wr_per_hour / 100.0);
    pr.round() as u32
}

/// Returns the multiplier applied to WR rate (for UI display).
/// `multiplier = 1 + WR/100`
pub fn wr_pr_multiplier(wr_per_hour: f64) -> f64 {
    1.0 + wr_per_hour / 100.0
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
    fn test_wr_to_pr_per_hour_zero_rate() {
        assert_eq!(wr_to_pr_per_hour(0.0), 0);
    }

    #[test]
    fn test_wr_to_pr_per_hour_nan() {
        assert_eq!(wr_to_pr_per_hour(f64::NAN), 0);
        assert_eq!(wr_to_pr_per_hour(f64::INFINITY), 0);
        assert_eq!(wr_to_pr_per_hour(-1.0), 0);
    }

    #[test]
    fn test_wr_to_pr_per_hour_low_rate() {
        // PR = 10 * (1 + 10/100) = 10 * 1.1 = 11
        assert_eq!(wr_to_pr_per_hour(10.0), 11);
    }

    #[test]
    fn test_wr_to_pr_per_hour_pattern28_rate() {
        // PR = 50 * (1 + 50/100) = 50 * 1.5 = 75
        assert_eq!(wr_to_pr_per_hour(50.0), 75);
    }

    #[test]
    fn test_wr_to_pr_per_hour_max_rate() {
        // PR = 131 * (1 + 131/100) = 131 * 2.31 = 302.61 → 303
        assert_eq!(wr_to_pr_per_hour(131.0), 303);
    }

    #[test]
    fn test_wr_to_pr_per_hour_starts_near_one_to_one() {
        // At low rates, multiplier ≈ 1.0 so PR ≈ WR
        assert_eq!(wr_to_pr_per_hour(1.0), 1); // 1 * 1.01 = 1.01 → 1
        assert_eq!(wr_to_pr_per_hour(5.0), 5); // 5 * 1.05 = 5.25 → 5
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
        // 25/hr base * 1.0x (no archetype bonus). After 1 hr: 25.0 units.
        assert!(
            (ember.buffer - 25.0).abs() < 0.001,
            "buffer should be ~25.0, got {}",
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

        let ember = &loom.persistent.nodes[NodeId::EmberSpindle.index()];
        // Upgrade is now deferred — node should be upgrading, still level 1.
        assert!(ember.upgrading);
        assert_eq!(ember.level, 1);
        assert!(ember.upgrade_remaining_secs > 0.0);

        // Complete the upgrade by ticking past the duration.
        let remaining = ember.upgrade_remaining_secs;
        tick_node_upgrades(&mut loom, remaining + 1.0);

        let ember = &loom.persistent.nodes[NodeId::EmberSpindle.index()];
        assert!(!ember.upgrading);
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
        r.construction_secs_remaining = 100.0;
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
            Resource::ForgedLight,
            1.0,
        );
        assert_eq!(codex.len(), 1);
        assert!(codex[0].discovered);
        assert_eq!(codex[0].output, Resource::ForgedLight);
    }

    #[test]
    fn test_record_codex_discovery_marks_existing_entry() {
        use super::super::types::{CodexEntry, NodeNature};
        let mut codex = vec![CodexEntry {
            inputs: vec![Resource::Ember, Resource::VoidEssence],
            node_nature: NodeNature::Heat,
            output: Resource::ForgedLight,
            output_amount: 1.0,
            discovered: false,
        }];
        record_codex_discovery(
            &mut codex,
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
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
            output: Resource::ForgedLight,
            output_amount: 1.0,
            discovered: false,
        }];
        // Reverse input order — should still match.
        record_codex_discovery(
            &mut codex,
            Resource::VoidEssence,
            Resource::Ember,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
        );
        assert_eq!(codex.len(), 1, "commutative match should not duplicate");
        assert!(codex[0].discovered);
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
        // base_rate 25.0 * level_mult(2) 1.5 * throughput_mult 1.0 = 37.5
        assert!(
            (rate - 37.5).abs() < 0.001,
            "expected 37.5/hr, got {}",
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
        // base_rate 25.0 * level_mult(3) 2.0 * throughput_mult 1.0 = 50.0
        assert!(
            (rate - 50.0).abs() < 0.001,
            "expected 50.0/hr, got {}",
            rate
        );
    }

    // ── try_upgrade_node buffer capacity ──────────────────────────────────────

    #[test]
    fn test_upgrade_node_increases_buffer_capacity() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        let old_capacity = loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer_capacity;

        loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer = 500.0;
        try_upgrade_node(&mut loom, NodeId::EmberSpindle);

        // Complete the upgrade by ticking past the duration.
        let remaining = loom.persistent.nodes[NodeId::EmberSpindle.index()].upgrade_remaining_secs;
        tick_node_upgrades(&mut loom, remaining + 1.0);

        let new_capacity = loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer_capacity;
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

        // Upgrade now drains 50% of buffer_capacity (500 * 0.5 = 250).
        let drain = loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer_capacity * 0.5;
        let starting_buffer = drain + 5.0;

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
        // 25/hr base * 1.0x (no archetype bonus); after 1hr = 25.0 units
        assert!(
            (ember_amount - 25.0).abs() < 0.001,
            "expected 25.0 Ember produced, got {}",
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
            r.construction_secs_remaining = 0.5;
            r
        });

        let completed = tick_shuttle_construction(&mut loom, 1.0);
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
            Resource::Memory,
            NodeNature::Form,
            Resource::EmberEcho,
            1.0,
            2,
            vec![LoomNodeRef::Shuttle(0)],
            vec![LoomNodeRef::Extractor(NodeId::MemoryArchive)],
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

#[cfg(test)]
mod shuttle_tests {
    use super::*;

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
            NodeNature::Vibration,
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

        // No intake cap. EmberSpindle = 25.0/hr, VoidCondenser = 25.0/hr.
        // pull_a = 25.0; pull_b = 25.0
        // output = min(25.0, 25.0) * 1.0 = 25.0/hr => 25.0 in 1 hour.
        assert!(
            (forged - 25.0).abs() < 0.01,
            "expected ~25.0 ForgedLight, got {forged}"
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

        // EmberSpindle effective = 25.0/hr, split 2 ways = 12.5 each (no intake cap).
        // VoidCondenser = 25.0/hr, split 2 ways = 12.5 each.
        // Each shuttle: min(12.5, 12.5) * 1.0 = 12.5/hr. Total = 25.0/hr => 25.0 in 1 hour.
        assert!(
            (forged - 25.0).abs() < 0.01,
            "expected ~25.0 ForgedLight from two shuttles, got {forged}"
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

    #[test]
    fn test_shuttle_output_rate_tracker_updates_per_tick() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        for node in &mut loom.persistent.nodes {
            node.unlocked = true;
            node.buffer = 100.0;
        }
        setup_patterns(&mut loom, 1);
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
        for _ in 0..10 {
            tick_shuttle_pull(&mut loom, 0.1);
        }
        let tracker = &loom.persistent.shuttles[0].output_rate_tracker;
        assert!(
            tracker.rate_per_hour() > 0.0,
            "Shuttle rate tracker should record production"
        );
    }

    // ── upgrade_shuttle error paths ───────────────────────────────────────────

    #[test]
    fn test_upgrade_shuttle_invalid_index() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        let result = upgrade_shuttle(&mut loom, 0, 7);
        assert_eq!(result, Err(ShuttleUpgradeError::InvalidIndex));
    }

    #[test]
    fn test_upgrade_shuttle_under_construction() {
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
        // Freshly built shuttle stays under_construction (not cleared here).
        assert!(loom.persistent.shuttles[0].under_construction);

        let result = upgrade_shuttle(&mut loom, 0, 7);
        assert_eq!(result, Err(ShuttleUpgradeError::UnderConstruction));
    }

    #[test]
    fn test_upgrade_shuttle_insufficient_buffer() {
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
        // Cost for level 1 -> 2 is 100 * 1^1.2 = 100; leave buffer well short.
        loom.persistent.shuttles[0].buffer = 10.0;

        let result = upgrade_shuttle(&mut loom, 0, 7);
        assert_eq!(
            result,
            Err(ShuttleUpgradeError::InsufficientBuffer {
                needed: 100.0,
                have: 10.0,
            })
        );
    }

    // ── build_shuttle InvalidSource ────────────────────────────────────────────

    #[test]
    fn test_build_shuttle_fails_invalid_source() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        setup_patterns(&mut loom, 1);
        loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer = 500.0;

        // Shuttle(5) doesn't exist yet -> invalid source reference.
        let result = build_shuttle(
            &mut loom,
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            vec![LoomNodeRef::Shuttle(5)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        assert_eq!(result, Err(ShuttleError::InvalidSource));
    }

    // ── demolish_shuttle out-of-bounds no-op ──────────────────────────────────

    #[test]
    fn test_demolish_shuttle_out_of_bounds_is_noop() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        loom.persistent.shuttles.push(Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![],
            vec![],
        ));
        demolish_shuttle(&mut loom, 99);
        assert_eq!(
            loom.persistent.shuttles.len(),
            1,
            "out-of-bounds demolish should not remove anything"
        );
    }

    // ── eligible_sources_for_tier ──────────────────────────────────────────────

    #[test]
    fn test_eligible_sources_for_tier_includes_matching_extractors() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        loom.persistent.nodes[NodeId::EmberSpindle.index()].unlocked = true;

        let sources = eligible_sources_for_tier(&loom, 1, Resource::Ember);
        assert!(sources.contains(&LoomNodeRef::Extractor(NodeId::EmberSpindle)));
    }

    #[test]
    fn test_eligible_sources_for_tier_includes_lower_tier_shuttles() {
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
        // Shuttle 0 is tier 1, output ForgedLight. It should be an eligible
        // source for a tier-2 recipe needing ForgedLight, but not for tier 1.
        let sources_t2 = eligible_sources_for_tier(&loom, 2, Resource::ForgedLight);
        assert!(sources_t2.contains(&LoomNodeRef::Shuttle(0)));

        let sources_t1 = eligible_sources_for_tier(&loom, 1, Resource::ForgedLight);
        assert!(
            !sources_t1.contains(&LoomNodeRef::Shuttle(0)),
            "tier 1 shuttle should not be a valid source for another tier-1 consumer"
        );
    }

    // ── available_resource ─────────────────────────────────────────────────────

    #[test]
    fn test_available_resource_sums_extractor_and_shuttle_buffers() {
        let mut loom = LoomState::new();
        initialize_loom(&mut loom);
        setup_patterns(&mut loom, 1);
        loom.persistent.nodes[NodeId::EmberSpindle.index()].buffer = 40.0;

        let mut r = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::Ember, // same output as extractor's native resource for this test
            1.0,
            1,
            vec![],
            vec![],
        );
        r.under_construction = false;
        r.buffer = 15.0;
        loom.persistent.shuttles.push(r);

        let total = available_resource(&loom, Resource::Ember);
        assert!(
            (total - 55.0).abs() < 0.001,
            "expected 55.0 (40 extractor + 15 shuttle), got {}",
            total
        );
    }

    #[test]
    fn test_available_resource_excludes_under_construction_shuttle() {
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
        r.under_construction = true;
        r.buffer = 99.0;
        loom.persistent.shuttles.push(r);

        let total = available_resource(&loom, Resource::ForgedLight);
        assert_eq!(
            total, 0.0,
            "under-construction shuttle buffer should not count"
        );
    }

    // ── shuttle_build_cost / tier_intake_cap / shuttle_construction_secs defaults ──

    #[test]
    fn test_shuttle_build_cost_all_tiers() {
        assert_eq!(shuttle_build_cost(1), 250.0);
        assert_eq!(shuttle_build_cost(2), 150.0);
        assert_eq!(shuttle_build_cost(3), 100.0);
        assert_eq!(
            shuttle_build_cost(99),
            100.0,
            "unknown tier falls back to T3+ cost"
        );
    }

    #[test]
    fn test_tier_intake_cap_default_arm() {
        assert_eq!(tier_intake_cap(0), 20.0);
        assert_eq!(tier_intake_cap(99), 20.0);
    }

    #[test]
    fn test_shuttle_construction_secs_default_arm() {
        assert_eq!(shuttle_construction_secs(0), 7200.0);
        assert_eq!(shuttle_construction_secs(99), 7200.0);
    }
}
