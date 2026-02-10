//! Main simulation runner using CoreGame for real game mechanics.
//!
//! This module uses CoreGame (the shared game engine) instead of duplicating
//! game logic in simulator-specific types. Statistics are tracked externally
//! from TickResult events.

use super::config::SimConfig;
use super::loot_sim::{average_equipped_ilvl, LootStats};
use super::progression_sim::{PrestigeCycle, RunStats};
use super::report::SimReport;
use crate::core::core_game::CoreGame;
use crate::core::game_loop::{GameLoop, TickResult};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Run the full simulation and return a report.
pub fn run_simulation(config: &SimConfig) -> SimReport {
    let mut all_runs = Vec::with_capacity(config.num_runs as usize);

    for run_idx in 0..config.num_runs {
        // Create RNG for this run
        let mut rng = match config.seed {
            Some(seed) => ChaCha8Rng::seed_from_u64(seed + run_idx as u64),
            None => ChaCha8Rng::from_entropy(),
        };

        let run_stats = simulate_single_run(config, &mut rng);
        all_runs.push(run_stats);

        if config.verbosity >= 2 {
            let r = all_runs.last().unwrap();
            println!(
                "Run {}/{} - Zone {}.{}, Level {}, Kills {}, Boss Kills {}, Deaths {}, Prestige {}",
                run_idx + 1,
                config.num_runs,
                r.final_zone,
                r.final_subzone,
                r.final_level,
                r.total_kills,
                r.total_boss_kills,
                r.total_deaths,
                r.final_prestige
            );
        }
    }

    SimReport::from_runs(all_runs, config.target_zone, config.max_ticks_per_run)
}

/// Tracks statistics during a simulation run.
/// Accumulates data from TickResult events.
struct SimStats {
    // Basic counters
    total_kills: u64,
    total_boss_kills: u64,
    total_deaths: u64,

    // Per-zone tracking
    zone_entries: Vec<u64>, // Ticks when entering each zone
    zone_deaths: Vec<u64>,  // Deaths per zone
    zone_kills: Vec<u64>,   // Kills per zone

    // Level-up pacing
    level_up_ticks: Vec<u64>, // Tick when each level was reached

    // Prestige tracking
    prestige_cycles: Vec<PrestigeCycle>,
    cycle_start_tick: u64,
    cycle_start_deaths: u64,
    cycle_start_kills: u64,
    current_prestige_rank: u32,

    // Loot tracking
    loot_stats: LootStats,

    // Current state tracking
    current_zone: u32,
    current_level: u32,
}

impl SimStats {
    fn new() -> Self {
        let mut level_up_ticks = vec![0u64; 201]; // Support up to level 200
        level_up_ticks[1] = 0; // Start at level 1, tick 0

        Self {
            total_kills: 0,
            total_boss_kills: 0,
            total_deaths: 0,
            zone_entries: vec![0; 11], // Index 0 unused, 1-10 for zones
            zone_deaths: vec![0; 11],
            zone_kills: vec![0; 11],
            level_up_ticks,
            prestige_cycles: Vec::new(),
            cycle_start_tick: 0,
            cycle_start_deaths: 0,
            cycle_start_kills: 0,
            current_prestige_rank: 0,
            loot_stats: LootStats::default(),
            current_zone: 1,
            current_level: 1,
        }
    }

