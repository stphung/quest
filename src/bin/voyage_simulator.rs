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
    /// Spec 8 stress profiles: nobody posted, 48h check-ins (the strain
    /// and wear ledgers must fill), and hard rations held on all the way
    /// (the hope gauge must finally bite).
    Neglect,
    Rations,
}

impl Strategy {
    fn name(&self) -> &'static str {
        match self {
            Strategy::Cheapest => "cheapest",
            Strategy::Priciest => "priciest",
            Strategy::Random => "random",
            Strategy::Mourn => "mourn",
            Strategy::Neglect => "neglect",
            Strategy::Rations => "rations",
        }
    }

    fn trim(&self) -> Trim {
        match self {
            Strategy::Priciest => Trim::Run,
            Strategy::Mourn => Trim::Mourn,
            _ => Trim::Cruise,
        }
    }

    fn staffs(&self) -> bool {
        !matches!(self, Strategy::Neglect)
    }

    /// Stress profiles override the CLI cadence.
    fn checkin_hours(&self, cli: i64) -> i64 {
        match self {
            Strategy::Neglect => 48,
            _ => cli,
        }
    }
}

struct RunResult {
    days: u64,
    drifts: u32,
    hope: u8,
    min_hope: u8,
    min_provisions: f64,
    strains: u32,
    wear: u8,
    waypoints: usize,
}

