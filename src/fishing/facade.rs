#![allow(dead_code)]
use rand::Rng;

use crate::core::game_state::GameState;
use crate::fishing::logic::{FishingTickResult, HavenFishingBonuses};
use crate::fishing::types::{FishingSession, FishingState};

/// Explicit inputs for the fishing tick facade.
/// Documents the decomposed interface — the fields the fishing system needs.
/// During migration, the facade takes `&mut GameState` and delegates to
/// `tick_fishing_with_haven_result`.
pub struct FishingInput<'a> {
    pub fishing: &'a mut FishingState,
    pub active_fishing: &'a mut Option<FishingSession>,
    pub player_level: u32,
    pub prestige_rank: u32,
    pub haven_bonuses: HavenFishingBonuses,
    pub stormglass: &'a mut u64,
    pub storm_lure_active: bool,
}

/// Facade: tick the fishing system.
///
/// Delegates to `logic::tick_fishing_with_haven_result()` which still
/// requires `&mut GameState`. The `FishingInput` struct above documents
/// the aspirational decomposed interface.
pub fn tick_fishing_facade<R: Rng>(
    state: &mut GameState,
    rng: &mut R,
    haven: &HavenFishingBonuses,
    god_item_fishing_reduction_percent: f64,
) -> FishingTickResult {
    crate::fishing::logic::tick_fishing_with_haven_result(
        state,
        rng,
        haven,
        god_item_fishing_reduction_percent,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fishing_facade_no_active_session() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut state = GameState::new("FishTest".to_string(), 0);
        let haven = HavenFishingBonuses {
            timer_reduction_percent: 0.0,
            double_fish_chance_percent: 0.0,
            max_fishing_rank_bonus: 0,
        };
        let result = tick_fishing_facade(&mut state, &mut rng, &haven, 0.0);
        assert!(result.messages.is_empty());
    }

    #[test]
    fn test_fishing_facade_with_active_session() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut state = GameState::new("FishTest".to_string(), 0);
        let session = crate::fishing::generation::generate_fishing_session(&mut rng);
        state.active_fishing = Some(session);
        let haven = HavenFishingBonuses {
            timer_reduction_percent: 0.0,
            double_fish_chance_percent: 0.0,
            max_fishing_rank_bonus: 0,
        };
        for _ in 0..50 {
            let _result = tick_fishing_facade(&mut state, &mut rng, &haven, 0.0);
        }
        // Should have processed without panicking
    }
}