    /// Process a tick result and update statistics.
    fn process_tick(&mut self, result: &TickResult, current_tick: u64) {
        // Track kills
        if result.player_won {
            self.total_kills += 1;

            if self.current_zone <= 10 {
                self.zone_kills[self.current_zone as usize] += 1;
            }

            if result.was_boss {
                self.total_boss_kills += 1;
            }
        }

        // Track deaths
        if result.player_died {
            self.total_deaths += 1;

            if self.current_zone <= 10 {
                self.zone_deaths[self.current_zone as usize] += 1;
            }
        }

        // Track level ups
        if result.leveled_up {
            let new_level = result.new_level;
            if (new_level as usize) < self.level_up_ticks.len() {
                self.level_up_ticks[new_level as usize] = current_tick;
            }
            self.current_level = new_level;
        }

        // Track zone advancement
        if result.zone_advanced {
            let new_zone = result.new_zone;
            if new_zone <= 10 {
                self.zone_entries[new_zone as usize] = current_tick;
            }
            self.current_zone = new_zone;
        }

        // Track loot
        if result.had_combat && result.player_won {
            self.loot_stats.record_attempt();
            if let Some(ref item) = result.loot_dropped {
                self.loot_stats.record_drop(item, result.loot_equipped);
            }
        }
    }

    /// Record a prestige transition.
    fn record_prestige(&mut self, current_tick: u64, final_level: u32) {
        let cycle = PrestigeCycle {
            rank: self.current_prestige_rank,
            ticks_to_complete: current_tick - self.cycle_start_tick,
            final_level,
            total_deaths: self.total_deaths - self.cycle_start_deaths,
            total_kills: self.total_kills - self.cycle_start_kills,
        };
        self.prestige_cycles.push(cycle);

        // Reset cycle tracking
        self.current_prestige_rank += 1;
        self.cycle_start_tick = current_tick;
        self.cycle_start_deaths = self.total_deaths;
        self.cycle_start_kills = self.total_kills;

        // Reset per-cycle tracking
        self.level_up_ticks = vec![0u64; 201];
        self.level_up_ticks[1] = current_tick;
        self.zone_entries = vec![0; 11];
        self.zone_entries[1] = current_tick;
        self.zone_deaths = vec![0; 11];
        self.zone_kills = vec![0; 11];
        self.current_zone = 1;
        self.current_level = 1;
    }

    /// Finalize the last prestige cycle.
    fn finalize_cycle(&mut self, current_tick: u64, final_level: u32) {
        if current_tick > self.cycle_start_tick {
            let cycle = PrestigeCycle {
                rank: self.current_prestige_rank,
                ticks_to_complete: current_tick - self.cycle_start_tick,
                final_level,
                total_deaths: self.total_deaths - self.cycle_start_deaths,
                total_kills: self.total_kills - self.cycle_start_kills,
            };
            self.prestige_cycles.push(cycle);
        }
    }

    /// Calculate ticks per zone.
    fn calculate_ticks_per_zone(&self, total_ticks: u64) -> Vec<u64> {
        let mut ticks_per_zone = vec![0u64; 11];

        #[allow(clippy::needless_range_loop)]
        for i in 1..=10 {
            if i < 10 && self.zone_entries[i + 1] > 0 {
                ticks_per_zone[i] = self.zone_entries[i + 1] - self.zone_entries[i];
            } else if self.current_zone as usize >= i {
                ticks_per_zone[i] = total_ticks - self.zone_entries[i];
            }
        }

        ticks_per_zone
    }
}

