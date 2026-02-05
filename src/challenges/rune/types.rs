//! Rune Deciphering challenge data structures.
//!
//! A Mastermind-style logic/deduction minigame where the player
//! decodes hidden sequences of ancient runes.

/// Rune symbols for display. First 5 used for Novice, first 6 for Apprentice/Journeyman, all 8 for Master.
pub const RUNE_SYMBOLS: &[char] = &['᛭', 'ᚦ', 'ᛟ', 'ᚱ', 'ᛊ', 'ᚹ', 'ᛏ', 'ᚲ'];

/// Rune difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuneDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

impl RuneDifficulty {
    pub const ALL: [RuneDifficulty; 4] = [
        RuneDifficulty::Novice,
        RuneDifficulty::Apprentice,
        RuneDifficulty::Journeyman,
        RuneDifficulty::Master,
    ];

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Novice,
            1 => Self::Apprentice,
            2 => Self::Journeyman,
            3 => Self::Master,
            _ => Self::Novice,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Novice => "Novice",
            Self::Apprentice => "Apprentice",
            Self::Journeyman => "Journeyman",
            Self::Master => "Master",
        }
    }

    pub fn num_runes(&self) -> usize {
        match self {
            Self::Novice => 5,
            Self::Apprentice | Self::Journeyman => 6,
            Self::Master => 8,
        }
    }

    pub fn num_slots(&self) -> usize {
        match self {
            Self::Novice => 3,
            Self::Apprentice | Self::Journeyman => 4,
            Self::Master => 5,
        }
    }

    pub fn max_guesses(&self) -> usize {
        match self {
            Self::Novice | Self::Apprentice => 10,
            Self::Journeyman | Self::Master => 8,
        }
    }

    pub fn allow_duplicates(&self) -> bool {
        match self {
            Self::Novice | Self::Apprentice => false,
            Self::Journeyman | Self::Master => true,
        }
    }
}

/// Result of a rune game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuneResult {
    Win,
    Loss,
}

/// Feedback for a single position in a guess
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackMark {
    /// Correct rune in correct position
    Exact,
    /// Correct rune, wrong position
    Misplaced,
    /// Rune not in code
    Wrong,
}

/// A submitted guess with feedback
#[derive(Debug, Clone)]
pub struct RuneGuess {
    /// Rune indices for each slot
    pub runes: Vec<usize>,
    /// Feedback marks (sorted: Exact first, then Misplaced, then Wrong)
    pub feedback: Vec<FeedbackMark>,
}

/// Full rune game state
#[derive(Debug, Clone)]
pub struct RuneGame {
    pub difficulty: RuneDifficulty,
    pub secret_code: Vec<usize>,
    pub guesses: Vec<RuneGuess>,
    pub current_guess: Vec<Option<usize>>,
    pub cursor_slot: usize,
    pub max_guesses: usize,
    pub num_runes: usize,
    pub num_slots: usize,
    pub allow_duplicates: bool,
    pub game_result: Option<RuneResult>,
    pub forfeit_pending: bool,
    pub reject_message: Option<String>,
}

