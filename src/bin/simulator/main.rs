//! Quest Headless Game Balance Simulator
//!
//! Runs the game tick loop without any UI, collecting metrics for game balance
//! analysis. Uses the exact same `game_tick()` function as the real game.
//!
//! Usage:
//!   cargo run --bin simulator -- [OPTIONS]
//!
//! Options:
//!   --ticks N       Ticks to simulate (default: 36000 = 1 hour game time)
//!   --seed N        RNG seed (default: 42)
//!   --prestige N    Starting prestige rank (default: 0)
//!   --runs N        Number of runs with incrementing seeds (default: 1)
//!   --verbose       Per-tick event logging
//!   --csv FILE      Write time-series CSV
//!   --quiet         Only final summary line

mod report;
mod stats;

use quest::achievements::Achievements;
use quest::character::derived_stats::DerivedStats;
use quest::core::game_state::GameState;
#[allow(deprecated)]
use quest::core::tick::game_tick;
use quest::enhancement::EnhancementProgress;
use quest::haven::{try_build_room, Haven, HavenRoomId};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::io::Write;

use report::{
    print_final_equipment, print_multi_run_summary, print_profile, print_summary,
    print_tick_events, ticks_to_time,
};
use stats::{SimStats, TickProfile};

// ── Haven Strategy ──────────────────────────────────────────────────

pub enum HavenStrategy {
    Combat,
    Qol,
    Balanced,
    Full,
}

impl HavenStrategy {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "combat" => Some(Self::Combat),
            "qol" => Some(Self::Qol),
            "balanced" => Some(Self::Balanced),
            "full" => Some(Self::Full),
            _ => None,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Combat => "combat",
            Self::Qol => "qol",
            Self::Balanced => "balanced",
            Self::Full => "full",
        }
    }

    fn room_priority(&self) -> &'static [HavenRoomId] {
        match self {
            Self::Combat => &[
                HavenRoomId::Hearthstone,
                HavenRoomId::Armory,
                HavenRoomId::TrainingYard,
                HavenRoomId::TrophyHall,
                HavenRoomId::Watchtower,
                HavenRoomId::AlchemyLab,
                HavenRoomId::WarRoom,
            ],
            Self::Qol => &[
                HavenRoomId::Hearthstone,
                HavenRoomId::Bedroom,
                HavenRoomId::Garden,
                HavenRoomId::Library,
                HavenRoomId::FishingDock,
                HavenRoomId::Workshop,
                HavenRoomId::Vault,
            ],
            Self::Balanced => &[
                HavenRoomId::Hearthstone,
                HavenRoomId::Armory,
                HavenRoomId::Bedroom,
                HavenRoomId::TrainingYard,
                HavenRoomId::Garden,
                HavenRoomId::TrophyHall,
                HavenRoomId::Library,
                HavenRoomId::Watchtower,
                HavenRoomId::FishingDock,
                HavenRoomId::AlchemyLab,
                HavenRoomId::Workshop,
                HavenRoomId::WarRoom,
                HavenRoomId::Vault,
            ],
            Self::Full => &[
                HavenRoomId::Hearthstone,
                HavenRoomId::Armory,
                HavenRoomId::Bedroom,
                HavenRoomId::TrainingYard,
                HavenRoomId::Garden,
                HavenRoomId::TrophyHall,
                HavenRoomId::Library,
                HavenRoomId::Watchtower,
                HavenRoomId::FishingDock,
                HavenRoomId::AlchemyLab,
                HavenRoomId::Workshop,
                HavenRoomId::WarRoom,
                HavenRoomId::Vault,
                HavenRoomId::StormForge,
            ],
        }
    }
}

// ── CLI Configuration ────────────────────────────────────────────────

pub struct SimConfig {
    pub ticks: u64,
    pub seed: u64,
    pub prestige: u32,
    pub runs: u32,
    pub verbose: bool,
    pub csv_path: Option<String>,
    pub quiet: bool,
    pub stormbreaker: bool,
    pub haven_strategy: Option<HavenStrategy>,
    pub profile: bool,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            ticks: 36_000,
            seed: 42,
            prestige: 0,
            runs: 1,
            verbose: false,
            csv_path: None,
            quiet: false,
            stormbreaker: false,
            haven_strategy: None,
            profile: false,
        }
    }
}