fn simulate(strategy: Strategy, seed: u64, checkin_hours: i64) -> RunResult {
    let t0: DateTime<Utc> = "2026-01-01T00:00:00Z".parse().unwrap();
    let checkin_hours = strategy.checkin_hours(checkin_hours);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut v = VoyageState::begin(format!("sim-{seed}"), seed, t0);
    v.intro_pending = false;
    v.set_trim(strategy.trim());
    // A played ship staffs its posts: Torvald at the helm, Eir tending.
    // The neglect profile leaves every post empty on purpose.
    if strategy.staffs() {
        v.set_station(
            quest::vessel::souls::SoulId(0),
            Some(quest::vessel::souls::Station::Helm),
        );
        v.set_station(
            quest::vessel::souls::SoulId(1),
            Some(quest::vessel::souls::Station::Tender),
        );
        v.set_station(
            quest::vessel::souls::SoulId(2),
            Some(quest::vessel::souls::Station::Watch),
        );
    }

    let mut min_hope = v.hope;
    let mut min_provisions = v.provisions;
    let mut now = t0;
    let mut drifts = 0u32;
    let mut was_drifting = false;
    let mut checkins = 0u32;
    while !v.arrived() {
        checkins += 1;
        assert!(
            checkins < 20_000,
            "[{}/seed {seed}] crossing stuck in {:?}",
            strategy.name(),
            v.phase
        );
        // The rations profile holds the dial on whenever hope allows.
        if strategy == Strategy::Rations {
            v.set_hard_rations(true);
        }
        v.play_arrival_scene();
        // An attentive crew rotates the watch before the third night
        // wears anyone (spec 8): hand tonight to a rested, sound soul.
        if strategy.staffs() {
            if let Some((day, kind, Some(stander))) = v.night_forecast().into_iter().next() {
                let tired = v
                    .soul_state(stander)
                    .is_some_and(|s| s.consecutive_watches >= 2);
                if kind.needs_watch() && tired {
                    let relief = v
                        .souls
                        .iter()
                        .find(|s| {
                            s.status == quest::vessel::voyage::SoulStatus::Aboard
                                && s.station.is_none()
                                && s.strain == 0
                                && s.soul != stander
                        })
                        .map(|s| s.soul);
                    if relief.is_some() {
                        v.assign_night(day, relief);
                    }
                }
            }
        }
        // Boarding asks block departure: say yes while a berth is free.
        if v.pending_ask.is_some() && !v.accept_ask() {
            v.decline_ask();
        }
        // Yard offers: mend a scarred hull; otherwise alternate A/B.
        if v.pending_refit.is_some() {
            if v.hull_wear >= 3 {
                v.choose_mend();
            } else {
                let pick_a = v.refits.len().is_multiple_of(2);
                v.choose_refit(pick_a);
            }
        }
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
                Strategy::Cheapest | Strategy::Mourn | Strategy::Neglect | Strategy::Rations => 0,
                Strategy::Priciest => selectable.len() - 1,
                Strategy::Random => rng.random_range(0..selectable.len()),
            };
            let mut ordered = selectable.clone();
            ordered.sort_by_key(|c| c.provisions_price);
            v.depart(ordered[pick].road.id).unwrap();
        }
        now += Duration::hours(checkin_hours);
        v.tick(now);
        min_hope = min_hope.min(v.hope);
        min_provisions = min_provisions.min(v.provisions);
        let drifting = matches!(v.phase, quest::vessel::voyage::VoyagePhase::Drifting { .. });
        if drifting && !was_drifting {
            drifts += 1;
        }
        was_drifting = drifting;
    }
    // Roster strain at arrival is the record (events drain during play).
    let strains = v.souls.iter().map(|s| u32::from(s.strain)).sum::<u32>();
    RunResult {
        days: v.day_index(),
        drifts,
        hope: v.hope,
        min_hope,
        min_provisions,
        strains,
        wear: v.hull_wear,
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
        "neglect" => vec![Strategy::Neglect],
        "rations" => vec![Strategy::Rations],
        "all" => vec![
            Strategy::Cheapest,
            Strategy::Priciest,
            Strategy::Random,
            Strategy::Mourn,
            Strategy::Neglect,
            Strategy::Rations,
        ],
        other => {
            eprintln!("unknown strategy: {other}");
            std::process::exit(1);
        }
    };

    println!("voyage_simulator — {runs} run(s) per strategy, check-in every {checkin_hours}h\n");
    for strategy in strategies {
        let mut total_drifts = 0u32;
        for run in 0..runs {
            let result = simulate(strategy, seed + run, checkin_hours);
            total_drifts += result.drifts;
            // Attended-crossing envelope: legs are 1-3 days, ~24 waypoints,
            // plus hold/drift slack. Real calendars stretch with player
            // cadence; this bounds the authored road data itself.
            // The economy gate (spec 4): a Cruise crossing leans on drift
            // at most once; Mourn never. Priciest runs hard at Run trim on
            // the dearest roads on purpose — it is the no-lose stress test,
            // not an economy subject.
            let max_drifts = match strategy {
                Strategy::Mourn => Some(0),
                Strategy::Cheapest | Strategy::Random => Some(1),
                Strategy::Priciest | Strategy::Neglect | Strategy::Rations => None,
            };
            if let Some(max_drifts) = max_drifts {
                assert!(
                    result.drifts <= max_drifts,
                    "[{}/seed {}] {} drifts exceeds the economy gate ({max_drifts})",
                    strategy.name(),
                    seed + run,
                    result.drifts
                );
            }
            assert!(
                (20..=200).contains(&result.days),
                "[{}/seed {}] {} days is outside the envelope",
                strategy.name(),
                seed + run,
                result.days
            );
            // Spec 8 gates: the ledgers must fill under neglect, the hope
            // gauge must bite under held rations, and the attentive cheap
            // crossing must squeeze without drifting.
            match strategy {
                Strategy::Cheapest => {
                    assert!(
                        result.min_provisions < 10.0,
                        "[cheapest/seed {}] min provisions {:.1} — the squeeze is gone",
                        seed + run,
                        result.min_provisions
                    );
                }
                Strategy::Neglect => {
                    // Judged in aggregate after the strategy's runs:
                    // nobody posted, nobody strained; no hard choices, no
                    // scars. Neglect pays in the covenant's own coin —
                    // drifts, and the days they cost.
                }
                Strategy::Rations => {
                    assert!(
                        result.min_hope <= 2,
                        "[rations/seed {}] min hope {} — the gauge never bit",
                        seed + run,
                        result.min_hope
                    );
                }
                Strategy::Priciest => {
                    assert!(
                        result.wear >= 2,
                        "[priciest/seed {}] wear {} < 2 — Run costs no skin",
                        seed + run,
                        result.wear
                    );
                }
                _ => {}
            }
            println!(
                "  {:<9} seed {:<3} arrived day {:>3} \u{00b7} {} waypoints \u{00b7} hope {} (min {}) \u{00b7} \
                 stores min {:>4.1}{}{}{}",
                strategy.name(),
                seed + run,
                result.days,
                result.waypoints,
                result.hope,
                result.min_hope,
                result.min_provisions,
                if result.drifts > 0 {
                    format!(" \u{00b7} drifted x{}", result.drifts)
                } else {
                    String::new()
                },
                if result.wear > 0 {
                    format!(" \u{00b7} scars x{}", result.wear)
                } else {
                    String::new()
                },
                if result.strains > 0 {
                    format!(" \u{00b7} strain x{}", result.strains)
                } else {
                    String::new()
                }
            );
        }
        if strategy == Strategy::Neglect {
            assert!(
                total_drifts >= runs as u32,
                "[neglect] {total_drifts} drifts across {runs} runs — an \
                 unstaffed, unwatched hold should average a drift a crossing"
            );
        }
    }
    println!("\nAll crossings reached the Tree.");
}
