//! Integration tests for The Deep module — Phase 1 Foundation Types.

use quest::deep::{
    GuildRank, InfrastructureType, LayerState, LayerTier, MercArchetype, MercStatus, Mercenary,
    MissionType, TheDeepState,
};

// =========================================================================
// TheDeepState::new() defaults
// =========================================================================

#[test]
fn test_the_deep_state_new_defaults() {
    let state = TheDeepState::new();
    assert!(!state.discovered);
    assert_eq!(state.guild_rank, GuildRank::Freelancers);
    assert_eq!(state.warband_marks, 0);
    assert!(state.mercenaries.is_empty());
    assert!(state.active_missions.is_empty());
    assert!(state.completed_missions.is_empty());
    assert!(state.layers.is_empty());
    assert!(state.recruitment_pool.is_empty());
    assert!(state.recruitment_refresh_date.is_none());
    assert_eq!(state.total_marks_earned, 0);
}

// =========================================================================
// GuildRank methods
// =========================================================================

#[test]
fn test_guild_rank_roster_cap() {
    assert_eq!(GuildRank::Freelancers.roster_cap(), 5);
    assert_eq!(GuildRank::Sellswords.roster_cap(), 7);
    assert_eq!(GuildRank::Company.roster_cap(), 9);
    assert_eq!(GuildRank::Battalion.roster_cap(), 12);
    assert_eq!(GuildRank::Legion.roster_cap(), 15);
}

#[test]
fn test_guild_rank_mission_slots() {
    assert_eq!(GuildRank::Freelancers.mission_slots(), 1);
    assert_eq!(GuildRank::Sellswords.mission_slots(), 1);
    assert_eq!(GuildRank::Company.mission_slots(), 2);
    assert_eq!(GuildRank::Battalion.mission_slots(), 3);
    assert_eq!(GuildRank::Legion.mission_slots(), 4);
}

#[test]
fn test_guild_rank_upgrade_cost() {
    // Freelancers needs marks to upgrade to Sellswords
    assert!(GuildRank::Freelancers.upgrade_cost().is_some());
    assert!(GuildRank::Sellswords.upgrade_cost().is_some());
    assert!(GuildRank::Company.upgrade_cost().is_some());
    assert!(GuildRank::Battalion.upgrade_cost().is_some());
    // Legion is max rank — no upgrade cost
    assert!(GuildRank::Legion.upgrade_cost().is_none());
}

#[test]
fn test_guild_rank_required_layer() {
    // Freelancers has no layer requirement (starting rank)
    assert!(GuildRank::Freelancers.required_layer().is_none());
    // Others require breakthrough on specific layers
    assert_eq!(GuildRank::Sellswords.required_layer(), Some(3));
    assert_eq!(GuildRank::Company.required_layer(), Some(7));
    assert_eq!(GuildRank::Battalion.required_layer(), Some(13));
    assert_eq!(GuildRank::Legion.required_layer(), Some(19));
}

#[test]
fn test_guild_rank_next() {
    assert_eq!(GuildRank::Freelancers.next(), Some(GuildRank::Sellswords));
    assert_eq!(GuildRank::Sellswords.next(), Some(GuildRank::Company));
    assert_eq!(GuildRank::Company.next(), Some(GuildRank::Battalion));
    assert_eq!(GuildRank::Battalion.next(), Some(GuildRank::Legion));
    assert_eq!(GuildRank::Legion.next(), None);
}

#[test]
fn test_guild_rank_default_is_freelancers() {
    let rank = GuildRank::default();
    assert_eq!(rank, GuildRank::Freelancers);
}

// =========================================================================
// InfrastructureType::cost()
// =========================================================================

#[test]
fn test_infrastructure_cost_values() {
    // All infrastructure types should have a positive cost
    assert!(InfrastructureType::Outpost.cost() > 0);
    assert!(InfrastructureType::SupplyCache.cost() > 0);
    assert!(InfrastructureType::Watchtower.cost() > 0);
    assert!(InfrastructureType::Bridge.cost() > 0);
}

