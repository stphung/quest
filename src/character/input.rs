//! UI-agnostic input handling for character management screens.

use crate::ui::character_creation::CharacterCreationScreen;
use crate::ui::character_delete::CharacterDeleteScreen;
use crate::ui::character_rename::CharacterRenameScreen;
use crate::ui::character_select::CharacterSelectScreen;

use super::manager::{CharacterInfo, CharacterManager};

/// Input events for character creation screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreationInput {
    /// Character typed
    Char(char),
    /// Backspace pressed
    Backspace,
    /// Enter pressed to create character
    Submit,
    /// Escape pressed to cancel
    Cancel,
    /// Any other key
    Other,
}

/// Input events for character select screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectInput {
    /// Move selection up
    Up,
    /// Move selection down
    Down,
    /// Load selected character
    Select,
    /// Create new character
    New,
    /// Delete selected character
    Delete,
    /// Rename selected character
    Rename,
    /// Quit the game
    Quit,
    /// Any other key
    Other,
}

/// Input events for character delete confirmation screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteInput {
    /// Character typed
    Char(char),
    /// Backspace pressed
    Backspace,
    /// Enter pressed to confirm deletion
    Submit,
    /// Escape pressed to cancel
    Cancel,
    /// Any other key
    Other,
}

/// Input events for character rename screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameInput {
    /// Character typed
    Char(char),
    /// Backspace pressed
    Backspace,
    /// Enter pressed to confirm rename
    Submit,
    /// Escape pressed to cancel
    Cancel,
    /// Any other key
    Other,
}

/// Result of processing character creation input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreationResult {
    /// Stay on creation screen
    Continue,
    /// Character created successfully, go to select screen
    Created,
    /// Cancelled, go back to select screen (only if characters exist)
    Cancelled,
    /// Save failed with error message
    SaveFailed(String),
}

/// Result of processing character select input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectResult {
    /// Stay on select screen
    Continue,
    /// No characters exist, should go to creation
    NoCharacters,
    /// Load selected character (returns filename)
    LoadCharacter(String),
    /// Go to character creation
    GoToCreation,
    /// Go to character delete screen
    GoToDelete,
    /// Go to character rename screen
    GoToRename,
    /// Quit the game
    Quit,
    /// Load failed with error
    #[allow(dead_code)]
    LoadFailed(String),
}

/// Result of processing character delete input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteResult {
    /// Stay on delete screen
    Continue,
    /// Character deleted successfully
    Deleted,
    /// Cancelled, go back to select screen
    Cancelled,
    /// Delete failed with error
    DeleteFailed(String),
}

/// Result of processing character rename input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameResult {
    /// Stay on rename screen
    Continue,
    /// Character renamed successfully
    Renamed,
    /// Cancelled, go back to select screen
    Cancelled,
    /// Rename failed with error message (sets validation_error)
    RenameFailed(String),
}

/// Process input for the character creation screen.
///
/// Returns the result of the input processing.
pub fn process_creation_input(
    screen: &mut CharacterCreationScreen,
    input: CreationInput,
    manager: &CharacterManager,
    has_existing_characters: bool,
) -> CreationResult {
    match input {
        CreationInput::Char(c) => {
            screen.handle_char_input(c);
            CreationResult::Continue
        }
        CreationInput::Backspace => {
            screen.handle_backspace();
            CreationResult::Continue
        }
        CreationInput::Submit => {
            if screen.is_valid() {
                let new_name = screen.get_name();
                let new_state = crate::core::game_state::GameState::new(
                    new_name,
                    chrono::Utc::now().timestamp(),
                );
                match manager.save_character(&new_state) {
                    Ok(()) => CreationResult::Created,
                    Err(e) => {
                        screen.validation_error = Some(format!("Save failed: {}", e));
                        CreationResult::SaveFailed(format!("Save failed: {}", e))
                    }
                }
            } else {
                CreationResult::Continue
            }
        }
        CreationInput::Cancel => {
            if has_existing_characters {
                CreationResult::Cancelled
            } else {
                CreationResult::Continue
            }
        }
        CreationInput::Other => CreationResult::Continue,
    }
}

