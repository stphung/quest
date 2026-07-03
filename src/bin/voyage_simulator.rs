//! Headless Act 2 voyage simulator — the balance gate for route authoring
//! (spec 2). Plays whole crossings with simple strategies against synthetic
//! wall-clock time and asserts the structural promises: every strategy
//! reaches the Tree, day counts stay inside the pacing envelope, and no
//! state ever lacks a selectable road.
//!
//! Usage:
//!   voyage_simulator [--runs N] [--seed N] [--strategy S] [--checkin-hours H]
//!   strategies: cheapest | priciest | random | mourn | all (default)

use chrono::{DateTime, Duration, Utc};
use quest::vessel::junction::current_junction_cards;
use quest::vessel::voyage::{Trim, VoyageState};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Clone, Copy, PartialEq)]
enum Strategy {
    Cheapest,
    Priciest,
    Random,
    Mourn,
}

impl Strategy {
    fn name(&self) -> &'static str {
        match self {
            Strategy::Cheapest => "cheapest",
            Strategy::Priciest => "priciest",
            Strategy::Random => "random",
            Strategy::Mourn => "mourn",
        }
    }

    fn trim(&self) -> Trim {
        match self {
            Strategy::Priciest => Trim::Run,
            Strategy::Mourn => Trim::Mourn,
            _ => Trim::Cruise,
        }
    }
}

struct RunResult {
    days: u64,
    drifts: bool,
    hope: u8,
    waypoints: usize,
}

fn simulate(strategy: Strategy, seed: u64, checkin_hours: i64) -> RunResult {
    let t0: DateTime<Utc> = "2026-01-01T00:00:00Z".parse().unwrap();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut v = VoyageState::begin(format!("sim-{seed}"), seed, t0);
    v.intro_pending = false;
    v.set_trim(strategy.trim());

    let mut now = t0;
    let mut drifted = false;
    let mut checkins = 0u32;
    while !v.arrived() {
        checkins += 1;
        assert!(
            checkins < 20_000,
            "[{}/seed {seed}] crossing stuck in {:?}",
            strategy.name(),
            v.phase
        );
        v.play_arrival_scene();
        let cards = current_junction_cards(&v);
        if !cards.is_empty() && !v.arrived() {
            let selectable: Vec<_> = cards.iter().filter(|c| c.selectable).collect();
            assert!(
                !selectable.is_empty(),
                "[{}/seed {seed}] stranded at {:?} with {} provisions",
                strategy.name(),
                v.current_waypoint(),
                v.provisions_display()
            );
            let pick = match strategy {
                Strategy::Cheapest | Strategy::Mourn => 0,
                Strategy::Priciest => selectable.len() - 1,
                Strategy::Random => rng.random_range(0..selectable.len()),
            };
            let mut ordered = selectable.clone();
            ordered.sort_by_key(|c| c.provisions_price);
            v.depart(ordered[pick].road.id).unwrap();
        }
        now += Duration::hours(checkin_hours);
        v.tick(now);
        if matches!(v.phase, quest::vessel::voyage::VoyagePhase::Drifting { .. }) {
            drifted = true;
        }
    }
    RunResult {
        days: v.day_index(),
        drifts: drifted,
        hope: v.hope,
        waypoints: v.visited.len(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut runs = 5u64;
    let mut seed = 1u64;
    let mut checkin_hours = 6i64;
    let mut strategy_arg = "all".to_string();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--runs" => {
                i += 1;
                runs = args[i].parse().expect("--runs takes a number");
            }
            "--seed" => {
                i += 1;
                seed = args[i].parse().expect("--seed takes a number");
            }
            "--checkin-hours" => {
                i += 1;
                checkin_hours = args[i].parse().expect("--checkin-hours takes a number");
            }
            "--strategy" => {
                i += 1;
                strategy_arg = args[i].clone();
            }
            other => {
                eprintln!("unknown flag: {other}");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let strategies: Vec<Strategy> = match strategy_arg.as_str() {
        "cheapest" => vec![Strategy::Cheapest],
        "priciest" => vec![Strategy::Priciest],
        "random" => vec![Strategy::Random],
        "mourn" => vec![Strategy::Mourn],
        "all" => vec![
            Strategy::Cheapest,
            Strategy::Priciest,
            Strategy::Random,
            Strategy::Mourn,
        ],
        other => {
            eprintln!("unknown strategy: {other}");
            std::process::exit(1);
        }
    };

    println!("voyage_simulator — {runs} run(s) per strategy, check-in every {checkin_hours}h\n");
    for strategy in strategies {
        for run in 0..runs {
            let result = simulate(strategy, seed + run, checkin_hours);
            // Attended-crossing envelope: legs are 1-3 days, ~24 waypoints,
            // plus hold/drift slack. Real calendars stretch with player
            // cadence; this bounds the authored road data itself.
            assert!(
                (20..=200).contains(&result.days),
                "[{}/seed {}] {} days is outside the envelope",
                strategy.name(),
                seed + run,
                result.days
            );
            println!(
                "  {:<9} seed {:<3} arrived day {:>3} \u{00b7} {} waypoints \u{00b7} hope {}{}",
                strategy.name(),
                seed + run,
                result.days,
                result.waypoints,
                result.hope,
                if result.drifts {
                    " \u{00b7} drifted"
                } else {
                    ""
                }
            );
        }
    }
    println!("\nAll crossings reached the Tree.");
}
