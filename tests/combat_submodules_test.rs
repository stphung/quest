#![allow(clippy::field_reassign_with_default)]
//! Comprehensive tests for combat submodules to improve coverage:
//! - combat/events.rs (CombatBonuses, CombatBonuses, CombatEvent)
//! - combat/attacks.rs (effective_enemy_attack_interval)
//! - combat/regen.rs (process_regen via update_combat)
//! - combat/orchestration.rs (update_combat early returns and attack phases)
//! - combat/damage.rs (handle_enemy_death via update_combat)
//! - combat/player_attack.rs (resolve_player_attack via update_combat)
//! - combat/enemy_attack.rs (resolve_enemy_attack via update_combat)

use quest::achievements::Achievements;
use quest::character::attributes::AttributeType;
use quest::character::derived_stats::DerivedStats;
use quest::combat::attacks::effective_enemy_attack_interval;
use quest::combat::events::{CombatBonuses, CombatEvent};
use quest::combat::logic::update_combat;
use quest::combat::types::Enemy;
use quest::core::constants::*;
use quest::core::game_state::GameState;
use quest::dungeon::generation::generate_dungeon;
use quest::dungeon::types::RoomType;
use quest::zones::get_all_zones;
use rand::SeedableRng;

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

fn fresh_state() -> GameState {
    GameState::new("CombatTest".to_string(), 0)
}

fn derived(state: &GameState) -> DerivedStats {
    DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7])
}

fn default_bonuses() -> CombatBonuses {
    CombatBonuses::default()
}
fn seeded_rng() -> rand_chacha::ChaCha8Rng {
    rand_chacha::ChaCha8Rng::seed_from_u64(42)
}

/// Creates a state with an enemy set up for combat.
fn state_with_enemy(enemy_hp: u32, enemy_dmg: u32, enemy_def: u32) -> GameState {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(Enemy::new_with_defense(
        "Test Enemy".to_string(),
        enemy_hp,
        enemy_dmg,
        enemy_def,
    ));
    state
}

/// Creates a state with a very strong player and weak enemy (guarantees kill).
fn state_with_weak_enemy() -> GameState {
    let mut state = fresh_state();
    state.attributes.set(AttributeType::Strength, 100);
    state.attributes.set(AttributeType::Intelligence, 100);
    state.combat_state.current_enemy = Some(Enemy::new("Weak".to_string(), 1, 1));
    state
}

/// Creates a state where the player will be killed by the enemy.
fn state_player_about_to_die() -> GameState {
    let mut state = fresh_state();
    state.combat_state.player_current_hp = 1;
    state.combat_state.current_enemy =
        Some(Enemy::new_with_defense("Killer".to_string(), 9999, 9999, 0));
    state
}

/// Force a player attack by setting timer to interval, suppressing enemy.
fn force_player_attack(
    rng: &mut impl rand::Rng,
    state: &mut GameState,
    bonuses: &CombatBonuses,
) -> Vec<CombatEvent> {
    let d = derived(state);
    state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
    state.combat_state.enemy_attack_timer = 0.0;
    let mut achievements = Achievements::default();
    update_combat(
        rng,
        state,
        0.0, // zero delta so enemy timer stays at 0
        bonuses,
        &mut achievements,
        &d,
    )
}

/// Force an enemy attack by setting enemy timer high, suppressing player.
fn force_enemy_attack(
    rng: &mut impl rand::Rng,
    state: &mut GameState,
    bonuses: &CombatBonuses,
) -> Vec<CombatEvent> {
    let d = derived(state);
    state.combat_state.player_attack_timer = 0.0;
    state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS;
    let mut achievements = Achievements::default();
    update_combat(rng, state, 0.0, bonuses, &mut achievements, &d)
}

// ═══════════════════════════════════════════════════════════════════
// 1. combat/events.rs — CombatBonuses, CombatBonuses
// ═══════════════════════════════════════════════════════════════════

#[test]
fn haven_combat_bonuses_default_all_zeros() {
    let h = CombatBonuses::default();
    assert!((h.hp_regen_percent - 0.0).abs() < f64::EPSILON);
    assert!((h.hp_regen_delay_reduction - 0.0).abs() < f64::EPSILON);
    assert!((h.damage_percent - 0.0).abs() < f64::EPSILON);
    assert!((h.crit_chance_percent - 0.0).abs() < f64::EPSILON);
    assert!((h.double_strike_chance - 0.0).abs() < f64::EPSILON);
    assert!((h.xp_gain_percent - 0.0).abs() < f64::EPSILON);
}

#[test]
fn god_item_combat_bonuses_default_all_zeros() {
    let g = CombatBonuses::default();
    assert!((g.damage_reduction_percent - 0.0).abs() < f64::EPSILON);
    assert!((g.attack_speed_percent - 0.0).abs() < f64::EPSILON);
    assert!((g.regen_reduction_percent - 0.0).abs() < f64::EPSILON);
    assert!((g.damage_percent - 0.0).abs() < f64::EPSILON);
}

#[test]
fn haven_combat_bonuses_fields_assignable() {
    let h = CombatBonuses {
        hp_regen_percent: 10.0,
        hp_regen_delay_reduction: 20.0,
        damage_percent: 30.0,
        crit_chance_percent: 5.0,
        double_strike_chance: 15.0,
        xp_gain_percent: 25.0,
        ..CombatBonuses::default()
    };
    assert!((h.hp_regen_percent - 10.0).abs() < f64::EPSILON);
    assert!((h.hp_regen_delay_reduction - 20.0).abs() < f64::EPSILON);
    assert!((h.damage_percent - 30.0).abs() < f64::EPSILON);
    assert!((h.crit_chance_percent - 5.0).abs() < f64::EPSILON);
    assert!((h.double_strike_chance - 15.0).abs() < f64::EPSILON);
    assert!((h.xp_gain_percent - 25.0).abs() < f64::EPSILON);
}

