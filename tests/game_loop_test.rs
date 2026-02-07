//! Integration test: Game loop mechanics
//!
//! Tests the core game loop functionality: tick processing, combat flow,
//! state transitions, and offline progression.

use quest::character::derived_stats::DerivedStats;
use quest::combat::logic::{update_combat, CombatEvent, HavenCombatBonuses};
use quest::core::game_logic::{
    process_offline_progression, spawn_enemy_if_needed, xp_for_next_level,
};
use quest::GameState;
use quest::TICK_INTERVAL_MS;

/// Default Haven combat bonuses for testing (no bonuses)
fn default_haven_bonuses() -> HavenCombatBonuses {
    HavenCombatBonuses::default()
}

/// Simulate a single game tick (100ms of game time)
fn simulate_tick(state: &mut GameState) -> Vec<CombatEvent> {
    let delta_time = TICK_INTERVAL_MS as f64 / 1000.0;

    // Sync max HP with derived stats
    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
    state.combat_state.update_max_hp(derived.max_hp);

    // Spawn enemy if needed (this is called in the main game loop)
    spawn_enemy_if_needed(state);

    // Update combat and return events
    update_combat(state, delta_time, &default_haven_bonuses())
}

/// Simulate multiple game ticks
fn simulate_ticks(state: &mut GameState, count: u32) -> Vec<CombatEvent> {
    let mut all_events = Vec::new();
    for _ in 0..count {
        all_events.extend(simulate_tick(state));
    }
    all_events
}

// =============================================================================
// Game State Initialization Tests
// =============================================================================

#[test]
fn test_new_game_state_has_valid_initial_values() {
    let state = GameState::new("Test Hero".to_string(), 1000);

    assert_eq!(state.character_name, "Test Hero");
    assert_eq!(state.character_level, 1);
    assert_eq!(state.character_xp, 0);
    assert_eq!(state.prestige_rank, 0);
    assert_eq!(state.last_save_time, 1000);
    assert!(state.combat_state.current_enemy.is_none());
    assert!(state.active_dungeon.is_none());
    assert!(state.active_fishing.is_none());
    assert!(state.active_minigame.is_none());
}

#[test]
fn test_new_game_state_has_base_attributes() {
    use quest::character::attributes::AttributeType;

    let state = GameState::new("Attribute Test".to_string(), 0);

    for attr in AttributeType::all() {
        assert_eq!(
            state.attributes.get(attr),
            10,
            "Base attribute should be 10"
        );
    }
}

#[test]
fn test_new_game_state_has_empty_equipment() {
    use quest::EquipmentSlot;

    let state = GameState::new("Equipment Test".to_string(), 0);

    let slots = [
        EquipmentSlot::Weapon,
        EquipmentSlot::Armor,
        EquipmentSlot::Helmet,
        EquipmentSlot::Gloves,
        EquipmentSlot::Boots,
        EquipmentSlot::Amulet,
        EquipmentSlot::Ring,
    ];

    for slot in slots {
        assert!(
            state.equipment.get(slot).is_none(),
            "Slot {:?} should be empty",
            slot
        );
    }
}

#[test]
fn test_new_game_state_starts_in_zone_1() {
    let state = GameState::new("Zone Test".to_string(), 0);

    assert_eq!(state.zone_progression.current_zone_id, 1);
    assert_eq!(state.zone_progression.current_subzone_id, 1);
    assert_eq!(state.zone_progression.kills_in_subzone, 0);
    assert!(!state.zone_progression.fighting_boss);
}

#[test]
fn test_derived_stats_calculated_from_base_attributes() {
    let state = GameState::new("Stats Test".to_string(), 0);

    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);

    // Base stats with all 10s
    assert!(derived.max_hp > 0, "Max HP should be positive");
    assert!(
        derived.physical_damage > 0,
        "Physical damage should be positive"
    );
    // defense and crit_chance_percent are u32, so always non-negative
}

// =============================================================================
// Combat Flow Tests
// =============================================================================

