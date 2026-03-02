//! Game state persistence (save all).

use crate::achievements;
use crate::character::manager::CharacterManager;
use crate::core::game_state::GameState;
use crate::deep;
use crate::enhancement;
use crate::haven;
use crate::history::{CommitMetadata, HistoryRepo, SaveEvent};

/// Save all game state files (character, achievements, haven, enhancement, deep).
///
/// If a `save_event` and `history_repo` are both provided, a git commit is
/// created after the JSON files are written.
///
/// Returns `true` if a git commit was successfully created, `false` otherwise.
#[allow(clippy::too_many_arguments)]
pub fn save_all(
    character_manager: &CharacterManager,
    state: &GameState,
    global_achievements: &achievements::Achievements,
    haven: &haven::Haven,
    enhancement: &enhancement::EnhancementProgress,
    deep: &deep::DeepState,
    save_event: Option<&SaveEvent>,
    history_repo: Option<&HistoryRepo>,
) -> bool {
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
        let meta = CommitMetadata {
            level: state.character_level,
            prestige: state.prestige_rank,
            zone_id: state.zone_progression.current_zone_id,
            subzone_id: state.zone_progression.current_subzone_id,
            play_time_seconds: state.play_time_seconds,
            character_name: state.character_name.clone(),
        };
        repo.commit(event, &meta).is_ok()
    } else {
        false
    }
}