/// Process input for the character select screen.
///
/// Returns the result of the input processing.
pub fn process_select_input(
    screen: &mut CharacterSelectScreen,
    input: SelectInput,
    characters: &[CharacterInfo],
) -> SelectResult {
    if characters.is_empty() {
        return SelectResult::NoCharacters;
    }

    // Clamp selected index if needed
    if screen.selected_index >= characters.len() {
        screen.selected_index = characters.len().saturating_sub(1);
    }

    match input {
        SelectInput::Up => {
            screen.move_up(characters);
            SelectResult::Continue
        }
        SelectInput::Down => {
            screen.move_down(characters);
            SelectResult::Continue
        }
        SelectInput::Select => {
            let selected = &characters[screen.selected_index];
            if selected.is_corrupted {
                SelectResult::Continue
            } else {
                SelectResult::LoadCharacter(selected.filename.clone())
            }
        }
        SelectInput::New => SelectResult::GoToCreation,
        SelectInput::Delete => {
            let selected = &characters[screen.selected_index];
            if selected.is_corrupted {
                SelectResult::Continue
            } else {
                SelectResult::GoToDelete
            }
        }
        SelectInput::Rename => {
            let selected = &characters[screen.selected_index];
            if selected.is_corrupted {
                SelectResult::Continue
            } else {
                SelectResult::GoToRename
            }
        }
        SelectInput::Quit => SelectResult::Quit,
        SelectInput::Other => SelectResult::Continue,
    }
}

/// Process input for the character delete confirmation screen.
///
/// Returns the result of the input processing.
pub fn process_delete_input(
    screen: &mut CharacterDeleteScreen,
    input: DeleteInput,
    manager: &CharacterManager,
    character: &CharacterInfo,
) -> DeleteResult {
    match input {
        DeleteInput::Char(c) => {
            screen.handle_char_input(c);
            DeleteResult::Continue
        }
        DeleteInput::Backspace => {
            screen.handle_backspace();
            DeleteResult::Continue
        }
        DeleteInput::Submit => {
            if screen.is_confirmed(&character.character_name) {
                match manager.delete_character(&character.filename) {
                    Ok(()) => DeleteResult::Deleted,
                    Err(e) => DeleteResult::DeleteFailed(format!("Failed to delete: {}", e)),
                }
            } else {
                DeleteResult::Continue
            }
        }
        DeleteInput::Cancel => DeleteResult::Cancelled,
        DeleteInput::Other => DeleteResult::Continue,
    }
}

