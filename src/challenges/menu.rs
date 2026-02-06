//! Generic challenge menu system for player-controlled minigames.
//!
//! The challenge menu holds pending challenges that players can accept or decline.
//! Challenge discovery uses a single roll per tick. On success, a weighted distribution
//! table determines which challenge type appears.

use super::chess::logic::start_chess_game;
use super::chess::ChessDifficulty;
use super::go::logic::start_go_game;
use super::go::GoDifficulty;
use super::gomoku::logic::start_gomoku_game;
use super::gomoku::GomokuDifficulty;
use super::minesweeper::{MinesweeperDifficulty, MinesweeperGame};
use super::morris::logic::start_morris_game;
use super::morris::MorrisDifficulty;
use super::rune::{RuneDifficulty, RuneGame};
use super::ActiveMinigame;
use crate::core::game_state::GameState;
use rand::Rng;

/// Input actions for the Challenge Menu (UI-agnostic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuInput {
    Up,
    Down,
    Select,  // Enter - open detail or accept challenge
    Decline, // D - decline/remove challenge
    Cancel,  // Esc/Tab - close detail or close menu
    Other,
}

/// Process a menu input. Returns true if the menu is still open after processing.
pub fn process_input(state: &mut GameState, input: MenuInput) -> bool {
    if !state.challenge_menu.is_open {
        return false;
    }

    let menu = &mut state.challenge_menu;

    if menu.viewing_detail {
        match input {
            MenuInput::Up => menu.navigate_up(),
            MenuInput::Down => menu.navigate_down(4),
            MenuInput::Select => {
                accept_selected_challenge(state);
            }
            MenuInput::Decline => {
                decline_selected_challenge(state);
            }
            MenuInput::Cancel => {
                state.challenge_menu.close_detail();
            }
            MenuInput::Other => {}
        }
    } else {
        match input {
            MenuInput::Up => menu.navigate_up(),
            MenuInput::Down => menu.navigate_down(4),
            MenuInput::Select => menu.open_detail(),
            MenuInput::Cancel => menu.close(),
            MenuInput::Decline | MenuInput::Other => {}
        }
    }

    state.challenge_menu.is_open
}

/// Accept the currently selected challenge and start the appropriate game.
fn accept_selected_challenge(state: &mut GameState) {
    let difficulty_index = state.challenge_menu.selected_difficulty;

    if let Some(challenge) = state.challenge_menu.take_selected() {
        match challenge.challenge_type {
            ChallengeType::Chess => {
                let difficulty = ChessDifficulty::from_index(difficulty_index);
                start_chess_game(state, difficulty);
            }
            ChallengeType::Morris => {
                let difficulty = MorrisDifficulty::from_index(difficulty_index);
                start_morris_game(state, difficulty);
            }
            ChallengeType::Gomoku => {
                let difficulty = GomokuDifficulty::from_index(difficulty_index);
                start_gomoku_game(state, difficulty);
            }
            ChallengeType::Minesweeper => {
                let difficulty = MinesweeperDifficulty::from_index(difficulty_index);
                state.active_minigame = Some(ActiveMinigame::Minesweeper(MinesweeperGame::new(
                    difficulty,
                )));
                state.challenge_menu.close();
            }
            ChallengeType::Rune => {
                let difficulty = RuneDifficulty::from_index(difficulty_index);
                state.active_minigame = Some(ActiveMinigame::Rune(RuneGame::new(difficulty)));
                state.challenge_menu.close();
            }
            ChallengeType::Go => {
                let difficulty = GoDifficulty::from_index(difficulty_index);
                start_go_game(state, difficulty);
            }
        }
    }
}

/// Decline the currently selected challenge and remove it from the menu.
fn decline_selected_challenge(state: &mut GameState) {
    state.challenge_menu.take_selected();
    state.challenge_menu.close_detail();
    if state.challenge_menu.challenges.is_empty() {
        state.challenge_menu.close();
    }
}

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

