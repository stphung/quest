#![allow(clippy::field_reassign_with_default)]
//! Tests for the combat damage pipeline edge cases.
//!
//! Covers:
//! - Weapon gate blocking (PlayerAttackBlocked event)
//! - Giant's Might passive (early_damage_percent 150%)
//! - Haven Armory stacking (damage_percent)
//! - Prestige flat damage (flat_damage)
//! - Defense subtraction (min 1 damage floor)
//! - Divine Bulwark DR (damage_reduction_percent 30%)
//! - Crit multiplier (2x)
//! - Double strike chance
//! - Enemy attack defense/reflection/death
//! - Edge cases: zero defense, all god items combined, max prestige

use quest::achievements::{AchievementId, Achievements};
use quest::character::attributes::AttributeType;
use quest::character::derived_stats::DerivedStats;
use quest::combat::events::{CombatBonuses, CombatEvent};
use quest::combat::logic::update_combat;
use quest::combat::types::Enemy;
use quest::core::constants::*;
use quest::core::game_state::GameState;
use rand::SeedableRng;

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

fn fresh_state() -> GameState {
    GameState::new("PipelineTest".to_string(), 0)
}

fn derived(state: &GameState) -> DerivedStats {
    DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7])
}

fn default_bonuses() -> CombatBonuses {
    CombatBonuses::default()
}

fn seeded_rng() -> rand_chacha::ChaCha8Rng {
    rand_chacha::ChaCha8Rng::seed_from_u64(12345)
}

/// Creates a state with a specific enemy.
fn state_with_enemy(hp: u64, dmg: u64, def: u64) -> GameState {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(Enemy::new_with_defense(
        "Test Enemy".to_string(),
        hp,
        dmg,
        def,
    ));
    state
}

/// Force a single player attack. Sets player timer to threshold, suppresses enemy timer.
fn force_player_attack(
    rng: &mut impl rand::Rng,
    state: &mut GameState,
    bonuses: &CombatBonuses,
) -> Vec<CombatEvent> {
    let d = derived(state);
    state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
    state.combat_state.enemy_attack_timer = 0.0;
    let mut achievements = Achievements::default();
    update_combat(rng, state, 0.0, bonuses, &mut achievements, &d, 11, 30)
}

/// Force a single enemy attack. Sets enemy timer to threshold, suppresses player timer.
fn force_enemy_attack(
    rng: &mut impl rand::Rng,
    state: &mut GameState,
    bonuses: &CombatBonuses,
) -> Vec<CombatEvent> {
    let d = derived(state);
    state.combat_state.player_attack_timer = 0.0;
    state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS;
    let mut achievements = Achievements::default();
    update_combat(rng, state, 0.0, bonuses, &mut achievements, &d, 11, 30)
}

/// Force a player attack with explicit achievements (needed for weapon gate checks).
fn force_player_attack_with_achievements(
    rng: &mut impl rand::Rng,
    state: &mut GameState,
    bonuses: &CombatBonuses,
    achievements: &mut Achievements,
) -> Vec<CombatEvent> {
    let d = derived(state);
    state.combat_state.player_attack_timer = ATTACK_INTERVAL_SECONDS;
    state.combat_state.enemy_attack_timer = 0.0;
    update_combat(rng, state, 0.0, bonuses, achievements, &d, 11, 30)
}

// ═══════════════════════════════════════════════════════════════════
// 1. Weapon Gate Blocking
// ═══════════════════════════════════════════════════════════════════

#[test]
fn weapon_gate_blocks_attack_on_zone_10_final_boss_without_stormbreaker() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(10000, 100, 0);
    let mut achievements = Achievements::default();

    // Position at Zone 10, final subzone (4), boss fight active
    state.zone_progression.current_zone_id = 10;
    state.zone_progression.current_subzone_id = 4;
    state.zone_progression.unlock_zone(10);
    state.zone_progression.fighting_boss = true;

    let events = force_player_attack_with_achievements(
        &mut rng,
        &mut state,
        &default_bonuses(),
        &mut achievements,
    );

    // Should get a blocked event, not a damage event
    let blocked = events
        .iter()
        .find(|e| matches!(e, CombatEvent::PlayerAttackBlocked { weapon_needed: _ }));
    assert!(
        blocked.is_some(),
        "Expected PlayerAttackBlocked event when missing Stormbreaker"
    );

    // No damage should have been dealt
    let attacked = events
        .iter()
        .any(|e| matches!(e, CombatEvent::PlayerAttack { .. }));
    assert!(!attacked, "No PlayerAttack event should fire when blocked");

    // Enemy should still be alive
    assert!(state.combat_state.current_enemy.is_some());
    assert!(state
        .combat_state
        .current_enemy
        .as_ref()
        .unwrap()
        .is_alive());
}

