//! Integration tests for The Deep — infrastructure building system.

use quest::deep::{
    DeepAccountState, DeepRunState, GuildRank, InfrastructureType, LayerState, TheDeepState,
    INFRA_COST_BRIDGE, INFRA_COST_OUTPOST, INFRA_COST_SUPPLY_CACHE, INFRA_COST_WATCHTOWER,
    OUTPOST_INCOME_HOLLOWS_PLUS, OUTPOST_INCOME_SHALLOWS, OUTPOST_INCOME_WARRENS,
};
use quest::deep::infrastructure::{
    build_infrastructure, can_build_infrastructure, infrastructure_on_layer,
    total_outpost_daily_income,
};

// =========================================================================
// Helpers
// =========================================================================

fn cleared_layer(layer_number: u8) -> LayerState {
    LayerState {
        layer_number,
        familiarity: 1.0,
        cleared: true,
        infrastructure: Vec::new(),
    }
}

fn uncleared_layer(layer_number: u8) -> LayerState {
    LayerState {
        layer_number,
        familiarity: 0.5,
        cleared: false,
        infrastructure: Vec::new(),
    }
}

fn state_with_marks(marks: u32) -> TheDeepState {
    TheDeepState {
        discovered: true,
        account: DeepAccountState {
            guild_rank: GuildRank::Freelancers,
            layers: Vec::new(),
            total_marks_earned: 0,
        },
        run: DeepRunState {
            warband_marks: marks,
            ..DeepRunState::new()
        },
    }
}

// =========================================================================
// can_build_infrastructure
// =========================================================================

#[test]
fn can_build_fails_when_layer_does_not_exist() {
    let state = state_with_marks(500);
    let result = can_build_infrastructure(&state, 1, InfrastructureType::Outpost);
    assert_eq!(result, Err("Layer does not exist"));
}

#[test]
fn can_build_fails_when_layer_not_cleared() {
    let mut state = state_with_marks(500);
    state.account.layers.push(uncleared_layer(1));

    let result = can_build_infrastructure(&state, 1, InfrastructureType::Outpost);
    assert_eq!(result, Err("Layer has not been cleared"));
}

#[test]
fn can_build_fails_when_two_slots_already_used() {
    let mut state = state_with_marks(500);
    let mut layer = cleared_layer(1);
    layer.infrastructure.push(InfrastructureType::Outpost);
    layer.infrastructure.push(InfrastructureType::Watchtower);
    state.account.layers.push(layer);

    let result = can_build_infrastructure(&state, 1, InfrastructureType::SupplyCache);
    assert_eq!(result, Err("Layer already has 2 infrastructure slots used"));
}

#[test]
fn can_build_fails_when_duplicate_type() {
    let mut state = state_with_marks(500);
    let mut layer = cleared_layer(1);
    layer.infrastructure.push(InfrastructureType::Outpost);
    state.account.layers.push(layer);

    let result = can_build_infrastructure(&state, 1, InfrastructureType::Outpost);
    assert_eq!(
        result,
        Err("Infrastructure type already built on this layer")
    );
}

#[test]
fn can_build_fails_when_insufficient_marks() {
    let mut state = state_with_marks(INFRA_COST_OUTPOST - 1);
    state.account.layers.push(cleared_layer(1));

    let result = can_build_infrastructure(&state, 1, InfrastructureType::Outpost);
    assert_eq!(result, Err("Insufficient Warband Marks"));
}

#[test]
fn can_build_succeeds_with_all_conditions_met() {
    let mut state = state_with_marks(500);
    state.account.layers.push(cleared_layer(1));

    let result = can_build_infrastructure(&state, 1, InfrastructureType::Outpost);
    assert!(result.is_ok());
}

#[test]
fn can_build_succeeds_for_each_infrastructure_type() {
    let types = [
        InfrastructureType::Outpost,
        InfrastructureType::SupplyCache,
        InfrastructureType::Watchtower,
        InfrastructureType::Bridge,
    ];
    let max_cost = INFRA_COST_BRIDGE;
    for infra_type in types {
        let mut state = state_with_marks(max_cost);
        state.account.layers.push(cleared_layer(1));
        assert!(
            can_build_infrastructure(&state, 1, infra_type).is_ok(),
            "Expected Ok for {:?}",
            infra_type
        );
    }
}

