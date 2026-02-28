#![allow(dead_code)]

use crate::challenges::ActiveMinigame;
use rand::Rng;

/// Facade: tick challenge AI with just the active minigame.
///
/// Dispatches to the appropriate `process_ai_thinking` function for Chess,
/// Morris, Gomoku, and Go. Other minigame variants (action games, puzzles)
/// have no AI tick and are ignored.
pub fn tick_challenge_ai_facade<R: Rng>(minigame: &mut Option<ActiveMinigame>, rng: &mut R) {
    match minigame {
        Some(ActiveMinigame::Chess(game)) => {
            crate::challenges::chess::logic::process_ai_thinking(game, rng);
        }
        Some(ActiveMinigame::Morris(game)) => {
            crate::challenges::morris::logic::process_ai_thinking(game, rng);
        }
        Some(ActiveMinigame::Gomoku(game)) => {
            crate::challenges::gomoku::logic::process_ai_thinking(game, rng);
        }
        Some(ActiveMinigame::Go(game)) => {
            crate::challenges::go::process_ai_thinking(game, rng);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn none_minigame_does_not_panic() {
        let mut minigame: Option<ActiveMinigame> = None;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        tick_challenge_ai_facade(&mut minigame, &mut rng);
        assert!(minigame.is_none());
    }

    #[test]
    fn chess_ai_thinking_advances() {
        use crate::challenges::chess::types::{ChessDifficulty, ChessGame};

        let mut game = ChessGame::new(ChessDifficulty::Novice);
        game.ai_thinking = true;
        game.ai_think_ticks = 0;

        let mut minigame = Some(ActiveMinigame::Chess(Box::new(game)));
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        tick_challenge_ai_facade(&mut minigame, &mut rng);

        // After one tick, ai_think_ticks should have incremented
        match &minigame {
            Some(ActiveMinigame::Chess(g)) => {
                assert_eq!(g.ai_think_ticks, 1);
            }
            _ => panic!("Expected Chess minigame"),
        }
    }
}
