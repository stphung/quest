//! Layer management, familiarity system, and infrastructure logic for The Deep.
//!
//! This module provides:
//! - `FamiliarityLevel` — named thresholds with duration and quality effects
//! - `layer_difficulty` — power thresholds by layer and mission type
//! - `base_mission_duration_secs` — tier-based durations before modifiers
//! - `apply_duration_modifiers` — full duration calculation pipeline
//! - `familiarity_gain` — how much familiarity each mission type awards
//! - `build_infrastructure` — validate and apply infrastructure to a `LayerRecord`
//! - `infrastructure_build_cost` — Mark cost to build each infrastructure type

use crate::deep::types::{
    DeepPersistent, Infrastructure, LayerRecord, LayerTier, MissionType,
};

// ── Familiarity ───────────────────────────────────────────────────────────────

/// Named band for a layer's familiarity percentage (0-100).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FamiliarityLevel {
    /// 0–24%: Poor auto-resolve, no duration bonus.
    Unknown,
    /// 25–49%: -10% duration, fair auto-resolve.
    Mapped,
    /// 50–74%: -20% duration, good auto-resolve.
    Familiar,
    /// 75–100%: -30% duration, excellent auto-resolve, +15% Mark yield.
    Mastered,
}

impl FamiliarityLevel {
    /// Classify a raw familiarity percentage (0–100) into a named level.
    pub fn from_familiarity(familiarity: u8) -> Self {
        match familiarity {
            0..=24 => FamiliarityLevel::Unknown,
            25..=49 => FamiliarityLevel::Mapped,
            50..=74 => FamiliarityLevel::Familiar,
            _ => FamiliarityLevel::Mastered,
        }
    }

    /// Mission duration reduction factor (1.0 = no reduction, 0.7 = -30%).
    pub fn duration_factor(self) -> f64 {
        match self {
            FamiliarityLevel::Unknown => 1.0,
            FamiliarityLevel::Mapped => 0.90,
            FamiliarityLevel::Familiar => 0.80,
            FamiliarityLevel::Mastered => 0.70,
        }
    }

    /// Auto-resolve success rate (fraction 0.0–1.0).
    pub fn auto_resolve_success_rate(self) -> f64 {
        match self {
            FamiliarityLevel::Unknown => 0.65,
            FamiliarityLevel::Mapped => 0.75,
            FamiliarityLevel::Familiar => 0.85,
            FamiliarityLevel::Mastered => 0.95,
        }
    }

    /// Bonus Mark yield multiplier (1.0 = no bonus, 1.15 = +15%).
    pub fn mark_yield_multiplier(self) -> f64 {
        match self {
            FamiliarityLevel::Mastered => 1.15,
            _ => 1.0,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            FamiliarityLevel::Unknown => "Unknown",
            FamiliarityLevel::Mapped => "Mapped",
            FamiliarityLevel::Familiar => "Familiar",
            FamiliarityLevel::Mastered => "Mastered",
        }
    }
}

// ── Familiarity Gain ──────────────────────────────────────────────────────────

/// Amount of familiarity (0–100 scale) gained from completing a mission.
///
/// Familiarity is capped at 100 and never decreases.
/// The Watchtower infrastructure grants an immediate bonus on construction
/// (handled separately in `build_infrastructure`).
pub fn familiarity_gain(mission_type: MissionType) -> u8 {
    match mission_type {
        MissionType::SupplyRun => 5,
        MissionType::Recon => 15,
        MissionType::Expedition => 10,
        MissionType::Breakthrough => 15,
        MissionType::Construction(_) => 5,
    }
}

/// Apply familiarity gain to a layer record, capped at 100.
pub fn apply_familiarity_gain(record: &mut LayerRecord, mission_type: MissionType) {
    let gain = familiarity_gain(mission_type);
    record.familiarity = record.familiarity.saturating_add(gain).min(100);
}

// ── Layer Difficulty ─────────────────────────────────────────────────────────

