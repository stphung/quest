//! Core unlock machinery for the achievement system.
//!
//! Contains the fundamental methods for checking and granting achievements:
//! `is_unlocked`, `unlock`, `unlock_with_name`, and `check_milestones`.

use super::types::{AchievementId, Achievements, UnlockedAchievement};

impl Achievements {
    /// Check if an achievement is unlocked.
    pub fn is_unlocked(&self, id: AchievementId) -> bool {
        self.unlocked.contains_key(&id)
    }

    /// Unlock an achievement. Returns true if newly unlocked.
    pub fn unlock(&mut self, id: AchievementId, character_name: Option<String>) -> bool {
        if self.is_unlocked(id) {
            return false;
        }
        self.unlocked.insert(
            id,
            UnlockedAchievement {
                unlocked_at: chrono::Utc::now().timestamp(),
                character_name,
            },
        );
        self.pending_notifications.push(id);
        self.newly_unlocked.push(id);

        // Add to modal queue and start accumulation timer if not already started
        self.modal_queue.push(id);
        if self.accumulation_start.is_none() {
            self.accumulation_start = Some(std::time::Instant::now());
        }

        true
    }

    /// Convenience wrapper: unlock with an `Option<&str>` name (avoids repeated `.map(|s| s.to_string())`).
    pub(super) fn unlock_with_name(
        &mut self,
        id: AchievementId,
        character_name: Option<&str>,
    ) -> bool {
        self.unlock(id, character_name.map(|s| s.to_string()))
    }

    /// Helper to check and unlock milestones. Checks all milestones in order.
    /// Short-circuits already-unlocked achievements to avoid String allocations
    /// and HashMap updates on every call.
    pub(super) fn check_milestones(
        &mut self,
        current: u64,
        milestones: &[(u64, AchievementId)],
        character_name: Option<&str>,
    ) {
        for &(threshold, achievement_id) in milestones {
            if self.is_unlocked(achievement_id) {
                continue;
            }
            if current >= threshold {
                self.unlock_with_name(achievement_id, character_name);
            } else {
                self.update_progress(achievement_id, current, threshold);
            }
        }
    }
}