#[test]
fn god_item_combat_bonuses_fields_assignable() {
    let g = CombatBonuses {
        damage_reduction_percent: 30.0,
        attack_speed_percent: 100.0,
        regen_reduction_percent: 50.0,
        damage_percent: 150.0,
        ..CombatBonuses::default()
    };
    assert!((g.damage_reduction_percent - 30.0).abs() < f64::EPSILON);
    assert!((g.attack_speed_percent - 100.0).abs() < f64::EPSILON);
    assert!((g.regen_reduction_percent - 50.0).abs() < f64::EPSILON);
    assert!((g.damage_percent - 150.0).abs() < f64::EPSILON);
}

#[test]
fn combat_event_player_attack_variant() {
    let ev = CombatEvent::PlayerAttack {
        damage: 42,
        was_crit: true,
    };
    match ev {
        CombatEvent::PlayerAttack { damage, was_crit } => {
            assert_eq!(damage, 42);
            assert!(was_crit);
        }
        _ => panic!("Expected PlayerAttack"),
    }
}

#[test]
fn combat_event_player_attack_blocked_variant() {
    let ev = CombatEvent::PlayerAttackBlocked {
        weapon_needed: "Stormbreaker".to_string(),
    };
    match ev {
        CombatEvent::PlayerAttackBlocked { weapon_needed } => {
            assert_eq!(weapon_needed, "Stormbreaker");
        }
        _ => panic!("Expected PlayerAttackBlocked"),
    }
}

#[test]
fn combat_event_enemy_attack_variant() {
    let ev = CombatEvent::EnemyAttack { damage: 15 };
    match ev {
        CombatEvent::EnemyAttack { damage } => assert_eq!(damage, 15),
        _ => panic!("Expected EnemyAttack"),
    }
}

#[test]
fn combat_event_damage_reflected_variant() {
    let ev = CombatEvent::DamageReflected { damage: 5 };
    match ev {
        CombatEvent::DamageReflected { damage } => assert_eq!(damage, 5),
        _ => panic!("Expected DamageReflected"),
    }
}

#[test]
fn combat_event_player_died_variant() {
    let ev = CombatEvent::PlayerDied;
    assert!(matches!(ev, CombatEvent::PlayerDied));
}

#[test]
fn combat_event_player_died_in_dungeon_variant() {
    let ev = CombatEvent::PlayerDiedInDungeon;
    assert!(matches!(ev, CombatEvent::PlayerDiedInDungeon));
}

#[test]
fn combat_event_enemy_died_variant() {
    let ev = CombatEvent::EnemyDied { xp_gained: 300 };
    match ev {
        CombatEvent::EnemyDied { xp_gained } => assert_eq!(xp_gained, 300),
        _ => panic!("Expected EnemyDied"),
    }
}

#[test]
fn combat_event_elite_defeated_variant() {
    let ev = CombatEvent::EliteDefeated { xp_gained: 500 };
    match ev {
        CombatEvent::EliteDefeated { xp_gained } => assert_eq!(xp_gained, 500),
        _ => panic!("Expected EliteDefeated"),
    }
}

#[test]
fn combat_event_boss_defeated_variant() {
    let ev = CombatEvent::BossDefeated { xp_gained: 1000 };
    match ev {
        CombatEvent::BossDefeated { xp_gained } => assert_eq!(xp_gained, 1000),
        _ => panic!("Expected BossDefeated"),
    }
}

#[test]
fn combat_event_regen_complete_variant() {
    let ev = CombatEvent::RegenComplete { healed: 25 };
    match ev {
        CombatEvent::RegenComplete { healed } => assert_eq!(healed, 25),
        _ => panic!("Expected RegenComplete"),
    }
}

// ═══════════════════════════════════════════════════════════════════
// 2. combat/attacks.rs — effective_enemy_attack_interval
// ═══════════════════════════════════════════════════════════════════

#[test]
fn attack_interval_normal_mob() {
    let state = state_with_enemy(50, 10, 0);
    let interval = effective_enemy_attack_interval(&state);
    assert!(
        (interval - ENEMY_ATTACK_INTERVAL_SECONDS).abs() < f64::EPSILON,
        "Normal mob should attack at {} but got {}",
        ENEMY_ATTACK_INTERVAL_SECONDS,
        interval
    );
}

#[test]
fn attack_interval_overworld_subzone_boss() {
    let mut state = state_with_enemy(200, 20, 5);
    state.zone_progression.fighting_boss = true;
    state.zone_progression.current_zone_id = 1;
    state.zone_progression.current_subzone_id = 1; // Not the last subzone
    let interval = effective_enemy_attack_interval(&state);
    assert!(
        (interval - ENEMY_BOSS_ATTACK_INTERVAL_SECONDS).abs() < f64::EPSILON,
        "Subzone boss should attack at {} but got {}",
        ENEMY_BOSS_ATTACK_INTERVAL_SECONDS,
        interval
    );
}

#[test]
fn attack_interval_overworld_zone_boss() {
    let zones = get_all_zones();
    let zone1 = &zones[0]; // Meadow has 3 subzones
    let last_subzone_id = zone1.subzones.len() as u32;

    let mut state = state_with_enemy(500, 50, 10);
    state.zone_progression.fighting_boss = true;
    state.zone_progression.current_zone_id = 1;
    state.zone_progression.current_subzone_id = last_subzone_id;

    let interval = effective_enemy_attack_interval(&state);
    assert!(
        (interval - ENEMY_ZONE_BOSS_ATTACK_INTERVAL_SECONDS).abs() < f64::EPSILON,
        "Zone boss should attack at {} but got {}",
        ENEMY_ZONE_BOSS_ATTACK_INTERVAL_SECONDS,
        interval
    );
}

