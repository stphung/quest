//! Generic challenge menu system for player-controlled minigames.
//!
//! The challenge menu holds pending challenges that players can accept or decline.
//! Chess is the first producer, but future minigames can add their own challenge types.

/// A single pending challenge in the menu
#[derive(Debug, Clone)]
pub struct PendingChallenge {
    pub challenge_type: ChallengeType,
    pub title: String,
    pub icon: &'static str,
    pub description: String,
}

/// Extensible enum for different minigame challenges
#[derive(Debug, Clone)]
pub enum ChallengeType {
    Chess,
}

/// Menu state for navigation
#[derive(Debug, Clone, Default)]
pub struct ChallengeMenu {
    pub challenges: Vec<PendingChallenge>,
    pub is_open: bool,
    pub selected_index: usize,
    pub viewing_detail: bool,
    pub selected_difficulty: usize,
}

impl ChallengeMenu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_chess_challenge(&self) -> bool {
        self.challenges
            .iter()
            .any(|c| matches!(c.challenge_type, ChallengeType::Chess))
    }

    pub fn add_challenge(&mut self, challenge: PendingChallenge) {
        self.challenges.push(challenge);
    }

    pub fn take_selected(&mut self) -> Option<PendingChallenge> {
        if self.challenges.is_empty() {
            return None;
        }
        let challenge = self.challenges.remove(self.selected_index);
        self.selected_index = self
            .selected_index
            .min(self.challenges.len().saturating_sub(1));
        Some(challenge)
    }

    pub fn navigate_up(&mut self) {
        if self.viewing_detail {
            if self.selected_difficulty > 0 {
                self.selected_difficulty -= 1;
            }
        } else if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn navigate_down(&mut self, max_difficulties: usize) {
        if self.viewing_detail {
            if self.selected_difficulty + 1 < max_difficulties {
                self.selected_difficulty += 1;
            }
        } else if self.selected_index + 1 < self.challenges.len() {
            self.selected_index += 1;
        }
    }

    pub fn open_detail(&mut self) {
        if !self.challenges.is_empty() {
            self.viewing_detail = true;
            self.selected_difficulty = 0;
        }
    }

    pub fn close_detail(&mut self) {
        self.viewing_detail = false;
        self.selected_difficulty = 0;
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.selected_index = 0;
        self.viewing_detail = false;
        self.selected_difficulty = 0;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.viewing_detail = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chess_challenge() -> PendingChallenge {
        PendingChallenge {
            challenge_type: ChallengeType::Chess,
            title: "Chess Challenge".to_string(),
            icon: "♟",
            description: "A mysterious figure challenges you to chess.".to_string(),
        }
    }

    #[test]
    fn test_new_menu_is_empty() {
        let menu = ChallengeMenu::new();
        assert!(menu.challenges.is_empty());
        assert!(!menu.is_open);
        assert!(!menu.viewing_detail);
    }

    #[test]
    fn test_add_and_check_challenge() {
        let mut menu = ChallengeMenu::new();
        assert!(!menu.has_chess_challenge());
        menu.add_challenge(make_chess_challenge());
        assert!(menu.has_chess_challenge());
        assert_eq!(menu.challenges.len(), 1);
    }

    #[test]
    fn test_take_selected() {
        let mut menu = ChallengeMenu::new();
        menu.add_challenge(make_chess_challenge());
        menu.add_challenge(make_chess_challenge());
        let taken = menu.take_selected();
        assert!(taken.is_some());
        assert_eq!(menu.challenges.len(), 1);
    }

    #[test]
    fn test_navigation() {
        let mut menu = ChallengeMenu::new();
        menu.add_challenge(make_chess_challenge());
        menu.add_challenge(make_chess_challenge());
        menu.add_challenge(make_chess_challenge());

        assert_eq!(menu.selected_index, 0);
        menu.navigate_down(4);
        assert_eq!(menu.selected_index, 1);
        menu.navigate_down(4);
        assert_eq!(menu.selected_index, 2);
        menu.navigate_down(4);
        assert_eq!(menu.selected_index, 2); // Can't go past end
        menu.navigate_up();
        assert_eq!(menu.selected_index, 1);
    }

    #[test]
    fn test_detail_view_navigation() {
        let mut menu = ChallengeMenu::new();
        menu.add_challenge(make_chess_challenge());
        menu.open_detail();

        assert!(menu.viewing_detail);
        assert_eq!(menu.selected_difficulty, 0);

        menu.navigate_down(4);
        assert_eq!(menu.selected_difficulty, 1);
        menu.navigate_down(4);
        assert_eq!(menu.selected_difficulty, 2);
        menu.navigate_down(4);
        assert_eq!(menu.selected_difficulty, 3);
        menu.navigate_down(4);
        assert_eq!(menu.selected_difficulty, 3); // Can't go past 3
    }

    #[test]
    fn test_open_close() {
        let mut menu = ChallengeMenu::new();
        menu.add_challenge(make_chess_challenge());
        menu.open();
        assert!(menu.is_open);
        menu.open_detail();
        assert!(menu.viewing_detail);
        menu.close();
        assert!(!menu.is_open);
        assert!(!menu.viewing_detail);
    }
}