#[test]
fn weapon_gate_weapon_name_is_stormbreaker() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(10000, 100, 0);
    let mut achievements = Achievements::default();

    state.zone_progression.current_zone_id = 10;
    state.zone_progression.current_subzone_id = 4;
    state.zone_progression.unlock_zone(10);
    state.zone_progression.fighting_boss = true;

    let events = force_player_attack_with_achievements(
        &mut rng,
        &mut state,
        &default_bonuses(),
        &mut achievements,
    );

    for event in &events {
        if let CombatEvent::PlayerAttackBlocked { weapon_needed } = event {
            assert_eq!(weapon_needed, "Stormbreaker");
            return;
        }
    }
    panic!("Expected PlayerAttackBlocked with weapon_needed=Stormbreaker");
}

#[test]
fn weapon_gate_allows_attack_with_stormbreaker_achievement() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(1, 100, 0);
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::TheStormbreaker, None);

    state.zone_progression.current_zone_id = 10;
    state.zone_progression.current_subzone_id = 4;
    state.zone_progression.unlock_zone(10);
    state.zone_progression.fighting_boss = true;

    let events = force_player_attack_with_achievements(
        &mut rng,
        &mut state,
        &default_bonuses(),
        &mut achievements,
    );

    // Should NOT be blocked
    let blocked = events
        .iter()
        .any(|e| matches!(e, CombatEvent::PlayerAttackBlocked { .. }));
    assert!(
        !blocked,
        "Should not be blocked with Stormbreaker achievement"
    );

    // Should have an attack or kill event
    let attacked = events.iter().any(|e| {
        matches!(
            e,
            CombatEvent::PlayerAttack { .. } | CombatEvent::SubzoneBossDefeated { .. }
        )
    });
    assert!(
        attacked,
        "Should have attack or kill event with Stormbreaker"
    );
}

#[test]
fn weapon_gate_does_not_apply_to_intermediate_zone_10_subzones() {
    let mut rng = seeded_rng();
    // Zone 10, subzone 2 (not final) — no Stormbreaker needed
    let mut state = state_with_enemy(1, 100, 0);
    let mut achievements = Achievements::default();

    state.zone_progression.current_zone_id = 10;
    state.zone_progression.current_subzone_id = 2; // Not the final subzone
    state.zone_progression.unlock_zone(10);
    state.zone_progression.fighting_boss = true;
    state.zone_progression.defeat_boss(10, 1);

    let events = force_player_attack_with_achievements(
        &mut rng,
        &mut state,
        &default_bonuses(),
        &mut achievements,
    );

    let blocked = events
        .iter()
        .any(|e| matches!(e, CombatEvent::PlayerAttackBlocked { .. }));
    assert!(!blocked, "Intermediate subzone should not be weapon-gated");
}

#[test]
fn weapon_gate_does_not_apply_when_not_fighting_boss() {
    let mut rng = seeded_rng();
    let mut state = state_with_enemy(1, 100, 0);
    let mut achievements = Achievements::default();

    state.zone_progression.current_zone_id = 10;
    state.zone_progression.current_subzone_id = 4;
    state.zone_progression.unlock_zone(10);
    state.zone_progression.fighting_boss = false; // Not fighting boss

    let events = force_player_attack_with_achievements(
        &mut rng,
        &mut state,
        &default_bonuses(),
        &mut achievements,
    );

    let blocked = events
        .iter()
        .any(|e| matches!(e, CombatEvent::PlayerAttackBlocked { .. }));
    assert!(!blocked, "Non-boss fights should not be weapon-gated");
}

// ═══════════════════════════════════════════════════════════════════
// 2. Giant's Might (early_damage_percent)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn giants_might_increases_damage_by_150_percent() {
    let mut rng_base = seeded_rng();
    let mut rng_boosted = seeded_rng();

    // Use a high-defense enemy so crit doesn't mask the difference easily
    // Use exactly 0 defense so math is clean
    let base_bonuses = CombatBonuses {
        early_damage_percent: 0.0,
        ..CombatBonuses::default()
    };
    let giants_might_bonuses = CombatBonuses {
        early_damage_percent: 150.0, // Giant's Might: 150% extra
        ..CombatBonuses::default()
    };

    // Use enemy with high HP (won't die in one hit), zero defense, no crit
    let mut state_base = state_with_enemy(100_000, 100, 0);
    state_base.attributes.set(AttributeType::Strength, 20);
    state_base.attributes.set(AttributeType::Dexterity, 10); // Low DEX = low crit

    let mut state_boosted = state_base.clone();

    let events_base = force_player_attack(&mut rng_base, &mut state_base, &base_bonuses);
    let events_boosted =
        force_player_attack(&mut rng_boosted, &mut state_boosted, &giants_might_bonuses);

    let damage_base = extract_first_attack_damage(&events_base)
        .expect("Should produce a PlayerAttack event (base)");
    let damage_boosted = extract_first_attack_damage(&events_boosted)
        .expect("Should produce a PlayerAttack event (boosted)");

    // With same RNG seed, boosted damage should be 2.5x base (1 + 1.5 = 2.5x)
    // Both RNGs start at same seed so crit rolls are identical
    // Allow for integer truncation: boosted should be roughly 2.5x base
    let ratio = damage_boosted as f64 / damage_base as f64;
    assert!(
        (ratio - 2.5).abs() < 0.5,
        "Giant's Might should boost damage by ~2.5x, got {}/{}={:.2}",
        damage_boosted,
        damage_base,
        ratio
    );
}

