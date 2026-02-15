//! Integration test: Complete prestige cycle
//!
//! Tests the full flow: new character → level up → prestige → verify reset

use quest::achievements::Achievements;
use quest::character::attributes::AttributeType;
use quest::character::prestige::{
    can_prestige, get_prestige_tier, perform_prestige, PrestigeCombatBonuses,
};
use quest::core::game_logic::{apply_tick_xp, xp_for_next_level};
use quest::GameState;

/// Test a complete prestige cycle from level 1 to first prestige
#[test]
fn test_complete_prestige_cycle_first_prestige() {
    // Create a fresh character
    let mut state = GameState::new("Integration Test Hero".to_string(), 0);

    // Verify initial state
    assert_eq!(state.character_level, 1);
    assert_eq!(state.prestige_rank, 0);
    assert_eq!(state.total_prestige_count, 0);
    assert_eq!(state.get_attribute_cap(), 20);
    assert!(!can_prestige(&state));

    // Calculate total XP needed to reach level 10
    let mut total_xp_needed: u64 = 0;
    for level in 1..10 {
        total_xp_needed += xp_for_next_level(level);
    }

    // Apply XP in chunks to simulate gameplay
    let chunk_size = total_xp_needed / 5;
    for _ in 0..5 {
        let (levelups, increased_attrs) = apply_tick_xp(&mut state, chunk_size as f64);

        // Verify level-ups grant attribute points
        if levelups > 0 {
            assert_eq!(increased_attrs.len(), (levelups * 3) as usize);
        }
    }

    // Apply any remaining XP to ensure we hit level 10
    while state.character_level < 10 {
        apply_tick_xp(&mut state, 1000.0);
    }

    assert!(state.character_level >= 10);
    assert!(can_prestige(&state));

    // Record pre-prestige state
    let pre_prestige_level = state.character_level;
    let pre_prestige_xp = state.character_xp;

    // Verify attributes have increased from base
    let total_attrs: u32 = AttributeType::all()
        .iter()
        .map(|a| state.attributes.get(*a))
        .sum();
    assert!(
        total_attrs > 60,
        "Attributes should have increased from leveling"
    );

    // Perform prestige
    perform_prestige(&mut state);

    // Verify prestige completed
    assert_eq!(state.prestige_rank, 1);
    assert_eq!(state.total_prestige_count, 1);

    // Verify character reset
    assert_eq!(state.character_level, 1);
    assert_eq!(state.character_xp, 0);

    // Verify attributes reset to base 10
    for attr in AttributeType::all() {
        assert_eq!(
            state.attributes.get(attr),
            10,
            "Attribute {:?} should reset to 10",
            attr
        );
    }

    // Verify attribute cap increased
    assert_eq!(state.get_attribute_cap(), 25); // Base 20 + 5 per prestige

    // Verify combat state reset
    assert_eq!(state.combat_state.player_current_hp, 50); // Base HP
    assert_eq!(state.combat_state.player_max_hp, 50);
    assert!(!state.combat_state.is_regenerating);
    assert!(state.combat_state.current_enemy.is_none());

    // Verify equipment cleared
    assert!(state.equipment.iter_equipped().count() == 0);

    // Verify dungeon cleared
    assert!(state.active_dungeon.is_none());

    // Verify zone progression reset
    assert_eq!(state.zone_progression.current_zone_id, 1);
    assert_eq!(state.zone_progression.current_subzone_id, 1);

    // Verify prestige tier info is correct
    let tier = get_prestige_tier(state.prestige_rank);
    assert_eq!(tier.name, "Bronze");
    // New formula: 1 + 0.5 * rank^0.7, so P1 = 1.5
    assert!((tier.multiplier - 1.5).abs() < 0.001);

    println!(
        "Prestige cycle complete: Level {} -> Prestige 1 (was level {}, {} XP)",
        state.character_level, pre_prestige_level, pre_prestige_xp
    );
}