// =========================================================================
// build_infrastructure
// =========================================================================

#[test]
fn build_infrastructure_deducts_marks_and_adds_to_layer() {
    let mut state = state_with_marks(500);
    state.account.layers.push(cleared_layer(1));

    let before_marks = state.run.warband_marks;
    build_infrastructure(&mut state, 1, InfrastructureType::Outpost).unwrap();

    assert_eq!(
        state.run.warband_marks,
        before_marks - INFRA_COST_OUTPOST,
        "Marks should be deducted"
    );
    assert_eq!(
        state.account.layers[0].infrastructure,
        vec![InfrastructureType::Outpost],
        "Outpost should be added to layer"
    );
}

#[test]
fn build_infrastructure_can_fill_both_slots() {
    let mut state = state_with_marks(1000);
    state.account.layers.push(cleared_layer(1));

    build_infrastructure(&mut state, 1, InfrastructureType::Outpost).unwrap();
    build_infrastructure(&mut state, 1, InfrastructureType::Watchtower).unwrap();

    assert_eq!(state.account.layers[0].infrastructure.len(), 2);
}

#[test]
fn build_infrastructure_fails_when_layer_not_cleared() {
    let mut state = state_with_marks(500);
    state.account.layers.push(uncleared_layer(1));

    let result = build_infrastructure(&mut state, 1, InfrastructureType::Outpost);
    assert_eq!(result, Err("Layer has not been cleared"));
    assert_eq!(
        state.run.warband_marks, 500,
        "Marks should not change on failure"
    );
}

#[test]
fn build_infrastructure_fails_when_two_slots_already_used() {
    let mut state = state_with_marks(1000);
    let mut layer = cleared_layer(1);
    layer.infrastructure.push(InfrastructureType::Outpost);
    layer.infrastructure.push(InfrastructureType::Watchtower);
    state.account.layers.push(layer);

    let result = build_infrastructure(&mut state, 1, InfrastructureType::SupplyCache);
    assert_eq!(result, Err("Layer already has 2 infrastructure slots used"));
    assert_eq!(state.run.warband_marks, 1000);
}

#[test]
fn build_infrastructure_fails_when_duplicate_type() {
    let mut state = state_with_marks(1000);
    let mut layer = cleared_layer(1);
    layer.infrastructure.push(InfrastructureType::Outpost);
    state.account.layers.push(layer);

    let result = build_infrastructure(&mut state, 1, InfrastructureType::Outpost);
    assert_eq!(
        result,
        Err("Infrastructure type already built on this layer")
    );
}

#[test]
fn build_infrastructure_fails_when_insufficient_marks() {
    let mut state = state_with_marks(INFRA_COST_BRIDGE - 1);
    state.account.layers.push(cleared_layer(1));

    let result = build_infrastructure(&mut state, 1, InfrastructureType::Bridge);
    assert_eq!(result, Err("Insufficient Warband Marks"));
    assert_eq!(state.account.layers[0].infrastructure.len(), 0);
}

#[test]
fn build_infrastructure_deducts_correct_cost_for_each_type() {
    let cases = [
        (InfrastructureType::Outpost, INFRA_COST_OUTPOST),
        (InfrastructureType::SupplyCache, INFRA_COST_SUPPLY_CACHE),
        (InfrastructureType::Watchtower, INFRA_COST_WATCHTOWER),
        (InfrastructureType::Bridge, INFRA_COST_BRIDGE),
    ];
    for (infra_type, expected_cost) in cases {
        let initial_marks = 1000;
        let mut state = state_with_marks(initial_marks);
        state.account.layers.push(cleared_layer(1));
        build_infrastructure(&mut state, 1, infra_type).unwrap();
        assert_eq!(
            state.run.warband_marks,
            initial_marks - expected_cost,
            "Wrong cost deducted for {:?}",
            infra_type
        );
    }
}

