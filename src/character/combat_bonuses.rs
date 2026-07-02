//! Prestige combat bonuses calculated from prestige rank.

use crate::core::constants::*;

/// Combat bonuses from prestige rank (independent of attributes/Haven).
/// These are flat values that bypass enemy HP scaling.
#[derive(Debug, Clone, Copy, Default)]
pub struct PrestigeCombatBonuses {
    /// Flat damage added after Haven % multiplier, before crit
    pub flat_damage: u64,
    /// Flat defense added to DEX-based defense
    pub flat_defense: u64,
    /// Percentage points added to crit chance
    pub crit_chance: f64,
    /// Flat HP added to combat HP (NOT to DerivedStats.max_hp)
    pub flat_hp: u64,
}

impl PrestigeCombatBonuses {
    pub fn from_rank(rank: u32) -> Self {
        if rank == 0 {
            return Self::default();
        }
        Self {
            flat_damage: (PRESTIGE_FLAT_DAMAGE_FACTOR
                * (rank as f64).powf(PRESTIGE_FLAT_DAMAGE_EXPONENT))
            .floor() as u64,
            flat_defense: (PRESTIGE_FLAT_DEFENSE_FACTOR
                * (rank as f64).powf(PRESTIGE_FLAT_DEFENSE_EXPONENT))
            .floor() as u64,
            crit_chance: (rank as f64 * PRESTIGE_CRIT_PER_RANK).min(PRESTIGE_CRIT_CAP),
            flat_hp: (PRESTIGE_FLAT_HP_FACTOR * (rank as f64).powf(PRESTIGE_FLAT_HP_EXPONENT))
                .floor() as u64,
        }
    }
}
