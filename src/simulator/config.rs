//! Simulation configuration.

/// Configuration for a simulation run.
#[derive(Debug, Clone)]
pub struct SimConfig {
    /// Number of simulation runs to perform
    pub num_runs: u32,

    /// Random seed for reproducibility (None = random)
    pub seed: Option<u64>,

    /// Maximum ticks (combat rounds) per run before timeout
    pub max_ticks_per_run: u64,

    /// Target zone to reach (1-10)
    pub target_zone: usize,

    /// Target prestige rank to reach
    pub target_prestige: u32,

    /// Starting prestige rank (to test balance at different progression points)
    pub starting_prestige: u32,

    /// Whether to simulate item drops and equipment
    pub simulate_loot: bool,

    /// Whether to simulate prestige resets
    pub simulate_prestige: bool,

    /// Log verbosity (0 = silent, 1 = summary, 2 = detailed)
    pub verbosity: u8,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            num_runs: 1000,
            seed: None,
            max_ticks_per_run: 1_000_000,
            target_zone: 10,
            target_prestige: 0,
            starting_prestige: 0,
            simulate_loot: true,
            simulate_prestige: false,
            verbosity: 1,
        }
    }
}

impl SimConfig {
    /// Quick config for testing zone balance
    pub fn zone_balance_test(target_zone: usize) -> Self {
        Self {
            num_runs: 100,
            target_zone,
            target_prestige: 0,
            simulate_loot: true,
            simulate_prestige: false,
            ..Default::default()
        }
    }

    /// Quick config for testing full progression
    pub fn full_progression_test() -> Self {
        Self {
            num_runs: 50,
            target_zone: 10,
            target_prestige: 5,
            simulate_loot: true,
            simulate_prestige: true,
            ..Default::default()
        }
    }

    /// Quick config for loot analysis
    pub fn loot_analysis(num_runs: u32) -> Self {
        Self {
            num_runs,
            target_zone: 10,
            simulate_loot: true,
            ..Default::default()
        }
    }
}
