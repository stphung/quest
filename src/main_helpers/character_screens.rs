//! Character management screen handlers extracted from main.rs.
//!
//! Each function handles one frame of a character management screen:
//! drawing the UI, polling for input, and returning a ScreenTransition
//! that tells the caller how to update state.

use crate::achievements;
use crate::character::input::{
    process_creation_input, process_delete_input, process_rename_input, process_select_input,
    CreationInput, CreationResult, DeleteInput, DeleteResult, RenameInput, RenameResult,
    SelectInput, SelectResult,
};
use crate::character::manager::CharacterManager;
use crate::core::game_state::GameState;
use crate::enhancement;
use crate::haven;
use crate::input::{HavenUiState, SoulforgeUiState};
use crate::ui;
use crate::ui::achievement_browser_scene::AchievementBrowserState;
use crate::ui::title_browser_scene::TitleBrowserState;
use crate::ui::character_creation::CharacterCreationScreen;
use crate::ui::character_delete::CharacterDeleteScreen;
use crate::ui::character_rename::CharacterRenameScreen;
use crate::ui::character_select::CharacterSelectScreen;

use chrono::Utc;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use super::achievements::log_synced_achievements;
use super::offline::apply_offline_xp;

/// Describes the screen transition resulting from a character screen frame.
pub enum ScreenTransition {
    /// Stay on the current screen (no transition).
    Stay,
    /// Go to the character select screen.
    GoToSelect,
    /// Go to the character creation screen.
    GoToCreation,
    /// Go to the character delete screen.
    GoToDelete,
    /// Go to the character rename screen.
    GoToRename,
    /// Load a character and enter the game. Contains the loaded GameState
    /// and an optional offline report.
    LoadCharacter {
        state: Box<GameState>,
        offline_report: Option<crate::core::game_logic::OfflineReport>,
    },
    /// Quit the application.
    Quit,
}

/// Handle one frame of the character creation screen.
///
/// Draws the creation UI and polls for input. Returns a ScreenTransition
/// indicating whether to stay, go to select, etc.
pub fn handle_creation_frame(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    creation_screen: &mut CharacterCreationScreen,
    character_manager: &CharacterManager,
) -> io::Result<ScreenTransition> {
    // Draw character creation screen
    terminal.draw(|f| {
        let area = f.area();
        let ctx = ui::responsive::LayoutContext::from_frame(f);
        creation_screen.draw(f, area, &ctx);
    })?;

    // Handle input
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind != KeyEventKind::Press {
                return Ok(ScreenTransition::Stay);
            }
            let input = match key_event.code {
                KeyCode::Char(c) => CreationInput::Char(c),
                KeyCode::Backspace => CreationInput::Backspace,
                KeyCode::Enter => CreationInput::Submit,
                KeyCode::Esc => CreationInput::Cancel,
                _ => CreationInput::Other,
            };

            let has_existing = !character_manager.list_characters()?.is_empty();
            let result =
                process_creation_input(creation_screen, input, character_manager, has_existing);

            match result {
                CreationResult::Created | CreationResult::Cancelled => {
                    return Ok(ScreenTransition::GoToSelect);
                }
                CreationResult::Continue | CreationResult::SaveFailed(_) => {}
            }
        }
    }

    Ok(ScreenTransition::Stay)
}

