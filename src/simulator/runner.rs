//! Main simulation runner.

use super::combat_sim::{simulate_combat, SimMonster, SimPlayer};
use super::config::SimConfig;
use super::loot_sim::{roll_boss_drop, roll_mob_drop, LootStats, SimEquipment};
use super::progression_sim::{RunStats, SimProgression};
use super::report::SimReport;
use rand::Rng;
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
            println!("Run {}/{} complete", run_idx + 1, config.num_runs);
        }
    }

    SimReport::from_runs(all_runs, config.target_zone, config.max_ticks_per_run)
}

/// Simulate a single run from start to target.
fn simulate_single_run(config: &SimConfig, rng: &mut impl Rng) -> RunStats {
    let mut progression = SimProgression::new();
    let mut equipment = SimEquipment::default();
    let mut loot_stats = LootStats::default();

    let mut ticks: u64 = 0;

    loop {
        // Check termination conditions
        if progression.current_zone >= config.target_zone {
            if !config.simulate_prestige || progression.prestige_rank >= config.target_prestige {
                break;
            }
            // Prestige and continue
            progression.prestige();
            // Keep equipment through prestige (simplified)
        }

        if ticks >= config.max_ticks_per_run {
            break;
        }

        // Create player with current stats
        let mut player = SimPlayer::new(progression.player_level);
        if config.simulate_loot {
            player.apply_gear_bonus(
                equipment.total_damage_mult(),
                equipment.total_hp_mult(),
                equipment.total_crit_bonus(),
            );
        }

        // Determine monster type
        let monster_level = progression.current_monster_level();
        let is_boss = progression.should_spawn_boss();
        let mut monster = if is_boss {
            SimMonster::boss(monster_level)
        } else {
            SimMonster::normal(monster_level)
        };

        // Simulate combat
        let combat_result = simulate_combat(&mut player, &mut monster, rng);
        ticks += combat_result.ticks_elapsed as u64;

        if combat_result.player_won {
            // Add XP
            progression.add_xp(combat_result.xp_gained as u64);
            progression.record_kill(combat_result.was_boss);

            // Handle loot
            if config.simulate_loot {
                let item = if combat_result.was_boss {
                    Some(roll_boss_drop(
                        progression.current_zone,
                        progression.current_zone == 10,
                        rng,
                    ))
                } else {
                    roll_mob_drop(progression.current_zone, rng)
                };

                if let Some(item) = item {
                    let was_upgrade = equipment.equip_if_upgrade(item.clone());
                    loot_stats.record_drop(&item, was_upgrade);
                }
            }
        } else {
            // Player died
            progression.record_death();
            // Simplified: instant respawn, continue from same spot
        }
    }

    RunStats {
        final_level: progression.player_level,
        final_zone: progression.current_zone,
        final_prestige: progression.prestige_rank,
        total_kills: progression.total_kills,
        total_boss_kills: progression.total_boss_kills,
        total_deaths: progression.total_deaths,
        total_ticks: ticks,
        loot_stats,
        final_avg_ilvl: equipment.average_ilvl(),
        reached_target: progression.current_zone >= config.target_zone,
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
            max_ticks_per_run: 100_000,
            target_zone: 3,
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
            num_runs: 10,
            seed: Some(42),
            target_zone: 2,
            max_ticks_per_run: 50_000,
            verbosity: 0,
            ..Default::default()
        };

        let report = run_simulation(&config);

        assert_eq!(report.num_runs, 10);
        assert!(report.avg_total_kills > 0.0);
    }

    #[test]
    fn test_deterministic_with_seed() {
        let config = SimConfig {
            num_runs: 5,
            seed: Some(99999),
            target_zone: 2,
            max_ticks_per_run: 50_000,
            verbosity: 0,
            ..Default::default()
        };

        let report1 = run_simulation(&config);
        let report2 = run_simulation(&config);

        // Same seed should give same results
        assert_eq!(report1.avg_total_kills, report2.avg_total_kills);
        assert_eq!(report1.avg_total_deaths, report2.avg_total_deaths);
    }
}
