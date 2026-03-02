//! Power Cores — passive prestige-rank generation from Deep layer milestones.
//!
//! Each Power Core is tied to one of the six Power Core achievements
//! (`PowerCoreI` through `PowerCoreVI`), unlocked at Deep layers 3, 7, 12,
//! 18, 25, and 30 (the fracture zone unlock layers).  When a player clears
//! the corresponding Deep layer the matching core becomes active and begins
//! generating prestige ranks passively at a fixed rate (2–18 PR/day).
//!
//! State is persisted as part of `DeepPersistent` in `~/.quest/deep.json`.

pub mod tick;
pub mod types;

#[allow(unused_imports)]
pub use tick::{apply_offline_power_cores, init_new_core, tick_power_cores};
#[allow(unused_imports)]
pub use types::{
    fill_duration_secs, get_power_core_def, get_unlocked_cores, PowerCoreDef, ALL_POWER_CORES,
};
