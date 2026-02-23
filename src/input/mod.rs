//! Input handling for the Game screen.
//!
//! Extracts the input dispatch logic from main.rs into a clean priority chain.

mod haven_input;
mod minigame_input;
mod prestige_input;
mod soulforge_input;
mod stormglass_input;
pub mod types;

// Re-export all types for backward compatibility
pub use types::*;

// Re-export soulforge UI types from enhancement module
pub use crate::enhancement::{SoulforgePhase, SoulforgeUiState};

use crate::achievements::get_achievements_by_category;
use crate::challenges::menu::{process_input as process_menu_input, MenuInput};
use crate::character::prestige::can_prestige;
use crate::core::game_state::GameState;
use crate::enhancement;
use crate::haven::Haven;
use crate::stormglass::types::ExchangeUiState;
use crate::utils::debug_menu::DebugMenu;
use haven_input::handle_haven;
use minigame_input::handle_minigame;
use prestige_input::{handle_prestige_confirm, handle_vault_selection};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use soulforge_input::handle_soulforge;
pub use stormglass_input::check_sigil_animation_timeout;
use stormglass_input::handle_stormglass_exchange;

/// Main dispatcher for Game screen input. Handles the priority chain.
#[allow(clippy::too_many_arguments)]
pub fn handle_game_input(
    key: KeyEvent,
    state: &mut GameState,
    haven: &mut Haven,
    haven_ui: &mut HavenUiState,
    soulforge_ui: &mut SoulforgeUiState,
    exchange_ui: &mut ExchangeUiState,
    enhancement: &mut enhancement::EnhancementProgress,
    overlay: &mut GameOverlay,
    debug_menu: &mut DebugMenu,
    debug_mode: bool,
    achievements: &mut crate::achievements::Achievements,
    update_available: bool,
    update_expanded: bool,
) -> InputResult {
    // 0. Offline welcome overlay (any key dismisses)
    if matches!(overlay, GameOverlay::OfflineWelcome { .. }) {
        *overlay = GameOverlay::None;
        return InputResult::Continue;
    }

    // 0.25. Storm Leviathan encounter modal (Enter dismisses)
    if matches!(overlay, GameOverlay::LeviathanEncounter { .. }) {
        if matches!(key.code, KeyCode::Enter) {
            *overlay = GameOverlay::None;
        }
        return InputResult::Continue;
    }

    // 0.5. Achievement browser overlay
    if let GameOverlay::Achievements {
        ref mut browser,
        ref mut title_browser,
    } = overlay
    {
        // Title browser takes priority when open
        if title_browser.showing {
            let unlocked = crate::achievements::titles::get_unlocked_titles(achievements);
            match key.code {
                KeyCode::Esc => title_browser.close(),
                KeyCode::Up => title_browser.move_up(),
                KeyCode::Down => title_browser.move_down(unlocked.len()),
                KeyCode::Enter => {
                    if let Some(title_def) = unlocked.get(title_browser.selected_index) {
                        achievements.selected_title = Some(title_def.achievement_id);
                        title_browser.close();
                        return InputResult::NeedsSave;
                    }
                }
                KeyCode::Backspace => {
                    achievements.selected_title = None;
                    title_browser.close();
                    return InputResult::NeedsSave;
                }
                _ => {}
            }
            return InputResult::Continue;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('a') | KeyCode::Char('A') => {
                achievements.clear_recently_unlocked();
                *overlay = GameOverlay::None;
            }
            KeyCode::Left => browser.prev_category(),
            KeyCode::Right => browser.next_category(),
            KeyCode::Up => browser.move_up(),
            KeyCode::Down => {
                let count = get_achievements_by_category(browser.selected_category).len();
                browser.move_down(count);
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                title_browser.open();
            }
            _ => {}
        }
        return InputResult::Continue;
    }

    // 0.75. Help overlay
    if matches!(overlay, GameOverlay::Help) {
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('?')) {
            *overlay = GameOverlay::None;
        }
        return InputResult::Continue;
    }

    // 0.8. Bug report overlay
    if matches!(overlay, GameOverlay::BugReport { .. }) {
        return handle_bug_report_overlay(key, overlay);
    }

    // 1. Haven discovery modal (blocks all other input)
    if matches!(overlay, GameOverlay::HavenDiscovery) {
        return handle_haven_discovery(key, overlay);
    }

    // 1a. Soulforge discovery modal (blocks all other input)
    if matches!(overlay, GameOverlay::SoulforgeDiscovery) {
        return handle_soulforge_discovery(key, overlay);
    }

    // 1b. Stormglass discovery modal (blocks all other input)
    if matches!(overlay, GameOverlay::StormglassDiscovery) {
        return handle_stormglass_discovery(key, overlay);
    }

    // 1c. Achievement unlocked modal (blocks all other input)
    if matches!(overlay, GameOverlay::AchievementUnlocked { .. }) {
        return handle_achievement_unlocked(key, overlay);
    }

    // 2. Haven screen (blocks other input when open)
    if haven_ui.showing {
        return handle_haven(key, state, haven, haven_ui, achievements);
    }

    // 2.5. Soulforge overlay
    if soulforge_ui.open {
        return handle_soulforge(key, soulforge_ui, enhancement, state.prestige_rank);
    }

    // 2.7. Stormglass Exchange overlay
    if exchange_ui.open {
        return handle_stormglass_exchange(key, exchange_ui, state);
    }

    // 3. Vault item selection
    if matches!(overlay, GameOverlay::VaultSelection { .. }) {
        return handle_vault_selection(key, state, haven, overlay);
    }

    // 4. Prestige confirmation
    if matches!(overlay, GameOverlay::PrestigeConfirm) {
        return handle_prestige_confirm(key, state, haven, overlay);
    }

    // 4.5. Quit confirmation (pending challenges warning)
    if matches!(overlay, GameOverlay::QuitConfirm) {
        match key.code {
            KeyCode::Enter => return InputResult::QuitToSelect,
            _ => {
                *overlay = GameOverlay::None;
                return InputResult::Continue;
            }
        }
    }

    // 5. Debug menu
    if debug_mode {
        if key.code == KeyCode::Char('`') {
            debug_menu.toggle();
            return InputResult::Continue;
        }
        if debug_menu.is_open {
            return handle_debug_menu(
                key,
                state,
                haven,
                enhancement,
                achievements,
                overlay,
                debug_menu,
            );
        }
    }

    // 6. Active minigame
    if state.active_minigame.is_some() {
        return handle_minigame(key, state);
    }

    // 7. Challenge menu
    if state.challenge_menu.is_open {
        return handle_challenge_menu(key, state);
    }

    // 8. Tab to open challenge menu
    if key.code == KeyCode::Tab && !state.challenge_menu.challenges.is_empty() {
        state.challenge_menu.open();
        return InputResult::Continue;
    }

    // 9. Base game input
    handle_base_game(
        key,
        state,
        haven,
        haven_ui,
        soulforge_ui,
        exchange_ui,
        enhancement,
        overlay,
        achievements,
        update_available,
        update_expanded,
    )
}