#[test]
fn test_infrastructure_description_not_empty() {
    assert!(!InfrastructureType::Outpost.description().is_empty());
    assert!(!InfrastructureType::SupplyCache.description().is_empty());
    assert!(!InfrastructureType::Watchtower.description().is_empty());
    assert!(!InfrastructureType::Bridge.description().is_empty());
}

// =========================================================================
// LayerTier::from_layer() mapping
// =========================================================================

#[test]
fn test_layer_tier_from_layer_shallows() {
    assert_eq!(LayerTier::from_layer(1), LayerTier::Shallows);
    assert_eq!(LayerTier::from_layer(2), LayerTier::Shallows);
    assert_eq!(LayerTier::from_layer(3), LayerTier::Shallows);
}

#[test]
fn test_layer_tier_from_layer_warrens() {
    assert_eq!(LayerTier::from_layer(4), LayerTier::Warrens);
    assert_eq!(LayerTier::from_layer(5), LayerTier::Warrens);
    assert_eq!(LayerTier::from_layer(6), LayerTier::Warrens);
    assert_eq!(LayerTier::from_layer(7), LayerTier::Warrens);
}

#[test]
fn test_layer_tier_from_layer_hollows() {
    assert_eq!(LayerTier::from_layer(8), LayerTier::Hollows);
    assert_eq!(LayerTier::from_layer(9), LayerTier::Hollows);
    assert_eq!(LayerTier::from_layer(12), LayerTier::Hollows);
}

#[test]
fn test_layer_tier_from_layer_sunken_reach() {
    assert_eq!(LayerTier::from_layer(13), LayerTier::SunkenReach);
    assert_eq!(LayerTier::from_layer(15), LayerTier::SunkenReach);
    assert_eq!(LayerTier::from_layer(18), LayerTier::SunkenReach);
}

#[test]
fn test_layer_tier_from_layer_abyss() {
    assert_eq!(LayerTier::from_layer(19), LayerTier::Abyss);
    assert_eq!(LayerTier::from_layer(22), LayerTier::Abyss);
    assert_eq!(LayerTier::from_layer(25), LayerTier::Abyss);
}

#[test]
fn test_layer_tier_from_layer_void() {
    assert_eq!(LayerTier::from_layer(26), LayerTier::Void);
    assert_eq!(LayerTier::from_layer(50), LayerTier::Void);
    assert_eq!(LayerTier::from_layer(255), LayerTier::Void);
}

// =========================================================================
// Serialization / deserialization roundtrip
// =========================================================================

#[test]
fn test_the_deep_state_serde_roundtrip_default() {
    let state = TheDeepState::new();
    let json = serde_json::to_string(&state).expect("serialization failed");
    let loaded: TheDeepState = serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(loaded.discovered, state.discovered);
    assert_eq!(loaded.guild_rank, state.guild_rank);
    assert_eq!(loaded.warband_marks, state.warband_marks);
    assert_eq!(loaded.total_marks_earned, state.total_marks_earned);
}