// =========================================================================
// infrastructure_on_layer
// =========================================================================

#[test]
fn infrastructure_on_layer_returns_correct_list() {
    let mut state = state_with_marks(1000);
    let mut layer = cleared_layer(2);
    layer.infrastructure.push(InfrastructureType::Watchtower);
    layer.infrastructure.push(InfrastructureType::SupplyCache);
    state.account.layers.push(layer);

    let infra = infrastructure_on_layer(&state, 2);
    assert_eq!(infra.len(), 2);
    assert!(infra.contains(&InfrastructureType::Watchtower));
    assert!(infra.contains(&InfrastructureType::SupplyCache));
}

#[test]
fn infrastructure_on_layer_returns_empty_for_missing_layer() {
    let state = state_with_marks(0);
    let infra = infrastructure_on_layer(&state, 99);
    assert!(infra.is_empty());
}

#[test]
fn infrastructure_on_layer_returns_empty_for_layer_with_no_infra() {
    let mut state = state_with_marks(0);
    state.account.layers.push(cleared_layer(3));

    let infra = infrastructure_on_layer(&state, 3);
    assert!(infra.is_empty());
}

// =========================================================================
// total_outpost_daily_income
// =========================================================================

#[test]
fn total_outpost_daily_income_is_zero_with_no_outposts() {
    let mut state = state_with_marks(0);
    // Watchtower only — no Outpost
    let mut layer = cleared_layer(1);
    layer.infrastructure.push(InfrastructureType::Watchtower);
    state.account.layers.push(layer);

    assert_eq!(total_outpost_daily_income(&state), 0);
}

#[test]
fn total_outpost_daily_income_counts_shallows_layer() {
    let mut state = state_with_marks(0);
    let mut layer = cleared_layer(1); // Shallows tier (layer 1-3)
    layer.infrastructure.push(InfrastructureType::Outpost);
    state.account.layers.push(layer);

    assert_eq!(
        total_outpost_daily_income(&state),
        OUTPOST_INCOME_SHALLOWS
    );
}

#[test]
fn total_outpost_daily_income_counts_warrens_layer() {
    let mut state = state_with_marks(0);
    let mut layer = cleared_layer(5); // Warrens tier (layer 4-7)
    layer.infrastructure.push(InfrastructureType::Outpost);
    state.account.layers.push(layer);

    assert_eq!(
        total_outpost_daily_income(&state),
        OUTPOST_INCOME_WARRENS
    );
}

#[test]
fn total_outpost_daily_income_counts_hollows_layer() {
    let mut state = state_with_marks(0);
    let mut layer = cleared_layer(9); // Hollows tier (layer 8-12)
    layer.infrastructure.push(InfrastructureType::Outpost);
    state.account.layers.push(layer);

    assert_eq!(
        total_outpost_daily_income(&state),
        OUTPOST_INCOME_HOLLOWS_PLUS
    );
}

#[test]
fn total_outpost_daily_income_sums_across_multiple_layers() {
    let mut state = state_with_marks(0);

    // Layer 1 (Shallows) — Outpost
    let mut l1 = cleared_layer(1);
    l1.infrastructure.push(InfrastructureType::Outpost);
    state.account.layers.push(l1);

    // Layer 5 (Warrens) — Outpost + Watchtower
    let mut l5 = cleared_layer(5);
    l5.infrastructure.push(InfrastructureType::Outpost);
    l5.infrastructure.push(InfrastructureType::Watchtower);
    state.account.layers.push(l5);

    // Layer 10 (Hollows) — SupplyCache only (no Outpost)
    let mut l10 = cleared_layer(10);
    l10.infrastructure.push(InfrastructureType::SupplyCache);
    state.account.layers.push(l10);

    let expected = OUTPOST_INCOME_SHALLOWS + OUTPOST_INCOME_WARRENS;
    assert_eq!(total_outpost_daily_income(&state), expected);
}

#[test]
fn total_outpost_daily_income_with_no_layers() {
    let state = state_with_marks(0);
    assert_eq!(total_outpost_daily_income(&state), 0);
}
