//! Notification state management for the achievement system.
//!
//! Methods for tracking pending notifications, recently unlocked achievements,
//! and category-based notification counts.

use super::types::{AchievementCategory, AchievementId, Achievements};

impl Achievements {
    /// Get the count of pending achievement notifications.
    pub fn pending_count(&self) -> usize {
        self.pending_notifications.len()
    }

    /// Clear pending notifications (call when user views achievements).
    pub fn clear_pending_notifications(&mut self) {
        self.recently_unlocked
            .append(&mut self.pending_notifications);
    }

    /// Clear recently unlocked list (call when achievement browser closes).
    pub fn clear_recently_unlocked(&mut self) {
        self.recently_unlocked.clear();
    }

    /// Check if an achievement was recently unlocked (for NEW badge in browser).
    #[allow(dead_code)] // Will be used by achievement browser UI
    pub fn is_recently_unlocked(&self, id: AchievementId) -> bool {
        self.recently_unlocked.contains(&id)
    }

    /// Count recently unlocked achievements in a category (for tab badges).
    #[allow(dead_code)] // Will be used by achievement browser UI
    pub fn count_recently_unlocked_by_category(&self, category: AchievementCategory) -> usize {
        use super::data::ALL_ACHIEVEMENTS;
        self.recently_unlocked
            .iter()
            .filter(|id| {
                ALL_ACHIEVEMENTS
                    .iter()
                    .any(|a| a.id == **id && a.category == category)
            })
            .count()
    }
}
