//! Game state persistence (save all).

use crate::achievements;
use crate::character::manager::CharacterManager;
use crate::core::game_state::GameState;
use crate::deep;
use crate::enhancement;
use crate::haven;
use crate::history::{HistoryRepo, SaveEvent};

/// Save all game state files (character, achievements, haven, enhancement, deep).
///
/// If a `save_event` and `history_repo` are both provided, a git commit is
/// created after the JSON files are written.
pub fn save_all(
    character_manager: &CharacterManager,
    state: &GameState,
    global_achievements: &achievements::Achievements,
    haven: &haven::Haven,
    enhancement: &enhancement::EnhancementProgress,
    deep: &deep::DeepState,
    save_event: Option<&SaveEvent>,
    history_repo: Option<&HistoryRepo>,
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

    if let (Some(event), Some(repo)) = (save_event, history_repo) {
        let _ = repo.commit(
            event,
            state.character_level,
            state.prestige_rank,
            state.zone_progression.current_zone_id,
            state.zone_progression.current_subzone_id,
            state.play_time_seconds,
            &state.character_name,
        );
    }
}
