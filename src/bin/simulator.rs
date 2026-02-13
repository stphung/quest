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

use quest::achievements::Achievements;
use quest::character::attributes::AttributeType;
use quest::character::derived_stats::DerivedStats;
use quest::core::game_state::GameState;
use quest::core::tick::{game_tick, TickEvent, TickResult};
use quest::haven::Haven;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use std::io::Write;

// ── CLI Configuration ────────────────────────────────────────────────

struct SimConfig {
    ticks: u64,
    seed: u64,
    prestige: u32,
    runs: u32,
    verbose: bool,
    csv_path: Option<String>,
    quiet: bool,
    stormbreaker: bool,
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
         \x20 --help, -h      Show this help"
    );
}

// ── Simulation Statistics ────────────────────────────────────────────

#[derive(Debug, Clone)]
struct SimStats {
    total_ticks: u64,
    total_kills: u64,
    total_deaths: u64,
    total_boss_kills: u64,
    total_crits: u64,
    total_xp_gained: u64,
    level_at_tick: HashMap<u32, u64>,
    zone_entry_tick: HashMap<(u32, u32), u64>,
    zone_boss_defeated_tick: HashMap<(u32, u32), u64>,
    deaths_per_zone: HashMap<(u32, u32), u64>,
    items_by_rarity: [u64; 5],
    items_equipped: u64,
    boss_items_dropped: u64,
    fish_caught: u64,
    fishing_rank_ups: u64,
    dungeons_completed: u64,
    dungeons_failed: u64,
    dungeons_discovered: u64,
    achievements_unlocked: u64,
    haven_discovered: bool,
    // Final state snapshot
    final_level: u32,
    final_xp: u64,
    final_prestige: u32,
    final_zone: (u32, u32),
    final_fishing_rank: u32,
    final_attributes: [u32; 6],
}

impl Default for SimStats {
    fn default() -> Self {
        Self {
            total_ticks: 0,
            total_kills: 0,
            total_deaths: 0,
            total_boss_kills: 0,
            total_crits: 0,
            total_xp_gained: 0,
            level_at_tick: HashMap::new(),
            zone_entry_tick: HashMap::new(),
            zone_boss_defeated_tick: HashMap::new(),
            deaths_per_zone: HashMap::new(),
            items_by_rarity: [0; 5],
            items_equipped: 0,
            boss_items_dropped: 0,
            fish_caught: 0,
            fishing_rank_ups: 0,
            dungeons_completed: 0,
            dungeons_failed: 0,
            dungeons_discovered: 0,
            achievements_unlocked: 0,
            haven_discovered: false,
            final_level: 1,
            final_xp: 0,
            final_prestige: 0,
            final_zone: (1, 1),
            final_fishing_rank: 1,
            final_attributes: [10; 6],
        }
    }
}

impl SimStats {
    fn record_zone_entry(&mut self, tick: u64, zone_id: u32, subzone_id: u32) {
        self.zone_entry_tick
            .entry((zone_id, subzone_id))
            .or_insert(tick);
    }