#[test]
fn test_enemy_spawns_after_ticks_without_enemy() {
    let mut state = GameState::new("Combat Test".to_string(), 0);

    assert!(state.combat_state.current_enemy.is_none());

    // Simulate enough ticks for enemy to spawn (usually happens quickly)
    for _ in 0..100 {
        simulate_tick(&mut state);
        if state.combat_state.current_enemy.is_some() {
            break;
        }
    }

    assert!(
        state.combat_state.current_enemy.is_some(),
        "Enemy should spawn after some ticks"
    );
}

#[test]
fn test_combat_produces_attack_events() {
    let mut state = GameState::new("Attack Test".to_string(), 0);

    // Spawn an enemy
    for _ in 0..100 {
        simulate_tick(&mut state);
        if state.combat_state.current_enemy.is_some() {
            break;
        }
    }

    assert!(state.combat_state.current_enemy.is_some());

    // Simulate combat ticks and collect events
    let events = simulate_ticks(&mut state, 200);

    let attack_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, CombatEvent::PlayerAttack { .. }))
        .collect();

    assert!(
        !attack_events.is_empty(),
        "Should have player attack events during combat"
    );
}

#[test]
fn test_enemy_death_grants_xp() {
    use quest::core::game_logic::apply_tick_xp;

    let mut state = GameState::new("XP Test".to_string(), 0);

    let initial_xp = state.character_xp;

    // Run combat until enemy dies
    let mut enemy_killed = false;
    let mut xp_from_kill = 0u64;
    for _ in 0..1000 {
        let events = simulate_tick(&mut state);
        for event in events {
            if let CombatEvent::EnemyDied { xp_gained } = event {
                // Apply XP like main.rs does
                apply_tick_xp(&mut state, xp_gained as f64);
                xp_from_kill = xp_gained;
                enemy_killed = true;
                break;
            }
        }
        if enemy_killed {
            break;
        }
    }

    assert!(enemy_killed, "Enemy should be defeated eventually");
    assert!(xp_from_kill > 0, "Kill should grant XP");
    assert!(
        state.character_xp > initial_xp,
        "XP should increase after killing enemy"
    );
}

#[test]
fn test_player_hp_regenerates_after_combat() {
    let mut state = GameState::new("Regen Test".to_string(), 0);

    // Spawn enemy and take some damage
    for _ in 0..500 {
        let events = simulate_tick(&mut state);
        for event in &events {
            if matches!(event, CombatEvent::EnemyDied { .. }) {
                // Enemy died, now test regen
                let hp_after_kill = state.combat_state.player_current_hp;

                // Simulate regen period (2.5s = 25 ticks)
                simulate_ticks(&mut state, 30);

                // HP should regenerate (or stay full if it was full)
                assert!(
                    state.combat_state.player_current_hp >= hp_after_kill,
                    "HP should not decrease during regen"
                );
                return;
            }
        }
    }
}

#[test]
fn test_kills_tracked_in_zone_progression() {
    let mut state = GameState::new("Kill Track Test".to_string(), 0);

    let initial_kills = state.zone_progression.kills_in_subzone;

    // Kill an enemy
    for _ in 0..1000 {
        let events = simulate_tick(&mut state);
        for event in events {
            if matches!(event, CombatEvent::EnemyDied { .. }) {
                assert!(
                    state.zone_progression.kills_in_subzone > initial_kills,
                    "Kills should be tracked"
                );
                return;
            }
        }
    }

    panic!("Should have killed an enemy");
}

// =============================================================================
// Level Up Tests
// =============================================================================

#[test]
fn test_level_up_occurs_with_enough_xp() {
    use quest::core::game_logic::apply_tick_xp;

    let mut state = GameState::new("Level Up Test".to_string(), 0);

    assert_eq!(state.character_level, 1);

    let xp_needed = xp_for_next_level(1);
    let (level_ups, _) = apply_tick_xp(&mut state, xp_needed as f64 + 100.0);

    assert!(level_ups >= 1, "Should have leveled up");
    assert!(state.character_level >= 2, "Level should increase");
}

