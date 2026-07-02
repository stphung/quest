//! Progression scenarios for CI regression gating.
//!
//! Each scenario simulates a slice of the player journey (early game, prestige
//! economy, endgame systems) across several seeds and asserts coarse
//! progression facts: the hero levels, clears zones, earns and spends PR, and
//! unlocks the late-game systems. Run via `simulator --check-progression`.
//!
//! Thresholds carry ~2x headroom over observed values because the tick loop is
//! not perfectly deterministic (some systems read wall-clock time), so a
//! failure here means progression genuinely regressed, not that RNG drifted.

use super::assertions::{Assertion, AssertionOp};
use super::report::ticks_to_time;
use super::stats::SimStats;
use super::strategy::StrategyProfile;
use super::{run_simulation, SimConfig};

pub struct Scenario {
    pub name: &'static str,
    pub description: &'static str,
    pub ticks: u64,
    pub seeds: &'static [u64],
    pub prestige: u32,
    pub strategy: Option<StrategyProfile>,
    pub stormbreaker: bool,
    pub assertions: fn() -> Vec<Assertion>,
}

impl Scenario {
    fn config(&self) -> SimConfig {
        SimConfig {
            ticks: self.ticks,
            prestige: self.prestige,
            strategy: self.strategy,
            stormbreaker: self.stormbreaker,
            quiet: true,
            ..SimConfig::default()
        }
    }
}

fn first_tick_at_zone(stats: &SimStats, zone: u32) -> f64 {
    stats
        .zone_entry_tick
        .iter()
        .filter(|((z, _), _)| *z >= zone)
        .map(|(_, tick)| *tick)
        .min()
        .unwrap_or(u64::MAX) as f64
}

fn max_zone_entered(stats: &SimStats) -> f64 {
    stats
        .zone_entry_tick
        .keys()
        .map(|(z, _)| *z)
        .max()
        .unwrap_or(1) as f64
}

fn max_level_reached(stats: &SimStats) -> f64 {
    stats
        .level_at_tick
        .keys()
        .copied()
        .max()
        .unwrap_or(1)
        .max(stats.final_level) as f64
}

/// Fresh hero, no strategy: the core kill/XP/zone loop must move on its own.
fn early_game_assertions() -> Vec<Assertion> {
    vec![
        Assertion {
            name: "Zone 2 reached within 45min (observed ~10-22min)",
            metric: |s| first_tick_at_zone(s, 2),
            op: AssertionOp::LessOrEqual,
            value: 27_000.0,
        },
        Assertion {
            name: "Level 20 reached within 1h (observed ~19-23min)",
            metric: |s| s.level_at_tick.get(&20).copied().unwrap_or(u64::MAX) as f64,
            op: AssertionOp::LessOrEqual,
            value: 36_000.0,
        },
        Assertion {
            name: "At least 300 kills in 2h (observed ~650-740)",
            metric: |s| s.total_kills as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 300.0,
        },
        Assertion {
            name: "At least 15 boss kills in 2h (observed ~41-48)",
            metric: |s| s.total_boss_kills as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 15.0,
        },
        Assertion {
            name: "At least 50 item drops in 2h (observed ~147-179)",
            metric: |s| s.items_by_rarity.iter().sum::<u64>() as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 50.0,
        },
        Assertion {
            name: "At most 20 deaths in 2h (observed 0-1)",
            metric: |s| s.total_deaths as f64,
            op: AssertionOp::LessOrEqual,
            value: 20.0,
        },
        Assertion {
            name: "At least 5 achievements in 2h (observed 10-13)",
            metric: |s| s.achievements_unlocked as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 5.0,
        },
    ]
}

/// Optimal-strategy hero from P0: prestige loop and PR economy must function.
fn prestige_economy_assertions() -> Vec<Assertion> {
    vec![
        Assertion {
            // PR doubles as spendable currency, so final prestige balance can
            // legitimately hit 0 — count prestige events instead.
            name: "Auto-prestige fires 3+ times in 6h (observed ~14-15)",
            metric: |s| s.prestiges_performed as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 3.0,
        },
        Assertion {
            name: "At least 15 PR earned in 6h (observed 28-46)",
            metric: |s| s.pr_earned as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 15.0,
        },
        Assertion {
            name: "PR income covers PR spending from P0",
            metric: |s| s.pr_earned as f64 - s.pr_spent as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 0.0,
        },
        Assertion {
            name: "At least 8 challenges won in 6h (observed ~17)",
            metric: |s| s.challenges_won as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 8.0,
        },
        Assertion {
            name: "At least 6 Haven rooms built in 6h (observed ~15)",
            metric: |s| s.haven_rooms_built as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 6.0,
        },
        Assertion {
            name: "At least 12 achievements in 6h (observed ~20)",
            metric: |s| s.achievements_unlocked as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 12.0,
        },
    ]
}

