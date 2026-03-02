//! Shared helpers for cloud sync operations to avoid duplication
//! between the game loop (main.rs) and character screens.

/// Spawn a background thread to push all branches to cloud.
///
/// Sets `cloud_op_in_flight` to `true` and sends the result via `tx`.
/// Callers must check `cloud_op_in_flight` and `cloud_config` before calling.
pub fn spawn_cloud_push(
    cloud_op_in_flight: &mut bool,
    tx: &std::sync::mpsc::Sender<crate::history::cloud::CloudOpResult>,
    quest_dir: &std::path::Path,
    token: &str,
) {
    *cloud_op_in_flight = true;
    let tx = tx.clone();
    let dir = quest_dir.to_path_buf();
    let tok = token.to_string();
    std::thread::spawn(move || {
        let tx2 = tx.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let res = match crate::history::cloud::push_all_branches(&dir, &tok) {
                Ok(()) => crate::history::cloud::CloudOpResult::Pushed,
                Err(e) => crate::history::cloud::CloudOpResult::Failed(e),
            };
            let _ = tx.send(res);
        }));
        if result.is_err() {
            let _ = tx2.send(crate::history::cloud::CloudOpResult::Failed(
                "Cloud operation failed unexpectedly".to_string(),
            ));
        }
    });
}

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
