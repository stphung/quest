//! Shared game context structs to reduce argument counts on hot-path functions.

use crate::achievements::Achievements;
use crate::core::game_state::GameState;
use crate::deep::types::{DeepState, DeepUiState};
use crate::enhancement::EnhancementProgress;
use crate::haven::Haven;
use crate::input::{GameOverlay, HavenUiState, SoulforgeUiState};
use crate::stormglass::types::{ChronoSurgeState, ChronoSurgeSummary, ExchangeUiState};
use crate::ui::responsive::LayoutContext;
use crate::utils::debug_menu::DebugMenu;
use std::time::Instant;

/// Bundles the mutable state references that are shared across the top-level
/// input and overlay functions. Constructed on the stack before each call and
/// dropped immediately after so individual variables remain accessible.
pub struct GameContext<'a> {
    pub state: &'a mut GameState,
    pub haven: &'a mut Haven,
    pub haven_ui: &'a mut HavenUiState,
    pub soulforge_ui: &'a mut SoulforgeUiState,
    pub exchange_ui: &'a mut ExchangeUiState,
    pub deep_state: &'a mut DeepState,
    pub deep_ui: &'a mut DeepUiState,
    pub enhancement: &'a mut EnhancementProgress,
    pub overlay: &'a mut GameOverlay,
    pub debug_menu: &'a mut DebugMenu,
    pub debug_mode: bool,
    pub achievements: &'a mut Achievements,
}

/// Extra read-only parameters needed by `draw_game_overlays` that are not
/// part of the core mutable game state.
pub struct OverlayExtras<'a> {
    pub last_save_instant: Option<Instant>,
    pub last_save_time: Option<chrono::DateTime<chrono::Local>>,
    pub last_commit_instant: Option<Instant>,
    pub last_commit_time: Option<chrono::DateTime<chrono::Local>>,
    pub last_push_instant: Option<Instant>,
    pub last_push_time: Option<chrono::DateTime<chrono::Local>>,
    pub has_history_repo: bool,
    pub has_cloud_config: bool,
    pub chrono_surge: Option<&'a ChronoSurgeState>,
    pub chrono_summary: Option<&'a ChronoSurgeSummary>,
    pub layout_ctx: &'a LayoutContext,
}