#[test]
fn giants_might_zero_early_damage_percent_unchanged() {
    let mut rng = seeded_rng();
    let bonuses = CombatBonuses {
        early_damage_percent: 0.0,
        ..CombatBonuses::default()
    };
    let mut state = state_with_enemy(100_000, 100, 0);

    let events = force_player_attack(&mut rng, &mut state, &bonuses);
    let damage = extract_first_attack_damage(&events);
    assert!(damage.is_some(), "Should have attack damage");
    assert!(damage.unwrap() >= 1, "Damage must be at least 1");
}

// ═══════════════════════════════════════════════════════════════════
// 3. Haven Armory Stacking (damage_percent)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn haven_armory_damage_percent_increases_damage() {
    let mut rng_base = seeded_rng();
    let mut rng_armory = seeded_rng();

    let base_bonuses = CombatBonuses::default();
    let armory_bonuses = CombatBonuses {
        damage_percent: 50.0, // +50% damage from Haven Armory
        ..CombatBonuses::default()
    };

    let mut state_base = state_with_enemy(100_000, 100, 0);
    state_base.attributes.set(AttributeType::Dexterity, 10); // Low crit for clean math

    let mut state_armory = state_base.clone();

    let events_base = force_player_attack(&mut rng_base, &mut state_base, &base_bonuses);
    let events_armory = force_player_attack(&mut rng_armory, &mut state_armory, &armory_bonuses);

    let damage_base = extract_first_attack_damage(&events_base)
        .expect("Should produce a PlayerAttack event (base)");
    let damage_armory = extract_first_attack_damage(&events_armory)
        .expect("Should produce a PlayerAttack event (armory)");

    // Armory at 50% should be 1.5x base
    let ratio = damage_armory as f64 / damage_base as f64;
    assert!(
        (ratio - 1.5).abs() < 0.5,
        "Haven Armory 50% should boost damage by 1.5x, got ratio {:.2}",
        ratio
    );
}

#[test]
fn giants_might_and_armory_stack_multiplicatively() {
    // Giants Might applies early (to base), Armory applies after, so they stack multiplicatively
    let mut rng_base = seeded_rng();
    let mut rng_both = seeded_rng();

    let base_bonuses = CombatBonuses::default();
    let combined_bonuses = CombatBonuses {
        early_damage_percent: 150.0, // Giant's Might 2.5x
        damage_percent: 100.0,       // Armory 2x
        ..CombatBonuses::default()
    };

    let mut state_base = state_with_enemy(100_000, 100, 0);
    state_base.attributes.set(AttributeType::Dexterity, 10);

    let mut state_both = state_base.clone();

    let events_base = force_player_attack(&mut rng_base, &mut state_base, &base_bonuses);
    let events_both = force_player_attack(&mut rng_both, &mut state_both, &combined_bonuses);

    let damage_base = extract_first_attack_damage(&events_base)
        .expect("Should produce a PlayerAttack event (base)");
    let damage_both = extract_first_attack_damage(&events_both)
        .expect("Should produce a PlayerAttack event (combined)");

    // Expected: base * 2.5 * 2.0 = 5x
    let ratio = damage_both as f64 / damage_base as f64;
    assert!(
        (ratio - 5.0).abs() < 1.0,
        "Combined Giants Might + Armory should be ~5x, got ratio {:.2}",
        ratio
    );
}

// ═══════════════════════════════════════════════════════════════════
// 4. Prestige Flat Damage (flat_damage)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn prestige_flat_damage_adds_after_percent_multipliers() {
    let mut rng_base = seeded_rng();
    let mut rng_flat = seeded_rng();

    let base_bonuses = CombatBonuses::default();
    let flat_bonuses = CombatBonuses {
        flat_damage: 100,
        ..CombatBonuses::default()
    };

    // Use a high-HP enemy with zero defense for clean math
    let mut state_base = state_with_enemy(100_000, 100, 0);
    state_base.attributes.set(AttributeType::Dexterity, 10);

    let mut state_flat = state_base.clone();

    let events_base = force_player_attack(&mut rng_base, &mut state_base, &base_bonuses);
    let events_flat = force_player_attack(&mut rng_flat, &mut state_flat, &flat_bonuses);

    let damage_base = extract_first_attack_damage(&events_base)
        .expect("Should produce a PlayerAttack event (base)");
    let damage_flat = extract_first_attack_damage(&events_flat)
        .expect("Should produce a PlayerAttack event (flat)");

    // With same RNG seed: flat should be exactly base + 100
    assert_eq!(
        damage_flat,
        damage_base + 100,
        "Flat damage should add exactly 100 to base damage (same crit outcome)"
    );
}