#[test]
fn test_the_deep_state_serde_roundtrip_with_data() {
    let mut state = TheDeepState::new();
    state.discovered = true;
    state.warband_marks = 500;
    state.total_marks_earned = 1000;
    state.guild_rank = GuildRank::Company;
    state.recruitment_refresh_date = Some("2026-02-22".to_string());

    let merc = Mercenary {
        id: 1,
        name: "Aldric the Bold".to_string(),
        archetype: MercArchetype::Vanguard,
        level: 3,
        power: 75,
        resilience: 60,
        status: MercStatus::Ready,
        missions_completed: 5,
    };
    state.mercenaries.push(merc);

    let layer = LayerState {
        layer_number: 1,
        familiarity: 0.5,
        cleared: true,
        infrastructure: vec![InfrastructureType::Outpost],
    };
    state.layers.push(layer);

    let json = serde_json::to_string(&state).expect("serialization failed");
    let loaded: TheDeepState = serde_json::from_str(&json).expect("deserialization failed");

    assert!(loaded.discovered);
    assert_eq!(loaded.warband_marks, 500);
    assert_eq!(loaded.total_marks_earned, 1000);
    assert_eq!(loaded.guild_rank, GuildRank::Company);
    assert_eq!(
        loaded.recruitment_refresh_date,
        Some("2026-02-22".to_string())
    );
    assert_eq!(loaded.mercenaries.len(), 1);
    assert_eq!(loaded.mercenaries[0].name, "Aldric the Bold");
    assert_eq!(loaded.mercenaries[0].archetype, MercArchetype::Vanguard);
    assert_eq!(loaded.mercenaries[0].level, 3);
    assert_eq!(loaded.layers.len(), 1);
    assert_eq!(loaded.layers[0].layer_number, 1);
    assert!(loaded.layers[0].cleared);
}

// =========================================================================
// MercArchetype Display
// =========================================================================

#[test]
fn test_merc_archetype_display() {
    assert_eq!(format!("{}", MercArchetype::Vanguard), "Vanguard");
    assert_eq!(format!("{}", MercArchetype::Scout), "Scout");
    assert_eq!(format!("{}", MercArchetype::Medic), "Medic");
    assert_eq!(format!("{}", MercArchetype::Saboteur), "Saboteur");
    assert_eq!(format!("{}", MercArchetype::Arcanist), "Arcanist");
}

// =========================================================================
// LayerState infrastructure max 2
// =========================================================================

#[test]
fn test_layer_state_infrastructure_can_hold_max_two() {
    let layer = LayerState {
        layer_number: 5,
        familiarity: 0.8,
        cleared: false,
        infrastructure: vec![InfrastructureType::Outpost, InfrastructureType::Watchtower],
    };
    assert_eq!(layer.infrastructure.len(), 2);
}

#[test]
fn test_layer_state_infrastructure_empty_by_default_construction() {
    let layer = LayerState {
        layer_number: 1,
        familiarity: 0.0,
        cleared: false,
        infrastructure: vec![],
    };
    assert!(layer.infrastructure.is_empty());
}

// =========================================================================
// MissionType variants exist and are serializable
// =========================================================================

#[test]
fn test_mission_type_serde_roundtrip() {
    let types = vec![
        MissionType::SupplyRun,
        MissionType::Recon,
        MissionType::Expedition,
        MissionType::Breakthrough,
        MissionType::Construction,
    ];
    for mission_type in types {
        let json = serde_json::to_string(&mission_type).expect("serialize failed");
        let loaded: MissionType = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(loaded, mission_type);
    }
}

// =========================================================================
// LayerTier Display
// =========================================================================

#[test]
fn test_layer_tier_display() {
    assert_eq!(format!("{}", LayerTier::Shallows), "The Shallows");
    assert_eq!(format!("{}", LayerTier::Warrens), "The Warrens");
    assert_eq!(format!("{}", LayerTier::Hollows), "The Hollows");
    assert_eq!(format!("{}", LayerTier::SunkenReach), "The Sunken Reach");
    assert_eq!(format!("{}", LayerTier::Abyss), "The Abyss");
    assert_eq!(format!("{}", LayerTier::Void), "The Void");
}

// =========================================================================
// MercStatus variants serializable
// =========================================================================

#[test]
fn test_merc_status_serde_roundtrip() {
    let statuses = vec![
        MercStatus::Ready,
        MercStatus::OnMission,
        MercStatus::Injured,
        MercStatus::Lost,
    ];
    for status in statuses {
        let json = serde_json::to_string(&status).expect("serialize failed");
        let loaded: MercStatus = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(loaded, status);
    }
}
