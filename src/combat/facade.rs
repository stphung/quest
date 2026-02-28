use crate::combat::events::CombatBonuses;
use crate::combat::types::CombatState;
use crate::character::derived_stats::DerivedStats;

/// Explicit inputs for the combat update facade.
pub struct CombatInput<'a> {
    pub combat_state: &'a mut CombatState,
    pub bonuses: &'a CombatBonuses,
    pub derived: &'a DerivedStats,
    pub prestige_rank: u32,
}

/// Facade: update combat with explicit inputs.
pub fn update_combat_facade(
    _input: &mut CombatInput,
) -> Option<()> {
    todo!("Wire to existing update_combat")
}
