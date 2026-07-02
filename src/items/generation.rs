use super::names::generate_display_name_with_rng;
use super::types::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};
use crate::core::constants::{
    ILVL_SCALING_BASE, ILVL_SCALING_DIVISOR, TIER_MULTIPLIERS, TIER_THRESHOLDS,
};
use rand::{Rng, RngExt};

/// Generate an item with the given slot, rarity, and item level.
/// ilvl determines stat scaling: ilvl 10 (zone 1) to ilvl 100 (zone 10).
pub fn generate_item(slot: EquipmentSlot, rarity: Rarity, ilvl: u32) -> Item {
    generate_item_with_rng(slot, rarity, ilvl, &mut rand::rng())
}

/// Like [`generate_item`], but rolls against a caller-provided RNG so
/// fixtures and tests can generate reproducible items from a seed.
pub fn generate_item_with_rng(
    slot: EquipmentSlot,
    rarity: Rarity,
    ilvl: u32,
    rng: &mut impl Rng,
) -> Item {
    let tier = roll_tier(rng);

    // Generate attribute bonuses based on rarity, ilvl, and tier
    let attributes = generate_attributes(rarity, ilvl, tier, rng);

    // Generate affixes based on rarity, ilvl, and tier
    let affixes = generate_affixes(rarity, ilvl, tier, rng);

    let mut item = Item {
        slot,
        rarity,
        ilvl,
        tier,
        base_name: String::new(),
        display_name: String::new(),
        attributes,
        affixes,
        god_item_id: None,
    };

    item.display_name = generate_display_name_with_rng(&item, rng);
    item.base_name = item.display_name.clone();

    item
}

/// Roll a tier (T0-T9) using an exponential drop curve.
/// T0 (38%) through T9 (0.1%).
pub fn roll_tier(rng: &mut impl Rng) -> u8 {
    let roll: f64 = rng.random();
    for (i, &threshold) in TIER_THRESHOLDS.iter().enumerate() {
        if roll < threshold {
            return i as u8;
        }
    }
    9
}

/// Returns the stat multiplier for a given tier (0.40x for T0, 1.00x for T9).
pub fn tier_multiplier(tier: u8) -> f64 {
    TIER_MULTIPLIERS
        .get(tier as usize)
        .copied()
        .unwrap_or(TIER_MULTIPLIERS[5])
}

/// Calculate the ilvl multiplier for scaling stats.
/// ilvl 10: 1.0x, ilvl 50: 2.33x, ilvl 100: 4.0x
fn ilvl_multiplier(ilvl: u32) -> f64 {
    1.0 + (ilvl.max(ILVL_SCALING_BASE as u32) as f64 - ILVL_SCALING_BASE) / ILVL_SCALING_DIVISOR
}

fn generate_attributes(
    rarity: Rarity,
    ilvl: u32,
    tier: u8,
    rng: &mut impl Rng,
) -> AttributeBonuses {
    // Base ranges at ilvl 10 (reduced from original)
    let (base_min, base_max) = match rarity {
        Rarity::Common => (1, 1),
        Rarity::Magic => (1, 2),
        Rarity::Rare => (2, 3),
        Rarity::Epic => (3, 4),
        Rarity::Legendary | Rarity::Mythic => (4, 6),
    };

    let multiplier = ilvl_multiplier(ilvl) * tier_multiplier(tier);

    // Pick a random number of attributes to boost (1-3)
    let num_attrs = rng.random_range(1..=3);
    let mut attrs = AttributeBonuses::new();

    for _ in 0..num_attrs {
        let base_value = rng.random_range(base_min..=base_max) as f64;
        let scaled_value = (base_value * multiplier).round() as u32;
        let value = scaled_value.max(1); // Minimum 1

        match rng.random_range(0..6) {
            0 => attrs.str += value,
            1 => attrs.dex += value,
            2 => attrs.con += value,
            3 => attrs.int += value,
            4 => attrs.wis += value,
            5 => attrs.cha += value,
            _ => unreachable!(),
        }
    }

    attrs
}

