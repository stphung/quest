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
            name: "Zone 5 reachable within 30min at P0",
            metric: |s| {
                s.zone_entry_tick
                    .keys()
                    .filter(|(z, _)| *z >= 5)
                    .filter_map(|k| s.zone_entry_tick.get(k))
                    .copied()
                    .min()
                    .unwrap_or(u64::MAX) as f64
            },
            op: AssertionOp::LessOrEqual,
            value: 18_000.0, // 30 min in ticks
        },
        Assertion {
            name: "Level 50 reachable within 1hr at P0",
            metric: |s| s.level_at_tick.get(&50).copied().unwrap_or(u64::MAX) as f64,
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