#[test]
fn attack_interval_dungeon_normal_room() {
    let mut state = state_with_enemy(50, 10, 0);
    let dungeon = generate_dungeon(1, 0, 1);
    // Move to entrance (which is a non-boss/non-elite room)
    // We need a Combat room, but the entrance is fine for normal interval
    state.active_dungeon = Some(dungeon);
    let interval = effective_enemy_attack_interval(&state);
    // Entrance room -> normal attack interval
    assert!(
        (interval - ENEMY_ATTACK_INTERVAL_SECONDS).abs() < f64::EPSILON,
        "Dungeon normal room should use {} but got {}",
        ENEMY_ATTACK_INTERVAL_SECONDS,
        interval
    );
}

#[test]
fn attack_interval_dungeon_elite_room() {
    let mut state = state_with_enemy(100, 20, 5);
    let mut dungeon = generate_dungeon(1, 0, 1);
    // Find an Elite room in the dungeon and move player there
    let elite_pos = find_room_position(&dungeon, RoomType::Elite);
    if let Some(pos) = elite_pos {
        dungeon.player_position = pos;
        state.active_dungeon = Some(dungeon);
        let interval = effective_enemy_attack_interval(&state);
        assert!(
            (interval - ENEMY_DUNGEON_ELITE_ATTACK_INTERVAL_SECONDS).abs() < f64::EPSILON,
            "Dungeon elite should attack at {} but got {}",
            ENEMY_DUNGEON_ELITE_ATTACK_INTERVAL_SECONDS,
            interval
        );
    }
}

#[test]
fn attack_interval_dungeon_boss_room() {
    let mut state = state_with_enemy(300, 40, 10);
    let mut dungeon = generate_dungeon(1, 0, 1);
    let boss_pos = find_room_position(&dungeon, RoomType::Boss);
    if let Some(pos) = boss_pos {
        dungeon.player_position = pos;
        state.active_dungeon = Some(dungeon);
        let interval = effective_enemy_attack_interval(&state);
        assert!(
            (interval - ENEMY_DUNGEON_BOSS_ATTACK_INTERVAL_SECONDS).abs() < f64::EPSILON,
            "Dungeon boss should attack at {} but got {}",
            ENEMY_DUNGEON_BOSS_ATTACK_INTERVAL_SECONDS,
            interval
        );
    }
}

/// Helper to find a room of a specific type in a dungeon grid.
/// Returns (x, y) matching player_position convention.
fn find_room_position(
    dungeon: &quest::dungeon::Dungeon,
    room_type: RoomType,
) -> Option<(usize, usize)> {
    // grid is indexed as grid[y][x], player_position is (x, y)
    for (y, row) in dungeon.grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if let Some(room) = cell {
                if room.room_type == room_type {
                    return Some((x, y));
                }
            }
        }
    }
    None
}

// ═══════════════════════════════════════════════════════════════════
// 3. combat/regen.rs — process_regen via update_combat
// ═══════════════════════════════════════════════════════════════════

#[test]
fn regen_is_triggered_after_enemy_death() {
    let mut rng = seeded_rng();
    let mut state = state_with_weak_enemy();

    // Kill the enemy
    let events = force_player_attack(&mut rng, &mut state, &default_bonuses());

    // Check that enemy died
    let has_kill_event = events
        .iter()
        .any(|e| matches!(e, CombatEvent::EnemyDied { .. }));
    assert!(has_kill_event, "Expected EnemyDied event");

    // State should now be regenerating
    assert!(state.combat_state.is_regenerating);
    assert!(state.combat_state.current_enemy.is_none());
    assert_eq!(state.combat_state.regen_timer, 0.0);
}

#[test]
fn regen_advances_timer_during_regeneration() {
    let mut rng = seeded_rng();
    let mut state = fresh_state();
    state.combat_state.is_regenerating = true;
    state.combat_state.regen_timer = 0.0;
    state.combat_state.player_current_hp = 30;
    state.combat_state.regen_start_hp = 30;
    state.combat_state.player_max_hp = 50;

    let d = derived(&state);
    let mut ach = Achievements::default();

    let _events = update_combat(&mut rng, &mut state, 0.5, &default_bonuses(), &mut ach, &d);

    // Timer should have advanced
    assert!(
        state.combat_state.regen_timer > 0.0,
        "Regen timer should advance"
    );
    // Should still be regenerating (not complete in 0.5s with 2.5s duration)
    assert!(state.combat_state.is_regenerating);
}

#[test]
fn regen_completes_after_full_duration() {
    let mut rng = seeded_rng();
    let mut state = fresh_state();
    state.combat_state.is_regenerating = true;
    state.combat_state.regen_timer = 0.0;
    state.combat_state.player_current_hp = 30;
    state.combat_state.regen_start_hp = 30;
    state.combat_state.player_max_hp = 50;

    let d = derived(&state);
    let mut ach = Achievements::default();

    // Pass enough time to complete regen (2.5s default)
    let events = update_combat(
        &mut rng,
        &mut state,
        HP_REGEN_DURATION_SECONDS + 0.1,
        &default_bonuses(),
        &mut ach,
        &d,
    );

    // Regen should be complete
    assert!(
        !state.combat_state.is_regenerating,
        "Should no longer be regenerating"
    );
    assert_eq!(
        state.combat_state.player_current_hp, state.combat_state.player_max_hp,
        "HP should be fully restored"
    );
    // Should emit RegenComplete event
    let has_regen_event = events
        .iter()
        .any(|e| matches!(e, CombatEvent::RegenComplete { .. }));
    assert!(has_regen_event, "Expected RegenComplete event");
}

#[test]
fn regen_with_haven_bonus_completes_faster() {
    let mut rng = seeded_rng();
    let mut state = fresh_state();
    state.combat_state.is_regenerating = true;
    state.combat_state.regen_timer = 0.0;
    state.combat_state.player_current_hp = 30;
    state.combat_state.regen_start_hp = 30;
    state.combat_state.player_max_hp = 50;

    let bonuses = CombatBonuses {
        hp_regen_delay_reduction: 50.0, // 50% reduction
        ..CombatBonuses::default()
    };

    let d = derived(&state);
    let mut ach = Achievements::default();

    // At 50% reduction, effective duration is ~1.25s. Use 1.5s.
    let _events = update_combat(&mut rng, &mut state, 1.5, &bonuses, &mut ach, &d);

    assert!(
        !state.combat_state.is_regenerating,
        "Regen with haven bonus should complete in 1.5s"
    );
}

