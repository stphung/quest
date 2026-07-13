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

    /// Get the total number of achievements (visible ones — the Vessel's
    /// stay out of every player-facing count while Act 2 is dark).
    pub fn total_count(&self) -> usize {
        use super::data::{achievement_visible, ALL_ACHIEVEMENTS};
        ALL_ACHIEVEMENTS
            .iter()
            .filter(|a| achievement_visible(a.id))
            .count()
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

    /// Get the total achievement score (sum of points for unlocked achievements).
    pub fn achievement_score(&self) -> u32 {
        use super::data::ALL_ACHIEVEMENTS;
        ALL_ACHIEVEMENTS
            .iter()
            .filter(|a| self.is_unlocked(a.id))
            .map(|a| a.points)
            .sum()
    }

    /// Get the maximum possible achievement score (sum of all visible
    /// achievements' points).
    pub fn max_achievement_score() -> u32 {
        use super::data::{achievement_visible, ALL_ACHIEVEMENTS};
        ALL_ACHIEVEMENTS
            .iter()
            .filter(|a| achievement_visible(a.id))
            .map(|a| a.points)
            .sum()
    }

    /// Get count of unlocked/total by category (visible achievements only).
    pub fn count_by_category(&self, category: AchievementCategory) -> (usize, usize) {
        use super::data::{achievement_visible, ALL_ACHIEVEMENTS};
        ALL_ACHIEVEMENTS
            .iter()
            .filter(|a| a.category == category && achievement_visible(a.id))
            .fold((0, 0), |(unlocked, total), a| {
                (unlocked + self.is_unlocked(a.id) as usize, total + 1)
            })
    }
}