/// Handle one frame of the character select screen.
///
/// Draws the select UI (including Haven, achievement browser, Soulforge,
/// and help overlays), polls for input, and returns a ScreenTransition.
#[allow(clippy::too_many_arguments)]
pub fn handle_select_frame(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    select_screen: &mut CharacterSelectScreen,
    character_manager: &CharacterManager,
    haven: &mut haven::Haven,
    haven_ui: &mut HavenUiState,
    soulforge_ui: &mut SoulforgeUiState,
    enhancement: &mut enhancement::EnhancementProgress,
    global_achievements: &mut achievements::Achievements,
    achievement_browser: &mut AchievementBrowserState,
    title_browser: &mut TitleBrowserState,
    help_overlay_showing: &mut bool,
) -> io::Result<ScreenTransition> {
    // Refresh character list
    let characters = character_manager.list_characters()?;

    // Draw character select screen (includes Haven tree visualization)
    terminal.draw(|f| {
        let area = f.area();
        let ctx = ui::responsive::LayoutContext::from_frame(f);
        select_screen.draw(f, area, &characters, haven, enhancement, global_achievements, &ctx);
        // Draw Haven management overlay if open
        if haven_ui.showing {
            ui::haven_scene::render_haven_tree(
                f,
                area,
                haven,
                haven_ui.selected_room,
                haven_ui.open_elapsed_ms(),
                0, // No character selected, so prestige rank = 0
                global_achievements,
                &ctx,
            );
        }
        // Draw achievement browser overlay if open
        if achievement_browser.showing {
            if title_browser.showing {
                ui::title_browser_scene::render_title_browser(
                    f,
                    area,
                    global_achievements,
                    title_browser,
                    "", // No character selected on this screen
                );
            } else {
                ui::achievement_browser_scene::render_achievement_browser(
                    f,
                    area,
                    global_achievements,
                    achievement_browser,
                    enhancement,
                    &ctx,
                );
            }
        }
        // Draw Soulforge overlay if open
        if soulforge_ui.open {
            ui::soulforge_scene::render_soulforge(f, area, soulforge_ui, enhancement, 0, &ctx);
        }
        // Draw Help overlay if open
        if *help_overlay_showing {
            ui::help_overlay::draw_help_overlay(f);
        }
    })?;

    // Handle input
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind != KeyEventKind::Press {
                return Ok(ScreenTransition::Stay);
            }
            // Handle Soulforge overlay (blocks other input when open)
            if soulforge_ui.open {
                match key_event.code {
                    KeyCode::Up => {
                        if soulforge_ui.selected_slot > 0 {
                            soulforge_ui.selected_slot -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if soulforge_ui.selected_slot < 6 {
                            soulforge_ui.selected_slot += 1;
                        }
                    }
                    KeyCode::Esc => soulforge_ui.close(),
                    _ => {}
                }
                return Ok(ScreenTransition::Stay);
            }

            // Handle Help overlay (blocks other input when open)
            if *help_overlay_showing {
                if matches!(key_event.code, KeyCode::Esc | KeyCode::Char('?')) {
                    *help_overlay_showing = false;
                }
                return Ok(ScreenTransition::Stay);
            }

            // Handle achievement browser (blocks other input when open)
            if achievement_browser.showing {
                // Title browser takes priority when open
                if title_browser.showing {
                    let unlocked =
                        crate::achievements::titles::get_unlocked_titles(global_achievements);
                    match key_event.code {
                        KeyCode::Esc => title_browser.close(),
                        KeyCode::Up => title_browser.move_up(),
                        KeyCode::Down => title_browser.move_down(unlocked.len()),
                        KeyCode::Enter => {
                            if let Some(title_def) = unlocked.get(title_browser.selected_index) {
                                global_achievements.selected_title =
                                    Some(title_def.achievement_id);
                                title_browser.close();
                            }
                        }
                        KeyCode::Backspace => {
                            global_achievements.selected_title = None;
                            title_browser.close();
                        }
                        _ => {}
                    }
                    return Ok(ScreenTransition::Stay);
                }

                let category_achievements = achievements::get_achievements_by_category(
                    achievement_browser.selected_category,
                );
                match key_event.code {
                    KeyCode::Up => achievement_browser.move_up(),
                    KeyCode::Down => achievement_browser.move_down(category_achievements.len()),
                    KeyCode::Left | KeyCode::Char(',') | KeyCode::Char('<') => {
                        achievement_browser.prev_category()
                    }
                    KeyCode::Right | KeyCode::Char('.') | KeyCode::Char('>') => {
                        achievement_browser.next_category()
                    }
                    KeyCode::Esc | KeyCode::Char('a') | KeyCode::Char('A') => {
                        global_achievements.clear_recently_unlocked();
                        achievement_browser.close();
                    }
                    KeyCode::Char('t') | KeyCode::Char('T') => {
                        title_browser.open();
                    }
                    _ => {}
                }
                return Ok(ScreenTransition::Stay);
            }

            // Handle achievement browser shortcut
            if matches!(key_event.code, KeyCode::Char('a') | KeyCode::Char('A')) {
                global_achievements.clear_pending_notifications();
                achievement_browser.open();
                return Ok(ScreenTransition::Stay);
            }

            // Help overlay shortcut
            if key_event.code == KeyCode::Char('?') {
                *help_overlay_showing = true;
                return Ok(ScreenTransition::Stay);
            }

            let input = match key_event.code {
                KeyCode::Up => SelectInput::Up,
                KeyCode::Down => SelectInput::Down,
                KeyCode::Enter => SelectInput::Select,
                KeyCode::Char('n') | KeyCode::Char('N') => SelectInput::New,
                KeyCode::Char('d') | KeyCode::Char('D') => SelectInput::Delete,
                KeyCode::Char('r') | KeyCode::Char('R') => SelectInput::Rename,
                KeyCode::Esc => SelectInput::Quit,
                _ => SelectInput::Other,
            };

            let result = process_select_input(select_screen, input, &characters);

            match result {
                SelectResult::NoCharacters => {
                    return Ok(ScreenTransition::GoToCreation);
                }
                SelectResult::LoadCharacter(filename) => {
                    match character_manager.load_character(&filename) {
                        Ok(mut state) => {
                            // Initialize cached derived stats and prestige bonuses
                            state.recalculate_derived_stats(&enhancement.levels);
                            state.recalculate_prestige_bonuses();

                            // Sanity check: clear stale enemy if HP is impossibly high
                            let derived = state.cached_derived_stats;
                            if let Some(enemy) = &state.combat_state.current_enemy {
                                if enemy.max_hp > (derived.max_hp as f64 * 2.5) as u32 {
                                    state.combat_state.current_enemy = None;
                                }
                            }

                            // Sync achievements from character state (retroactive unlocks)
                            let defeated_bosses = state.zone_progression.defeated_bosses.to_vec();
                            global_achievements.sync_from_game_state(
                                state.character_level,
                                state.prestige_rank,
                                state.fishing.rank,
                                state.fishing.total_fish_caught,
                                &defeated_bosses,
                                Some(&state.character_name),
                            );
                            global_achievements.sync_from_haven(
                                haven.discovered,
                                &haven.rooms,
                                Some(&state.character_name),
                            );

                            // Retroactive enhancement/soulforge achievement sync
                            if enhancement.discovered {
                                global_achievements
                                    .on_soulforge_discovered(Some(&state.character_name));
                            }
                            global_achievements.on_enhancement_upgraded(
                                enhancement.highest_level_reached,
                                &enhancement.levels,
                                enhancement.total_attempts,
                                Some(&state.character_name),
                            );

                            log_synced_achievements(&mut state, global_achievements);

                            // Process offline progression
                            let current_time = Utc::now().timestamp();
                            let elapsed_seconds = current_time - state.last_save_time;

                            let offline_report = if elapsed_seconds > 60 {
                                apply_offline_xp(&mut state, haven)
                            } else {
                                None
                            };
                            // Always sync last_save_time on load so suspension
                            // detection doesn't false-trigger from a stale value
                            state.last_save_time = Utc::now().timestamp();

                            return Ok(ScreenTransition::LoadCharacter {
                                state: Box::new(state),
                                offline_report,
                            });
                        }
                        Err(e) => {
                            eprintln!("Failed to load character: {}", e);
                        }
                    }
                }
                SelectResult::GoToCreation => {
                    return Ok(ScreenTransition::GoToCreation);
                }
                SelectResult::GoToDelete => {
                    return Ok(ScreenTransition::GoToDelete);
                }
                SelectResult::GoToRename => {
                    return Ok(ScreenTransition::GoToRename);
                }
                SelectResult::Quit => {
                    return Ok(ScreenTransition::Quit);
                }
                SelectResult::Continue | SelectResult::LoadFailed(_) => {}
            }
        }
    }

    Ok(ScreenTransition::Stay)
}

