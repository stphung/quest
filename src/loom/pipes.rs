/// Phase 4: Pipe Data Model, Flow Simulation, Split Ratio UI, Upgrading & Demolishing.
///
/// Pipes are directional connections between nodes. Each pipe has a bandwidth tier,
/// a split ratio (fraction of the source node's output it carries), and can be
/// under construction for 2 hours (7200 ticks × 0.1s).
///
/// Rules:
/// - Max 3 outgoing, 3 incoming per node.
/// - Split ratios sum to ≤ 1.0 per source node; remainder goes to the node's own buffer.
/// - Construction costs are resource-based; demolishing refunds 50%.
/// - Bandwidth upgrade is instant; pipe building takes 2 hours construction.
use super::types::{LoomState, NodeId, Pipe, PipeTier};

// ── Construction constants ────────────────────────────────────────────────────

/// Ticks required for a pipe to finish construction (2 hours at 100ms/tick = 72000 ticks).
pub const PIPE_CONSTRUCTION_TICKS: u32 = 72_000;

/// Resource cost to build a T1 pipe (paid from the source node's buffer).
pub const PIPE_BUILD_COST_T1: f64 = 10.0;

/// Upgrade costs per tier (from T1→T2, T2→T3, T3→T4).
pub const PIPE_UPGRADE_COSTS: [f64; 3] = [15.0, 25.0, 40.0];

// ── Validation helpers ────────────────────────────────────────────────────────

/// Returns the total number of outgoing pipes from `node_id` (including under construction).
/// Used for the max-3-outgoing capacity check.
pub fn outgoing_pipe_count(loom: &LoomState, node_id: NodeId) -> usize {
    loom.persistent
        .pipes
        .iter()
        .filter(|p| p.from == node_id)
        .count()
}

/// Returns the total number of incoming pipes to `node_id` (including under construction).
/// Used for the max-3-incoming capacity check.
pub fn incoming_pipe_count(loom: &LoomState, node_id: NodeId) -> usize {
    loom.persistent
        .pipes
        .iter()
        .filter(|p| p.to == node_id)
        .count()
}

/// Returns the number of active (non-construction) outgoing pipes from `node_id`.
pub fn active_outgoing_pipe_count(loom: &LoomState, node_id: NodeId) -> usize {
    loom.persistent
        .pipes
        .iter()
        .filter(|p| p.from == node_id && !p.under_construction)
        .count()
}

/// Returns true if a pipe already exists between `from` and `to` (regardless of direction).
pub fn pipe_exists(loom: &LoomState, from: NodeId, to: NodeId) -> bool {
    loom.persistent
        .pipes
        .iter()
        .any(|p| p.from == from && p.to == to)
}

/// Returns the sum of split ratios for all outgoing pipes from `node_id`.
pub fn total_split_ratio(loom: &LoomState, node_id: NodeId) -> f64 {
    loom.persistent
        .pipes
        .iter()
        .filter(|p| p.from == node_id && !p.under_construction)
        .map(|p| p.split_ratio)
        .sum()
}

// ── Pipe building ─────────────────────────────────────────────────────────────

/// Error conditions for pipe operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeError {
    /// Source or destination node is not unlocked.
    NodeNotUnlocked,
    /// Source and destination are the same node.
    SelfLoop,
    /// A pipe already exists between these two nodes.
    AlreadyExists,
    /// Source node already has 3 outgoing pipes.
    TooManyOutgoing,
    /// Destination node already has 3 incoming pipes.
    TooManyIncoming,
    /// Source node's buffer doesn't have enough resources for the build cost.
    InsufficientResources,
    /// The referenced pipe index does not exist.
    PipeNotFound,
    /// Split ratio adjustment would result in negative or >1.0 ratios.
    InvalidSplitRatio,
}

