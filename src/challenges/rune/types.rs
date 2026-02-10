//! Rune Deciphering challenge data structures.
//!
//! A Mastermind-style logic/deduction minigame where the player
//! decodes hidden sequences of ancient runes.

use crate::challenges::{ChallengeDifficulty, ChallengeResult};

/// Rune symbols for display. First 5 used for Novice, first 6 for Apprentice/Journeyman, all 8 for Master.
pub const RUNE_SYMBOLS: &[char] = &['᛭', 'ᚦ', 'ᛟ', 'ᚱ', 'ᛊ', 'ᚹ', 'ᛏ', 'ᚲ'];

/// Number of available rune symbols for the given difficulty.
pub fn num_runes_for(difficulty: ChallengeDifficulty) -> usize {
    match difficulty {
        ChallengeDifficulty::Novice => 5,
        ChallengeDifficulty::Apprentice | ChallengeDifficulty::Journeyman => 6,
        ChallengeDifficulty::Master => 8,
    }
}

/// Number of code slots for the given difficulty.
pub fn num_slots_for(difficulty: ChallengeDifficulty) -> usize {
    match difficulty {
        ChallengeDifficulty::Novice => 3,
        ChallengeDifficulty::Apprentice | ChallengeDifficulty::Journeyman => 4,
        ChallengeDifficulty::Master => 5,
    }
}

/// Maximum guesses allowed for the given difficulty.
pub fn max_guesses_for(difficulty: ChallengeDifficulty) -> usize {
    match difficulty {
        ChallengeDifficulty::Novice | ChallengeDifficulty::Apprentice => 10,
        ChallengeDifficulty::Journeyman | ChallengeDifficulty::Master => 8,
    }
}

/// Whether duplicate runes are allowed in the code for the given difficulty.
pub fn allow_duplicates_for(difficulty: ChallengeDifficulty) -> bool {
    match difficulty {
        ChallengeDifficulty::Novice | ChallengeDifficulty::Apprentice => false,
        ChallengeDifficulty::Journeyman | ChallengeDifficulty::Master => true,
    }
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
    pub difficulty: ChallengeDifficulty,
    pub secret_code: Vec<usize>,
    pub guesses: Vec<RuneGuess>,
    pub current_guess: Vec<Option<usize>>,
    pub cursor_slot: usize,
    pub max_guesses: usize,
    pub num_runes: usize,
    pub num_slots: usize,
    pub allow_duplicates: bool,
    pub game_result: Option<ChallengeResult>,
    pub forfeit_pending: bool,
    pub reject_message: Option<String>,
}

impl RuneGame {
    pub fn new(difficulty: ChallengeDifficulty) -> Self {
        let num_slots = num_slots_for(difficulty);
        Self {
            difficulty,
            secret_code: Vec::new(),
            guesses: Vec::new(),
            current_guess: vec![None; num_slots],
            cursor_slot: 0,
            max_guesses: max_guesses_for(difficulty),
            num_runes: num_runes_for(difficulty),
            num_slots,
            allow_duplicates: allow_duplicates_for(difficulty),
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
        let current = self.current_guess[slot].unwrap_or(0);
        self.current_guess[slot] = Some(if current == 0 {
            self.num_runes - 1
        } else {
            current - 1
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
        let n = ChallengeDifficulty::Novice;
        assert_eq!(num_runes_for(n), 5);
        assert_eq!(num_slots_for(n), 3);
        assert_eq!(max_guesses_for(n), 10);
        assert!(!allow_duplicates_for(n));

        let a = ChallengeDifficulty::Apprentice;
        assert_eq!(num_runes_for(a), 6);
        assert_eq!(num_slots_for(a), 4);
        assert_eq!(max_guesses_for(a), 10);
        assert!(!allow_duplicates_for(a));

        let j = ChallengeDifficulty::Journeyman;
        assert_eq!(num_runes_for(j), 6);
        assert_eq!(num_slots_for(j), 4);
        assert_eq!(max_guesses_for(j), 8);
        assert!(allow_duplicates_for(j));

        let m = ChallengeDifficulty::Master;
        assert_eq!(num_runes_for(m), 8);
        assert_eq!(num_slots_for(m), 5);
        assert_eq!(max_guesses_for(m), 8);
        assert!(allow_duplicates_for(m));
    }

    #[test]
    fn test_rune_game_new() {
        let game = RuneGame::new(ChallengeDifficulty::Novice);
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
        let mut game = RuneGame::new(ChallengeDifficulty::Apprentice);
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
        let mut game = RuneGame::new(ChallengeDifficulty::Novice);
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
        let mut game = RuneGame::new(ChallengeDifficulty::Novice);
        game.current_guess[0] = Some(0);
        game.current_guess[1] = Some(1);
        game.current_guess[2] = Some(2);

        game.clear_guess();
        assert!(game.current_guess.iter().all(|s| s.is_none()));
        assert_eq!(game.cursor_slot, 0);
    }

    #[test]
    fn test_guess_complete() {
        let mut game = RuneGame::new(ChallengeDifficulty::Novice);
        assert!(!game.is_guess_complete());

        game.current_guess[0] = Some(0);
        assert!(!game.is_guess_complete());

        game.current_guess[1] = Some(1);
        game.current_guess[2] = Some(2);
        assert!(game.is_guess_complete());
    }

    #[test]
    fn test_guesses_remaining() {
        let game = RuneGame::new(ChallengeDifficulty::Novice);
        assert_eq!(game.guesses_remaining(), 10);
    }

    #[test]
    fn test_reward_structure() {
        use crate::challenges::menu::ChallengeType;

        let novice = ChallengeType::Rune.reward(ChallengeDifficulty::Novice);
        assert_eq!(novice.xp_percent, 25);
        assert_eq!(novice.prestige_ranks, 0);
        assert_eq!(novice.fishing_ranks, 0);

        let apprentice = ChallengeType::Rune.reward(ChallengeDifficulty::Apprentice);
        assert_eq!(apprentice.xp_percent, 50);

        let journeyman = ChallengeType::Rune.reward(ChallengeDifficulty::Journeyman);
        assert_eq!(journeyman.fishing_ranks, 1);
        assert_eq!(journeyman.xp_percent, 75);

        let master = ChallengeType::Rune.reward(ChallengeDifficulty::Master);
        assert_eq!(master.prestige_ranks, 1);
        assert_eq!(master.fishing_ranks, 2);
        assert_eq!(master.xp_percent, 0);
    }

    #[test]
    fn test_game_new_all_difficulties() {
        for &diff in &ChallengeDifficulty::ALL {
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
        let mut game = RuneGame::new(ChallengeDifficulty::Novice);
        game.reject_message = Some("test".to_string());

        game.cycle_rune_up();
        assert!(game.reject_message.is_none());

        game.reject_message = Some("test".to_string());
        game.cycle_rune_down();
        assert!(game.reject_message.is_none());
    }

    #[test]
    fn test_cycle_rune_wraps_all_difficulties() {
        for &diff in &ChallengeDifficulty::ALL {
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