    fn process_tick(
        &mut self,
        tick: u64,
        result: &TickResult,
        _state: &GameState,
        current_zone: (u32, u32),
    ) {
        self.total_ticks = tick + 1;

        for event in &result.events {
            match event {
                TickEvent::EnemyDefeated { xp_gained, .. } => {
                    self.total_kills += 1;
                    self.total_xp_gained += xp_gained;
                }
                TickEvent::PlayerDied { .. } => {
                    self.total_deaths += 1;
                    *self.deaths_per_zone.entry(current_zone).or_insert(0) += 1;
                }
                TickEvent::PlayerDiedInDungeon { .. } => {
                    self.total_deaths += 1;
                }
                TickEvent::SubzoneBossDefeated { xp_gained, .. } => {
                    self.total_boss_kills += 1;
                    self.total_xp_gained += xp_gained;
                    self.zone_boss_defeated_tick
                        .entry(current_zone)
                        .or_insert(tick);
                }
                TickEvent::PlayerAttack { was_crit, .. } => {
                    if *was_crit {
                        self.total_crits += 1;
                    }
                }
                TickEvent::ItemDropped {
                    rarity,
                    equipped,
                    from_boss,
                    ..
                } => {
                    let idx = *rarity as usize;
                    if idx < 5 {
                        self.items_by_rarity[idx] += 1;
                    }
                    if *equipped {
                        self.items_equipped += 1;
                    }
                    if *from_boss {
                        self.boss_items_dropped += 1;
                    }
                }
                TickEvent::LeveledUp { new_level } => {
                    self.level_at_tick.entry(*new_level).or_insert(tick);
                }
                TickEvent::DungeonCompleted { .. } | TickEvent::DungeonBossDefeated { .. } => {
                    self.dungeons_completed += 1;
                }
                TickEvent::DungeonFailed { .. } => {
                    self.dungeons_failed += 1;
                }
                TickEvent::DungeonDiscovered { .. } => {
                    self.dungeons_discovered += 1;
                }
                TickEvent::FishCaught { .. } => {
                    self.fish_caught += 1;
                }
                TickEvent::FishingRankUp { .. } => {
                    self.fishing_rank_ups += 1;
                }
                TickEvent::AchievementUnlocked { .. } => {
                    self.achievements_unlocked += 1;
                }
                TickEvent::HavenDiscovered => {
                    self.haven_discovered = true;
                }
                _ => {}
            }
        }

        // Track XP from dungeon boss completions separately
        for event in &result.events {
            if let TickEvent::DungeonBossDefeated { total_xp, .. } = event {
                self.total_xp_gained += total_xp;
            }
        }
    }

    fn finalize(&mut self, state: &GameState) {
        self.final_level = state.character_level;
        self.final_xp = state.character_xp;
        self.final_prestige = state.prestige_rank;
        self.final_zone = (
            state.zone_progression.current_zone_id,
            state.zone_progression.current_subzone_id,
        );
        self.final_fishing_rank = state.fishing.rank;

        for attr in AttributeType::all() {
            self.final_attributes[attr.index()] = state.attributes.get(attr);
        }
    }
}

// ── Core Simulation Loop ─────────────────────────────────────────────

fn run_simulation(config: &SimConfig, seed: u64) -> (SimStats, GameState) {
    let mut state = GameState::new("Simulator".to_string(), 0);
    state.prestige_rank = config.prestige;

    // Recalculate derived stats after setting prestige to get correct HP/damage
    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
    state.combat_state.player_max_hp = derived.max_hp;
    state.combat_state.player_current_hp = derived.max_hp;

    let mut haven = Haven::default();
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
            "tick,game_time_s,level,xp,zone_id,subzone_id,prestige_rank,total_kills,total_deaths,fishing_rank,items_found"
        )
        .expect("Failed to write CSV header");
        w
    });

    for tick in 0..config.ticks {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut achievements,
            false,
            &mut rng,
        );

        // Detect zone changes
        let curr_zone = (
            state.zone_progression.current_zone_id,
            state.zone_progression.current_subzone_id,
        );
        if curr_zone != prev_zone {
            stats.record_zone_entry(tick, curr_zone.0, curr_zone.1);
            prev_zone = curr_zone;
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
                    "{},{:.1},{},{},{},{},{},{},{},{},{}",
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
                )
                .expect("Failed to write CSV row");
            }
        }
    }

    // Flush CSV
    if let Some(ref mut w) = csv_writer {
        w.flush().expect("Failed to flush CSV");
    }

    stats.finalize(&state);
    (stats, state)
}

// ── Verbose Output ───────────────────────────────────────────────────

