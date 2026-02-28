use crate::fishing::logic::HavenFishingBonuses;
use crate::fishing::types::{FishingSession, FishingState};

/// Explicit inputs for the fishing tick facade.
pub struct FishingInput<'a> {
    pub fishing: &'a mut FishingState,
    pub active_fishing: &'a mut Option<FishingSession>,
    pub player_level: u32,
    pub prestige_rank: u32,
    pub haven_bonuses: HavenFishingBonuses,
    pub stormglass: &'a mut u64,
    pub storm_lure_active: bool,
}
