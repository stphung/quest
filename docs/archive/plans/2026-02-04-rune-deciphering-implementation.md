# Rune Deciphering Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a Mastermind-style "Rune Deciphering" challenge minigame to Quest.

**Architecture:** Follow the exact same integration pattern as the Minesweeper challenge. Core data in `rune.rs`, game logic in `rune_logic.rs`, UI in `ui/rune_scene.rs`. Wire into challenge_menu, game_state, main.rs input handling, debug menu, and UI rendering dispatch.

**Tech Stack:** Rust, Ratatui (terminal UI), rand (RNG)

**Design doc:** `docs/plans/2026-02-04-rune-deciphering-design.md`

---

### Task 1: Core Data Structures (`src/rune.rs`)

**Files:**
- Create: `src/rune.rs`

**Step 1: Write tests for difficulty configuration**

Add at bottom of `src/rune.rs`:

```rust
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
        // Secret code not generated until first guess submitted (or at creation - design choice)
    }

    #[test]
    fn test_rune_symbols() {
        assert!(RUNE_SYMBOLS.len() >= 8);
    }

    #[test]
    fn test_cursor_movement() {
        let mut game = RuneGame::new(RuneDifficulty::Apprentice); // 4 slots
        assert_eq!(game.cursor_slot, 0);

        game.move_cursor_right();
        assert_eq!(game.cursor_slot, 1);

        game.move_cursor_right();
        game.move_cursor_right();
        assert_eq!(game.cursor_slot, 3);

        // Can't go past end
        game.move_cursor_right();
        assert_eq!(game.cursor_slot, 3);

        game.move_cursor_left();
        assert_eq!(game.cursor_slot, 2);

        // Can't go before start
        game.move_cursor_left();
        game.move_cursor_left();
        game.move_cursor_left();
        assert_eq!(game.cursor_slot, 0);
    }

    #[test]
    fn test_cycle_rune() {
        let mut game = RuneGame::new(RuneDifficulty::Novice); // 5 runes
        assert_eq!(game.current_guess[0], None);

        game.cycle_rune_up();
        assert_eq!(game.current_guess[0], Some(0));

        game.cycle_rune_up();
        assert_eq!(game.current_guess[0], Some(1));

        // Cycle down from 1 -> 0
        game.cycle_rune_down();
        assert_eq!(game.current_guess[0], Some(0));

        // Cycle down from 0 -> wraps to last rune
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
        let mut game = RuneGame::new(RuneDifficulty::Novice); // 3 slots
        assert!(!game.is_guess_complete());

        game.current_guess[0] = Some(0);
        assert!(!game.is_guess_complete());

        game.current_guess[1] = Some(1);
        game.current_guess[2] = Some(2);
        assert!(game.is_guess_complete());
    }

    #[test]
    fn test_guesses_remaining() {
        let game = RuneGame::new(RuneDifficulty::Novice); // 10 max
        assert_eq!(game.guesses_remaining(), 10);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib rune::tests -- --nocapture 2>&1 | head -20`
