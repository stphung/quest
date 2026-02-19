//! Character attributes, stats, and persistence.

#![allow(unused_imports)]

pub mod attributes;
pub mod calculation;
pub mod combat_bonuses;
pub mod creation;
pub mod delete;
pub mod derived_stats;
pub mod input;
pub mod manager;
pub mod multipliers;
pub mod name_validation;
pub mod persistence;
pub mod prestige;
pub mod prestige_actions;
pub mod rename;
pub mod select;
pub mod tiers;

pub use attributes::*;
pub use derived_stats::*;
pub use input::*;
pub use manager::*;
pub use prestige::*;