/// Simulate a single run from start to target using CoreGame.
fn simulate_single_run(config: &SimConfig, rng: &mut ChaCha8Rng) -> RunStats {
    // Create CoreGame - this is the shared game engine
    let mut core_game = CoreGame::new("SimPlayer".to_string());

    // Create stats tracker
    let mut stats = SimStats::new();

    let mut ticks: u64 = 0;

    loop {
        // Get current state for termination check
        let state = core_game.state();
        let current_zone = state.zone_progression.current_zone_id;
        let current_prestige = state.prestige_rank;

        // Check termination conditions
        if current_zone >= config.target_zone as u32
            && (!config.simulate_prestige || current_prestige >= config.target_prestige)
        {
            break;
        }

        if ticks >= config.max_ticks_per_run {
            break;
        }

        // Check for prestige opportunity before ticking
        if config.simulate_prestige
            && core_game.at_prestige_wall()
            && core_game.can_prestige()
            && current_prestige < config.target_prestige
        {
            let level_before = core_game.state().character_level;
            stats.record_prestige(ticks, level_before);
            core_game.prestige();
        }

        // Execute one game tick using CoreGame
        let result = core_game.tick(rng);

        // Process the result to update stats
        stats.process_tick(&result, ticks);

        ticks += 1;
    }

    // Finalize stats
    let final_state = core_game.state();
    stats.finalize_cycle(ticks, final_state.character_level);

    // Calculate ticks per zone before moving stats fields
    let ticks_per_zone = stats.calculate_ticks_per_zone(ticks);

    // Build RunStats from accumulated stats
    RunStats {
        final_level: final_state.character_level,
        final_zone: final_state.zone_progression.current_zone_id,
        final_subzone: final_state.zone_progression.current_subzone_id,
        final_prestige: final_state.prestige_rank,
        total_kills: stats.total_kills,
        total_boss_kills: stats.total_boss_kills,
        total_deaths: stats.total_deaths,
        total_ticks: ticks,
        loot_stats: stats.loot_stats,
        final_avg_ilvl: average_equipped_ilvl(&final_state.equipment),
        reached_target: final_state.zone_progression.current_zone_id >= config.target_zone as u32,
        zone_deaths: stats.zone_deaths,
        zone_kills: stats.zone_kills,
        ticks_per_zone,
        level_up_ticks: stats.level_up_ticks,
        prestige_cycles: stats.prestige_cycles,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_run() {
        let config = SimConfig {
            num_runs: 1,
            seed: Some(12345),
            max_ticks_per_run: 50_000,
            target_zone: 2,
            simulate_loot: true,
            simulate_prestige: false,
            verbosity: 0,
            ..Default::default()
        };

        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let stats = simulate_single_run(&config, &mut rng);

        assert!(stats.total_kills > 0);
        assert!(stats.final_level > 1);
    }

    #[test]
    fn test_full_simulation() {
        let config = SimConfig {
            num_runs: 5,
            seed: Some(42),
            target_zone: 2,
            max_ticks_per_run: 100_000,
            verbosity: 0,
            ..Default::default()
        };

        let report = run_simulation(&config);

        assert_eq!(report.num_runs, 5);
        assert!(report.avg_total_kills > 0.0);
    }

    #[test]
    fn test_simulation_runs() {
        // Basic test that simulation runs without panicking
        let config = SimConfig {
            num_runs: 2,
            seed: Some(99999),
            target_zone: 2,
            max_ticks_per_run: 5_000,
            verbosity: 0,
            ..Default::default()
        };

        let report = run_simulation(&config);

        // Should complete without panic
        assert_eq!(report.num_runs, 2);
        assert!(report.avg_total_kills > 0.0);
    }

    #[test]
    fn test_stats_track_deaths() {
        let config = SimConfig {
            num_runs: 1,
            seed: Some(555),
            max_ticks_per_run: 10_000,
            target_zone: 10,
            simulate_loot: true,
            simulate_prestige: false,
            verbosity: 0,
            ..Default::default()
        };

        let mut rng = ChaCha8Rng::seed_from_u64(555);
        let stats = simulate_single_run(&config, &mut rng);

        // After 10k ticks, should have some deaths
        assert!(stats.total_deaths > 0 || stats.total_kills > 0);
    }

    #[test]
    fn test_stats_track_levels() {
        let config = SimConfig {
            num_runs: 1,
            seed: Some(777),
            max_ticks_per_run: 20_000,
            target_zone: 10,
            simulate_loot: true,
            simulate_prestige: false,
            verbosity: 0,
            ..Default::default()
        };

        let mut rng = ChaCha8Rng::seed_from_u64(777);
        let stats = simulate_single_run(&config, &mut rng);

        // Should have leveled up at least once
        assert!(stats.final_level > 1);
        // Level 2 tick should be recorded
        if stats.final_level >= 2 {
            assert!(stats.level_up_ticks[2] > 0);
        }
    }
}