/// Power thresholds for each layer, indexed by mission type.
///
/// Returns the minimum total squad Power recommended for a comfortable run.
/// Breakthrough threshold is the primary value; others are fractions of it.
pub struct LayerPowerThresholds {
    pub breakthrough: u32,
    pub expedition: u32,
    pub recon: u32,
    pub supply_run: u32,
}

/// Return the squad Power thresholds for a given 1-based layer index.
///
/// For Void layers (26+), thresholds scale linearly beyond layer 25.
pub fn layer_power_thresholds(layer: u32) -> LayerPowerThresholds {
    let layer = layer.max(1);

    // Void scaling constants (per layer above 25)
    const VOID_BREAK: u32 = 80;
    const VOID_EXP: u32 = 60;
    const VOID_REC: u32 = 45;
    const VOID_SUP: u32 = 30;

    if layer >= 26 {
        let extra = layer - 25;
        return LayerPowerThresholds {
            breakthrough: 930 + VOID_BREAK * extra,
            expedition: 700 + VOID_EXP * extra,
            recon: 525 + VOID_REC * extra,
            supply_run: 350 + VOID_SUP * extra,
        };
    }

    // Lookup table for layers 1–25 from the balance design document.
    let (breakthrough, expedition, recon, supply_run) = match layer {
        1  => (25,  20,  15,  10),
        2  => (40,  30,  20,  15),
        3  => (55,  40,  30,  20),
        4  => (75,  55,  40,  25),
        5  => (95,  70,  50,  30),
        6  => (115, 85,  60,  40),
        7  => (130, 100, 75,  50),
        8  => (155, 115, 85,  55),
        9  => (180, 135, 100, 65),
        10 => (205, 155, 115, 75),
        11 => (230, 175, 130, 85),
        12 => (260, 195, 145, 95),
        13 => (295, 220, 165, 110),
        14 => (330, 250, 185, 125),
        15 => (370, 280, 210, 140),
        16 => (410, 310, 230, 155),
        17 => (450, 340, 255, 170),
        18 => (495, 370, 280, 185),
        19 => (545, 410, 310, 205),
        20 => (600, 450, 340, 225),
        21 => (660, 495, 370, 250),
        22 => (720, 540, 405, 275),
        23 => (785, 590, 440, 300),
        24 => (855, 640, 480, 325),
        25 => (930, 700, 525, 350),
        _  => unreachable!(), // layers >= 26 handled above
    };

    LayerPowerThresholds { breakthrough, expedition, recon, supply_run }
}

/// Return the power threshold relevant for a given mission type on a layer.
pub fn mission_power_threshold(layer: u32, mission_type: MissionType) -> u32 {
    let t = layer_power_thresholds(layer);
    match mission_type {
        MissionType::Breakthrough => t.breakthrough,
        MissionType::Expedition => t.expedition,
        MissionType::Recon => t.recon,
        MissionType::SupplyRun | MissionType::Construction(_) => t.supply_run,
    }
}

// ── Mission Durations ─────────────────────────────────────────────────────────

/// Minimum wall-clock mission duration in seconds (30 minutes).
pub const MIN_MISSION_DURATION_SECS: u64 = 30 * 60;