fn generate_affixes(rarity: Rarity, ilvl: u32, tier: u8, rng: &mut impl Rng) -> Vec<Affix> {
    let count = match rarity {
        Rarity::Common => 0,
        Rarity::Magic => 1,
        Rarity::Rare => rng.random_range(2..=3),
        Rarity::Epic => rng.random_range(3..=4),
        Rarity::Legendary | Rarity::Mythic => rng.random_range(4..=5),
    };

    let mut affixes = Vec::new();
    let all_affix_types = [
        AffixType::DamagePercent,
        AffixType::CritChance,
        AffixType::CritMultiplier,
        AffixType::AttackSpeed,
        AffixType::HPBonus,
        AffixType::DamageReduction,
        AffixType::HPRegen,
        AffixType::DamageReflection,
        AffixType::XPGain,
    ];

    for _ in 0..count {
        let affix_type = all_affix_types[rng.random_range(0..all_affix_types.len())];
        let value = generate_affix_value(affix_type, rarity, ilvl, tier, rng);
        affixes.push(Affix { affix_type, value });
    }

    affixes
}

fn generate_affix_value(
    affix_type: AffixType,
    rarity: Rarity,
    ilvl: u32,
    tier: u8,
    rng: &mut impl Rng,
) -> f64 {
    let multiplier = ilvl_multiplier(ilvl) * tier_multiplier(tier);

    // Base ranges at ilvl 10 (significantly reduced from original)
    let (base_min, base_max) = match rarity {
        Rarity::Common => (0.0, 0.0),
        Rarity::Magic => (1.0, 3.0),
        Rarity::Rare => (2.0, 4.0),
        Rarity::Epic => (4.0, 6.0),
        Rarity::Legendary | Rarity::Mythic => (6.0, 10.0),
    };

    match affix_type {
        AffixType::HPBonus => {
            // Flat HP bonus uses different base ranges
            let (hp_min, hp_max) = match rarity {
                Rarity::Common => (0.0, 0.0),
                Rarity::Magic => (10.0, 20.0),
                Rarity::Rare => (20.0, 35.0),
                Rarity::Epic => (30.0, 50.0),
                Rarity::Legendary | Rarity::Mythic => (50.0, 80.0),
            };
            let base = rng.random_range(hp_min..=hp_max);
            (base * multiplier).round()
        }
        AffixType::CritMultiplier => {
            // Crit multiplier is smaller (0.1x - 0.5x range)
            let (cm_min, cm_max) = match rarity {
                Rarity::Common => (0.0, 0.0),
                Rarity::Magic => (0.05, 0.1),
                Rarity::Rare => (0.1, 0.15),
                Rarity::Epic => (0.15, 0.25),
                Rarity::Legendary | Rarity::Mythic => (0.2, 0.35),
            };
            let base = rng.random_range(cm_min..=cm_max);
            ((base * multiplier) * 100.0).round() / 100.0 // Round to 2 decimals
        }
        AffixType::AttackSpeed => {
            // Attack speed uses 1/3 base ranges so god item Windborne (100%) stands out
            let (as_min, as_max) = match rarity {
                Rarity::Common => (0.0, 0.0),
                Rarity::Magic => (0.3, 1.0),
                Rarity::Rare => (0.7, 1.3),
                Rarity::Epic => (1.3, 2.0),
                Rarity::Legendary | Rarity::Mythic => (2.0, 3.3),
            };
            let base = rng.random_range(as_min..=as_max);
            (base * multiplier).round()
        }
        _ => {
            // Percentage affixes
            let base = rng.random_range(base_min..=base_max);
            (base * multiplier).round()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ilvl_multiplier() {
        assert!((ilvl_multiplier(10) - 1.0).abs() < 0.01);
        assert!((ilvl_multiplier(40) - 2.0).abs() < 0.01);
        assert!((ilvl_multiplier(70) - 3.0).abs() < 0.01);
        assert!((ilvl_multiplier(100) - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_generate_common_item() {
        let item = generate_item(EquipmentSlot::Weapon, Rarity::Common, 10);
        assert_eq!(item.rarity, Rarity::Common);
        assert_eq!(item.slot, EquipmentSlot::Weapon);
        assert_eq!(item.ilvl, 10);
        assert_eq!(item.affixes.len(), 0);
        assert!(item.attributes.total() > 0);
    }

    #[test]
    fn test_generate_magic_item_has_affix() {
        let item = generate_item(EquipmentSlot::Armor, Rarity::Magic, 50);
        assert_eq!(item.rarity, Rarity::Magic);
        assert_eq!(item.ilvl, 50);
        assert_eq!(item.affixes.len(), 1);
    }

    #[test]
    fn test_generate_rare_item_has_multiple_affixes() {
        let item = generate_item(EquipmentSlot::Helmet, Rarity::Rare, 100);
        assert_eq!(item.rarity, Rarity::Rare);
        assert!(item.affixes.len() >= 2 && item.affixes.len() <= 3);
    }

    #[test]
    fn test_generate_legendary_item() {
        let item = generate_item(EquipmentSlot::Weapon, Rarity::Legendary, 100);
        assert_eq!(item.rarity, Rarity::Legendary);
        assert_eq!(item.ilvl, 100);
        assert!(item.affixes.len() >= 4 && item.affixes.len() <= 5);
    }

    #[test]
    fn test_item_has_display_name() {
        let item = generate_item(EquipmentSlot::Weapon, Rarity::Magic, 50);
        assert!(!item.display_name.is_empty());
    }

    #[test]
    fn test_higher_ilvl_stronger_items() {
        // Over many samples, ilvl 100 should produce higher average totals than ilvl 10
        let sample = |ilvl: u32| -> f64 {
            let sum: u32 = (0..100)
                .map(|_| {
                    generate_item(EquipmentSlot::Weapon, Rarity::Rare, ilvl)
                        .attributes
                        .total()
                })
                .sum();
            sum as f64 / 100.0
        };

        let ilvl_10_avg = sample(10);
        let ilvl_50_avg = sample(50);
        let ilvl_100_avg = sample(100);

        assert!(
            ilvl_10_avg < ilvl_50_avg,
            "ilvl 10 ({ilvl_10_avg}) should be < ilvl 50 ({ilvl_50_avg})"
        );
        assert!(
            ilvl_50_avg < ilvl_100_avg,
            "ilvl 50 ({ilvl_50_avg}) should be < ilvl 100 ({ilvl_100_avg})"
        );
    }

    #[test]
    fn test_ilvl_100_legendary_reasonable_values() {
        // Verify ilvl 100 legendaries are in expected range
        // Tier ranges from T0 (0.40x) to T9 (1.00x), so multiply old bounds by 0.40 on low end
        for _ in 0..20 {
            let item = generate_item(EquipmentSlot::Weapon, Rarity::Legendary, 100);

            // Attributes: 4-6 base * (4.0 * 0.40..1.00) * 1-3 stats = 6-72 total
            assert!(
                item.attributes.total() >= 6,
                "Legendary attrs too low: {}",
                item.attributes.total()
            );
            assert!(
                item.attributes.total() <= 80,
                "Legendary attrs too high: {}",
                item.attributes.total()
            );

            // Check affix values are reasonable
            for affix in &item.affixes {
                match affix.affix_type {
                    AffixType::HPBonus => {
                        // 50-80 base * (4.0 * 0.40..1.00) = 80-320
                        assert!(
                            affix.value >= 60.0 && affix.value <= 350.0,
                            "HP bonus out of range: {}",
                            affix.value
                        );
                    }
                    AffixType::CritMultiplier => {
                        // 0.2-0.35 base * (4.0 * 0.40..1.00) = 0.32-1.4
                        assert!(
                            affix.value >= 0.2 && affix.value <= 1.5,
                            "Crit mult out of range: {}",
                            affix.value
                        );
                    }
                    AffixType::AttackSpeed => {
                        // 2.0-3.3 base * (4.0 * 0.40..1.00) = 3-13%
                        assert!(
                            affix.value >= 2.0 && affix.value <= 14.0,
                            "AttackSpeed out of range: {}",
                            affix.value
                        );
                    }
                    _ => {
                        // 6-10 base * (4.0 * 0.40..1.00) = 10-40%
                        assert!(
                            affix.value >= 8.0 && affix.value <= 45.0,
                            "Affix {:?} out of range: {}",
                            affix.affix_type,
                            affix.value
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_ilvl_10_items_weak() {
        // Verify ilvl 10 items are appropriately weak
        // With tier T0 (0.40x), minimum is even lower: 4 * 0.40 = 1.6 → rounds to 2
        for _ in 0..20 {
            let item = generate_item(EquipmentSlot::Weapon, Rarity::Legendary, 10);

            // Attributes: 4-6 base * (1.0 * 0.40..1.00) * 1-3 stats = 1-18 total
            assert!(
                item.attributes.total() >= 1,
                "Legendary attrs too low: {}",
                item.attributes.total()
            );
            assert!(
                item.attributes.total() <= 20,
                "Legendary attrs too high at ilvl 10: {}",
                item.attributes.total()
            );
        }
    }

    #[test]
    fn test_common_items_never_have_affixes() {
        for _ in 0..50 {
            let item = generate_item(EquipmentSlot::Gloves, Rarity::Common, 100);
            assert_eq!(
                item.affixes.len(),
                0,
                "Common items should never have affixes"
            );
        }
    }

    #[test]
    fn test_roll_tier_distribution() {
        use rand::SeedableRng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
        let mut counts = [0u32; 10];
        let n = 100_000;
        for _ in 0..n {
            let tier = roll_tier(&mut rng);
            counts[tier as usize] += 1;
        }
        // Expected percentages: T0=38%, T1=24%, T2=15%, T3=10%, T4=6%, T5=3.5%, T6=2%, T7=1%, T8=0.4%, T9=0.1%
        let expected = [
            0.380, 0.240, 0.150, 0.100, 0.060, 0.035, 0.020, 0.010, 0.004, 0.001,
        ];
        for (i, &exp) in expected.iter().enumerate() {
            let actual = counts[i] as f64 / n as f64;
            let tolerance = if exp < 0.01 { 0.005 } else { 0.02 };
            assert!(
                (actual - exp).abs() < tolerance,
                "T{} distribution off: expected {:.1}%, got {:.1}%",
                i,
                exp * 100.0,
                actual * 100.0
            );
        }
    }

    #[test]
    fn test_tier_multiplier_values() {
        let expected = [0.40, 0.47, 0.54, 0.61, 0.68, 0.74, 0.80, 0.86, 0.93, 1.00];
        for (i, &exp) in expected.iter().enumerate() {
            assert!(
                (tier_multiplier(i as u8) - exp).abs() < f64::EPSILON,
                "T{} multiplier wrong: expected {}, got {}",
                i,
                exp,
                tier_multiplier(i as u8)
            );
        }
        // Out-of-range falls back to T5
        assert!((tier_multiplier(10) - 0.74).abs() < f64::EPSILON);
        assert!((tier_multiplier(255) - 0.74).abs() < f64::EPSILON);
    }

    #[test]
    fn test_generated_items_have_valid_tier() {
        for _ in 0..100 {
            let item = generate_item(EquipmentSlot::Weapon, Rarity::Rare, 50);
            assert!(item.tier <= 9, "Tier should be 0-9, got {}", item.tier);
        }
    }

    #[test]
    fn test_tier_affects_item_power() {
        // T9 items at ilvl 50 should be stronger than T0 items on average
        use rand::SeedableRng;
        let mut t0_total = 0u64;
        let mut t9_total = 0u64;
        let samples = 200;
        for seed in 0..samples {
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
            let t0_attrs = generate_attributes(Rarity::Rare, 50, 0, &mut rng);
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
            let t9_attrs = generate_attributes(Rarity::Rare, 50, 9, &mut rng);
            t0_total += t0_attrs.total() as u64;
            t9_total += t9_attrs.total() as u64;
        }
        assert!(
            t9_total > t0_total,
            "T9 total ({}) should exceed T0 total ({})",
            t9_total,
            t0_total
        );
        // T9 should be ~2.5x T0 (1.00/0.40)
        let ratio = t9_total as f64 / t0_total as f64;
        assert!(
            ratio > 2.0 && ratio < 3.0,
            "T9/T0 ratio should be ~2.5, got {:.2}",
            ratio
        );
    }
}
