use crate::challenges::chess::ChessStats;
use crate::challenges::menu::ChallengeMenu;
use crate::challenges::ActiveMinigame;
use crate::challenges::MinigameWinInfo;
use crate::fishing::types::{FishingSession, FishingState};
use crate::stormglass::sigils::StormSigils;
use serde::{Deserialize, Serialize};

/// Non-combat progression state grouping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionState {
    #[serde(default)]
    pub fishing: FishingState,
    #[serde(skip)]
    pub active_fishing: Option<FishingSession>,
    #[serde(default)]
    pub stormglass: u64,
    #[serde(default)]
    pub stormglass_discovered: bool,
    #[serde(default)]
    pub storm_sigils: StormSigils,
    #[serde(skip)]
    pub challenge_menu: ChallengeMenu,
    #[serde(skip)]
    pub chess_stats: ChessStats,
    #[serde(skip)]
    pub active_minigame: Option<ActiveMinigame>,
    #[serde(skip)]
    pub last_minigame_win: Option<MinigameWinInfo>,
}
