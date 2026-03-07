//! Game state persistence (save files & commit).

use crate::achievements;
use crate::character::manager::CharacterManager;
use crate::core::game_state::GameState;
use crate::deep;
use crate::enhancement;
use crate::haven;
use crate::history::{CommitMetadata, HistoryRepo, SaveEvent};
use crate::loom;

/// Save all game state files to disk (JSON only, no git commit).
///
/// Writes character, achievements, haven, enhancement, deep, and loom state.
/// Call [`commit_save`] separately to create a git commit.
#[allow(clippy::too_many_arguments)]
pub fn save_files(
    character_manager: &CharacterManager,
    state: &GameState,
    global_achievements: &achievements::Achievements,
    haven: &haven::Haven,
    enhancement: &enhancement::EnhancementProgress,
    deep: &deep::DeepState,
    loom_state: &loom::LoomState,
) {
    let _ = character_manager.save_character(state);
    achievements::save_achievements(global_achievements).ok();
    if haven.discovered {
        haven::save_haven(haven).ok();
    }
    if enhancement.discovered {
        enhancement::save_enhancement(enhancement).ok();
    }
    if deep.persistent.discovered {
        deep::save_deep(deep).ok();
    }
    if loom_state.persistent.discovered {
        loom::save_loom(loom_state).ok();
    }
}

/// Create a git commit for the current save state.
///
/// Returns `true` if the commit was created successfully.
pub fn commit_save(state: &GameState, save_event: &SaveEvent, history_repo: &HistoryRepo) -> bool {
    let meta = CommitMetadata {
        level: state.character_level,
        prestige: state.prestige_rank,
        zone_id: state.zone_progression.current_zone_id,
        subzone_id: state.zone_progression.current_subzone_id,
        play_time_seconds: state.play_time_seconds,
        character_name: state.character_name.clone(),
    };
    history_repo.commit(save_event, &meta).is_ok()
}

/// Save all game state files and optionally create a git commit.
///
/// Convenience wrapper that calls [`save_files`] then [`commit_save`].
/// Returns `true` if a git commit was successfully created, `false` otherwise.
#[allow(clippy::too_many_arguments)]
pub fn save_all(
    character_manager: &CharacterManager,
    state: &GameState,
    global_achievements: &achievements::Achievements,
    haven: &haven::Haven,
    enhancement: &enhancement::EnhancementProgress,
    deep: &deep::DeepState,
    loom_state: &loom::LoomState,
    save_event: Option<&SaveEvent>,
    history_repo: Option<&HistoryRepo>,
) -> bool {
    save_files(
        character_manager,
        state,
        global_achievements,
        haven,
        enhancement,
        deep,
        loom_state,
    );

    if let (Some(event), Some(repo)) = (save_event, history_repo) {
        commit_save(state, event, repo)
    } else {
        false
    }
}