#[test]
fn regen_with_god_item_bonus_completes_faster() {
    let mut rng = seeded_rng();
    let mut state = fresh_state();
    state.combat_state.is_regenerating = true;
    state.combat_state.regen_timer = 0.0;
    state.combat_state.player_current_hp = 30;
    state.combat_state.regen_start_hp = 30;
    state.combat_state.player_max_hp = 50;

    let bonuses = CombatBonuses {
        regen_reduction_percent: 50.0, // Sleipnir: 50% reduction
        ..CombatBonuses::default()
    };

    let d = derived(&state);
    let mut ach = Achievements::default();

    let _events = update_combat(&mut rng, &mut state, 1.5, &bonuses, &mut ach, &d);

    assert!(
        !state.combat_state.is_regenerating,
        "Regen with god item bonus should complete faster"
    );
}

#[test]
fn regen_at_full_hp_emits_zero_heal() {
    let mut rng = seeded_rng();
    let mut state = fresh_state();
    state.combat_state.is_regenerating = true;
    state.combat_state.regen_timer = 0.0;
    state.combat_state.player_current_hp = 50;
    state.combat_state.regen_start_hp = 50;
    state.combat_state.player_max_hp = 50;

    let d = derived(&state);
    let mut ach = Achievements::default();

    // Complete the regen
    let events = update_combat(
        &mut rng,
        &mut state,
        HP_REGEN_DURATION_SECONDS + 0.1,
        &default_bonuses(),
        &mut ach,
        &d,
    );

    // When healing from max to max (0 healed), no RegenComplete event should fire
    let has_regen_event = events
        .iter()
        .any(|e| matches!(e, CombatEvent::RegenComplete { .. }));
    assert!(
        !has_regen_event,
        "Should not emit RegenComplete when already at full HP"
    );
    assert!(!state.combat_state.is_regenerating);
}

// ═══════════════════════════════════════════════════════════════════
// 4. combat/orchestration.rs — update_combat early returns
// ═══════════════════════════════════════════════════════════════════

#[test]
fn update_combat_returns_empty_when_no_enemy() {
    let mut rng = seeded_rng();
    let mut state = fresh_state();
    // No enemy, not regenerating
    assert!(state.combat_state.current_enemy.is_none());
    assert!(!state.combat_state.is_regenerating);

    let d = derived(&state);
    let mut ach = Achievements::default();

    let events = update_combat(&mut rng, &mut state, 0.1, &default_bonuses(), &mut ach, &d);

    assert!(events.is_empty(), "No events when no enemy and not regen");
}

#[test]
fn update_combat_delegates_to_regen_when_regenerating() {
    let mut rng = seeded_rng();
    let mut state = fresh_state();
    state.combat_state.is_regenerating = true;
    state.combat_state.regen_timer = 0.0;
    state.combat_state.player_current_hp = 40;
    state.combat_state.regen_start_hp = 40;

    let d = derived(&state);
    let mut ach = Achievements::default();

    // When regenerating, update_combat should delegate to process_regen
    let _events = update_combat(&mut rng, &mut state, 0.1, &default_bonuses(), &mut ach, &d);

    // Timer should advance (delegation to regen module)
    assert!(state.combat_state.regen_timer > 0.0);
}

#[test]
fn update_combat_accumulates_both_timers() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(9999, 1, 0);
    // Both timers at zero
    state.combat_state.player_attack_timer = 0.0;
    state.combat_state.enemy_attack_timer = 0.0;

    let d = derived(&state);
    let mut ach = Achievements::default();

    update_combat(&mut rng, &mut state, 0.5, &default_bonuses(), &mut ach, &d);

    // Both timers should have accumulated 0.5
    assert!(
        (state.combat_state.player_attack_timer - 0.5).abs() < f64::EPSILON,
        "Player timer should accumulate delta_time"
    );
    assert!(
        (state.combat_state.enemy_attack_timer - 0.5).abs() < f64::EPSILON,
        "Enemy timer should accumulate delta_time"
    );
}

#[test]
fn update_combat_player_attacks_when_timer_ready() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(9999, 1, 0);
    // Set player timer just below threshold, then tick forward
    state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS - 0.05;
    state.combat_state.enemy_attack_timer = 0.0;

    let d = derived(&state);
    let mut ach = Achievements::default();

    let events = update_combat(&mut rng, &mut state, 0.1, &default_bonuses(), &mut ach, &d);

    let has_player_attack = events
        .iter()
        .any(|e| matches!(e, CombatEvent::PlayerAttack { .. }));
    assert!(
        has_player_attack,
        "Player should attack when timer is ready"
    );
    // Timer should be reset
    assert!(
        state.combat_state.player_attack_timer < 0.2,
        "Player timer should reset after attack"
    );
}

#[test]
fn update_combat_enemy_attacks_when_timer_ready() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(9999, 10, 0);
    state.combat_state.player_attack_timer = 0.0;
    state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS - 0.05;

    let d = derived(&state);
    let mut ach = Achievements::default();

    let events = update_combat(&mut rng, &mut state, 0.1, &default_bonuses(), &mut ach, &d);

    let has_enemy_attack = events
        .iter()
        .any(|e| matches!(e, CombatEvent::EnemyAttack { .. }));
    assert!(has_enemy_attack, "Enemy should attack when timer is ready");
}