/// Base mission duration in seconds before any modifiers, keyed on layer tier.
///
/// Matches the balance design table (§2, Base Durations by Mission Type and Tier).
pub fn base_mission_duration_secs(tier: LayerTier, mission_type: MissionType) -> u64 {
    let hours: f64 = match (tier, mission_type) {
        // Supply Run
        (LayerTier::Shallows,    MissionType::SupplyRun)       => 2.0,
        (LayerTier::Warrens,     MissionType::SupplyRun)       => 2.5,
        (LayerTier::Hollows,     MissionType::SupplyRun)       => 3.0,
        (LayerTier::SunkenReach, MissionType::SupplyRun)       => 3.5,
        (LayerTier::Abyss,       MissionType::SupplyRun)       => 4.0,
        (LayerTier::Void,        MissionType::SupplyRun)       => 4.0,
        // Recon
        (LayerTier::Shallows,    MissionType::Recon)           => 4.0,
        (LayerTier::Warrens,     MissionType::Recon)           => 5.0,
        (LayerTier::Hollows,     MissionType::Recon)           => 6.0,
        (LayerTier::SunkenReach, MissionType::Recon)           => 7.0,
        (LayerTier::Abyss,       MissionType::Recon)           => 8.0,
        (LayerTier::Void,        MissionType::Recon)           => 8.0,
        // Expedition
        (LayerTier::Shallows,    MissionType::Expedition)      => 8.0,
        (LayerTier::Warrens,     MissionType::Expedition)      => 10.0,
        (LayerTier::Hollows,     MissionType::Expedition)      => 12.0,
        (LayerTier::SunkenReach, MissionType::Expedition)      => 14.0,
        (LayerTier::Abyss,       MissionType::Expedition)      => 16.0,
        (LayerTier::Void,        MissionType::Expedition)      => 16.0,
        // Breakthrough
        (LayerTier::Shallows,    MissionType::Breakthrough)    => 18.0,
        (LayerTier::Warrens,     MissionType::Breakthrough)    => 20.0,
        (LayerTier::Hollows,     MissionType::Breakthrough)    => 22.0,
        (LayerTier::SunkenReach, MissionType::Breakthrough)    => 24.0,
        (LayerTier::Abyss,       MissionType::Breakthrough)    => 24.0,
        (LayerTier::Void,        MissionType::Breakthrough)    => 24.0,
        // Construction
        (LayerTier::Shallows,    MissionType::Construction(_)) => 4.0,
        (LayerTier::Warrens,     MissionType::Construction(_)) => 5.0,
        (LayerTier::Hollows,     MissionType::Construction(_)) => 6.0,
        (LayerTier::SunkenReach, MissionType::Construction(_)) => 7.0,
        (LayerTier::Abyss,       MissionType::Construction(_)) => 8.0,
        (LayerTier::Void,        MissionType::Construction(_)) => 8.0,
    };
    (hours * 3600.0) as u64
}

/// Parameters that can reduce mission duration beyond the base value.
pub struct DurationModifiers {
    /// Whether an Outpost has been built on this layer.
    pub has_outpost: bool,
    /// Raw familiarity percentage (0–100) for this layer.
    pub familiarity: u8,
    /// Whether a Saboteur is in the squad.
    pub has_saboteur: bool,
    /// Whether the Saboteur (if present) is Level 10+.
    pub saboteur_is_veteran: bool,
    /// Whether the squad Power exceeds 150% of the mission's threshold.
    pub is_overpowered: bool,
}

/// Apply all duration modifiers multiplicatively and enforce the minimum floor.
///
/// Modifiers are applied in order: Outpost → Familiarity → Saboteur → Overpower.
/// The result is clamped to `MIN_MISSION_DURATION_SECS`.
pub fn apply_duration_modifiers(base_secs: u64, mods: &DurationModifiers) -> u64 {
    let mut duration = base_secs as f64;

    if mods.has_outpost {
        duration *= 1.0 - Infrastructure::Outpost.duration_reduction(); // -25%
    }

    let fam = FamiliarityLevel::from_familiarity(mods.familiarity);
    duration *= fam.duration_factor();

    if mods.has_saboteur {
        let saboteur_factor = if mods.saboteur_is_veteran { 0.85 } else { 0.90 };
        duration *= saboteur_factor;
    }

    if mods.is_overpowered {
        duration *= 0.90; // -10%
    }

    (duration as u64).max(MIN_MISSION_DURATION_SECS)
}

// ── Infrastructure ────────────────────────────────────────────────────────────

/// Mark cost to build a specific infrastructure type on a given 1-based layer.
///
/// Costs scale with layer depth. See balance design §6.
pub fn infrastructure_build_cost(infra: Infrastructure, layer: u32) -> u32 {
    let layer = layer.max(1);
    match infra {
        Infrastructure::Outpost => 60 + 4 * layer,
        Infrastructure::SupplyCache => 80 + 5 * layer,
        Infrastructure::Watchtower => 70 + 4 * layer,
        Infrastructure::Bridge => 100 + 5 * layer,
    }
}

