//! Zone and level progression simulation using real game data.

use super::loot_sim::LootStats;
use crate::core::balance::xp_required_for_level;
use crate::core::progression::{can_access_zone, max_zone_for_prestige, Progression};
#[cfg(test)]
use crate::zones::get_all_zones;
use crate::zones::{get_zone, Zone};

/// XP required to level up - uses shared balance config.
pub fn xp_for_level(level: u32) -> u64 {
    xp_required_for_level(level)
}

/// Get zone data for simulation.
pub fn get_zone_data(zone_id: u32) -> Option<Zone> {
    get_zone(zone_id)
}

/// Get number of subzones in a zone.
pub fn subzones_in_zone(zone_id: u32) -> u32 {
    get_zone(zone_id)
        .map(|z| z.subzones.len() as u32)
        .unwrap_or(3)
}

/// Simulated game progression state.
#[derive(Debug, Clone)]
pub struct SimProgression {
    pub player_level: u32,
    pub current_xp: u64,
    pub xp_to_next_level: u64,
    pub current_zone: u32,
    pub current_subzone: u32,
    pub kills_in_subzone: u32,
    pub prestige_rank: u32,
    pub total_kills: u64,
    pub total_boss_kills: u64,
    pub total_deaths: u64,
    pub total_ticks: u64,

    // Per-zone tracking
    pub zone_entries: Vec<u64>, // Ticks when entering each zone
    pub zone_deaths: Vec<u64>,  // Deaths per zone
    pub zone_kills: Vec<u64>,   // Kills per zone
}

impl SimProgression {
    pub fn new() -> Self {
        Self {
            player_level: 1,
            current_xp: 0,
            xp_to_next_level: xp_for_level(1),
            current_zone: 1,
            current_subzone: 1,
            kills_in_subzone: 0,
            prestige_rank: 0,
            total_kills: 0,
            total_boss_kills: 0,
            total_deaths: 0,
            total_ticks: 0,
            zone_entries: vec![0; 11], // Index 0 unused, 1-10 for zones
            zone_deaths: vec![0; 11],
            zone_kills: vec![0; 11],
        }
    }

    /// Add XP and handle level ups.
    pub fn add_xp(&mut self, xp: u64) {
        self.current_xp += xp;

        while self.current_xp >= self.xp_to_next_level {
            self.current_xp -= self.xp_to_next_level;
            self.player_level += 1;
            self.xp_to_next_level = xp_for_level(self.player_level);
        }
    }

    /// Record a kill with simulation tracking.
    pub fn record_kill_sim(&mut self, was_boss: bool, current_ticks: u64) {
        self.total_kills += 1;

        if self.current_zone <= 10 {
            self.zone_kills[self.current_zone as usize] += 1;
        }

        // Use trait method for core kill tracking
        Progression::record_kill(self);

        if was_boss {
            self.total_boss_kills += 1;
            self.advance_after_boss_sim(current_ticks);
        }
    }

    /// Record a death with simulation tracking.
    /// If was_boss_fight is true, resets kill progress (matches real game).
    pub fn record_death_sim(&mut self, was_boss_fight: bool) {
        self.total_deaths += 1;
        if self.current_zone <= 10 {
            self.zone_deaths[self.current_zone as usize] += 1;
        }
        // Use trait implementation for core logic
        Progression::record_death(self, was_boss_fight);
    }

    /// Advance after defeating a boss (with tick tracking for sim).
    pub fn advance_after_boss_sim(&mut self, current_ticks: u64) {
        let old_zone = self.current_zone;

        // Use trait implementation for core advancement logic
        Progression::advance_after_boss(self);

        // Record zone entry time if we advanced
        if self.current_zone != old_zone && self.current_zone <= 10 {
            self.zone_entries[self.current_zone as usize] = current_ticks;
        }
    }

    /// Check if can prestige (level meets requirement for next rank).
    pub fn can_prestige(&self) -> bool {
        use crate::character::prestige::get_next_prestige_tier;
        self.player_level >= get_next_prestige_tier(self.prestige_rank).required_level
    }

    /// Perform prestige reset.
    pub fn prestige(&mut self) {
        self.prestige_rank += 1;
        self.player_level = 1;
        self.current_xp = 0;
        self.xp_to_next_level = xp_for_level(1);
        self.current_zone = 1;
        self.current_subzone = 1;
        self.kills_in_subzone = 0;
        // Keep equipment through prestige (in real game)
    }
}

impl Default for SimProgression {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement the shared Progression trait for SimProgression.
impl Progression for SimProgression {
    fn current_zone(&self) -> u32 {
        self.current_zone
    }

    fn current_subzone(&self) -> u32 {
        self.current_subzone
    }

    fn kills_in_subzone(&self) -> u32 {
        self.kills_in_subzone
    }

    fn prestige_rank(&self) -> u32 {
        self.prestige_rank
    }

    fn record_kill(&mut self) -> bool {
        self.kills_in_subzone += 1;
        self.should_spawn_boss()
    }

    fn record_death(&mut self, was_boss_fight: bool) {
        if was_boss_fight {
            self.kills_in_subzone = 0;
        }
    }

    fn advance_after_boss(&mut self) {
        self.kills_in_subzone = 0;

        let max_subzones = subzones_in_zone(self.current_zone);

        if self.current_subzone >= max_subzones {
            // Cleared final subzone, check if can advance to next zone
            if self.current_zone < 10 {
                let next_zone = self.current_zone + 1;

                // Use shared prestige check
                if can_access_zone(self.prestige_rank, next_zone) {
                    self.current_zone = next_zone;
                    self.current_subzone = 1;
                }
                // Else: prestige gated, stay in current zone
            }
            // At zone 10, stay in zone 10 (endgame)
        } else {
            // Advance to next subzone
            self.current_subzone += 1;
        }
    }

    fn at_max_zone_for_prestige(&self) -> bool {
        self.current_zone >= max_zone_for_prestige(self.prestige_rank)
    }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_curve() {
        assert!(xp_for_level(1) > 0);
        assert!(xp_for_level(50) > xp_for_level(1));
        assert!(xp_for_level(100) > xp_for_level(50));
    }

    #[test]
    fn test_progression_level_up() {
        let mut prog = SimProgression::new();
        let xp_needed = prog.xp_to_next_level;

        prog.add_xp(xp_needed);
        assert_eq!(prog.player_level, 2);
    }

    #[test]
    fn test_zone_advancement() {
        let mut prog = SimProgression::new();

        // Get actual subzones for zone 1
        let subzones = subzones_in_zone(1);

        // Kill mobs and bosses to advance through all subzones
        for _ in 0..subzones {
            for _ in 0..10 {
                prog.record_kill_sim(false, 0);
            }
            prog.record_kill_sim(true, 0); // Boss kill
        }

        assert_eq!(
            prog.current_zone, 2,
            "Should advance to zone 2 after clearing zone 1"
        );
    }

    #[test]
    fn test_real_zone_data() {
        let zones = get_all_zones();
        assert!(zones.len() >= 10, "Should have at least 10 zones");
        assert_eq!(zones[0].id, 1);
        // Verify zone 10 exists
        assert!(zones.iter().any(|z| z.id == 10), "Zone 10 should exist");
    }
}
