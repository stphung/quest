//! Character delete confirmation screen input handling.

use crate::character::manager::{CharacterInfo, CharacterManager};
use crate::ui::character_delete::CharacterDeleteScreen;

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
            ascension_level: 0,
            storm_sigils: crate::stormglass::sigils::StormSigils::new(),
            is_corrupted: false,
        }
    }

    #[test]
    fn test_delete_char_input_adds_character() {
        let mut screen = CharacterDeleteScreen::new();
        let (manager, _dir) = temp_manager();
        let character = create_test_character();

        let result =
            process_delete_input(&mut screen, DeleteInput::Char('T'), &manager, &character);

        assert_eq!(result, DeleteResult::Continue);
        assert_eq!(screen.confirmation_input, "T");
    }

    #[test]
    fn test_delete_backspace_removes_character() {
        let mut screen = CharacterDeleteScreen::new();
        let (manager, _dir) = temp_manager();
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
        let (manager, _dir) = temp_manager();
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
        let (manager, _dir) = temp_manager();
        let character = create_test_character();

        let result = process_delete_input(&mut screen, DeleteInput::Cancel, &manager, &character);

        assert_eq!(result, DeleteResult::Cancelled);
    }

    #[test]
    fn test_delete_other_continues() {
        let mut screen = CharacterDeleteScreen::new();
        let (manager, _dir) = temp_manager();
        let character = create_test_character();

        let result = process_delete_input(&mut screen, DeleteInput::Other, &manager, &character);

        assert_eq!(result, DeleteResult::Continue);
    }

    #[test]
    fn test_delete_submit_confirmed_deletes_character_file() {
        let mut screen = CharacterDeleteScreen::new();
        let (manager, _dir) = temp_manager();
        let character = create_test_character(); // name "TestHero", filename "testhero.json"

        let state = crate::core::game_state::GameState::new(character.character_name.clone(), 0);
        manager.save_character(&state).unwrap();
        assert!(manager.quest_dir.join(&character.filename).exists());

        for c in character.character_name.chars() {
            process_delete_input(&mut screen, DeleteInput::Char(c), &manager, &character);
        }

        let result = process_delete_input(&mut screen, DeleteInput::Submit, &manager, &character);

        assert_eq!(result, DeleteResult::Deleted);
        assert!(!manager.quest_dir.join(&character.filename).exists());
    }

    #[test]
    fn test_delete_submit_confirmed_but_file_missing_returns_delete_failed() {
        let mut screen = CharacterDeleteScreen::new();
        let (manager, _dir) = temp_manager();
        let character = create_test_character(); // file never saved to disk

        for c in character.character_name.chars() {
            process_delete_input(&mut screen, DeleteInput::Char(c), &manager, &character);
        }

        let result = process_delete_input(&mut screen, DeleteInput::Submit, &manager, &character);

        match result {
            DeleteResult::DeleteFailed(msg) => assert!(msg.contains("Failed to delete")),
            other => panic!("expected DeleteFailed, got {:?}", other),
        }
    }
}