fn parse_args() -> SimConfig {
    let args: Vec<String> = std::env::args().collect();
    let mut config = SimConfig::default();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--ticks" => {
                i += 1;
                config.ticks = args[i].parse().expect("--ticks requires a number");
            }
            "--seed" => {
                i += 1;
                config.seed = args[i].parse().expect("--seed requires a number");
            }
            "--prestige" => {
                i += 1;
                config.prestige = args[i].parse().expect("--prestige requires a number");
            }
            "--runs" => {
                i += 1;
                config.runs = args[i].parse().expect("--runs requires a number");
            }
            "--verbose" => config.verbose = true,
            "--csv" => {
                i += 1;
                config.csv_path = Some(args[i].clone());
            }
            "--quiet" => config.quiet = true,
            "--stormbreaker" => config.stormbreaker = true,
            "--profile" => config.profile = true,
            "--haven" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--haven requires a strategy: combat, qol, balanced, full");
                    std::process::exit(1);
                }
                config.haven_strategy =
                    Some(HavenStrategy::from_str(&args[i]).unwrap_or_else(|| {
                        eprintln!(
                            "Unknown haven strategy: {}. Options: combat, qol, balanced, full",
                            args[i]
                        );
                        std::process::exit(1);
                    }));
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => {
                eprintln!("Unknown argument: {other}");
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }
    config
}

fn print_usage() {
    eprintln!(
        "Quest Headless Game Simulator\n\
         \n\
         Usage: quest-simulator [OPTIONS]\n\
         \n\
         Options:\n\
         \x20 --ticks N       Ticks to simulate (default: 36000 = 1 hour)\n\
         \x20 --seed N        RNG seed (default: 42)\n\
         \x20 --prestige N    Starting prestige rank (default: 0)\n\
         \x20 --runs N        Number of runs with incrementing seeds (default: 1)\n\
         \x20 --verbose       Per-tick event logging\n\
         \x20 --csv FILE      Write time-series CSV\n\
         \x20 --quiet         Only final summary line\n\
         \x20 --stormbreaker  Unlock Stormbreaker achievement (access Zone 10 boss)\n\
         \x20 --haven STR     Haven auto-build strategy (combat, qol, balanced, full)\n\
         \x20 --profile       Print per-tick timing profile\n\
         \x20 --help, -h      Show this help"
    );
}

/// Auto-build one Haven room using the given strategy.
/// Returns the (room, new_tier, cost) if a room was built this tick.
fn auto_build_haven(
    haven: &mut Haven,
    prestige_rank: &mut u32,
    priority: &[HavenRoomId],
) -> Option<(HavenRoomId, u8, u32)> {
    for &room in priority {
        if haven.room_tier(room) >= room.max_tier() {
            continue;
        }
        if let Some((new_tier, cost)) = try_build_room(room, haven, prestige_rank) {
            return Some((room, new_tier, cost));
        }
    }
    None
}

// ── Core Simulation Loop ─────────────────────────────────────────────

