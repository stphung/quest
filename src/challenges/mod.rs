//! Challenge minigames: Chess, Gomoku, Minesweeper, Morris, Rune, Sigil Surge, Go, JezzBall, Snake, Flappy Bird, Sigil Matrix.

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

/// Generate a standard `apply_game_result` function for a challenge minigame.
///
/// The `result_body` block receives these bindings and must evaluate to `(bool, &str)`:
/// - `result`: the game result enum value (e.g., `ChessResult::Win`)
/// - `state`: `&mut GameState` (for custom stats tracking or logging)
/// - `reward`: `ChallengeReward` (for chess stats that reference prestige earned)
///
/// # Usage
/// ```text
/// impl_apply_game_result! {
///     variant: Gomoku;
///     result_body: |result, state, reward| {
///         match result {
///             GomokuResult::Win => (true, ""),
///             GomokuResult::Loss => (false, "The strategist nods respectfully."),
///             GomokuResult::Draw => (false, "A rare draw."),
///         }
///     }
///     game_type: MinigameType::Gomoku;
///     icon: "\u{25CE}";
///     win_message: "Victory!";
/// }
/// ```
///
/// For a custom function name (e.g., Go uses `apply_go_result`):
/// ```text
/// impl_apply_game_result! {
///     fn apply_go_result;
///     variant: Go;
///     ...
/// }
/// ```
macro_rules! impl_apply_game_result {
    (
        fn $fn_name:ident;
        variant: $Variant:ident;
        result_body: |$result_var:ident, $state_var:ident, $reward_var:ident| { $($body:tt)* }
        game_type: $game_type:expr;
        icon: $icon:expr;
        win_message: $win_msg:expr;
    ) => {
        pub fn $fn_name(
            $state_var: &mut crate::core::game_state::GameState,
        ) -> Option<crate::challenges::MinigameWinInfo> {
            use crate::challenges::menu::DifficultyInfo;
            use crate::challenges::{apply_challenge_rewards, ActiveMinigame, GameResultInfo};

            let (game_result_opt, difficulty) = match $state_var.active_minigame.as_ref() {
                Some(ActiveMinigame::$Variant(g)) => (g.game_result, g.difficulty),
                _ => return None,
            };
            let $result_var = game_result_opt?;
            let $reward_var = difficulty.reward();

            let (won, loss_message): (bool, &str) = { $($body)* };

            apply_challenge_rewards(
                $state_var,
                GameResultInfo {
                    won,
                    game_type: $game_type,
                    difficulty: difficulty.difficulty_enum(),
                    reward: $reward_var,
                    icon: $icon,
                    win_message: $win_msg,
                    loss_message,
                },
            )
        }
    };
    (
        variant: $Variant:ident;
        result_body: |$result_var:ident, $state_var:ident, $reward_var:ident| { $($body:tt)* }
        game_type: $game_type:expr;
        icon: $icon:expr;
        win_message: $win_msg:expr;
    ) => {
        impl_apply_game_result! {
            fn apply_game_result;
            variant: $Variant;
            result_body: |$result_var, $state_var, $reward_var| { $($body)* }
            game_type: $game_type;
            icon: $icon;
            win_message: $win_msg;
        }
    };
}

pub mod chess;
pub mod facade;
pub mod flappy;
pub mod go;
pub mod gomoku;
pub mod jezzball;
pub mod menu;
pub mod minesweeper;
pub mod morris;
pub mod rune;
pub mod runic_lights;
pub mod runic_shift;
pub mod shard_fusion;
pub mod snake;
pub mod sudoku;

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
pub use runic_lights::{RunicLightsDifficulty, RunicLightsGame, RunicLightsResult};
pub use runic_shift::{
    Block as RunicShiftBlock, BlockState as RunicShiftBlockState, RuneColor, RunicShiftDifficulty,
    RunicShiftGame, RunicShiftResult, GRID_COLS as RUNIC_SHIFT_GRID_COLS,
    GRID_ROWS as RUNIC_SHIFT_GRID_ROWS,
};
pub use shard_fusion::{
    ShardFusionAnimState, ShardFusionDifficulty, ShardFusionGame, ShardFusionResult, TileMove,
    FLASH_TICKS, SLIDE_TICKS,
};
pub use snake::{SnakeDifficulty, SnakeGame, SnakeResult};
pub use sudoku::{SudokuDifficulty, SudokuGame, SudokuInput, SudokuResult};