/// Process input for the character rename screen.
///
/// Returns the result of the input processing.
pub fn process_rename_input(
    screen: &mut CharacterRenameScreen,
    input: RenameInput,
    manager: &CharacterManager,
    character: &CharacterInfo,
) -> RenameResult {
    match input {
        RenameInput::Char(c) => {
            screen.handle_char_input(c);
            RenameResult::Continue
        }
        RenameInput::Backspace => {
            screen.handle_backspace();
            RenameResult::Continue
        }
        RenameInput::Submit => {
            if screen.is_valid() {
                let new_name = screen.get_name();
                match manager.rename_character(&character.filename, new_name) {
                    Ok(()) => RenameResult::Renamed,
                    Err(e) => {
                        screen.validation_error = Some(format!("Rename failed: {}", e));
                        RenameResult::RenameFailed(format!("Rename failed: {}", e))
                    }
                }
            } else {
                RenameResult::Continue
            }
        }
        RenameInput::Cancel => RenameResult::Cancelled,
        RenameInput::Other => RenameResult::Continue,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // CreationInput tests
    // =========================================================================

    #[test]
    fn test_creation_char_input_adds_character() {
        let mut screen = CharacterCreationScreen::new();
        let manager = CharacterManager::new().unwrap();

        let result = process_creation_input(&mut screen, CreationInput::Char('H'), &manager, false);

        assert_eq!(result, CreationResult::Continue);
        assert_eq!(screen.name_input, "H");
        assert_eq!(screen.cursor_position, 1);
    }

    #[test]
    fn test_creation_multiple_chars() {
        let mut screen = CharacterCreationScreen::new();
        let manager = CharacterManager::new().unwrap();

        process_creation_input(&mut screen, CreationInput::Char('H'), &manager, false);
        process_creation_input(&mut screen, CreationInput::Char('e'), &manager, false);
        process_creation_input(&mut screen, CreationInput::Char('r'), &manager, false);
        process_creation_input(&mut screen, CreationInput::Char('o'), &manager, false);

        assert_eq!(screen.name_input, "Hero");
        assert_eq!(screen.cursor_position, 4);
    }

    #[test]
    fn test_creation_backspace_removes_character() {
        let mut screen = CharacterCreationScreen::new();
        let manager = CharacterManager::new().unwrap();

        process_creation_input(&mut screen, CreationInput::Char('A'), &manager, false);
        process_creation_input(&mut screen, CreationInput::Char('B'), &manager, false);
        let result = process_creation_input(&mut screen, CreationInput::Backspace, &manager, false);

        assert_eq!(result, CreationResult::Continue);
        assert_eq!(screen.name_input, "A");
        assert_eq!(screen.cursor_position, 1);
    }

    #[test]
    fn test_creation_backspace_on_empty_does_nothing() {
        let mut screen = CharacterCreationScreen::new();
        let manager = CharacterManager::new().unwrap();

        let result = process_creation_input(&mut screen, CreationInput::Backspace, &manager, false);

        assert_eq!(result, CreationResult::Continue);
        assert_eq!(screen.name_input, "");
        assert_eq!(screen.cursor_position, 0);
    }

    #[test]
    fn test_creation_cancel_with_existing_characters_returns_cancelled() {
        let mut screen = CharacterCreationScreen::new();
        let manager = CharacterManager::new().unwrap();

        let result = process_creation_input(&mut screen, CreationInput::Cancel, &manager, true);

        assert_eq!(result, CreationResult::Cancelled);
    }

    #[test]
    fn test_creation_cancel_without_existing_characters_continues() {
        let mut screen = CharacterCreationScreen::new();
        let manager = CharacterManager::new().unwrap();

        let result = process_creation_input(&mut screen, CreationInput::Cancel, &manager, false);

        assert_eq!(result, CreationResult::Continue);
    }

    #[test]
    fn test_creation_submit_with_empty_name_continues() {
        let mut screen = CharacterCreationScreen::new();
        let manager = CharacterManager::new().unwrap();

        let result = process_creation_input(&mut screen, CreationInput::Submit, &manager, false);

        assert_eq!(result, CreationResult::Continue);
    }

    #[test]
    fn test_creation_other_input_continues() {
        let mut screen = CharacterCreationScreen::new();
        let manager = CharacterManager::new().unwrap();

        let result = process_creation_input(&mut screen, CreationInput::Other, &manager, false);

        assert_eq!(result, CreationResult::Continue);
    }

    // =========================================================================
    // SelectInput tests
    // =========================================================================

    fn create_test_characters() -> Vec<CharacterInfo> {
        vec![
            CharacterInfo {
                character_id: "id1".to_string(),
                character_name: "Hero1".to_string(),
                filename: "hero1.json".to_string(),
                character_level: 10,
                prestige_rank: 1,
                play_time_seconds: 3600,
                last_save_time: 1000,
                attributes: crate::character::attributes::Attributes::new(),
                equipment: crate::items::Equipment::new(),
                is_corrupted: false,
            },
            CharacterInfo {
                character_id: "id2".to_string(),
                character_name: "Hero2".to_string(),
                filename: "hero2.json".to_string(),
                character_level: 20,
                prestige_rank: 2,
                play_time_seconds: 7200,
                last_save_time: 2000,
                attributes: crate::character::attributes::Attributes::new(),
                equipment: crate::items::Equipment::new(),
                is_corrupted: false,
            },
        ]
    }

    #[test]
    fn test_select_no_characters_returns_no_characters() {
        let mut screen = CharacterSelectScreen::new();
        let characters: Vec<CharacterInfo> = vec![];

        let result = process_select_input(&mut screen, SelectInput::Select, &characters);

        assert_eq!(result, SelectResult::NoCharacters);
    }

    #[test]
    fn test_select_up_moves_selection() {
        let mut screen = CharacterSelectScreen::new();
        screen.selected_index = 1;
        let characters = create_test_characters();

        let result = process_select_input(&mut screen, SelectInput::Up, &characters);

        assert_eq!(result, SelectResult::Continue);
        assert_eq!(screen.selected_index, 0);
    }

    #[test]
    fn test_select_down_moves_selection() {
        let mut screen = CharacterSelectScreen::new();
        let characters = create_test_characters();

        let result = process_select_input(&mut screen, SelectInput::Down, &characters);

        assert_eq!(result, SelectResult::Continue);
        assert_eq!(screen.selected_index, 1);
    }

    #[test]
    fn test_select_up_at_top_stays_at_top() {
        let mut screen = CharacterSelectScreen::new();
        screen.selected_index = 0;
        let characters = create_test_characters();

        process_select_input(&mut screen, SelectInput::Up, &characters);

        assert_eq!(screen.selected_index, 0);
    }

    #[test]
    fn test_select_down_at_bottom_stays_at_bottom() {
        let mut screen = CharacterSelectScreen::new();
        screen.selected_index = 1;
        let characters = create_test_characters();

        process_select_input(&mut screen, SelectInput::Down, &characters);

        assert_eq!(screen.selected_index, 1);
    }

    #[test]
    fn test_select_enter_returns_load_character() {
        let mut screen = CharacterSelectScreen::new();
        let characters = create_test_characters();

        let result = process_select_input(&mut screen, SelectInput::Select, &characters);

        assert_eq!(
            result,
            SelectResult::LoadCharacter("hero1.json".to_string())
        );
    }

    #[test]
    fn test_select_enter_on_corrupted_continues() {
        let mut screen = CharacterSelectScreen::new();
        let mut characters = create_test_characters();
        characters[0].is_corrupted = true;

        let result = process_select_input(&mut screen, SelectInput::Select, &characters);

        assert_eq!(result, SelectResult::Continue);
    }

    #[test]
    fn test_select_new_returns_go_to_creation() {
        let mut screen = CharacterSelectScreen::new();
        let characters = create_test_characters();

        let result = process_select_input(&mut screen, SelectInput::New, &characters);

        assert_eq!(result, SelectResult::GoToCreation);
    }

    #[test]
    fn test_select_delete_returns_go_to_delete() {
        let mut screen = CharacterSelectScreen::new();
        let characters = create_test_characters();

        let result = process_select_input(&mut screen, SelectInput::Delete, &characters);

        assert_eq!(result, SelectResult::GoToDelete);
    }

    #[test]
    fn test_select_delete_on_corrupted_continues() {
        let mut screen = CharacterSelectScreen::new();
        let mut characters = create_test_characters();
        characters[0].is_corrupted = true;

        let result = process_select_input(&mut screen, SelectInput::Delete, &characters);

        assert_eq!(result, SelectResult::Continue);
    }

    #[test]
    fn test_select_rename_returns_go_to_rename() {
        let mut screen = CharacterSelectScreen::new();
        let characters = create_test_characters();

        let result = process_select_input(&mut screen, SelectInput::Rename, &characters);

        assert_eq!(result, SelectResult::GoToRename);
    }

    #[test]
    fn test_select_rename_on_corrupted_continues() {
        let mut screen = CharacterSelectScreen::new();
        let mut characters = create_test_characters();
        characters[0].is_corrupted = true;

        let result = process_select_input(&mut screen, SelectInput::Rename, &characters);

        assert_eq!(result, SelectResult::Continue);
    }

    #[test]
    fn test_select_quit_returns_quit() {
        let mut screen = CharacterSelectScreen::new();
        let characters = create_test_characters();

        let result = process_select_input(&mut screen, SelectInput::Quit, &characters);

        assert_eq!(result, SelectResult::Quit);
    }

    #[test]
    fn test_select_other_continues() {
        let mut screen = CharacterSelectScreen::new();
        let characters = create_test_characters();

        let result = process_select_input(&mut screen, SelectInput::Other, &characters);

        assert_eq!(result, SelectResult::Continue);
    }

    #[test]
    fn test_select_clamps_index_when_out_of_bounds() {
        let mut screen = CharacterSelectScreen::new();
        screen.selected_index = 10; // Way out of bounds
        let characters = create_test_characters();

        process_select_input(&mut screen, SelectInput::Other, &characters);

        assert_eq!(screen.selected_index, 1); // Clamped to last valid index
    }

    // =========================================================================
    // DeleteInput tests
    // =========================================================================

    fn create_test_character() -> CharacterInfo {
        CharacterInfo {
            character_id: "id1".to_string(),
            character_name: "TestHero".to_string(),
            filename: "testhero.json".to_string(),
            character_level: 10,
            prestige_rank: 1,
            play_time_seconds: 3600,
            last_save_time: 1000,
            attributes: crate::character::attributes::Attributes::new(),
            equipment: crate::items::Equipment::new(),
            is_corrupted: false,
        }
    }

    #[test]
    fn test_delete_char_input_adds_character() {
        let mut screen = CharacterDeleteScreen::new();
        let manager = CharacterManager::new().unwrap();
        let character = create_test_character();

        let result =
            process_delete_input(&mut screen, DeleteInput::Char('T'), &manager, &character);

        assert_eq!(result, DeleteResult::Continue);
        assert_eq!(screen.confirmation_input, "T");
    }

    #[test]
    fn test_delete_backspace_removes_character() {
        let mut screen = CharacterDeleteScreen::new();
        let manager = CharacterManager::new().unwrap();
        let character = create_test_character();

        process_delete_input(&mut screen, DeleteInput::Char('A'), &manager, &character);
        process_delete_input(&mut screen, DeleteInput::Char('B'), &manager, &character);
        let result =
            process_delete_input(&mut screen, DeleteInput::Backspace, &manager, &character);

        assert_eq!(result, DeleteResult::Continue);
        assert_eq!(screen.confirmation_input, "A");
    }

    #[test]
    fn test_delete_submit_without_match_continues() {
        let mut screen = CharacterDeleteScreen::new();
        let manager = CharacterManager::new().unwrap();
        let character = create_test_character();

        process_delete_input(&mut screen, DeleteInput::Char('W'), &manager, &character);
        process_delete_input(&mut screen, DeleteInput::Char('r'), &manager, &character);
        process_delete_input(&mut screen, DeleteInput::Char('o'), &manager, &character);
        process_delete_input(&mut screen, DeleteInput::Char('n'), &manager, &character);
        process_delete_input(&mut screen, DeleteInput::Char('g'), &manager, &character);

        let result = process_delete_input(&mut screen, DeleteInput::Submit, &manager, &character);

        assert_eq!(result, DeleteResult::Continue);
    }

    #[test]
    fn test_delete_cancel_returns_cancelled() {
        let mut screen = CharacterDeleteScreen::new();
        let manager = CharacterManager::new().unwrap();
        let character = create_test_character();

        let result = process_delete_input(&mut screen, DeleteInput::Cancel, &manager, &character);

        assert_eq!(result, DeleteResult::Cancelled);
    }

    #[test]
    fn test_delete_other_continues() {
        let mut screen = CharacterDeleteScreen::new();
        let manager = CharacterManager::new().unwrap();
        let character = create_test_character();

        let result = process_delete_input(&mut screen, DeleteInput::Other, &manager, &character);

        assert_eq!(result, DeleteResult::Continue);
    }

    // =========================================================================
    // RenameInput tests
    // =========================================================================

    #[test]
    fn test_rename_char_input_adds_character() {
        let mut screen = CharacterRenameScreen::new();
        let manager = CharacterManager::new().unwrap();
        let character = create_test_character();

        let result =
            process_rename_input(&mut screen, RenameInput::Char('N'), &manager, &character);

        assert_eq!(result, RenameResult::Continue);
        assert_eq!(screen.new_name_input, "N");
    }

    #[test]
    fn test_rename_backspace_removes_character() {
        let mut screen = CharacterRenameScreen::new();
        let manager = CharacterManager::new().unwrap();
        let character = create_test_character();

        process_rename_input(&mut screen, RenameInput::Char('A'), &manager, &character);
        process_rename_input(&mut screen, RenameInput::Char('B'), &manager, &character);
        let result =
            process_rename_input(&mut screen, RenameInput::Backspace, &manager, &character);

        assert_eq!(result, RenameResult::Continue);
        assert_eq!(screen.new_name_input, "A");
    }

    #[test]
    fn test_rename_submit_with_empty_name_continues() {
        let mut screen = CharacterRenameScreen::new();
        let manager = CharacterManager::new().unwrap();
        let character = create_test_character();

        let result = process_rename_input(&mut screen, RenameInput::Submit, &manager, &character);

        assert_eq!(result, RenameResult::Continue);
    }

    #[test]
    fn test_rename_cancel_returns_cancelled() {
        let mut screen = CharacterRenameScreen::new();
        let manager = CharacterManager::new().unwrap();
        let character = create_test_character();

        let result = process_rename_input(&mut screen, RenameInput::Cancel, &manager, &character);

        assert_eq!(result, RenameResult::Cancelled);
    }

    #[test]
    fn test_rename_other_continues() {
        let mut screen = CharacterRenameScreen::new();
        let manager = CharacterManager::new().unwrap();
        let character = create_test_character();

        let result = process_rename_input(&mut screen, RenameInput::Other, &manager, &character);

        assert_eq!(result, RenameResult::Continue);
    }

    #[test]
    fn test_rename_submit_with_invalid_name_continues() {
        let mut screen = CharacterRenameScreen::new();
        let manager = CharacterManager::new().unwrap();
        let character = create_test_character();

        // Type an invalid name (special characters)
        for c in "Invalid@Name!".chars() {
            process_rename_input(&mut screen, RenameInput::Char(c), &manager, &character);
        }

        // Screen should have validation error
        assert!(screen.validation_error.is_some());

        // Submit should continue (not rename)
        let result = process_rename_input(&mut screen, RenameInput::Submit, &manager, &character);
        assert_eq!(result, RenameResult::Continue);
    }
}