/// Errors that can occur when attempting to build infrastructure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InfrastructureBuildError {
    /// The layer has not been cleared (breakthrough not yet completed).
    LayerNotCleared,
    /// This infrastructure type is already built on this layer.
    AlreadyBuilt,
}

/// Attempt to build `infra` on `record`.
///
/// Validates preconditions (cleared, not duplicate) then mutates the record.
/// The Warband Marks cost check and deduction must be handled by the caller.
///
/// If the built infrastructure is a Watchtower, immediately applies the +25
/// familiarity bonus.
pub fn build_infrastructure(
    record: &mut LayerRecord,
    infra: Infrastructure,
) -> Result<(), InfrastructureBuildError> {
    if !record.cleared {
        return Err(InfrastructureBuildError::LayerNotCleared);
    }
    if record.infrastructure.contains(&infra) {
        return Err(InfrastructureBuildError::AlreadyBuilt);
    }

    record.infrastructure.push(infra);

    // Watchtower grants an immediate +25 familiarity bonus.
    if infra == Infrastructure::Watchtower {
        record.familiarity = record.familiarity.saturating_add(25).min(100);
    }

    Ok(())
}

// ── Layer Record Helpers ───────────────────────────────────────────────────────

/// Mark a layer as cleared (breakthrough completed).
///
/// Also records the highest layer reached in `DeepPersistent`.
pub fn mark_layer_cleared(persistent: &mut DeepPersistent, layer: u32) {
    let record = persistent.layer_record_mut(layer);
    record.cleared = true;
    if layer > persistent.deepest_layer_reached {
        persistent.deepest_layer_reached = layer;
    }
}

/// Return `true` if the given layer is the current frontier.
///
/// The frontier is the first uncleared layer. If all existing layers are
/// cleared, the frontier is `deepest_layer_reached + 1`.
pub fn is_frontier_layer(persistent: &DeepPersistent, layer: u32) -> bool {
    persistent.frontier_layer() == layer
}

