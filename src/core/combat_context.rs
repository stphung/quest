use crate::combat::types::CombatState;
use crate::dungeon::types::Dungeon;
use crate::items::equipment::Equipment;
use crate::zones::ZoneProgression;
use serde::{Deserialize, Serialize};

/// Combat-related state grouping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatContext {
    pub combat_state: CombatState,
    pub equipment: Equipment,
    #[serde(default)]
    pub zone_progression: ZoneProgression,
    #[serde(default)]
    pub active_dungeon: Option<Dungeon>,
    #[serde(skip)]
    pub session_kills: u64,
    #[serde(skip)]
    pub consecutive_deaths: u32,
}