/// A currently active challenge minigame. Only one can be active at a time.
#[derive(Debug, Clone)]
pub enum ActiveMinigame {
    Chess(Box<ChessGame>),
    FlappyBird(FlappyBirdGame),
    Morris(MorrisGame),
    Gomoku(GomokuGame),
    Minesweeper(MinesweeperGame),
    Rune(RuneGame),
    RunicLights(RunicLightsGame),
    RunicShift(RunicShiftGame),
    ShardFusion(ShardFusionGame),
    Go(GoGame),
    Jezzball(JezzballGame),
    Snake(SnakeGame),
    Sudoku(SudokuGame),
}

impl ActiveMinigame {
    /// Returns true if this minigame has a final result (win/loss/draw).
    pub fn has_game_result(&self) -> bool {
        match self {
            ActiveMinigame::Chess(g) => g.game_result.is_some(),
            ActiveMinigame::FlappyBird(g) => g.game_result.is_some(),
            ActiveMinigame::Morris(g) => g.game_result.is_some(),
            ActiveMinigame::Gomoku(g) => g.game_result.is_some(),
            ActiveMinigame::Minesweeper(g) => g.game_result.is_some(),
            ActiveMinigame::Rune(g) => g.game_result.is_some(),
            ActiveMinigame::RunicLights(g) => g.game_result.is_some(),
            ActiveMinigame::RunicShift(g) => g.game_result.is_some(),
            ActiveMinigame::ShardFusion(g) => g.game_result.is_some(),
            ActiveMinigame::Go(g) => g.game_result.is_some(),
            ActiveMinigame::Jezzball(g) => g.game_result.is_some(),
            ActiveMinigame::Snake(g) => g.game_result.is_some(),
            ActiveMinigame::Sudoku(g) => g.game_result.is_some(),
        }
    }
}

/// Information about a minigame win for achievement tracking.
#[derive(Debug, Clone)]
pub struct MinigameWinInfo {
    /// The type of game.
    pub game_type: crate::achievements::MinigameType,
    /// The difficulty level.
    pub difficulty: crate::achievements::MinigameDifficulty,
}

/// Describes a completed challenge for the shared reward-application helper.
pub struct GameResultInfo {
    /// Whether the player won
    pub won: bool,
    /// Game type enum for achievements.
    pub game_type: crate::achievements::MinigameType,
    /// Difficulty enum for achievements.
    pub difficulty: crate::achievements::MinigameDifficulty,
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

        // Stormglass reward (or XP fallback if not discovered)
        let (sg_gained, xp_gained) = if info.reward.stormglass > 0 {
            if state.stormglass_discovered {
                state.stormglass += info.reward.stormglass;
                (info.reward.stormglass, 0u64)
            } else {
                // Fallback: stormglass / 10 = XP% of next level
                let xp_percent = info.reward.stormglass / 10;
                let xp_for_level =
                    crate::core::game_logic::xp_for_next_level(state.character_level.max(1));
                let xp = (xp_for_level * xp_percent) / 100;
                state.character_xp += xp;
                (0u64, xp)
            }
        } else {
            (0u64, 0u64)
        };

        // Prestige reward
        state.prestige_rank += info.reward.prestige_ranks;

        // Fishing rank reward (capped at absolute max)
        let max_rank = crate::core::constants::MAX_FISHING_RANK;
        let fishing_rank_up = if info.reward.fishing_ranks > 0 && state.fishing.rank < max_rank {
            state.fishing.rank = (state.fishing.rank + info.reward.fishing_ranks).min(max_rank);
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
        if sg_gained > 0 {
            state.combat_state.add_log_entry(
                format!("{} +{} Stormglass", info.icon, sg_gained),
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
            difficulty: info.difficulty,
        })
    } else {
        None
    }
}

/// Shared forfeit confirmation handler.
/// Returns true if the forfeit was confirmed (game_result set to loss).
/// Call this when the player presses Esc/Forfeit.
pub fn handle_forfeit<R>(
    game_result: &mut Option<R>,
    forfeit_pending: &mut bool,
    loss_variant: R,
) -> bool {
    if *forfeit_pending {
        *game_result = Some(loss_variant);
        true
    } else {
        *forfeit_pending = true;
        false
    }
}