/// Veteran P200 speedrunner: Deep, fracture zones, Loom, and Ascension must
/// all unlock and compound so the hero pushes past Zone 10.
fn endgame_systems_assertions() -> Vec<Assertion> {
    vec![
        Assertion {
            name: "Deep reaches layer 25+ in 30h (observed 30)",
            metric: |s| s.deep_layers_reached as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 25.0,
        },
        Assertion {
            name: "Fracture zone cap reaches Z27+ (observed Z30)",
            metric: |s| s.fracture_zone_cap as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 27.0,
        },
        Assertion {
            name: "At least 4 Loom patterns completed (observed 6)",
            metric: |s| s.loom_patterns_completed as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 4.0,
        },
        Assertion {
            name: "Loom zone cap reaches Z31+ (observed Z34)",
            metric: |s| s.loom_zone_cap as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 31.0,
        },
        Assertion {
            name: "Ascension I+ reached (observed II)",
            metric: |s| s.ascension_level as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 1.0,
        },
        Assertion {
            name: "Pushes into fracture zones Z13+ (observed Z16-17)",
            metric: max_zone_entered,
            op: AssertionOp::GreaterOrEqual,
            value: 13.0,
        },
        Assertion {
            name: "Reaches level 100+ (observed ~370-420)",
            metric: max_level_reached,
            op: AssertionOp::GreaterOrEqual,
            value: 100.0,
        },
    ]
}

pub fn scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "early-game",
            description: "Fresh P0 hero, no strategy, 2h",
            ticks: 72_000,
            seeds: &[1, 4, 7, 10, 13],
            prestige: 0,
            strategy: None,
            stormbreaker: false,
            assertions: early_game_assertions,
        },
        Scenario {
            name: "prestige-economy",
            description: "P0 hero, optimal strategy, 6h",
            ticks: 216_000,
            seeds: &[1, 5, 9],
            prestige: 0,
            strategy: Some(StrategyProfile::Optimal),
            stormbreaker: true,
            assertions: prestige_economy_assertions,
        },
        Scenario {
            name: "endgame-systems",
            description: "P200 hero, speedrun strategy, 30h",
            ticks: 1_080_000,
            seeds: &[1, 9],
            prestige: 200,
            strategy: Some(StrategyProfile::Speedrun),
            stormbreaker: true,
            assertions: endgame_systems_assertions,
        },
    ]
}

/// Run every scenario across all its seeds and print a pass/fail report.
/// Returns true only if every assertion passes for every seed.
pub fn run_progression_check() -> bool {
    let mut all_pass = true;

    println!("=== Progression Check ===");

    for scenario in scenarios() {
        let config = scenario.config();
        let assertions = (scenario.assertions)();

        println!();
        println!(
            "--- {} ({}, {} ticks = {}, {} seed(s)) ---",
            scenario.name,
            scenario.description,
            scenario.ticks,
            ticks_to_time(scenario.ticks),
            scenario.seeds.len(),
        );

        for &seed in scenario.seeds {
            let (stats, _, _) = run_simulation(&config, seed);

            for assertion in &assertions {
                let actual = (assertion.metric)(&stats);
                let passed = assertion.check(&stats);
                if passed {
                    println!(
                        "  \u{2714} [seed {seed:>2}] {} (actual={actual:.0})",
                        assertion.name
                    );
                } else {
                    println!(
                        "  \u{2718} [seed {seed:>2}] {} FAILED: actual={actual:.0}, expected {:?} {:.0}",
                        assertion.name, assertion.op, assertion.value
                    );
                    all_pass = false;
                }
            }
        }
    }

    println!();
    if all_pass {
        println!("Progression check PASSED.");
    } else {
        println!("Progression check FAILED — the game no longer progresses as expected.");
    }

    all_pass
}
