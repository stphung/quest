//! Storm Sigil types, grading, generation, and bonus aggregation.
//!
//! Storm Sigils are character-level persistent sigil slots that survive prestige
//! resets. Players spend Stormglass to unlock slots, inscribe sigils, and reroll
//! for better values. Each inscribe/reroll presents 3 randomly rolled sigils and
//! the player picks one. Rolls are graded S through F based on percentile.

use rand::RngExt;
use serde::{Deserialize, Serialize};

// ── Constants ────────────────────────────────────────────────────────────
pub const MAX_SIGIL_SLOTS: usize = 5;
pub const INSCRIBE_COST: u64 = 25_000;

/// Slot unlock costs: 25k, 50k, 100k, 200k, 400k (2x exponential)
pub const SLOT_UNLOCK_COSTS: [u64; MAX_SIGIL_SLOTS] = [25_000, 50_000, 100_000, 200_000, 400_000];

/// Exponential curve constant for value distribution (k=3).
/// Higher values compress low grades and stretch high grades.
const EXPONENTIAL_K: f64 = 3.0;

// ── Sigil Effect Types ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)] // "Percent" suffix is semantically meaningful
pub enum SigilEffectType {
    XpPercent,
    DamagePercent,
    DamageReductionPercent,
    CritChancePercent,
    DropRatePercent,
    MaxHpPercent,
    FishingSpeedPercent,
    OfflineXpPercent,
    AttackSpeedPercent,
    DoubleStrikePercent,
    RegenDelayPercent,
}

impl SigilEffectType {
    pub const ALL: [SigilEffectType; 11] = [
        Self::XpPercent,
        Self::DamagePercent,
        Self::DamageReductionPercent,
        Self::CritChancePercent,
        Self::DropRatePercent,
        Self::MaxHpPercent,
        Self::FishingSpeedPercent,
        Self::OfflineXpPercent,
        Self::AttackSpeedPercent,
        Self::DoubleStrikePercent,
        Self::RegenDelayPercent,
    ];

    /// (min, max) value range for this effect type.
    pub fn range(self) -> (f64, f64) {
        match self {
            Self::XpPercent => (5.0, 25.0),
            Self::DamagePercent => (3.0, 15.0),
            Self::DamageReductionPercent => (1.0, 5.0),
            Self::CritChancePercent => (2.0, 8.0),
            Self::DropRatePercent => (2.0, 10.0),
            Self::MaxHpPercent => (3.0, 15.0),
            Self::FishingSpeedPercent => (5.0, 25.0),
            Self::OfflineXpPercent => (5.0, 20.0),
            Self::AttackSpeedPercent => (2.0, 10.0),
            Self::DoubleStrikePercent => (1.0, 5.0),
            Self::RegenDelayPercent => (2.0, 10.0),
        }
    }

    /// Display name for this sigil effect type.
    pub fn sigil_name(self) -> &'static str {
        match self {
            Self::XpPercent => "Sigil of Wisdom",
            Self::DamagePercent => "Sigil of Fury",
            Self::DamageReductionPercent => "Sigil of the Bulwark",
            Self::CritChancePercent => "Sigil of Precision",
            Self::DropRatePercent => "Sigil of Fortune",
            Self::MaxHpPercent => "Sigil of Vitality",
            Self::FishingSpeedPercent => "Sigil of the Tide",
            Self::OfflineXpPercent => "Sigil of Echoes",
            Self::AttackSpeedPercent => "Sigil of Swiftness",
            Self::DoubleStrikePercent => "Sigil of the Twin Strike",
            Self::RegenDelayPercent => "Sigil of Renewal",
        }
    }

    /// Short label for display (e.g., "+12.3% XP").
    pub fn format_value(self, value: f64) -> String {
        match self {
            Self::RegenDelayPercent => format!("-{:.1}% Regen Delay", value),
            _ => {
                let label = match self {
                    Self::XpPercent => "XP",
                    Self::DamagePercent => "Damage",
                    Self::DamageReductionPercent => "DR",
                    Self::CritChancePercent => "Crit",
                    Self::DropRatePercent => "Drop Rate",
                    Self::MaxHpPercent => "HP",
                    Self::FishingSpeedPercent => "Fishing Speed",
                    Self::OfflineXpPercent => "Offline XP",
                    Self::AttackSpeedPercent => "Attack Speed",
                    Self::DoubleStrikePercent => "Double Strike",
                    Self::RegenDelayPercent => unreachable!(),
                };
                format!("+{:.1}% {}", value, label)
            }
        }
    }

    /// Format the range for display on pick-1-of-3 screen.
    pub fn format_range(self) -> String {
        let (min, max) = self.range();
        format!("{:.0}-{:.0}%", min, max)
    }
}