#[test]
fn update_combat_player_kills_enemy_skips_enemy_attack() {
    let mut rng = seeded_rng();
    // If player kills enemy, enemy attack should not happen even if timer is ready
    let mut state = state_with_weak_enemy();
    state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
    state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS;

    let d = derived(&state);
    let mut ach = Achievements::default();

    let events = update_combat(&mut rng, &mut state, 0.0, &default_bonuses(), &mut ach, &d);

    // Should have player attack and enemy death, but no enemy attack
    let has_enemy_attack = events
        .iter()
        .any(|e| matches!(e, CombatEvent::EnemyAttack { .. }));
    assert!(
        !has_enemy_attack,
        "Enemy should not attack if killed by player this tick"
    );
    let has_kill = events
        .iter()
        .any(|e| matches!(e, CombatEvent::EnemyDied { .. }));
    assert!(has_kill, "Enemy should have died");
}

// ═══════════════════════════════════════════════════════════════════
// 5. combat/damage.rs — handle_enemy_death
// ═══════════════════════════════════════════════════════════════════

#[test]
fn handle_enemy_death_awards_xp() {
    let mut rng = seeded_rng();
    let mut state = state_with_weak_enemy();

    let events = force_player_attack(&mut rng, &mut state, &default_bonuses());

    let xp_event = events.iter().find_map(|e| match e {
        CombatEvent::EnemyDied { xp_gained } => Some(*xp_gained),
        _ => None,
    });
    assert!(xp_event.is_some(), "Should award XP on kill");
    assert!(xp_event.unwrap() > 0, "XP should be positive");
}

#[test]
fn handle_enemy_death_starts_regen() {
    let mut rng = seeded_rng();
    let mut state = state_with_weak_enemy();
    state.combat_state.player_current_hp = 30; // Take some damage first

    force_player_attack(&mut rng, &mut state, &default_bonuses());

    assert!(state.combat_state.is_regenerating);
    assert!(state.combat_state.current_enemy.is_none());
    assert_eq!(state.combat_state.enemy_attack_timer, 0.0);
    assert_eq!(state.combat_state.regen_start_hp, 30);
}

#[test]
fn handle_enemy_death_records_kill_for_zone_progression() {
    let mut rng = seeded_rng();
    let mut state = state_with_weak_enemy();
    let initial_kills = state.zone_progression.kills_in_subzone;

    force_player_attack(&mut rng, &mut state, &default_bonuses());

    // Kill tracking should increase
    assert!(
        state.zone_progression.kills_in_subzone > initial_kills
            || state.zone_progression.fighting_boss,
        "Should record kill or trigger boss"
    );
}

#[test]
fn handle_enemy_death_in_dungeon_does_not_affect_zone_progression() {
    let mut rng = seeded_rng();
    let mut state = state_with_weak_enemy();
    state.active_dungeon = Some(generate_dungeon(1, 0, 1));
    let initial_kills = state.zone_progression.kills_in_subzone;

    force_player_attack(&mut rng, &mut state, &default_bonuses());

    // Zone progression should not change in dungeons
    assert_eq!(
        state.zone_progression.kills_in_subzone, initial_kills,
        "Dungeon kills should not affect zone progression"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 6. combat/player_attack.rs — resolve_player_attack damage pipeline
// ═══════════════════════════════════════════════════════════════════

#[test]
fn player_attack_deals_damage_to_enemy() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(100, 1, 0);

    let events = force_player_attack(&mut rng, &mut state, &default_bonuses());

    let has_attack = events
        .iter()
        .any(|e| matches!(e, CombatEvent::PlayerAttack { .. }));
    assert!(has_attack, "Should emit PlayerAttack event");

    // Enemy should have taken damage
    let enemy = state.combat_state.current_enemy.as_ref().unwrap();
    assert!(
        enemy.current_hp < enemy.max_hp,
        "Enemy should have taken damage"
    );
}

#[test]
fn player_attack_damage_pipeline_with_haven_bonus() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(9999, 1, 0);

    // Get damage without haven bonus first
    let events_no_haven = force_player_attack(&mut rng, &mut state, &default_bonuses());
    let damage_no_haven = extract_player_damage(&events_no_haven);

    // Reset enemy
    state
        .combat_state
        .current_enemy
        .as_mut()
        .unwrap()
        .reset_hp();

    // Get damage with haven bonus (+100% damage)
    let bonuses_with_haven = CombatBonuses {
        damage_percent: 100.0,
        ..CombatBonuses::default()
    };
    let events_haven = force_player_attack(&mut rng, &mut state, &bonuses_with_haven);
    let damage_haven = extract_player_damage(&events_haven);

    assert!(
        damage_haven > damage_no_haven,
        "Haven damage bonus should increase damage: {} vs {}",
        damage_haven,
        damage_no_haven
    );
}

#[test]
fn player_attack_damage_pipeline_with_god_item_bonus() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(9999, 1, 0);

    // Get damage without god item bonus
    let events_base = force_player_attack(&mut rng, &mut state, &default_bonuses());
    let damage_base = extract_player_damage(&events_base);

    // Reset enemy
    state
        .combat_state
        .current_enemy
        .as_mut()
        .unwrap()
        .reset_hp();

    // Get damage with god item bonus (Giant's Might: 150% early damage)
    let bonuses_with_god = CombatBonuses {
        early_damage_percent: 150.0,
        ..CombatBonuses::default()
    };
    let events_god = force_player_attack(&mut rng, &mut state, &bonuses_with_god);
    let damage_god = extract_player_damage(&events_god);

    assert!(
        damage_god > damage_base,
        "God item damage bonus should increase damage: {} vs {}",
        damage_god,
        damage_base
    );
}

#[test]
fn player_attack_damage_pipeline_with_prestige_bonus() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(9999, 1, 0);

    // Get damage without prestige bonus
    let events_base = force_player_attack(&mut rng, &mut state, &default_bonuses());
    let damage_base = extract_player_damage(&events_base);

    // Reset enemy
    state
        .combat_state
        .current_enemy
        .as_mut()
        .unwrap()
        .reset_hp();

    // Get damage with prestige flat damage bonus (P10 = ~50 flat damage)
    let prestige_bonuses = CombatBonuses {
        flat_damage: 50,
        ..CombatBonuses::default()
    };
    let events_prestige = force_player_attack(&mut rng, &mut state, &prestige_bonuses);
    let damage_prestige = extract_player_damage(&events_prestige);

    assert!(
        damage_prestige > damage_base,
        "Prestige flat damage should increase damage: {} vs {}",
        damage_prestige,
        damage_base
    );
}

