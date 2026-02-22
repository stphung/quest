//! Achievement system module.
//!
//! Provides a global achievement system that tracks player progress
//! across all characters. Achievements are stored in `~/.quest/achievements.json`.

pub mod data;
pub mod handlers;
pub mod milestones;
pub mod modal;
pub mod notifications;
pub mod persistence;
pub mod stats;
pub mod titles;
pub mod types;
pub mod unlock;

pub use data::{get_achievement_def, get_achievements_by_category};
pub use milestones::{MinigameDifficulty, MinigameType};
pub use persistence::{load_achievements, save_achievements};
#[allow(unused_imports)]
pub use titles::{get_title_text, get_unlocked_titles};
pub use types::{
    AchievementCategory, AchievementId, Achievements, UiBorderStyle, SELECTABLE_UI_BORDER_STYLES,
};