/// Test multiple prestige cycles
#[test]
fn test_multiple_prestige_cycles() {
    let mut state = GameState::new("Multi-Prestige Hero".to_string(), 0);

    // First prestige: level 10
    while state.character_level < 10 {
        apply_tick_xp(&mut state, 10000.0);
    }
    assert!(can_prestige(&state));
    perform_prestige(&mut state);
    assert_eq!(state.prestige_rank, 1);
    assert_eq!(state.get_attribute_cap(), 25);

    // Second prestige: level 25
    while state.character_level < 25 {
        apply_tick_xp(&mut state, 50000.0);
    }
    assert!(can_prestige(&state));
    perform_prestige(&mut state);
    assert_eq!(state.prestige_rank, 2);
    assert_eq!(state.get_attribute_cap(), 30);
    assert_eq!(state.total_prestige_count, 2);

    // Third prestige: level 50
    while state.character_level < 50 {
        apply_tick_xp(&mut state, 200000.0);
    }
    assert!(can_prestige(&state));
    perform_prestige(&mut state);
    assert_eq!(state.prestige_rank, 3);
    assert_eq!(state.get_attribute_cap(), 35);

    // Verify tier names progress correctly
    assert_eq!(get_prestige_tier(1).name, "Bronze");
    assert_eq!(get_prestige_tier(2).name, "Silver");
    assert_eq!(get_prestige_tier(3).name, "Gold");

    // Verify multiplier scaling (formula: 1 + 0.5 * rank^0.7)
    let tier3 = get_prestige_tier(3);
    let expected_p3 = 1.0 + 0.5 * 3.0_f64.powf(0.7); // ~2.08
    assert!((tier3.multiplier - expected_p3).abs() < 0.001);
}

/// Test prestige preserves fishing progress
#[test]
fn test_prestige_preserves_fishing() {
    let mut state = GameState::new("Fisher Hero".to_string(), 0);

    // Set up fishing progress
    state.fishing.rank = 15;
    state.fishing.total_fish_caught = 1000;
    state.fishing.legendary_catches = 10;

    // Level up and prestige
    while state.character_level < 10 {
        apply_tick_xp(&mut state, 10000.0);
    }
    perform_prestige(&mut state);

    // Verify fishing preserved
    assert_eq!(state.fishing.rank, 15);
    assert_eq!(state.fishing.total_fish_caught, 1000);
    assert_eq!(state.fishing.legendary_catches, 10);

    // But active session should be cleared
    assert!(state.active_fishing.is_none());
}

/// Test prestige at exactly required level
#[test]
fn test_prestige_at_exact_level() {
    let mut state = GameState::new("Precise Hero".to_string(), 0);

    // Get to exactly level 10
    while state.character_level < 10 {
        let needed = xp_for_next_level(state.character_level);
        let remaining = needed - state.character_xp;
        apply_tick_xp(&mut state, remaining as f64);
    }

    assert_eq!(state.character_level, 10);
    assert!(can_prestige(&state));

    // Prestige should work at exactly required level
    perform_prestige(&mut state);
    assert_eq!(state.prestige_rank, 1);
}

/// Test cannot prestige when not eligible
#[test]
fn test_cannot_prestige_when_ineligible() {
    let mut state = GameState::new("Impatient Hero".to_string(), 0);

    // Level to 9 (just below requirement)
    while state.character_level < 9 {
        apply_tick_xp(&mut state, 10000.0);
    }

    assert_eq!(state.character_level, 9);
    assert!(!can_prestige(&state));

    // Attempt prestige should do nothing
    let old_rank = state.prestige_rank;
    let old_level = state.character_level;
    perform_prestige(&mut state);

    assert_eq!(state.prestige_rank, old_rank);
    assert_eq!(state.character_level, old_level);
}

