//! Challenge minigames: Chess, Gomoku, Minesweeper, Morris, Rune, Go.

#![allow(unused_imports)]

pub mod chess;
pub mod go;
pub mod gomoku;
pub mod menu;
pub mod minesweeper;
pub mod morris;
pub mod rune;

pub use chess::ChessGame;
pub use go::{GoGame, GoMove, Stone, BOARD_SIZE as GO_BOARD_SIZE};
pub use gomoku::{GomokuGame, Player as GomokuPlayer, BOARD_SIZE};
pub use menu::*;
pub use minesweeper::MinesweeperGame;
pub use morris::{MorrisGame, MorrisPhase, Player as MorrisPlayer, ADJACENCIES};
pub use rune::{FeedbackMark, RuneGame, RuneGuess, RUNE_SYMBOLS};

use serde::{Deserialize, Serialize};

// ============================================================================
// Shared enums for all challenge minigames
// ============================================================================

/// Shared difficulty level for all challenge minigames.
/// All 6 games use the same 4-tier difficulty system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

impl ChallengeDifficulty {
    pub const ALL: [ChallengeDifficulty; 4] = [
        ChallengeDifficulty::Novice,
        ChallengeDifficulty::Apprentice,
        ChallengeDifficulty::Journeyman,
        ChallengeDifficulty::Master,
    ];

    pub fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or(Self::Novice)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Novice => "Novice",
            Self::Apprentice => "Apprentice",
            Self::Journeyman => "Journeyman",
            Self::Master => "Master",
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Self::Novice => "novice",
            Self::Apprentice => "apprentice",
            Self::Journeyman => "journeyman",
            Self::Master => "master",
        }
    }
}

/// Shared result type for all challenge minigames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeResult {
    Win,
    Loss,
    Draw,
    Forfeit,
}

/// UI-agnostic input actions shared by all minigames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinigameInput {
    Up,
    Down,
    Left,
    Right,
    /// Primary action (Enter): select piece, place stone, reveal cell, submit guess
    Primary,
    /// Secondary action (game-specific): pass in Go, toggle flag in Minesweeper, clear guess in Rune
    Secondary,
    /// Cancel (Esc): cancel selection or trigger forfeit
    Cancel,
    Other,
}

/// A currently active challenge minigame. Only one can be active at a time.
#[derive(Debug, Clone)]
pub enum ActiveMinigame {
    Chess(Box<ChessGame>),
    Morris(MorrisGame),
    Gomoku(GomokuGame),
    Minesweeper(MinesweeperGame),
    Rune(RuneGame),
    Go(GoGame),
}

/// Information about a minigame win for achievement tracking.
#[derive(Debug, Clone)]
pub struct MinigameWinInfo {
    /// The type of game: "chess", "morris", "gomoku", "minesweeper", "rune", "go"
    pub game_type: &'static str,
    /// The difficulty level: "novice", "apprentice", "journeyman", "master"
    pub difficulty: &'static str,
}

