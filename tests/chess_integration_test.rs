//! Integration test: Chess minigame flow
//!
//! Tests the full chess flow: discovery → challenge menu → game → result

use quest::challenges::chess::logic::{apply_game_result, start_chess_game};
use quest::challenges::ActiveMinigame;
use quest::GameState;
use quest::{ChessDifficulty, ChessResult};

#[test]
fn test_complete_chess_win_flow() {
    let mut state = GameState::new("Chess Master".to_string(), 0);
    state.prestige_rank = 5;

    // Start a chess game
    start_chess_game(&mut state, ChessDifficulty::Master);
    assert!(matches!(
        state.active_minigame,
        Some(ActiveMinigame::Chess(_))
    ));

    // Simulate a win
    if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
        game.game_result = Some(ChessResult::Win);
    } else {
        panic!("expected chess");
    }

    // Apply result
    let processed = apply_game_result(&mut state);
    assert!(processed.is_some()); // Win returns Some(MinigameWinInfo)
    assert_eq!(state.prestige_rank, 10); // 5 + 5 (Master reward)
    assert!(state.active_minigame.is_none());
}

#[test]
fn test_chess_loss_no_penalty() {
    let mut state = GameState::new("Chess Learner".to_string(), 0);
    state.prestige_rank = 3;

    start_chess_game(&mut state, ChessDifficulty::Novice);
    if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
        game.game_result = Some(ChessResult::Loss);
    } else {
        panic!("expected chess");
    }

    let processed = apply_game_result(&mut state);
    assert!(processed.is_none()); // Loss returns None
    assert_eq!(state.prestige_rank, 3); // Unchanged
}

#[test]
fn test_chess_draw_no_penalty() {
    let mut state = GameState::new("Chess Player".to_string(), 0);
    state.prestige_rank = 7;

    start_chess_game(&mut state, ChessDifficulty::Journeyman);
    if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
        game.game_result = Some(ChessResult::Draw);
    } else {
        panic!("expected chess");
    }

    let processed = apply_game_result(&mut state);
    assert!(processed.is_none()); // Draw returns None
    assert_eq!(state.prestige_rank, 7); // Unchanged
}

#[test]
fn test_chess_forfeit_counts_as_loss() {
    let mut state = GameState::new("Quitter".to_string(), 0);
    state.prestige_rank = 2;

    start_chess_game(&mut state, ChessDifficulty::Apprentice);
    if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
        game.game_result = Some(ChessResult::Forfeit);
    } else {
        panic!("expected chess");
    }

    let processed = apply_game_result(&mut state);
    assert!(processed.is_none()); // Forfeit returns None
    assert_eq!(state.prestige_rank, 2); // Unchanged
    assert_eq!(state.chess_stats.games_lost, 1);
}

#[test]
fn test_difficulty_rewards() {
    assert_eq!(ChessDifficulty::Novice.reward_prestige(), 1);
    assert_eq!(ChessDifficulty::Apprentice.reward_prestige(), 2);
    assert_eq!(ChessDifficulty::Journeyman.reward_prestige(), 3);
    assert_eq!(ChessDifficulty::Master.reward_prestige(), 5);
}

#[test]
fn test_difficulty_elo_estimates() {
    assert_eq!(ChessDifficulty::Novice.estimated_elo(), 500);
    assert_eq!(ChessDifficulty::Apprentice.estimated_elo(), 800);
    assert_eq!(ChessDifficulty::Journeyman.estimated_elo(), 1100);
    assert_eq!(ChessDifficulty::Master.estimated_elo(), 1350);
}

#[test]
fn test_chess_stats_tracking() {
    let mut state = GameState::new("Stats Tracker".to_string(), 0);
    state.prestige_rank = 1;

    // Win a game
    start_chess_game(&mut state, ChessDifficulty::Novice);
    if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
        game.game_result = Some(ChessResult::Win);
    } else {
        panic!("expected chess");
    }
    apply_game_result(&mut state);

    // Lose a game
    start_chess_game(&mut state, ChessDifficulty::Master);
    if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
        game.game_result = Some(ChessResult::Loss);
    } else {
        panic!("expected chess");
    }
    apply_game_result(&mut state);

    // Draw a game
    start_chess_game(&mut state, ChessDifficulty::Apprentice);
    if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
        game.game_result = Some(ChessResult::Draw);
    } else {
        panic!("expected chess");
    }
    apply_game_result(&mut state);

    assert_eq!(state.chess_stats.games_played, 3);
    assert_eq!(state.chess_stats.games_won, 1);
    assert_eq!(state.chess_stats.games_lost, 1);
    assert_eq!(state.chess_stats.games_drawn, 1);
    assert_eq!(state.chess_stats.prestige_earned, 1); // Only from Novice win
}

#[test]
fn test_prestige_accumulates_from_multiple_wins() {
    let mut state = GameState::new("Winner".to_string(), 0);
    state.prestige_rank = 0;

    // Win each difficulty level
    for difficulty in ChessDifficulty::ALL {
        start_chess_game(&mut state, difficulty);
        if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
            game.game_result = Some(ChessResult::Win);
        } else {
            panic!("expected chess");
        }
        apply_game_result(&mut state);
    }

    // Total prestige: 1 + 2 + 3 + 5 = 11
    assert_eq!(state.prestige_rank, 11);
    assert_eq!(state.chess_stats.prestige_earned, 11);
    assert_eq!(state.chess_stats.games_won, 4);
}