/// Test full combat → XP → level-up → prestige loop end-to-end
#[test]
fn test_combat_to_prestige_full_loop() {
    use quest::character::derived_stats::DerivedStats;
    use quest::character::prestige::{can_prestige, perform_prestige};
    use quest::combat::logic::{update_combat, CombatEvent, HavenCombatBonuses};
    use quest::core::game_logic::spawn_enemy_if_needed;
    use quest::TICK_INTERVAL_MS;

    let mut state = GameState::new("Combat Prestige Hero".to_string(), 0);
    let mut achievements = Achievements::default();
    let delta_time = TICK_INTERVAL_MS as f64 / 1000.0;

    assert_eq!(state.prestige_rank, 0);
    assert_eq!(state.character_level, 1);
    assert!(!can_prestige(&state));

    // Phase 1: Gain XP purely through combat kills until we reach prestige requirement
    let mut total_kills = 0u32;
    let target_level = 10;

    for tick in 0..20_000 {
        // Sync derived stats
        let derived =
            DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7]);
        state.combat_state.update_max_hp(derived.max_hp);

        // Spawn enemy
        spawn_enemy_if_needed(&mut state);

        // Combat tick
        let events = update_combat(
            &mut state,
            delta_time,
            &HavenCombatBonuses::default(),
            &PrestigeCombatBonuses::default(),
            &mut achievements,
            &derived,
        );

        // Apply XP from kills (mimics main.rs game loop)
        for event in &events {
            if let CombatEvent::EnemyDied { xp_gained }
            | CombatEvent::SubzoneBossDefeated { xp_gained, .. } = event
            {
                apply_tick_xp(&mut state, *xp_gained as f64);
                total_kills += 1;
            }
        }

        if state.character_level >= target_level {
            break;
        }

        // Safety: ensure we don't infinite loop
        assert!(
            tick < 199_999,
            "Should reach level {} within 200k ticks",
            target_level
        );
    }

    // Verify combat produced real progression
    assert!(total_kills > 0, "Should have killed enemies through combat");
    assert!(
        state.character_level >= target_level,
        "Should reach level {}",
        target_level
    );

    // Verify attributes increased from level-ups
    let total_attrs: u32 = AttributeType::all()
        .iter()
        .map(|a| state.attributes.get(*a))
        .sum();
    assert!(
        total_attrs > 60,
        "Should have gained attributes from leveling (got {} total)",
        total_attrs
    );

    // Phase 2: Prestige
    assert!(can_prestige(&state));
    perform_prestige(&mut state);

    // Verify complete reset
    assert_eq!(state.prestige_rank, 1);
    assert_eq!(state.total_prestige_count, 1);
    assert_eq!(state.character_level, 1);
    assert_eq!(state.character_xp, 0);

    for attr in AttributeType::all() {
        assert_eq!(
            state.attributes.get(attr),
            10,
            "{:?} should reset to 10",
            attr
        );
    }

    assert_eq!(state.combat_state.player_current_hp, 50);
    assert!(state.equipment.iter_equipped().count() == 0);
    assert!(state.active_dungeon.is_none());
    assert_eq!(state.zone_progression.current_zone_id, 1);
    assert_eq!(state.zone_progression.current_subzone_id, 1);

    // Phase 3: Verify post-prestige combat still works
    let mut post_prestige_kill = false;
    for _ in 0..2000 {
        let derived =
            DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7]);
        state.combat_state.update_max_hp(derived.max_hp);
        spawn_enemy_if_needed(&mut state);
        let events = update_combat(
            &mut state,
            delta_time,
            &HavenCombatBonuses::default(),
            &PrestigeCombatBonuses::default(),
            &mut achievements,
            &derived,
        );
        for event in &events {
            if matches!(event, CombatEvent::EnemyDied { .. }) {
                post_prestige_kill = true;
                break;
            }
        }
        if post_prestige_kill {
            break;
        }
    }
    assert!(post_prestige_kill, "Combat should work after prestige");
}

/// Test prestige XP multiplier affects future gains
#[test]
fn test_prestige_xp_multiplier() {
    use quest::core::game_logic::xp_gain_per_tick;

    // Compare XP rates at different prestige levels
    // Formula: 1 + 0.5 * rank^0.7
    let xp_rank_0 = xp_gain_per_tick(0, 0, 0);
    let xp_rank_1 = xp_gain_per_tick(1, 0, 0);
    let xp_rank_5 = xp_gain_per_tick(5, 0, 0);

    // Rank 1: 1 + 0.5 * 1^0.7 = 1.5
    assert!((xp_rank_1 / xp_rank_0 - 1.5).abs() < 0.001);

    // Rank 5: 1 + 0.5 * 5^0.7 ≈ 2.54
    let expected_p5 = 1.0 + 0.5 * 5.0_f64.powf(0.7);
    assert!((xp_rank_5 / xp_rank_0 - expected_p5).abs() < 0.01);
}