/// Apply the result of a finished minigame: grant rewards, log results, clean up.
///
/// This is the single shared function replacing 6 per-game `apply_game_result` functions.
/// Returns `Some(MinigameWinInfo)` on win for achievement tracking.
pub fn apply_minigame_result(
    state: &mut crate::core::game_state::GameState,
) -> Option<MinigameWinInfo> {
    // Extract result, difficulty, and challenge type from the active minigame
    let (result, difficulty, challenge_type) = match state.active_minigame.as_ref()? {
        ActiveMinigame::Chess(g) => (g.game_result?, g.difficulty, menu::ChallengeType::Chess),
        ActiveMinigame::Go(g) => (g.game_result?, g.difficulty, menu::ChallengeType::Go),
        ActiveMinigame::Morris(g) => (g.game_result?, g.difficulty, menu::ChallengeType::Morris),
        ActiveMinigame::Gomoku(g) => (g.game_result?, g.difficulty, menu::ChallengeType::Gomoku),
        ActiveMinigame::Minesweeper(g) => (
            g.game_result?,
            g.difficulty,
            menu::ChallengeType::Minesweeper,
        ),
        ActiveMinigame::Rune(g) => (g.game_result?, g.difficulty, menu::ChallengeType::Rune),
    };

    // Chess-specific stats tracking
    if challenge_type == menu::ChallengeType::Chess {
        state.chess_stats.games_played += 1;
        match result {
            ChallengeResult::Win => state.chess_stats.games_won += 1,
            ChallengeResult::Loss | ChallengeResult::Forfeit => {
                state.chess_stats.games_lost += 1;
            }
            ChallengeResult::Draw => state.chess_stats.games_drawn += 1,
        }
    }

    let won = result == ChallengeResult::Win;
    let old_prestige = state.prestige_rank;
    let reward = challenge_type.reward(difficulty);
    let icon = challenge_type.log_icon();

    if won {
        // XP reward (floor of 100 XP when xp_percent > 0)
        let xp_gained = if reward.xp_percent > 0 {
            let xp_for_level =
                crate::core::game_logic::xp_for_next_level(state.character_level.max(1));
            let xp = ((xp_for_level * reward.xp_percent as u64) / 100).max(100);
            state.character_xp += xp;
            xp
        } else {
            0
        };

        // Prestige reward
        state.prestige_rank += reward.prestige_ranks;
        if challenge_type == menu::ChallengeType::Chess {
            state.chess_stats.prestige_earned += reward.prestige_ranks;
        }

        // Fishing rank reward (capped at 30)
        let fishing_rank_up = if reward.fishing_ranks > 0 && state.fishing.rank < 30 {
            state.fishing.rank = (state.fishing.rank + reward.fishing_ranks).min(30);
            true
        } else {
            false
        };

        // Log entries
        state.combat_state.add_log_entry(
            challenge_type.result_flavor(result).to_string(),
            false,
            true,
        );
        if reward.prestige_ranks > 0 {
            state.combat_state.add_log_entry(
                format!(
                    "{} +{} Prestige Ranks (P{} → P{})",
                    icon, reward.prestige_ranks, old_prestige, state.prestige_rank
                ),
                false,
                true,
            );
        }
        if fishing_rank_up {
            state.combat_state.add_log_entry(
                format!(
                    "{} Fishing rank up! Now rank {}: {}",
                    icon,
                    state.fishing.rank,
                    state.fishing.rank_name()
                ),
                false,
                true,
            );
        }
        if xp_gained > 0 {
            state
                .combat_state
                .add_log_entry(format!("{} +{} XP", icon, xp_gained), false, true);
        }
    } else {
        // Loss/Draw/Forfeit
        state.combat_state.add_log_entry(
            challenge_type.result_flavor(result).to_string(),
            false,
            true,
        );
    }

    state.active_minigame = None;

    if won {
        Some(MinigameWinInfo {
            game_type: challenge_type.game_type_str(),
            difficulty: difficulty.to_str(),
        })
    } else {
        None
    }
}