fn print_tick_events(tick: u64, result: &TickResult) {
    for event in &result.events {
        let label = match event {
            TickEvent::PlayerAttack {
                damage, was_crit, ..
            } => {
                if *was_crit {
                    format!("CRIT {damage} damage")
                } else {
                    format!("Hit {damage} damage")
                }
            }
            TickEvent::EnemyAttack {
                damage, enemy_name, ..
            } => {
                format!("{enemy_name} hit for {damage}")
            }
            TickEvent::EnemyDefeated {
                xp_gained,
                enemy_name,
                ..
            } => {
                format!("Killed {enemy_name} (+{xp_gained} XP)")
            }
            TickEvent::PlayerDied { .. } => "DIED".to_string(),
            TickEvent::PlayerDiedInDungeon { .. } => "DIED (dungeon)".to_string(),
            TickEvent::SubzoneBossDefeated { xp_gained, .. } => {
                format!("Boss defeated (+{xp_gained} XP)")
            }
            TickEvent::ItemDropped {
                item_name,
                rarity,
                equipped,
                ..
            } => {
                let eq = if *equipped { " [EQUIPPED]" } else { "" };
                format!("Item: {} ({:?}){}", item_name, rarity, eq)
            }
            TickEvent::LeveledUp { new_level } => format!("Level up! -> {new_level}"),
            TickEvent::DungeonDiscovered { .. } => "Dungeon discovered!".to_string(),
            TickEvent::DungeonCompleted { xp_earned, .. } => {
                format!("Dungeon completed (+{xp_earned} XP)")
            }
            TickEvent::DungeonFailed { .. } => "Dungeon failed".to_string(),
            TickEvent::FishCaught {
                fish_name, rarity, ..
            } => {
                format!("Caught {fish_name} ({rarity:?})")
            }
            TickEvent::FishingRankUp { .. } => "Fishing rank up!".to_string(),
            TickEvent::AchievementUnlocked { name, .. } => format!("Achievement: {name}"),
            TickEvent::HavenDiscovered => "Haven discovered!".to_string(),
            _ => return,
        };
        println!("[t={tick:>6}] {label}");
    }
}

// ── Report Output ────────────────────────────────────────────────────

fn ticks_to_time(ticks: u64) -> String {
    let total_secs = ticks / 10;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    if hours > 0 {
        format!("{hours}h {minutes:02}m {seconds:02}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds:02}s")
    } else {
        format!("{seconds}s")
    }
}