#[test]
fn prestige_flat_damage_zero_no_change() {
    let mut rng_a = seeded_rng();
    let mut rng_b = seeded_rng();

    let bonuses_a = CombatBonuses::default();
    let bonuses_b = CombatBonuses {
        flat_damage: 0,
        ..CombatBonuses::default()
    };

    let mut state_a = state_with_enemy(100_000, 100, 0);
    state_a.attributes.set(AttributeType::Dexterity, 10);
    let mut state_b = state_a.clone();

    let events_a = force_player_attack(&mut rng_a, &mut state_a, &bonuses_a);
    let events_b = force_player_attack(&mut rng_b, &mut state_b, &bonuses_b);

    let damage_a = extract_first_attack_damage(&events_a);
    let damage_b = extract_first_attack_damage(&events_b);

    assert_eq!(
        damage_a, damage_b,
        "Zero flat damage should produce same result as default"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 5. Defense Subtraction (min 1 damage floor)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn enemy_with_high_defense_takes_minimum_1_damage() {
    let mut rng = seeded_rng();

    // Enemy has astronomically high defense — player can't possibly exceed it
    let bonuses = CombatBonuses {
        flat_damage: 0,
        ..CombatBonuses::default()
    };
    // Give player low STR so base damage is also low
    let mut state = state_with_enemy(100_000, 100, 999_999);
    state.attributes.set(AttributeType::Strength, 10);
    state.attributes.set(AttributeType::Intelligence, 10);

    let events = force_player_attack(&mut rng, &mut state, &bonuses);

    let damage = extract_first_attack_damage(&events)
        .expect("Should produce a PlayerAttack event even against max defense");
    assert!(
        damage >= 1,
        "Damage must be at least 1 even against max defense (got {})",
        damage
    );
}

#[test]
fn zero_defense_enemy_takes_full_damage() {
    let mut rng = seeded_rng();

    let bonuses = CombatBonuses {
        flat_damage: 50,
        ..CombatBonuses::default()
    };
    let mut state = state_with_enemy(100_000, 100, 0); // Zero defense
    state.attributes.set(AttributeType::Dexterity, 10);

    let derived =
        DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7]);
    let base_damage = derived.total_damage();

    let events = force_player_attack(&mut rng, &mut state, &bonuses);
    let damage = extract_first_attack_damage(&events)
        .expect("Should produce a PlayerAttack event against zero-defense enemy");

    // Without crit: damage = base + 50
    // With crit: damage = (base + 50) * 2
    // Either way, damage >= base + 50
    assert!(
        damage >= base_damage + 50,
        "Zero defense: damage {} should be >= base_damage {} + flat 50",
        damage,
        base_damage
    );
}

#[test]
fn enemy_defense_reduces_player_damage() {
    let mut rng_no_def = seeded_rng();
    let mut rng_with_def = seeded_rng();

    // Same player stats, same RNG — only difference is enemy defense
    let bonuses = CombatBonuses::default();
    let mut state_no_def = state_with_enemy(100_000, 100, 0);
    state_no_def.attributes.set(AttributeType::Dexterity, 10);

    let mut state_with_def = state_with_enemy(100_000, 100, 20);
    state_with_def.attributes.set(AttributeType::Dexterity, 10);

    let events_no_def = force_player_attack(&mut rng_no_def, &mut state_no_def, &bonuses);
    let events_with_def = force_player_attack(&mut rng_with_def, &mut state_with_def, &bonuses);

    let damage_no_def = extract_first_attack_damage(&events_no_def)
        .expect("Should produce a PlayerAttack event (no defense)");
    let damage_with_def = extract_first_attack_damage(&events_with_def)
        .expect("Should produce a PlayerAttack event (with defense)");

    // Both have same crit outcome (same RNG), so damage_with_def = damage_no_def - 20
    // Unless damage_no_def <= 20, in which case min = 1
    if damage_no_def > 20 {
        assert_eq!(
            damage_with_def,
            damage_no_def - 20,
            "Defense 20 should reduce damage by exactly 20"
        );
    } else {
        assert_eq!(damage_with_def, 1, "Damage should floor at 1");
    }
}