Expected: Compilation error (module doesn't exist yet)

**Step 3: Implement rune.rs**

```rust
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
}

impl RuneGame {
    pub fn new(difficulty: RuneDifficulty) -> Self {
        let num_slots = difficulty.num_slots();
        Self {
            difficulty,
            secret_code: Vec::new(), // Generated on first guess or via generate_code()
            guesses: Vec::new(),
            current_guess: vec![None; num_slots],
            cursor_slot: 0,
            max_guesses: difficulty.max_guesses(),
            num_runes: difficulty.num_runes(),
            num_slots,
            allow_duplicates: difficulty.allow_duplicates(),
            game_result: None,
            forfeit_pending: false,
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
        let slot = self.cursor_slot;
        self.current_guess[slot] = Some(match self.current_guess[slot] {
            None => 0,
            Some(i) => (i + 1) % self.num_runes,
        });
    }

    pub fn cycle_rune_down(&mut self) {
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
```

**Step 4: Add module to lib.rs**

In `src/lib.rs`, add after line 38 (`pub mod prestige;`):

```rust
pub mod rune;
pub mod rune_logic;
```

Note: `rune_logic` will be an empty file for now so it compiles. Create `src/rune_logic.rs` with:

```rust
//! Rune Deciphering game logic.
```

**Step 5: Run tests to verify they pass**

Run: `cargo test --lib rune::tests -v`
Expected: All tests PASS

**Step 6: Commit**

```bash
git add src/rune.rs src/rune_logic.rs src/lib.rs
git commit -m "feat(rune): add core data structures and difficulty config"
```

---

### Task 2: Game Logic (`src/rune_logic.rs`)

**Files:**
- Create: `src/rune_logic.rs` (replace placeholder)

**Step 1: Write tests for code generation and feedback**

Add to `src/rune_logic.rs`:

```rust
//! Rune Deciphering game logic.
//!
//! Handles secret code generation, feedback calculation, and guess submission.

use crate::rune::{FeedbackMark, RuneGame, RuneGuess, RuneResult};
use rand::Rng;

/// Generate the secret code for a rune game.
pub fn generate_code<R: Rng>(game: &mut RuneGame, rng: &mut R) {
    if game.allow_duplicates {
        game.secret_code = (0..game.num_slots)
            .map(|_| rng.gen_range(0..game.num_runes))
            .collect();
    } else {
        // Sample without replacement using Fisher-Yates partial shuffle
        let mut pool: Vec<usize> = (0..game.num_runes).collect();
        for i in 0..game.num_slots {
            let j = rng.gen_range(i..pool.len());
            pool.swap(i, j);
        }
        game.secret_code = pool[..game.num_slots].to_vec();
    }
}

/// Calculate feedback for a guess against the secret code.
/// Returns feedback sorted: Exact first, then Misplaced, then Wrong.
pub fn calculate_feedback(guess: &[usize], secret: &[usize]) -> Vec<FeedbackMark> {
    let len = guess.len();
    let mut result = vec![FeedbackMark::Wrong; len];
    let mut secret_used = vec![false; len];
    let mut guess_used = vec![false; len];

    // Pass 1: Find exact matches
    for i in 0..len {
        if guess[i] == secret[i] {
            result[i] = FeedbackMark::Exact;
            secret_used[i] = true;
            guess_used[i] = true;
        }
    }

    // Pass 2: Find misplaced matches
    for i in 0..len {
        if guess_used[i] {
            continue;
        }
        for j in 0..len {
            if !secret_used[j] && guess[i] == secret[j] {
                result[i] = FeedbackMark::Misplaced;
                secret_used[j] = true;
                break;
            }
        }
    }

    // Sort: Exact first, then Misplaced, then Wrong
    result.sort_by_key(|m| match m {
        FeedbackMark::Exact => 0,
        FeedbackMark::Misplaced => 1,
        FeedbackMark::Wrong => 2,
    });

    result
}

/// Submit the current guess. Returns true if the guess was accepted.
/// Generates secret code on first guess if not yet generated.
pub fn submit_guess<R: Rng>(game: &mut RuneGame, rng: &mut R) -> bool {
    if !game.is_guess_complete() || game.game_result.is_some() {
        return false;
    }

    // Generate code on first guess
    if game.secret_code.is_empty() {
        generate_code(game, rng);
    }

    let guess_runes: Vec<usize> = game.current_guess.iter().map(|s| s.unwrap()).collect();

    // Validate no duplicates if not allowed
    if !game.allow_duplicates {
        let mut seen = std::collections::HashSet::new();
        for &r in &guess_runes {
            if !seen.insert(r) {
                return false; // Duplicate in no-dupe mode
            }
        }
    }

    let feedback = calculate_feedback(&guess_runes, &game.secret_code);

    let all_exact = feedback.iter().all(|m| *m == FeedbackMark::Exact);

    game.guesses.push(RuneGuess {
        runes: guess_runes,
        feedback,
    });

    // Clear current guess for next round
    game.clear_guess();

    // Check win/loss
    if all_exact {
        game.game_result = Some(RuneResult::Win);
    } else if game.guesses.len() >= game.max_guesses {
        game.game_result = Some(RuneResult::Loss);
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn seeded_rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    #[test]
    fn test_generate_code_no_dupes() {
        let mut game = RuneGame::new(crate::rune::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        generate_code(&mut game, &mut rng);

        assert_eq!(game.secret_code.len(), 3);
        // All within range
        assert!(game.secret_code.iter().all(|&r| r < 5));
        // No duplicates
        let mut sorted = game.secret_code.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), game.secret_code.len());
    }

    #[test]
    fn test_generate_code_with_dupes() {
        let mut game = RuneGame::new(crate::rune::RuneDifficulty::Master);
        let mut rng = seeded_rng();
        generate_code(&mut game, &mut rng);

        assert_eq!(game.secret_code.len(), 5);
        assert!(game.secret_code.iter().all(|&r| r < 8));
        // Duplicates allowed - just check length and range
    }

    #[test]
    fn test_feedback_all_exact() {
        let feedback = calculate_feedback(&[0, 1, 2], &[0, 1, 2]);
        assert_eq!(feedback, vec![FeedbackMark::Exact, FeedbackMark::Exact, FeedbackMark::Exact]);
    }

    #[test]
    fn test_feedback_all_wrong() {
        let feedback = calculate_feedback(&[0, 1, 2], &[3, 4, 5]);
        assert_eq!(feedback, vec![FeedbackMark::Wrong, FeedbackMark::Wrong, FeedbackMark::Wrong]);
    }

    #[test]
    fn test_feedback_all_misplaced() {
        let feedback = calculate_feedback(&[0, 1, 2], &[2, 0, 1]);
        assert_eq!(feedback, vec![
            FeedbackMark::Misplaced,
            FeedbackMark::Misplaced,
            FeedbackMark::Misplaced,
        ]);
    }

    #[test]
    fn test_feedback_mixed() {
        // Secret: [0, 1, 2, 3]
        // Guess:  [0, 2, 3, 4]
        // Slot 0: exact (0==0)
        // Slot 1: misplaced (2 is in secret at pos 2)
        // Slot 2: misplaced (3 is in secret at pos 3)
        // Slot 3: wrong (4 not in secret)
        let feedback = calculate_feedback(&[0, 2, 3, 4], &[0, 1, 2, 3]);
        assert_eq!(feedback, vec![
            FeedbackMark::Exact,
            FeedbackMark::Misplaced,
            FeedbackMark::Misplaced,
            FeedbackMark::Wrong,
        ]);
    }

    #[test]
    fn test_feedback_duplicate_in_guess_with_single_in_secret() {
        // Secret: [0, 1, 2]
        // Guess:  [0, 0, 0]
        // Slot 0: exact (0==0)
        // Slot 1: wrong (0 already matched)
        // Slot 2: wrong (0 already matched)
        let feedback = calculate_feedback(&[0, 0, 0], &[0, 1, 2]);
        assert_eq!(feedback, vec![
            FeedbackMark::Exact,
            FeedbackMark::Wrong,
            FeedbackMark::Wrong,
        ]);
    }

    #[test]
    fn test_submit_guess_win() {
        let mut game = RuneGame::new(crate::rune::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        generate_code(&mut game, &mut rng);

        let code = game.secret_code.clone();
        for (i, &r) in code.iter().enumerate() {
            game.current_guess[i] = Some(r);
        }

        let accepted = submit_guess(&mut game, &mut rng);
        assert!(accepted);
        assert_eq!(game.game_result, Some(RuneResult::Win));
    }

    #[test]
    fn test_submit_guess_loss_after_max() {
        let mut game = RuneGame::new(crate::rune::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        game.secret_code = vec![0, 1, 2]; // Set known code

        // Submit wrong guesses until max
        for _ in 0..10 {
            game.current_guess = vec![Some(3), Some(4), Some(0)];
            submit_guess(&mut game, &mut rng);
            if game.game_result.is_some() {
                break;
            }
        }

        assert_eq!(game.game_result, Some(RuneResult::Loss));
    }

    #[test]
    fn test_submit_incomplete_guess_rejected() {
        let mut game = RuneGame::new(crate::rune::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        game.current_guess[0] = Some(0);
        // Slots 1 and 2 are None

        let accepted = submit_guess(&mut game, &mut rng);
        assert!(!accepted);
    }

    #[test]
    fn test_submit_duplicate_rejected_in_no_dupe_mode() {
        let mut game = RuneGame::new(crate::rune::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        game.current_guess = vec![Some(0), Some(0), Some(1)]; // duplicate 0

        let accepted = submit_guess(&mut game, &mut rng);
        assert!(!accepted);
    }
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test --lib rune_logic::tests -v`
Expected: All tests PASS

**Step 3: Commit**

```bash
git add src/rune_logic.rs
git commit -m "feat(rune): add game logic - code generation, feedback, guess submission"
```

---

### Task 3: Challenge Menu Integration (`src/challenge_menu.rs`)

**Files:**
- Modify: `src/challenge_menu.rs`

**Step 1: Add `Rune` to `ChallengeType` enum**

In `src/challenge_menu.rs`, add to `ChallengeType` enum (after line 209 `Minesweeper,`):

```rust
    Rune,
```

**Step 2: Add `RuneDifficulty` import**

At top of `src/challenge_menu.rs`, add after the minesweeper import (line 10):

```rust
use crate::rune::RuneDifficulty;
```

**Step 3: Add `DifficultyInfo` impl for `RuneDifficulty`**

After the `MinesweeperDifficulty` impl block (after line 161), add:

```rust
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
        let dupes = if self.allow_duplicates() { ", dupes" } else { "" };
        Some(format!("{} runes, {} slots{}", self.num_runes(), self.num_slots(), dupes))
    }
}
```

**Step 4: Add to weighted distribution table**

In `CHALLENGE_TABLE` (after line 191), add:

```rust
    ChallengeWeight {
        challenge_type: ChallengeType::Rune,
        weight: 25,
    },
```

**Step 5: Add to `create_challenge` function**

In `create_challenge` match (after the Minesweeper arm, before the closing `}`), add:

```rust
        ChallengeType::Rune => PendingChallenge {
            challenge_type: ChallengeType::Rune,
            title: "Rune Deciphering: Ancient Tablet".to_string(),
            icon: "ᚱ",
            description: "You stumble upon a stone tablet covered in glowing runes. \
                A spectral voice echoes: 'Decipher the hidden sequence, mortal. \
                Each attempt reveals clues—exact matches, misplaced symbols, or \
                false leads. Prove your logic worthy of ancient knowledge.'"
                .to_string(),
        },
```

**Step 6: Add `active_rune` check to `try_discover_challenge`**

In `try_discover_challenge` (around line 301), add after `|| state.active_minesweeper.is_some()`:

```rust
        || state.active_rune.is_some()
```

**Step 7: Add to discovery log in `main.rs`**

In `main.rs` challenge discovery match (around line 1215), add after the Minesweeper arm:

```rust
                ChallengeType::Rune => (
                    "ᚱ",
                    "A glowing stone tablet materializes before you...",
                ),
```

**Step 8: Add test for reward structure**

Add to tests in `src/rune.rs`:

```rust
    #[test]
    fn test_reward_structure() {
        use crate::challenge_menu::DifficultyInfo;

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
```

**Step 9: Run tests**

Run: `cargo test --lib 2>&1 | tail -5`
Expected: All tests PASS (compilation may fail until game_state is updated — do that in next task)

**Step 10: Commit**

```bash
git add src/challenge_menu.rs src/rune.rs
git commit -m "feat(rune): integrate with challenge menu - rewards, discovery, narrative"
```

---

### Task 4: Game State Integration (`src/game_state.rs`)

**Files:**
- Modify: `src/game_state.rs`

**Step 1: Add import**

At top of `src/game_state.rs`, add the rune import alongside existing challenge imports:

```rust
use crate::rune::RuneGame;
```

**Step 2: Add field to `GameState` struct**

After line 58 (`pub active_minesweeper: Option<MinesweeperGame>,`), add:

```rust
    /// Active rune game (transient, not saved)
    #[serde(skip)]
    pub active_rune: Option<RuneGame>,
```

**Step 3: Initialize in `GameState::new()`**

After line 91 (`active_minesweeper: None,`), add:

```rust
            active_rune: None,
```

**Step 4: Run tests**

Run: `cargo test --lib 2>&1 | tail -5`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/game_state.rs
git commit -m "feat(rune): add active_rune field to GameState"
```

---

### Task 5: Main.rs Integration (Module Declarations, Input Handling, Accept Block)

**Files:**
- Modify: `src/main.rs`

**Step 1: Add module declarations**

After line 29 (`mod minesweeper_logic;`), add:

```rust
mod rune;
mod rune_logic;
```

**Step 2: Add use statements**

After line 57 (`use minesweeper_logic::{handle_first_click, reveal_cell, toggle_flag};`), add:

```rust
use rune::{RuneDifficulty, RuneGame, RuneResult};
use rune_logic::submit_guess;
```

**Step 3: Add input handling block**

Before the minesweeper input handling (before line 695 `// Handle active minesweeper game input`), add the rune input handling block. Follow the exact same pattern as minesweeper:

```rust
                            // Handle active rune game input
                            if let Some(ref mut rune_game) = state.active_rune {
                                if let Some(result) = rune_game.game_result {
                                    // Any key dismisses result and applies rewards
                                    if result == RuneResult::Win {
                                        let reward = rune_game.difficulty.reward();
                                        if reward.xp_percent > 0 {
                                            let xp_for_level = game_logic::xp_for_next_level(
                                                state.character_level.max(1),
                                            );
                                            let xp_gain =
                                                (xp_for_level * reward.xp_percent as u64) / 100;
                                            state.character_xp += xp_gain;
                                        }
                                        if reward.prestige_ranks > 0 {
                                            state.prestige_rank += reward.prestige_ranks;
                                        }
                                        if reward.fishing_ranks > 0 {
                                            state.fishing.rank = state
                                                .fishing
                                                .rank
                                                .saturating_add(reward.fishing_ranks);
                                        }
                                    }
                                    state.active_rune = None;
                                    continue;
                                }

                                // Handle forfeit confirmation (double-Esc)
                                if rune_game.forfeit_pending {
                                    match key_event.code {
                                        KeyCode::Esc => {
                                            rune_game.game_result = Some(RuneResult::Loss);
                                        }
                                        _ => {
                                            rune_game.forfeit_pending = false;
                                        }
                                    }
                                    continue;
                                }

                                // Normal game input
                                match key_event.code {
                                    KeyCode::Left => rune_game.move_cursor_left(),
                                    KeyCode::Right => rune_game.move_cursor_right(),
                                    KeyCode::Up => rune_game.cycle_rune_up(),
                                    KeyCode::Down => rune_game.cycle_rune_down(),
                                    KeyCode::Enter => {
                                        let mut rng = rand::thread_rng();
                                        submit_guess(rune_game, &mut rng);
                                    }
                                    KeyCode::Char('f') | KeyCode::Char('F') => {
                                        rune_game.clear_guess();
                                    }
                                    KeyCode::Esc => {
                                        rune_game.forfeit_pending = true;
                                    }
                                    _ => {}
                                }
                                continue;
                            }
```

**Step 4: Add challenge accept block**

In the challenge menu accept match (after the Minesweeper arm around line 1075), add:

```rust
                                                    ChallengeType::Rune => {
                                                        let difficulty =
                                                            RuneDifficulty::from_index(
                                                                menu.selected_difficulty,
                                                            );
                                                        state.active_rune =
                                                            Some(RuneGame::new(difficulty));
                                                    }
```

**Step 5: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`
Expected: Successful build (UI scene not wired yet, but no compilation errors)

**Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat(rune): wire input handling and challenge accept in main.rs"
```

---

### Task 6: UI Scene (`src/ui/rune_scene.rs`)

**Files:**
- Create: `src/ui/rune_scene.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Create rune_scene.rs**

```rust
//! Rune Deciphering game UI rendering.

use crate::rune::{FeedbackMark, RuneGame, RuneResult, RUNE_SYMBOLS};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the rune deciphering game scene.
pub fn render_rune(frame: &mut Frame, area: Rect, game: &RuneGame) {
    frame.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),
            Constraint::Length(24),
        ])
        .split(area);

    render_grid(frame, chunks[0], game);
    render_info_panel(frame, chunks[1], game);

    if game.game_result.is_some() {
        render_game_over_overlay(frame, chunks[0], game);
    }
}