/// Attempt to start building a pipe from `from` to `to`.
///
/// Deducts `PIPE_BUILD_COST_T1` from the source node's buffer.
/// The pipe starts with `under_construction = true` and a default split ratio of 0.0.
/// The caller should call `normalize_split_ratios()` after construction completes.
///
/// Returns the index of the newly created pipe in `loom.persistent.pipes`, or an error.
pub fn build_pipe(loom: &mut LoomState, from: NodeId, to: NodeId) -> Result<usize, PipeError> {
    // Validate nodes.
    let from_node = loom
        .persistent
        .nodes
        .iter()
        .find(|n| n.id == from)
        .ok_or(PipeError::NodeNotUnlocked)?;
    if !from_node.unlocked {
        return Err(PipeError::NodeNotUnlocked);
    }
    let to_node = loom
        .persistent
        .nodes
        .iter()
        .find(|n| n.id == to)
        .ok_or(PipeError::NodeNotUnlocked)?;
    if !to_node.unlocked {
        return Err(PipeError::NodeNotUnlocked);
    }

    if from == to {
        return Err(PipeError::SelfLoop);
    }
    if pipe_exists(loom, from, to) {
        return Err(PipeError::AlreadyExists);
    }
    if outgoing_pipe_count(loom, from) >= 3 {
        return Err(PipeError::TooManyOutgoing);
    }
    if incoming_pipe_count(loom, to) >= 3 {
        return Err(PipeError::TooManyIncoming);
    }

    // Deduct build cost from source buffer.
    let from_node = loom
        .persistent
        .nodes
        .iter_mut()
        .find(|n| n.id == from)
        .unwrap();
    if from_node.buffer < PIPE_BUILD_COST_T1 {
        return Err(PipeError::InsufficientResources);
    }
    from_node.buffer -= PIPE_BUILD_COST_T1;

    let pipe = Pipe {
        from,
        to,
        tier: PipeTier::T1,
        split_ratio: 0.0, // Will be set after construction completes.
        under_construction: true,
        construction_ticks_remaining: PIPE_CONSTRUCTION_TICKS,
    };

    loom.persistent.pipes.push(pipe);
    Ok(loom.persistent.pipes.len() - 1)
}

// ── Construction ticking ──────────────────────────────────────────────────────

/// Tick construction progress for all pipes under construction.
///
/// Decrements `construction_ticks_remaining` by 1 each call.
/// When a pipe completes, sets `under_construction = false` and assigns
/// an auto-balanced split ratio (equal share of remaining unallocated ratio).
///
/// Returns the indices of pipes that completed construction this tick.
pub fn tick_pipe_construction(loom: &mut LoomState) -> Vec<usize> {
    let mut completed: Vec<usize> = Vec::new();

    for (i, pipe) in loom.persistent.pipes.iter_mut().enumerate() {
        if !pipe.under_construction {
            continue;
        }
        pipe.construction_ticks_remaining = pipe.construction_ticks_remaining.saturating_sub(1);
        if pipe.construction_ticks_remaining == 0 {
            pipe.under_construction = false;
            completed.push(i);
        }
    }

    // Auto-assign split ratios for newly completed pipes.
    for &idx in &completed {
        let from = loom.persistent.pipes[idx].from;
        auto_balance_split_ratio(loom, from, idx);
    }

    completed
}

/// Assign a balanced split ratio to a newly completed pipe.
/// Distributes the remaining unallocated ratio equally among pipes with ratio 0.
fn auto_balance_split_ratio(loom: &mut LoomState, from: NodeId, new_pipe_idx: usize) {
    let allocated: f64 = loom
        .persistent
        .pipes
        .iter()
        .enumerate()
        .filter(|(i, p)| p.from == from && !p.under_construction && *i != new_pipe_idx)
        .map(|(_, p)| p.split_ratio)
        .sum();

    let remaining = (1.0_f64 - allocated).max(0.0);
    if let Some(pipe) = loom.persistent.pipes.get_mut(new_pipe_idx) {
        pipe.split_ratio = remaining;
    }
}

// ── Flow simulation ───────────────────────────────────────────────────────────

