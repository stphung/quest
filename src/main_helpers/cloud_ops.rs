//! Shared helpers for cloud sync operations to avoid duplication
//! between the game loop (main.rs) and character screens.

/// Reload all account-level state from disk after a cloud pull or resolve.
///
/// Called after `fast_forward_all()`, `reset_to_remote()`, or `backup_and_reset()`
/// updates files on disk.
pub fn reload_account_state(
    haven: &mut crate::haven::types::Haven,
    enhancement: &mut crate::enhancement::types::EnhancementProgress,
    global_achievements: &mut crate::achievements::types::Achievements,
) {
    *haven = crate::haven::load_haven();
    *enhancement = crate::enhancement::load_enhancement();
    *global_achievements = crate::achievements::load_achievements();
    crate::achievements::titles::validate_selected_title(global_achievements);
    global_achievements.refresh_progress();
}
