//! Rune Deciphering game logic.
//!
//! Handles secret code generation, feedback calculation, and guess submission.

use super::{FeedbackMark, RuneDifficulty, RuneGame, RuneGuess, RuneResult};
use crate::challenges::ActiveMinigame;
use rand::{Rng, RngExt};

/// Input actions for the Rune game (UI-agnostic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuneInput {
    Left,
    Right,
    Up,
    Down,
    Submit,
    ClearGuess,
    Forfeit,
    Other,
}

/// Process a key input during active Rune game.
/// Returns true if the input was handled (game should continue processing).
pub fn process_input<R: Rng>(game: &mut RuneGame, input: RuneInput, rng: &mut R) -> bool {
    // Handle forfeit confirmation (double-Esc pattern)
    if game.forfeit_pending {
        match input {
            RuneInput::Forfeit => {
                game.game_result = Some(RuneResult::Loss);
            }
            _ => {
                game.forfeit_pending = false;
            }
        }
        return true;
    }

    // Normal game input
    match input {
        RuneInput::Left => game.move_cursor_left(),
        RuneInput::Right => game.move_cursor_right(),
        RuneInput::Up => game.cycle_rune_up(),
        RuneInput::Down => game.cycle_rune_down(),
        RuneInput::Submit => {
            submit_guess(game, rng);
        }
        RuneInput::ClearGuess => {
            game.clear_guess();
        }
        RuneInput::Forfeit => {
            game.forfeit_pending = true;
        }
        RuneInput::Other => {}
    }
    true
}

/// Generate the secret code for a rune game.
pub fn generate_code<R: Rng>(game: &mut RuneGame, rng: &mut R) {
    if game.allow_duplicates {
        game.secret_code = (0..game.num_slots)
            .map(|_| rng.random_range(0..game.num_runes))
            .collect();
    } else {
        // Sample without replacement using Fisher-Yates partial shuffle
        let mut pool: Vec<usize> = (0..game.num_runes).collect();
        for i in 0..game.num_slots {
            let j = rng.random_range(i..pool.len());
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
                game.reject_message = Some("No duplicate runes!".to_string());
                return false;
            }
        }
    }

    game.reject_message = None;

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