/// Handle one frame of the character delete confirmation screen.
///
/// Draws the delete UI, polls for input, and returns a ScreenTransition.
pub fn handle_delete_frame(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    delete_screen: &mut CharacterDeleteScreen,
    select_screen: &CharacterSelectScreen,
    character_manager: &CharacterManager,
) -> io::Result<ScreenTransition> {
    // Get current character list and selected character
    let characters = character_manager.list_characters()?;
    if characters.is_empty() || select_screen.selected_index >= characters.len() {
        return Ok(ScreenTransition::GoToSelect);
    }
    let selected_character = &characters[select_screen.selected_index];

    // Draw delete confirmation screen
    terminal.draw(|f| {
        let area = f.area();
        let ctx = ui::responsive::LayoutContext::from_frame(f);
        delete_screen.draw(f, area, selected_character, &ctx);
    })?;

    // Handle input
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind != KeyEventKind::Press {
                return Ok(ScreenTransition::Stay);
            }
            let input = match key_event.code {
                KeyCode::Char(c) => DeleteInput::Char(c),
                KeyCode::Backspace => DeleteInput::Backspace,
                KeyCode::Enter => DeleteInput::Submit,
                KeyCode::Esc => DeleteInput::Cancel,
                _ => DeleteInput::Other,
            };

            let result =
                process_delete_input(delete_screen, input, character_manager, selected_character);

            match result {
                DeleteResult::Deleted | DeleteResult::Cancelled => {
                    return Ok(ScreenTransition::GoToSelect);
                }
                DeleteResult::DeleteFailed(e) => {
                    eprintln!("Failed to delete character: {}", e);
                }
                DeleteResult::Continue => {}
            }
        }
    }

    Ok(ScreenTransition::Stay)
}

