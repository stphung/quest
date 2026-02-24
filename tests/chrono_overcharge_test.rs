//! Tests for Chrono Surge overcharge proc mechanics.

use quest::stormglass::sigils::{Sigil, SigilBonuses, SigilEffectType, SigilGrade, StormSigils};
use quest::stormglass::types::ChronoSurgeState;

#[test]
fn test_overcharged_surge_has_50_percent_more_ticks() {
    let base_ticks: u64 = 36_000;
    let overcharged_ticks = (base_ticks as f64 * 1.5) as u64;

    let normal = ChronoSurgeState::new(base_ticks, false);
    let boosted = ChronoSurgeState::new(overcharged_ticks, true);

    assert_eq!(normal.ticks_total, 36_000);
    assert!(!normal.overcharged);

    assert_eq!(boosted.ticks_total, 54_000);
    assert!(boosted.overcharged);
}

#[test]
fn test_overcharge_bonus_zero_when_no_sigil() {
    let sigils = StormSigils::new();
    let bonuses = SigilBonuses::compute(&sigils);
    assert!(
        bonuses.chrono_overcharge_percent.abs() < 1e-10,
        "Should be 0 with no sigils"
    );
}

#[test]
fn test_overcharge_sigil_stacks() {
    let mut sigils = StormSigils::new();
    sigils.slots_unlocked = 3;
    sigils.sigils[0] = Some(Sigil {
        effect: SigilEffectType::ChronoOverchargePercent,
        value: 10.0,
        grade: SigilGrade::C,
    });
    sigils.sigils[1] = Some(Sigil {
        effect: SigilEffectType::DamagePercent,
        value: 5.0,
        grade: SigilGrade::C,
    });
    sigils.sigils[2] = Some(Sigil {
        effect: SigilEffectType::ChronoOverchargePercent,
        value: 7.5,
        grade: SigilGrade::D,
    });
    let bonuses = SigilBonuses::compute(&sigils);
    assert!(
        (bonuses.chrono_overcharge_percent - 17.5).abs() < 1e-10,
        "Should stack to 17.5%, got {}",
        bonuses.chrono_overcharge_percent
    );
    assert!(
        (bonuses.damage_percent - 5.0).abs() < 1e-10,
        "Damage should be unaffected"
    );
}
