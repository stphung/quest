// =========================================================================
//  The Deep — narrative layer text coverage tests
// =========================================================================

use quest::deep::narratives::layer_narrative;
use quest::deep::{Infrastructure, MissionType};

// =========================================================================
//  Coverage — every layer × mission type returns non-empty text
// =========================================================================

#[test]
fn every_layer_has_all_mission_types() {
    let mission_types = [
        MissionType::SupplyRun,
        MissionType::Recon,
        MissionType::Expedition,
        MissionType::Breakthrough,
        MissionType::Construction(Infrastructure::Outpost),
        MissionType::Construction(Infrastructure::SupplyCache),
        MissionType::Construction(Infrastructure::Watchtower),
        MissionType::Construction(Infrastructure::Bridge),
    ];
    for layer in 1..=30 {
        for mt in &mission_types {
            let text = layer_narrative(layer, mt);
            assert!(
                !text.is_empty(),
                "Missing narrative for layer {} {:?}",
                layer,
                mt
            );
        }
    }
}

#[test]
fn gateway_expedition_has_narrative() {
    let text = layer_narrative(30, &MissionType::GatewayExpedition);
    assert!(!text.is_empty(), "GatewayExpedition should have narrative");
    assert!(
        text.contains("Gateway"),
        "GatewayExpedition narrative should mention the Gateway"
    );
}

// =========================================================================
//  Out-of-range layers return empty
// =========================================================================

#[test]
fn layer_zero_returns_empty() {
    assert_eq!(layer_narrative(0, &MissionType::SupplyRun), "");
}

#[test]
fn layer_above_30_returns_empty() {
    assert_eq!(layer_narrative(31, &MissionType::Recon), "");
    assert_eq!(layer_narrative(100, &MissionType::Expedition), "");
}

// =========================================================================
//  Construction variants have unique per-infrastructure narratives
// =========================================================================

#[test]
fn construction_variants_are_unique_per_layer() {
    for layer in 1..=30 {
        let outpost = layer_narrative(layer, &MissionType::Construction(Infrastructure::Outpost));
        let cache = layer_narrative(
            layer,
            &MissionType::Construction(Infrastructure::SupplyCache),
        );
        let tower = layer_narrative(
            layer,
            &MissionType::Construction(Infrastructure::Watchtower),
        );
        let bridge = layer_narrative(layer, &MissionType::Construction(Infrastructure::Bridge));

        assert_ne!(outpost, cache, "Layer {} Outpost == SupplyCache", layer);
        assert_ne!(outpost, tower, "Layer {} Outpost == Watchtower", layer);
        assert_ne!(outpost, bridge, "Layer {} Outpost == Bridge", layer);
        assert_ne!(cache, tower, "Layer {} SupplyCache == Watchtower", layer);
        assert_ne!(cache, bridge, "Layer {} SupplyCache == Bridge", layer);
        assert_ne!(tower, bridge, "Layer {} Watchtower == Bridge", layer);
    }
}

// =========================================================================
//  Uniqueness — no two (layer, type) pairs share the same text
// =========================================================================

#[test]
fn narratives_are_unique() {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mission_types = [
        MissionType::SupplyRun,
        MissionType::Recon,
        MissionType::Expedition,
        MissionType::Breakthrough,
        MissionType::Construction(Infrastructure::Outpost),
        MissionType::Construction(Infrastructure::SupplyCache),
        MissionType::Construction(Infrastructure::Watchtower),
        MissionType::Construction(Infrastructure::Bridge),
    ];
    for layer in 1..=30 {
        for mt in &mission_types {
            let text = layer_narrative(layer, mt);
            assert!(
                seen.insert(text),
                "Duplicate narrative: layer {} {:?} = {:?}",
                layer,
                mt,
                text
            );
        }
    }
    // GatewayExpedition is also unique
    let gw = layer_narrative(30, &MissionType::GatewayExpedition);
    assert!(
        seen.insert(gw),
        "GatewayExpedition narrative is a duplicate"
    );
}

// =========================================================================
//  Tier consistency — spot-check that narratives match their tier theme
// =========================================================================

#[test]
fn shallows_narratives_are_grounded() {
    // Shallows (L1-3) should describe mundane underground features
    let l1 = layer_narrative(1, &MissionType::SupplyRun);
    assert!(
        l1.contains("Ration") || l1.contains("cache") || l1.contains("door"),
        "L1 SupplyRun should describe supplies: {}",
        l1
    );
}

#[test]
fn void_narratives_are_abstract() {
    // Void (L26-30) should describe reality-breaking phenomena
    let l26 = layer_narrative(26, &MissionType::Recon);
    assert!(
        l26.contains("void") || l26.contains("knowing") || l26.contains("certainty"),
        "L26 Recon should be abstract: {}",
        l26
    );
}
