//! Scene classification helpers for terminal redraw management.

use crate::challenges;
use crate::core::game_state::GameState;

/// Returns true if the active minigame requires real-time (high FPS) updates.
pub fn is_realtime_minigame(state: &GameState) -> bool {
    matches!(
        state.active_minigame,
        Some(challenges::ActiveMinigame::FlappyBird(_))
            | Some(challenges::ActiveMinigame::Jezzball(_))
            | Some(challenges::ActiveMinigame::Snake(_))
            | Some(challenges::ActiveMinigame::RunicShift(_))
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SceneKind {
    RunicShift,
    ChallengeMenu,
    Other,
}

pub fn current_scene_kind(state: &GameState) -> SceneKind {
    if matches!(
        state.active_minigame,
        Some(challenges::ActiveMinigame::RunicShift(_))
    ) {
        SceneKind::RunicShift
    } else if state.challenge_menu.is_open {
        SceneKind::ChallengeMenu
    } else {
        SceneKind::Other
    }
}

pub fn is_wide_scene(scene: SceneKind) -> bool {
    matches!(scene, SceneKind::RunicShift | SceneKind::ChallengeMenu)
}
