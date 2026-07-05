//! Balance assertions for CI regression detection.

use super::stats::SimStats;

#[derive(Debug)]
pub enum AssertionOp {
    LessOrEqual,
    GreaterOrEqual,
}

pub struct Assertion {
    pub name: &'static str,
    pub metric: fn(&SimStats) -> f64,
    pub op: AssertionOp,
    pub value: f64,
}

impl Assertion {
    pub fn check(&self, stats: &SimStats) -> bool {
        let actual = (self.metric)(stats);
        match self.op {
            AssertionOp::LessOrEqual => actual <= self.value,
            AssertionOp::GreaterOrEqual => actual >= self.value,
        }
    }
}

pub fn builtin_assertions() -> Vec<Assertion> {
    vec![
        Assertion {
            name: "Zone 2 reachable within 30min at P0",
            metric: |s| {
                s.zone_entry_tick
                    .keys()
                    .filter(|(z, _)| *z >= 2)
                    .filter_map(|k| s.zone_entry_tick.get(k))
                    .copied()
                    .min()
                    .unwrap_or(u64::MAX) as f64
            },
            op: AssertionOp::LessOrEqual,
            value: 18_000.0, // 30 min in ticks
        },
        Assertion {
            name: "Level 20 reachable within 1hr at P0",
            metric: |s| s.level_at_tick.get(&20).copied().unwrap_or(u64::MAX) as f64,
            op: AssertionOp::LessOrEqual,
            value: 36_000.0,
        },
        Assertion {
            name: "PR income exceeds PR spending by tick 50000",
            metric: |s| {
                if s.total_ticks >= 50_000 && (s.pr_earned > 0 || s.pr_spent > 0) {
                    s.pr_earned as f64 - s.pr_spent as f64
                } else {
                    1.0 // pass if sim didn't run long enough or no strategy
                }
            },
            op: AssertionOp::GreaterOrEqual,
            value: 0.0,
        },
    ]
}

pub fn run_assertions(stats: &SimStats) -> bool {
    let assertions = builtin_assertions();
    let mut all_pass = true;

    println!();
    println!("=== Balance Assertions ===");
    println!();

    for assertion in &assertions {
        let passed = assertion.check(stats);
        let actual = (assertion.metric)(stats);
        let status = if passed { "PASS" } else { "FAIL" };
        let icon = if passed { "\u{2714}" } else { "\u{2718}" };

        println!("{icon} [{status}] {}", assertion.name);
        if !passed {
            println!(
                "         actual={actual:.0}, expected {:?} {:.0}",
                assertion.op, assertion.value
            );
            all_pass = false;
        }
    }

    println!();
    if all_pass {
        println!("All assertions passed.");
    } else {
        println!("Some assertions FAILED.");
    }

    all_pass
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats_with(zone_tick: u64, level_20_tick: u64, pr_earned: u64, pr_spent: u64) -> SimStats {
        let mut stats = SimStats::default();
        stats.zone_entry_tick.insert((2, 1), zone_tick);
        stats.level_at_tick.insert(20, level_20_tick);
        stats.total_ticks = 50_000;
        stats.pr_earned = pr_earned;
        stats.pr_spent = pr_spent;
        stats
    }

    #[test]
    fn less_or_equal_passes_at_and_under_threshold() {
        let assertion = Assertion {
            name: "test",
            metric: |s| s.total_ticks as f64,
            op: AssertionOp::LessOrEqual,
            value: 100.0,
        };
        let mut stats = SimStats {
            total_ticks: 100,
            ..Default::default()
        };
        assert!(assertion.check(&stats));
        stats.total_ticks = 101;
        assert!(!assertion.check(&stats));
    }

    #[test]
    fn greater_or_equal_passes_at_and_over_threshold() {
        let assertion = Assertion {
            name: "test",
            metric: |s| s.total_kills as f64,
            op: AssertionOp::GreaterOrEqual,
            value: 10.0,
        };
        let mut stats = SimStats {
            total_kills: 10,
            ..Default::default()
        };
        assert!(assertion.check(&stats));
        stats.total_kills = 9;
        assert!(!assertion.check(&stats));
    }

    #[test]
    fn builtin_assertions_has_expected_names() {
        let assertions = builtin_assertions();
        assert_eq!(assertions.len(), 3);
        assert!(assertions.iter().any(|a| a.name.contains("Zone 2")));
        assert!(assertions.iter().any(|a| a.name.contains("Level 20")));
        assert!(assertions.iter().any(|a| a.name.contains("PR income")));
    }

    #[test]
    fn run_assertions_passes_with_healthy_stats() {
        let stats = stats_with(1_000, 2_000, 100, 50);
        assert!(run_assertions(&stats));
    }

    #[test]
    fn run_assertions_fails_when_zone_2_unreachable() {
        let stats = stats_with(u64::MAX, 2_000, 100, 50);
        assert!(!run_assertions(&stats));
    }

    #[test]
    fn run_assertions_pr_check_skips_when_sim_too_short() {
        // total_ticks < 50_000 with pr_earned/pr_spent both 0 => metric returns
        // the pass-through 1.0 sentinel regardless of actual PR balance.
        let mut stats = SimStats::default();
        stats.zone_entry_tick.insert((2, 1), 100);
        stats.level_at_tick.insert(20, 100);
        stats.total_ticks = 1_000;
        assert!(run_assertions(&stats));
    }
}
