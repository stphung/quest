//! Achievement system module.
//!
//! Provides a global achievement system that tracks player progress
//! across all characters. Achievements are stored in `~/.quest/achievements.json`.

pub mod data;
pub mod persistence;
pub mod types;

pub use data::{get_achievement_def, get_achievements_by_category};
pub use persistence::{load_achievements, save_achievements};
pub use types::{AchievementCategory, AchievementId, Achievements};
