//! Challenge minigames: Chess, Gomoku, Minesweeper, Morris, Rune, Go, JezzBall.

#![allow(unused_imports)]

/// Generate the standard `ALL`, `from_index()`, and `name()` methods shared by
/// all four-variant difficulty enums (Novice / Apprentice / Journeyman / Master).
macro_rules! difficulty_enum_impl {
    ($name:ident) => {
        impl $name {
            pub const ALL: [$name; 4] = [
                $name::Novice,
                $name::Apprentice,
                $name::Journeyman,
                $name::Master,
            ];

            pub fn from_index(index: usize) -> Self {
                Self::ALL.get(index).copied().unwrap_or($name::Novice)
            }

            pub fn name(&self) -> &'static str {
                match self {
                    Self::Novice => "Novice",
                    Self::Apprentice => "Apprentice",
                    Self::Journeyman => "Journeyman",
                    Self::Master => "Master",
                }
            }
        }
    };
}

pub mod chess;
pub mod flappy;
pub mod go;
pub mod gomoku;
pub mod jezzball;
pub mod menu;
pub mod minesweeper;
pub mod morris;
pub mod rune;
pub mod snake;

pub use chess::{ChessDifficulty, ChessGame, ChessResult};
pub use flappy::{FlappyBirdDifficulty, FlappyBirdGame, FlappyBirdResult};
pub use go::{GoDifficulty, GoGame, GoMove, GoResult, Stone, BOARD_SIZE as GO_BOARD_SIZE};
pub use gomoku::{GomokuDifficulty, GomokuGame, GomokuResult, Player as GomokuPlayer, BOARD_SIZE};
pub use jezzball::{
    ActiveWall, Ball as JezzballBall, JezzballDifficulty, JezzballGame, JezzballResult,
    Position as JezzballPosition, WallOrientation,
};
pub use menu::*;
pub use minesweeper::{MinesweeperDifficulty, MinesweeperGame, MinesweeperResult};
pub use morris::{
    MorrisDifficulty, MorrisGame, MorrisPhase, MorrisResult, Player as MorrisPlayer, ADJACENCIES,
};
pub use rune::{FeedbackMark, RuneDifficulty, RuneGame, RuneResult, RUNE_SYMBOLS};
pub use snake::{SnakeDifficulty, SnakeGame, SnakeResult};

/// A currently active challenge minigame. Only one can be active at a time.
#[derive(Debug, Clone)]
pub enum ActiveMinigame {
    Chess(Box<ChessGame>),
    FlappyBird(FlappyBirdGame),
    Morris(MorrisGame),
    Gomoku(GomokuGame),
    Minesweeper(MinesweeperGame),
    Rune(RuneGame),
    Go(GoGame),
    Jezzball(JezzballGame),
    Snake(SnakeGame),
}

/// Information about a minigame win for achievement tracking.
#[derive(Debug, Clone)]
pub struct MinigameWinInfo {
    /// The type of game: "chess", "morris", "gomoku", "minesweeper", "rune", "go", etc.
    pub game_type: &'static str,
    /// The difficulty level: "novice", "apprentice", "journeyman", "master"
    pub difficulty: &'static str,
}

/// Describes a completed challenge for the shared reward-application helper.
pub struct GameResultInfo {
    /// Whether the player won
    pub won: bool,
    /// Game type string for achievements (e.g., "chess", "go")
    pub game_type: &'static str,
    /// Lowercase difficulty string for achievements (e.g., "novice")
    pub difficulty_str: &'static str,
    /// The reward to apply (only used if won)
    pub reward: menu::ChallengeReward,
    /// Icon prefix for combat log entries (e.g., "♟", "◎")
    pub icon: &'static str,
    /// Combat log message on win
    pub win_message: &'static str,
    /// Combat log message on loss/forfeit/draw
    pub loss_message: &'static str,
}

