//! Derived stats calculation engine.

use super::attributes::{AttributeType, Attributes};
use crate::core::constants::*;
use crate::items::Equipment;

/// Calculate derived stats from attributes and equipment bonuses.
///
/// Equipment bonuses are added to base attributes before calculating modifiers.
/// Affixes are then applied as multipliers/bonuses to the calculated stats.
/// Enhancement levels scale item attribute and affix contributions per slot.
pub fn calculate_derived_stats(
    attrs: &Attributes,
    equipment: &Equipment,
    enhancement_levels: &[u8; 7],
) -> super::DerivedStats {
    // Sum equipment attribute bonuses (scaled by enhancement)
    let slots = [
        &equipment.weapon,
        &equipment.armor,
        &equipment.helmet,
        &equipment.gloves,
        &equipment.boots,
        &equipment.amulet,
        &equipment.ring,
    ];

    let mut total_attrs = *attrs;
    for (idx, slot_item) in slots.iter().enumerate() {
        if let Some(item) = slot_item {
            let mult = crate::enhancement::enhancement_multiplier(enhancement_levels[idx]);
            let attrs_bonus = item.attributes.to_attributes();
            total_attrs.add_scaled(&attrs_bonus, mult);
        }
    }

    let str_mod = total_attrs.modifier(AttributeType::Strength);
    let dex_mod = total_attrs.modifier(AttributeType::Dexterity);
    let con_mod = total_attrs.modifier(AttributeType::Constitution);
    let int_mod = total_attrs.modifier(AttributeType::Intelligence);
    let wis_mod = total_attrs.modifier(AttributeType::Wisdom);

    // Max HP = BASE_HP + (CON_mod × HP_PER_CON_MODIFIER)
    let mut max_hp = (BASE_HP + con_mod * HP_PER_CON_MODIFIER).max(1) as u32;

    // Physical Damage = BASE_PHYSICAL_DAMAGE + (STR_mod × DAMAGE_PER_STR_MODIFIER)
    let mut physical_damage =
        (BASE_PHYSICAL_DAMAGE + str_mod * DAMAGE_PER_STR_MODIFIER).max(1) as u32;

    // Magic Damage = BASE_MAGIC_DAMAGE + (INT_mod × DAMAGE_PER_INT_MODIFIER)
    let mut magic_damage = (BASE_MAGIC_DAMAGE + int_mod * DAMAGE_PER_INT_MODIFIER).max(1) as u32;

    // Defense = 0 + (DEX_mod × 1)
    let mut defense = dex_mod.max(0) as u32;

    // Crit Chance = BASE_CRIT_CHANCE_PERCENT + (DEX_mod × 1%)
    let mut crit_chance_percent = (BASE_CRIT_CHANCE_PERCENT + dex_mod).max(0) as u32;

    // XP Multiplier = 1.0 + (WIS_mod × XP_MULT_PER_WIS_MODIFIER)
    let mut xp_multiplier = 1.0 + (wis_mod as f64 * XP_MULT_PER_WIS_MODIFIER);

    // Apply equipment affixes as multipliers/bonuses
    let mut hp_bonus: f64 = 0.0;
    let mut damage_mult: f64 = 1.0;
    let mut defense_mult: f64 = 1.0;
    let mut crit_bonus: f64 = 0.0;
    let mut crit_mult_bonus: f64 = 0.0;
    let mut attack_speed_bonus: f64 = 0.0;
    let mut hp_regen_bonus: f64 = 0.0;
    let mut damage_reflection: f64 = 0.0;
    let mut xp_mult: f64 = 1.0;

    for (idx, slot_item) in slots.iter().enumerate() {
        if let Some(item) = slot_item {
            let mult = crate::enhancement::enhancement_multiplier(enhancement_levels[idx]);
            for affix in &item.affixes {
                use crate::items::types::AffixType;
                let scaled_value = affix.value * mult;
                match affix.affix_type {
                    AffixType::DamagePercent => {
                        damage_mult *= 1.0 + (scaled_value / AFFIX_PERCENT_DIVISOR)
                    }
                    AffixType::CritChance => crit_bonus += scaled_value,
                    AffixType::CritMultiplier => crit_mult_bonus += scaled_value,
                    AffixType::AttackSpeed => attack_speed_bonus += scaled_value,
                    AffixType::HPBonus => hp_bonus += scaled_value,
                    AffixType::DamageReduction => {
                        defense_mult *= 1.0 + (scaled_value / AFFIX_PERCENT_DIVISOR)
                    }
                    AffixType::HPRegen => hp_regen_bonus += scaled_value,
                    AffixType::DamageReflection => damage_reflection += scaled_value,
                    AffixType::XPGain => xp_mult *= 1.0 + (scaled_value / AFFIX_PERCENT_DIVISOR),
                    AffixType::Unknown => {}
                }
            }
        }
    }

    // Apply multipliers to stats
    max_hp = ((max_hp as f64 + hp_bonus) as u32).max(1);
    physical_damage = ((physical_damage as f64 * damage_mult) as u32).max(1);
    magic_damage = ((magic_damage as f64 * damage_mult) as u32).max(1);
    defense = (defense as f64 * defense_mult) as u32;
    crit_chance_percent = (crit_chance_percent as f64 + crit_bonus) as u32;
    xp_multiplier *= xp_mult;

    // Base crit multiplier, affix adds percentage (e.g., +50% means 2.5x)
    let crit_multiplier = BASE_CRIT_MULTIPLIER + (crit_mult_bonus / AFFIX_PERCENT_DIVISOR);

    // Attack speed: higher = faster attacks (1.0 = normal, 1.25 = 25% faster)
    let attack_speed_multiplier = 1.0 + (attack_speed_bonus / AFFIX_PERCENT_DIVISOR);

    // HP regen: higher = faster regen (1.0 = normal, 1.5 = 50% faster)
    let hp_regen_multiplier = 1.0 + (hp_regen_bonus / AFFIX_PERCENT_DIVISOR);

    // Damage reflection: percentage of damage taken reflected back to attacker
    let damage_reflection_percent = damage_reflection;

    super::DerivedStats {
        max_hp,
        physical_damage,
        magic_damage,
        defense,
        crit_chance_percent,
        crit_multiplier,
        attack_speed_multiplier,
        hp_regen_multiplier,
        damage_reflection_percent,
        xp_multiplier,
    }
}
