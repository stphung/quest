//! Simulation statistics types.
//!
//! This module contains types for tracking and reporting simulation results.
//! The actual game logic is now in CoreGame; these are just data structures.

use super::loot_sim::LootStats;

/// Tracks a single prestige cycle's metrics.
#[derive(Debug, Clone, Default)]
pub struct PrestigeCycle {
    pub rank: u32,
    pub ticks_to_complete: u64,
    pub final_level: u32,
    pub total_deaths: u64,
    pub total_kills: u64,
}

/// Statistics for a single simulation run.
#[derive(Debug, Clone, Default)]
pub struct RunStats {
    pub final_level: u32,
    pub final_zone: u32,
    pub final_subzone: u32,
    pub final_prestige: u32,
    pub total_kills: u64,
    pub total_boss_kills: u64,
    pub total_deaths: u64,
    pub total_ticks: u64,
    pub loot_stats: LootStats,
    pub final_avg_ilvl: f64,
    pub reached_target: bool,

    // Per-zone stats
    pub zone_deaths: Vec<u64>,
    pub zone_kills: Vec<u64>,
    pub ticks_per_zone: Vec<u64>,

    // Level-up pacing
    pub level_up_ticks: Vec<u64>,

    // Prestige cycles
    pub prestige_cycles: Vec<PrestigeCycle>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prestige_cycle_default() {
        let cycle = PrestigeCycle::default();
        assert_eq!(cycle.rank, 0);
        assert_eq!(cycle.ticks_to_complete, 0);
    }

    #[test]
    fn test_run_stats_default() {
        let stats = RunStats::default();
        assert_eq!(stats.final_level, 0);
        assert_eq!(stats.total_kills, 0);
    }
}