/// Render guess history and current input.
fn render_grid(frame: &mut Frame, area: Rect, game: &RuneGame) {
    let block = Block::default()
        .title(" Rune Deciphering ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut y = inner.y;

    // Render submitted guesses
    for (i, guess) in game.guesses.iter().enumerate() {
        if y >= inner.y + inner.height {
            break;
        }
        let mut spans = Vec::new();
        spans.push(Span::styled(
            format!("{:>2}: ", i + 1),
            Style::default().fg(Color::DarkGray),
        ));

        for &rune_idx in &guess.runes {
            let ch = RUNE_SYMBOLS[rune_idx];
            spans.push(Span::styled(
                format!("{} ", ch),
                Style::default().fg(Color::White),
            ));
        }

        spans.push(Span::raw("  "));

        for mark in &guess.feedback {
            let (sym, color) = match mark {
                FeedbackMark::Exact => ("●", Color::Green),
                FeedbackMark::Misplaced => ("○", Color::Yellow),
                FeedbackMark::Wrong => ("·", Color::DarkGray),
            };
            spans.push(Span::styled(format!("{} ", sym), Style::default().fg(color)));
        }

        let line = Paragraph::new(Line::from(spans));
        frame.render_widget(line, Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 1));
        y += 1;
    }

    // Blank line separator
    if !game.guesses.is_empty() && game.game_result.is_none() {
        y += 1;
    }

    // Render current guess input (only if game not over)
    if game.game_result.is_none() && y < inner.y + inner.height {
        let mut spans = Vec::new();
        spans.push(Span::styled(
            format!("{:>2}: ", game.guesses.len() + 1),
            Style::default().fg(Color::DarkGray),
        ));

        for (i, slot) in game.current_guess.iter().enumerate() {
            let is_cursor = i == game.cursor_slot;
            let text = match slot {
                Some(idx) => format!("{} ", RUNE_SYMBOLS[*idx]),
                None => "_ ".to_string(),
            };
            let mut style = Style::default().fg(Color::Cyan);
            if is_cursor {
                style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
            }
            spans.push(Span::styled(text, style));
        }

        let line = Paragraph::new(Line::from(spans));
        frame.render_widget(line, Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 1));
        y += 2;
    }

    // Available runes
    if game.game_result.is_none() && y < inner.y + inner.height {
        let mut spans = vec![Span::styled(
            "Runes: ",
            Style::default().fg(Color::DarkGray),
        )];
        for i in 0..game.num_runes {
            spans.push(Span::styled(
                format!("{} ", RUNE_SYMBOLS[i]),
                Style::default().fg(Color::White),
            ));
        }
        let line = Paragraph::new(Line::from(spans));
        frame.render_widget(line, Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 1));
    }
}

