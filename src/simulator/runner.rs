//! Main simulation runner using real game mechanics.

use super::combat_sim::{simulate_combat, SimEnemy, SimPlayer};
use super::config::SimConfig;
use super::loot_sim::{average_equipped_ilvl, roll_boss_drop_real, roll_mob_drop_real, LootStats};
use super::progression_sim::{RunStats, SimProgression};
use super::report::SimReport;
use crate::core::game_state::GameState;
use crate::items::scoring::auto_equip_if_better;
use chrono::Utc;
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
            println!(
                "Run {}/{} complete - Zone {}, Level {}, Deaths {}",
                run_idx + 1,
                config.num_runs,
                all_runs.last().unwrap().final_zone,
                all_runs.last().unwrap().final_level,
                all_runs.last().unwrap().total_deaths
            );
        }
    }

    SimReport::from_runs(all_runs, config.target_zone, config.max_ticks_per_run)
}

/// Simulate a single run from start to target using real game mechanics.
fn simulate_single_run(config: &SimConfig, rng: &mut impl Rng) -> RunStats {
    let mut progression = SimProgression::new();
    let mut player = SimPlayer::new();
    let mut loot_stats = LootStats::default();

    // Create a temporary GameState for item scoring (needed by scoring system)
    let mut temp_game_state = GameState::new("SimPlayer".to_string(), Utc::now().timestamp());

    let mut ticks: u64 = 0;

    loop {
        // Check termination conditions
        if progression.current_zone >= config.target_zone as u32 {
            if !config.simulate_prestige || progression.prestige_rank >= config.target_prestige {
                break;
            }
            // Prestige and continue
            progression.prestige();
            // Reset player level but keep equipment
            player = SimPlayer::at_level(1);
            // Re-apply equipment to new player
            for item in temp_game_state.equipment.iter_equipped() {
                player.equip(item.clone());
            }
        }

        if ticks >= config.max_ticks_per_run {
            break;
        }

        // Sync player level with progression
        if player.level != progression.player_level {
            player = SimPlayer::at_level(progression.player_level);
            // Re-apply equipment
            player.equipment = temp_game_state.equipment.clone();
            player.recalculate_stats();
        }

        // Determine if this fight is a boss
        let is_boss = progression.should_spawn_boss();

        // Generate enemy using real game logic
        let mut enemy = if is_boss {
            SimEnemy::boss_for_zone(
                progression.current_zone,
                progression.current_subzone,
                &player,
            )
        } else {
            SimEnemy::for_zone(
                progression.current_zone,
                progression.current_subzone,
                &player,
            )
        };

        // Simulate combat
        let combat_result = simulate_combat(&mut player, &mut enemy, rng);
        ticks += combat_result.ticks_elapsed as u64;

        if combat_result.player_won {
            // Add XP
            progression.add_xp(combat_result.xp_gained as u64);
            progression.record_kill(combat_result.was_boss, ticks);

            // Level up player if needed
            while player.level < progression.player_level {
                player = SimPlayer::at_level(progression.player_level);
                player.equipment = temp_game_state.equipment.clone();
                player.recalculate_stats();
            }

            // Handle loot
            if config.simulate_loot {
                loot_stats.record_attempt();

                let item = if combat_result.was_boss {
                    Some(roll_boss_drop_real(
                        progression.current_zone as usize,
                        progression.current_zone >= 10,
                        rng,
                    ))
                } else {
                    roll_mob_drop_real(
                        progression.current_zone as usize,
                        progression.prestige_rank,
                        0.0, // No haven bonuses in sim (could add later)
                        0.0,
                        rng,
                    )
                };

                if let Some(item) = item {
                    // Check if upgrade using real scoring
                    let was_upgrade = auto_equip_if_better(item.clone(), &mut temp_game_state);

                    if was_upgrade {
                        // Apply to sim player too
                        player.equip(item.clone());
                    }

                    loot_stats.record_drop(&item, was_upgrade);
                }
            }

            // Heal after combat (simplified - real game has regen)
            player.heal_full();
        } else {
            // Player died
            progression.record_death();
            // Respawn at full HP
            player.heal_full();
        }
    }

    // Calculate ticks per zone
    let mut ticks_per_zone = vec![0u64; 11];
    for i in 1..=10 {
        if i < 10 && progression.zone_entries[i + 1] > 0 {
            ticks_per_zone[i] = progression.zone_entries[i + 1] - progression.zone_entries[i];
        } else if progression.current_zone as usize >= i {
            ticks_per_zone[i] = ticks - progression.zone_entries[i];
        }
    }

    RunStats {
        final_level: progression.player_level,
        final_zone: progression.current_zone,
        final_subzone: progression.current_subzone,
        final_prestige: progression.prestige_rank,
        total_kills: progression.total_kills,
        total_boss_kills: progression.total_boss_kills,
        total_deaths: progression.total_deaths,
        total_ticks: ticks,
        loot_stats,
        final_avg_ilvl: average_equipped_ilvl(&temp_game_state.equipment),
        reached_target: progression.current_zone >= config.target_zone as u32,
        zone_deaths: progression.zone_deaths,
        zone_kills: progression.zone_kills,
        ticks_per_zone,
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
}