#[test]
fn player_attack_respects_enemy_defense() {
    let mut rng = seeded_rng();
    // Enemy with high defense should reduce damage
    let mut state = state_with_enemy(9999, 1, 1000);

    let events = force_player_attack(&mut rng, &mut state, &default_bonuses());
    let damage = extract_player_damage(&events);

    // With 1000 defense and ~10 base damage, should be clamped to min 1
    // (may be 2 if a crit occurs, since base crit chance is 5%)
    assert!(
        damage <= 2,
        "Damage should be min-clamped to 1 (or 2 with crit) against high defense, got {}",
        damage
    );
}

#[test]
fn player_attack_resets_attack_timer() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(9999, 1, 0);
    state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;

    force_player_attack(&mut rng, &mut state, &default_bonuses());

    assert!(
        state.combat_state.player_attack_timer < 0.01,
        "Player attack timer should be reset after attack"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 7. combat/enemy_attack.rs — resolve_enemy_attack
// ═══════════════════════════════════════════════════════════════════

#[test]
fn enemy_attack_deals_damage_to_player() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(9999, 10, 0); // Enemy with 10 damage (non-lethal)

    let initial_hp = state.combat_state.player_current_hp;

    let events = force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    let has_enemy_attack = events
        .iter()
        .any(|e| matches!(e, CombatEvent::EnemyAttack { .. }));
    assert!(has_enemy_attack, "Should emit EnemyAttack event");

    assert!(
        state.combat_state.player_current_hp < initial_hp,
        "Player should have taken damage: hp={} initial={}",
        state.combat_state.player_current_hp,
        initial_hp
    );
}

#[test]
fn enemy_attack_respects_player_defense() {
    let mut rng = seeded_rng();
    // High defense player
    let mut state = state_with_enemy(9999, 5, 0);
    state.attributes.set(AttributeType::Dexterity, 30); // High DEX = high defense

    // High DEX provides defense; test that enemy damage is min-clamped
    let events = force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    let damage_taken = extract_enemy_damage(&events);
    // With high defense vs 5 enemy damage, should clamp to min 1
    assert_eq!(
        damage_taken, 1,
        "Damage should be min 1 when defense exceeds enemy damage"
    );
}

#[test]
fn enemy_attack_divine_bulwark_reduces_damage() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(9999, 100, 0);
    let dr_bonuses = CombatBonuses {
        damage_reduction_percent: 30.0, // Asprika: 30% DR
        ..CombatBonuses::default()
    };

    let events = force_enemy_attack(&mut rng, &mut state, &dr_bonuses);
    let damage_with_dr = extract_enemy_damage(&events);

    // Without DR
    let mut state2 = state_with_enemy(9999, 100, 0);
    let events2 = force_enemy_attack(&mut rng, &mut state2, &default_bonuses());
    let damage_without_dr = extract_enemy_damage(&events2);

    assert!(
        damage_with_dr < damage_without_dr,
        "Divine Bulwark should reduce damage: {} vs {}",
        damage_with_dr,
        damage_without_dr
    );
}

#[test]
fn enemy_attack_resets_attack_timer() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(9999, 10, 0);
    state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS;

    force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    assert!(
        state.combat_state.enemy_attack_timer < 0.01,
        "Enemy attack timer should be reset after attack"
    );
}

#[test]
fn player_death_resets_hp_to_max() {
    let mut rng = seeded_rng();
    let mut state = state_player_about_to_die();

    let events = force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    let has_death = events.iter().any(|e| matches!(e, CombatEvent::PlayerDied));
    assert!(has_death, "Player should have died");

    // HP should be reset to max
    assert_eq!(
        state.combat_state.player_current_hp, state.combat_state.player_max_hp,
        "Player HP should be restored to max after death"
    );
}

#[test]
fn player_death_in_dungeon_exits_dungeon() {
    let mut rng = seeded_rng();
    let mut state = state_player_about_to_die();
    state.active_dungeon = Some(generate_dungeon(1, 0, 1));

    let events = force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    let has_dungeon_death = events
        .iter()
        .any(|e| matches!(e, CombatEvent::PlayerDiedInDungeon));
    assert!(has_dungeon_death, "Should emit PlayerDiedInDungeon");

    assert!(
        state.active_dungeon.is_none(),
        "Dungeon should be cleared on death"
    );
}

#[test]
fn player_death_to_subzone_boss_sets_retry_kills() {
    let mut rng = seeded_rng();
    let mut state = state_player_about_to_die();
    // Zone 1, subzone 1 — NOT a zone boss (zone boss is subzone 3)
    state.zone_progression.current_zone_id = 1;
    state.zone_progression.current_subzone_id = 1;
    state.zone_progression.fighting_boss = true;
    state.zone_progression.kills_in_subzone = KILLS_FOR_BOSS;

    force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    // Boss flag should be cleared
    assert!(
        !state.zone_progression.fighting_boss,
        "Boss flag should be cleared on death"
    );
    // Stays in same subzone
    assert_eq!(state.zone_progression.current_subzone_id, 1);
    // Kills should be set so only KILLS_FOR_BOSS_RETRY more kills needed
    assert_eq!(
        state.zone_progression.kills_in_subzone,
        KILLS_FOR_BOSS.saturating_sub(KILLS_FOR_BOSS_RETRY),
        "Kills should be set for retry (need {} more kills)",
        KILLS_FOR_BOSS_RETRY
    );
}

