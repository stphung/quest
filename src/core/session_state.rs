use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::PrestigeCombatBonuses;
use crate::core::recent_drops::RecentDrop;
use crate::core::ticker::Ticker;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::Instant;

/// Transient session state — caches, timers, UI state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SessionState {
    pub last_save_time: i64,
    pub play_time_seconds: u64,
    #[serde(skip)]
    pub chrono_surge_active: bool,
    #[serde(skip)]
    pub debug_force_overcharge: bool,
    #[serde(skip)]
    pub recent_drops: VecDeque<RecentDrop>,
    #[serde(skip)]
    pub xp_rate_samples: VecDeque<u64>,
    #[serde(skip)]
    pub xp_this_second: u64,
    #[serde(skip)]
    pub ticker: Ticker,
    #[serde(skip)]
    pub cached_derived_stats: DerivedStats,
    #[serde(skip)]
    pub cached_prestige_bonuses: PrestigeCombatBonuses,
    #[serde(skip)]
    pub derived_stats_dirty: bool,
    #[serde(skip)]
    pub combat_seconds_this_tick: bool,
    #[serde(skip)]
    pub game_over_shown_at: Option<Instant>,
}
