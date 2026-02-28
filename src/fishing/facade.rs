#![allow(dead_code)]
use rand::Rng;

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

/// Facade: tick the fishing system with explicit inputs.
pub fn tick_fishing_facade<R: Rng>(_input: &mut FishingInput, _rng: &mut R) -> Option<()> {
    // Will delegate to existing tick_fishing_with_haven_result()
    // Not wired yet — this establishes the API surface
    todo!("Wire to existing tick_fishing_with_haven_result")
}