/// Apply game result: update stats, grant rewards, and add combat log entries.
/// Returns Some(MinigameWinInfo) if the player won, None otherwise.
pub fn apply_game_result(
    state: &mut crate::core::game_state::GameState,
) -> Option<crate::challenges::MinigameWinInfo> {
    use crate::challenges::menu::DifficultyInfo;
    use crate::challenges::{apply_challenge_rewards, GameResultInfo};

    let game = match state.active_minigame.as_ref() {
        Some(ActiveMinigame::Rune(g)) => g,
        _ => return None,
    };
    let result = game.game_result?;
    let difficulty = game.difficulty;
    let reward = difficulty.reward();

    let (won, loss_message) = match result {
        RuneResult::Win => (true, ""),
        RuneResult::Loss => (false, "The tablet fades. The code remains a mystery."),
    };

    apply_challenge_rewards(
        state,
        GameResultInfo {
            won,
            game_type: "rune",
            difficulty_str: difficulty.difficulty_str(),
            reward,
            icon: "\u{16B1}",
            win_message: "The runes glow with approval! Code deciphered.",
            loss_message,
        },
    )
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
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        generate_code(&mut game, &mut rng);

        assert_eq!(game.secret_code.len(), 3);
        assert!(game.secret_code.iter().all(|&r| r < 5));
        let mut sorted = game.secret_code.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), game.secret_code.len());
    }

    #[test]
    fn test_generate_code_with_dupes() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Master);
        let mut rng = seeded_rng();
        generate_code(&mut game, &mut rng);

        assert_eq!(game.secret_code.len(), 5);
        assert!(game.secret_code.iter().all(|&r| r < 8));
    }

    #[test]
    fn test_feedback_all_exact() {
        let feedback = calculate_feedback(&[0, 1, 2], &[0, 1, 2]);
        assert_eq!(
            feedback,
            vec![
                FeedbackMark::Exact,
                FeedbackMark::Exact,
                FeedbackMark::Exact
            ]
        );
    }

    #[test]
    fn test_feedback_all_wrong() {
        let feedback = calculate_feedback(&[0, 1, 2], &[3, 4, 5]);
        assert_eq!(
            feedback,
            vec![
                FeedbackMark::Wrong,
                FeedbackMark::Wrong,
                FeedbackMark::Wrong
            ]
        );
    }

    #[test]
    fn test_feedback_all_misplaced() {
        let feedback = calculate_feedback(&[0, 1, 2], &[2, 0, 1]);
        assert_eq!(
            feedback,
            vec![
                FeedbackMark::Misplaced,
                FeedbackMark::Misplaced,
                FeedbackMark::Misplaced,
            ]
        );
    }

    #[test]
    fn test_feedback_mixed() {
        let feedback = calculate_feedback(&[0, 2, 3, 4], &[0, 1, 2, 3]);
        assert_eq!(
            feedback,
            vec![
                FeedbackMark::Exact,
                FeedbackMark::Misplaced,
                FeedbackMark::Misplaced,
                FeedbackMark::Wrong,
            ]
        );
    }

    #[test]
    fn test_feedback_duplicate_in_guess_with_single_in_secret() {
        let feedback = calculate_feedback(&[0, 0, 0], &[0, 1, 2]);
        assert_eq!(
            feedback,
            vec![
                FeedbackMark::Exact,
                FeedbackMark::Wrong,
                FeedbackMark::Wrong,
            ]
        );
    }

    #[test]
    fn test_submit_guess_win() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
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
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        game.secret_code = vec![0, 1, 2];

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
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        game.current_guess[0] = Some(0);

        let accepted = submit_guess(&mut game, &mut rng);
        assert!(!accepted);
    }

    #[test]
    fn test_submit_duplicate_rejected_in_no_dupe_mode() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        game.current_guess = vec![Some(0), Some(0), Some(1)];

        let accepted = submit_guess(&mut game, &mut rng);
        assert!(!accepted);
        assert_eq!(game.reject_message, Some("No duplicate runes!".to_string()));
        // Guess list should not grow on rejection
        assert!(game.guesses.is_empty());
    }

    #[test]
    fn test_submit_duplicates_accepted_in_dupe_mode() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Journeyman);
        let mut rng = seeded_rng();
        game.secret_code = vec![0, 1, 2, 3];
        game.current_guess = vec![Some(0), Some(0), Some(0), Some(0)];

        let accepted = submit_guess(&mut game, &mut rng);
        assert!(accepted);
        assert_eq!(game.guesses.len(), 1);
    }

    #[test]
    fn test_feedback_duplicate_in_secret() {
        // Guess [0, 1] vs Secret [0, 0] — first exact, second wrong (not in secret twice for matching)
        let feedback = calculate_feedback(&[0, 1, 2], &[0, 0, 3]);
        assert_eq!(
            feedback,
            vec![
                FeedbackMark::Exact,
                FeedbackMark::Wrong,
                FeedbackMark::Wrong,
            ]
        );
    }

    #[test]
    fn test_feedback_double_in_both() {
        // Guess [0, 0, 1] vs Secret [0, 0, 2] — two exact, one wrong
        let feedback = calculate_feedback(&[0, 0, 1], &[0, 0, 2]);
        assert_eq!(
            feedback,
            vec![
                FeedbackMark::Exact,
                FeedbackMark::Exact,
                FeedbackMark::Wrong,
            ]
        );
    }

    #[test]
    fn test_feedback_misplaced_with_duplicates() {
        // Guess [0, 1, 0] vs Secret [1, 0, 2] — no exact, two misplaced, one wrong
        let feedback = calculate_feedback(&[0, 1, 0], &[1, 0, 2]);
        // Sorted: Misplaced, Misplaced, Wrong
        assert_eq!(
            feedback,
            vec![
                FeedbackMark::Misplaced,
                FeedbackMark::Misplaced,
                FeedbackMark::Wrong,
            ]
        );
    }

    #[test]
    fn test_feedback_sorting_order() {
        // Verify feedback is always sorted: Exact, Misplaced, Wrong
        let feedback = calculate_feedback(&[0, 3, 1, 4], &[0, 1, 3, 5]);
        // Slot 0: Exact(0=0), Slot 1: Misplaced(3 in secret), Slot 2: Misplaced(1 in secret), Slot 3: Wrong
        assert_eq!(feedback[0], FeedbackMark::Exact);
        assert_eq!(feedback[1], FeedbackMark::Misplaced);
        assert_eq!(feedback[2], FeedbackMark::Misplaced);
        assert_eq!(feedback[3], FeedbackMark::Wrong);
    }

    #[test]
    fn test_feedback_five_slots() {
        // Master difficulty: 5 slots
        let feedback = calculate_feedback(&[0, 1, 2, 3, 4], &[0, 1, 2, 3, 4]);
        assert_eq!(feedback.len(), 5);
        assert!(feedback.iter().all(|m| *m == FeedbackMark::Exact));
    }

    #[test]
    fn test_generate_code_all_difficulties() {
        for &diff in &super::super::RuneDifficulty::ALL {
            let mut game = RuneGame::new(diff);
            let mut rng = seeded_rng();
            generate_code(&mut game, &mut rng);

            assert_eq!(game.secret_code.len(), game.num_slots);
            assert!(game.secret_code.iter().all(|&r| r < game.num_runes));

            if !game.allow_duplicates {
                let mut sorted = game.secret_code.clone();
                sorted.sort();
                sorted.dedup();
                assert_eq!(sorted.len(), game.secret_code.len());
            }
        }
    }

    #[test]
    fn test_lazy_code_generation() {
        // Secret code should be generated on first valid submit, not before
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        assert!(game.secret_code.is_empty());

        game.current_guess = vec![Some(0), Some(1), Some(2)];
        submit_guess(&mut game, &mut rng);
        assert!(!game.secret_code.is_empty());
    }

    #[test]
    fn test_win_on_last_guess() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        game.secret_code = vec![0, 1, 2];

        // Use 9 wrong guesses
        for _ in 0..9 {
            game.current_guess = vec![Some(3), Some(4), Some(0)];
            submit_guess(&mut game, &mut rng);
        }
        assert!(game.game_result.is_none());
        assert_eq!(game.guesses_remaining(), 1);

        // 10th guess is correct — should be Win, not Loss
        game.current_guess = vec![Some(0), Some(1), Some(2)];
        let accepted = submit_guess(&mut game, &mut rng);
        assert!(accepted);
        assert_eq!(game.game_result, Some(RuneResult::Win));
    }

    #[test]
    fn test_submit_after_win_rejected() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        game.secret_code = vec![0, 1, 2];
        game.current_guess = vec![Some(0), Some(1), Some(2)];
        submit_guess(&mut game, &mut rng);
        assert_eq!(game.game_result, Some(RuneResult::Win));

        // Try to submit again
        game.current_guess = vec![Some(3), Some(4), Some(0)];
        let accepted = submit_guess(&mut game, &mut rng);
        assert!(!accepted);
        assert_eq!(game.guesses.len(), 1);
    }

    #[test]
    fn test_submit_after_loss_rejected() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        game.secret_code = vec![0, 1, 2];

        for _ in 0..10 {
            game.current_guess = vec![Some(3), Some(4), Some(0)];
            submit_guess(&mut game, &mut rng);
        }
        assert_eq!(game.game_result, Some(RuneResult::Loss));

        game.current_guess = vec![Some(0), Some(1), Some(2)];
        let accepted = submit_guess(&mut game, &mut rng);
        assert!(!accepted);
        assert_eq!(game.guesses.len(), 10);
    }

    #[test]
    fn test_reject_message_cleared_on_valid_submit() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        game.secret_code = vec![0, 1, 2];

        // First: trigger reject with duplicates
        game.current_guess = vec![Some(0), Some(0), Some(1)];
        submit_guess(&mut game, &mut rng);
        assert!(game.reject_message.is_some());

        // Then: valid submit should clear it
        game.current_guess = vec![Some(0), Some(1), Some(3)];
        let accepted = submit_guess(&mut game, &mut rng);
        assert!(accepted);
        assert!(game.reject_message.is_none());
    }

    #[test]
    fn test_reject_preserves_state() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();

        // Submit a valid guess first
        game.current_guess = vec![Some(0), Some(1), Some(2)];
        submit_guess(&mut game, &mut rng);
        assert_eq!(game.guesses.len(), 1);

        // Try to submit duplicates — should not change guess count
        game.current_guess = vec![Some(0), Some(0), Some(1)];
        let accepted = submit_guess(&mut game, &mut rng);
        assert!(!accepted);
        assert_eq!(game.guesses.len(), 1);
        // Current guess should be unchanged
        assert_eq!(game.current_guess, vec![Some(0), Some(0), Some(1)]);
    }

    // ============ apply_game_result Tests ============

    #[test]
    fn test_apply_win_result_with_xp() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        let initial_xp = state.character_xp;
        let initial_fishing = state.fishing.rank;

        // Journeyman gives XP and fishing ranks
        let mut game = RuneGame::new(super::super::RuneDifficulty::Journeyman);
        game.game_result = Some(RuneResult::Win);
        state.active_minigame = Some(ActiveMinigame::Rune(game));

        let processed = apply_game_result(&mut state);
        assert!(processed.is_some()); // Win returns Some(MinigameWinInfo)
        assert!(state.character_xp > initial_xp); // Journeyman gives 75% XP
        assert!(state.fishing.rank > initial_fishing); // Journeyman gives fishing ranks
        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_apply_win_result_with_prestige() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;
        let initial_fishing = state.fishing.rank;

        // Master gives prestige and fishing ranks (no XP)
        let mut game = RuneGame::new(super::super::RuneDifficulty::Master);
        game.game_result = Some(RuneResult::Win);
        state.active_minigame = Some(ActiveMinigame::Rune(game));

        let processed = apply_game_result(&mut state);
        assert!(processed.is_some()); // Win returns Some(MinigameWinInfo)
        assert!(state.prestige_rank > 5); // Master gives prestige
        assert!(state.fishing.rank > initial_fishing); // Master gives fishing ranks
        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_apply_loss_result() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;
        let initial_xp = state.character_xp;

        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        game.game_result = Some(RuneResult::Loss);
        state.active_minigame = Some(ActiveMinigame::Rune(game));

        let processed = apply_game_result(&mut state);
        assert!(processed.is_none()); // Loss returns None
        assert_eq!(state.character_xp, initial_xp); // XP unchanged
        assert_eq!(state.prestige_rank, 5); // Prestige unchanged
        assert!(state.active_minigame.is_none()); // But game is still cleared
    }

    #[test]
    fn test_apply_result_no_game() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        state.active_minigame = None;

        let processed = apply_game_result(&mut state);
        assert!(processed.is_none());
    }

    #[test]
    fn test_apply_result_no_result() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        let game = RuneGame::new(super::super::RuneDifficulty::Novice);
        // game.game_result is None
        state.active_minigame = Some(ActiveMinigame::Rune(game));

        let processed = apply_game_result(&mut state);
        assert!(processed.is_none());
        // Game should still be active
        assert!(matches!(
            state.active_minigame,
            Some(ActiveMinigame::Rune(_))
        ));
    }

    #[test]
    fn test_loss_at_max_guesses_journeyman() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Journeyman);
        let mut rng = seeded_rng();
        game.secret_code = vec![0, 1, 2, 3];

        for _ in 0..8 {
            game.current_guess = vec![Some(4), Some(5), Some(4), Some(5)];
            submit_guess(&mut game, &mut rng);
        }
        assert_eq!(game.game_result, Some(RuneResult::Loss));
        assert_eq!(game.guesses.len(), 8);
    }

    #[test]
    fn test_full_game_sequence() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Apprentice);
        let mut rng = seeded_rng();
        game.secret_code = vec![0, 1, 2, 3];

        // Guess 1: all wrong, unique runes (Apprentice = no dupes)
        game.current_guess = vec![Some(4), Some(5), Some(4), Some(5)];
        assert!(!submit_guess(&mut game, &mut rng)); // Rejected: has duplicates

        game.current_guess = vec![Some(4), Some(5), Some(3), Some(0)];
        submit_guess(&mut game, &mut rng);
        assert_eq!(game.guesses.len(), 1);
        assert!(game.game_result.is_none());
        // Rune 0 is misplaced (in secret at pos 0, guessed at pos 3), rest wrong
        let exact_count = game.guesses[0]
            .feedback
            .iter()
            .filter(|m| **m == FeedbackMark::Exact)
            .count();
        assert_eq!(exact_count, 0);

        // Guess 2: two misplaced (swap 0 and 1)
        game.current_guess = vec![Some(1), Some(0), Some(4), Some(5)];
        submit_guess(&mut game, &mut rng);
        assert_eq!(game.guesses.len(), 2);
        let misplaced_count = game.guesses[1]
            .feedback
            .iter()
            .filter(|m| **m == FeedbackMark::Misplaced)
            .count();
        assert_eq!(misplaced_count, 2);

        // Guess 3: correct
        game.current_guess = vec![Some(0), Some(1), Some(2), Some(3)];
        submit_guess(&mut game, &mut rng);
        assert_eq!(game.game_result, Some(RuneResult::Win));
        assert_eq!(game.guesses.len(), 3);
        assert_eq!(game.guesses_remaining(), 7);
    }

    // ============ process_input Tests ============

    #[test]
    fn test_process_input_cursor_movement() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();

        assert_eq!(game.cursor_slot, 0);

        process_input(&mut game, RuneInput::Right, &mut rng);
        assert_eq!(game.cursor_slot, 1);

        process_input(&mut game, RuneInput::Left, &mut rng);
        assert_eq!(game.cursor_slot, 0);
    }

    #[test]
    fn test_process_input_rune_cycling() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();

        // Initial state: no rune selected
        assert_eq!(game.current_guess[0], None);

        // Cycle up to select first rune
        process_input(&mut game, RuneInput::Up, &mut rng);
        assert_eq!(game.current_guess[0], Some(0));

        // Cycle up again
        process_input(&mut game, RuneInput::Up, &mut rng);
        assert_eq!(game.current_guess[0], Some(1));

        // Cycle down
        process_input(&mut game, RuneInput::Down, &mut rng);
        assert_eq!(game.current_guess[0], Some(0));
    }

    #[test]
    fn test_process_input_clear_guess() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();

        // Set up a partial guess
        game.current_guess = vec![Some(0), Some(1), None];

        process_input(&mut game, RuneInput::ClearGuess, &mut rng);

        // All slots should be cleared
        assert!(game.current_guess.iter().all(|s| s.is_none()));
    }

    #[test]
    fn test_process_input_forfeit_single_esc() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();

        assert!(!game.forfeit_pending);

        process_input(&mut game, RuneInput::Forfeit, &mut rng);

        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none()); // Not forfeited yet
    }

    #[test]
    fn test_process_input_forfeit_double_esc() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();

        // First Esc sets pending
        process_input(&mut game, RuneInput::Forfeit, &mut rng);
        assert!(game.forfeit_pending);

        // Second Esc confirms forfeit
        process_input(&mut game, RuneInput::Forfeit, &mut rng);

        assert_eq!(game.game_result, Some(RuneResult::Loss));
    }

    #[test]
    fn test_process_input_forfeit_cancelled() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();

        // First Esc sets pending
        process_input(&mut game, RuneInput::Forfeit, &mut rng);
        assert!(game.forfeit_pending);

        // Any other key cancels forfeit
        process_input(&mut game, RuneInput::Other, &mut rng);

        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_process_input_submit() {
        let mut game = RuneGame::new(super::super::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        game.secret_code = vec![0, 1, 2];

        // Set up complete guess
        game.current_guess = vec![Some(0), Some(1), Some(2)];

        process_input(&mut game, RuneInput::Submit, &mut rng);

        assert_eq!(game.game_result, Some(RuneResult::Win));
    }
}
