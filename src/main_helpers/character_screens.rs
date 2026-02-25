//! Character management screen handlers extracted from main.rs.
//!
//! Each function handles one frame of a character management screen:
//! drawing the UI, polling for input, and returning a ScreenTransition
//! that tells the caller how to update state.

use crate::character::input::{
    process_creation_input, process_delete_input, process_rename_input, CreationInput,
    CreationResult, DeleteInput, DeleteResult, RenameInput, RenameResult,
};
use crate::character::manager::CharacterManager;
use crate::ui;
use crate::ui::character_creation::CharacterCreationScreen;
use crate::ui::character_delete::CharacterDeleteScreen;
use crate::ui::character_rename::CharacterRenameScreen;
use crate::ui::character_select::CharacterSelectScreen;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

/// Describes the screen transition resulting from a character screen frame.
pub enum ScreenTransition {
    /// Stay on the current screen (no transition).
    Stay,
    /// Return to the splash/select screen.
    GoToSelect,
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
