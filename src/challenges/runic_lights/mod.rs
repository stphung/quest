//! Runic Lights challenge — Lights Out puzzle.

mod logic;
mod types;

pub use logic::{process_input, start_runic_lights_game};
pub use types::{RunicLightsDifficulty, RunicLightsGame, RunicLightsInput, RunicLightsResult};

impl_apply_game_result! {
    fn apply_runic_lights_result;
    variant: RunicLights;
    result_body: |result, _state, _reward| {
        use RunicLightsResult::*;
        match result {
            Win => (true, ""),
            Loss => (false, "The runes remain ablaze."),
        }
    }
    game_type: crate::achievements::MinigameType::RunicLights;
    icon: "\u{25C7}";
    win_message: "All runes extinguished!";
}
