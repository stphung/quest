//! Layer system for The Deep.
//!
//! Provides tier classification, difficulty scaling, familiarity tracking,
//! and mission type availability based on a layer's familiarity level.

use crate::deep::types::{LayerState, LayerTier, MissionType};

/// Returns the [`LayerTier`] for a given layer number (1-based).
///
/// Tier boundaries:
/// - 1-3:   Shallows
/// - 4-7:   Warrens
/// - 8-12:  Hollows
/// - 13-18: SunkenReach
/// - 19-25: Abyss
/// - 26+:   Void
pub fn layer_tier(layer_id: u8) -> LayerTier {
    LayerTier::from_layer(layer_id)
}

/// Returns the base difficulty rating for a given layer.
///
/// Formula: `layer_id * 10 + tier_bonus`
///
/// Tier bonuses:
/// - Shallows:    0
/// - Warrens:    20
/// - Hollows:    50
/// - SunkenReach: 100
/// - Abyss:      200
/// - Void:       400
pub fn layer_difficulty(layer_id: u8) -> u32 {
    let tier_bonus = match layer_tier(layer_id) {
        LayerTier::Shallows => 0u32,
        LayerTier::Warrens => 20,
        LayerTier::Hollows => 50,
        LayerTier::SunkenReach => 100,
        LayerTier::Abyss => 200,
        LayerTier::Void => 400,
    };
    (layer_id as u32) * 10 + tier_bonus
}

/// Increases familiarity on a layer, clamping to the range `0.0..=1.0`.
pub fn increase_familiarity(layer: &mut LayerState, amount: f32) {
    layer.familiarity = (layer.familiarity + amount).clamp(0.0, 1.0);
}

/// Returns the set of mission types available on a layer given its current familiarity.
///
/// Availability thresholds:
/// - 0.00–0.25: [`MissionType::SupplyRun`], [`MissionType::Recon`]
/// - 0.25–0.50: + [`MissionType::Expedition`]
/// - 0.50–0.75: + [`MissionType::Construction`] (if layer is cleared)
/// - 0.75–1.00: + [`MissionType::Breakthrough`]
///
/// The `cleared` flag is required for the Construction threshold check at 0.50+.
pub fn available_mission_types(familiarity: f32, cleared: bool) -> Vec<MissionType> {
    let mut types = vec![MissionType::SupplyRun, MissionType::Recon];

    if familiarity >= 0.25 {
        types.push(MissionType::Expedition);
    }
    if familiarity >= 0.50 && cleared {
        types.push(MissionType::Construction);
    }
    if familiarity >= 0.75 {
        types.push(MissionType::Breakthrough);
    }

    types
}
