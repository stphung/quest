//! Generic challenge menu system for player-controlled minigames.
//!
//! The challenge menu holds pending challenges that players can accept or decline.
//! Challenge discovery uses a single roll per tick. On success, a weighted distribution
//! table determines which challenge type appears.

use crate::chess::ChessDifficulty;
use crate::game_state::GameState;
use crate::gomoku::GomokuDifficulty;
use crate::morris::MorrisDifficulty;
use rand::Rng;

/// Structured reward for challenge victories - single source of truth
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChallengeReward {
    pub prestige_ranks: u32,
    pub xp_percent: u32,
    pub fishing_ranks: u32,
}

impl ChallengeReward {
    /// Generate display text from structured data
    /// Order: Prestige -> Fishing -> XP
    pub fn description(&self) -> String {
        let mut parts = Vec::new();

        if self.prestige_ranks == 1 {
            parts.push("+1 Prestige Rank".to_string());
        } else if self.prestige_ranks > 1 {
            parts.push(format!("+{} Prestige Ranks", self.prestige_ranks));
        }

        if self.fishing_ranks == 1 {
            parts.push("+1 Fish Rank".to_string());
        } else if self.fishing_ranks > 1 {
            parts.push(format!("+{} Fish Ranks", self.fishing_ranks));
        }

        if self.xp_percent > 0 {
            parts.push(format!("+{}% level XP", self.xp_percent));
        }

        if parts.is_empty() {
            "No reward".to_string()
        } else {
            format!("Win: {}", parts.join(", "))
        }
    }
}

/// Trait for difficulty levels that can be displayed in the challenge menu
pub trait DifficultyInfo {
    /// Display name (e.g., "Novice", "Master")
    fn name(&self) -> &'static str;

    /// Structured reward for winning at this difficulty
    fn reward(&self) -> ChallengeReward;

    /// Optional extra info shown between name and reward (e.g., "~500 ELO")
    fn extra_info(&self) -> Option<String> {
        None
    }
}

impl DifficultyInfo for ChessDifficulty {
    fn name(&self) -> &'static str {
        ChessDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        ChallengeReward {
            prestige_ranks: self.reward_prestige(),
            ..Default::default()
        }
    }

    fn extra_info(&self) -> Option<String> {
        Some(format!("~{} ELO", self.estimated_elo()))
    }
}

impl DifficultyInfo for MorrisDifficulty {
    fn name(&self) -> &'static str {
        MorrisDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        ChallengeReward {
            xp_percent: self.reward_xp_percent(),
            fishing_ranks: if *self == MorrisDifficulty::Master {
                1
            } else {
                0
            },
            ..Default::default()
        }
    }
}

impl DifficultyInfo for GomokuDifficulty {
    fn name(&self) -> &'static str {
        GomokuDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            GomokuDifficulty::Novice => ChallengeReward {
                xp_percent: 75,
                ..Default::default()
            },
            GomokuDifficulty::Apprentice => ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            },
            GomokuDifficulty::Journeyman => ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 50,
                ..Default::default()
            },
            GomokuDifficulty::Master => ChallengeReward {
                prestige_ranks: 2,
                xp_percent: 100,
                ..Default::default()
            },
        }
    }
}

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
            title: "Chess: The Hooded Challenger".to_string(),
            icon: "♟",
            description: "In the corner of a dimly lit tavern, a hooded figure sits \
                motionless before a chess board. The pieces seem to shimmer with an \
                otherworldly glow. As you approach, they speak without looking up: \
                \"I've been waiting for a worthy opponent. The stakes? Your wit against \
                mine. Do you dare?\""
                .to_string(),
        },
        ChallengeType::Morris => PendingChallenge {
            challenge_type: ChallengeType::Morris,
            title: "Morris: The Millkeeper's Game".to_string(),
            icon: "\u{25CB}",
            description: "An ancient sage materializes from the morning mist, carrying a \
                weathered board etched with concentric squares. \"The game of mills,\" \
                they whisper, placing nine polished stones before you. \"Form three in \
                a row to capture. Reduce me to two pieces, and victory is yours. But \
                beware—I've played this game for centuries.\""
                .to_string(),
        },
        ChallengeType::Gomoku => PendingChallenge {
            challenge_type: ChallengeType::Gomoku,
            title: "Gomoku: Five Stones".to_string(),
            icon: "◎",
            description: "A wandering strategist blocks your path, unfurling a grid-lined \
                cloth upon a flat stone. \"They call this the hand-talk game,\" she says, \
                placing black and white stones in her palms. \"First to align five stones \
                claims victory. The rules are simple, but mastery takes a lifetime. Shall \
                we test your strategic mind?\""
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

    // ============ ChallengeReward Description Tests ============

    #[test]
    fn test_reward_description_prestige_only() {
        let reward = ChallengeReward {
            prestige_ranks: 1,
            ..Default::default()
        };
        assert_eq!(reward.description(), "Win: +1 Prestige Rank");

        let reward = ChallengeReward {
            prestige_ranks: 5,
            ..Default::default()
        };
        assert_eq!(reward.description(), "Win: +5 Prestige Ranks");
    }

    #[test]
    fn test_reward_description_xp_only() {
        let reward = ChallengeReward {
            xp_percent: 75,
            ..Default::default()
        };
        assert_eq!(reward.description(), "Win: +75% level XP");
    }

    #[test]
    fn test_reward_description_fishing_only() {
        let reward = ChallengeReward {
            fishing_ranks: 1,
            ..Default::default()
        };
        assert_eq!(reward.description(), "Win: +1 Fish Rank");

        let reward = ChallengeReward {
            fishing_ranks: 2,
            ..Default::default()
        };
        assert_eq!(reward.description(), "Win: +2 Fish Ranks");
    }

    #[test]
    fn test_reward_description_mixed() {
        // Prestige + XP
        let reward = ChallengeReward {
            prestige_ranks: 1,
            xp_percent: 50,
            ..Default::default()
        };
        assert_eq!(reward.description(), "Win: +1 Prestige Rank, +50% level XP");

        // All three (order: prestige -> fishing -> XP)
        let reward = ChallengeReward {
            prestige_ranks: 2,
            fishing_ranks: 1,
            xp_percent: 100,
        };
        assert_eq!(
            reward.description(),
            "Win: +2 Prestige Ranks, +1 Fish Rank, +100% level XP"
        );
    }

    #[test]
    fn test_reward_description_empty() {
        let reward = ChallengeReward::default();
        assert_eq!(reward.description(), "No reward");
    }
}