fn print_summary(stats: &SimStats, seed: u64, config: &SimConfig) {
    if config.quiet {
        let total_items: u64 = stats.items_by_rarity.iter().sum();
        println!(
            "seed={seed} ticks={} level={} zone={}-{} kills={} deaths={} items={} achievements={}",
            stats.total_ticks,
            stats.final_level,
            stats.final_zone.0,
            stats.final_zone.1,
            stats.total_kills,
            stats.total_deaths,
            total_items,
            stats.achievements_unlocked,
        );
        return;
    }

    let game_time = ticks_to_time(stats.total_ticks);
    println!("============================================================");
    println!("  Quest Simulation Report  (seed={seed})");
    println!("============================================================");
    println!();

    // Duration
    println!("Duration: {} ticks ({game_time})", stats.total_ticks);
    println!("Starting prestige: P{}", config.prestige);
    println!();

    // Final state
    println!("--- Final State ---");
    println!(
        "Level: {}  |  XP: {}  |  Prestige: P{}",
        stats.final_level, stats.final_xp, stats.final_prestige
    );
    println!(
        "Zone: {}-{}  |  Fishing Rank: {}",
        stats.final_zone.0, stats.final_zone.1, stats.final_fishing_rank
    );

    let attr_names = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];
    let attr_str: Vec<String> = attr_names
        .iter()
        .zip(stats.final_attributes.iter())
        .map(|(name, val)| format!("{name}:{val}"))
        .collect();
    println!("Attributes: {}", attr_str.join("  "));
    println!();

    // Combat stats
    println!("--- Combat ---");
    println!(
        "Kills: {}  |  Deaths: {}  |  K/D: {:.1}",
        stats.total_kills,
        stats.total_deaths,
        if stats.total_deaths > 0 {
            stats.total_kills as f64 / stats.total_deaths as f64
        } else {
            stats.total_kills as f64
        }
    );
    println!(
        "Boss kills: {}  |  Crits: {}",
        stats.total_boss_kills, stats.total_crits,
    );
    println!(
        "Total XP: {}  |  Avg XP/kill: {:.0}",
        stats.total_xp_gained,
        if stats.total_kills > 0 {
            stats.total_xp_gained as f64 / stats.total_kills as f64
        } else {
            0.0
        }
    );
    println!();

    // Items
    println!("--- Items ---");
    let rarity_names = ["Common", "Magic", "Rare", "Epic", "Legendary"];
    let total_items: u64 = stats.items_by_rarity.iter().sum();
    println!(
        "Total drops: {total_items}  |  Equipped: {}",
        stats.items_equipped
    );
    for (i, name) in rarity_names.iter().enumerate() {
        if stats.items_by_rarity[i] > 0 {
            println!("  {name}: {}", stats.items_by_rarity[i]);
        }
    }
    if stats.boss_items_dropped > 0 {
        println!("  From bosses: {}", stats.boss_items_dropped);
    }
    println!();

    // Dungeons
    if stats.dungeons_discovered > 0 || stats.dungeons_completed > 0 || stats.dungeons_failed > 0 {
        println!("--- Dungeons ---");
        println!(
            "Discovered: {}  |  Completed: {}  |  Failed: {}",
            stats.dungeons_discovered, stats.dungeons_completed, stats.dungeons_failed
        );
        println!();
    }

    // Fishing
    if stats.fish_caught > 0 || stats.fishing_rank_ups > 0 {
        println!("--- Fishing ---");
        println!(
            "Fish caught: {}  |  Rank ups: {}  |  Final rank: {}",
            stats.fish_caught, stats.fishing_rank_ups, stats.final_fishing_rank
        );
        println!();
    }

    // Achievements
    if stats.achievements_unlocked > 0 || stats.haven_discovered {
        println!("--- Discoveries ---");
        println!("Achievements: {}", stats.achievements_unlocked);
        if stats.haven_discovered {
            println!("Haven: discovered");
        }
        println!();
    }

    // Level milestones
    let milestones = [5, 10, 15, 20, 25, 50, 75, 100];
    let reached: Vec<String> = milestones
        .iter()
        .filter_map(|&lvl| {
            stats
                .level_at_tick
                .get(&lvl)
                .map(|t| format!("L{lvl} @ {}", ticks_to_time(*t)))
        })
        .collect();
    if !reached.is_empty() {
        println!("--- Level Milestones ---");
        for m in &reached {
            println!("  {m}");
        }
        println!();
    }

    // Zone progression
    let mut zones: Vec<(&(u32, u32), &u64)> = stats.zone_entry_tick.iter().collect();
    zones.sort_by_key(|&(k, v)| (*v, k.0, k.1));
    if zones.len() > 1 {
        println!("--- Zone Progression ---");
        for ((z, s), t) in &zones {
            println!("  Zone {z}-{s} entered @ {}", ticks_to_time(**t));
        }
        println!();
    }

    // Deaths by zone
    if !stats.deaths_per_zone.is_empty() {
        let mut death_zones: Vec<_> = stats.deaths_per_zone.iter().collect();
        death_zones.sort_by_key(|&(k, _)| (k.0, k.1));
        println!("--- Deaths by Zone ---");
        for ((z, s), count) in &death_zones {
            println!("  Zone {z}-{s}: {count} deaths");
        }
        println!();
    }
}