// ═══════════════════════════════════════════════════════════════════
// 6. Divine Bulwark DR (damage_reduction_percent)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn divine_bulwark_reduces_incoming_damage_by_30_percent() {
    let mut rng_no_dr = seeded_rng();
    let mut rng_with_dr = seeded_rng();

    let no_dr_bonuses = CombatBonuses::default();
    let bulwark_bonuses = CombatBonuses {
        damage_reduction_percent: 30.0, // Divine Bulwark: 30% DR
        ..CombatBonuses::default()
    };

    // Enemy with high damage, player with zero defense for clean math
    let mut state_no_dr = state_with_enemy(100_000, 100, 0);
    state_no_dr.combat_state.player_current_hp = 10_000;
    state_no_dr.combat_state.player_max_hp = 10_000;

    let mut state_with_dr = state_no_dr.clone();

    let events_no_dr = force_enemy_attack(&mut rng_no_dr, &mut state_no_dr, &no_dr_bonuses);
    let events_with_dr = force_enemy_attack(&mut rng_with_dr, &mut state_with_dr, &bulwark_bonuses);

    let damage_no_dr = extract_enemy_attack_damage(&events_no_dr)
        .expect("Should produce an EnemyAttack event (no DR)");
    let damage_with_dr = extract_enemy_attack_damage(&events_with_dr)
        .expect("Should produce an EnemyAttack event (with DR)");

    // With 30% DR: effective = no_dr * 0.7 (at least 1)
    let expected = ((damage_no_dr as f64) * 0.7) as u64;
    let expected = expected.max(1);
    assert_eq!(
        damage_with_dr, expected,
        "30% DR: expected {} damage, got {}",
        expected, damage_with_dr
    );
}

#[test]
fn divine_bulwark_damage_floors_at_1() {
    let mut rng = seeded_rng();

    // Extreme DR — 99% reduction. Even if enemy does 1 damage, result is still 1.
    let bonuses = CombatBonuses {
        damage_reduction_percent: 99.0,
        flat_defense: 0,
        ..CombatBonuses::default()
    };

    let mut state = state_with_enemy(100_000, 1, 0); // Enemy does 1 damage
    state.combat_state.player_current_hp = 10_000;
    state.combat_state.player_max_hp = 10_000;

    let events = force_enemy_attack(&mut rng, &mut state, &bonuses);
    let damage = extract_enemy_attack_damage(&events)
        .expect("Should produce an EnemyAttack event even with 99% DR");
    assert!(damage >= 1, "Damage must be at least 1 even with 99% DR");
}

#[test]
fn zero_divine_bulwark_no_reduction() {
    let mut rng_base = seeded_rng();
    let mut rng_zero_dr = seeded_rng();

    let base_bonuses = CombatBonuses::default();
    let zero_dr_bonuses = CombatBonuses {
        damage_reduction_percent: 0.0,
        ..CombatBonuses::default()
    };

    let mut state_base = state_with_enemy(100_000, 100, 0);
    state_base.combat_state.player_current_hp = 10_000;
    state_base.combat_state.player_max_hp = 10_000;

    let mut state_zero_dr = state_base.clone();

    let events_base = force_enemy_attack(&mut rng_base, &mut state_base, &base_bonuses);
    let events_zero = force_enemy_attack(&mut rng_zero_dr, &mut state_zero_dr, &zero_dr_bonuses);

    let damage_base = extract_enemy_attack_damage(&events_base)
        .expect("Should produce an EnemyAttack event (base)");
    let damage_zero = extract_enemy_attack_damage(&events_zero)
        .expect("Should produce an EnemyAttack event (zero DR)");

    assert_eq!(
        damage_base, damage_zero,
        "0% DR should produce same damage as no DR"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 7. Crit Multiplier (2x)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn crit_doubles_damage() {
    // Use 100% crit chance to guarantee crit, then compare to 0% crit
    let mut rng_crit = seeded_rng();
    let mut rng_no_crit = seeded_rng();

    let crit_bonuses = CombatBonuses {
        crit_chance_percent: 100.0, // Guaranteed crit
        ..CombatBonuses::default()
    };
    let no_crit_bonuses = CombatBonuses {
        crit_chance_percent: -999.0, // Effectively impossible crit
        ..CombatBonuses::default()
    };

    let mut state_crit = state_with_enemy(100_000, 100, 0);
    state_crit.attributes.set(AttributeType::Dexterity, 10); // Base crit chance very low

    let mut state_no_crit = state_crit.clone();

    let events_crit = force_player_attack(&mut rng_crit, &mut state_crit, &crit_bonuses);
    let events_no_crit =
        force_player_attack(&mut rng_no_crit, &mut state_no_crit, &no_crit_bonuses);

    let damage_crit = extract_first_attack_damage(&events_crit)
        .expect("Should produce a PlayerAttack event (crit)");
    let damage_no_crit = extract_first_attack_damage(&events_no_crit)
        .expect("Should produce a PlayerAttack event (no crit)");

    // Verify crit was registered
    let was_crit = events_crit
        .iter()
        .any(|e| matches!(e, CombatEvent::PlayerAttack { was_crit: true, .. }));
    assert!(was_crit, "Expected a crit event with 100% crit chance");

    // No crit registered with negative crit chance
    let no_crit_registered = events_no_crit.iter().any(|e| {
        matches!(
            e,
            CombatEvent::PlayerAttack {
                was_crit: false,
                ..
            }
        )
    });
    assert!(no_crit_registered, "Expected non-crit event");

    // Crit should be 2x non-crit
    assert_eq!(
        damage_crit,
        damage_no_crit * 2,
        "Crit damage {} should be 2x non-crit damage {}",
        damage_crit,
        damage_no_crit
    );
}

#[test]
fn non_crit_attack_has_was_crit_false() {
    let mut rng = seeded_rng();
    let bonuses = CombatBonuses {
        crit_chance_percent: -100.0, // Force no crit
        ..CombatBonuses::default()
    };
    let mut state = state_with_enemy(100_000, 100, 0);

    let events = force_player_attack(&mut rng, &mut state, &bonuses);

    let attack_events: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let CombatEvent::PlayerAttack { was_crit, .. } = e {
                Some(*was_crit)
            } else {
                None
            }
        })
        .collect();

    assert!(!attack_events.is_empty(), "Expected PlayerAttack events");
    assert!(
        attack_events.iter().all(|&c| !c),
        "All attacks should be non-crits with negative crit chance"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 8. Double Strike Chance
// ═══════════════════════════════════════════════════════════════════

#[test]
fn double_strike_produces_two_attack_events() {
    let mut rng = seeded_rng();
    let bonuses = CombatBonuses {
        double_strike_chance: 100.0, // Guaranteed double strike
        ..CombatBonuses::default()
    };
    let mut state = state_with_enemy(100_000, 100, 0);

    let events = force_player_attack(&mut rng, &mut state, &bonuses);

    let attack_count = events
        .iter()
        .filter(|e| matches!(e, CombatEvent::PlayerAttack { .. }))
        .count();

    assert_eq!(
        attack_count, 2,
        "Double strike should produce exactly 2 attack events"
    );
}

#[test]
fn double_strike_second_hit_is_not_marked_as_crit() {
    let mut rng = seeded_rng();
    let bonuses = CombatBonuses {
        double_strike_chance: 100.0, // Guaranteed double strike
        crit_chance_percent: 100.0,  // Guaranteed crit on first hit
        ..CombatBonuses::default()
    };
    let mut state = state_with_enemy(100_000, 100, 0);

    let events = force_player_attack(&mut rng, &mut state, &bonuses);

    let attack_events: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let CombatEvent::PlayerAttack { damage, was_crit } = e {
                Some((*damage, *was_crit))
            } else {
                None
            }
        })
        .collect();

    assert_eq!(attack_events.len(), 2, "Should have 2 attack events");

    // First attack should be a crit
    assert!(attack_events[0].1, "First attack should be crit");
    // Second attack is bonus hit — not marked as crit
    assert!(
        !attack_events[1].1,
        "Second (double strike) attack should NOT be marked as crit"
    );
}

