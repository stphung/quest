//! Character select screen input handling.

use crate::character::manager::CharacterInfo;
use crate::ui::character_select::CharacterSelectScreen;

/// Input events for character select screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::Equipment;

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
                equipment: Equipment::new(),
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
                equipment: Equipment::new(),
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
}
