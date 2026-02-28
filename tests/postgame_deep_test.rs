use quest::deep::{DeepPersistent, DeepState};

#[test]
fn test_deep_persistent_postgame_zone_cap_defaults_to_11() {
    let deep = DeepState::new();
    assert_eq!(deep.persistent.postgame_zone_cap, 11);
}

#[test]
fn test_deep_persistent_pending_region_defaults_to_none() {
    let deep = DeepState::new();
    assert!(deep.persistent.pending_postgame_region_unlock.is_none());
}

#[test]
fn test_deep_persistent_serde_defaults() {
    // Simulate loading from old save without new fields
    let json = r#"{"discovered":false,"guild_rank":1,"guild_upgrade_cost":500,"layers":[],"deepest_layer_reached":0,"merc_id_counter":0,"mission_id_counter":0}"#;
    let persistent: DeepPersistent = serde_json::from_str(json).unwrap();
    assert_eq!(persistent.postgame_zone_cap, 11);
    assert!(persistent.pending_postgame_region_unlock.is_none());
}

#[test]
fn test_deep_persistent_serde_round_trip_with_postgame_fields() {
    let mut deep = DeepState::new();
    deep.persistent.postgame_zone_cap = 14;
    deep.persistent.pending_postgame_region_unlock = Some(quest::zones::PostgameRegion::RedFault);

    let json = serde_json::to_string(&deep.persistent).unwrap();
    let loaded: DeepPersistent = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.postgame_zone_cap, 14);
    assert_eq!(
        loaded.pending_postgame_region_unlock,
        Some(quest::zones::PostgameRegion::RedFault)
    );
}
