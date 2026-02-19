//! Prestige multiplier and scaling calculations.

use super::attributes::{AttributeType, Attributes};
use crate::core::constants::*;
use crate::items::Equipment;

/// Calculates prestige multiplier from base multiplier and attributes.
pub fn prestige_multiplier(base_multiplier: f64, attrs: &Attributes) -> f64 {
    let cha_mod = attrs.modifier(AttributeType::Charisma);
    base_multiplier + (cha_mod as f64 * PRESTIGE_MULT_PER_CHA_MODIFIER)
}

/// Calculates prestige multiplier with equipment bonuses included.
pub fn prestige_multiplier_with_equipment(
    base_multiplier: f64,
    attrs: &Attributes,
    equipment: &Equipment,
) -> f64 {
    // Sum equipment charisma bonuses
    let total_cha: u32 = attrs.get(AttributeType::Charisma)
        + equipment
            .iter_equipped()
            .map(|i| i.attributes.cha)
            .sum::<u32>();

    let mut temp_attrs = *attrs;
    temp_attrs.set(AttributeType::Charisma, total_cha);
    prestige_multiplier(base_multiplier, &temp_attrs)
}
