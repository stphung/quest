//! Character creation screen input handling.

use crate::character::manager::CharacterManager;
use crate::ui::character_creation::CharacterCreationScreen;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
