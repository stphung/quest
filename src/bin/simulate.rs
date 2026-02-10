//! Game balance simulator CLI.
//!
//! Run Monte Carlo simulations to analyze game balance.
//!
//! Usage:
//!   cargo run --bin simulate -- [OPTIONS]
//!
//! Examples:
//!   cargo run --bin simulate                    # Default: 1000 runs to zone 10
//!   cargo run --bin simulate -- -n 100 -z 5    # 100 runs to zone 5
//!   cargo run --bin simulate -- --seed 42      # Reproducible run

use quest::simulator::{run_simulation, SimConfig};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (config, show_level_curve) = parse_args(&args);

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║              QUEST BALANCE SIMULATOR                          ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Configuration:");
    println!("  Runs:           {}", config.num_runs);
    println!("  Target Zone:    {}", config.target_zone);
    println!("  Simulate Loot:  {}", config.simulate_loot);
    println!("  Max Ticks:      {}", config.max_ticks_per_run);
    if config.starting_prestige > 0 {
        println!("  Starting at:    P{}", config.starting_prestige);
    }
    if config.simulate_prestige {
        println!(
            "  Prestige:       enabled (target P{})",
            config.target_prestige
        );
    }
    if let Some(seed) = config.seed {
        println!("  Seed:           {}", seed);
    }
    println!();
    println!("Running simulation...");
    println!();

    let report = run_simulation(&config);

    println!("{}", report.to_text());

    // Show detailed level curve if requested
    if show_level_curve {
        println!("{}", report.level_curve_text());
    }

    // Optionally save JSON report
    if args.iter().any(|a| a == "--json") {
        let json = report.to_json();
        let filename = format!(
            "sim_report_{}.json",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        std::fs::write(&filename, json).expect("Failed to write JSON report");
        println!("JSON report saved to: {}", filename);
    }
}

fn parse_args(args: &[String]) -> (SimConfig, bool) {
    let mut config = SimConfig::default();
    let mut show_level_curve = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--runs" => {
                if i + 1 < args.len() {
                    config.num_runs = args[i + 1].parse().unwrap_or(1000);
                    i += 1;
                }
            }
            "-z" | "--zone" => {
                if i + 1 < args.len() {
                    config.target_zone = args[i + 1].parse().unwrap_or(10);
                    i += 1;
                }
            }
            "-s" | "--seed" => {
                if i + 1 < args.len() {
                    config.seed = args[i + 1].parse().ok();
                    i += 1;
                }
            }
            "-t" | "--ticks" => {
                if i + 1 < args.len() {
                    config.max_ticks_per_run = args[i + 1].parse().unwrap_or(1_000_000);
                    i += 1;
                }
            }
            "--no-loot" => {
                config.simulate_loot = false;
            }
            "--prestige" => {
                config.simulate_prestige = true;
                if i + 1 < args.len() {
                    if let Ok(rank) = args[i + 1].parse::<u32>() {
                        config.target_prestige = rank;
                        i += 1;
                    }
                }
            }
            "--start" => {
                if i + 1 < args.len() {
                    if let Ok(rank) = args[i + 1].parse::<u32>() {
                        config.starting_prestige = rank;
                        i += 1;
                    }
                }
            }
            "--level-curve" => {
                show_level_curve = true;
            }
            "-v" | "--verbose" => {
                config.verbosity = 2;
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--quick" => {
                config = SimConfig::zone_balance_test(5);
            }
            "--full" => {
                config = SimConfig::full_progression_test();
            }
            _ => {}
        }
        i += 1;
    }

    (config, show_level_curve)
}

fn print_help() {
    println!("Quest Balance Simulator");
    println!();
    println!("USAGE:");
    println!("    cargo run --bin simulate -- [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --runs <N>      Number of simulation runs (default: 1000)");
    println!("    -z, --zone <Z>      Target zone to reach (default: 10)");
    println!("    -s, --seed <S>      Random seed for reproducibility");
    println!("    -t, --ticks <T>     Max ticks per run (default: 1,000,000)");
    println!("    --no-loot           Disable loot simulation");
    println!("    --start <R>         Start at prestige rank R (test mid-game balance)");
    println!("    --prestige <R>      Simulate prestige to rank R");
    println!("    --level-curve       Show detailed level-up pacing");
    println!("    -v, --verbose       Verbose output");
    println!("    --json              Save JSON report");
    println!("    --quick             Quick test (100 runs to zone 5)");
    println!("    --full              Full test (50 runs with prestige)");
    println!("    -h, --help          Show this help");
    println!();
    println!("EXAMPLES:");
    println!("    cargo run --bin simulate                    # Default run");
    println!("    cargo run --bin simulate -- -n 100 -z 5    # 100 runs to zone 5");
    println!("    cargo run --bin simulate -- --seed 42      # Reproducible");
    println!("    cargo run --bin simulate -- --quick        # Quick balance check");
    println!("    cargo run --bin simulate -- --prestige 3   # Prestige to rank 3");
    println!("    cargo run --bin simulate -- --full --level-curve  # Full with level pacing");
}