/// Tick pipe flow for all active (non-construction) pipes.
///
/// For each active pipe, transfers resources from the source node's buffer
/// to the destination node's buffer, limited by:
/// 1. The pipe's bandwidth cap (tier rate × delta_hours)
/// 2. The source node's allocated flow (split_ratio × source_rate × delta_hours)
/// 3. Available space in the destination buffer
///
/// `delta_seconds` is the wall-clock elapsed time (typically 0.1s per tick).
pub fn tick_pipe_flow(loom: &mut LoomState, delta_seconds: f64) {
    let delta_hours = delta_seconds / 3600.0;

    // Snapshot source rates and buffer states to avoid borrow conflicts.
    let node_states: Vec<(NodeId, f64, f64, f64)> = loom
        .persistent
        .nodes
        .iter()
        .map(|n| {
            let rate = crate::loom::logic::node_effective_rate(loom, n);
            (n.id, rate, n.buffer, n.buffer_capacity)
        })
        .collect();

    // Build transfer list: (pipe_idx, from_node_idx, to_node_idx, transfer_amount)
    let mut transfers: Vec<(usize, usize, usize, f64)> = Vec::new();

    for (pipe_idx, pipe) in loom.persistent.pipes.iter().enumerate() {
        if pipe.under_construction {
            continue;
        }

        let from_idx = match node_states
            .iter()
            .position(|(id, _, _, _)| *id == pipe.from)
        {
            Some(i) => i,
            None => continue,
        };
        let to_idx = match node_states.iter().position(|(id, _, _, _)| *id == pipe.to) {
            Some(i) => i,
            None => continue,
        };

        let (_, from_rate, from_buffer, _) = node_states[from_idx];
        let (_, _, to_buffer, to_capacity) = node_states[to_idx];

        // Max this pipe can carry based on split ratio and source rate.
        let rate_limited = from_rate * pipe.split_ratio * delta_hours;
        // Max this pipe can carry based on bandwidth tier.
        let bandwidth_limited = pipe.tier.bandwidth() * delta_hours;
        // Max that fits in destination buffer.
        let space = (to_capacity - to_buffer).max(0.0);

        let transfer = rate_limited
            .min(bandwidth_limited)
            .min(from_buffer)
            .min(space);
        if transfer > 0.0 {
            transfers.push((pipe_idx, from_idx, to_idx, transfer));
        }
    }

    // Apply transfers.
    for (_, from_idx, to_idx, amount) in transfers {
        // Find node ids by index.
        let from_id = node_states[from_idx].0;
        let to_id = node_states[to_idx].0;

        if let Some(from_node) = loom.persistent.nodes.iter_mut().find(|n| n.id == from_id) {
            from_node.buffer = (from_node.buffer - amount).max(0.0);
        }
        if let Some(to_node) = loom.persistent.nodes.iter_mut().find(|n| n.id == to_id) {
            to_node.buffer = (to_node.buffer + amount).min(to_node.buffer_capacity);
        }
    }
}

// ── Split ratio adjustment ────────────────────────────────────────────────────

/// Adjust the split ratio of a pipe by `delta` (positive or negative).
///
/// Clamps the result so the pipe's ratio stays in [0.0, 1.0] and
/// the total ratio for the source node stays ≤ 1.0.
/// Adjustment step: 0.05 (5%).
///
/// Returns Ok(()) if the adjustment was applied, Err if the pipe was not found.
pub fn adjust_split_ratio(
    loom: &mut LoomState,
    pipe_idx: usize,
    delta: f64,
) -> Result<(), PipeError> {
    let pipe = loom
        .persistent
        .pipes
        .get(pipe_idx)
        .ok_or(PipeError::PipeNotFound)?;
    let from = pipe.from;

    // Total of other pipes from the same source (excluding this one).
    let other_total: f64 = loom
        .persistent
        .pipes
        .iter()
        .enumerate()
        .filter(|(i, p)| p.from == from && !p.under_construction && *i != pipe_idx)
        .map(|(_, p)| p.split_ratio)
        .sum();

    let current = loom.persistent.pipes[pipe_idx].split_ratio;
    let max_allowed = (1.0 - other_total).max(0.0);
    let new_ratio = (current + delta).clamp(0.0, max_allowed);

    loom.persistent.pipes[pipe_idx].split_ratio = new_ratio;
    Ok(())
}