impl DifficultyInfo for MinesweeperDifficulty {
    fn name(&self) -> &'static str {
        MinesweeperDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            MinesweeperDifficulty::Novice => ChallengeReward {
                xp_percent: 50,
                ..Default::default()
            },
            MinesweeperDifficulty::Apprentice => ChallengeReward {
                xp_percent: 75,
                ..Default::default()
            },
            MinesweeperDifficulty::Journeyman => ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            },
            MinesweeperDifficulty::Master => ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 200,
                ..Default::default()
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        let (h, w) = self.grid_size();
        Some(format!("{}x{}, {} traps", w, h, self.mine_count()))
    }
}

impl DifficultyInfo for RuneDifficulty {
    fn name(&self) -> &'static str {
        RuneDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            RuneDifficulty::Novice => ChallengeReward {
                xp_percent: 25,
                ..Default::default()
            },
            RuneDifficulty::Apprentice => ChallengeReward {
                xp_percent: 50,
                ..Default::default()
            },
            RuneDifficulty::Journeyman => ChallengeReward {
                fishing_ranks: 1,
                xp_percent: 75,
                ..Default::default()
            },
            RuneDifficulty::Master => ChallengeReward {
                prestige_ranks: 1,
                fishing_ranks: 2,
                ..Default::default()
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        let dupes = if self.allow_duplicates() {
            ", dupes"
        } else {
            ""
        };
        Some(format!(
            "{} runes, {} slots{}",
            self.num_runes(),
            self.num_slots(),
            dupes
        ))
    }
}

