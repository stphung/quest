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
        assert!(game.secret_code.iter().all(|&r| r < 5));
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
        let mut game = RuneGame::new(crate::rune::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        game.current_guess[0] = Some(0);

        let accepted = submit_guess(&mut game, &mut rng);
        assert!(!accepted);
    }

    #[test]
    fn test_submit_duplicate_rejected_in_no_dupe_mode() {
        let mut game = RuneGame::new(crate::rune::RuneDifficulty::Novice);
        let mut rng = seeded_rng();
        game.current_guess = vec![Some(0), Some(0), Some(1)];

        let accepted = submit_guess(&mut game, &mut rng);
        assert!(!accepted);
    }
}
