//! Integration test: Chess minigame flow
//!
//! Tests the full chess flow: discovery → challenge menu → game → result

use quest::challenges::menu::ChallengeType;
use quest::challenges::{apply_minigame_result, start_minigame, ActiveMinigame};
use quest::{ChallengeDifficulty, ChallengeResult, GameState};

#[test]
fn test_complete_chess_win_flow() {
    let mut state = GameState::new("Chess Master".to_string(), 0);
    state.prestige_rank = 5;

    // Start a chess game
    start_minigame(
        &mut state,
        &ChallengeType::Chess,
        ChallengeDifficulty::Master,
    );
    assert!(matches!(
        state.active_minigame,
        Some(ActiveMinigame::Chess(_))
    ));

    // Simulate a win
    if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
        game.game_result = Some(ChallengeResult::Win);
    } else {
        panic!("expected chess");
    }

    // Apply result
    let processed = apply_minigame_result(&mut state);
    assert!(processed.is_some()); // Win returns Some(MinigameWinInfo)
    assert_eq!(state.prestige_rank, 10); // 5 + 5 (Master reward)
    assert!(state.active_minigame.is_none());
}

#[test]
fn test_chess_loss_no_penalty() {
    let mut state = GameState::new("Chess Learner".to_string(), 0);
    state.prestige_rank = 3;

    start_minigame(
        &mut state,
        &ChallengeType::Chess,
        ChallengeDifficulty::Novice,
    );
    if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
        game.game_result = Some(ChallengeResult::Loss);
    } else {
        panic!("expected chess");
    }

    let processed = apply_minigame_result(&mut state);
    assert!(processed.is_none()); // Loss returns None
    assert_eq!(state.prestige_rank, 3); // Unchanged
}

#[test]
fn test_chess_draw_no_penalty() {
    let mut state = GameState::new("Chess Player".to_string(), 0);
    state.prestige_rank = 7;

    start_minigame(
        &mut state,
        &ChallengeType::Chess,
        ChallengeDifficulty::Journeyman,
    );
    if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
        game.game_result = Some(ChallengeResult::Draw);
    } else {
        panic!("expected chess");
    }

    let processed = apply_minigame_result(&mut state);
    assert!(processed.is_none()); // Draw returns None
    assert_eq!(state.prestige_rank, 7); // Unchanged
}

#[test]
fn test_chess_forfeit_counts_as_loss() {
    let mut state = GameState::new("Quitter".to_string(), 0);
    state.prestige_rank = 2;

    start_minigame(
        &mut state,
        &ChallengeType::Chess,
        ChallengeDifficulty::Apprentice,
    );
    if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
        game.game_result = Some(ChallengeResult::Forfeit);
    } else {
        panic!("expected chess");
    }

    let processed = apply_minigame_result(&mut state);
    assert!(processed.is_none()); // Forfeit returns None
    assert_eq!(state.prestige_rank, 2); // Unchanged
    assert_eq!(state.chess_stats.games_lost, 1);
}

#[test]
fn test_difficulty_rewards() {
    assert_eq!(
        ChallengeType::Chess
            .reward(ChallengeDifficulty::Novice)
            .prestige_ranks,
        1
    );
    assert_eq!(
        ChallengeType::Chess
            .reward(ChallengeDifficulty::Apprentice)
            .prestige_ranks,
        2
    );
    assert_eq!(
        ChallengeType::Chess
            .reward(ChallengeDifficulty::Journeyman)
            .prestige_ranks,
        3
    );
    assert_eq!(
        ChallengeType::Chess
            .reward(ChallengeDifficulty::Master)
            .prestige_ranks,
        5
    );
}

#[test]
fn test_chess_stats_tracking() {
    let mut state = GameState::new("Stats Tracker".to_string(), 0);
    state.prestige_rank = 1;

    // Win a game
    start_minigame(
        &mut state,
        &ChallengeType::Chess,
        ChallengeDifficulty::Novice,
    );
    if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
        game.game_result = Some(ChallengeResult::Win);
    } else {
        panic!("expected chess");
    }
    apply_minigame_result(&mut state);

    // Lose a game
    start_minigame(
        &mut state,
        &ChallengeType::Chess,
        ChallengeDifficulty::Master,
    );
    if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
        game.game_result = Some(ChallengeResult::Loss);
    } else {
        panic!("expected chess");
    }
    apply_minigame_result(&mut state);

    // Draw a game
    start_minigame(
        &mut state,
        &ChallengeType::Chess,
        ChallengeDifficulty::Apprentice,
    );
    if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
        game.game_result = Some(ChallengeResult::Draw);
    } else {
        panic!("expected chess");
    }
    apply_minigame_result(&mut state);

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
    for difficulty in ChallengeDifficulty::ALL {
        start_minigame(&mut state, &ChallengeType::Chess, difficulty);
        if let Some(ActiveMinigame::Chess(game)) = &mut state.active_minigame {
            game.game_result = Some(ChallengeResult::Win);
        } else {
            panic!("expected chess");
        }
        apply_minigame_result(&mut state);
    }

    // Total prestige: 1 + 2 + 3 + 5 = 11
    assert_eq!(state.prestige_rank, 11);
    assert_eq!(state.chess_stats.prestige_earned, 11);
    assert_eq!(state.chess_stats.games_won, 4);
}
