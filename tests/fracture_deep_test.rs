use chrono::{Duration, Utc};
use quest::achievements::Achievements;
use quest::core::tick::{game_tick_with_context, TickEvent};
use quest::core::tick_context::TickContext;
use quest::deep::{
    DeepPersistent, DeepState, MercArchetype, MercStatus, Mercenary, Mission, MissionStatus,
    MissionType,
};
use quest::enhancement::EnhancementProgress;
use quest::haven::Haven;
use quest::zones::FractureRegion;
use quest::GameState;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[test]
fn test_deep_persistent_fracture_zone_cap_defaults_to_11() {
    let deep = DeepState::new();
    assert_eq!(deep.persistent.fracture_zone_cap, 11);
}

#[test]
fn test_deep_persistent_pending_region_defaults_to_none() {
    let deep = DeepState::new();
    assert!(deep.persistent.pending_fracture_region_unlock.is_none());
}

#[test]
fn test_deep_persistent_serde_defaults() {
    // Simulate loading from old save without new fields
    let json = r#"{"discovered":false,"guild_rank":1,"guild_upgrade_cost":500,"layers":[],"deepest_layer_reached":0,"merc_id_counter":0,"mission_id_counter":0}"#;
    let persistent: DeepPersistent = serde_json::from_str(json).unwrap();
    assert_eq!(persistent.fracture_zone_cap, 11);
    assert!(persistent.pending_fracture_region_unlock.is_none());
}

#[test]
fn test_deep_persistent_serde_round_trip_with_fracture_fields() {
    let mut deep = DeepState::new();
    deep.persistent.fracture_zone_cap = 14;
    deep.persistent.pending_fracture_region_unlock = Some(quest::zones::FractureRegion::RedFault);

    let json = serde_json::to_string(&deep.persistent).unwrap();
    let loaded: DeepPersistent = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.fracture_zone_cap, 14);
    assert_eq!(
        loaded.pending_fracture_region_unlock,
        Some(quest::zones::FractureRegion::RedFault)
    );
}

// ── Task 12: Deep Breakthrough Triggers Fracture Region Unlock ──────────

#[test]
fn test_layer_3_breakthrough_sets_cap_to_14() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.deepest_layer_reached = 3;

    // Simulate checking if a breakthrough should unlock a region
    if let Some(region) = FractureRegion::from_layer(3) {
        let new_cap = region.end_zone_id();
        if new_cap > deep.persistent.fracture_zone_cap {
            deep.persistent.fracture_zone_cap = new_cap;
            deep.persistent.pending_fracture_region_unlock = Some(region);
        }
    }

    assert_eq!(deep.persistent.fracture_zone_cap, 14);
    assert_eq!(
        deep.persistent.pending_fracture_region_unlock,
        Some(FractureRegion::RedFault)
    );
}

#[test]
fn test_layer_7_breakthrough_sets_cap_to_17() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.fracture_zone_cap = 14; // Already unlocked Red Fault

    if let Some(region) = FractureRegion::from_layer(7) {
        let new_cap = region.end_zone_id();
        if new_cap > deep.persistent.fracture_zone_cap {
            deep.persistent.fracture_zone_cap = new_cap;
            deep.persistent.pending_fracture_region_unlock = Some(region);
        }
    }

    assert_eq!(deep.persistent.fracture_zone_cap, 17);
    assert_eq!(
        deep.persistent.pending_fracture_region_unlock,
        Some(FractureRegion::MirrorScar)
    );
}

#[test]
fn test_layer_12_breakthrough_sets_cap_to_20() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.fracture_zone_cap = 17;

    if let Some(region) = FractureRegion::from_layer(12) {
        let new_cap = region.end_zone_id();
        if new_cap > deep.persistent.fracture_zone_cap {
            deep.persistent.fracture_zone_cap = new_cap;
            deep.persistent.pending_fracture_region_unlock = Some(region);
        }
    }

    assert_eq!(deep.persistent.fracture_zone_cap, 20);
    assert_eq!(
        deep.persistent.pending_fracture_region_unlock,
        Some(FractureRegion::BlackMouth)
    );
}

#[test]
fn test_repeated_breakthrough_does_not_downgrade_cap() {
    let mut deep = DeepState::new();
    deep.persistent.fracture_zone_cap = 17;
    deep.persistent.pending_fracture_region_unlock = None;

    // Layer 3 again shouldn't downgrade from 17 to 14
    if let Some(region) = FractureRegion::from_layer(3) {
        let new_cap = region.end_zone_id();
        if new_cap > deep.persistent.fracture_zone_cap {
            deep.persistent.fracture_zone_cap = new_cap;
            deep.persistent.pending_fracture_region_unlock = Some(region);
        }
    }

    assert_eq!(deep.persistent.fracture_zone_cap, 17); // unchanged
    assert!(deep.persistent.pending_fracture_region_unlock.is_none()); // not set
}