/// Normalize split ratios for all outgoing pipes from `node_id` to sum to 1.0.
/// If total is 0, distributes equally. If total > 1.0, scales down proportionally.
pub fn normalize_split_ratios(loom: &mut LoomState, node_id: NodeId) {
    let indices: Vec<usize> = loom
        .persistent
        .pipes
        .iter()
        .enumerate()
        .filter(|(_, p)| p.from == node_id && !p.under_construction)
        .map(|(i, _)| i)
        .collect();

    if indices.is_empty() {
        return;
    }

    let total: f64 = indices
        .iter()
        .map(|&i| loom.persistent.pipes[i].split_ratio)
        .sum();

    if total <= 0.0 {
        let equal = 1.0 / indices.len() as f64;
        for &i in &indices {
            loom.persistent.pipes[i].split_ratio = equal;
        }
    } else if total > 1.0 {
        let factor = 1.0 / total;
        for &i in &indices {
            loom.persistent.pipes[i].split_ratio *= factor;
        }
    }
}

// ── Pipe upgrading ────────────────────────────────────────────────────────────

/// Upgrade a pipe's bandwidth tier (T1 → T2 → T3 → T4).
/// Cost is deducted from the source node's buffer in its native resource.
/// Returns Ok(()) on success.
pub fn upgrade_pipe(loom: &mut LoomState, pipe_idx: usize) -> Result<(), PipeError> {
    let pipe = loom
        .persistent
        .pipes
        .get(pipe_idx)
        .ok_or(PipeError::PipeNotFound)?;

    if pipe.under_construction {
        return Err(PipeError::PipeNotFound);
    }

    let upgrade_cost = match pipe.tier {
        PipeTier::T1 => PIPE_UPGRADE_COSTS[0],
        PipeTier::T2 => PIPE_UPGRADE_COSTS[1],
        PipeTier::T3 => PIPE_UPGRADE_COSTS[2],
        PipeTier::T4 => return Err(PipeError::PipeNotFound), // Already at max.
    };

    let from = pipe.from;

    let from_node = loom
        .persistent
        .nodes
        .iter_mut()
        .find(|n| n.id == from)
        .ok_or(PipeError::NodeNotUnlocked)?;

    if from_node.buffer < upgrade_cost {
        return Err(PipeError::InsufficientResources);
    }
    from_node.buffer -= upgrade_cost;

    let pipe = &mut loom.persistent.pipes[pipe_idx];
    pipe.tier = match pipe.tier {
        PipeTier::T1 => PipeTier::T2,
        PipeTier::T2 => PipeTier::T3,
        PipeTier::T3 => PipeTier::T4,
        PipeTier::T4 => PipeTier::T4,
    };

    Ok(())
}

// ── Pipe demolishing ──────────────────────────────────────────────────────────

/// Demolish a pipe and refund 50% of its build cost to the source node's buffer.
/// Removes the pipe from `loom.persistent.pipes`.
///
/// Returns Ok(refund_amount) on success.
pub fn demolish_pipe(loom: &mut LoomState, pipe_idx: usize) -> Result<f64, PipeError> {
    if pipe_idx >= loom.persistent.pipes.len() {
        return Err(PipeError::PipeNotFound);
    }

    let pipe = &loom.persistent.pipes[pipe_idx];
    let from = pipe.from;

    // Refund based on current tier's build equivalent cost.
    let base_cost = match pipe.tier {
        PipeTier::T1 => PIPE_BUILD_COST_T1,
        PipeTier::T2 => PIPE_BUILD_COST_T1 + PIPE_UPGRADE_COSTS[0],
        PipeTier::T3 => PIPE_BUILD_COST_T1 + PIPE_UPGRADE_COSTS[0] + PIPE_UPGRADE_COSTS[1],
        PipeTier::T4 => {
            PIPE_BUILD_COST_T1
                + PIPE_UPGRADE_COSTS[0]
                + PIPE_UPGRADE_COSTS[1]
                + PIPE_UPGRADE_COSTS[2]
        }
    };
    let refund = base_cost * 0.5;

    loom.persistent.pipes.remove(pipe_idx);

    if let Some(from_node) = loom.persistent.nodes.iter_mut().find(|n| n.id == from) {
        from_node.buffer = (from_node.buffer + refund).min(from_node.buffer_capacity);
    }

    Ok(refund)
}

// ── Resource flow rate query ──────────────────────────────────────────────────

