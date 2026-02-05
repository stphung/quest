//! Debug menu for testing chance-based discoveries.
//!
//! Activated with `--debug` flag. Press backtick to toggle menu.

use crate::challenges::menu::{create_challenge, ChallengeType};
use crate::core::game_state::GameState;
use crate::dungeon::generation::generate_dungeon;
use crate::fishing::generation::generate_fishing_session;

/// Menu options available in debug mode
pub const DEBUG_OPTIONS: &[&str] = &[
    "Trigger Dungeon",
    "Trigger Fishing",
    "Trigger Chess Challenge",
    "Trigger Morris Challenge",
    "Trigger Gomoku Challenge",
    "Trigger Minesweeper Challenge",
    "Trigger Rune Challenge",
];

/// Debug menu state
#[derive(Debug, Clone, Default)]
pub struct DebugMenu {
    pub is_open: bool,
    pub selected_index: usize,
}

impl DebugMenu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn navigate_down(&mut self) {
        if self.selected_index + 1 < DEBUG_OPTIONS.len() {
            self.selected_index += 1;
        }
    }

    /// Trigger the selected debug action. Returns a message describing what happened.
    pub fn trigger_selected(&mut self, state: &mut GameState) -> &'static str {
        let msg = match self.selected_index {
            0 => trigger_dungeon(state),
            1 => trigger_fishing(state),
            2 => trigger_chess_challenge(state),
            3 => trigger_morris_challenge(state),
            4 => trigger_gomoku_challenge(state),
            5 => trigger_minesweeper_challenge(state),
            6 => trigger_rune_challenge(state),
            _ => "Unknown option",
        };
        self.close();
        msg
    }
}

fn trigger_dungeon(state: &mut GameState) -> &'static str {
    if state.active_dungeon.is_some() {
        return "Already in a dungeon!";
    }
    state.active_dungeon = Some(generate_dungeon(state.character_level, state.prestige_rank));
    "Dungeon discovered!"
}

fn trigger_fishing(state: &mut GameState) -> &'static str {
    if state.active_fishing.is_some() {
        return "Already fishing!";
    }
    if state.active_dungeon.is_some() {
        return "Cannot fish while in dungeon!";
    }
    let mut rng = rand::thread_rng();
    state.active_fishing = Some(generate_fishing_session(&mut rng));
    "Fishing spot found!"
}

fn trigger_chess_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Chess) {
        return "Chess challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Chess));
    "Chess challenge added!"
}

fn trigger_morris_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Morris) {
        return "Morris challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Morris));
    "Morris challenge added!"
}

fn trigger_gomoku_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Gomoku) {
        return "Gomoku challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Gomoku));
    "Gomoku challenge added!"
}

fn trigger_rune_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Rune) {
        return "Rune challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Rune));
    "Rune challenge added!"
}

fn trigger_minesweeper_challenge(state: &mut GameState) -> &'static str {
    if state
        .challenge_menu
        .has_challenge(&ChallengeType::Minesweeper)
    {
        return "Minesweeper challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Minesweeper));
    "Minesweeper challenge added!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_navigation() {
        let mut menu = DebugMenu::new();
        menu.open();
        assert_eq!(menu.selected_index, 0);

        menu.navigate_down();
        assert_eq!(menu.selected_index, 1);

        menu.navigate_down();
        menu.navigate_down();
        menu.navigate_down();
        menu.navigate_down();
        menu.navigate_down();
        assert_eq!(menu.selected_index, 6);

        // Can't go past end
        menu.navigate_down();
        assert_eq!(menu.selected_index, 6);

        menu.navigate_up();
        assert_eq!(menu.selected_index, 5);

        // Can't go before start
        menu.navigate_up();
        menu.navigate_up();
        menu.navigate_up();
        menu.navigate_up();
        menu.navigate_up();
        menu.navigate_up();
        assert_eq!(menu.selected_index, 0);
    }

    #[test]
    fn test_toggle() {
        let mut menu = DebugMenu::new();
        assert!(!menu.is_open);

        menu.toggle();
        assert!(menu.is_open);

        menu.toggle();
        assert!(!menu.is_open);
    }

    #[test]
    fn test_trigger_dungeon() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_dungeon(&mut state);
        assert_eq!(msg, "Dungeon discovered!");
        assert!(state.active_dungeon.is_some());

        // Can't trigger again
        let msg = trigger_dungeon(&mut state);
        assert_eq!(msg, "Already in a dungeon!");
    }

    #[test]
    fn test_trigger_fishing() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_fishing(&mut state);
        assert_eq!(msg, "Fishing spot found!");
        assert!(state.active_fishing.is_some());

        // Can't trigger again
        let msg = trigger_fishing(&mut state);
        assert_eq!(msg, "Already fishing!");
    }

    #[test]
    fn test_trigger_chess_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_chess_challenge(&mut state);
        assert_eq!(msg, "Chess challenge added!");
        assert!(state.challenge_menu.has_challenge(&ChallengeType::Chess));

        // Can't add duplicate
        let msg = trigger_chess_challenge(&mut state);
        assert_eq!(msg, "Chess challenge already pending!");
    }

    #[test]
    fn test_trigger_morris_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_morris_challenge(&mut state);
        assert_eq!(msg, "Morris challenge added!");
        assert!(state.challenge_menu.has_challenge(&ChallengeType::Morris));

        // Can't add duplicate
        let msg = trigger_morris_challenge(&mut state);
        assert_eq!(msg, "Morris challenge already pending!");
    }

    #[test]
    fn test_trigger_gomoku_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_gomoku_challenge(&mut state);
        assert_eq!(msg, "Gomoku challenge added!");
        assert!(state.challenge_menu.has_challenge(&ChallengeType::Gomoku));

        // Can't add duplicate
        let msg = trigger_gomoku_challenge(&mut state);
        assert_eq!(msg, "Gomoku challenge already pending!");
    }

    #[test]
    fn test_trigger_rune_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_rune_challenge(&mut state);
        assert_eq!(msg, "Rune challenge added!");
        assert!(state.challenge_menu.has_challenge(&ChallengeType::Rune));

        let msg = trigger_rune_challenge(&mut state);
        assert_eq!(msg, "Rune challenge already pending!");
    }

    #[test]
    fn test_trigger_minesweeper_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_minesweeper_challenge(&mut state);
        assert_eq!(msg, "Minesweeper challenge added!");
        assert!(state
            .challenge_menu
            .has_challenge(&ChallengeType::Minesweeper));

        // Can't add duplicate
        let msg = trigger_minesweeper_challenge(&mut state);
        assert_eq!(msg, "Minesweeper challenge already pending!");
    }
}
