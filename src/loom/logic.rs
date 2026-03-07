#![allow(dead_code)]
use super::types::{LoomArchetype, LoomNode, LoomState, NodeId, Resource};

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
    loom.persistent.archetype = Some(archetype);

    let (first, _second) = archetype_nodes(archetype);

    // Unlock and apply passives to the first node.
    if let Some(node) = loom.persistent.nodes.iter_mut().find(|n| n.id == first) {
        node.unlocked = true;
        apply_node_passive_on_unlock(node.id, loom);
    }

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

// ── Phase 4: Pipe Building ────────────────────────────────────────────────────

/// Maximum pipes allowed in or out of a single node.
pub const MAX_PIPES_PER_DIRECTION: usize = 3;

/// Ticks of construction time for a new pipe (2 hours at 10 ticks/sec).
pub const PIPE_CONSTRUCTION_TICKS: u32 = 72_000;

/// Cost (in the source node's native resource) to build a new T1 pipe.
pub const PIPE_BUILD_COST: f64 = 20.0;

/// Reasons a pipe build can fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeBuildError {
    /// Source node does not exist or is not yet unlocked.
    SourceNotUnlocked,
    /// Destination node does not exist or is not yet unlocked.
    DestNotUnlocked,
    /// Cannot build a pipe from a node to itself.
    SameNode,
    /// A pipe already exists between these two nodes in this direction.
    AlreadyExists,
    /// Source node already has the maximum number of outgoing pipes.
    TooManyOutgoing,
    /// Destination node already has the maximum number of incoming pipes.
    TooManyIncoming,
    /// Source node's buffer does not have enough resources to pay the build cost.
    InsufficientResources,
}

/// Attempt to begin construction of a new T1 pipe from `from` to `to`.
///
/// Deducts `PIPE_BUILD_COST` from the source node's buffer immediately.
/// The pipe is placed in `loom.persistent.pipes` with `under_construction = true`
/// and `construction_ticks_remaining = PIPE_CONSTRUCTION_TICKS`.
///
/// Returns `Ok(())` on success or a `PipeBuildError` describing why it failed.
pub fn try_build_pipe(
    loom: &mut LoomState,
    from: NodeId,
    to: NodeId,
) -> Result<(), PipeBuildError> {
    use super::types::{Pipe, PipeTier};

    if from == to {
        return Err(PipeBuildError::SameNode);
    }

    // Validate source is unlocked.
    let src_unlocked = loom
        .persistent
        .nodes
        .iter()
        .any(|n| n.id == from && n.unlocked);
    if !src_unlocked {
        return Err(PipeBuildError::SourceNotUnlocked);
    }

    // Validate destination is unlocked.
    let dst_unlocked = loom
        .persistent
        .nodes
        .iter()
        .any(|n| n.id == to && n.unlocked);
    if !dst_unlocked {
        return Err(PipeBuildError::DestNotUnlocked);
    }

    // Check for existing pipe in this direction.
    if loom
        .persistent
        .pipes
        .iter()
        .any(|p| p.from == from && p.to == to)
    {
        return Err(PipeBuildError::AlreadyExists);
    }

    // Count outgoing pipes from source.
    let outgoing = loom
        .persistent
        .pipes
        .iter()
        .filter(|p| p.from == from)
        .count();
    if outgoing >= MAX_PIPES_PER_DIRECTION {
        return Err(PipeBuildError::TooManyOutgoing);
    }

    // Count incoming pipes to destination.
    let incoming = loom.persistent.pipes.iter().filter(|p| p.to == to).count();
    if incoming >= MAX_PIPES_PER_DIRECTION {
        return Err(PipeBuildError::TooManyIncoming);
    }

    // Deduct build cost from source buffer.
    let src_node = loom
        .persistent
        .nodes
        .iter_mut()
        .find(|n| n.id == from)
        .unwrap(); // safe: we verified it exists above
    if src_node.buffer < PIPE_BUILD_COST {
        return Err(PipeBuildError::InsufficientResources);
    }
    src_node.buffer -= PIPE_BUILD_COST;

    // Add the pipe in construction state.
    loom.persistent.pipes.push(Pipe {
        from,
        to,
        tier: PipeTier::T1,
        split_ratio: 1.0,
        under_construction: true,
        construction_ticks_remaining: PIPE_CONSTRUCTION_TICKS,
    });

    // Rebalance split ratios for the source node's outgoing pipes equally.
    rebalance_split_ratios(loom, from);

    Ok(())
}

/// Rebalance split ratios for all outgoing pipes from `node_id` to divide 1.0 equally.
pub fn rebalance_split_ratios(loom: &mut LoomState, node_id: NodeId) {
    let count = loom
        .persistent
        .pipes
        .iter()
        .filter(|p| p.from == node_id)
        .count();
    if count == 0 {
        return;
    }
    let equal_share = 1.0 / count as f64;
    for pipe in loom.persistent.pipes.iter_mut() {
        if pipe.from == node_id {
            pipe.split_ratio = equal_share;
        }
    }
}