/// Start a minigame with the given challenge type and difficulty.
///
/// This is the single shared function replacing 6 per-game `start_xxx_game` functions.
pub fn start_minigame(
    state: &mut crate::core::game_state::GameState,
    challenge_type: &menu::ChallengeType,
    difficulty: ChallengeDifficulty,
) {
    state.active_minigame = Some(match challenge_type {
        menu::ChallengeType::Chess => ActiveMinigame::Chess(Box::new(ChessGame::new(difficulty))),
        menu::ChallengeType::Go => ActiveMinigame::Go(GoGame::new(difficulty)),
        menu::ChallengeType::Morris => ActiveMinigame::Morris(MorrisGame::new(difficulty)),
        menu::ChallengeType::Gomoku => ActiveMinigame::Gomoku(GomokuGame::new(difficulty)),
        menu::ChallengeType::Minesweeper => {
            ActiveMinigame::Minesweeper(MinesweeperGame::new(difficulty))
        }
        menu::ChallengeType::Rune => ActiveMinigame::Rune(RuneGame::new(difficulty)),
    });
    state.challenge_menu.close();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::game_state::GameState;

    // ========== ChallengeDifficulty tests ==========

    #[test]
    fn test_difficulty_to_str() {
        assert_eq!(ChallengeDifficulty::Novice.to_str(), "novice");
        assert_eq!(ChallengeDifficulty::Apprentice.to_str(), "apprentice");
        assert_eq!(ChallengeDifficulty::Journeyman.to_str(), "journeyman");
        assert_eq!(ChallengeDifficulty::Master.to_str(), "master");
    }

    #[test]
    fn test_difficulty_name() {
        assert_eq!(ChallengeDifficulty::Novice.name(), "Novice");
        assert_eq!(ChallengeDifficulty::Apprentice.name(), "Apprentice");
        assert_eq!(ChallengeDifficulty::Journeyman.name(), "Journeyman");
        assert_eq!(ChallengeDifficulty::Master.name(), "Master");
    }

    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(
            ChallengeDifficulty::from_index(0),
            ChallengeDifficulty::Novice
        );
        assert_eq!(
            ChallengeDifficulty::from_index(1),
            ChallengeDifficulty::Apprentice
        );
        assert_eq!(
            ChallengeDifficulty::from_index(2),
            ChallengeDifficulty::Journeyman
        );
        assert_eq!(
            ChallengeDifficulty::from_index(3),
            ChallengeDifficulty::Master
        );
        // Out of bounds defaults to Novice
        assert_eq!(
            ChallengeDifficulty::from_index(99),
            ChallengeDifficulty::Novice
        );
    }

    #[test]
    fn test_difficulty_all_has_four_entries() {
        assert_eq!(ChallengeDifficulty::ALL.len(), 4);
    }

    // ========== start_minigame tests ==========

    #[test]
    fn test_start_minigame_go() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.open();
        start_minigame(
            &mut state,
            &menu::ChallengeType::Go,
            ChallengeDifficulty::Journeyman,
        );
        assert!(matches!(state.active_minigame, Some(ActiveMinigame::Go(_))));
        assert!(!state.challenge_menu.is_open);
    }

    #[test]
    fn test_start_minigame_all_types() {
        let types = [
            (menu::ChallengeType::Chess, "Chess"),
            (menu::ChallengeType::Go, "Go"),
            (menu::ChallengeType::Morris, "Morris"),
            (menu::ChallengeType::Gomoku, "Gomoku"),
            (menu::ChallengeType::Minesweeper, "Minesweeper"),
            (menu::ChallengeType::Rune, "Rune"),
        ];
        for (ct, name) in &types {
            let mut state = GameState::new("Test".to_string(), 0);
            state.challenge_menu.open();
            start_minigame(&mut state, ct, ChallengeDifficulty::Novice);
            assert!(
                state.active_minigame.is_some(),
                "{} should create active minigame",
                name
            );
            assert!(!state.challenge_menu.is_open, "{} should close menu", name);
        }
    }

    // ========== apply_minigame_result tests ==========

    fn setup_state_with_game(
        challenge_type: menu::ChallengeType,
        difficulty: ChallengeDifficulty,
        result: ChallengeResult,
    ) -> GameState {
        let mut state = GameState::new("Test".to_string(), 0);
        state.character_level = 1;
        match challenge_type {
            menu::ChallengeType::Chess => {
                let mut game = ChessGame::new(difficulty);
                game.game_result = Some(result);
                state.active_minigame = Some(ActiveMinigame::Chess(Box::new(game)));
            }
            menu::ChallengeType::Go => {
                let mut game = GoGame::new(difficulty);
                game.game_result = Some(result);
                state.active_minigame = Some(ActiveMinigame::Go(game));
            }
            menu::ChallengeType::Morris => {
                let mut game = MorrisGame::new(difficulty);
                game.game_result = Some(result);
                state.active_minigame = Some(ActiveMinigame::Morris(game));
            }
            menu::ChallengeType::Gomoku => {
                let mut game = GomokuGame::new(difficulty);
                game.game_result = Some(result);
                state.active_minigame = Some(ActiveMinigame::Gomoku(game));
            }
            menu::ChallengeType::Minesweeper => {
                let mut game = MinesweeperGame::new(difficulty);
                game.game_result = Some(result);
                state.active_minigame = Some(ActiveMinigame::Minesweeper(game));
            }
            menu::ChallengeType::Rune => {
                let mut game = RuneGame::new(difficulty);
                game.game_result = Some(result);
                state.active_minigame = Some(ActiveMinigame::Rune(game));
            }
        }
        state
    }

    #[test]
    fn test_apply_result_xp_floor_of_100() {
        // Rune Novice gives 25% XP. At level 1, xp_for_next_level = 100.
        // 100 * 25 / 100 = 25, but floor is 100. So player should get 100 XP.
        let mut state = setup_state_with_game(
            menu::ChallengeType::Rune,
            ChallengeDifficulty::Novice,
            ChallengeResult::Win,
        );
        state.character_level = 1;
        state.character_xp = 0;

        let result = apply_minigame_result(&mut state);
        assert!(result.is_some());
        assert_eq!(state.character_xp, 100, "XP should be floored at 100");
    }

    #[test]
    fn test_apply_result_xp_above_floor() {
        // Morris Journeyman gives 150% XP. At level 10, xp_for_next_level is much > 100.
        let mut state = setup_state_with_game(
            menu::ChallengeType::Morris,
            ChallengeDifficulty::Journeyman,
            ChallengeResult::Win,
        );
        state.character_level = 10;
        state.character_xp = 0;

        let xp_for_level = crate::core::game_logic::xp_for_next_level(state.character_level.max(1));
        let expected = ((xp_for_level * 150) / 100).max(100);

        apply_minigame_result(&mut state);
        assert_eq!(state.character_xp, expected);
        assert!(state.character_xp > 100, "XP should be above the floor");
    }

    #[test]
    fn test_apply_result_no_xp_for_prestige_only_rewards() {
        // Chess gives prestige only (0% XP). XP should stay unchanged.
        let mut state = setup_state_with_game(
            menu::ChallengeType::Chess,
            ChallengeDifficulty::Novice,
            ChallengeResult::Win,
        );
        state.character_xp = 500;

        apply_minigame_result(&mut state);
        assert_eq!(state.character_xp, 500, "XP should not change for Chess");
    }

    #[test]
    fn test_apply_result_fishing_rank_cap_at_30() {
        // Rune Master gives +2 fishing ranks. Start at rank 29 => should cap at 30.
        let mut state = setup_state_with_game(
            menu::ChallengeType::Rune,
            ChallengeDifficulty::Master,
            ChallengeResult::Win,
        );
        state.fishing.rank = 29;

        apply_minigame_result(&mut state);
        assert_eq!(
            state.fishing.rank, 30,
            "Fishing rank should cap at 30, not 31"
        );
    }

    #[test]
    fn test_apply_result_fishing_rank_no_increase_at_cap() {
        // Already at rank 30 => should stay at 30.
        let mut state = setup_state_with_game(
            menu::ChallengeType::Rune,
            ChallengeDifficulty::Master,
            ChallengeResult::Win,
        );
        state.fishing.rank = 30;

        apply_minigame_result(&mut state);
        assert_eq!(state.fishing.rank, 30, "Fishing rank should not exceed 30");
    }

    #[test]
    fn test_apply_result_loss_returns_none() {
        let mut state = setup_state_with_game(
            menu::ChallengeType::Morris,
            ChallengeDifficulty::Novice,
            ChallengeResult::Loss,
        );
        let result = apply_minigame_result(&mut state);
        assert!(result.is_none());
        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_apply_result_forfeit_returns_none() {
        let mut state = setup_state_with_game(
            menu::ChallengeType::Go,
            ChallengeDifficulty::Novice,
            ChallengeResult::Forfeit,
        );
        let result = apply_minigame_result(&mut state);
        assert!(result.is_none());
        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_apply_result_draw_returns_none() {
        let mut state = setup_state_with_game(
            menu::ChallengeType::Go,
            ChallengeDifficulty::Novice,
            ChallengeResult::Draw,
        );
        let result = apply_minigame_result(&mut state);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_result_win_returns_correct_info() {
        let mut state = setup_state_with_game(
            menu::ChallengeType::Go,
            ChallengeDifficulty::Master,
            ChallengeResult::Win,
        );
        let result = apply_minigame_result(&mut state);
        let info = result.expect("Win should return Some");
        assert_eq!(info.game_type, "go");
        assert_eq!(info.difficulty, "master");
    }

    #[test]
    fn test_apply_result_game_type_str_all_games() {
        let cases = vec![
            (menu::ChallengeType::Chess, "chess"),
            (menu::ChallengeType::Go, "go"),
            (menu::ChallengeType::Morris, "morris"),
            (menu::ChallengeType::Gomoku, "gomoku"),
            (menu::ChallengeType::Minesweeper, "minesweeper"),
            (menu::ChallengeType::Rune, "rune"),
        ];
        for (ct, expected_type) in cases {
            let mut state =
                setup_state_with_game(ct, ChallengeDifficulty::Novice, ChallengeResult::Win);
            let result = apply_minigame_result(&mut state);
            let info = result.unwrap();
            assert_eq!(info.game_type, expected_type);
            assert_eq!(info.difficulty, "novice");
        }
    }

    #[test]
    fn test_apply_result_log_entries_on_win() {
        let mut state = setup_state_with_game(
            menu::ChallengeType::Chess,
            ChallengeDifficulty::Novice,
            ChallengeResult::Win,
        );
        state.prestige_rank = 5;

        apply_minigame_result(&mut state);

        // Should have at least the flavor text and prestige log entries
        assert!(
            state.combat_state.combat_log.len() >= 2,
            "Should have flavor text and prestige log"
        );

        // First entry: flavor text, should be marked as important
        let first = &state.combat_state.combat_log[0];
        assert!(
            first.message.contains("Checkmate"),
            "First log should be win flavor text"
        );
        assert!(first.is_player_action, "Flavor text should be important");

        // Second entry: prestige reward
        let second = &state.combat_state.combat_log[1];
        assert!(
            second.message.contains("Prestige Ranks"),
            "Second log should mention prestige"
        );
        assert!(
            second.is_player_action,
            "Prestige log should be marked important"
        );
    }

    #[test]
    fn test_apply_result_log_entries_on_loss() {
        let mut state = setup_state_with_game(
            menu::ChallengeType::Chess,
            ChallengeDifficulty::Novice,
            ChallengeResult::Loss,
        );

        apply_minigame_result(&mut state);

        // Should have just the loss flavor text
        assert_eq!(state.combat_state.combat_log.len(), 1);
        let entry = &state.combat_state.combat_log[0];
        assert!(
            entry.is_player_action,
            "Loss flavor text should be important"
        );
    }

    #[test]
    fn test_apply_result_no_active_minigame_returns_none() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.active_minigame = None;
        let result = apply_minigame_result(&mut state);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_result_no_game_result_returns_none() {
        let mut state = GameState::new("Test".to_string(), 0);
        // Game exists but has no result yet
        let game = ChessGame::new(ChallengeDifficulty::Novice);
        assert!(game.game_result.is_none());
        state.active_minigame = Some(ActiveMinigame::Chess(Box::new(game)));

        let result = apply_minigame_result(&mut state);
        assert!(result.is_none());
        // Game should still be active (not cleared)
        assert!(state.active_minigame.is_some());
    }

    #[test]
    fn test_apply_result_chess_forfeit_counts_as_loss() {
        let mut state = setup_state_with_game(
            menu::ChallengeType::Chess,
            ChallengeDifficulty::Novice,
            ChallengeResult::Forfeit,
        );

        apply_minigame_result(&mut state);
        assert_eq!(state.chess_stats.games_played, 1);
        assert_eq!(state.chess_stats.games_lost, 1);
        assert_eq!(state.chess_stats.games_won, 0);
    }

    #[test]
    fn test_apply_result_loss_no_rewards() {
        let mut state = setup_state_with_game(
            menu::ChallengeType::Morris,
            ChallengeDifficulty::Master,
            ChallengeResult::Loss,
        );
        state.prestige_rank = 5;
        state.character_xp = 100;
        state.fishing.rank = 10;

        apply_minigame_result(&mut state);
        assert_eq!(state.prestige_rank, 5, "Prestige should not change on loss");
        assert_eq!(state.character_xp, 100, "XP should not change on loss");
        assert_eq!(
            state.fishing.rank, 10,
            "Fishing rank should not change on loss"
        );
    }
}