/// Handle one frame of the character rename screen.
///
/// Draws the rename UI, polls for input, and returns a ScreenTransition.
pub fn handle_rename_frame(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rename_screen: &mut CharacterRenameScreen,
    select_screen: &CharacterSelectScreen,
    character_manager: &CharacterManager,
) -> io::Result<ScreenTransition> {
    // Get current character list and selected character
    let characters = character_manager.list_characters()?;
    if characters.is_empty() || select_screen.selected_index >= characters.len() {
        return Ok(ScreenTransition::GoToSelect);
    }
    let selected_character = &characters[select_screen.selected_index];

    // Draw rename screen
    terminal.draw(|f| {
        let area = f.area();
        let ctx = ui::responsive::LayoutContext::from_frame(f);
        rename_screen.draw(f, area, selected_character, &ctx);
    })?;

    // Handle input
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind != KeyEventKind::Press {
                return Ok(ScreenTransition::Stay);
            }
            let input = match key_event.code {
                KeyCode::Char(c) => RenameInput::Char(c),
                KeyCode::Backspace => RenameInput::Backspace,
                KeyCode::Enter => RenameInput::Submit,
                KeyCode::Esc => RenameInput::Cancel,
                _ => RenameInput::Other,
            };

            let result =
                process_rename_input(rename_screen, input, character_manager, selected_character);

            match result {
                RenameResult::Renamed | RenameResult::Cancelled => {
                    return Ok(ScreenTransition::GoToSelect);
                }
                RenameResult::RenameFailed(_) | RenameResult::Continue => {}
            }
        }
    }

    Ok(ScreenTransition::Stay)
}