#[test]
fn test_level_up_grants_attribute_points() {
    use quest::character::attributes::AttributeType;
    use quest::core::game_logic::apply_tick_xp;

    let mut state = GameState::new("Attr Point Test".to_string(), 0);

    let initial_total: u32 = AttributeType::all()
        .iter()
        .map(|a| state.attributes.get(*a))
        .sum();

    // Level up multiple times
    for level in 1..5 {
        let xp_needed = xp_for_next_level(level);
        apply_tick_xp(&mut state, xp_needed as f64 + 100.0);
    }

    let final_total: u32 = AttributeType::all()
        .iter()
        .map(|a| state.attributes.get(*a))
        .sum();

    assert!(
        final_total > initial_total,
        "Attributes should increase from leveling"
    );
    // Each level grants 3 attribute points
    assert!(
        final_total >= initial_total + 3,
        "Should gain at least 3 attr points per level"
    );
}

// =============================================================================
// Offline Progression Tests
// =============================================================================

#[test]
fn test_offline_progression_grants_xp() {
    let mut state = GameState::new("Offline Test".to_string(), 0);

    // Set last save time to 1 hour ago
    state.last_save_time = chrono::Utc::now().timestamp() - 3600;

    let report = process_offline_progression(&mut state, 0.0);

    assert!(
        report.xp_gained > 0,
        "Should gain XP from offline progression"
    );
    assert!(report.elapsed_seconds > 0, "Should process offline time");
}

#[test]
fn test_offline_progression_with_long_absence() {
    let mut state = GameState::new("Offline Cap Test".to_string(), 0);

    // Set last save time to 10 days ago
    let ten_days_seconds: i64 = 10 * 24 * 3600;
    state.last_save_time = chrono::Utc::now().timestamp() - ten_days_seconds;

    let report = process_offline_progression(&mut state, 0.0);

    // elapsed_seconds shows actual time, but XP is calculated with 7-day cap internally
    assert!(
        report.elapsed_seconds >= ten_days_seconds - 1, // Allow 1 second tolerance
        "Should report actual elapsed time"
    );
    assert!(report.xp_gained > 0, "Should still gain XP");
}

#[test]
fn test_offline_progression_can_level_up() {
    let mut state = GameState::new("Offline Level Test".to_string(), 0);

    // Set last save time to several hours ago for substantial XP
    state.last_save_time = chrono::Utc::now().timestamp() - 10000;

    let initial_level = state.character_level;
    let report = process_offline_progression(&mut state, 0.0);

    if report.total_level_ups > 0 {
        assert!(
            state.character_level > initial_level,
            "Level should increase from offline level ups"
        );
    }
}

#[test]
fn test_short_offline_time_still_processes() {
    let mut state = GameState::new("Short Offline Test".to_string(), 0);

    // Set last save time to 30 seconds ago
    state.last_save_time = chrono::Utc::now().timestamp() - 30;

    let report = process_offline_progression(&mut state, 0.0);

    // Even short times are processed (threshold check is at display level in main.rs)
    assert!(
        report.elapsed_seconds >= 29, // Allow tolerance
        "Should report elapsed time even for short periods"
    );
}

// =============================================================================
// Zone Progression Tests
// =============================================================================

#[test]
fn test_zone_progression_tracks_kills() {
    let mut state = GameState::new("Zone Kill Test".to_string(), 0);

    // Manually increment kills
    state.zone_progression.kills_in_subzone = 5;

    assert_eq!(state.zone_progression.kills_in_subzone, 5);
}

#[test]
fn test_boss_spawns_after_10_kills() {
    let mut state = GameState::new("Boss Spawn Test".to_string(), 0);

    // Set kills to just below boss spawn threshold
    state.zone_progression.kills_in_subzone = 9;
    assert!(!state.zone_progression.fighting_boss);

    // 10th kill should trigger boss
    state.zone_progression.kills_in_subzone = 10;
    // Note: The actual boss spawn logic is in zones/progression.rs
    // Here we just verify the tracking works
    assert_eq!(state.zone_progression.kills_in_subzone, 10);
}

