//! Infrastructure building for The Deep.
//!
//! Provides validation and mutation functions for building permanent
//! infrastructure on cleared layers. Infrastructure slots are limited to 2
//! per layer, and each type may only appear once per layer.

use crate::deep::types::{InfrastructureType, LayerTier, TheDeepState};

/// Maximum number of infrastructure slots per layer.
const MAX_INFRA_PER_LAYER: usize = 2;

/// Validates whether infrastructure of `infra_type` can be built on `layer_id`.
///
/// Checks (in order):
/// 1. The layer exists in account state.
/// 2. The layer has been cleared (breakthrough completed).
/// 3. The layer has fewer than 2 infrastructure slots used.
/// 4. This infrastructure type is not already built on the layer.
/// 5. The player has enough Warband Marks to cover the cost.
pub fn can_build_infrastructure(
    state: &TheDeepState,
    layer_id: u8,
    infra_type: InfrastructureType,
) -> Result<(), &'static str> {
    let layer = state
        .account
        .layers
        .iter()
        .find(|l| l.layer_number == layer_id)
        .ok_or("Layer does not exist")?;

    if !layer.cleared {
        return Err("Layer has not been cleared");
    }

    if layer.infrastructure.len() >= MAX_INFRA_PER_LAYER {
        return Err("Layer already has 2 infrastructure slots used");
    }

    if layer.infrastructure.contains(&infra_type) {
        return Err("Infrastructure type already built on this layer");
    }

    if state.run.warband_marks < infra_type.cost() {
        return Err("Insufficient Warband Marks");
    }

    Ok(())
}

/// Builds infrastructure of `infra_type` on `layer_id`, deducting the cost.
///
/// Calls [`can_build_infrastructure`] for all validation before mutating state.
pub fn build_infrastructure(
    state: &mut TheDeepState,
    layer_id: u8,
    infra_type: InfrastructureType,
) -> Result<(), &'static str> {
    can_build_infrastructure(state, layer_id, infra_type)?;

    let cost = infra_type.cost();
    state.run.warband_marks -= cost;

    let layer = state
        .account
        .layers
        .iter_mut()
        .find(|l| l.layer_number == layer_id)
        .expect("layer exists: validated above");

    layer.infrastructure.push(infra_type);
    Ok(())
}

/// Returns the list of infrastructure built on `layer_id`.
///
/// Returns an empty vec if the layer does not exist in account state.
pub fn infrastructure_on_layer(state: &TheDeepState, layer_id: u8) -> Vec<InfrastructureType> {
    state
        .account
        .layers
        .iter()
        .find(|l| l.layer_number == layer_id)
        .map(|l| l.infrastructure.clone())
        .unwrap_or_default()
}

/// Sums daily passive Warband Marks income from all Outposts across every layer.
pub fn total_outpost_daily_income(state: &TheDeepState) -> u32 {
    state
        .account
        .layers
        .iter()
        .filter(|l| l.infrastructure.contains(&InfrastructureType::Outpost))
        .map(|l| InfrastructureType::outpost_daily_income(l.layer_number))
        .sum()
}

// =========================================================================
// Private helpers
// =========================================================================

/// Returns the daily Outpost income for a layer by its tier.
///
/// Mirrors [`InfrastructureType::outpost_daily_income`] — kept here for
/// symmetry; callers should prefer the method on the type.
#[allow(dead_code)]
fn outpost_income_for_layer(layer_id: u8) -> u32 {
    InfrastructureType::outpost_daily_income(layer_id)
}

/// Maps a layer number to its [`LayerTier`] (re-exported for test convenience).
#[allow(dead_code)]
fn tier_of(layer_id: u8) -> LayerTier {
    LayerTier::from_layer(layer_id)
}