#[allow(deprecated)]
fn run_simulation(config: &SimConfig, seed: u64) -> (SimStats, GameState, Option<TickProfile>) {
    let mut state = GameState::new("Simulator".to_string(), 0);
    state.prestige_rank = config.prestige;

    // Recalculate derived stats after setting prestige to get correct HP/damage
    let derived =
        DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7]);
    state.combat_state.player_max_hp = derived.max_hp;
    state.combat_state.player_current_hp = derived.max_hp;

    let mut haven = Haven::default();
    let haven_priority = config.haven_strategy.as_ref().map(|s| s.room_priority());
    if haven_priority.is_some() {
        haven.discovered = true;
    }
    let mut enhancement = EnhancementProgress::new();
    let mut deep_state = quest::deep::DeepState::new();
    let mut achievements = Achievements::default();
    // Force-unlock Stormbreaker achievement if requested
    if config.stormbreaker {
        use quest::achievements::AchievementId;
        achievements.unlock(
            AchievementId::TheStormbreaker,
            Some("Simulator".to_string()),
        );
    }

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut tick_counter: u32 = 0;
    let mut stats = SimStats::default();
    let mut tick_profile = if config.profile {
        Some(TickProfile::default())
    } else {
        None
    };

    // Track zone changes
    let mut prev_zone = (
        state.zone_progression.current_zone_id,
        state.zone_progression.current_subzone_id,
    );
    stats.record_zone_entry(0, prev_zone.0, prev_zone.1);

    // CSV writer (if requested)
    let mut csv_writer = config.csv_path.as_ref().map(|path| {
        let file = std::fs::File::create(path).expect("Failed to create CSV file");
        let mut w = std::io::BufWriter::new(file);
        writeln!(
            w,
            "tick,game_time_s,level,xp,zone_id,subzone_id,prestige_rank,total_kills,total_deaths,fishing_rank,items_found,haven_rooms_built,haven_prestige_spent"
        )
        .expect("Failed to write CSV header");
        w
    });

    for tick in 0..config.ticks {
        let result = if let Some(ref mut profile) = tick_profile {
            let start = std::time::Instant::now();
            let r = game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut enhancement,
                &mut deep_state,
                &mut achievements,
                false,
                &mut rng,
            );
            profile.record(start.elapsed().as_nanos());
            r
        } else {
            game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut enhancement,
                &mut deep_state,
                &mut achievements,
                false,
                &mut rng,
            )
        };

        // Detect zone changes
        let curr_zone = (
            state.zone_progression.current_zone_id,
            state.zone_progression.current_subzone_id,
        );
        if curr_zone != prev_zone {
            stats.record_zone_entry(tick, curr_zone.0, curr_zone.1);
            prev_zone = curr_zone;
        }

        // Auto-build Haven rooms if strategy is active
        if let Some(priority) = haven_priority {
            if let Some((room, new_tier, cost)) =
                auto_build_haven(&mut haven, &mut state.prestige_rank, priority)
            {
                state.invalidate_bonuses();
                stats.haven_rooms_built += 1;
                stats.haven_prestige_spent += cost;
                if config.verbose {
                    println!(
                        "[t={tick:>6}] Haven: {} upgraded to T{new_tier} (cost {cost} PR)",
                        room.name()
                    );
                }
            }
        }

        stats.process_tick(tick, &result, &state, curr_zone);

        if config.verbose {
            print_tick_events(tick, &result);
        }

        // CSV snapshot every 100 ticks
        if let Some(ref mut w) = csv_writer {
            if tick % 100 == 0 {
                let total_items: u64 = stats.items_by_rarity.iter().sum();
                writeln!(
                    w,
                    "{},{:.1},{},{},{},{},{},{},{},{},{},{},{}",
                    tick,
                    tick as f64 / 10.0,
                    state.character_level,
                    state.character_xp,
                    state.zone_progression.current_zone_id,
                    state.zone_progression.current_subzone_id,
                    state.prestige_rank,
                    stats.total_kills,
                    stats.total_deaths,
                    state.fishing.rank,
                    total_items,
                    stats.haven_rooms_built,
                    stats.haven_prestige_spent,
                )
                .expect("Failed to write CSV row");
            }
        }
    }

    // Flush CSV
    if let Some(ref mut w) = csv_writer {
        w.flush().expect("Failed to flush CSV");
    }

    if config.haven_strategy.is_some() {
        for room in HavenRoomId::ALL {
            let tier = haven.room_tier(room);
            if tier > 0 {
                stats
                    .haven_final_tiers
                    .push((room.name().to_string(), tier));
            }
        }
    }

    stats.finalize(&state);
    (stats, state, tick_profile)
}

// ── Main ─────────────────────────────────────────────────────────────

fn main() {
    let config = parse_args();

    if !config.quiet {
        let haven_str = config
            .haven_strategy
            .as_ref()
            .map(|s| format!(", haven={}", s.name()))
            .unwrap_or_default();
        eprintln!(
            "Quest Simulator: {} ticks ({}) x {} run(s), seed={}, prestige=P{}, stormbreaker={}{}",
            config.ticks,
            ticks_to_time(config.ticks),
            config.runs,
            config.seed,
            config.prestige,
            config.stormbreaker,
            haven_str,
        );
    }

    let mut all_stats = Vec::with_capacity(config.runs as usize);

    for run in 0..config.runs {
        let seed = config.seed + run as u64;

        if !config.quiet && config.runs > 1 {
            eprintln!("--- Run {}/{} (seed={seed}) ---", run + 1, config.runs);
        }

        let (stats, final_state, tick_profile) = run_simulation(&config, seed);

        if config.runs == 1 {
            // Single run: print full final state
            print_summary(&stats, seed, &config);
            print_final_equipment(&final_state);
            if let Some(ref profile) = tick_profile {
                print_profile(profile, &config);
            }
        } else if !config.quiet {
            // Multi-run: print one-liner per run
            let total_items: u64 = stats.items_by_rarity.iter().sum();
            println!(
                "  Run {}: L{} zone={}-{} kills={} deaths={} items={} achievements={}",
                run + 1,
                stats.final_level,
                stats.final_zone.0,
                stats.final_zone.1,
                stats.total_kills,
                stats.total_deaths,
                total_items,
                stats.achievements_unlocked,
            );
        }

        all_stats.push(stats);
    }

    if config.runs > 1 {
        println!();
        print_multi_run_summary(&all_stats);
    }
}