// ── Tier Grading ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SigilGrade {
    FMinus,
    F,
    FPlus,
    EMinus,
    E,
    EPlus,
    DMinus,
    D,
    DPlus,
    CMinus,
    C,
    CPlus,
    BMinus,
    B,
    BPlus,
    AMinus,
    A,
    APlus,
    SMinus,
    S,
    SPlus,
}

impl SigilGrade {
    /// Determine grade from a percentile (0.0 to 1.0).
    pub fn from_percentile(p: f64) -> Self {
        match p {
            p if p >= 0.985 => Self::SPlus,
            p if p >= 0.96 => Self::S,
            p if p >= 0.95 => Self::SMinus,
            p if p >= 0.92 => Self::APlus,
            p if p >= 0.85 => Self::A,
            p if p >= 0.80 => Self::AMinus,
            p if p >= 0.75 => Self::BPlus,
            p if p >= 0.67 => Self::B,
            p if p >= 0.60 => Self::BMinus,
            p if p >= 0.55 => Self::CPlus,
            p if p >= 0.47 => Self::C,
            p if p >= 0.40 => Self::CMinus,
            p if p >= 0.35 => Self::DPlus,
            p if p >= 0.27 => Self::D,
            p if p >= 0.20 => Self::DMinus,
            p if p >= 0.17 => Self::EPlus,
            p if p >= 0.13 => Self::E,
            p if p >= 0.10 => Self::EMinus,
            p if p >= 0.07 => Self::FPlus,
            p if p >= 0.03 => Self::F,
            _ => Self::FMinus,
        }
    }

    /// Display string for this grade (e.g., "S+", "A", "B-").
    pub fn label(self) -> &'static str {
        match self {
            Self::SPlus => "S+",
            Self::S => "S",
            Self::SMinus => "S-",
            Self::APlus => "A+",
            Self::A => "A",
            Self::AMinus => "A-",
            Self::BPlus => "B+",
            Self::B => "B",
            Self::BMinus => "B-",
            Self::CPlus => "C+",
            Self::C => "C",
            Self::CMinus => "C-",
            Self::DPlus => "D+",
            Self::D => "D",
            Self::DMinus => "D-",
            Self::EPlus => "E+",
            Self::E => "E",
            Self::EMinus => "E-",
            Self::FPlus => "F+",
            Self::F => "F",
            Self::FMinus => "F-",
        }
    }

    /// Base letter tier (for color selection).
    pub fn tier_letter(self) -> char {
        match self {
            Self::SPlus | Self::S | Self::SMinus => 'S',
            Self::APlus | Self::A | Self::AMinus => 'A',
            Self::BPlus | Self::B | Self::BMinus => 'B',
            Self::CPlus | Self::C | Self::CMinus => 'C',
            Self::DPlus | Self::D | Self::DMinus => 'D',
            Self::EPlus | Self::E | Self::EMinus => 'E',
            Self::FPlus | Self::F | Self::FMinus => 'F',
        }
    }

    /// Whether this is a + variant (for BOLD styling).
    pub fn is_plus(self) -> bool {
        matches!(
            self,
            Self::SPlus
                | Self::APlus
                | Self::BPlus
                | Self::CPlus
                | Self::DPlus
                | Self::EPlus
                | Self::FPlus
        )
    }

    /// Whether this is a - variant (for DIM styling).
    pub fn is_minus(self) -> bool {
        matches!(
            self,
            Self::SMinus
                | Self::AMinus
                | Self::BMinus
                | Self::CMinus
                | Self::DMinus
                | Self::EMinus
                | Self::FMinus
        )
    }
}

// ── Sigil Data ──────────────────────────────────────────────────────────

/// A single inscribed sigil.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sigil {
    pub effect: SigilEffectType,
    pub value: f64,
    pub grade: SigilGrade,
}