/// Cancel a pending forfeit. Call this on any non-Esc input
/// when forfeit_pending is true.
pub fn cancel_forfeit_if_pending(forfeit_pending: &mut bool) {
    if *forfeit_pending {
        *forfeit_pending = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::game_state::GameState;

    fn make_info(won: bool, reward: menu::ChallengeReward) -> GameResultInfo {
        GameResultInfo {
            won,
            game_type: crate::achievements::MinigameType::Chess,
            difficulty: crate::achievements::MinigameDifficulty::Novice,
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
        assert_eq!(info.game_type, crate::achievements::MinigameType::Chess);
        assert_eq!(
            info.difficulty,
            crate::achievements::MinigameDifficulty::Novice
        );
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
        state.stormglass_discovered = false;
        state.character_level = 5;
        let old_xp = state.character_xp;
        let reward = menu::ChallengeReward {
            stormglass: 500,
            ..Default::default()
        };

        apply_challenge_rewards(&mut state, make_info(true, reward));

        assert!(state.character_xp > old_xp);
    }

    #[test]
    fn test_apply_rewards_zero_stormglass_grants_nothing() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.character_level = 5;
        let old_xp = state.character_xp;
        let reward = menu::ChallengeReward {
            prestige_ranks: 1,
            stormglass: 0,
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
    fn test_apply_rewards_fishing_rank_capped_at_max() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.fishing.rank = 38;
        let reward = menu::ChallengeReward {
            fishing_ranks: 5,
            ..Default::default()
        };

        apply_challenge_rewards(&mut state, make_info(true, reward));

        assert_eq!(state.fishing.rank, crate::core::constants::MAX_FISHING_RANK);
    }

    #[test]
    fn test_apply_rewards_fishing_rank_not_granted_at_cap() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.fishing.rank = crate::core::constants::MAX_FISHING_RANK;
        let reward = menu::ChallengeReward {
            fishing_ranks: 1,
            ..Default::default()
        };

        apply_challenge_rewards(&mut state, make_info(true, reward));

        assert_eq!(state.fishing.rank, crate::core::constants::MAX_FISHING_RANK);
    }

    #[test]
    fn test_apply_rewards_fishing_rank_granted_above_30() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.fishing.rank = 35;
        let reward = menu::ChallengeReward {
            fishing_ranks: 1,
            ..Default::default()
        };

        apply_challenge_rewards(&mut state, make_info(true, reward));

        assert_eq!(state.fishing.rank, 36);
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
            stormglass: 1000,
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
            stormglass: 500,
            ..Default::default()
        };

        apply_challenge_rewards(&mut state, make_info(true, reward));

        // Should have win message + prestige + SG/XP entries
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

    #[test]
    fn test_apply_rewards_grants_stormglass_when_discovered() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.stormglass_discovered = true;
        state.stormglass = 100;
        let old_xp = state.character_xp;
        let reward = menu::ChallengeReward {
            stormglass: 1000,
            ..Default::default()
        };
        apply_challenge_rewards(&mut state, make_info(true, reward));
        assert_eq!(state.stormglass, 1100);
        assert_eq!(state.character_xp, old_xp);
    }

    #[test]
    fn test_apply_rewards_xp_fallback_when_not_discovered() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.stormglass_discovered = false;
        state.character_level = 5;
        let old_xp = state.character_xp;
        let old_sg = state.stormglass;
        let reward = menu::ChallengeReward {
            stormglass: 1000,
            ..Default::default()
        };
        apply_challenge_rewards(&mut state, make_info(true, reward));
        assert!(state.character_xp > old_xp);
        assert_eq!(state.stormglass, old_sg);
    }

    mod forfeit_tests {
        use super::super::*;

        #[derive(Debug, PartialEq)]
        #[allow(dead_code)]
        enum TestResult {
            Win,
            Loss,
        }

        #[test]
        fn test_handle_forfeit_first_press_sets_pending() {
            let mut result: Option<TestResult> = None;
            let mut pending = false;
            let confirmed = handle_forfeit(&mut result, &mut pending, TestResult::Loss);
            assert!(!confirmed);
            assert!(pending);
            assert!(result.is_none());
        }

        #[test]
        fn test_handle_forfeit_second_press_confirms() {
            let mut result: Option<TestResult> = None;
            let mut pending = true;
            let confirmed = handle_forfeit(&mut result, &mut pending, TestResult::Loss);
            assert!(confirmed);
            assert_eq!(result, Some(TestResult::Loss));
        }

        #[test]
        fn test_cancel_forfeit_clears_pending() {
            let mut pending = true;
            cancel_forfeit_if_pending(&mut pending);
            assert!(!pending);
        }

        #[test]
        fn test_cancel_forfeit_noop_when_not_pending() {
            let mut pending = false;
            cancel_forfeit_if_pending(&mut pending);
            assert!(!pending);
        }
    }
}