/// Returns the effective flow rate through a given pipe in units/hour.
/// Accounts for split ratio and bandwidth cap.
pub fn pipe_flow_rate(loom: &LoomState, pipe_idx: usize) -> f64 {
    let pipe = match loom.persistent.pipes.get(pipe_idx) {
        Some(p) => p,
        None => return 0.0,
    };
    if pipe.under_construction {
        return 0.0;
    }

    let from_node = match loom.persistent.nodes.iter().find(|n| n.id == pipe.from) {
        Some(n) => n,
        None => return 0.0,
    };

    let from_rate = crate::loom::logic::node_effective_rate(loom, from_node);
    let rate_limited = from_rate * pipe.split_ratio;
    rate_limited.min(pipe.tier.bandwidth())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loom::{logic::select_archetype, types::LoomArchetype};

    fn loom_with_two_unlocked_nodes() -> LoomState {
        let mut loom = LoomState::new();
        select_archetype(&mut loom, LoomArchetype::BurnBright);
        // EmberSpindle is unlocked. Also unlock VoidCondenser manually for testing.
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::VoidCondenser)
            .unwrap()
            .unlocked = true;
        loom
    }

    // ── build_pipe ────────────────────────────────────────────────────────────

    #[test]
    fn test_build_pipe_success() {
        let mut loom = loom_with_two_unlocked_nodes();
        // Give EmberSpindle enough buffer for the build cost.
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer = 50.0;

        let result = build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser);
        assert!(result.is_ok());
        assert_eq!(loom.persistent.pipes.len(), 1);
        assert!(loom.persistent.pipes[0].under_construction);
        assert_eq!(
            loom.persistent.pipes[0].construction_ticks_remaining,
            PIPE_CONSTRUCTION_TICKS
        );
    }

    #[test]
    fn test_build_pipe_deducts_cost() {
        let mut loom = loom_with_two_unlocked_nodes();
        let ember = loom
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        ember.buffer = 50.0;

        build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser).unwrap();

        let ember = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        assert!((ember.buffer - (50.0 - PIPE_BUILD_COST_T1)).abs() < 0.001);
    }

    #[test]
    fn test_build_pipe_fails_self_loop() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer = 50.0;

        let result = build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::EmberSpindle);
        assert_eq!(result, Err(PipeError::SelfLoop));
    }

    #[test]
    fn test_build_pipe_fails_already_exists() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer = 100.0;

        build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser).unwrap();
        let result = build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser);
        assert_eq!(result, Err(PipeError::AlreadyExists));
    }

    #[test]
    fn test_build_pipe_fails_insufficient_resources() {
        let mut loom = loom_with_two_unlocked_nodes();
        // Buffer is 0 — can't afford build cost.

        let result = build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser);
        assert_eq!(result, Err(PipeError::InsufficientResources));
    }

    #[test]
    fn test_build_pipe_fails_too_many_outgoing() {
        let mut loom = LoomState::new();
        // Unlock all 6 nodes.
        for node in &mut loom.persistent.nodes {
            node.unlocked = true;
            node.buffer = 1000.0;
        }

        // Build 3 outgoing pipes from EmberSpindle.
        build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser).unwrap();
        build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::MemoryArchive).unwrap();
        build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::SilenceWell).unwrap();

        // 4th pipe should fail.
        let result = build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::ResonanceForge);
        assert_eq!(result, Err(PipeError::TooManyOutgoing));
    }

    #[test]
    fn test_build_pipe_fails_locked_destination() {
        let mut loom = LoomState::new();
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .unlocked = true;
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer = 50.0;
        // VoidCondenser remains locked.

        let result = build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser);
        assert_eq!(result, Err(PipeError::NodeNotUnlocked));
    }

    // ── tick_pipe_construction ────────────────────────────────────────────────

    #[test]
    fn test_pipe_construction_completes_after_ticks() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer = 50.0;

        build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser).unwrap();

        // Tick down to completion.
        loom.persistent.pipes[0].construction_ticks_remaining = 1;
        let completed = tick_pipe_construction(&mut loom);

        assert_eq!(completed, vec![0]);
        assert!(!loom.persistent.pipes[0].under_construction);
    }

    #[test]
    fn test_pipe_not_completed_before_ticks() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer = 50.0;

        build_pipe(&mut loom, NodeId::EmberSpindle, NodeId::VoidCondenser).unwrap();
        tick_pipe_construction(&mut loom); // one tick, still 71999 remaining

        assert!(loom.persistent.pipes[0].under_construction);
    }

    // ── tick_pipe_flow ────────────────────────────────────────────────────────

    #[test]
    fn test_pipe_flow_transfers_resources() {
        let mut loom = loom_with_two_unlocked_nodes();

        // Add an active (non-construction) pipe.
        loom.persistent.pipes.push(Pipe {
            from: NodeId::EmberSpindle,
            to: NodeId::VoidCondenser,
            tier: PipeTier::T1,
            split_ratio: 1.0,
            under_construction: false,
            construction_ticks_remaining: 0,
        });

        // Fill source buffer.
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer = 100.0;

        // Run flow for 1 hour.
        tick_pipe_flow(&mut loom, 3600.0);

        let void_n = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::VoidCondenser)
            .unwrap();
        // Flow rate = min(5/hr source * 1.5x * 1.0 ratio, T1 bandwidth 5/hr) = 5/hr
        // After 1 hour: 5 units transferred.
        assert!(
            void_n.buffer > 0.0,
            "destination buffer should have received resources"
        );
    }

    #[test]
    fn test_pipe_flow_capped_by_bandwidth() {
        let mut loom = LoomState::new();
        // Unlock both nodes with high production rate.
        for node in &mut loom.persistent.nodes {
            node.unlocked = true;
            node.buffer = 1000.0;
            node.base_rate = 100.0; // Very high rate
        }

        loom.persistent.pipes.push(Pipe {
            from: NodeId::EmberSpindle,
            to: NodeId::VoidCondenser,
            tier: PipeTier::T1, // 5/hr cap
            split_ratio: 1.0,
            under_construction: false,
            construction_ticks_remaining: 0,
        });

        let before = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::VoidCondenser)
            .unwrap()
            .buffer;

        tick_pipe_flow(&mut loom, 3600.0);

        let after = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::VoidCondenser)
            .unwrap()
            .buffer;
        let transferred = after - before;
        // Should be capped at T1 bandwidth (5/hr), not the full 100/hr rate.
        assert!(
            transferred <= 5.0 + 0.01,
            "transferred {} should be capped at T1 bandwidth 5/hr",
            transferred
        );
    }

    // ── adjust_split_ratio ────────────────────────────────────────────────────

    #[test]
    fn test_adjust_split_ratio_increases() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent.pipes.push(Pipe {
            from: NodeId::EmberSpindle,
            to: NodeId::VoidCondenser,
            tier: PipeTier::T1,
            split_ratio: 0.5,
            under_construction: false,
            construction_ticks_remaining: 0,
        });

        adjust_split_ratio(&mut loom, 0, 0.1).unwrap();
        assert!((loom.persistent.pipes[0].split_ratio - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_adjust_split_ratio_clamped_at_one() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent.pipes.push(Pipe {
            from: NodeId::EmberSpindle,
            to: NodeId::VoidCondenser,
            tier: PipeTier::T1,
            split_ratio: 0.9,
            under_construction: false,
            construction_ticks_remaining: 0,
        });

        adjust_split_ratio(&mut loom, 0, 0.5).unwrap(); // Would exceed 1.0
        assert!((loom.persistent.pipes[0].split_ratio - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_adjust_split_ratio_clamped_at_zero() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent.pipes.push(Pipe {
            from: NodeId::EmberSpindle,
            to: NodeId::VoidCondenser,
            tier: PipeTier::T1,
            split_ratio: 0.1,
            under_construction: false,
            construction_ticks_remaining: 0,
        });

        adjust_split_ratio(&mut loom, 0, -0.5).unwrap(); // Would go negative
        assert!((loom.persistent.pipes[0].split_ratio - 0.0).abs() < 0.001);
    }

    // ── upgrade_pipe ──────────────────────────────────────────────────────────

    #[test]
    fn test_upgrade_pipe_t1_to_t2() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent.pipes.push(Pipe {
            from: NodeId::EmberSpindle,
            to: NodeId::VoidCondenser,
            tier: PipeTier::T1,
            split_ratio: 1.0,
            under_construction: false,
            construction_ticks_remaining: 0,
        });
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer = 100.0;

        let result = upgrade_pipe(&mut loom, 0);
        assert!(result.is_ok());
        assert_eq!(loom.persistent.pipes[0].tier, PipeTier::T2);
    }

    #[test]
    fn test_upgrade_pipe_fails_at_t4() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent.pipes.push(Pipe {
            from: NodeId::EmberSpindle,
            to: NodeId::VoidCondenser,
            tier: PipeTier::T4,
            split_ratio: 1.0,
            under_construction: false,
            construction_ticks_remaining: 0,
        });
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer = 1000.0;

        let result = upgrade_pipe(&mut loom, 0);
        assert_eq!(result, Err(PipeError::PipeNotFound));
    }

    #[test]
    fn test_upgrade_pipe_fails_insufficient_resources() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent.pipes.push(Pipe {
            from: NodeId::EmberSpindle,
            to: NodeId::VoidCondenser,
            tier: PipeTier::T1,
            split_ratio: 1.0,
            under_construction: false,
            construction_ticks_remaining: 0,
        });
        // Buffer is 0 — can't afford upgrade.

        let result = upgrade_pipe(&mut loom, 0);
        assert_eq!(result, Err(PipeError::InsufficientResources));
    }

    // ── demolish_pipe ─────────────────────────────────────────────────────────

    #[test]
    fn test_demolish_pipe_removes_pipe() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent.pipes.push(Pipe {
            from: NodeId::EmberSpindle,
            to: NodeId::VoidCondenser,
            tier: PipeTier::T1,
            split_ratio: 1.0,
            under_construction: false,
            construction_ticks_remaining: 0,
        });

        let result = demolish_pipe(&mut loom, 0);
        assert!(result.is_ok());
        assert!(loom.persistent.pipes.is_empty());
    }

    #[test]
    fn test_demolish_pipe_refunds_50_percent() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent.pipes.push(Pipe {
            from: NodeId::EmberSpindle,
            to: NodeId::VoidCondenser,
            tier: PipeTier::T1,
            split_ratio: 1.0,
            under_construction: false,
            construction_ticks_remaining: 0,
        });
        loom.persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap()
            .buffer = 0.0;

        let refund = demolish_pipe(&mut loom, 0).unwrap();
        assert!((refund - PIPE_BUILD_COST_T1 * 0.5).abs() < 0.001);

        let ember = loom
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == NodeId::EmberSpindle)
            .unwrap();
        assert!((ember.buffer - refund).abs() < 0.001);
    }

    #[test]
    fn test_demolish_pipe_invalid_index() {
        let mut loom = LoomState::new();
        let result = demolish_pipe(&mut loom, 99);
        assert_eq!(result, Err(PipeError::PipeNotFound));
    }

    // ── pipe_flow_rate ────────────────────────────────────────────────────────

    #[test]
    fn test_pipe_flow_rate_under_construction_is_zero() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent.pipes.push(Pipe {
            from: NodeId::EmberSpindle,
            to: NodeId::VoidCondenser,
            tier: PipeTier::T1,
            split_ratio: 1.0,
            under_construction: true,
            construction_ticks_remaining: 1000,
        });

        assert_eq!(pipe_flow_rate(&loom, 0), 0.0);
    }

    #[test]
    fn test_pipe_flow_rate_active() {
        let mut loom = loom_with_two_unlocked_nodes();
        loom.persistent.pipes.push(Pipe {
            from: NodeId::EmberSpindle,
            to: NodeId::VoidCondenser,
            tier: PipeTier::T2, // 12/hr
            split_ratio: 0.5,
            under_construction: false,
            construction_ticks_remaining: 0,
        });

        // BurnBright EmberSpindle: 5/hr * 1.5x = 7.5/hr. 50% split = 3.75/hr.
        // T2 bandwidth = 12/hr. So effective = min(3.75, 12) = 3.75/hr.
        let rate = pipe_flow_rate(&loom, 0);
        assert!((rate - 3.75).abs() < 0.01, "expected 3.75/hr, got {}", rate);
    }
}
