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
    #[allow(dead_code)]
    pub session_kills: u64,
    #[serde(skip)]
    #[allow(dead_code)]
    pub consecutive_deaths: u32,
}

impl Default for CombatContext {
    fn default() -> Self {
        Self {
            combat_state: CombatState::new(50),
            equipment: Equipment::new(),
            zone_progression: ZoneProgression::new(),
            active_dungeon: None,
            session_kills: 0,
            consecutive_deaths: 0,
        }
    }
}