/// Tick construction progress for all pipes under construction.
/// Each call advances them by one tick. When `construction_ticks_remaining`
/// reaches zero the pipe becomes active.
///
/// Returns the list of pipes (from, to) that just completed construction.
pub fn tick_pipe_construction(loom: &mut LoomState) -> Vec<(NodeId, NodeId)> {
    let mut completed = Vec::new();
    for pipe in loom.persistent.pipes.iter_mut() {
        if !pipe.under_construction {
            continue;
        }
        if pipe.construction_ticks_remaining > 0 {
            pipe.construction_ticks_remaining -= 1;
        }
        if pipe.construction_ticks_remaining == 0 {
            pipe.under_construction = false;
            completed.push((pipe.from, pipe.to));
        }
    }
    completed
}

/// Demolish an existing pipe (by from/to). Refunds 50% of build cost to source buffer.
/// Returns true if the pipe was found and removed.
pub fn demolish_pipe(loom: &mut LoomState, from: NodeId, to: NodeId) -> bool {
    let idx = loom
        .persistent
        .pipes
        .iter()
        .position(|p| p.from == from && p.to == to);
    let Some(idx) = idx else {
        return false;
    };

    loom.persistent.pipes.remove(idx);

    // Refund 50% of build cost to source node's buffer (no cap — refund bypasses normal limits).
    if let Some(src) = loom.persistent.nodes.iter_mut().find(|n| n.id == from) {
        src.buffer += PIPE_BUILD_COST * 0.5;
    }

    // Rebalance remaining outgoing pipes.
    rebalance_split_ratios(loom, from);

    true
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

    // ── Phase 4: Pipe Building tests ──────────────────────────────────────────

    fn loom_with_two_unlocked_nodes() -> LoomState {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        // Unlock VoidCondenser manually (second archetype node).
        tick_loom_staggered_unlock(&mut loom, SECOND_NODE_UNLOCK_SECONDS);
        // Give EmberSpindle enough buffer and capacity to pay build cost and receive refunds.
        let ember = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        ember.buffer_capacity = 200.0;
        ember.buffer = 100.0;
        loom
    }

    #[test]
    fn test_build_pipe_succeeds() {
        let mut loom = loom_with_two_unlocked_nodes();
        let result = try_build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser);
        assert_eq!(result, Ok(()));
        assert_eq!(loom.persistent.pipes.len(), 1);
        assert!(loom.persistent.pipes[0].under_construction);
    }

    #[test]
    fn test_build_pipe_deducts_cost() {
        let mut loom = loom_with_two_unlocked_nodes();
        let before = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer;
        try_build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser).unwrap();
        let after = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer;
        assert!((before - after - PIPE_BUILD_COST).abs() < 0.001);
    }

    #[test]
    fn test_build_pipe_fails_same_node() {
        let mut loom = loom_with_two_unlocked_nodes();
        let result = try_build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::EmberSpindle);
        assert_eq!(result, Err(PipeBuildError::SameNode));
    }

    #[test]
    fn test_build_pipe_fails_locked_destination() {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        // VoidCondenser still locked.
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer = 100.0;
        let result = try_build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser);
        assert_eq!(result, Err(PipeBuildError::DestNotUnlocked));
    }

    #[test]
    fn test_build_pipe_fails_duplicate() {
        let mut loom = loom_with_two_unlocked_nodes();
        try_build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser).unwrap();
        // Restore buffer.
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer = 100.0;
        let result = try_build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser);
        assert_eq!(result, Err(PipeBuildError::AlreadyExists));
    }

    #[test]
    fn test_build_pipe_fails_insufficient_resources() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer = 0.0;
        let result = try_build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser);
        assert_eq!(result, Err(PipeBuildError::InsufficientResources));
    }

    #[test]
    fn test_pipe_construction_completes_after_ticks() {
        let mut loom = loom_with_two_unlocked_nodes();
        try_build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser).unwrap();
        assert!(loom.persistent.pipes[0].under_construction);

        // Run all construction ticks.
        for _ in 0..PIPE_CONSTRUCTION_TICKS {
            tick_pipe_construction(&mut loom);
        }

        assert!(!loom.persistent.pipes[0].under_construction);
    }

    #[test]
    fn test_demolish_pipe_removes_and_refunds() {
        let mut loom = loom_with_two_unlocked_nodes();
        try_build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser).unwrap();
        let buffer_after_build = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer;

        let removed = demolish_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser);
        assert!(removed);
        assert!(loom.persistent.pipes.is_empty());

        let buffer_after_demolish = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer;
        let refund = PIPE_BUILD_COST * 0.5;
        assert!((buffer_after_demolish - buffer_after_build - refund).abs() < 0.001);
    }

    #[test]
    fn test_split_ratios_rebalanced_on_build() {
        let mut loom = loom_with_two_unlocked_nodes();
        // Unlock a third node to pipe to two destinations.
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::ReflectionLens)
            .unwrap()
            .unlocked = true;

        try_build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser).unwrap();
        // Restore buffer.
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer = 100.0;
        try_build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::ReflectionLens).unwrap();

        // Both outgoing pipes should share 50% each.
        for pipe in loom
            .persistent
            .pipes
            .iter()
            .filter(|p| p.from == NodeId::EmberSpindle)
        {
            assert!(
                (pipe.split_ratio - 0.5).abs() < 0.001,
                "expected 0.5 split, got {}",
                pipe.split_ratio
            );
        }
    }
}