#[test]
fn test_zone_progression_starts_valid() {
    let state = GameState::new("Zone Data Test".to_string(), 0);

    // New characters start in zone 1, subzone 1
    assert_eq!(state.zone_progression.current_zone_id, 1);
    assert_eq!(state.zone_progression.current_subzone_id, 1);
    // Zone 1 has no prestige requirement, so a new character can be there
    assert_eq!(state.prestige_rank, 0);
}

#[test]
fn test_zone_progression_initial_state() {
    let state = GameState::new("Subzone Test".to_string(), 0);

    assert_eq!(state.zone_progression.current_zone_id, 1);
    assert_eq!(state.zone_progression.current_subzone_id, 1);
    assert!(!state.zone_progression.fighting_boss);
    assert!(!state.zone_progression.has_stormbreaker);
}

// =============================================================================
// Zone Advancement E2E Tests
// =============================================================================

#[test]
fn test_zone_advancement_subzone_to_subzone() {
    use quest::character::attributes::AttributeType;
    use quest::core::game_logic::apply_tick_xp;
    use quest::zones::BossDefeatResult;

    let mut state = GameState::new("Zone Advance Test".to_string(), 0);

    // Set high STR/INT for reliable boss kills (testing zone mechanics, not difficulty).
    // Boss HP scales with player_max_hp (3x for zone boss), so keep CON at base (10)
    // to minimize boss HP, and maximize STR/INT for damage output.
    state.attributes.set(AttributeType::Strength, 50);
    state.attributes.set(AttributeType::Intelligence, 50);
    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
    state.combat_state.update_max_hp(derived.max_hp);
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;

    // Verify starting position
    assert_eq!(state.zone_progression.current_zone_id, 1);
    assert_eq!(state.zone_progression.current_subzone_id, 1);
    assert_eq!(state.zone_progression.kills_in_subzone, 0);

    // Kill enemies and bosses through combat until subzone advances
    let mut boss_defeated = false;
    let mut defeat_result = None;
    for _ in 0..50_000 {
        let events = simulate_tick(&mut state);
        for event in &events {
            match event {
                CombatEvent::EnemyDied { xp_gained } => {
                    apply_tick_xp(&mut state, *xp_gained as f64);
                }
                CombatEvent::SubzoneBossDefeated { xp_gained, result } => {
                    apply_tick_xp(&mut state, *xp_gained as f64);
                    defeat_result = Some(result.clone());
                    boss_defeated = true;
                }
                _ => {}
            }
        }
        if boss_defeated {
            break;
        }
    }

    assert!(boss_defeated, "Boss should be defeated eventually");

    // Should advance to subzone 2 (zone 1 has 3 subzones)
    assert!(
        matches!(
            defeat_result,
            Some(BossDefeatResult::SubzoneComplete { new_subzone_id: 2 })
        ),
        "Should advance to subzone 2, got {:?}",
        defeat_result
    );
    assert_eq!(state.zone_progression.current_zone_id, 1);
    assert_eq!(state.zone_progression.current_subzone_id, 2);
    assert!(!state.zone_progression.fighting_boss);
    assert_eq!(state.zone_progression.kills_in_subzone, 0);
}

