//! Generic challenge menu system for player-controlled minigames.
//!
//! The challenge menu holds pending challenges that players can accept or decline.
//! Challenge discovery uses a single roll per tick. On success, a weighted distribution
//! table determines which challenge type appears.

use crate::game_state::GameState;
use rand::Rng;

/// Chance per tick to discover any challenge (~2 hour average)
/// At 10 ticks/sec, 0.000014 chance/tick ≈ 71,429 ticks ≈ 2 hours average
pub const CHALLENGE_DISCOVERY_CHANCE: f64 = 0.000014;

/// Entry in the challenge distribution table
struct ChallengeWeight {
    challenge_type: ChallengeType,
    weight: u32,
}

/// Weighted distribution table for challenge types.
/// Higher weight = more likely to appear when a challenge is discovered.
const CHALLENGE_TABLE: &[ChallengeWeight] = &[
    ChallengeWeight {
        challenge_type: ChallengeType::Chess,
        weight: 33,
    },
    ChallengeWeight {
        challenge_type: ChallengeType::Morris,
        weight: 33,
    },
    ChallengeWeight {
        challenge_type: ChallengeType::Gomoku,
        weight: 34,
    },
];

/// A single pending challenge in the menu
#[derive(Debug, Clone)]
pub struct PendingChallenge {
    pub challenge_type: ChallengeType,
    pub title: String,
    pub icon: &'static str,
    pub description: String,
}

/// Extensible enum for different minigame challenges
#[derive(Debug, Clone, PartialEq)]
pub enum ChallengeType {
    Chess,
    Morris,
    Gomoku,
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

    pub fn has_challenge(&self, ct: &ChallengeType) -> bool {
        self.challenges.iter().any(|c| c.challenge_type == *ct)
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

/// Check if challenge discovery conditions are met, roll once, and pick from weighted table.
/// Returns the discovered ChallengeType if one was added to the menu, or None.
pub fn try_discover_challenge<R: Rng>(state: &mut GameState, rng: &mut R) -> Option<ChallengeType> {
    // Requirements: P1+, not in dungeon, not fishing, not in active minigame
    if state.prestige_rank < 1
        || state.active_dungeon.is_some()
        || state.active_fishing.is_some()
        || state.active_chess.is_some()
        || state.active_morris.is_some()
        || state.active_gomoku.is_some()
    {
        return None;
    }

    // Single roll for any challenge
    if rng.gen::<f64>() >= CHALLENGE_DISCOVERY_CHANCE {
        return None;
    }

    // Build eligible entries: exclude types already pending in the menu
    let eligible: Vec<&ChallengeWeight> = CHALLENGE_TABLE
        .iter()
        .filter(|entry| !state.challenge_menu.has_challenge(&entry.challenge_type))
        .collect();

    if eligible.is_empty() {
        return None;
    }

    let total_weight: u32 = eligible.iter().map(|e| e.weight).sum();
    let mut roll = rng.gen_range(0..total_weight);

    for entry in &eligible {
        if roll < entry.weight {
            let challenge = create_challenge(&entry.challenge_type);
            let ct = entry.challenge_type.clone();
            state.challenge_menu.add_challenge(challenge);
            return Some(ct);
        }
        roll -= entry.weight;
    }

    None
}

/// Create a PendingChallenge from a ChallengeType
fn create_challenge(ct: &ChallengeType) -> PendingChallenge {
    match ct {
        ChallengeType::Chess => PendingChallenge {
            challenge_type: ChallengeType::Chess,
            title: "Chess Challenge".to_string(),
            icon: "♟",
            description: "A hooded figure sits alone at a stone table, chess pieces \
                gleaming in the firelight. \"Care for a game?\" they ask."
                .to_string(),
        },
        ChallengeType::Morris => PendingChallenge {
            challenge_type: ChallengeType::Morris,
            title: "Nine Men's Morris".to_string(),
            icon: "\u{25CB}",
            description: "An elderly sage arranges nine white stones on a weathered board. \
                \"The game of mills,\" they say. \"Three in a row captures. Shall we play?\""
                .to_string(),
        },
        ChallengeType::Gomoku => PendingChallenge {
            challenge_type: ChallengeType::Gomoku,
            title: "Gomoku".to_string(),
            icon: "◎",
            description: "A wandering strategist places a worn board before you. \
                \"Five stones in a row,\" they explain. \"Simple rules, deep tactics.\""
                .to_string(),
        },
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
        assert!(!menu.has_challenge(&ChallengeType::Chess));
        menu.add_challenge(make_chess_challenge());
        assert!(menu.has_challenge(&ChallengeType::Chess));
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
