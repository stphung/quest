use crate::achievements::types::Achievements;
use crate::core::game_state::GameState;
use crate::deep::DeepState;
use crate::enhancement::types::EnhancementProgress;
use crate::haven::types::Haven;

/// Bundles all mutable references needed by game_tick() into one parameter.
#[allow(dead_code)]
pub struct TickContext<'a> {
    pub state: &'a mut GameState,
    pub tick_counter: &'a mut u32,
    pub haven: &'a mut Haven,
    pub enhancement: &'a mut EnhancementProgress,
    pub deep: &'a mut DeepState,
    pub achievements: &'a mut Achievements,
    pub debug_mode: bool,
}