/// Render the info panel on the right side.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &RuneGame) {
    let block = Block::default()
        .title(" Info ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "Rune Deciphering",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Runes: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", game.num_runes),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Slots: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", game.num_slots),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Guesses: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} remaining", game.guesses_remaining()),
                Style::default().fg(if game.guesses_remaining() <= 2 {
                    Color::Red
                } else {
                    Color::White
                }),
            ),
        ]),
    ];

    if game.allow_duplicates {
        lines.push(Line::from(Span::styled(
            "Duplicates: Yes",
            Style::default().fg(Color::Yellow),
        )));
    }

    lines.push(Line::from(""));

    // Status
    let status = if game.game_result.is_some() {
        Span::styled("", Style::default())
    } else if game.forfeit_pending {
        Span::styled("Forfeit game?", Style::default().fg(Color::LightRed))
    } else if game.guesses.is_empty() {
        Span::styled("Begin deciphering", Style::default().fg(Color::Yellow))
    } else {
        Span::styled("Deciphering...", Style::default().fg(Color::Green))
    };
    lines.push(Line::from(status));
    lines.push(Line::from(""));

    // Controls
    if game.game_result.is_none() {
        if game.forfeit_pending {
            lines.push(Line::from(Span::styled(
                "[Esc] Confirm forfeit",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                "[Any] Cancel",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "[←→] Move slot",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                "[↑↓] Cycle rune",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                "[Enter] Submit guess",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                "[F] Clear guess",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                "[Esc] Forfeit",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    let text = Paragraph::new(lines);
    frame.render_widget(text, inner);
}

/// Render the game over overlay.
fn render_game_over_overlay(frame: &mut Frame, area: Rect, game: &RuneGame) {
    let result = game.game_result.as_ref().unwrap();

    let (title, color) = match result {
        RuneResult::Win => ("Runes Deciphered!", Color::Green),
        RuneResult::Loss => ("Runes Remain Hidden", Color::Red),
    };

    let mut overlay_lines = vec![
        Line::from(Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // Show secret code on loss
    if *result == RuneResult::Loss {
        let mut code_spans = vec![Span::styled("Code: ", Style::default().fg(Color::DarkGray))];
        for &idx in &game.secret_code {
            code_spans.push(Span::styled(
                format!("{} ", RUNE_SYMBOLS[idx]),
                Style::default().fg(Color::White),
            ));
        }
        overlay_lines.push(Line::from(code_spans));
    }

    // Reward text
    use crate::challenge_menu::DifficultyInfo;
    let reward_text = if *result == RuneResult::Win {
        game.difficulty.reward().description()
    } else {
        "No reward".to_string()
    };
    overlay_lines.push(Line::from(Span::styled(
        reward_text,
        Style::default().fg(Color::White),
    )));

    overlay_lines.push(Line::from(Span::styled(
        "[Any key to continue]",
        Style::default().fg(Color::DarkGray),
    )));

    let height = overlay_lines.len() as u16 + 2; // +2 for borders
    let width = 30;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let overlay_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));
    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let text = Paragraph::new(overlay_lines).alignment(Alignment::Center);
    frame.render_widget(text, inner);
}
```

**Step 2: Add to ui/mod.rs**

After line 15 (`pub mod minesweeper_scene;`), add:

```rust
pub mod rune_scene;
```

In the render dispatch (after line 85 `minesweeper_scene::render_minesweeper(...)` block), add rune as highest priority (before minesweeper):

```rust
    if let Some(ref game) = game_state.active_rune {
        rune_scene::render_rune(frame, chunks[1], game);
    } else if let Some(ref game) = game_state.active_minesweeper {
```

In the banner check (line 46), add:

```rust
        && game_state.active_rune.is_none()
```

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`
Expected: Successful build

**Step 4: Commit**

```bash
git add src/ui/rune_scene.rs src/ui/mod.rs
git commit -m "feat(rune): add game UI scene with grid, info panel, and overlay"
```

---

### Task 7: Debug Menu & Challenge Menu Scene

**Files:**
- Modify: `src/debug_menu.rs`
- Modify: `src/ui/challenge_menu_scene.rs`

**Step 1: Add to debug menu**

In `src/debug_menu.rs`, add `"Trigger Rune Challenge"` to `DEBUG_OPTIONS` array (after `"Trigger Minesweeper Challenge"`).

Update `trigger_selected` match to add index 6:

```rust
            6 => trigger_rune_challenge(state),
```

Add trigger function:

```rust
fn trigger_rune_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Rune) {
        return "Rune challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Rune));
    "Rune challenge added!"
}
```

**Step 2: Add to challenge menu scene**

In `src/ui/challenge_menu_scene.rs`, add the Rune difficulty selector in the match block (after the Minesweeper arm):

```rust
        ChallengeType::Rune => {
            render_difficulty_selector(
                frame,
                chunks[2],
                &RuneDifficulty::ALL,
                menu.selected_difficulty,
            );
        }
```

Add the import at the top:

```rust
use crate::rune::RuneDifficulty;
```

**Step 3: Update debug menu test for navigation bounds**

In `debug_menu.rs` tests, update `test_menu_navigation` to navigate down to index 6 (instead of 5) and check bounds at 6.

**Step 4: Add debug menu trigger test**

```rust
    #[test]
    fn test_trigger_rune_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_rune_challenge(&mut state);
        assert_eq!(msg, "Rune challenge added!");
        assert!(state.challenge_menu.has_challenge(&ChallengeType::Rune));

        let msg = trigger_rune_challenge(&mut state);
        assert_eq!(msg, "Rune challenge already pending!");
    }
```

**Step 5: Run all tests**

Run: `cargo test 2>&1 | tail -5`
Expected: All tests PASS

**Step 6: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | tail -10`
Expected: No warnings

**Step 7: Commit**

```bash
git add src/debug_menu.rs src/ui/challenge_menu_scene.rs
git commit -m "feat(rune): add debug menu trigger and challenge menu difficulty selector"
```

---

### Task 8: Final Quality Checks

**Files:** None (verification only)

**Step 1: Run full CI checks**

Run: `make check`
Expected: All 5 checks pass (format, clippy, test, build, audit)

**Step 2: Fix any issues**

If formatting fails: `make fmt` then re-run `make check`.

**Step 3: Final commit if needed**

```bash
git add -A
git commit -m "chore(rune): formatting and cleanup"
```

---

### Integration Point Summary

| File | Change |
|------|--------|
| `src/rune.rs` | NEW — Data structures, difficulty, game state |
| `src/rune_logic.rs` | NEW — Code generation, feedback, guess submission |
| `src/ui/rune_scene.rs` | NEW — Grid, info panel, game over overlay |
| `src/lib.rs` | Add `pub mod rune; pub mod rune_logic;` |
| `src/game_state.rs` | Add `active_rune: Option<RuneGame>` field + init |
| `src/challenge_menu.rs` | Add `Rune` variant, DifficultyInfo impl, rewards, weight, create_challenge, discovery check |
| `src/main.rs` | Add mod/use, input handling block, challenge accept block, discovery log |
| `src/ui/mod.rs` | Add `pub mod rune_scene`, render dispatch, banner check |
| `src/debug_menu.rs` | Add option + trigger function |
| `src/ui/challenge_menu_scene.rs` | Add Rune difficulty selector |