/// Apply challenge rewards to game state, clear active_minigame, and log results.
/// Returns `Some(MinigameWinInfo)` if the player won, `None` otherwise.
#[allow(clippy::needless_pass_by_value)]
pub fn apply_challenge_rewards(
    state: &mut crate::core::game_state::GameState,
    info: GameResultInfo,
) -> Option<MinigameWinInfo> {
    if info.won {
        let old_prestige = state.prestige_rank;

        // XP reward
        let xp_gained = if info.reward.xp_percent > 0 {
            let xp_for_level =
                crate::core::game_logic::xp_for_next_level(state.character_level.max(1));
            let xp = (xp_for_level * info.reward.xp_percent as u64) / 100;
            state.character_xp += xp;
            xp
        } else {
            0
        };

        // Prestige reward
        state.prestige_rank += info.reward.prestige_ranks;

        // Fishing rank reward (capped at 30)
        let fishing_rank_up = if info.reward.fishing_ranks > 0 && state.fishing.rank < 30 {
            state.fishing.rank = (state.fishing.rank + info.reward.fishing_ranks).min(30);
            true
        } else {
            false
        };

        // Combat log entries
        state.combat_state.add_log_entry(
            format!("{} {}", info.icon, info.win_message),
            false,
            true,
        );
        if info.reward.prestige_ranks > 0 {
            state.combat_state.add_log_entry(
                format!(
                    "{} +{} Prestige Ranks (P{} \u{2192} P{})",
                    info.icon, info.reward.prestige_ranks, old_prestige, state.prestige_rank
                ),
                false,
                true,
            );
        }
        if fishing_rank_up {
            state.combat_state.add_log_entry(
                format!(
                    "{} Fishing rank up! Now rank {}: {}",
                    info.icon,
                    state.fishing.rank,
                    state.fishing.rank_name()
                ),
                false,
                true,
            );
        }
        if xp_gained > 0 {
            state.combat_state.add_log_entry(
                format!("{} +{} XP", info.icon, xp_gained),
                false,
                true,
            );
        }
    } else {
        state.combat_state.add_log_entry(
            format!("{} {}", info.icon, info.loss_message),
            false,
            true,
        );
    }

    state.active_minigame = None;

    if info.won {
        Some(MinigameWinInfo {
            game_type: info.game_type,
            difficulty: info.difficulty_str,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::game_state::GameState;

    fn make_info(won: bool, reward: menu::ChallengeReward) -> GameResultInfo {
        GameResultInfo {
            won,
            game_type: "test",
            difficulty_str: "novice",
            reward,
            icon: "T",
            win_message: "You won!",
            loss_message: "You lost.",
        }
    }

    #[test]
    fn test_apply_rewards_win_returns_minigame_win_info() {
        let mut state = GameState::new("Test".to_string(), 0);
        let reward = menu::ChallengeReward {
            prestige_ranks: 1,
            ..Default::default()
        };

        let result = apply_challenge_rewards(&mut state, make_info(true, reward));

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.game_type, "test");
        assert_eq!(info.difficulty, "novice");
    }

    #[test]
    fn test_apply_rewards_loss_returns_none() {
        let mut state = GameState::new("Test".to_string(), 0);
        let reward = menu::ChallengeReward::default();

        let result = apply_challenge_rewards(&mut state, make_info(false, reward));

        assert!(result.is_none());
    }

    #[test]
    fn test_apply_rewards_clears_active_minigame() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.active_minigame = Some(ActiveMinigame::Rune(RuneGame::new(RuneDifficulty::Novice)));
        let reward = menu::ChallengeReward::default();

        apply_challenge_rewards(&mut state, make_info(false, reward));

        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_apply_rewards_grants_xp() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.character_level = 5;
        let old_xp = state.character_xp;
        let reward = menu::ChallengeReward {
            xp_percent: 50,
            ..Default::default()
        };

        apply_challenge_rewards(&mut state, make_info(true, reward));

        assert!(state.character_xp > old_xp);
    }

    #[test]
    fn test_apply_rewards_zero_xp_percent_grants_no_xp() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.character_level = 5;
        let old_xp = state.character_xp;
        let reward = menu::ChallengeReward {
            prestige_ranks: 1,
            xp_percent: 0,
            ..Default::default()
        };

        apply_challenge_rewards(&mut state, make_info(true, reward));

        assert_eq!(state.character_xp, old_xp);
    }

    #[test]
    fn test_apply_rewards_grants_prestige() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;
        let reward = menu::ChallengeReward {
            prestige_ranks: 3,
            ..Default::default()
        };

        apply_challenge_rewards(&mut state, make_info(true, reward));

        assert_eq!(state.prestige_rank, 8);
    }

    #[test]
    fn test_apply_rewards_grants_fishing_ranks() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.fishing.rank = 10;
        let reward = menu::ChallengeReward {
            fishing_ranks: 2,
            ..Default::default()
        };

        apply_challenge_rewards(&mut state, make_info(true, reward));

        assert_eq!(state.fishing.rank, 12);
    }

    #[test]
    fn test_apply_rewards_fishing_rank_capped_at_30() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.fishing.rank = 29;
        let reward = menu::ChallengeReward {
            fishing_ranks: 5,
            ..Default::default()
        };

        apply_challenge_rewards(&mut state, make_info(true, reward));

        assert_eq!(state.fishing.rank, 30);
    }

    #[test]
    fn test_apply_rewards_fishing_rank_not_granted_at_cap() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.fishing.rank = 30;
        let reward = menu::ChallengeReward {
            fishing_ranks: 1,
            ..Default::default()
        };

        apply_challenge_rewards(&mut state, make_info(true, reward));

        assert_eq!(state.fishing.rank, 30);
    }

    #[test]
    fn test_apply_rewards_loss_grants_nothing() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;
        state.character_level = 5;
        let old_xp = state.character_xp;
        let old_fishing = state.fishing.rank;
        let reward = menu::ChallengeReward {
            prestige_ranks: 3,
            xp_percent: 100,
            fishing_ranks: 2,
        };

        apply_challenge_rewards(&mut state, make_info(false, reward));

        assert_eq!(state.prestige_rank, 5);
        assert_eq!(state.character_xp, old_xp);
        assert_eq!(state.fishing.rank, old_fishing);
    }

    #[test]
    fn test_apply_rewards_adds_combat_log_on_win() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.combat_state.combat_log.clear();
        let reward = menu::ChallengeReward {
            prestige_ranks: 1,
            xp_percent: 50,
            ..Default::default()
        };

        apply_challenge_rewards(&mut state, make_info(true, reward));

        // Should have win message + prestige + XP entries
        assert!(state.combat_state.combat_log.len() >= 2);
        assert!(state.combat_state.combat_log[0]
            .message
            .contains("You won!"));
    }

    #[test]
    fn test_apply_rewards_adds_combat_log_on_loss() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.combat_state.combat_log.clear();
        let reward = menu::ChallengeReward::default();

        apply_challenge_rewards(&mut state, make_info(false, reward));

        assert_eq!(state.combat_state.combat_log.len(), 1);
        assert!(state.combat_state.combat_log[0]
            .message
            .contains("You lost."));
    }
}