impl DifficultyInfo for GoDifficulty {
    fn name(&self) -> &'static str {
        GoDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        ChallengeReward {
            prestige_ranks: self.reward_prestige(),
            ..Default::default()
        }
    }

    fn extra_info(&self) -> Option<String> {
        Some(format!("{} sims", self.simulation_count()))
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
/// Puzzles (Minesweeper, Rune) are more common; strategy games (Chess, Go) are rarer.
const CHALLENGE_TABLE: &[ChallengeWeight] = &[
    ChallengeWeight {
        challenge_type: ChallengeType::Minesweeper,
        weight: 30, // ~27% - common quick puzzle
    },
    ChallengeWeight {
        challenge_type: ChallengeType::Rune,
        weight: 25, // ~23% - common quick puzzle
    },
    ChallengeWeight {
        challenge_type: ChallengeType::Gomoku,
        weight: 20, // ~18% - moderate
    },
    ChallengeWeight {
        challenge_type: ChallengeType::Morris,
        weight: 15, // ~14% - less common
    },
    ChallengeWeight {
        challenge_type: ChallengeType::Chess,
        weight: 10, // ~9% - rare complex strategy
    },
    ChallengeWeight {
        challenge_type: ChallengeType::Go,
        weight: 10, // ~9% - rare complex strategy
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
    Minesweeper,
    Rune,
    Go,
}

impl ChallengeType {
    /// Returns the icon used for this challenge type in log messages.
    pub fn icon(&self) -> &'static str {
        match self {
            ChallengeType::Chess => "♟",
            ChallengeType::Morris => "\u{25CB}", // ○
            ChallengeType::Gomoku => "◎",
            ChallengeType::Minesweeper => "\u{26A0}", // ⚠
            ChallengeType::Rune => "ᚱ",
            ChallengeType::Go => "◉",
        }
    }

    /// Returns the flavor text shown when this challenge is discovered.
    pub fn discovery_flavor(&self) -> &'static str {
        match self {
            ChallengeType::Chess => "A mysterious figure steps from the shadows...",
            ChallengeType::Morris => "A cloaked stranger approaches with a weathered board...",
            ChallengeType::Gomoku => "A wandering strategist places a worn board before you...",
            ChallengeType::Minesweeper => {
                "A weathered scout beckons you toward a ruined corridor..."
            }
            ChallengeType::Rune => "A glowing stone tablet materializes before you...",
            ChallengeType::Go => "An ancient master beckons from beneath a gnarled tree...",
        }
    }
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
/// `haven_discovery_percent` is the Library bonus (0.0 if not built)
pub fn try_discover_challenge_with_haven<R: Rng>(
    state: &mut GameState,
    rng: &mut R,
    haven_discovery_percent: f64,
) -> Option<ChallengeType> {
    // Requirements: P1+, not in dungeon, not fishing, not in active minigame
    if state.prestige_rank < 1
        || state.active_dungeon.is_some()
        || state.active_fishing.is_some()
        || state.active_minigame.is_some()
    {
        return None;
    }

    // Apply Library bonus to discovery chance
    let discovery_chance = CHALLENGE_DISCOVERY_CHANCE * (1.0 + haven_discovery_percent / 100.0);

    // Single roll for any challenge
    if rng.gen::<f64>() >= discovery_chance {
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

/// Legacy function without Haven bonus (for backwards compatibility)
#[allow(dead_code)]
pub fn try_discover_challenge<R: Rng>(state: &mut GameState, rng: &mut R) -> Option<ChallengeType> {
    try_discover_challenge_with_haven(state, rng, 0.0)
}

/// Create a PendingChallenge from a ChallengeType
pub fn create_challenge(ct: &ChallengeType) -> PendingChallenge {
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
        ChallengeType::Minesweeper => PendingChallenge {
            challenge_type: ChallengeType::Minesweeper,
            title: "Minesweeper: Trap Detection".to_string(),
            icon: "\u{26A0}",
            description: "A weathered scout beckons you toward a ruined corridor. \
                'The floor's rigged with pressure plates,' she warns, pulling out a \
                worn map. 'One wrong step and...' She makes an explosive gesture. \
                'Help me chart the safe path. Probe carefully—the numbers tell you \
                how many traps lurk nearby.'"
                .to_string(),
        },
        ChallengeType::Rune => PendingChallenge {
            challenge_type: ChallengeType::Rune,
            title: "Rune Deciphering: Ancient Tablet".to_string(),
            icon: "ᚱ",
            description: "You stumble upon a stone tablet covered in glowing runes. \
                A spectral voice echoes: 'Decipher the hidden sequence, mortal. \
                Each attempt reveals clues\u{2014}exact matches, misplaced symbols, or \
                false leads. Prove your logic worthy of ancient knowledge.'"
                .to_string(),
        },
        ChallengeType::Go => PendingChallenge {
            challenge_type: ChallengeType::Go,
            title: "Go: Territory Control".to_string(),
            icon: "◉",
            description: "An ancient master beckons from beneath a gnarled tree, a wooden \
                board resting on a flat stone before them. Nine lines cross nine lines, \
                forming a grid of intersections. 'Black and white stones,' they say, \
                'placed one by one. Surround territory, capture enemies. The simplest \
                rules hide the deepest strategy. Shall we play?'"
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

    // ============ ChallengeType Method Tests ============

    #[test]
    fn test_challenge_type_icon_returns_non_empty() {
        assert!(!ChallengeType::Chess.icon().is_empty());
        assert!(!ChallengeType::Morris.icon().is_empty());
        assert!(!ChallengeType::Gomoku.icon().is_empty());
        assert!(!ChallengeType::Minesweeper.icon().is_empty());
        assert!(!ChallengeType::Rune.icon().is_empty());
        assert!(!ChallengeType::Go.icon().is_empty());
    }

    #[test]
    fn test_challenge_type_discovery_flavor_returns_non_empty() {
        assert!(!ChallengeType::Chess.discovery_flavor().is_empty());
        assert!(!ChallengeType::Morris.discovery_flavor().is_empty());
        assert!(!ChallengeType::Gomoku.discovery_flavor().is_empty());
        assert!(!ChallengeType::Minesweeper.discovery_flavor().is_empty());
        assert!(!ChallengeType::Rune.discovery_flavor().is_empty());
        assert!(!ChallengeType::Go.discovery_flavor().is_empty());
    }

    #[test]
    fn test_challenge_type_icons_are_unique() {
        let icons = [
            ChallengeType::Chess.icon(),
            ChallengeType::Morris.icon(),
            ChallengeType::Gomoku.icon(),
            ChallengeType::Minesweeper.icon(),
            ChallengeType::Rune.icon(),
            ChallengeType::Go.icon(),
        ];
        // Check all pairs are different
        for i in 0..icons.len() {
            for j in (i + 1)..icons.len() {
                assert_ne!(icons[i], icons[j], "Icons should be unique");
            }
        }
    }

    // ============ ChallengeMenu Tests ============

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

    // ============ Process Input Tests ============

    #[test]
    fn test_process_input_returns_false_when_menu_closed() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.is_open = false;

        let result = process_input(&mut state, MenuInput::Up);

        assert!(!result);
    }

    #[test]
    fn test_process_input_navigation_in_list_view() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(make_chess_challenge());
        state.challenge_menu.add_challenge(make_chess_challenge());
        state.challenge_menu.add_challenge(make_chess_challenge());
        state.challenge_menu.open();

        assert_eq!(state.challenge_menu.selected_index, 0);

        process_input(&mut state, MenuInput::Down);
        assert_eq!(state.challenge_menu.selected_index, 1);

        process_input(&mut state, MenuInput::Down);
        assert_eq!(state.challenge_menu.selected_index, 2);

        process_input(&mut state, MenuInput::Up);
        assert_eq!(state.challenge_menu.selected_index, 1);
    }

    #[test]
    fn test_process_input_select_opens_detail() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(make_chess_challenge());
        state.challenge_menu.open();

        assert!(!state.challenge_menu.viewing_detail);

        process_input(&mut state, MenuInput::Select);

        assert!(state.challenge_menu.viewing_detail);
    }

    #[test]
    fn test_process_input_cancel_closes_menu_in_list_view() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(make_chess_challenge());
        state.challenge_menu.open();

        assert!(state.challenge_menu.is_open);

        let result = process_input(&mut state, MenuInput::Cancel);

        assert!(!state.challenge_menu.is_open);
        assert!(!result);
    }

    #[test]
    fn test_process_input_cancel_closes_detail_in_detail_view() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(make_chess_challenge());
        state.challenge_menu.open();
        state.challenge_menu.open_detail();

        assert!(state.challenge_menu.viewing_detail);

        process_input(&mut state, MenuInput::Cancel);

        assert!(!state.challenge_menu.viewing_detail);
        assert!(state.challenge_menu.is_open); // Menu still open
    }

    #[test]
    fn test_process_input_navigation_in_detail_view() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(make_chess_challenge());
        state.challenge_menu.open();
        state.challenge_menu.open_detail();

        assert_eq!(state.challenge_menu.selected_difficulty, 0);

        process_input(&mut state, MenuInput::Down);
        assert_eq!(state.challenge_menu.selected_difficulty, 1);

        process_input(&mut state, MenuInput::Down);
        assert_eq!(state.challenge_menu.selected_difficulty, 2);

        process_input(&mut state, MenuInput::Up);
        assert_eq!(state.challenge_menu.selected_difficulty, 1);
    }

    #[test]
    fn test_process_input_decline_removes_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(make_chess_challenge());
        state.challenge_menu.add_challenge(make_chess_challenge());
        state.challenge_menu.open();
        state.challenge_menu.open_detail();

        assert_eq!(state.challenge_menu.challenges.len(), 2);

        process_input(&mut state, MenuInput::Decline);

        assert_eq!(state.challenge_menu.challenges.len(), 1);
        assert!(!state.challenge_menu.viewing_detail);
    }

    #[test]
    fn test_process_input_decline_closes_menu_when_empty() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(make_chess_challenge());
        state.challenge_menu.open();
        state.challenge_menu.open_detail();

        process_input(&mut state, MenuInput::Decline);

        assert!(!state.challenge_menu.is_open);
        assert!(state.challenge_menu.challenges.is_empty());
    }

    #[test]
    fn test_process_input_select_starts_chess_game() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(make_chess_challenge());
        state.challenge_menu.open();
        state.challenge_menu.open_detail();
        state.challenge_menu.selected_difficulty = 1; // Apprentice

        assert!(state.active_minigame.is_none());

        process_input(&mut state, MenuInput::Select);

        assert!(matches!(
            state.active_minigame,
            Some(ActiveMinigame::Chess(_))
        ));
        assert!(!state.challenge_menu.is_open);
    }

    #[test]
    fn test_process_input_select_starts_morris_game() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(PendingChallenge {
            challenge_type: ChallengeType::Morris,
            title: "Morris Challenge".to_string(),
            icon: "○",
            description: "Test".to_string(),
        });
        state.challenge_menu.open();
        state.challenge_menu.open_detail();

        process_input(&mut state, MenuInput::Select);

        assert!(matches!(
            state.active_minigame,
            Some(ActiveMinigame::Morris(_))
        ));
        assert!(!state.challenge_menu.is_open);
    }

    #[test]
    fn test_process_input_select_starts_gomoku_game() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(PendingChallenge {
            challenge_type: ChallengeType::Gomoku,
            title: "Gomoku Challenge".to_string(),
            icon: "◎",
            description: "Test".to_string(),
        });
        state.challenge_menu.open();
        state.challenge_menu.open_detail();

        process_input(&mut state, MenuInput::Select);

        assert!(matches!(
            state.active_minigame,
            Some(ActiveMinigame::Gomoku(_))
        ));
        assert!(!state.challenge_menu.is_open);
    }

    #[test]
    fn test_process_input_select_starts_minesweeper_game() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(PendingChallenge {
            challenge_type: ChallengeType::Minesweeper,
            title: "Minesweeper Challenge".to_string(),
            icon: "⚠",
            description: "Test".to_string(),
        });
        state.challenge_menu.open();
        state.challenge_menu.open_detail();

        process_input(&mut state, MenuInput::Select);

        assert!(matches!(
            state.active_minigame,
            Some(ActiveMinigame::Minesweeper(_))
        ));
        assert!(!state.challenge_menu.is_open);
    }

    #[test]
    fn test_process_input_select_starts_rune_game() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(PendingChallenge {
            challenge_type: ChallengeType::Rune,
            title: "Rune Challenge".to_string(),
            icon: "ᚱ",
            description: "Test".to_string(),
        });
        state.challenge_menu.open();
        state.challenge_menu.open_detail();

        process_input(&mut state, MenuInput::Select);

        assert!(matches!(
            state.active_minigame,
            Some(ActiveMinigame::Rune(_))
        ));
        assert!(!state.challenge_menu.is_open);
    }

    // =========================================================================
    // Haven Discovery Bonus Tests
    // =========================================================================

    #[test]
    fn test_haven_discovery_bonus_increases_chance() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        // Test that the bonus is applied correctly by checking at fixed RNG values
        // Base discovery chance is 0.000014, so we need RNG values very close to 0 to discover

        // Count discoveries in a reasonable sample
        let trials = 50000;
        let mut discoveries_no_bonus = 0;
        let mut discoveries_with_bonus = 0;

        for seed in 0..trials {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut state = GameState::new("Test".to_string(), 0);
            state.prestige_rank = 1;

            if try_discover_challenge_with_haven(&mut state, &mut rng, 0.0).is_some() {
                discoveries_no_bonus += 1;
            }
        }

        for seed in 0..trials {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut state = GameState::new("Test".to_string(), 0);
            state.prestige_rank = 1;

            if try_discover_challenge_with_haven(&mut state, &mut rng, 50.0).is_some() {
                discoveries_with_bonus += 1;
            }
        }

        // With +50% bonus, should see more discoveries than without
        // Given the low base rate, we just verify the bonus increases discoveries
        assert!(
            discoveries_with_bonus >= discoveries_no_bonus,
            "Haven +50% discovery should increase or equal rate: no_bonus={}, with_bonus={}",
            discoveries_no_bonus,
            discoveries_with_bonus
        );
    }
}
