//! Debug menu for testing chance-based discoveries.
//!
//! Activated with `--debug` flag. Press backtick to toggle menu.

use crate::challenge_menu::{ChallengeType, PendingChallenge};
use crate::dungeon_generation::generate_dungeon;
use crate::fishing_generation::generate_fishing_session;
use crate::game_state::GameState;

/// Menu options available in debug mode
pub const DEBUG_OPTIONS: &[&str] = &[
    "Trigger Dungeon",
    "Trigger Fishing",
    "Trigger Chess Challenge",
    "Trigger Morris Challenge",
    "Trigger Gomoku Challenge",
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
    state.challenge_menu.add_challenge(PendingChallenge {
        challenge_type: ChallengeType::Chess,
        title: "Chess Challenge".to_string(),
        icon: "♟",
        description: "A hooded figure sits alone at a stone table, chess pieces \
            gleaming in the firelight. \"Care for a game?\" they ask."
            .to_string(),
    });
    "Chess challenge added!"
}

fn trigger_morris_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Morris) {
        return "Morris challenge already pending!";
    }
    state.challenge_menu.add_challenge(PendingChallenge {
        challenge_type: ChallengeType::Morris,
        title: "Nine Men's Morris".to_string(),
        icon: "\u{25CB}",
        description: "An elderly sage arranges nine white stones on a weathered board. \
            \"The game of mills,\" they say. \"Three in a row captures. Shall we play?\""
            .to_string(),
    });
    "Morris challenge added!"
}

fn trigger_gomoku_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Gomoku) {
        return "Gomoku challenge already pending!";
    }
    state.challenge_menu.add_challenge(PendingChallenge {
        challenge_type: ChallengeType::Gomoku,
        title: "Gomoku".to_string(),
        icon: "◎",
        description: "A wandering strategist places a worn board before you. \
            \"Five stones in a row,\" they explain. \"Simple rules, deep tactics.\""
            .to_string(),
    });
    "Gomoku challenge added!"
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
        assert_eq!(menu.selected_index, 4);

        // Can't go past end
        menu.navigate_down();
        assert_eq!(menu.selected_index, 4);

        menu.navigate_up();
        assert_eq!(menu.selected_index, 3);

        // Can't go before start
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
}
