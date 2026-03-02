#![allow(dead_code)] // Functions wired into the game loop incrementally
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

use crate::deep::types::{DeepPersistent, Infrastructure, LayerRecord, LayerTier, MissionType};

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
    /// 75–100%: -30% duration, excellent auto-resolve, +25% Mark yield.
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

    /// Mission duration reduction factor (1.0 = no reduction, 0.55 = -45%).
    pub fn duration_factor(self) -> f64 {
        match self {
            FamiliarityLevel::Unknown => 1.0,
            FamiliarityLevel::Mapped => 0.85,
            FamiliarityLevel::Familiar => 0.70,
            FamiliarityLevel::Mastered => 0.55,
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

    /// Bonus Mark yield multiplier (1.0 = no bonus, 1.25 = +25%).
    pub fn mark_yield_multiplier(self) -> f64 {
        match self {
            FamiliarityLevel::Familiar => 1.10,
            FamiliarityLevel::Mastered => 1.25,
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
        MissionType::SupplyRun => 2,
        MissionType::Recon => 5,
        MissionType::Expedition => 15,
        MissionType::Breakthrough | MissionType::GatewayExpedition => 15,
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
    const VOID_BREAK: u32 = 60;
    const VOID_EXP: u32 = 45;
    const VOID_REC: u32 = 35;
    const VOID_SUP: u32 = 25;

    if layer >= 26 {
        let extra = layer - 25;
        return LayerPowerThresholds {
            breakthrough: 700 + VOID_BREAK * extra,
            expedition: 525 + VOID_EXP * extra,
            recon: 395 + VOID_REC * extra,
            supply_run: 265 + VOID_SUP * extra,
        };
    }

    // Lookup table for layers 1–25 from the balance design document.
    let (breakthrough, expedition, recon, supply_run) = match layer {
        1 => (25, 20, 15, 10),
        2 => (40, 30, 20, 15),
        3 => (55, 40, 30, 20),
        4 => (75, 55, 40, 25),
        5 => (95, 70, 50, 30),
        6 => (115, 85, 60, 40),
        7 => (130, 100, 75, 50),
        8 => (155, 115, 85, 55),
        9 => (180, 135, 100, 65),
        10 => (205, 155, 115, 75),
        11 => (230, 175, 130, 85),
        12 => (260, 195, 145, 95),
        13 => (295, 220, 165, 110),
        14 => (330, 250, 185, 125),
        15 => (370, 280, 210, 140),
        16 => (410, 310, 230, 155),
        17 => (450, 340, 255, 170),
        18 => (495, 370, 280, 185),
        19 => (410, 310, 230, 155),
        20 => (445, 335, 250, 165),
        21 => (480, 360, 270, 180),
        22 => (520, 390, 290, 195),
        23 => (565, 425, 320, 210),
        24 => (625, 470, 350, 235),
        25 => (700, 525, 395, 265),
        _ => unreachable!(), // layers >= 26 handled above
    };

    LayerPowerThresholds {
        breakthrough,
        expedition,
        recon,
        supply_run,
    }
}

/// Return the power threshold relevant for a given mission type on a layer.
pub fn mission_power_threshold(layer: u32, mission_type: MissionType) -> u32 {
    let t = layer_power_thresholds(layer);
    match mission_type {
        MissionType::Breakthrough | MissionType::GatewayExpedition => t.breakthrough,
        MissionType::Expedition => t.expedition,
        MissionType::Recon => t.recon,
        MissionType::SupplyRun | MissionType::Construction(_) => t.supply_run,
    }
}

// ── Mission Durations ─────────────────────────────────────────────────────────

/// Mission duration in seconds, keyed on layer tier and mission type.
///
/// This is the final duration — no modifiers, no minimums, no base/effective split.
pub fn mission_duration_secs(tier: LayerTier, mission_type: MissionType) -> u64 {
    match (tier, mission_type) {
        // Gateway Expedition — fixed 48h regardless of tier
        (_, MissionType::GatewayExpedition) => 172_800,
        // Shallows (Recon 1h base: Construction 2h, Expedition 3h, Breakthrough 4h)
        (LayerTier::Shallows, MissionType::SupplyRun) => 3_600,
        (LayerTier::Shallows, MissionType::Recon) => 3_600,
        (LayerTier::Shallows, MissionType::Construction(_)) => 7_200,
        (LayerTier::Shallows, MissionType::Expedition) => 10_800,
        (LayerTier::Shallows, MissionType::Breakthrough) => 14_400,
        // Warrens (Recon 3h base)
        (LayerTier::Warrens, MissionType::SupplyRun) => 7_200,
        (LayerTier::Warrens, MissionType::Recon) => 10_800,
        (LayerTier::Warrens, MissionType::Construction(_)) => 21_600,
        (LayerTier::Warrens, MissionType::Expedition) => 32_400,
        (LayerTier::Warrens, MissionType::Breakthrough) => 43_200,
        // Hollows (Recon 5h base)
        (LayerTier::Hollows, MissionType::SupplyRun) => 10_800,
        (LayerTier::Hollows, MissionType::Recon) => 18_000,
        (LayerTier::Hollows, MissionType::Construction(_)) => 36_000,
        (LayerTier::Hollows, MissionType::Expedition) => 54_000,
        (LayerTier::Hollows, MissionType::Breakthrough) => 72_000,
        // Sunken Reach (Recon 6h base)
        (LayerTier::SunkenReach, MissionType::SupplyRun) => 14_400,
        (LayerTier::SunkenReach, MissionType::Recon) => 21_600,
        (LayerTier::SunkenReach, MissionType::Construction(_)) => 43_200,
        (LayerTier::SunkenReach, MissionType::Expedition) => 64_800,
        (LayerTier::SunkenReach, MissionType::Breakthrough) => 86_400,
        // Abyss (Recon 8h base)
        (LayerTier::Abyss, MissionType::SupplyRun) => 18_000,
        (LayerTier::Abyss, MissionType::Recon) => 28_800,
        (LayerTier::Abyss, MissionType::Construction(_)) => 57_600,
        (LayerTier::Abyss, MissionType::Expedition) => 86_400,
        (LayerTier::Abyss, MissionType::Breakthrough) => 115_200,
        // Void (Recon 10h base)
        (LayerTier::Void, MissionType::SupplyRun) => 21_600,
        (LayerTier::Void, MissionType::Recon) => 36_000,
        (LayerTier::Void, MissionType::Construction(_)) => 72_000,
        (LayerTier::Void, MissionType::Expedition) => 108_000,
        (LayerTier::Void, MissionType::Breakthrough) => 144_000,
    }
}

/// Returns the auto-resolve success bonus from a Watchtower (+5%).
pub fn watchtower_auto_resolve_bonus(has_watchtower: bool) -> f64 {
    if has_watchtower {
        0.05
    } else {
        0.0
    }
}

// ── Infrastructure ────────────────────────────────────────────────────────────

/// Mark cost to build a specific infrastructure type on a given 1-based layer.
///
/// Costs scale with layer depth. See balance design §6.
pub fn infrastructure_build_cost(infra: Infrastructure, layer: u32) -> u32 {
    let layer = layer.max(1);
    match infra {
        Infrastructure::Outpost => 85 + 6 * layer,
        Infrastructure::SupplyCache => 110 + 7 * layer,
        Infrastructure::Watchtower => 100 + 6 * layer,
        Infrastructure::Bridge => 140 + 7 * layer,
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

    // Watchtower grants an immediate +40 familiarity bonus.
    if infra == Infrastructure::Watchtower {
        record.familiarity = record.familiarity.saturating_add(40).min(100);
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
    // Abyss entry bonus: clearing L18 grants Mapped familiarity on L19
    if layer == 18 {
        let l19 = persistent.layer_record_mut(19);
        l19.familiarity = l19.familiarity.max(25);
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
        assert_eq!(
            FamiliarityLevel::from_familiarity(0),
            FamiliarityLevel::Unknown
        );
        assert_eq!(
            FamiliarityLevel::from_familiarity(24),
            FamiliarityLevel::Unknown
        );
        assert_eq!(
            FamiliarityLevel::from_familiarity(25),
            FamiliarityLevel::Mapped
        );
        assert_eq!(
            FamiliarityLevel::from_familiarity(49),
            FamiliarityLevel::Mapped
        );
        assert_eq!(
            FamiliarityLevel::from_familiarity(50),
            FamiliarityLevel::Familiar
        );
        assert_eq!(
            FamiliarityLevel::from_familiarity(74),
            FamiliarityLevel::Familiar
        );
        assert_eq!(
            FamiliarityLevel::from_familiarity(75),
            FamiliarityLevel::Mastered
        );
        assert_eq!(
            FamiliarityLevel::from_familiarity(100),
            FamiliarityLevel::Mastered
        );
    }

    #[test]
    fn test_familiarity_level_duration_factors() {
        assert_eq!(FamiliarityLevel::Unknown.duration_factor(), 1.0);
        assert_eq!(FamiliarityLevel::Mapped.duration_factor(), 0.85);
        assert_eq!(FamiliarityLevel::Familiar.duration_factor(), 0.70);
        assert_eq!(FamiliarityLevel::Mastered.duration_factor(), 0.55);
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
        assert_eq!(FamiliarityLevel::Familiar.mark_yield_multiplier(), 1.10);
        assert_eq!(FamiliarityLevel::Mastered.mark_yield_multiplier(), 1.25);
    }

    // ── Familiarity Gain ─────────────────────────────────────────────────────

    #[test]
    fn test_familiarity_gain_values() {
        assert_eq!(familiarity_gain(MissionType::SupplyRun), 2);
        assert_eq!(familiarity_gain(MissionType::Recon), 5);
        assert_eq!(familiarity_gain(MissionType::Expedition), 15);
        assert_eq!(familiarity_gain(MissionType::Breakthrough), 15);
        assert_eq!(
            familiarity_gain(MissionType::Construction(Infrastructure::Outpost)),
            5
        );
    }

    #[test]
    fn test_apply_familiarity_gain_caps_at_100() {
        let mut record = LayerRecord::new(1);
        record.familiarity = 95;
        apply_familiarity_gain(&mut record, MissionType::Expedition); // +15 would overflow
        assert_eq!(record.familiarity, 100);
    }

    #[test]
    fn test_apply_familiarity_gain_accumulates() {
        let mut record = LayerRecord::new(1);
        apply_familiarity_gain(&mut record, MissionType::SupplyRun); // +2
        apply_familiarity_gain(&mut record, MissionType::Recon); // +5
        assert_eq!(record.familiarity, 7);
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
        assert_eq!(t.breakthrough, 700);
        assert_eq!(t.expedition, 525);
        assert_eq!(t.recon, 395);
        assert_eq!(t.supply_run, 265);
    }

    #[test]
    fn test_power_thresholds_void_layer_26() {
        let t = layer_power_thresholds(26);
        // Void: 700 + 60*(26-25) = 760, etc.
        assert_eq!(t.breakthrough, 760);
        assert_eq!(t.expedition, 570);
        assert_eq!(t.recon, 430);
        assert_eq!(t.supply_run, 290);
    }

    #[test]
    fn test_power_thresholds_increase_within_tier() {
        // Thresholds increase monotonically within each layer tier.
        // The Abyss tier (L19-25) was intentionally softened, so L19 may be
        // lower than L18 (Sunken Reach), but each tier is internally monotonic.
        let tier_ranges: &[std::ops::RangeInclusive<u32>] =
            &[1..=3, 4..=7, 8..=12, 13..=18, 19..=25];
        for range in tier_ranges {
            let layers: Vec<u32> = range.clone().collect();
            for window in layers.windows(2) {
                let a = layer_power_thresholds(window[0]);
                let b = layer_power_thresholds(window[1]);
                assert!(
                    b.breakthrough >= a.breakthrough,
                    "Breakthrough threshold should increase within tier: layer {} -> {}",
                    window[0],
                    window[1]
                );
            }
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

    // ── Mission Durations ─────────────────────────────────────────────────────

    #[test]
    fn test_shallows_durations() {
        assert_eq!(
            mission_duration_secs(LayerTier::Shallows, MissionType::SupplyRun),
            3600
        );
        assert_eq!(
            mission_duration_secs(LayerTier::Shallows, MissionType::Recon),
            3600
        );
        assert_eq!(
            mission_duration_secs(LayerTier::Shallows, MissionType::Expedition),
            10_800
        );
        assert_eq!(
            mission_duration_secs(
                LayerTier::Shallows,
                MissionType::Construction(Infrastructure::Outpost)
            ),
            7200
        );
        assert_eq!(
            mission_duration_secs(LayerTier::Shallows, MissionType::Breakthrough),
            14_400
        );
    }

    #[test]
    fn test_void_durations() {
        assert_eq!(
            mission_duration_secs(LayerTier::Void, MissionType::SupplyRun),
            21_600
        );
        assert_eq!(
            mission_duration_secs(LayerTier::Void, MissionType::Breakthrough),
            144_000
        );
    }

    #[test]
    fn test_gateway_duration() {
        assert_eq!(
            mission_duration_secs(LayerTier::Void, MissionType::GatewayExpedition),
            172_800
        );
        // Gateway is the same regardless of tier
        assert_eq!(
            mission_duration_secs(LayerTier::Shallows, MissionType::GatewayExpedition),
            172_800
        );
    }

    #[test]
    fn test_durations_increase_with_tier() {
        let tiers = [
            LayerTier::Shallows,
            LayerTier::Warrens,
            LayerTier::Hollows,
            LayerTier::SunkenReach,
            LayerTier::Abyss,
            LayerTier::Void,
        ];
        for mission in [
            MissionType::SupplyRun,
            MissionType::Recon,
            MissionType::Expedition,
            MissionType::Breakthrough,
        ] {
            let mut prev = 0u64;
            for &tier in &tiers {
                let d = mission_duration_secs(tier, mission);
                assert!(
                    d >= prev,
                    "{:?} {:?} duration should not decrease: {} < {}",
                    tier,
                    mission,
                    d,
                    prev,
                );
                prev = d;
            }
        }
    }

    // ── Infrastructure Build Cost ─────────────────────────────────────────────

    #[test]
    fn test_infrastructure_build_cost_outpost_layer_1() {
        // 85 + 6*1 = 91
        assert_eq!(infrastructure_build_cost(Infrastructure::Outpost, 1), 91);
    }

    #[test]
    fn test_infrastructure_build_cost_supply_cache_layer_10() {
        // 110 + 7*10 = 180
        assert_eq!(
            infrastructure_build_cost(Infrastructure::SupplyCache, 10),
            180
        );
    }

    #[test]
    fn test_infrastructure_build_cost_watchtower_layer_1() {
        // 100 + 6*1 = 106
        assert_eq!(
            infrastructure_build_cost(Infrastructure::Watchtower, 1),
            106
        );
    }

    #[test]
    fn test_infrastructure_build_cost_bridge_layer_20() {
        // 140 + 7*20 = 280
        assert_eq!(infrastructure_build_cost(Infrastructure::Bridge, 20), 280);
    }

    #[test]
    fn test_infrastructure_build_cost_scales_with_depth() {
        for infra in Infrastructure::ALL {
            let shallow = infrastructure_build_cost(*infra, 1);
            let deep = infrastructure_build_cost(*infra, 20);
            assert!(
                deep > shallow,
                "{:?} cost should increase with layer",
                infra
            );
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
        assert_eq!(record.familiarity, 70); // 30 + 40
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
        assert!(persistent
            .layer_record(3)
            .map(|r| r.cleared)
            .unwrap_or(false));
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

    #[test]
    fn test_clearing_layer_18_grants_l19_familiarity_bonus() {
        let mut persistent = DeepPersistent::new();
        // Clear layers 1 through 18
        for l in 1..=18 {
            mark_layer_cleared(&mut persistent, l);
        }
        let l19 = persistent.layer_record(19).unwrap();
        assert!(
            l19.familiarity >= 25,
            "L19 should have at least Mapped familiarity after L18 breakthrough"
        );
    }

    #[test]
    fn test_clearing_layer_18_does_not_reduce_existing_l19_familiarity() {
        let mut persistent = DeepPersistent::new();
        // Give L19 high familiarity first
        persistent.layer_record_mut(19).familiarity = 80;
        mark_layer_cleared(&mut persistent, 18);
        assert_eq!(persistent.layer_record(19).unwrap().familiarity, 80);
    }
}