impl RuneGame {
    pub fn new(difficulty: RuneDifficulty) -> Self {
        let num_slots = difficulty.num_slots();
        Self {
            difficulty,
            secret_code: Vec::new(),
            guesses: Vec::new(),
            current_guess: vec![None; num_slots],
            cursor_slot: 0,
            max_guesses: difficulty.max_guesses(),
            num_runes: difficulty.num_runes(),
            num_slots,
            allow_duplicates: difficulty.allow_duplicates(),
            game_result: None,
            forfeit_pending: false,
            reject_message: None,
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_slot > 0 {
            self.cursor_slot -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_slot + 1 < self.num_slots {
            self.cursor_slot += 1;
        }
    }

    pub fn cycle_rune_up(&mut self) {
        self.reject_message = None;
        let slot = self.cursor_slot;
        self.current_guess[slot] = Some(match self.current_guess[slot] {
            None => 0,
            Some(i) => (i + 1) % self.num_runes,
        });
    }

    pub fn cycle_rune_down(&mut self) {
        self.reject_message = None;
        let slot = self.cursor_slot;
        self.current_guess[slot] = Some(match self.current_guess[slot] {
            None => self.num_runes - 1,
            Some(0) => self.num_runes - 1,
            Some(i) => i - 1,
        });
    }

    pub fn clear_guess(&mut self) {
        for slot in self.current_guess.iter_mut() {
            *slot = None;
        }
        self.cursor_slot = 0;
    }

    pub fn is_guess_complete(&self) -> bool {
        self.current_guess.iter().all(|s| s.is_some())
    }

    pub fn guesses_remaining(&self) -> usize {
        self.max_guesses - self.guesses.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_config() {
        let n = RuneDifficulty::Novice;
        assert_eq!(n.num_runes(), 5);
        assert_eq!(n.num_slots(), 3);
        assert_eq!(n.max_guesses(), 10);
        assert!(!n.allow_duplicates());

        let a = RuneDifficulty::Apprentice;
        assert_eq!(a.num_runes(), 6);
        assert_eq!(a.num_slots(), 4);
        assert_eq!(a.max_guesses(), 10);
        assert!(!a.allow_duplicates());

        let j = RuneDifficulty::Journeyman;
        assert_eq!(j.num_runes(), 6);
        assert_eq!(j.num_slots(), 4);
        assert_eq!(j.max_guesses(), 8);
        assert!(j.allow_duplicates());

        let m = RuneDifficulty::Master;
        assert_eq!(m.num_runes(), 8);
        assert_eq!(m.num_slots(), 5);
        assert_eq!(m.max_guesses(), 8);
        assert!(m.allow_duplicates());
    }

    #[test]
    fn test_difficulty_names() {
        assert_eq!(RuneDifficulty::Novice.name(), "Novice");
        assert_eq!(RuneDifficulty::Apprentice.name(), "Apprentice");
        assert_eq!(RuneDifficulty::Journeyman.name(), "Journeyman");
        assert_eq!(RuneDifficulty::Master.name(), "Master");
    }

    #[test]
    fn test_from_index() {
        assert_eq!(RuneDifficulty::from_index(0), RuneDifficulty::Novice);
        assert_eq!(RuneDifficulty::from_index(1), RuneDifficulty::Apprentice);
        assert_eq!(RuneDifficulty::from_index(2), RuneDifficulty::Journeyman);
        assert_eq!(RuneDifficulty::from_index(3), RuneDifficulty::Master);
        assert_eq!(RuneDifficulty::from_index(99), RuneDifficulty::Novice);
    }

    #[test]
    fn test_all_constant() {
        assert_eq!(RuneDifficulty::ALL.len(), 4);
    }

    #[test]
    fn test_rune_game_new() {
        let game = RuneGame::new(RuneDifficulty::Novice);
        assert_eq!(game.num_slots, 3);
        assert_eq!(game.num_runes, 5);
        assert_eq!(game.max_guesses, 10);
        assert!(!game.allow_duplicates);
        assert!(game.guesses.is_empty());
        assert_eq!(game.current_guess.len(), 3);
        assert!(game.current_guess.iter().all(|s| s.is_none()));
        assert!(game.game_result.is_none());
        assert!(!game.forfeit_pending);
    }

    #[test]
    fn test_rune_symbols() {
        assert!(RUNE_SYMBOLS.len() >= 8);
    }

    #[test]
    fn test_cursor_movement() {
        let mut game = RuneGame::new(RuneDifficulty::Apprentice);
        assert_eq!(game.cursor_slot, 0);

        game.move_cursor_right();
        assert_eq!(game.cursor_slot, 1);

        game.move_cursor_right();
        game.move_cursor_right();
        assert_eq!(game.cursor_slot, 3);

        game.move_cursor_right();
        assert_eq!(game.cursor_slot, 3);

        game.move_cursor_left();
        assert_eq!(game.cursor_slot, 2);

        game.move_cursor_left();
        game.move_cursor_left();
        game.move_cursor_left();
        assert_eq!(game.cursor_slot, 0);
    }

    #[test]
    fn test_cycle_rune() {
        let mut game = RuneGame::new(RuneDifficulty::Novice);
        assert_eq!(game.current_guess[0], None);

        game.cycle_rune_up();
        assert_eq!(game.current_guess[0], Some(0));

        game.cycle_rune_up();
        assert_eq!(game.current_guess[0], Some(1));

        game.cycle_rune_down();
        assert_eq!(game.current_guess[0], Some(0));

        game.cycle_rune_down();
        assert_eq!(game.current_guess[0], Some(4));
    }

    #[test]
    fn test_clear_guess() {
        let mut game = RuneGame::new(RuneDifficulty::Novice);
        game.current_guess[0] = Some(0);
        game.current_guess[1] = Some(1);
        game.current_guess[2] = Some(2);

        game.clear_guess();
        assert!(game.current_guess.iter().all(|s| s.is_none()));
        assert_eq!(game.cursor_slot, 0);
    }

    #[test]
    fn test_guess_complete() {
        let mut game = RuneGame::new(RuneDifficulty::Novice);
        assert!(!game.is_guess_complete());

        game.current_guess[0] = Some(0);
        assert!(!game.is_guess_complete());

        game.current_guess[1] = Some(1);
        game.current_guess[2] = Some(2);
        assert!(game.is_guess_complete());
    }

    #[test]
    fn test_guesses_remaining() {
        let game = RuneGame::new(RuneDifficulty::Novice);
        assert_eq!(game.guesses_remaining(), 10);
    }

    #[test]
    fn test_reward_structure() {
        use crate::challenges::menu::DifficultyInfo;

        let novice = RuneDifficulty::Novice.reward();
        assert_eq!(novice.xp_percent, 25);
        assert_eq!(novice.prestige_ranks, 0);
        assert_eq!(novice.fishing_ranks, 0);

        let apprentice = RuneDifficulty::Apprentice.reward();
        assert_eq!(apprentice.xp_percent, 50);

        let journeyman = RuneDifficulty::Journeyman.reward();
        assert_eq!(journeyman.fishing_ranks, 1);
        assert_eq!(journeyman.xp_percent, 75);

        let master = RuneDifficulty::Master.reward();
        assert_eq!(master.prestige_ranks, 1);
        assert_eq!(master.fishing_ranks, 2);
        assert_eq!(master.xp_percent, 0);
    }

    #[test]
    fn test_game_new_all_difficulties() {
        for &diff in &RuneDifficulty::ALL {
            let game = RuneGame::new(diff);
            assert_eq!(game.current_guess.len(), game.num_slots);
            assert!(game.current_guess.iter().all(|s| s.is_none()));
            assert!(game.secret_code.is_empty());
            assert!(game.guesses.is_empty());
            assert!(game.game_result.is_none());
            assert!(game.reject_message.is_none());
        }
    }

    #[test]
    fn test_cycle_rune_clears_reject_message() {
        let mut game = RuneGame::new(RuneDifficulty::Novice);
        game.reject_message = Some("test".to_string());

        game.cycle_rune_up();
        assert!(game.reject_message.is_none());

        game.reject_message = Some("test".to_string());
        game.cycle_rune_down();
        assert!(game.reject_message.is_none());
    }

    #[test]
    fn test_cycle_rune_wraps_all_difficulties() {
        for &diff in &RuneDifficulty::ALL {
            let mut game = RuneGame::new(diff);
            // Cycle up through all runes and verify wrap
            for i in 0..game.num_runes {
                game.cycle_rune_up();
                assert_eq!(game.current_guess[0], Some(i));
            }
            // Should wrap back to 0
            game.cycle_rune_up();
            assert_eq!(game.current_guess[0], Some(0));
        }
    }
}