/// Character-level sigil storage, persists through prestige.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StormSigils {
    pub slots_unlocked: u8,
    pub sigils: Vec<Option<Sigil>>,
}

impl StormSigils {
    pub fn new() -> Self {
        Self {
            slots_unlocked: 0, // All slots start locked
            sigils: vec![None; MAX_SIGIL_SLOTS],
        }
    }

    /// Number of inscribed (non-empty) sigils.
    pub fn inscribed_count(&self) -> usize {
        self.sigils.iter().filter(|s| s.is_some()).count()
    }

    /// Cost to unlock the next slot, or None if all unlocked.
    pub fn next_unlock_cost(&self) -> Option<u64> {
        let next = self.slots_unlocked as usize;
        SLOT_UNLOCK_COSTS.get(next).copied()
    }
}

impl Default for StormSigils {
    fn default() -> Self {
        Self::new()
    }
}

// ── Rolling / Generation ────────────────────────────────────────────────

/// Convert a uniform random [0, 1) into an exponentially-distributed value.
/// Uses curve: f(p) = (e^(k*p) - 1) / (e^k - 1)
/// This compresses low grades and stretches S/A grades apart.
pub fn exponential_value(p: f64) -> f64 {
    let ek = EXPONENTIAL_K.exp();
    ((EXPONENTIAL_K * p).exp() - 1.0) / (ek - 1.0)
}

/// Roll a single random sigil given a uniform random value in [0, 1).
/// Returns the effect type, rolled value, and computed grade.
pub fn roll_sigil(effect: SigilEffectType, uniform_roll: f64) -> Sigil {
    let (min, max) = effect.range();
    let curved = exponential_value(uniform_roll);
    let value = min + (max - min) * curved;
    // Round to 1 decimal place
    let value = (value * 10.0).round() / 10.0;
    // Clamp to range (rounding may push slightly beyond)
    let value = value.clamp(min, max);
    let grade = SigilGrade::from_percentile(uniform_roll);
    Sigil {
        effect,
        value,
        grade,
    }
}

/// Generate 3 random sigil choices for the pick-1-of-3 screen.
/// Each sigil rolls an independent random effect type and value.
pub fn generate_sigil_choices<R: rand::Rng>(rng: &mut R) -> [Sigil; 3] {
    let types = SigilEffectType::ALL;
    std::array::from_fn(|_| {
        let effect = types[rng.random_range(0..types.len())];
        roll_sigil(effect, rng.random::<f64>())
    })
}

// ── Bonus Aggregation ───────────────────────────────────────────────────

/// Aggregated bonuses from all inscribed sigils, for parameter injection.
#[derive(Debug, Clone, Default)]
pub struct SigilBonuses {
    pub xp_percent: f64,
    pub damage_percent: f64,
    pub damage_reduction_percent: f64,
    pub crit_chance_percent: f64,
    pub drop_rate_percent: f64,
    pub max_hp_percent: f64,
    pub fishing_speed_percent: f64,
    pub offline_xp_percent: f64,
    pub attack_speed_percent: f64,
    pub double_strike_percent: f64,
    pub regen_delay_percent: f64,
}