/// Whether a layer is safe (cleared) or risky (frontier/beyond).
pub fn is_safe_layer(persistent: &DeepPersistent, layer: u32) -> bool {
    persistent
        .layer_record(layer)
        .map(|r| r.cleared)
        .unwrap_or(false)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep::types::{DeepPersistent, LayerRecord};

    // ── FamiliarityLevel ──────────────────────────────────────────────────────

    #[test]
    fn test_familiarity_level_boundaries() {
        assert_eq!(FamiliarityLevel::from_familiarity(0), FamiliarityLevel::Unknown);
        assert_eq!(FamiliarityLevel::from_familiarity(24), FamiliarityLevel::Unknown);
        assert_eq!(FamiliarityLevel::from_familiarity(25), FamiliarityLevel::Mapped);
        assert_eq!(FamiliarityLevel::from_familiarity(49), FamiliarityLevel::Mapped);
        assert_eq!(FamiliarityLevel::from_familiarity(50), FamiliarityLevel::Familiar);
        assert_eq!(FamiliarityLevel::from_familiarity(74), FamiliarityLevel::Familiar);
        assert_eq!(FamiliarityLevel::from_familiarity(75), FamiliarityLevel::Mastered);
        assert_eq!(FamiliarityLevel::from_familiarity(100), FamiliarityLevel::Mastered);
    }

    #[test]
    fn test_familiarity_level_duration_factors() {
        assert_eq!(FamiliarityLevel::Unknown.duration_factor(), 1.0);
        assert_eq!(FamiliarityLevel::Mapped.duration_factor(), 0.90);
        assert_eq!(FamiliarityLevel::Familiar.duration_factor(), 0.80);
        assert_eq!(FamiliarityLevel::Mastered.duration_factor(), 0.70);
    }

    #[test]
    fn test_familiarity_level_auto_resolve_rates() {
        assert_eq!(FamiliarityLevel::Unknown.auto_resolve_success_rate(), 0.65);
        assert_eq!(FamiliarityLevel::Mapped.auto_resolve_success_rate(), 0.75);
        assert_eq!(FamiliarityLevel::Familiar.auto_resolve_success_rate(), 0.85);
        assert_eq!(FamiliarityLevel::Mastered.auto_resolve_success_rate(), 0.95);
    }

    #[test]
    fn test_mastered_mark_yield_bonus() {
        assert_eq!(FamiliarityLevel::Unknown.mark_yield_multiplier(), 1.0);
        assert_eq!(FamiliarityLevel::Mapped.mark_yield_multiplier(), 1.0);
        assert_eq!(FamiliarityLevel::Familiar.mark_yield_multiplier(), 1.0);
        assert_eq!(FamiliarityLevel::Mastered.mark_yield_multiplier(), 1.15);
    }

    // ── Familiarity Gain ─────────────────────────────────────────────────────

    #[test]
    fn test_familiarity_gain_values() {
        assert_eq!(familiarity_gain(MissionType::SupplyRun), 5);
        assert_eq!(familiarity_gain(MissionType::Recon), 15);
        assert_eq!(familiarity_gain(MissionType::Expedition), 10);
        assert_eq!(familiarity_gain(MissionType::Breakthrough), 15);
        assert_eq!(familiarity_gain(MissionType::Construction(Infrastructure::Outpost)), 5);
    }

    #[test]
    fn test_apply_familiarity_gain_caps_at_100() {
        let mut record = LayerRecord::new(1);
        record.familiarity = 95;
        apply_familiarity_gain(&mut record, MissionType::Recon); // +15 would overflow
        assert_eq!(record.familiarity, 100);
    }

    #[test]
    fn test_apply_familiarity_gain_accumulates() {
        let mut record = LayerRecord::new(1);
        apply_familiarity_gain(&mut record, MissionType::SupplyRun); // +5
        apply_familiarity_gain(&mut record, MissionType::Recon);     // +15
        assert_eq!(record.familiarity, 20);
    }

    // ── Power Thresholds ─────────────────────────────────────────────────────

    #[test]
    fn test_power_thresholds_layer_1() {
        let t = layer_power_thresholds(1);
        assert_eq!(t.breakthrough, 25);
        assert_eq!(t.expedition, 20);
        assert_eq!(t.recon, 15);
        assert_eq!(t.supply_run, 10);
    }

    #[test]
    fn test_power_thresholds_layer_25() {
        let t = layer_power_thresholds(25);
        assert_eq!(t.breakthrough, 930);
        assert_eq!(t.expedition, 700);
        assert_eq!(t.recon, 525);
        assert_eq!(t.supply_run, 350);
    }

    #[test]
    fn test_power_thresholds_void_layer_26() {
        let t = layer_power_thresholds(26);
        // Void: 930 + 80*(26-25) = 1010, etc.
        assert_eq!(t.breakthrough, 1010);
        assert_eq!(t.expedition, 760);
        assert_eq!(t.recon, 570);
        assert_eq!(t.supply_run, 380);
    }

    #[test]
    fn test_power_thresholds_increase_with_depth() {
        for layer in 1..25 {
            let a = layer_power_thresholds(layer);
            let b = layer_power_thresholds(layer + 1);
            assert!(
                b.breakthrough > a.breakthrough,
                "Breakthrough threshold should increase: layer {} -> {}",
                layer, layer + 1
            );
        }
    }

    #[test]
    fn test_mission_power_threshold_selects_correct_tier() {
        // Breakthrough should return the breakthrough threshold.
        assert_eq!(mission_power_threshold(1, MissionType::Breakthrough), 25);
        assert_eq!(mission_power_threshold(1, MissionType::Expedition), 20);
        assert_eq!(mission_power_threshold(1, MissionType::Recon), 15);
        assert_eq!(mission_power_threshold(1, MissionType::SupplyRun), 10);
        // Construction uses supply_run threshold.
        assert_eq!(
            mission_power_threshold(1, MissionType::Construction(Infrastructure::Outpost)),
            10
        );
    }

    // ── Base Durations ───────────────────────────────────────────────────────

    #[test]
    fn test_base_durations_shallows() {
        assert_eq!(
            base_mission_duration_secs(LayerTier::Shallows, MissionType::SupplyRun),
            2 * 3600
        );
        assert_eq!(
            base_mission_duration_secs(LayerTier::Shallows, MissionType::Breakthrough),
            18 * 3600
        );
    }

    #[test]
    fn test_base_durations_increase_with_tier() {
        let tiers = [
            LayerTier::Shallows,
            LayerTier::Warrens,
            LayerTier::Hollows,
            LayerTier::SunkenReach,
            LayerTier::Abyss,
        ];
        for mission in [
            MissionType::SupplyRun,
            MissionType::Recon,
            MissionType::Expedition,
        ] {
            let mut prev = 0u64;
            for &tier in &tiers {
                let d = base_mission_duration_secs(tier, mission);
                assert!(d >= prev, "{:?} {:?} duration should not decrease", tier, mission);
                prev = d;
            }
        }
    }

    // ── Duration Modifiers ───────────────────────────────────────────────────

    #[test]
    fn test_duration_no_modifiers_returns_base() {
        let base = 8 * 3600;
        let mods = DurationModifiers {
            has_outpost: false,
            familiarity: 0,
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
        };
        assert_eq!(apply_duration_modifiers(base, &mods), base as u64);
    }

    #[test]
    fn test_duration_outpost_reduces_by_25_percent() {
        let base = 8 * 3600u64;
        let mods = DurationModifiers {
            has_outpost: true,
            familiarity: 0,
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
        };
        let result = apply_duration_modifiers(base, &mods);
        let expected = (base as f64 * 0.75) as u64;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_duration_mastered_familiarity_reduces_by_30_percent() {
        let base = 8 * 3600u64;
        let mods = DurationModifiers {
            has_outpost: false,
            familiarity: 80, // Mastered
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
        };
        let result = apply_duration_modifiers(base, &mods);
        let expected = (base as f64 * 0.70) as u64;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_duration_stacks_multiplicatively() {
        // Outpost -25%, Mastered -30%, Saboteur veteran -15%, Overpowered -10%
        // Net: 0.75 * 0.70 * 0.85 * 0.90 ≈ 0.4016
        let base = 8 * 3600u64;
        let mods = DurationModifiers {
            has_outpost: true,
            familiarity: 80,
            has_saboteur: true,
            saboteur_is_veteran: true,
            is_overpowered: true,
        };
        let result = apply_duration_modifiers(base, &mods);
        let expected = (base as f64 * 0.75 * 0.70 * 0.85 * 0.90) as u64;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_duration_clamped_to_minimum() {
        // Very short base with all reductions would go below 30 minutes.
        let base = 30 * 60u64; // already at minimum
        let mods = DurationModifiers {
            has_outpost: true,
            familiarity: 80,
            has_saboteur: true,
            saboteur_is_veteran: true,
            is_overpowered: true,
        };
        let result = apply_duration_modifiers(base, &mods);
        assert_eq!(result, MIN_MISSION_DURATION_SECS);
    }

    // ── Infrastructure Build Cost ─────────────────────────────────────────────

    #[test]
    fn test_infrastructure_build_cost_outpost_layer_1() {
        // 60 + 4*1 = 64
        assert_eq!(infrastructure_build_cost(Infrastructure::Outpost, 1), 64);
    }

    #[test]
    fn test_infrastructure_build_cost_supply_cache_layer_10() {
        // 80 + 5*10 = 130
        assert_eq!(infrastructure_build_cost(Infrastructure::SupplyCache, 10), 130);
    }

    #[test]
    fn test_infrastructure_build_cost_watchtower_layer_1() {
        // 70 + 4*1 = 74
        assert_eq!(infrastructure_build_cost(Infrastructure::Watchtower, 1), 74);
    }

    #[test]
    fn test_infrastructure_build_cost_bridge_layer_20() {
        // 100 + 5*20 = 200
        assert_eq!(infrastructure_build_cost(Infrastructure::Bridge, 20), 200);
    }

    #[test]
    fn test_infrastructure_build_cost_scales_with_depth() {
        for infra in Infrastructure::ALL {
            let shallow = infrastructure_build_cost(*infra, 1);
            let deep = infrastructure_build_cost(*infra, 20);
            assert!(deep > shallow, "{:?} cost should increase with layer", infra);
        }
    }

    // ── Build Infrastructure ─────────────────────────────────────────────────

    #[test]
    fn test_build_infrastructure_success() {
        let mut record = LayerRecord::new(1);
        record.cleared = true;
        let result = build_infrastructure(&mut record, Infrastructure::Outpost);
        assert!(result.is_ok());
        assert!(record.infrastructure.contains(&Infrastructure::Outpost));
    }

    #[test]
    fn test_build_infrastructure_fails_if_not_cleared() {
        let mut record = LayerRecord::new(1);
        // cleared = false by default
        let result = build_infrastructure(&mut record, Infrastructure::Outpost);
        assert_eq!(result, Err(InfrastructureBuildError::LayerNotCleared));
        assert!(record.infrastructure.is_empty());
    }

    #[test]
    fn test_build_infrastructure_fails_on_duplicate() {
        let mut record = LayerRecord::new(1);
        record.cleared = true;
        build_infrastructure(&mut record, Infrastructure::Outpost).unwrap();
        let result = build_infrastructure(&mut record, Infrastructure::Outpost);
        assert_eq!(result, Err(InfrastructureBuildError::AlreadyBuilt));
        assert_eq!(record.infrastructure.len(), 1);
    }

    #[test]
    fn test_build_infrastructure_watchtower_grants_familiarity() {
        let mut record = LayerRecord::new(1);
        record.cleared = true;
        record.familiarity = 30;
        build_infrastructure(&mut record, Infrastructure::Watchtower).unwrap();
        assert_eq!(record.familiarity, 55); // 30 + 25
    }

    #[test]
    fn test_build_infrastructure_watchtower_familiarity_caps_at_100() {
        let mut record = LayerRecord::new(1);
        record.cleared = true;
        record.familiarity = 90;
        build_infrastructure(&mut record, Infrastructure::Watchtower).unwrap();
        assert_eq!(record.familiarity, 100);
    }

    #[test]
    fn test_build_all_infrastructure_types_on_same_layer() {
        let mut record = LayerRecord::new(1);
        record.cleared = true;
        for &infra in Infrastructure::ALL {
            assert!(build_infrastructure(&mut record, infra).is_ok());
        }
        assert_eq!(record.infrastructure.len(), Infrastructure::ALL.len());
    }

    // ── Layer State Helpers ───────────────────────────────────────────────────

    #[test]
    fn test_mark_layer_cleared_updates_deepest_reached() {
        let mut persistent = DeepPersistent::new();
        assert_eq!(persistent.deepest_layer_reached, 0);
        mark_layer_cleared(&mut persistent, 3);
        assert_eq!(persistent.deepest_layer_reached, 3);
        assert!(persistent.layer_record(3).map(|r| r.cleared).unwrap_or(false));
    }

    #[test]
    fn test_mark_layer_cleared_does_not_decrease_deepest() {
        let mut persistent = DeepPersistent::new();
        mark_layer_cleared(&mut persistent, 5);
        mark_layer_cleared(&mut persistent, 3);
        assert_eq!(persistent.deepest_layer_reached, 5);
    }

    #[test]
    fn test_is_frontier_layer() {
        let mut persistent = DeepPersistent::new();
        // No layers cleared — frontier is 1.
        assert!(is_frontier_layer(&persistent, 1));
        assert!(!is_frontier_layer(&persistent, 2));

        // Clear layers 1 and 2.
        mark_layer_cleared(&mut persistent, 1);
        let _ = persistent.layer_record_mut(2);
        assert!(!is_frontier_layer(&persistent, 1));
    }

    #[test]
    fn test_is_safe_layer() {
        let mut persistent = DeepPersistent::new();
        assert!(!is_safe_layer(&persistent, 1));
        mark_layer_cleared(&mut persistent, 1);
        assert!(is_safe_layer(&persistent, 1));
        assert!(!is_safe_layer(&persistent, 2));
    }
}