fn handle_haven_discovery(key: KeyEvent, overlay: &mut GameOverlay) -> InputResult {
    if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
        *overlay = GameOverlay::None;
    }
    InputResult::Continue
}

fn handle_soulforge_discovery(key: KeyEvent, overlay: &mut GameOverlay) -> InputResult {
    if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
        *overlay = GameOverlay::None;
    }
    InputResult::Continue
}

fn handle_stormglass_discovery(key: KeyEvent, overlay: &mut GameOverlay) -> InputResult {
    if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
        *overlay = GameOverlay::None;
    }
    InputResult::Continue
}

fn handle_achievement_unlocked(key: KeyEvent, overlay: &mut GameOverlay) -> InputResult {
    // Any key dismisses the achievement modal
    if matches!(key.code, KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ')) {
        *overlay = GameOverlay::None;
    }
    InputResult::Continue
}

fn handle_bug_report_overlay(key: KeyEvent, overlay: &mut GameOverlay) -> InputResult {
    match key.code {
        KeyCode::Enter => {
            if let GameOverlay::BugReport { ref summary, .. } = overlay {
                let url = crate::utils::bug_report::build_issue_url(summary);
                if let Err(e) = crate::utils::bug_report::open_browser(&url) {
                    *overlay = GameOverlay::BugReport {
                        summary: String::new(),
                        clipboard_ready: false,
                        error: Some(format!("Failed to open browser: {e}")),
                    };
                } else {
                    *overlay = GameOverlay::None;
                }
            }
            InputResult::Continue
        }
        KeyCode::Esc => {
            *overlay = GameOverlay::None;
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

fn handle_debug_menu(
    key: KeyEvent,
    state: &mut GameState,
    haven: &mut Haven,
    enhancement: &mut enhancement::EnhancementProgress,
    achievements: &mut crate::achievements::Achievements,
    overlay: &mut GameOverlay,
    debug_menu: &mut DebugMenu,
) -> InputResult {
    match key.code {
        KeyCode::Tab | KeyCode::Right => debug_menu.navigate_next_category(),
        KeyCode::BackTab | KeyCode::Left => debug_menu.navigate_prev_category(),
        KeyCode::Up => debug_menu.navigate_up(),
        KeyCode::Down => debug_menu.navigate_down(),
        KeyCode::Enter => {
            let msg = debug_menu.trigger_selected(state, haven, enhancement, achievements);
            state
                .combat_state
                .add_log_entry(format!("[DEBUG] {}", msg), false, true);
            // Show discovery modals (no save in debug mode)
            if msg == "Haven discovered!" {
                *overlay = GameOverlay::HavenDiscovery;
            } else if msg == "Soulforge discovered!" {
                *overlay = GameOverlay::SoulforgeDiscovery;
            } else if msg == "Stormglass discovered!" {
                *overlay = GameOverlay::StormglassDiscovery;
            }
        }
        KeyCode::Esc => debug_menu.close(),
        _ => {}
    }
    InputResult::Continue
}

fn handle_challenge_menu(key: KeyEvent, state: &mut GameState) -> InputResult {
    let input = match key.code {
        KeyCode::Up => MenuInput::Up,
        KeyCode::Down => MenuInput::Down,
        KeyCode::Enter => MenuInput::Select,
        KeyCode::Char('d') | KeyCode::Char('D') => MenuInput::Decline,
        KeyCode::Esc | KeyCode::Tab => MenuInput::Cancel,
        _ => MenuInput::Other,
    };
    process_menu_input(state, input);
    InputResult::Continue
}

#[allow(clippy::too_many_arguments)]
fn handle_base_game(
    key: KeyEvent,
    state: &mut GameState,
    haven: &Haven,
    haven_ui: &mut HavenUiState,
    soulforge_ui: &mut SoulforgeUiState,
    exchange_ui: &mut ExchangeUiState,
    enhancement: &enhancement::EnhancementProgress,
    overlay: &mut GameOverlay,
    achievements: &mut crate::achievements::Achievements,
    update_available: bool,
    update_expanded: bool,
) -> InputResult {
    match key.code {
        KeyCode::Esc => {
            if state.challenge_menu.challenges.is_empty() {
                InputResult::QuitToSelect
            } else {
                *overlay = GameOverlay::QuitConfirm;
                InputResult::Continue
            }
        }
        KeyCode::Char('u') | KeyCode::Char('U') => {
            // Toggle update details if update available OR already expanded
            if update_available || update_expanded {
                InputResult::ToggleUpdateDetails
            } else {
                InputResult::Continue
            }
        }
        KeyCode::Char('p') | KeyCode::Char('P') => {
            if can_prestige(state) {
                *overlay = GameOverlay::PrestigeConfirm;
            }
            InputResult::Continue
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            if haven.discovered {
                haven_ui.open();
            }
            InputResult::Continue
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if enhancement.discovered {
                soulforge_ui.open();
            }
            InputResult::Continue
        }
        KeyCode::Char('g') | KeyCode::Char('G') => {
            if state.stormglass_discovered {
                exchange_ui.open();
            }
            InputResult::Continue
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            // Clear pending notifications when opening achievements
            achievements.clear_pending_notifications();
            *overlay = GameOverlay::Achievements {
                browser: crate::ui::achievement_browser_scene::AchievementBrowserState::new(),
                title_browser: crate::ui::title_browser_scene::TitleBrowserState::new(),
            };
            InputResult::Continue
        }
        KeyCode::Char('?') => {
            *overlay = GameOverlay::Help;
            InputResult::Continue
        }
        KeyCode::Char('!') => {
            let report = crate::utils::bug_report::generate_bug_report(
                state,
                haven,
                enhancement,
                achievements,
            );
            let clipboard_err =
                crate::utils::bug_report::copy_to_clipboard(&report.clipboard_content).err();
            *overlay = GameOverlay::BugReport {
                summary: report.summary,
                clipboard_ready: clipboard_err.is_none(),
                error: clipboard_err,
            };
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}
