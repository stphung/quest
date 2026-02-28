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