#[test]
fn player_death_to_zone_boss_resets_to_subzone_1() {
    let mut rng = seeded_rng();
    let mut state = state_player_about_to_die();
    // Zone 1, subzone 3 — the zone boss (Sporeling Queen)
    state.zone_progression.current_zone_id = 1;
    state.zone_progression.current_subzone_id = 3;
    state.zone_progression.fighting_boss = true;
    state.zone_progression.kills_in_subzone = KILLS_FOR_BOSS;

    force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    assert!(!state.zone_progression.fighting_boss);
    // Sent back to subzone 1
    assert_eq!(
        state.zone_progression.current_subzone_id, 1,
        "Death to zone boss should reset to subzone 1"
    );
    // Kills reset to 0 (fresh start)
    assert_eq!(
        state.zone_progression.kills_in_subzone, 0,
        "Kills should be fully reset after zone boss death"
    );
}

#[test]
fn player_death_to_undying_storm_resets_to_lightning_fields() {
    let mut rng = seeded_rng();
    let mut state = state_player_about_to_die();
    // Zone 10 (Storm Citadel), subzone 4 (Apex Spire) — The Undying Storm
    state.zone_progression.current_zone_id = 10;
    state.zone_progression.current_subzone_id = 4;
    state.zone_progression.fighting_boss = true;
    state.zone_progression.kills_in_subzone = KILLS_FOR_BOSS;

    force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    assert!(!state.zone_progression.fighting_boss);
    // Sent back to subzone 1 (Lightning Fields)
    assert_eq!(
        state.zone_progression.current_subzone_id, 1,
        "Death to The Undying Storm should reset to Lightning Fields (subzone 1)"
    );
    assert_eq!(state.zone_progression.kills_in_subzone, 0);
}

#[test]
fn player_death_to_zone_boss_preserves_zone_id() {
    let mut rng = seeded_rng();
    let mut state = state_player_about_to_die();
    // Zone 5 (Volcanic Wastes), subzone 4 (Magma Core) — Infernal Titan (zone boss)
    state.zone_progression.current_zone_id = 5;
    state.zone_progression.current_subzone_id = 4;
    state.zone_progression.fighting_boss = true;
    state.zone_progression.kills_in_subzone = KILLS_FOR_BOSS;

    force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    // Stays in same zone, just reset to subzone 1
    assert_eq!(
        state.zone_progression.current_zone_id, 5,
        "Zone should not change on zone boss death"
    );
    assert_eq!(
        state.zone_progression.current_subzone_id, 1,
        "Should reset to subzone 1 of same zone"
    );
    assert_eq!(state.zone_progression.kills_in_subzone, 0);
}

#[test]
fn player_death_to_expanse_zone_boss_resets_to_subzone_1() {
    let mut rng = seeded_rng();
    let mut state = state_player_about_to_die();
    // Zone 11 (The Expanse), subzone 4 (The Endless) — Avatar of Infinity (zone boss)
    state.zone_progression.current_zone_id = 11;
    state.zone_progression.current_subzone_id = 4;
    state.zone_progression.fighting_boss = true;
    state.zone_progression.kills_in_subzone = KILLS_FOR_BOSS;

    force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    assert_eq!(
        state.zone_progression.current_zone_id, 11,
        "Should stay in The Expanse"
    );
    assert_eq!(
        state.zone_progression.current_subzone_id, 1,
        "Death to Avatar of Infinity should reset to Void's Edge (subzone 1)"
    );
    assert_eq!(state.zone_progression.kills_in_subzone, 0);
}

#[test]
fn player_death_to_mid_subzone_boss_in_4_subzone_zone_uses_retry() {
    let mut rng = seeded_rng();
    let mut state = state_player_about_to_die();
    // Zone 5 (Volcanic Wastes), subzone 2 (Lava Rivers) — Magma Serpent (NOT zone boss)
    state.zone_progression.current_zone_id = 5;
    state.zone_progression.current_subzone_id = 2;
    state.zone_progression.fighting_boss = true;
    state.zone_progression.kills_in_subzone = KILLS_FOR_BOSS;

    force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    // Stays in same subzone with retry mechanic
    assert_eq!(state.zone_progression.current_zone_id, 5);
    assert_eq!(
        state.zone_progression.current_subzone_id, 2,
        "Should stay in same subzone for non-zone boss"
    );
    assert_eq!(
        state.zone_progression.kills_in_subzone,
        KILLS_FOR_BOSS.saturating_sub(KILLS_FOR_BOSS_RETRY),
        "Should use 5-kill retry for subzone boss"
    );
}

#[test]
fn player_death_to_zone_boss_emits_player_died_event() {
    let mut rng = seeded_rng();
    let mut state = state_player_about_to_die();
    // Zone 1, subzone 3 — Sporeling Queen (zone boss)
    state.zone_progression.current_zone_id = 1;
    state.zone_progression.current_subzone_id = 3;
    state.zone_progression.fighting_boss = true;
    state.zone_progression.kills_in_subzone = KILLS_FOR_BOSS;

    let events = force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    assert!(
        events.iter().any(|e| matches!(e, CombatEvent::PlayerDied)),
        "Should emit PlayerDied event on zone boss death"
    );
}

#[test]
fn player_death_to_normal_enemy_resets_enemy_hp() {
    let mut rng = seeded_rng();
    let mut state = fresh_state();
    state.combat_state.player_current_hp = 1;
    state.combat_state.current_enemy =
        Some(Enemy::new_with_defense("Strong".to_string(), 100, 9999, 0));

    force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    // Enemy should still exist with full HP (reset on player death)
    let enemy = state.combat_state.current_enemy.as_ref();
    assert!(
        enemy.is_some(),
        "Enemy should still exist after player death to normal mob"
    );
    assert_eq!(
        enemy.unwrap().current_hp,
        enemy.unwrap().max_hp,
        "Enemy HP should be reset"
    );
}

#[test]
fn player_death_resets_both_timers() {
    let mut rng = seeded_rng();
    let mut state = state_player_about_to_die();
    state.combat_state.player_attack_timer = 1.0;
    state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS;

    force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    assert_eq!(
        state.combat_state.player_attack_timer, 0.0,
        "Player timer should reset on death"
    );
    assert_eq!(
        state.combat_state.enemy_attack_timer, 0.0,
        "Enemy timer should reset on death"
    );
}