impl SigilBonuses {
    /// Compute aggregate bonuses from all inscribed sigils.
    pub fn compute(sigils: &StormSigils) -> Self {
        let mut bonuses = Self::default();
        for sigil in sigils.sigils.iter().flatten() {
            match sigil.effect {
                SigilEffectType::XpPercent => bonuses.xp_percent += sigil.value,
                SigilEffectType::DamagePercent => bonuses.damage_percent += sigil.value,
                SigilEffectType::DamageReductionPercent => {
                    bonuses.damage_reduction_percent += sigil.value
                }
                SigilEffectType::CritChancePercent => bonuses.crit_chance_percent += sigil.value,
                SigilEffectType::DropRatePercent => bonuses.drop_rate_percent += sigil.value,
                SigilEffectType::MaxHpPercent => bonuses.max_hp_percent += sigil.value,
                SigilEffectType::FishingSpeedPercent => {
                    bonuses.fishing_speed_percent += sigil.value
                }
                SigilEffectType::OfflineXpPercent => bonuses.offline_xp_percent += sigil.value,
                SigilEffectType::AttackSpeedPercent => bonuses.attack_speed_percent += sigil.value,
                SigilEffectType::DoubleStrikePercent => {
                    bonuses.double_strike_percent += sigil.value
                }
                SigilEffectType::RegenDelayPercent => bonuses.regen_delay_percent += sigil.value,
            }
        }
        bonuses
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_sigil_effect_type_ranges_valid() {
        for effect in SigilEffectType::ALL {
            let (min, max) = effect.range();
            assert!(
                min < max,
                "{:?} range invalid: min={} max={}",
                effect,
                min,
                max
            );
            assert!(min > 0.0, "{:?} min must be positive", effect);
        }
    }

    #[test]
    fn test_sigil_grade_from_percentile() {
        // Boundary values
        assert_eq!(SigilGrade::from_percentile(0.0), SigilGrade::FMinus);
        assert_eq!(SigilGrade::from_percentile(0.02), SigilGrade::FMinus);
        assert_eq!(SigilGrade::from_percentile(0.03), SigilGrade::F);
        assert_eq!(SigilGrade::from_percentile(0.07), SigilGrade::FPlus);
        assert_eq!(SigilGrade::from_percentile(0.10), SigilGrade::EMinus);
        assert_eq!(SigilGrade::from_percentile(0.13), SigilGrade::E);
        assert_eq!(SigilGrade::from_percentile(0.17), SigilGrade::EPlus);
        assert_eq!(SigilGrade::from_percentile(0.20), SigilGrade::DMinus);
        assert_eq!(SigilGrade::from_percentile(0.27), SigilGrade::D);
        assert_eq!(SigilGrade::from_percentile(0.35), SigilGrade::DPlus);
        assert_eq!(SigilGrade::from_percentile(0.40), SigilGrade::CMinus);
        assert_eq!(SigilGrade::from_percentile(0.47), SigilGrade::C);
        assert_eq!(SigilGrade::from_percentile(0.55), SigilGrade::CPlus);
        assert_eq!(SigilGrade::from_percentile(0.60), SigilGrade::BMinus);
        assert_eq!(SigilGrade::from_percentile(0.67), SigilGrade::B);
        assert_eq!(SigilGrade::from_percentile(0.75), SigilGrade::BPlus);
        assert_eq!(SigilGrade::from_percentile(0.80), SigilGrade::AMinus);
        assert_eq!(SigilGrade::from_percentile(0.85), SigilGrade::A);
        assert_eq!(SigilGrade::from_percentile(0.92), SigilGrade::APlus);
        assert_eq!(SigilGrade::from_percentile(0.95), SigilGrade::SMinus);
        assert_eq!(SigilGrade::from_percentile(0.96), SigilGrade::S);
        assert_eq!(SigilGrade::from_percentile(0.985), SigilGrade::SPlus);
        assert_eq!(SigilGrade::from_percentile(1.0), SigilGrade::SPlus);
    }

    #[test]
    fn test_exponential_value_boundaries() {
        let at_zero = exponential_value(0.0);
        assert!(at_zero.abs() < 1e-10, "f(0) should be 0, got {}", at_zero);

        let at_one = exponential_value(1.0);
        assert!(
            (at_one - 1.0).abs() < 1e-10,
            "f(1) should be 1, got {}",
            at_one
        );
    }

    #[test]
    fn test_exponential_value_monotonic() {
        let mut prev = exponential_value(0.0);
        for i in 1..=100 {
            let p = i as f64 / 100.0;
            let val = exponential_value(p);
            assert!(val >= prev, "exponential_value not monotonic at p={}", p);
            prev = val;
        }
    }

    #[test]
    fn test_roll_sigil_within_range() {
        for effect in SigilEffectType::ALL {
            let (min, max) = effect.range();
            // Test at extremes and middle
            for &roll in &[0.0, 0.25, 0.5, 0.75, 0.999] {
                let sigil = roll_sigil(effect, roll);
                assert!(
                    sigil.value >= min && sigil.value <= max,
                    "{:?} roll={}: value {} not in [{}, {}]",
                    effect,
                    roll,
                    sigil.value,
                    min,
                    max
                );
            }
        }
    }

    #[test]
    fn test_generate_sigil_choices_produces_three() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let choices = generate_sigil_choices(&mut rng);
        assert_eq!(choices.len(), 3);
        for sigil in &choices {
            let (min, max) = sigil.effect.range();
            assert!(sigil.value >= min && sigil.value <= max);
        }
    }