#[test]
fn test_non_unlock_layer_does_nothing() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;

    // Layer 5 doesn't unlock any region
    if let Some(region) = FractureRegion::from_layer(5) {
        let new_cap = region.end_zone_id();
        if new_cap > deep.persistent.fracture_zone_cap {
            deep.persistent.fracture_zone_cap = new_cap;
            deep.persistent.pending_fracture_region_unlock = Some(region);
        }
    }

    assert_eq!(deep.persistent.fracture_zone_cap, 11); // unchanged
    assert!(deep.persistent.pending_fracture_region_unlock.is_none());
}

// ── Integration Tests: Full game_tick Pipeline ──────────────────────────────
//
// These tests exercise the real production code path:
//   tick_all_missions() → tick_deep_missions() → FractureRegion::from_layer()
//   → fracture_zone_cap update → pending_fracture_region_unlock
//   → sync_account_zone_unlocks() → FractureRegionUnlocked event
//
// Unlike the unit tests above which re-implement the logic inline, these call
// game_tick() and verify end-to-end behavior.

/// Create an overpowered mercenary assigned to a mission.
fn test_merc(id: u64, mission_id: u64) -> Mercenary {
    Mercenary {
        id,
        name: format!("TestMerc{}", id),
        archetype: MercArchetype::Vanguard,
        power: 500, // Way above any layer threshold for guaranteed success
        resilience: 500,
        expertise: 100,
        level: 1,
        missions_completed: 0,
        status: MercStatus::OnMission(mission_id),
    }
}

/// Create a Breakthrough mission whose timer has already elapsed.
fn elapsed_breakthrough(id: u64, layer: u32, squad: Vec<u64>) -> Mission {
    let now = Utc::now();
    Mission {
        id,
        mission_type: MissionType::Breakthrough,
        layer,
        squad,
        started_at: now - Duration::hours(25),
        ends_at: now - Duration::hours(1),
        events: vec![],
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
        is_first_orders: false,
    }
}

/// Set up a DeepState with a single elapsed Breakthrough mission on the given layer.
/// Returns (deep, mission_id) so the caller can verify post-tick state.
fn deep_with_breakthrough(layer: u32, initial_cap: u32) -> DeepState {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.fracture_zone_cap = initial_cap;
    deep.persistent.merc_id_counter = 3;
    deep.persistent.mission_id_counter = 2;

    let mission_id = 1;
    deep.prestige.roster.push(test_merc(1, mission_id));
    deep.prestige.roster.push(test_merc(2, mission_id));
    deep.prestige
        .active_missions
        .push(elapsed_breakthrough(mission_id, layer, vec![1, 2]));

    deep
}

/// Run game_tick and find a seed that produces a successful Breakthrough.
/// With 500 power vs max threshold of 295 (layer 13), the ratio is always >= 1.5,
/// giving 95% success per seed. We try up to 20 seeds to find one that works.
fn run_tick_with_successful_breakthrough(
    layer: u32,
    initial_cap: u32,
) -> (GameState, DeepState, Achievements, Vec<TickEvent>) {
    for seed in 0..20 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut state = GameState::new("FractureTest".to_string(), 0);
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut achievements = Achievements::default();
        let mut deep = deep_with_breakthrough(layer, initial_cap);

        let mut ctx = TickContext {
            state: &mut state,
            tick_counter: &mut tick_counter,
            haven: &mut haven,
            enhancement: &mut enhancement,
            deep: &mut deep,
            achievements: &mut achievements,
            debug_mode: false,
        };
        let result = game_tick_with_context(&mut ctx, &mut rng);

        // Check if breakthrough succeeded (cap was raised)
        if deep.persistent.fracture_zone_cap > initial_cap {
            return (state, deep, achievements, result.events);
        }
    }
    panic!(
        "Failed to get successful Breakthrough on layer {} after 20 seeds",
        layer
    );
}

