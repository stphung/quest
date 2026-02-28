use quest::deep::{DeepPersistent, DeepState};
use quest::zones::PostgameRegion;

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

// ── Task 12: Deep Breakthrough Triggers Postgame Region Unlock ──────────

#[test]
fn test_layer_3_breakthrough_sets_cap_to_14() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.deepest_layer_reached = 3;

    // Simulate checking if a breakthrough should unlock a region
    if let Some(region) = PostgameRegion::from_layer(3) {
        let new_cap = region.end_zone_id();
        if new_cap > deep.persistent.postgame_zone_cap {
            deep.persistent.postgame_zone_cap = new_cap;
            deep.persistent.pending_postgame_region_unlock = Some(region);
        }
    }

    assert_eq!(deep.persistent.postgame_zone_cap, 14);
    assert_eq!(
        deep.persistent.pending_postgame_region_unlock,
        Some(PostgameRegion::RedFault)
    );
}

#[test]
fn test_layer_7_breakthrough_sets_cap_to_17() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.postgame_zone_cap = 14; // Already unlocked Red Fault

    if let Some(region) = PostgameRegion::from_layer(7) {
        let new_cap = region.end_zone_id();
        if new_cap > deep.persistent.postgame_zone_cap {
            deep.persistent.postgame_zone_cap = new_cap;
            deep.persistent.pending_postgame_region_unlock = Some(region);
        }
    }

    assert_eq!(deep.persistent.postgame_zone_cap, 17);
    assert_eq!(
        deep.persistent.pending_postgame_region_unlock,
        Some(PostgameRegion::MirrorScar)
    );
}

#[test]
fn test_layer_13_breakthrough_sets_cap_to_20() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.postgame_zone_cap = 17;

    if let Some(region) = PostgameRegion::from_layer(13) {
        let new_cap = region.end_zone_id();
        if new_cap > deep.persistent.postgame_zone_cap {
            deep.persistent.postgame_zone_cap = new_cap;
            deep.persistent.pending_postgame_region_unlock = Some(region);
        }
    }

    assert_eq!(deep.persistent.postgame_zone_cap, 20);
    assert_eq!(
        deep.persistent.pending_postgame_region_unlock,
        Some(PostgameRegion::BlackMouth)
    );
}

#[test]
fn test_repeated_breakthrough_does_not_downgrade_cap() {
    let mut deep = DeepState::new();
    deep.persistent.postgame_zone_cap = 17;
    deep.persistent.pending_postgame_region_unlock = None;

    // Layer 3 again shouldn't downgrade from 17 to 14
    if let Some(region) = PostgameRegion::from_layer(3) {
        let new_cap = region.end_zone_id();
        if new_cap > deep.persistent.postgame_zone_cap {
            deep.persistent.postgame_zone_cap = new_cap;
            deep.persistent.pending_postgame_region_unlock = Some(region);
        }
    }

    assert_eq!(deep.persistent.postgame_zone_cap, 17); // unchanged
    assert!(deep.persistent.pending_postgame_region_unlock.is_none()); // not set
}

#[test]
fn test_non_unlock_layer_does_nothing() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;

    // Layer 5 doesn't unlock any region
    if let Some(region) = PostgameRegion::from_layer(5) {
        let new_cap = region.end_zone_id();
        if new_cap > deep.persistent.postgame_zone_cap {
            deep.persistent.postgame_zone_cap = new_cap;
            deep.persistent.pending_postgame_region_unlock = Some(region);
        }
    }

    assert_eq!(deep.persistent.postgame_zone_cap, 11); // unchanged
    assert!(deep.persistent.pending_postgame_region_unlock.is_none());
}