    #[test]
    fn test_sigil_bonuses_compute_sums_correctly() {
        let mut sigils = StormSigils::new();
        sigils.slots_unlocked = 3;
        sigils.sigils[0] = Some(Sigil {
            effect: SigilEffectType::XpPercent,
            value: 10.0,
            grade: SigilGrade::B,
        });
        sigils.sigils[1] = Some(Sigil {
            effect: SigilEffectType::XpPercent,
            value: 15.0,
            grade: SigilGrade::A,
        });
        sigils.sigils[2] = Some(Sigil {
            effect: SigilEffectType::DamagePercent,
            value: 8.0,
            grade: SigilGrade::C,
        });

        let bonuses = SigilBonuses::compute(&sigils);
        assert!((bonuses.xp_percent - 25.0).abs() < 1e-10);
        assert!((bonuses.damage_percent - 8.0).abs() < 1e-10);
        assert!((bonuses.crit_chance_percent - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_storm_sigils_new_has_zero_slots() {
        let sigils = StormSigils::new();
        assert_eq!(sigils.slots_unlocked, 0);
        assert_eq!(sigils.sigils.len(), MAX_SIGIL_SLOTS);
        assert_eq!(sigils.inscribed_count(), 0);
    }

    #[test]
    fn test_sigil_grade_label() {
        assert_eq!(SigilGrade::SPlus.label(), "S+");
        assert_eq!(SigilGrade::S.label(), "S");
        assert_eq!(SigilGrade::SMinus.label(), "S-");
        assert_eq!(SigilGrade::APlus.label(), "A+");
        assert_eq!(SigilGrade::A.label(), "A");
        assert_eq!(SigilGrade::AMinus.label(), "A-");
        assert_eq!(SigilGrade::BPlus.label(), "B+");
        assert_eq!(SigilGrade::B.label(), "B");
        assert_eq!(SigilGrade::BMinus.label(), "B-");
        assert_eq!(SigilGrade::CPlus.label(), "C+");
        assert_eq!(SigilGrade::C.label(), "C");
        assert_eq!(SigilGrade::CMinus.label(), "C-");
        assert_eq!(SigilGrade::DPlus.label(), "D+");
        assert_eq!(SigilGrade::D.label(), "D");
        assert_eq!(SigilGrade::DMinus.label(), "D-");
        assert_eq!(SigilGrade::EPlus.label(), "E+");
        assert_eq!(SigilGrade::E.label(), "E");
        assert_eq!(SigilGrade::EMinus.label(), "E-");
        assert_eq!(SigilGrade::FPlus.label(), "F+");
        assert_eq!(SigilGrade::F.label(), "F");
        assert_eq!(SigilGrade::FMinus.label(), "F-");
    }

    #[test]
    fn test_sigil_grade_is_plus_minus() {
        // Plus variants
        assert!(SigilGrade::SPlus.is_plus());
        assert!(SigilGrade::APlus.is_plus());
        assert!(SigilGrade::BPlus.is_plus());
        assert!(SigilGrade::CPlus.is_plus());
        assert!(SigilGrade::DPlus.is_plus());
        assert!(SigilGrade::EPlus.is_plus());
        assert!(SigilGrade::FPlus.is_plus());
        assert!(!SigilGrade::S.is_plus());
        assert!(!SigilGrade::SMinus.is_plus());

        // Minus variants
        assert!(SigilGrade::SMinus.is_minus());
        assert!(SigilGrade::AMinus.is_minus());
        assert!(SigilGrade::BMinus.is_minus());
        assert!(SigilGrade::CMinus.is_minus());
        assert!(SigilGrade::DMinus.is_minus());
        assert!(SigilGrade::EMinus.is_minus());
        assert!(SigilGrade::FMinus.is_minus());
        assert!(!SigilGrade::S.is_minus());
        assert!(!SigilGrade::SPlus.is_minus());
    }

    #[test]
    fn test_slot_unlock_costs() {
        assert_eq!(SLOT_UNLOCK_COSTS[0], 25_000);
        assert_eq!(SLOT_UNLOCK_COSTS[1], 50_000);
        assert_eq!(SLOT_UNLOCK_COSTS[2], 100_000);
        assert_eq!(SLOT_UNLOCK_COSTS[3], 200_000);
        assert_eq!(SLOT_UNLOCK_COSTS[4], 400_000);
    }

    #[test]
    fn test_next_unlock_cost() {
        let mut sigils = StormSigils::new();
        // 0 slots unlocked, next is slot index 0 → 25k
        assert_eq!(sigils.next_unlock_cost(), Some(25_000));

        sigils.slots_unlocked = 1;
        assert_eq!(sigils.next_unlock_cost(), Some(50_000));

        sigils.slots_unlocked = 2;
        assert_eq!(sigils.next_unlock_cost(), Some(100_000));

        sigils.slots_unlocked = 5;
        assert_eq!(sigils.next_unlock_cost(), None);
    }

    #[test]
    fn test_sigil_grade_tier_letter() {
        assert_eq!(SigilGrade::SPlus.tier_letter(), 'S');
        assert_eq!(SigilGrade::S.tier_letter(), 'S');
        assert_eq!(SigilGrade::SMinus.tier_letter(), 'S');
        assert_eq!(SigilGrade::A.tier_letter(), 'A');
        assert_eq!(SigilGrade::BPlus.tier_letter(), 'B');
        assert_eq!(SigilGrade::CMinus.tier_letter(), 'C');
        assert_eq!(SigilGrade::D.tier_letter(), 'D');
        assert_eq!(SigilGrade::EPlus.tier_letter(), 'E');
        assert_eq!(SigilGrade::FMinus.tier_letter(), 'F');
    }

    #[test]
    fn test_sigil_grade_ordering() {
        assert!(SigilGrade::SPlus > SigilGrade::S);
        assert!(SigilGrade::S > SigilGrade::SMinus);
        assert!(SigilGrade::SMinus > SigilGrade::APlus);
        assert!(SigilGrade::APlus > SigilGrade::A);
        assert!(SigilGrade::FMinus < SigilGrade::F);
        assert!(SigilGrade::F < SigilGrade::FPlus);
    }

    #[test]
    fn test_sigil_effect_type_sigil_names() {
        // Verify all types have non-empty names
        for effect in SigilEffectType::ALL {
            assert!(
                !effect.sigil_name().is_empty(),
                "{:?} has empty name",
                effect
            );
            assert!(
                effect.sigil_name().starts_with("Sigil of"),
                "{:?} name doesn't start with 'Sigil of'",
                effect
            );
        }
    }

    #[test]
    fn test_sigil_effect_format_value() {
        assert_eq!(SigilEffectType::XpPercent.format_value(12.3), "+12.3% XP");
        assert_eq!(
            SigilEffectType::DamagePercent.format_value(8.0),
            "+8.0% Damage"
        );
        assert_eq!(
            SigilEffectType::RegenDelayPercent.format_value(6.5),
            "-6.5% Regen Delay"
        );
    }

    #[test]
    fn test_sigil_effect_format_range() {
        assert_eq!(SigilEffectType::XpPercent.format_range(), "5-25%");
        assert_eq!(SigilEffectType::DamagePercent.format_range(), "3-15%");
        assert_eq!(
            SigilEffectType::DamageReductionPercent.format_range(),
            "1-5%"
        );
    }

    #[test]
    fn test_storm_sigils_inscribed_count() {
        let mut sigils = StormSigils::new();
        assert_eq!(sigils.inscribed_count(), 0);

        sigils.sigils[0] = Some(Sigil {
            effect: SigilEffectType::XpPercent,
            value: 10.0,
            grade: SigilGrade::C,
        });
        assert_eq!(sigils.inscribed_count(), 1);

        sigils.sigils[2] = Some(Sigil {
            effect: SigilEffectType::DamagePercent,
            value: 5.0,
            grade: SigilGrade::D,
        });
        assert_eq!(sigils.inscribed_count(), 2);
    }

    #[test]
    fn test_storm_sigils_default() {
        let sigils = StormSigils::default();
        assert_eq!(sigils.slots_unlocked, 0);
        assert_eq!(sigils.sigils.len(), MAX_SIGIL_SLOTS);
    }

    #[test]
    fn test_sigil_serde_round_trip() {
        let sigil = Sigil {
            effect: SigilEffectType::CritChancePercent,
            value: 5.5,
            grade: SigilGrade::APlus,
        };
        let json = serde_json::to_string(&sigil).unwrap();
        let loaded: Sigil = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.effect, SigilEffectType::CritChancePercent);
        assert!((loaded.value - 5.5).abs() < 1e-10);
        assert_eq!(loaded.grade, SigilGrade::APlus);
    }

    #[test]
    fn test_storm_sigils_serde_round_trip() {
        let mut sigils = StormSigils::new();
        sigils.slots_unlocked = 3;
        sigils.sigils[0] = Some(Sigil {
            effect: SigilEffectType::XpPercent,
            value: 20.0,
            grade: SigilGrade::S,
        });
        sigils.sigils[1] = Some(Sigil {
            effect: SigilEffectType::DamagePercent,
            value: 7.5,
            grade: SigilGrade::B,
        });

        let json = serde_json::to_string(&sigils).unwrap();
        let loaded: StormSigils = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.slots_unlocked, 3);
        assert!(loaded.sigils[0].is_some());
        assert!(loaded.sigils[1].is_some());
        assert!(loaded.sigils[2].is_none());
        assert_eq!(loaded.inscribed_count(), 2);
    }

    #[test]
    fn test_sigil_bonuses_all_effects() {
        let mut sigils = StormSigils::new();
        sigils.slots_unlocked = 5;

        // Fill slots with different effects
        sigils.sigils[0] = Some(Sigil {
            effect: SigilEffectType::XpPercent,
            value: 10.0,
            grade: SigilGrade::C,
        });
        sigils.sigils[1] = Some(Sigil {
            effect: SigilEffectType::DamagePercent,
            value: 8.0,
            grade: SigilGrade::C,
        });
        sigils.sigils[2] = Some(Sigil {
            effect: SigilEffectType::CritChancePercent,
            value: 5.0,
            grade: SigilGrade::C,
        });
        sigils.sigils[3] = Some(Sigil {
            effect: SigilEffectType::RegenDelayPercent,
            value: 6.0,
            grade: SigilGrade::C,
        });
        sigils.sigils[4] = Some(Sigil {
            effect: SigilEffectType::FishingSpeedPercent,
            value: 15.0,
            grade: SigilGrade::C,
        });

        let bonuses = SigilBonuses::compute(&sigils);
        assert!((bonuses.xp_percent - 10.0).abs() < 1e-10);
        assert!((bonuses.damage_percent - 8.0).abs() < 1e-10);
        assert!((bonuses.crit_chance_percent - 5.0).abs() < 1e-10);
        assert!((bonuses.regen_delay_percent - 6.0).abs() < 1e-10);
        assert!((bonuses.fishing_speed_percent - 15.0).abs() < 1e-10);
        // Unset effects should be zero
        assert!((bonuses.damage_reduction_percent - 0.0).abs() < 1e-10);
        assert!((bonuses.drop_rate_percent - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_sigil_bonuses_empty_sigils() {
        let sigils = StormSigils::new();
        let bonuses = SigilBonuses::compute(&sigils);
        assert!((bonuses.xp_percent - 0.0).abs() < 1e-10);
        assert!((bonuses.damage_percent - 0.0).abs() < 1e-10);
        assert!((bonuses.crit_chance_percent - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_roll_sigil_low_roll_gives_low_value() {
        let sigil = roll_sigil(SigilEffectType::XpPercent, 0.01);
        // Low roll should give value near min (5.0)
        assert!(
            sigil.value < 10.0,
            "Low roll gave high value: {}",
            sigil.value
        );
        assert_eq!(sigil.grade, SigilGrade::FMinus);
    }

    #[test]
    fn test_roll_sigil_high_roll_gives_high_value() {
        let sigil = roll_sigil(SigilEffectType::XpPercent, 0.99);
        // High roll should give value near max (25.0)
        assert!(
            sigil.value > 20.0,
            "High roll gave low value: {}",
            sigil.value
        );
        assert!(sigil.grade >= SigilGrade::SPlus);
    }
}