fn print_multi_run_summary(all_stats: &[SimStats]) {
    let n = all_stats.len() as f64;
    println!("============================================================");
    println!("  Aggregate Results ({} runs)", all_stats.len());
    println!("============================================================");
    println!();

    // Helper closures
    let avg = |vals: &[u64]| -> f64 { vals.iter().sum::<u64>() as f64 / n };
    let min_max = |vals: &[u64]| -> (u64, u64) {
        (
            *vals.iter().min().unwrap_or(&0),
            *vals.iter().max().unwrap_or(&0),
        )
    };

    let levels: Vec<u64> = all_stats.iter().map(|s| s.final_level as u64).collect();
    let kills: Vec<u64> = all_stats.iter().map(|s| s.total_kills).collect();
    let deaths: Vec<u64> = all_stats.iter().map(|s| s.total_deaths).collect();
    let boss_kills: Vec<u64> = all_stats.iter().map(|s| s.total_boss_kills).collect();
    let items: Vec<u64> = all_stats
        .iter()
        .map(|s| s.items_by_rarity.iter().sum())
        .collect();
    let xp: Vec<u64> = all_stats.iter().map(|s| s.total_xp_gained).collect();
    let achievements: Vec<u64> = all_stats.iter().map(|s| s.achievements_unlocked).collect();

    let (lmin, lmax) = min_max(&levels);
    let (kmin, kmax) = min_max(&kills);
    let (dmin, dmax) = min_max(&deaths);
    let (bmin, bmax) = min_max(&boss_kills);
    let (imin, imax) = min_max(&items);
    let (xmin, xmax) = min_max(&xp);
    let (amin, amax) = min_max(&achievements);

    println!("{:<20} {:>10} {:>10} {:>10}", "Metric", "Min", "Avg", "Max");
    println!("{}", "-".repeat(52));
    println!(
        "{:<20} {:>10} {:>10.1} {:>10}",
        "Final Level",
        lmin,
        avg(&levels),
        lmax
    );
    println!(
        "{:<20} {:>10} {:>10.1} {:>10}",
        "Kills",
        kmin,
        avg(&kills),
        kmax
    );
    println!(
        "{:<20} {:>10} {:>10.1} {:>10}",
        "Deaths",
        dmin,
        avg(&deaths),
        dmax
    );
    println!(
        "{:<20} {:>10} {:>10.1} {:>10}",
        "Boss Kills",
        bmin,
        avg(&boss_kills),
        bmax
    );
    println!(
        "{:<20} {:>10} {:>10.1} {:>10}",
        "Items Found",
        imin,
        avg(&items),
        imax
    );
    println!(
        "{:<20} {:>10} {:>10.1} {:>10}",
        "Total XP",
        xmin,
        avg(&xp),
        xmax
    );
    println!(
        "{:<20} {:>10} {:>10.1} {:>10}",
        "Achievements",
        amin,
        avg(&achievements),
        amax
    );
    println!();

    // Final zone distribution
    let mut zone_counts: HashMap<(u32, u32), u32> = HashMap::new();
    for s in all_stats {
        *zone_counts.entry(s.final_zone).or_insert(0) += 1;
    }
    let mut zone_dist: Vec<_> = zone_counts.iter().collect();
    zone_dist.sort_by_key(|&(k, _)| (k.0, k.1));
    println!("Final zone distribution:");
    for ((z, s), count) in &zone_dist {
        println!("  Zone {z}-{s}: {count} runs");
    }
    println!();
}

// ── Main ─────────────────────────────────────────────────────────────

fn main() {
    let config = parse_args();

    if !config.quiet {
        eprintln!(
            "Quest Simulator: {} ticks ({}) x {} run(s), seed={}, prestige=P{}, stormbreaker={}",
            config.ticks,
            ticks_to_time(config.ticks),
            config.runs,
            config.seed,
            config.prestige,
            config.stormbreaker,
        );
    }

    let mut all_stats = Vec::with_capacity(config.runs as usize);

    for run in 0..config.runs {
        let seed = config.seed + run as u64;

        if !config.quiet && config.runs > 1 {
            eprintln!("--- Run {}/{} (seed={seed}) ---", run + 1, config.runs);
        }

        let (stats, final_state) = run_simulation(&config, seed);

        if config.runs == 1 {
            // Single run: print full final state
            print_summary(&stats, seed, &config);
            print_final_equipment(&final_state);
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

fn print_final_equipment(state: &GameState) {
    println!("--- Final Equipment ---");
    let slots = [
        ("Weapon", &state.equipment.weapon),
        ("Armor", &state.equipment.armor),
        ("Helmet", &state.equipment.helmet),
        ("Gloves", &state.equipment.gloves),
        ("Boots", &state.equipment.boots),
        ("Amulet", &state.equipment.amulet),
        ("Ring", &state.equipment.ring),
    ];
    for (name, item) in &slots {
        match item {
            Some(i) => println!("  {name}: {} ({:?})", i.display_name, i.rarity),
            None => println!("  {name}: (empty)"),
        }
    }
    println!();
}
