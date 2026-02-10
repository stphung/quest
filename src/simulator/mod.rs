//! Game balance simulator for Monte Carlo analysis.
//!
//! Run thousands of simulated playthroughs to analyze:
//! - Time to clear zones
//! - Item drop rates and upgrade patterns
//! - Damage/HP balance at each stage
//! - Prestige progression rates
//!
//! The simulator uses CombatLoop (src/core/combat_engine.rs) for all game logic,
//! ensuring simulation results match real gameplay behavior.

mod combat_sim; // Deprecated, kept for backward compatibility
mod config;
mod loot_sim;
mod progression_sim;
mod report;
mod runner;

pub use config::SimConfig;
pub use report::SimReport;
pub use runner::run_simulation;