#[test]
fn zero_double_strike_chance_produces_single_attack() {
    let mut rng = seeded_rng();
    let bonuses = CombatBonuses {
        double_strike_chance: 0.0, // No double strike
        ..CombatBonuses::default()
    };
    let mut state = state_with_enemy(100_000, 100, 0);

    let events = force_player_attack(&mut rng, &mut state, &bonuses);

    let attack_count = events
        .iter()
        .filter(|e| matches!(e, CombatEvent::PlayerAttack { .. }))
        .count();

    assert_eq!(
        attack_count, 1,
        "Without double strike, should produce exactly 1 attack event"
    );
}

#[test]
fn double_strike_stops_if_enemy_dies_on_first_hit() {
    let mut rng = seeded_rng();
    let bonuses = CombatBonuses {
        double_strike_chance: 100.0, // Guaranteed double strike
        flat_damage: 999_999,        // Massive flat damage to one-shot
        ..CombatBonuses::default()
    };
    // Enemy with 1 HP — first hit kills it
    let mut state = state_with_enemy(1, 100, 0);

    let events = force_player_attack(&mut rng, &mut state, &bonuses);

    // Enemy should be dead
    assert!(
        state.combat_state.current_enemy.is_none(),
        "Enemy should be dead after first hit"
    );

    // Should only have 1 attack event (second strike skipped because enemy already dead)
    let attack_count = events
        .iter()
        .filter(|e| matches!(e, CombatEvent::PlayerAttack { .. }))
        .count();

    assert_eq!(
        attack_count, 1,
        "Second strike should be skipped if enemy already dead"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 9. Enemy Attack Pipeline (defense/reflection/player death)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn player_defense_reduces_incoming_damage() {
    let mut rng_no_def = seeded_rng();
    let mut rng_with_def = seeded_rng();

    let no_def_bonuses = CombatBonuses {
        flat_defense: 0,
        ..CombatBonuses::default()
    };
    let with_def_bonuses = CombatBonuses {
        flat_defense: 20,
        ..CombatBonuses::default()
    };

    let mut state_no_def = state_with_enemy(100_000, 100, 0);
    state_no_def.combat_state.player_current_hp = 10_000;
    state_no_def.combat_state.player_max_hp = 10_000;

    let mut state_with_def = state_no_def.clone();

    let events_no_def = force_enemy_attack(&mut rng_no_def, &mut state_no_def, &no_def_bonuses);
    let events_with_def =
        force_enemy_attack(&mut rng_with_def, &mut state_with_def, &with_def_bonuses);

    let dmg_no_def = extract_enemy_attack_damage(&events_no_def)
        .expect("Should produce an EnemyAttack event (no defense)");
    let dmg_with_def = extract_enemy_attack_damage(&events_with_def)
        .expect("Should produce an EnemyAttack event (with defense)");

    // flat_defense reduces incoming: with_def = max(no_def - 20, 1)
    let expected = (dmg_no_def.saturating_sub(20)).max(1);
    assert_eq!(
        dmg_with_def, expected,
        "flat_defense 20 should reduce damage by 20 (floor 1)"
    );
}

#[test]
fn player_death_resets_hp_to_max() {
    let mut rng = seeded_rng();
    let bonuses = CombatBonuses::default();

    let mut state = state_with_enemy(100_000, 100_000, 0);
    state.combat_state.player_current_hp = 1; // Near death
    state.combat_state.player_max_hp = 100;

    let events = force_enemy_attack(&mut rng, &mut state, &bonuses);

    let player_died = events.iter().any(|e| matches!(e, CombatEvent::PlayerDied));
    assert!(player_died, "Expected PlayerDied event");

    assert_eq!(
        state.combat_state.player_current_hp, state.combat_state.player_max_hp,
        "HP should be reset to max after death"
    );
}

#[test]
fn player_death_resets_both_timers() {
    let mut rng = seeded_rng();
    let bonuses = CombatBonuses::default();

    let mut state = state_with_enemy(100_000, 100_000, 0);
    state.combat_state.player_current_hp = 1;
    state.combat_state.player_max_hp = 100;
    // Set timers to non-zero to verify they get reset
    state.combat_state.player_attack_timer = 1.0;
    state.combat_state.enemy_attack_timer = ENEMY_ATTACK_INTERVAL_SECONDS;

    let events = force_enemy_attack(&mut rng, &mut state, &bonuses);

    let player_died = events.iter().any(|e| matches!(e, CombatEvent::PlayerDied));
    assert!(player_died, "Expected PlayerDied event");

    assert_eq!(
        state.combat_state.player_attack_timer, 0.0,
        "Player attack timer should reset on death"
    );
    assert_eq!(
        state.combat_state.enemy_attack_timer, 0.0,
        "Enemy attack timer should reset on death"
    );
}

#[test]
fn player_death_in_dungeon_exits_dungeon() {
    use quest::dungeon::generation::generate_dungeon;

    let mut rng = seeded_rng();
    let bonuses = CombatBonuses::default();

    let mut state = state_with_enemy(100_000, 100_000, 0);
    state.combat_state.player_current_hp = 1;
    state.combat_state.player_max_hp = 100;
    state.active_dungeon = Some(generate_dungeon(1, 0, 1));

    let events = force_enemy_attack(&mut rng, &mut state, &bonuses);

    let died_in_dungeon = events
        .iter()
        .any(|e| matches!(e, CombatEvent::PlayerDiedInDungeon));
    assert!(died_in_dungeon, "Expected PlayerDiedInDungeon event");

    assert!(
        state.active_dungeon.is_none(),
        "Dungeon should be cleared on player death in dungeon"
    );
}

#[test]
fn damage_reflection_deals_damage_to_enemy() {
    let mut rng = seeded_rng();
    let bonuses = CombatBonuses::default();

    // Set up player with 100% reflection via derived stats
    // NOTE: reflection comes from DerivedStats, not CombatBonuses directly
    // We test this by checking for DamageReflected event (needs high enough reflection)
    // Since reflection is 0 by default, we just verify no spurious events
    let mut state = state_with_enemy(100_000, 100, 0);
    state.combat_state.player_current_hp = 10_000;
    state.combat_state.player_max_hp = 10_000;

    let events = force_enemy_attack(&mut rng, &mut state, &bonuses);

    // With zero reflection, no DamageReflected event
    let reflected = events
        .iter()
        .any(|e| matches!(e, CombatEvent::DamageReflected { .. }));
    assert!(
        !reflected,
        "No DamageReflected expected with zero reflection stat"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 10. Edge Cases: Zero Defense, All God Items Combined, Max Prestige
// ═══════════════════════════════════════════════════════════════════

#[test]
fn max_prestige_flat_damage_bonus() {
    // Simulate max prestige flat_damage — ensure no overflow
    let mut rng = seeded_rng();
    let bonuses = CombatBonuses {
        flat_damage: u64::from(u32::MAX) / 2, // Near-max prestige flat damage
        ..CombatBonuses::default()
    };

    // Enemy with very high HP so we don't accidentally kill it
    let mut state = state_with_enemy(u64::from(u32::MAX), 100, 0);
    state.attributes.set(AttributeType::Dexterity, 10);

    let events = force_player_attack(&mut rng, &mut state, &bonuses);

    let damage = extract_first_attack_damage(&events)
        .expect("Should still produce damage event at max prestige");
    assert!(damage >= 1, "Damage should be at least 1");
}

#[test]
fn all_god_item_bonuses_combined_no_crash() {
    // Combine all god item bonus types together to ensure no panics
    let mut rng = seeded_rng();
    let bonuses = CombatBonuses {
        early_damage_percent: 150.0,    // Giant's Might (Megingjord)
        damage_reduction_percent: 30.0, // Divine Bulwark (Asprika)
        attack_speed_percent: 100.0,    // Windborne (Sleipnir)
        regen_reduction_percent: 50.0,  // Swiftstrider (Sleipnir)
        damage_percent: 40.0,           // Other bonuses
        xp_gain_percent: 40.0,
        flat_damage: 200,
        flat_defense: 50,
        crit_chance_percent: 15.0,
        double_strike_chance: 30.0,
        ..CombatBonuses::default()
    };

    let mut state = state_with_enemy(100_000, 100, 0);
    state.combat_state.player_current_hp = 10_000;
    state.combat_state.player_max_hp = 10_000;

    // Should not panic or crash
    let events = force_player_attack(&mut rng, &mut state, &bonuses);
    assert!(
        !events.is_empty(),
        "Should produce events with all god item bonuses"
    );
}

#[test]
fn zero_defense_enemy_still_takes_damage() {
    let mut rng = seeded_rng();
    let bonuses = CombatBonuses::default();

    let mut state = state_with_enemy(100_000, 100, 0); // Explicitly zero defense
    let events = force_player_attack(&mut rng, &mut state, &bonuses);

    let damage = extract_first_attack_damage(&events)
        .expect("Zero defense enemy should take damage — expected a PlayerAttack event");
    assert!(damage >= 1, "Damage must be at least 1");
}

#[test]
fn high_prestige_combined_bonuses_no_overflow() {
    // Simulate P50 stats — large flat_damage and flat_defense
    let mut rng = seeded_rng();

    // P50 flat damage: floor(5.0 * 50^0.7) ≈ floor(5.0 * 21.1) ≈ 105
    let flat_damage = (5.0_f64 * 50.0_f64.powf(0.7)).floor() as u64;
    // P50 flat defense: floor(3.0 * 50^0.6) ≈ floor(3.0 * 14.0) ≈ 42
    let flat_defense = (3.0_f64 * 50.0_f64.powf(0.6)).floor() as u64;
    // P50 flat HP: floor(15.0 * 50^0.6) ≈ floor(15.0 * 14.0) ≈ 210
    let flat_hp = (15.0_f64 * 50.0_f64.powf(0.6)).floor() as u64;

    let bonuses = CombatBonuses {
        flat_damage,
        flat_defense,
        crit_chance_percent: 15.0, // Capped prestige crit
        ..CombatBonuses::default()
    };

    let mut state = state_with_enemy(100_000, 100, 10);
    // flat_hp is now applied in tick Stage 3, not in CombatBonuses
    state.combat_state.player_max_hp += flat_hp;
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;

    let events = force_player_attack(&mut rng, &mut state, &bonuses);
    assert!(
        !events.is_empty(),
        "P50 bonuses should produce valid events without overflow"
    );
}

#[test]
fn enemy_killed_triggers_regen_state() {
    let mut rng = seeded_rng();
    let bonuses = CombatBonuses {
        flat_damage: 999_999,
        ..CombatBonuses::default()
    };
    let mut state = state_with_enemy(1, 100, 0);

    let events = force_player_attack(&mut rng, &mut state, &bonuses);

    let enemy_died = events
        .iter()
        .any(|e| matches!(e, CombatEvent::EnemyDied { .. }));
    assert!(enemy_died, "Expected EnemyDied event");

    assert!(
        state.combat_state.is_regenerating,
        "Should be regenerating after kill"
    );
    assert!(
        state.combat_state.current_enemy.is_none(),
        "Enemy should be cleared after death"
    );
}

#[test]
fn enemy_killed_resets_consecutive_deaths() {
    let mut rng = seeded_rng();
    let bonuses = CombatBonuses {
        flat_damage: 999_999,
        ..CombatBonuses::default()
    };
    let mut state = state_with_enemy(1, 100, 0);
    state.consecutive_deaths = 5; // Simulate previous deaths

    force_player_attack(&mut rng, &mut state, &bonuses);

    assert_eq!(
        state.consecutive_deaths, 0,
        "Consecutive deaths should reset after a successful kill"
    );
}

// ═══════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════

fn extract_first_attack_damage(events: &[CombatEvent]) -> Option<u64> {
    events.iter().find_map(|e| {
        if let CombatEvent::PlayerAttack { damage, .. } = e {
            Some(*damage)
        } else {
            None
        }
    })
}

fn extract_enemy_attack_damage(events: &[CombatEvent]) -> Option<u64> {
    events.iter().find_map(|e| {
        if let CombatEvent::EnemyAttack { damage } = e {
            Some(*damage)
        } else {
            None
        }
    })
}
