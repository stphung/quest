//! Zone and level progression simulation.

use super::loot_sim::LootStats;

/// XP required to level up.
pub fn xp_for_level(level: u32) -> u64 {
    // Simple exponential curve
    (100.0 * (1.1_f64).powi(level as i32)) as u64
}

/// Simulated game progression state.
#[derive(Debug, Clone)]
pub struct SimProgression {
    pub player_level: u32,
    pub current_xp: u64,
    pub xp_to_next_level: u64,
    pub current_zone: usize,
    pub current_floor: u32,
    pub kills_on_floor: u32,
    pub prestige_rank: u32,
    pub total_kills: u64,
    pub total_boss_kills: u64,
    pub total_deaths: u64,
    pub total_ticks: u64,
}

impl SimProgression {
    pub fn new() -> Self {
        Self {
            player_level: 1,
            current_xp: 0,
            xp_to_next_level: xp_for_level(1),
            current_zone: 1,
            current_floor: 1,
            kills_on_floor: 0,
            prestige_rank: 0,
            total_kills: 0,
            total_boss_kills: 0,
            total_deaths: 0,
            total_ticks: 0,
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

    /// Record a kill and advance floor if needed.
    pub fn record_kill(&mut self, was_boss: bool) {
        self.total_kills += 1;
        self.kills_on_floor += 1;

        if was_boss {
            self.total_boss_kills += 1;
            // Boss defeated = advance to next floor or zone
            self.advance_floor();
        } else if self.kills_on_floor >= 10 {
            // Every 10 kills on a floor, spawn boss (simplified)
            // Handled by caller
        }
    }

    /// Record a death.
    pub fn record_death(&mut self) {
        self.total_deaths += 1;
        // Reset to floor start (simplified: just reset kills)
        self.kills_on_floor = 0;
    }

    /// Advance to next floor or zone.
    fn advance_floor(&mut self) {
        self.current_floor += 1;
        self.kills_on_floor = 0;

        // Every 10 floors = new zone (simplified)
        if self.current_floor > 10 {
            if self.current_zone < 10 {
                self.current_zone += 1;
                self.current_floor = 1;
            } else {
                // At zone 10, stay at floor 10 (endgame farming)
                self.current_floor = 10;
            }
        }
    }

    /// Get the monster level for current location.
    pub fn current_monster_level(&self) -> u32 {
        let base = ((self.current_zone - 1) * 10) as u32;
        base + self.current_floor
    }

    /// Check if should spawn boss.
    pub fn should_spawn_boss(&self) -> bool {
        self.kills_on_floor >= 10
    }

    /// Perform prestige reset.
    pub fn prestige(&mut self) {
        self.prestige_rank += 1;
        self.player_level = 1;
        self.current_xp = 0;
        self.xp_to_next_level = xp_for_level(1);
        self.current_zone = 1;
        self.current_floor = 1;
        self.kills_on_floor = 0;
    }

    /// Check if can prestige (reached zone 10).
    pub fn can_prestige(&self) -> bool {
        self.current_zone >= 10
    }
}

impl Default for SimProgression {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a simulation run.
#[derive(Debug, Clone, Default)]
pub struct RunStats {
    pub final_level: u32,
    pub final_zone: usize,
    pub final_prestige: u32,
    pub total_kills: u64,
    pub total_boss_kills: u64,
    pub total_deaths: u64,
    pub total_ticks: u64,
    pub loot_stats: LootStats,
    pub final_avg_ilvl: f64,
    pub reached_target: bool,
}

/// Zone-specific statistics.
#[derive(Debug, Clone, Default)]
pub struct ZoneStats {
    pub zone_id: usize,
    pub times_entered: u32,
    pub kills_in_zone: u64,
    pub deaths_in_zone: u64,
    pub ticks_in_zone: u64,
    pub avg_player_level_on_entry: f64,
    pub avg_gear_ilvl_on_entry: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_curve() {
        assert_eq!(xp_for_level(1), 110);
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

        // Kill 10 mobs + boss to advance floor
        for _ in 0..10 {
            prog.record_kill(false);
        }
        prog.record_kill(true); // Boss kill advances floor

        assert_eq!(prog.current_floor, 2);
    }
}