#[test]
fn damage_reflection_hurts_enemy() {
    let mut rng = seeded_rng();
    // Create a player with damage reflection gear
    let mut state = state_with_enemy(100, 50, 0);
    // We need DerivedStats with damage_reflection_percent > 0
    // Since we can't easily set gear in integration tests without full item creation,
    // we'll verify the code path by using a strong enemy that can't kill player in one hit
    // and checking that the pipeline at least runs the enemy attack code

    let events = force_enemy_attack(&mut rng, &mut state, &default_bonuses());

    // Without reflection gear, there should be no DamageReflected event
    let has_reflection = events
        .iter()
        .any(|e| matches!(e, CombatEvent::DamageReflected { .. }));
    assert!(!has_reflection, "No reflection without reflection gear");
}

#[test]
fn damage_reflection_with_gear_reflects_damage() {
    let mut rng = seeded_rng();
    use quest::items::types::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};

    let mut state = state_with_enemy(100, 50, 0);
    // Equip armor with damage reflection
    let armor = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Rare,
        ilvl: 10,
        tier: 5,
        base_name: "Armor".to_string(),
        display_name: "Thorned Armor".to_string(),
        attributes: AttributeBonuses::new(),
        affixes: vec![Affix {
            affix_type: AffixType::DamageReflection,
            value: 50.0, // 50% reflection
        }],
        god_item_id: None,
    };
    state.equipment.set(EquipmentSlot::Armor, Some(armor));

    // Use update_combat directly with properly calculated derived stats
    let d = derived(&state);
    state.combat_state.player_attack_timer = 0.0;
    state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS;
    let mut ach = Achievements::default();

    let events = update_combat(&mut rng, &mut state, 0.0, &default_bonuses(), &mut ach, &d);

    let has_reflection = events
        .iter()
        .any(|e| matches!(e, CombatEvent::DamageReflected { .. }));
    assert!(
        has_reflection,
        "Should reflect damage with 50% reflection gear"
    );

    // Verify reflected damage amount
    let reflected_amount = events.iter().find_map(|e| match e {
        CombatEvent::DamageReflected { damage } => Some(*damage),
        _ => None,
    });
    assert!(
        reflected_amount.unwrap() > 0,
        "Reflected damage should be > 0"
    );
}

// ═══════════════════════════════════════════════════════════════════
// Integration: Full combat cycle tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn full_combat_cycle_attack_kill_regen() {
    let mut rng = seeded_rng();
    let mut state = state_with_weak_enemy();
    state.combat_state.player_current_hp = 30; // Take some damage

    // Phase 1: Player attacks and kills enemy
    let _events = force_player_attack(&mut rng, &mut state, &default_bonuses());
    assert!(state.combat_state.is_regenerating);
    assert!(state.combat_state.current_enemy.is_none());

    // Phase 2: Regen tick
    let d = derived(&state);
    let mut ach = Achievements::default();
    update_combat(&mut rng, &mut state, 0.5, &default_bonuses(), &mut ach, &d);
    assert!(state.combat_state.is_regenerating, "Still regenerating");
    assert!(
        state.combat_state.player_current_hp > 30,
        "Should have regained some HP"
    );

    // Phase 3: Complete regen
    let _events = update_combat(
        &mut rng,
        &mut state,
        HP_REGEN_DURATION_SECONDS,
        &default_bonuses(),
        &mut ach,
        &d,
    );
    assert!(!state.combat_state.is_regenerating, "Regen should complete");
    assert_eq!(
        state.combat_state.player_current_hp, state.combat_state.player_max_hp,
        "Should be at full HP"
    );
}

#[test]
fn both_player_and_enemy_attack_same_tick() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(9999, 10, 0);
    // Set both timers ready
    state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
    state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS;

    let d = derived(&state);
    let mut ach = Achievements::default();

    let events = update_combat(&mut rng, &mut state, 0.0, &default_bonuses(), &mut ach, &d);

    let player_attacks = events
        .iter()
        .filter(|e| matches!(e, CombatEvent::PlayerAttack { .. }))
        .count();
    let enemy_attacks = events
        .iter()
        .filter(|e| matches!(e, CombatEvent::EnemyAttack { .. }))
        .count();

    assert!(player_attacks >= 1, "Player should attack");
    assert!(enemy_attacks >= 1, "Enemy should also attack");
}

#[test]
fn god_item_attack_speed_reduces_player_interval() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(9999, 1, 0);
    let speed_bonuses = CombatBonuses {
        attack_speed_percent: 100.0, // Sleipnir: 100% bonus speed
        ..CombatBonuses::default()
    };

    // Player timer at 0.75 (half of 1.5s base interval)
    // With 100% attack speed bonus, effective interval = 1.5 / (1.0 + 1.0) = 0.75
    state.combat_state.player_attack_timer = 0.7;
    state.combat_state.enemy_attack_timer = 0.0;

    let d = derived(&state);
    let mut ach = Achievements::default();

    let events = update_combat(&mut rng, &mut state, 0.1, &speed_bonuses, &mut ach, &d);

    let has_attack = events
        .iter()
        .any(|e| matches!(e, CombatEvent::PlayerAttack { .. }));
    assert!(
        has_attack,
        "Player should attack with reduced interval from god item speed bonus"
    );
}

// ═══════════════════════════════════════════════════════════════════
// Damage extraction helpers
// ═══════════════════════════════════════════════════════════════════

fn extract_player_damage(events: &[CombatEvent]) -> u32 {
    events
        .iter()
        .filter_map(|e| match e {
            CombatEvent::PlayerAttack { damage, .. } => Some(*damage),
            _ => None,
        })
        .sum()
}

fn extract_enemy_damage(events: &[CombatEvent]) -> u32 {
    events
        .iter()
        .filter_map(|e| match e {
            CombatEvent::EnemyAttack { damage } => Some(*damage),
            _ => None,
        })
        .sum()
}