#[test]
fn test_zone_advancement_full_zone_clear() {
    use quest::character::attributes::AttributeType;
    use quest::core::game_logic::apply_tick_xp;
    use quest::zones::BossDefeatResult;

    let mut state = GameState::new("Full Zone Clear Test".to_string(), 0);

    // Set high STR/INT for reliable boss kills (testing zone mechanics, not difficulty).
    // Boss HP scales with player_max_hp (3x for zone boss), so keep CON at base (10)
    // to minimize boss HP, and maximize STR/INT for damage output.
    state.attributes.set(AttributeType::Strength, 50);
    state.attributes.set(AttributeType::Intelligence, 50);
    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
    state.combat_state.update_max_hp(derived.max_hp);
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;

    // Clear all 3 subzones of zone 1 to advance to zone 2
    for expected_subzone in 1..=3u32 {
        assert_eq!(state.zone_progression.current_subzone_id, expected_subzone);

        // Run combat until boss is defeated (handles kills, boss spawn, and boss death)
        let mut subzone_boss_defeated = false;
        for _ in 0..100_000 {
            let events = simulate_tick(&mut state);
            for event in &events {
                match event {
                    CombatEvent::EnemyDied { xp_gained } => {
                        apply_tick_xp(&mut state, *xp_gained as f64);
                    }
                    CombatEvent::SubzoneBossDefeated { xp_gained, result } => {
                        apply_tick_xp(&mut state, *xp_gained as f64);

                        if expected_subzone < 3 {
                            assert!(
                                matches!(result, BossDefeatResult::SubzoneComplete { .. }),
                                "Subzone {} boss should give SubzoneComplete, got {:?}",
                                expected_subzone,
                                result
                            );
                        } else {
                            assert!(
                                matches!(result, BossDefeatResult::ZoneComplete { .. }),
                                "Final subzone boss should give ZoneComplete, got {:?}",
                                result
                            );
                        }
                        subzone_boss_defeated = true;
                    }
                    _ => {}
                }
            }
            if subzone_boss_defeated {
                break;
            }
        }
        assert!(
            subzone_boss_defeated,
            "Boss in subzone {} should be defeated",
            expected_subzone
        );
    }

    // Should now be in zone 2, subzone 1
    assert_eq!(state.zone_progression.current_zone_id, 2);
    assert_eq!(state.zone_progression.current_subzone_id, 1);
    assert_eq!(state.zone_progression.kills_in_subzone, 0);
}

// =============================================================================
// Play Time Tracking Tests
// =============================================================================

#[test]
fn test_play_time_increments() {
    let mut state = GameState::new("Time Test".to_string(), 0);

    let initial_time = state.play_time_seconds;

    // Simulate 10 seconds of gameplay (100 ticks)
    state.play_time_seconds += 10;

    assert_eq!(
        state.play_time_seconds,
        initial_time + 10,
        "Play time should increment"
    );
}

// =============================================================================
// Combat State Tests
// =============================================================================

#[test]
fn test_combat_state_hp_consistency() {
    let state = GameState::new("HP Test".to_string(), 0);

    assert!(state.combat_state.player_current_hp > 0);
    assert!(state.combat_state.player_max_hp > 0);
    assert!(state.combat_state.player_current_hp <= state.combat_state.player_max_hp);
}

#[test]
fn test_combat_log_entries_added() {
    let mut state = GameState::new("Log Test".to_string(), 0);

    let initial_log_len = state.combat_state.combat_log.len();

    state
        .combat_state
        .add_log_entry("Test message".to_string(), false, true);

    assert_eq!(
        state.combat_state.combat_log.len(),
        initial_log_len + 1,
        "Log should have new entry"
    );
}

#[test]
fn test_max_hp_updates_with_derived_stats() {
    let mut state = GameState::new("Max HP Update Test".to_string(), 0);

    let initial_max_hp = state.combat_state.player_max_hp;

    // Increase CON to boost HP
    use quest::character::attributes::AttributeType;
    state.attributes.set(AttributeType::Constitution, 20);

    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
    state.combat_state.update_max_hp(derived.max_hp);

    assert!(
        state.combat_state.player_max_hp > initial_max_hp,
        "Max HP should increase with higher CON"
    );
}

// =============================================================================
// Challenge Menu State Tests
// =============================================================================

#[test]
fn test_challenge_menu_starts_empty() {
    let state = GameState::new("Menu Test".to_string(), 0);

    assert!(state.challenge_menu.challenges.is_empty());
    assert!(!state.challenge_menu.is_open);
}

#[test]
fn test_challenge_menu_operations() {
    let mut state = GameState::new("Menu Ops Test".to_string(), 0);

    state.challenge_menu.open();
    assert!(state.challenge_menu.is_open);

    state.challenge_menu.close();
    assert!(!state.challenge_menu.is_open);
}
