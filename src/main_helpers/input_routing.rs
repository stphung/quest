//! Game input result routing extracted from main.rs game loop.

use chrono::Local;
use std::time::Instant;

use crate::character::manager::CharacterManager;
use crate::core::game_state::GameState;
use crate::history::HistoryRepo;
use crate::input::InputResult;

use super::persistence::save_all;

/// What the game loop should do after processing an input result.
pub enum InputAction {
    /// Continue the game loop normally.
    Continue,
    /// Player quit to character select — break the game loop.
    QuitToSelect,
}

/// Process the result of `handle_game_input()` and perform side effects
/// (saving, toggling update details).
///
/// Returns an [`InputAction`] telling the caller whether to continue or
/// exit the game loop.
#[allow(clippy::too_many_arguments)]
pub fn route_game_input(
    result: InputResult,
    state: &GameState,
    character_manager: &CharacterManager,
    global_achievements: &crate::achievements::Achievements,
    haven: &crate::haven::Haven,
    enhancement: &crate::enhancement::EnhancementProgress,
    debug_mode: bool,
    update_expanded: &mut bool,
    last_save_instant: &mut Option<Instant>,
    last_save_time: &mut Option<chrono::DateTime<Local>>,
    history_repo: Option<&HistoryRepo>,
) -> InputAction {
    match result {
        InputResult::Continue => InputAction::Continue,
        InputResult::QuitToSelect => {
            if !debug_mode {
                save_all(
                    character_manager,
                    state,
                    global_achievements,
                    haven,
                    enhancement,
                    None,
                    history_repo,
                );
            }
            InputAction::QuitToSelect
        }
        InputResult::NeedsSave | InputResult::NeedsSaveAll => {
            if !debug_mode {
                save_all(
                    character_manager,
                    state,
                    global_achievements,
                    haven,
                    enhancement,
                    None,
                    history_repo,
                );
                *last_save_instant = Some(Instant::now());
                *last_save_time = Some(Local::now());
            }
            InputAction::Continue
        }
        InputResult::NeedsSaveWithEvent(ref event)
        | InputResult::NeedsSaveAllWithEvent(ref event) => {
            if !debug_mode {
                save_all(
                    character_manager,
                    state,
                    global_achievements,
                    haven,
                    enhancement,
                    Some(event),
                    history_repo,
                );
                *last_save_instant = Some(Instant::now());
                *last_save_time = Some(Local::now());
            }
            InputAction::Continue
        }
        InputResult::ToggleUpdateDetails => {
            *update_expanded = !*update_expanded;
            InputAction::Continue
        }
        // StartChronoSurge is handled directly in main.rs before reaching
        // route_game_input, but must be matched for exhaustiveness.
        InputResult::StartChronoSurge { .. } => InputAction::Continue,
        // Time Vault actions are handled directly in main.rs before reaching
        // route_game_input, but must be matched for exhaustiveness.
        InputResult::OpenTimeVault
        | InputResult::RestoreSave { .. }
        | InputResult::RefreshSaveHistoryCommits { .. }
        | InputResult::ForkSave { .. }
        | InputResult::SwitchSaveBranch { .. }
        | InputResult::DeleteSaveBranch { .. } => InputAction::Continue,
    }
}
