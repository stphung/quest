//! Achievement statistics and progress tracking.
//!
//! Methods for querying achievement counts, unlock percentages,
//! progress on multi-stage achievements, and category breakdowns.

use super::types::{AchievementCategory, AchievementId, AchievementProgress, Achievements};

impl Achievements {
    /// Take newly unlocked achievements for logging (clears the list).
    pub fn take_newly_unlocked(&mut self) -> Vec<AchievementId> {
        std::mem::take(&mut self.newly_unlocked)
    }

    /// Update progress on a tracked achievement.
    pub fn update_progress(&mut self, id: AchievementId, current: u64, target: u64) {
        self.progress
            .insert(id, AchievementProgress { current, target });
    }

    /// Get the progress for an achievement, if any.
    pub fn get_progress(&self, id: AchievementId) -> Option<&AchievementProgress> {
        self.progress.get(&id)
    }

    /// Get the total number of achievements.
    pub fn total_count(&self) -> usize {
        use super::data::ALL_ACHIEVEMENTS;
        ALL_ACHIEVEMENTS.len()
    }

    /// Get the number of unlocked achievements.
    pub fn unlocked_count(&self) -> usize {
        self.unlocked.len()
    }

    /// Get unlock percentage (0.0 - 100.0).
    pub fn unlock_percentage(&self) -> f32 {
        let total = self.total_count();
        if total == 0 {
            return 0.0;
        }
        (self.unlocked_count() as f32 / total as f32) * 100.0
    }

    /// Get count of unlocked/total by category.
    pub fn count_by_category(&self, category: AchievementCategory) -> (usize, usize) {
        use super::data::ALL_ACHIEVEMENTS;
        ALL_ACHIEVEMENTS
            .iter()
            .filter(|a| a.category == category)
            .fold((0, 0), |(unlocked, total), a| {
                (unlocked + self.is_unlocked(a.id) as usize, total + 1)
            })
    }
}
