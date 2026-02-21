//! Character rename screen input handling.

use crate::character::manager::{CharacterInfo, CharacterManager};
use crate::ui::character_rename::CharacterRenameScreen;

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
    use crate::items::Equipment;

    fn temp_manager() -> (CharacterManager, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let manager =
            CharacterManager::with_dir(dir.path().to_path_buf()).expect("Failed to create manager");
        (manager, dir)
    }

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
            equipment: Equipment::new(),
            is_corrupted: false,
        }
    }

    #[test]
    fn test_rename_char_input_adds_character() {
        let mut screen = CharacterRenameScreen::new();
        let (manager, _dir) = temp_manager();
        let character = create_test_character();

        let result =
            process_rename_input(&mut screen, RenameInput::Char('N'), &manager, &character);

        assert_eq!(result, RenameResult::Continue);
        assert_eq!(screen.new_name_input, "N");
    }

    #[test]
    fn test_rename_backspace_removes_character() {
        let mut screen = CharacterRenameScreen::new();
        let (manager, _dir) = temp_manager();
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
        let (manager, _dir) = temp_manager();
        let character = create_test_character();

        let result = process_rename_input(&mut screen, RenameInput::Submit, &manager, &character);

        assert_eq!(result, RenameResult::Continue);
    }

    #[test]
    fn test_rename_cancel_returns_cancelled() {
        let mut screen = CharacterRenameScreen::new();
        let (manager, _dir) = temp_manager();
        let character = create_test_character();

        let result = process_rename_input(&mut screen, RenameInput::Cancel, &manager, &character);

        assert_eq!(result, RenameResult::Cancelled);
    }

    #[test]
    fn test_rename_other_continues() {
        let mut screen = CharacterRenameScreen::new();
        let (manager, _dir) = temp_manager();
        let character = create_test_character();

        let result = process_rename_input(&mut screen, RenameInput::Other, &manager, &character);

        assert_eq!(result, RenameResult::Continue);
    }

    #[test]
    fn test_rename_submit_with_invalid_name_continues() {
        let mut screen = CharacterRenameScreen::new();
        let (manager, _dir) = temp_manager();
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
