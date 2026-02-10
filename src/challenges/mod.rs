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