#[test]
fn test_game_tick_layer_3_breakthrough_unlocks_red_fault() {
    let (state, deep, _achievements, events) = run_tick_with_successful_breakthrough(3, 11);

    // Verify fracture_zone_cap was raised to 14 (Red Fault end zone)
    assert_eq!(deep.persistent.fracture_zone_cap, 14);

    // Verify pending was consumed (taken by stage 11d)
    assert!(deep.persistent.pending_fracture_region_unlock.is_none());

    // Verify FractureRegionUnlocked event was emitted for RedFault
    let fracture_events: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            TickEvent::FractureRegionUnlocked { region, .. } => Some(region),
            _ => None,
        })
        .collect();
    assert_eq!(fracture_events.len(), 1);
    assert_eq!(*fracture_events[0], FractureRegion::RedFault);

    // Verify zones 12-14 are now unlocked in zone_progression
    assert!(state.zone_progression.is_zone_unlocked(12));
    assert!(state.zone_progression.is_zone_unlocked(13));
    assert!(state.zone_progression.is_zone_unlocked(14));
    assert!(!state.zone_progression.is_zone_unlocked(15));
}

#[test]
fn test_game_tick_layer_7_breakthrough_unlocks_mirror_scar() {
    let (state, deep, _achievements, events) = run_tick_with_successful_breakthrough(7, 14);

    assert_eq!(deep.persistent.fracture_zone_cap, 17);
    assert!(deep.persistent.pending_fracture_region_unlock.is_none());

    let fracture_events: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            TickEvent::FractureRegionUnlocked { region, .. } => Some(region),
            _ => None,
        })
        .collect();
    assert_eq!(fracture_events.len(), 1);
    assert_eq!(*fracture_events[0], FractureRegion::MirrorScar);

    // Zones 12-14 should already be unlocked (initial cap was 14)
    // New zones 15-17 should now be unlocked
    assert!(state.zone_progression.is_zone_unlocked(15));
    assert!(state.zone_progression.is_zone_unlocked(16));
    assert!(state.zone_progression.is_zone_unlocked(17));
    assert!(!state.zone_progression.is_zone_unlocked(18));
}

#[test]
fn test_game_tick_layer_12_breakthrough_unlocks_black_mouth() {
    let (state, deep, _achievements, events) = run_tick_with_successful_breakthrough(12, 17);

    assert_eq!(deep.persistent.fracture_zone_cap, 20);
    assert!(deep.persistent.pending_fracture_region_unlock.is_none());

    let fracture_events: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            TickEvent::FractureRegionUnlocked { region, .. } => Some(region),
            _ => None,
        })
        .collect();
    assert_eq!(fracture_events.len(), 1);
    assert_eq!(*fracture_events[0], FractureRegion::BlackMouth);

    assert!(state.zone_progression.is_zone_unlocked(18));
    assert!(state.zone_progression.is_zone_unlocked(19));
    assert!(state.zone_progression.is_zone_unlocked(20));
}

#[test]
fn test_game_tick_layer_5_breakthrough_emits_no_fracture_event() {
    // Layer 5 is not a fracture unlock layer — no region should be emitted.
    for seed in 0..20 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut state = GameState::new("FractureTest".to_string(), 0);
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut achievements = Achievements::default();
        let mut deep = deep_with_breakthrough(5, 11);

        let mut ctx = TickContext {
            state: &mut state,
            tick_counter: &mut tick_counter,
            haven: &mut haven,
            enhancement: &mut enhancement,
            deep: &mut deep,
            achievements: &mut achievements,
            debug_mode: false,
        };
        let result = game_tick_with_context(&mut ctx, &mut rng);

        // Regardless of mission success/failure, no fracture event should fire
        let fracture_events: Vec<_> = result
            .events
            .iter()
            .filter(|e| matches!(e, TickEvent::FractureRegionUnlocked { .. }))
            .collect();
        assert!(
            fracture_events.is_empty(),
            "Layer 5 breakthrough should not emit FractureRegionUnlocked (seed={})",
            seed
        );

        // fracture_zone_cap should remain at 11
        assert_eq!(
            deep.persistent.fracture_zone_cap, 11,
            "Layer 5 breakthrough should not change fracture_zone_cap (seed={})",
            seed
        );
    }
}

#[test]
fn test_game_tick_breakthrough_fires_deep_breakthrough_achievement() {
    let (_state, _deep, achievements, _events) = run_tick_with_successful_breakthrough(3, 11);

    // Layer 3 breakthrough should trigger the deep breakthrough achievement handler.
    // The handler calls on_deep_breakthrough(3, ...) which checks milestone thresholds.
    // At minimum, the DeepBreakthrough achievement (first breakthrough) should unlock.
    assert!(
        achievements.is_unlocked(quest::achievements::AchievementId::FirstBreakthrough),
        "First breakthrough should unlock the FirstBreakthrough achievement"
    );
}
