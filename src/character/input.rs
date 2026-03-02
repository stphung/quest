//! UI-agnostic input handling for character management screens.
//!
//! Each screen has its own sibling module with input enum, result enum, and process function.
//! This module re-exports all types for backward compatibility.

pub use super::creation::{process_creation_input, CreationInput, CreationResult};
pub use super::delete::{process_delete_input, DeleteInput, DeleteResult};
pub use super::rename::{process_rename_input, RenameInput, RenameResult};
pub use super::select::{process_select_input, SelectInput, SelectResult};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::manager::{CharacterInfo, CharacterManager};
    use crate::ui::character_creation::CharacterCreationScreen;
    use crate::ui::character_delete::CharacterDeleteScreen;
    use crate::ui::character_rename::CharacterRenameScreen;
    use crate::ui::character_select::CharacterSelectScreen;

    fn temp_manager() -> (CharacterManager, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let manager =
            CharacterManager::with_dir(dir.path().to_path_buf()).expect("Failed to create manager");
        (manager, dir)
    }

    // =========================================================================
    // CreationInput tests
    // =========================================================================

    #[test]
    fn test_creation_char_input_adds_character() {
        let mut screen = CharacterCreationScreen::new();
        let (manager, _dir) = temp_manager();

        let result = process_creation_input(&mut screen, CreationInput::Char('H'), &manager, false);

        assert_eq!(result, CreationResult::Continue);
        assert_eq!(screen.name_input, "H");
        assert_eq!(screen.cursor_position, 1);
    }

    #[test]
    fn test_creation_multiple_chars() {
        let mut screen = CharacterCreationScreen::new();
        let (manager, _dir) = temp_manager();

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
        let (manager, _dir) = temp_manager();

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
        let (manager, _dir) = temp_manager();

        let result = process_creation_input(&mut screen, CreationInput::Backspace, &manager, false);

        assert_eq!(result, CreationResult::Continue);
        assert_eq!(screen.name_input, "");
        assert_eq!(screen.cursor_position, 0);
    }

    #[test]
    fn test_creation_cancel_with_existing_characters_returns_cancelled() {
        let mut screen = CharacterCreationScreen::new();
        let (manager, _dir) = temp_manager();

        let result = process_creation_input(&mut screen, CreationInput::Cancel, &manager, true);

        assert_eq!(result, CreationResult::Cancelled);
    }

    #[test]
    fn test_creation_cancel_without_existing_characters_quits() {
        let mut screen = CharacterCreationScreen::new();
        let (manager, _dir) = temp_manager();

        let result = process_creation_input(&mut screen, CreationInput::Cancel, &manager, false);

        assert_eq!(result, CreationResult::Quit);
    }

    #[test]
    fn test_creation_submit_with_empty_name_continues() {
        let mut screen = CharacterCreationScreen::new();
        let (manager, _dir) = temp_manager();

        let result = process_creation_input(&mut screen, CreationInput::Submit, &manager, false);

        assert_eq!(result, CreationResult::Continue);
    }

    #[test]
    fn test_creation_other_input_continues() {
        let mut screen = CharacterCreationScreen::new();
        let (manager, _dir) = temp_manager();

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
                ascension_level: 0,
                storm_sigils: crate::stormglass::sigils::StormSigils::new(),
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
                ascension_level: 0,
                storm_sigils: crate::stormglass::sigils::StormSigils::new(),
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
        let characters = create_test_characters();
        // With 2 characters, max index is 2 (the create slot)
        screen.selected_index = 2;

        process_select_input(&mut screen, SelectInput::Down, &characters);

        assert_eq!(screen.selected_index, 2);
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

    // =========================================================================
    // RenameInput tests
    // =========================================================================

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
