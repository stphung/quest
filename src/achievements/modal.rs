//! Modal notification queue for the achievement system.
//!
//! Manages the 500ms accumulation window that batches achievement notifications
//! before displaying them in a modal overlay.

use super::types::{AchievementId, Achievements};

impl Achievements {
    /// Check if the achievement modal is ready to show.
    /// Returns true if there are queued achievements and 500ms has elapsed.
    pub fn is_modal_ready(&self) -> bool {
        if self.modal_queue.is_empty() {
            return false;
        }
        if let Some(start) = self.accumulation_start {
            start.elapsed() >= std::time::Duration::from_millis(500)
        } else {
            false
        }
    }

    /// Take the modal queue for display (clears queue and resets timer).
    pub fn take_modal_queue(&mut self) -> Vec<AchievementId> {
        self.accumulation_start = None;
        std::mem::take(&mut self.modal_queue)
    }
}
