//! Game input result routing extracted from main.rs game loop.

use chrono::Local;
use std::time::Instant;

use crate::character::manager::CharacterManager;
use crate::history::SaveEvent;
use crate::input::InputResult;
use crate::main_helpers::game_context::GameContext;

use super::persistence::{save_all, save_files};

/// What the game loop should do after processing an input result.
pub enum InputAction {
    /// Continue the game loop normally.
    Continue,
    /// Save completed; commit deferred to next tick for sequential indicator.
    DeferCommit(SaveEvent),
    /// Player quit to character select — break the game loop.
    QuitToSelect,
}

/// Process the result of `handle_game_input()` and perform side effects
/// (saving, toggling update details).
///
/// Returns an [`InputAction`] telling the caller whether to continue or
/// exit the game loop.
pub fn route_game_input(
    result: InputResult,
    ctx: &GameContext<'_>,
    character_manager: &CharacterManager,
    last_save_instant: &mut Option<Instant>,
    last_save_time: &mut Option<chrono::DateTime<Local>>,
) -> InputAction {
    let state = &*ctx.state;
    let global_achievements = &*ctx.achievements;
    let haven = &*ctx.haven;
    let enhancement = &*ctx.enhancement;
    let deep = &*ctx.deep_state;
    let debug_mode = ctx.debug_mode;

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
                    deep,
                    None,
                    None,
                );
            }
            InputAction::QuitToSelect
        }
        InputResult::NeedsSave | InputResult::NeedsSaveAll => {
            if !debug_mode {
                save_files(
                    character_manager,
                    state,
                    global_achievements,
                    haven,
                    enhancement,
                    deep,
                );
                *last_save_instant = Some(Instant::now());
                *last_save_time = Some(Local::now());
            }
            InputAction::Continue
        }
        InputResult::NeedsSaveWithEvent(event) | InputResult::NeedsSaveAllWithEvent(event) => {
            if !debug_mode {
                save_files(
                    character_manager,
                    state,
                    global_achievements,
                    haven,
                    enhancement,
                    deep,
                );
                *last_save_instant = Some(Instant::now());
                *last_save_time = Some(Local::now());
                InputAction::DeferCommit(event)
            } else {
                InputAction::Continue
            }
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
        // Cloud actions are handled directly in main.rs before reaching
        // route_game_input, but must be matched for exhaustiveness.
        InputResult::ValidateToken { .. }
        | InputResult::ChangeRepo
        | InputResult::LinkCloud { .. }
        | InputResult::PushCloud
        | InputResult::PullCloud
        | InputResult::UnlinkCloud
        | InputResult::ResolveKeepLocal
        | InputResult::ResolveUseCloud
        | InputResult::ResolveKeepBoth
        | InputResult::UpdateToken { .. } => InputAction::Continue,
    }
}
