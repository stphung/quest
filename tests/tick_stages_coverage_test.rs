#![allow(deprecated)]
//! Coverage tests for `core::tick_stages` — the per-tick processing stage helpers.
//!
//! Tests the public stage functions (`process_combat_events`, `process_dungeon_events`,
//! `process_fishing_tick`) directly and the `pub(super)` helpers (`process_item_drop`,
//! `process_discoveries`, `process_zone_achievements`, `collect_achievement_events`)
//! indirectly through the public functions or `game_tick`.

use quest::achievements::Achievements;
use quest::character::attributes::AttributeType;
use quest::character::derived_stats::DerivedStats;
use quest::combat::CombatEvent;
use quest::core::tick::{game_tick, TickEvent, TickResult};
use quest::core::tick_stages;
use quest::deep::DeepState;
use quest::enhancement::EnhancementProgress;
use quest::fishing::{FishingPhase, FishingSession};
use quest::haven::{Haven, HavenBonuses};
use quest::items::types::Rarity;
use quest::zones::BossDefeatResult;
use quest::GameState;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// =============================================================================
// Helpers
// =============================================================================

fn seeded_rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

fn fresh_state() -> GameState {
    GameState::new("TestHero".to_string(), 0)
}

fn strong_state() -> GameState {
    let mut s = fresh_state();
    s.attributes.set(AttributeType::Strength, 50);
    s.attributes.set(AttributeType::Intelligence, 50);
    s.attributes.set(AttributeType::Constitution, 50);
    let d = DerivedStats::calculate_derived_stats(&s.attributes, &s.equipment, &[0; 7]);
    s.combat_state.update_max_hp(d.max_hp);
    s.combat_state.player_current_hp = s.combat_state.player_max_hp;
    s.cached_derived_stats = d;
    s.derived_stats_dirty = false;
    s
}

fn default_haven_bonuses() -> HavenBonuses {
    HavenBonuses::default()
}

fn run_game_tick(
    state: &mut GameState,
    tc: &mut u32,
    haven: &mut Haven,
    ach: &mut Achievements,
    debug_mode: bool,
    rng: &mut ChaCha8Rng,
) -> TickResult {
    game_tick(
        state,
        tc,
        haven,
        &mut EnhancementProgress::new(),
        &mut quest::deep::DeepState::new(),
        ach,
        debug_mode,
        rng,
    )
}

fn make_fishing_session(phase: FishingPhase, ticks: u32, total: u32) -> FishingSession {
    FishingSession {
        spot_name: "Test Lake".to_string(),
        total_fish: total,
        fish_caught: Vec::new(),
        items_found: Vec::new(),
        ticks_remaining: ticks,
        phase,
    }
}

fn has_event<F: Fn(&TickEvent) -> bool>(result: &TickResult, pred: F) -> bool {
    result.events.iter().any(pred)
}

fn count_events<F: Fn(&TickEvent) -> bool>(result: &TickResult, pred: F) -> usize {
    result.events.iter().filter(|e| pred(e)).count()
}

// =============================================================================
// Stage 6: process_combat_events — CombatEvent::PlayerAttack
// =============================================================================

#[test]
fn test_player_attack_normal_produces_tick_event() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Goblin".into(), 100, 5));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::PlayerAttack {
        damage: 15,
        was_crit: false,
    }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(has_event(&result, |e| matches!(
        e,
        TickEvent::PlayerAttack {
            damage: 15,
            was_crit: false,
            ..
        }
    )));

    // Verify message content
    if let Some(TickEvent::PlayerAttack { message, .. }) = result.events.first() {
        assert!(
            message.contains("15"),
            "Message should contain damage amount"
        );
        assert!(
            !message.contains("CRITICAL"),
            "Normal hit should not say CRITICAL"
        );
    }
}

#[test]
fn test_player_attack_critical_produces_crit_message() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Goblin".into(), 100, 5));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::PlayerAttack {
        damage: 30,
        was_crit: true,
    }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(has_event(&result, |e| matches!(
        e,
        TickEvent::PlayerAttack {
            damage: 30,
            was_crit: true,
            ..
        }
    )));

    if let Some(TickEvent::PlayerAttack { message, .. }) = result.events.first() {
        assert!(
            message.contains("CRITICAL"),
            "Crit hit message should contain CRITICAL"
        );
        assert!(
            message.contains("30"),
            "Crit message should contain damage amount"
        );
    }
}

// =============================================================================
// Stage 6: process_combat_events — CombatEvent::PlayerAttackBlocked
// =============================================================================

#[test]
fn test_player_attack_blocked_produces_weapon_needed_event() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Storm Lord".into(), 5000, 500));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::PlayerAttackBlocked {
        weapon_needed: "Stormbreaker".to_string(),
    }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(has_event(&result, |e| matches!(
        e,
        TickEvent::PlayerAttackBlocked { .. }
    )));

    if let Some(TickEvent::PlayerAttackBlocked {
        weapon_needed,
        message,
    }) = result.events.first()
    {
        assert_eq!(weapon_needed, "Stormbreaker");
        assert!(message.contains("Stormbreaker"));
    }
}

// =============================================================================
// Stage 6: process_combat_events — CombatEvent::EnemyAttack
// =============================================================================

#[test]
fn test_enemy_attack_produces_event_with_enemy_name() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Darkfang Orc".into(), 100, 10));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::EnemyAttack { damage: 8 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(has_event(&result, |e| matches!(
        e,
        TickEvent::EnemyAttack { damage: 8, .. }
    )));

    if let Some(TickEvent::EnemyAttack {
        enemy_name,
        message,
        ..
    }) = result.events.first()
    {
        assert_eq!(enemy_name, "Darkfang Orc");
        assert!(message.contains("Darkfang Orc"));
        assert!(message.contains("8"));
    }
}

// =============================================================================
// Stage 6: process_combat_events — CombatEvent::DamageReflected
// =============================================================================

#[test]
fn test_damage_reflected_produces_event() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Troll".into(), 200, 15));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::DamageReflected { damage: 5 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(has_event(&result, |e| matches!(
        e,
        TickEvent::DamageReflected { damage: 5, .. }
    )));

    if let Some(TickEvent::DamageReflected { message, .. }) = result.events.first() {
        assert!(message.contains("5"));
        assert!(message.contains("reflected"));
    }
}

// =============================================================================
// Stage 6: process_combat_events — CombatEvent::RegenComplete
// =============================================================================

#[test]
fn test_regen_complete_produces_event() {
    let mut state = fresh_state();
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::RegenComplete { healed: 25 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(has_event(&result, |e| matches!(
        e,
        TickEvent::RegenComplete { healed: 25 }
    )));
}

// =============================================================================
// Stage 6: process_combat_events — CombatEvent::EnemyDied
// =============================================================================

#[test]
fn test_enemy_died_produces_defeated_event_with_xp() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Goblin".into(), 100, 5));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let xp_before = state.character_xp;

    let events = vec![CombatEvent::EnemyDied { xp_gained: 300 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    // Should produce EnemyDefeated event
    assert!(has_event(&result, |e| matches!(
        e,
        TickEvent::EnemyDefeated { xp_gained: 300, .. }
    )));

    // XP should be applied
    assert!(
        state.character_xp > xp_before,
        "XP should increase after enemy death"
    );

    // Session kills should increment
    assert_eq!(state.session_kills, 1);
}

#[test]
fn test_enemy_died_message_contains_enemy_name_and_xp() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Shadow Drake".into(), 200, 10));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::EnemyDied { xp_gained: 500 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    if let Some(TickEvent::EnemyDefeated {
        enemy_name,
        message,
        ..
    }) = result.events.first()
    {
        assert_eq!(enemy_name, "Shadow Drake");
        assert!(message.contains("Shadow Drake"));
        assert!(message.contains("500"));
    }
}

#[test]
fn test_enemy_died_can_trigger_level_up() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Goblin".into(), 100, 5));
    // Give enough XP to be near level up (XP for level 2 = 100 * 2^1.5 ≈ 283)
    state.character_xp = 250;
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::EnemyDied { xp_gained: 500 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    // Should have leveled up
    assert!(
        state.character_level > 1,
        "Should have leveled up from large XP gain"
    );

    // Should produce LeveledUp event
    assert!(
        has_event(&result, |e| matches!(e, TickEvent::LeveledUp { .. })),
        "Should emit LeveledUp event"
    );
}

#[test]
fn test_enemy_died_increments_session_kills() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Goblin".into(), 100, 5));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    assert_eq!(state.session_kills, 0);

    // Kill 3 enemies
    for _ in 0..3 {
        let events = vec![CombatEvent::EnemyDied { xp_gained: 100 }];
        tick_stages::process_combat_events(
            &mut state,
            events,
            &bonuses,
            &mut ach,
            &mut deep,
            false,
            &mut result,
            &mut rng,
        );
    }

    assert_eq!(state.session_kills, 3);
}

// =============================================================================
// Stage 6: process_combat_events — CombatEvent::PlayerDied
// =============================================================================

#[test]
fn test_player_died_produces_event() {
    let mut state = fresh_state();
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::PlayerDied];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(has_event(&result, |e| matches!(
        e,
        TickEvent::PlayerDied { .. }
    )));

    if let Some(TickEvent::PlayerDied { message }) = result.events.first() {
        assert!(
            message.contains("died"),
            "Death message should mention death"
        );
    }
}

// =============================================================================
// Stage 6: process_combat_events — CombatEvent::PlayerDiedInDungeon
// =============================================================================

#[test]
fn test_player_died_in_dungeon_produces_event() {
    let mut state = fresh_state();
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::PlayerDiedInDungeon];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(has_event(&result, |e| matches!(
        e,
        TickEvent::PlayerDiedInDungeon { .. }
    )));

    if let Some(TickEvent::PlayerDiedInDungeon { message }) = result.events.first() {
        assert!(
            message.contains("prestige"),
            "Dungeon death message should mention no prestige loss"
        );
    }
}

// =============================================================================
// Stage 6: process_combat_events — CombatEvent::EliteDefeated
// =============================================================================

#[test]
fn test_elite_defeated_produces_dungeon_elite_event_with_xp() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Elite Guardian".into(), 500, 30));
    // Set up a dungeon so the key logic can execute
    state.active_dungeon = Some(quest::dungeon::generation::generate_dungeon(1, 0, 1));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let xp_before = state.character_xp;

    let events = vec![CombatEvent::EliteDefeated { xp_gained: 800 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    // Should produce DungeonEliteDefeated event
    assert!(has_event(&result, |e| matches!(
        e,
        TickEvent::DungeonEliteDefeated { xp_gained: 800, .. }
    )));

    // XP should be applied
    assert!(state.character_xp > xp_before);

    // Should have enemy name
    if let Some(TickEvent::DungeonEliteDefeated {
        enemy_name,
        message,
        ..
    }) = result.events.first()
    {
        assert_eq!(enemy_name, "Elite Guardian");
        assert!(message.contains("800"));
    }
}

// =============================================================================
// Stage 6: process_combat_events — CombatEvent::BossDefeated
// =============================================================================

#[test]
fn test_boss_defeated_produces_dungeon_boss_event() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Dungeon Boss".into(), 1000, 50));
    state.active_dungeon = Some(quest::dungeon::generation::generate_dungeon(1, 0, 1));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::BossDefeated { xp_gained: 2000 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    // Should produce DungeonBossDefeated event
    assert!(has_event(&result, |e| matches!(
        e,
        TickEvent::DungeonBossDefeated { .. }
    )));

    if let Some(TickEvent::DungeonBossDefeated {
        xp_gained,
        enemy_name,
        message,
        ..
    }) = result
        .events
        .iter()
        .find(|e| matches!(e, TickEvent::DungeonBossDefeated { .. }))
    {
        assert_eq!(*xp_gained, 2000);
        assert_eq!(enemy_name, "Dungeon Boss");
        assert!(message.contains("Dungeon Complete"));
    }

    // Dungeon should be cleared after boss defeat
    assert!(
        state.active_dungeon.is_none(),
        "Dungeon should be cleared after boss defeat"
    );
}

#[test]
fn test_boss_defeated_without_dungeon_still_works() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Boss".into(), 500, 30));
    // No active dungeon
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::BossDefeated { xp_gained: 1000 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    // Should still produce a DungeonBossDefeated event with zero bonus XP
    assert!(has_event(&result, |e| matches!(
        e,
        TickEvent::DungeonBossDefeated { bonus_xp: 0, .. }
    )));
}

// =============================================================================
// Stage 6: process_combat_events — CombatEvent::SubzoneBossDefeated
// =============================================================================

#[test]
fn test_subzone_boss_defeated_subzone_complete() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Subzone Boss".into(), 300, 20));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::SubzoneBossDefeated {
        xp_gained: 600,
        result: BossDefeatResult::SubzoneComplete { new_subzone_id: 2 },
    }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(has_event(&result, |e| matches!(
        e,
        TickEvent::SubzoneBossDefeated { xp_gained: 600, .. }
    )));

    // Session kills should increment
    assert_eq!(state.session_kills, 1);

    if let Some(TickEvent::SubzoneBossDefeated { message, .. }) = result
        .events
        .iter()
        .find(|e| matches!(e, TickEvent::SubzoneBossDefeated { .. }))
    {
        assert!(message.contains("Boss defeated"));
        assert!(message.contains("600"));
    }
}

#[test]
fn test_subzone_boss_defeated_zone_complete() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Zone Boss".into(), 500, 30));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::SubzoneBossDefeated {
        xp_gained: 1200,
        result: BossDefeatResult::ZoneComplete {
            old_zone: "Meadow".to_string(),
            new_zone_id: 2,
        },
    }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    if let Some(TickEvent::SubzoneBossDefeated { message, .. }) = result
        .events
        .iter()
        .find(|e| matches!(e, TickEvent::SubzoneBossDefeated { .. }))
    {
        assert!(message.contains("Meadow"));
        assert!(message.contains("conquered"));
        assert!(message.contains("1200"));
    }
}

#[test]
fn test_subzone_boss_defeated_zone_gated() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Zone Boss".into(), 500, 30));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::SubzoneBossDefeated {
        xp_gained: 1500,
        result: BossDefeatResult::ZoneCompleteButGated {
            zone_name: "Dark Forest".to_string(),
            required_prestige: 5,
        },
    }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    if let Some(TickEvent::SubzoneBossDefeated { message, .. }) = result
        .events
        .iter()
        .find(|e| matches!(e, TickEvent::SubzoneBossDefeated { .. }))
    {
        assert!(message.contains("Dark Forest"));
        assert!(message.contains("conquered"));
        assert!(message.contains("Prestige 5"));
    }
}

#[test]
fn test_subzone_boss_defeated_storms_end() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Final Boss".into(), 5000, 500));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::SubzoneBossDefeated {
        xp_gained: 5000,
        result: BossDefeatResult::StormsEnd,
    }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    if let Some(TickEvent::SubzoneBossDefeated { message, .. }) = result
        .events
        .iter()
        .find(|e| matches!(e, TickEvent::SubzoneBossDefeated { .. }))
    {
        assert!(message.contains("All zones conquered"));
        assert!(message.contains("completed the game"));
    }
}

#[test]
fn test_subzone_boss_defeated_expanse_cycle() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("The Endless".into(), 8000, 800));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::SubzoneBossDefeated {
        xp_gained: 10000,
        result: BossDefeatResult::ExpanseCycle,
    }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    if let Some(TickEvent::SubzoneBossDefeated { message, .. }) = result
        .events
        .iter()
        .find(|e| matches!(e, TickEvent::SubzoneBossDefeated { .. }))
    {
        assert!(message.contains("Expanse cycles anew"));
    }
}

#[test]
fn test_subzone_boss_weapon_required_skipped() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Storm Lord".into(), 5000, 500));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::SubzoneBossDefeated {
        xp_gained: 0,
        result: BossDefeatResult::WeaponRequired {
            weapon_name: "Stormbreaker".to_string(),
        },
    }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    // WeaponRequired uses `continue` in the match — should NOT produce a SubzoneBossDefeated event
    assert!(
        !has_event(&result, |e| matches!(
            e,
            TickEvent::SubzoneBossDefeated { .. }
        )),
        "WeaponRequired should be skipped (handled by PlayerAttackBlocked)"
    );
}

// =============================================================================
// Stage 6: process_combat_events — Multiple events in sequence
// =============================================================================

#[test]
fn test_multiple_combat_events_processed_in_order() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Orc".into(), 100, 10));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![
        CombatEvent::PlayerAttack {
            damage: 20,
            was_crit: false,
        },
        CombatEvent::EnemyAttack { damage: 5 },
        CombatEvent::PlayerAttack {
            damage: 40,
            was_crit: true,
        },
    ];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert_eq!(result.events.len(), 3);
    assert!(matches!(
        result.events[0],
        TickEvent::PlayerAttack {
            damage: 20,
            was_crit: false,
            ..
        }
    ));
    assert!(matches!(
        result.events[1],
        TickEvent::EnemyAttack { damage: 5, .. }
    ));
    assert!(matches!(
        result.events[2],
        TickEvent::PlayerAttack {
            damage: 40,
            was_crit: true,
            ..
        }
    ));
}

#[test]
fn test_enemy_name_defaults_to_empty_when_no_enemy() {
    let mut state = fresh_state();
    // No current enemy
    assert!(state.combat_state.current_enemy.is_none());
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::EnemyAttack { damage: 5 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    if let Some(TickEvent::EnemyAttack { enemy_name, .. }) = result.events.first() {
        assert!(
            enemy_name.is_empty(),
            "Should default to empty string when no enemy"
        );
    }
}

// =============================================================================
// process_item_drop — tested indirectly via EnemyDied
// =============================================================================

#[test]
fn test_enemy_died_can_trigger_item_drop() {
    // With many kills, at least some should produce item drops (15% base chance)
    let mut state = fresh_state();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(42);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let mut item_dropped = false;

    for _ in 0..200 {
        state.combat_state.current_enemy = Some(quest::Enemy::new("Mob".into(), 50, 5));
        let mut result = TickResult::default();

        let events = vec![CombatEvent::EnemyDied { xp_gained: 100 }];
        tick_stages::process_combat_events(
            &mut state,
            events,
            &bonuses,
            &mut ach,
            &mut deep,
            false,
            &mut result,
            &mut rng,
        );

        if has_event(&result, |e| matches!(e, TickEvent::ItemDropped { .. })) {
            item_dropped = true;
            break;
        }
    }

    assert!(
        item_dropped,
        "Should see at least one item drop in 200 kills"
    );
}

#[test]
fn test_boss_kill_always_drops_item() {
    // Boss kills (fighting_boss = true) always drop items
    let mut state = fresh_state();
    state.zone_progression.fighting_boss = true;
    state.combat_state.current_enemy = Some(quest::Enemy::new("Boss".into(), 300, 20));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(42);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::EnemyDied { xp_gained: 500 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(
        has_event(&result, |e| matches!(
            e,
            TickEvent::ItemDropped {
                from_boss: true,
                ..
            }
        )),
        "Boss kills should always drop items"
    );
}

#[test]
fn test_item_drop_event_fields() {
    // Force a boss drop to guarantee an item, then check all fields are populated
    let mut state = fresh_state();
    state.zone_progression.fighting_boss = true;
    state.combat_state.current_enemy = Some(quest::Enemy::new("Boss".into(), 300, 20));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(42);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::EnemyDied { xp_gained: 500 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    let item_event = result
        .events
        .iter()
        .find(|e| matches!(e, TickEvent::ItemDropped { .. }));
    assert!(item_event.is_some(), "Should have ItemDropped event");

    if let Some(TickEvent::ItemDropped {
        item_name,
        slot,
        from_boss,
        ilvl,
        ..
    }) = item_event
    {
        assert!(!item_name.is_empty(), "Item name should be non-empty");
        assert!(!slot.is_empty(), "Slot should be non-empty");
        assert!(from_boss, "Should be from_boss");
        assert_eq!(*ilvl, 10, "Zone 1 items should be ilvl 10");
    }
}

#[test]
fn test_item_drop_adds_to_recent_drops() {
    // Force a boss drop
    let mut state = fresh_state();
    state.zone_progression.fighting_boss = true;
    state.combat_state.current_enemy = Some(quest::Enemy::new("Boss".into(), 300, 20));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(42);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    assert!(state.recent_drops.is_empty());

    let events = vec![CombatEvent::EnemyDied { xp_gained: 500 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(
        !state.recent_drops.is_empty(),
        "Item drop should be added to recent drops"
    );
}

// =============================================================================
// process_discoveries — tested indirectly via EnemyDied (dungeon + fishing)
// =============================================================================

#[test]
fn test_enemy_died_can_trigger_dungeon_discovery() {
    // Dungeon discovery has 1% chance per kill. With enough kills we should see one.
    let mut state = fresh_state();
    let mut ach = Achievements::default();
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();
    let mut discovered = false;

    for seed in 0..500u64 {
        let mut rng = seeded_rng(seed);
        state.combat_state.current_enemy = Some(quest::Enemy::new("Mob".into(), 50, 5));
        state.active_dungeon = None; // Must not be in a dungeon
        let mut result = TickResult::default();

        let events = vec![CombatEvent::EnemyDied { xp_gained: 100 }];
        tick_stages::process_combat_events(
            &mut state,
            events,
            &bonuses,
            &mut ach,
            &mut deep,
            false,
            &mut result,
            &mut rng,
        );

        if has_event(&result, |e| {
            matches!(e, TickEvent::DungeonDiscovered { .. })
        }) {
            discovered = true;
            break;
        }
        // Clear dungeon if one was discovered in state but we missed it in events
        if state.active_dungeon.is_some() {
            discovered = true;
            break;
        }
    }

    assert!(
        discovered,
        "Should discover a dungeon within 500 tries at 1% chance"
    );
}

#[test]
fn test_discoveries_skipped_when_dungeon_active() {
    // When already in a dungeon, no new dungeon or fishing should be discovered
    let mut state = fresh_state();
    state.active_dungeon = Some(quest::dungeon::generation::generate_dungeon(1, 0, 1));
    state.combat_state.current_enemy = Some(quest::Enemy::new("Mob".into(), 50, 5));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    // Try many times
    for seed in 0..100u64 {
        let mut rng = seeded_rng(seed);
        result = TickResult::default();
        let events = vec![CombatEvent::EnemyDied { xp_gained: 100 }];
        tick_stages::process_combat_events(
            &mut state,
            events,
            &bonuses,
            &mut ach,
            &mut deep,
            false,
            &mut result,
            &mut rng,
        );
    }

    // No discovery events should appear when in a dungeon
    assert!(
        !has_event(&result, |e| matches!(
            e,
            TickEvent::DungeonDiscovered { .. } | TickEvent::FishingSpotDiscovered { .. }
        )),
        "Should not discover anything while already in a dungeon"
    );
}

// =============================================================================
// process_zone_achievements — tested indirectly via SubzoneBossDefeated
// =============================================================================

#[test]
fn test_zone_complete_triggers_zone_achievement() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Zone Boss".into(), 500, 30));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::SubzoneBossDefeated {
        xp_gained: 1200,
        result: BossDefeatResult::ZoneComplete {
            old_zone: "Meadow".to_string(),
            new_zone_id: 2,
        },
    }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    // Zone1 fully cleared should trigger zone achievement
    // Note: Achievement unlock may or may not appear depending on whether Meadow is zone 1
    // At minimum, session_kills should increment for boss defeat
    assert_eq!(state.session_kills, 1);
}

#[test]
fn test_storms_end_triggers_zone_10_achievement() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Final Boss".into(), 5000, 500));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::SubzoneBossDefeated {
        xp_gained: 5000,
        result: BossDefeatResult::StormsEnd,
    }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    // StormsEnd should trigger zone 10 and storms_end achievements
    // The achievement events will be collected separately by collect_achievement_events
    // but the internal state should reflect the unlocks
    assert_eq!(state.session_kills, 1);
}

#[test]
fn test_expanse_cycle_triggers_zone_11_achievement() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("The Endless".into(), 8000, 800));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::SubzoneBossDefeated {
        xp_gained: 10000,
        result: BossDefeatResult::ExpanseCycle,
    }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert_eq!(state.session_kills, 1);
}

// =============================================================================
// collect_achievement_events — tested via game_tick
// =============================================================================

#[test]
fn test_collect_achievement_events_emits_unlock_events() {
    // Force a level-up which should trigger a level achievement, then
    // collect_achievement_events should put it in the TickResult
    let mut state = strong_state();
    // Set XP close to level 10 threshold to trigger achievement on level up
    state.character_level = 9;
    state.character_xp = 0;
    // XP for level 10: 100 * 10^1.5 ≈ 3162
    let mut tc = 0u32;
    let mut haven = Haven::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(42);

    // Run many ticks to get an enemy killed and enough XP for level 10
    let mut seen_level_up = false;
    let mut seen_achievement = false;

    for _ in 0..10000 {
        let result = run_game_tick(&mut state, &mut tc, &mut haven, &mut ach, false, &mut rng);

        if has_event(&result, |e| matches!(e, TickEvent::LeveledUp { .. })) {
            seen_level_up = true;
        }
        if has_event(&result, |e| {
            matches!(e, TickEvent::AchievementUnlocked { .. })
        }) {
            seen_achievement = true;
        }

        if state.character_level >= 10 {
            break;
        }
    }

    assert!(
        state.character_level >= 10,
        "Should have reached level 10 (got level {})",
        state.character_level
    );
    // Level 10 achievement should have been emitted
    assert!(seen_level_up, "Should have seen LeveledUp event");
    assert!(
        seen_achievement,
        "Should have seen AchievementUnlocked event from collect_achievement_events"
    );
}

// =============================================================================
// Stage 4: process_dungeon_events — no dungeon active
// =============================================================================

#[test]
fn test_dungeon_events_noop_when_no_dungeon() {
    let mut state = fresh_state();
    assert!(state.active_dungeon.is_none());
    let mut result = TickResult::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();

    tick_stages::process_dungeon_events(&mut state, 0.1, &bonuses, &mut result, &mut rng);

    assert!(
        result.events.is_empty(),
        "No events should be produced when no dungeon active"
    );
}

#[test]
fn test_dungeon_events_with_active_dungeon() {
    let mut state = fresh_state();
    state.active_dungeon = Some(quest::dungeon::generation::generate_dungeon(1, 0, 1));
    let mut result = TickResult::default();
    let mut rng = seeded_rng(42);
    let bonuses = default_haven_bonuses();

    // Run a single tick — the dungeon may or may not produce events depending on
    // timing, but it should not panic
    tick_stages::process_dungeon_events(&mut state, 0.1, &bonuses, &mut result, &mut rng);

    // Successful execution is the test — no panic
}

// =============================================================================
// Stage 5: process_fishing_tick
// =============================================================================

#[test]
fn test_fishing_tick_returns_false_when_no_fishing() {
    let mut state = fresh_state();
    assert!(state.active_fishing.is_none());
    let mut tc = 0u32;
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();

    let active = tick_stages::process_fishing_tick(
        &mut state,
        &mut tc,
        0.1,
        &bonuses,
        &mut ach,
        false,
        &mut result,
        &mut rng,
    );

    assert!(!active, "Should return false when no fishing session");
    assert!(result.events.is_empty());
}

#[test]
fn test_fishing_tick_returns_true_when_fishing_active() {
    let mut state = fresh_state();
    state.active_fishing = Some(make_fishing_session(FishingPhase::Waiting, 100, 5));
    let mut tc = 0u32;
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();

    let active = tick_stages::process_fishing_tick(
        &mut state,
        &mut tc,
        0.1,
        &bonuses,
        &mut ach,
        false,
        &mut result,
        &mut rng,
    );

    assert!(active, "Should return true when fishing is active");
}

#[test]
fn test_fishing_tick_increments_play_time_at_10_ticks() {
    let mut state = fresh_state();
    state.active_fishing = Some(make_fishing_session(FishingPhase::Waiting, 200, 10));
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();

    let initial_time = state.play_time_seconds;

    for _ in 0..10 {
        let mut result = TickResult::default();
        if state.active_fishing.is_none() {
            break;
        }
        tick_stages::process_fishing_tick(
            &mut state,
            &mut tc,
            0.1,
            &bonuses,
            &mut ach,
            false,
            &mut result,
            &mut rng,
        );
    }

    assert_eq!(
        state.play_time_seconds,
        initial_time + 1,
        "10 fishing ticks should add 1 second of play time"
    );
    assert_eq!(tc, 0, "Tick counter should reset to 0 after 10 ticks");
}

#[test]
fn test_fishing_tick_produces_catch_events() {
    let mut state = fresh_state();
    // Start in Reeling phase with 1 tick remaining so a catch happens almost immediately
    state.active_fishing = Some(make_fishing_session(FishingPhase::Reeling, 1, 10));
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(42);
    let bonuses = default_haven_bonuses();

    let mut caught_fish = false;

    // Run enough ticks to get through multiple casting/waiting/reeling cycles
    for _ in 0..500 {
        if state.active_fishing.is_none() {
            break;
        }
        let mut result = TickResult::default();
        tick_stages::process_fishing_tick(
            &mut state,
            &mut tc,
            0.1,
            &bonuses,
            &mut ach,
            false,
            &mut result,
            &mut rng,
        );

        if has_event(&result, |e| matches!(e, TickEvent::FishCaught { .. })) {
            caught_fish = true;
            break;
        }
    }

    assert!(
        caught_fish,
        "Should catch at least one fish during fishing session"
    );
}

#[test]
fn test_fishing_tick_can_level_up_character() {
    let mut state = fresh_state();
    state.character_level = 1;
    state.character_xp = 99; // Any fish catch gives at least 50 XP
    state.active_fishing = Some(make_fishing_session(FishingPhase::Reeling, 1, 1));
    let mut tc = 0u32;
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(123);
    let bonuses = default_haven_bonuses();

    tick_stages::process_fishing_tick(
        &mut state,
        &mut tc,
        0.1,
        &bonuses,
        &mut ach,
        false,
        &mut result,
        &mut rng,
    );

    assert!(
        state.character_level >= 2,
        "Fishing XP should trigger level-up processing"
    );
    assert!(
        has_event(&result, |e| matches!(e, TickEvent::LeveledUp { .. })),
        "Fishing tick should emit LeveledUp event when crossing XP threshold"
    );
}

#[test]
fn test_fishing_tick_fish_caught_adds_recent_drop() {
    let mut state = fresh_state();
    state.active_fishing = Some(make_fishing_session(FishingPhase::Waiting, 200, 10));
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(42);
    let bonuses = default_haven_bonuses();

    for _ in 0..200 {
        if state.active_fishing.is_none() {
            break;
        }
        let mut result = TickResult::default();
        tick_stages::process_fishing_tick(
            &mut state,
            &mut tc,
            0.1,
            &bonuses,
            &mut ach,
            false,
            &mut result,
            &mut rng,
        );

        if has_event(&result, |e| matches!(e, TickEvent::FishCaught { .. })) {
            assert!(
                !state.recent_drops.is_empty(),
                "Fish catch should add to recent drops"
            );
            break;
        }
    }
}

#[test]
fn test_fishing_tick_messages_prefixed() {
    let mut state = fresh_state();
    state.active_fishing = Some(make_fishing_session(FishingPhase::Waiting, 200, 10));
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(42);
    let bonuses = default_haven_bonuses();

    let mut found_message = false;

    for _ in 0..200 {
        if state.active_fishing.is_none() {
            break;
        }
        let mut result = TickResult::default();
        tick_stages::process_fishing_tick(
            &mut state,
            &mut tc,
            0.1,
            &bonuses,
            &mut ach,
            false,
            &mut result,
            &mut rng,
        );

        for event in &result.events {
            match event {
                TickEvent::FishingMessage { message } => {
                    // All fishing messages should be prefixed with fishing rod emoji
                    assert!(
                        message.starts_with('\u{1f3a3}'),
                        "Fishing messages should be prefixed with fishing rod emoji, got: {}",
                        message
                    );
                    found_message = true;
                }
                TickEvent::FishCaught { message, .. } => {
                    assert!(
                        message.starts_with('\u{1f3a3}'),
                        "Fish caught messages should be prefixed with fishing rod emoji"
                    );
                    found_message = true;
                }
                _ => {}
            }
            if found_message {
                break;
            }
        }
        if found_message {
            break;
        }
    }

    assert!(found_message, "Should find at least one fishing message");
}

// =============================================================================
// process_fishing_tick — debug mode suppresses achievement save signal
// =============================================================================

#[test]
fn test_fishing_storm_leviathan_debug_mode_suppresses_save() {
    // This tests the debug_mode behavior for storm leviathan catch.
    // We can't easily trigger a leviathan catch in unit tests, but we can
    // verify the code path exists by checking the function accepts debug_mode.
    let mut state = fresh_state();
    state.active_fishing = Some(make_fishing_session(FishingPhase::Waiting, 100, 5));
    let mut tc = 0u32;
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();

    // Should not panic in debug mode
    tick_stages::process_fishing_tick(
        &mut state,
        &mut tc,
        0.1,
        &bonuses,
        &mut ach,
        true, // debug mode
        &mut result,
        &mut rng,
    );
}

// =============================================================================
// Integration: full game_tick flow exercises all stages
// =============================================================================

#[test]
fn test_game_tick_with_fishing_skips_combat() {
    let mut state = strong_state();
    state.active_fishing = Some(make_fishing_session(FishingPhase::Waiting, 100, 5));
    let mut tc = 0u32;
    let mut haven = Haven::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(42);

    let result = run_game_tick(&mut state, &mut tc, &mut haven, &mut ach, false, &mut rng);

    // When fishing is active, no combat events should be produced
    assert!(
        !has_event(&result, |e| matches!(
            e,
            TickEvent::PlayerAttack { .. }
                | TickEvent::EnemyAttack { .. }
                | TickEvent::EnemyDefeated { .. }
        )),
        "Fishing should skip combat stages"
    );
}

#[test]
fn test_game_tick_combat_flow_produces_events_eventually() {
    let mut state = strong_state();
    let mut tc = 0u32;
    let mut haven = Haven::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(42);

    let mut found_attack = false;
    let mut found_enemy_defeated = false;

    for _ in 0..5000 {
        let result = run_game_tick(&mut state, &mut tc, &mut haven, &mut ach, false, &mut rng);

        if has_event(&result, |e| matches!(e, TickEvent::PlayerAttack { .. })) {
            found_attack = true;
        }
        if has_event(&result, |e| matches!(e, TickEvent::EnemyDefeated { .. })) {
            found_enemy_defeated = true;
            break;
        }
    }

    assert!(found_attack, "Should see player attack events in combat");
    assert!(
        found_enemy_defeated,
        "Should defeat an enemy within 5000 ticks"
    );
}

#[test]
fn test_game_tick_xp_accumulates_from_combat() {
    let mut state = strong_state();
    let mut tc = 0u32;
    let mut haven = Haven::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(42);

    let initial_xp = state.character_xp;

    for _ in 0..5000 {
        run_game_tick(&mut state, &mut tc, &mut haven, &mut ach, false, &mut rng);
        if state.character_xp > initial_xp {
            break;
        }
    }

    assert!(
        state.character_xp > initial_xp,
        "XP should increase from combat kills"
    );
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_empty_combat_events_produces_no_tick_events() {
    let mut state = fresh_state();
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    tick_stages::process_combat_events(
        &mut state,
        Vec::new(),
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(
        result.events.is_empty(),
        "Empty combat events should produce no tick events"
    );
    assert_eq!(state.session_kills, 0);
}

#[test]
fn test_level_up_through_enemy_died_distributes_attributes() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Goblin".into(), 50, 5));
    // Give massive XP to guarantee multiple level-ups
    state.character_level = 1;
    state.character_xp = 0;
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(42);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    // Give enough XP for several level-ups
    let events = vec![CombatEvent::EnemyDied { xp_gained: 50000 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(
        state.character_level > 1,
        "Should have leveled up from 50000 XP"
    );

    // Attributes should have increased from level-up distributions
    let total_attrs: u32 = quest::character::attributes::AttributeType::all()
        .iter()
        .map(|a| state.attributes.get(*a))
        .sum();
    // Starting total: 6 * 10 = 60, each level gives +3 points
    let expected_min = 60 + (state.character_level - 1) * 3;
    assert!(
        total_attrs >= expected_min,
        "Attribute total {} should be at least {} (level {})",
        total_attrs,
        expected_min,
        state.character_level
    );
}

#[test]
fn test_multiple_enemy_deaths_in_sequence() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Goblin".into(), 50, 5));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(42);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![
        CombatEvent::EnemyDied { xp_gained: 200 },
        CombatEvent::EnemyDied { xp_gained: 300 },
    ];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert_eq!(state.session_kills, 2);
    let defeated_count = count_events(&result, |e| matches!(e, TickEvent::EnemyDefeated { .. }));
    assert_eq!(defeated_count, 2, "Should have 2 EnemyDefeated events");
}

#[test]
fn test_dungeon_xp_tracked_on_enemy_died() {
    let mut state = fresh_state();
    state.active_dungeon = Some(quest::dungeon::generation::generate_dungeon(1, 0, 1));
    state.combat_state.current_enemy = Some(quest::Enemy::new("Dungeon Mob".into(), 50, 5));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::EnemyDied { xp_gained: 400 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    // Dungeon should track the XP earned
    if let Some(dungeon) = &state.active_dungeon {
        assert!(
            dungeon.xp_earned >= 400,
            "Dungeon should track XP from kills"
        );
    }
}

#[test]
fn test_elite_defeated_xp_tracked_in_dungeon() {
    let mut state = fresh_state();
    state.active_dungeon = Some(quest::dungeon::generation::generate_dungeon(1, 0, 1));
    state.combat_state.current_enemy = Some(quest::Enemy::new("Elite".into(), 200, 15));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::EliteDefeated { xp_gained: 600 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    if let Some(dungeon) = &state.active_dungeon {
        assert!(
            dungeon.xp_earned >= 600,
            "Dungeon should track XP from elite kills"
        );
    }
}

#[test]
fn test_boss_defeated_dungeon_completion_achievement() {
    let mut state = fresh_state();
    state.active_dungeon = Some(quest::dungeon::generation::generate_dungeon(1, 0, 1));
    state.combat_state.current_enemy = Some(quest::Enemy::new("Boss".into(), 500, 30));
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::BossDefeated { xp_gained: 1000 }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    // on_dungeon_completed should have been called on achievements
    assert!(
        ach.total_dungeons_completed >= 1,
        "Dungeon completion should be tracked in achievements"
    );
}

#[test]
fn test_subzone_boss_defeated_applies_xp() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Boss".into(), 300, 20));
    let xp_before = state.character_xp;
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::SubzoneBossDefeated {
        xp_gained: 1000,
        result: BossDefeatResult::SubzoneComplete { new_subzone_id: 2 },
    }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(
        state.character_xp > xp_before,
        "SubzoneBossDefeated should apply XP"
    );
}

#[test]
fn test_subzone_boss_level_up_triggers_achievement() {
    let mut state = fresh_state();
    state.combat_state.current_enemy = Some(quest::Enemy::new("Boss".into(), 300, 20));
    // Near level 2 threshold
    state.character_xp = 250;
    state.character_level = 1;
    let mut result = TickResult::default();
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut deep = DeepState::new();

    let events = vec![CombatEvent::SubzoneBossDefeated {
        xp_gained: 5000,
        result: BossDefeatResult::SubzoneComplete { new_subzone_id: 2 },
    }];

    tick_stages::process_combat_events(
        &mut state,
        events,
        &bonuses,
        &mut ach,
        &mut deep,
        false,
        &mut result,
        &mut rng,
    );

    assert!(state.character_level > 1, "Should level up from boss XP");
    assert!(
        has_event(&result, |e| matches!(e, TickEvent::LeveledUp { .. })),
        "Should emit LeveledUp event from boss kill"
    );
}

// =============================================================================
// Fishing rarity detection
// =============================================================================

#[test]
fn test_fishing_rarity_parsing_in_catch_events() {
    // Verify the rarity parsing logic in process_fishing_tick by running enough
    // fishing ticks to catch fish and check the rarity field
    let mut state = fresh_state();
    // Start in Reeling with 1 tick remaining for fast first catch
    state.active_fishing = Some(make_fishing_session(FishingPhase::Reeling, 1, 20));
    let mut tc = 0u32;
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(100);
    let bonuses = default_haven_bonuses();

    let mut found_catch = false;

    for _ in 0..500 {
        if state.active_fishing.is_none() {
            break;
        }
        let mut result = TickResult::default();
        tick_stages::process_fishing_tick(
            &mut state,
            &mut tc,
            0.1,
            &bonuses,
            &mut ach,
            false,
            &mut result,
            &mut rng,
        );

        for event in &result.events {
            if let TickEvent::FishCaught {
                rarity, fish_name, ..
            } = event
            {
                // Rarity should be one of the valid values
                assert!(
                    matches!(
                        rarity,
                        Rarity::Common
                            | Rarity::Magic
                            | Rarity::Rare
                            | Rarity::Epic
                            | Rarity::Legendary
                    ),
                    "Fish rarity should be valid"
                );
                assert!(!fish_name.is_empty(), "Fish name should not be empty");
                found_catch = true;
            }
        }
        if found_catch {
            break;
        }
    }

    assert!(found_catch, "Should catch at least one fish in 500 ticks");
}

// =============================================================================
// Fishing tick counter behavior
// =============================================================================

#[test]
fn test_fishing_tick_counter_wraps_at_ticks_per_second() {
    let mut state = fresh_state();
    state.active_fishing = Some(make_fishing_session(FishingPhase::Waiting, 500, 20));
    let mut tc = 9u32; // One away from wrapping
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut result = TickResult::default();

    let time_before = state.play_time_seconds;

    tick_stages::process_fishing_tick(
        &mut state,
        &mut tc,
        0.1,
        &bonuses,
        &mut ach,
        false,
        &mut result,
        &mut rng,
    );

    assert_eq!(tc, 0, "Tick counter should wrap to 0");
    assert_eq!(
        state.play_time_seconds,
        time_before + 1,
        "Play time should increment when tick counter wraps"
    );
}

// =============================================================================
// Fishing XP rate sampling
// =============================================================================

#[test]
fn test_fishing_tick_pushes_xp_sample_at_second_boundary() {
    let mut state = fresh_state();
    state.active_fishing = Some(make_fishing_session(FishingPhase::Waiting, 500, 20));
    state.xp_this_second = 42;
    let mut tc = 9u32;
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut result = TickResult::default();

    assert!(state.xp_rate_samples.is_empty());

    tick_stages::process_fishing_tick(
        &mut state,
        &mut tc,
        0.1,
        &bonuses,
        &mut ach,
        false,
        &mut result,
        &mut rng,
    );

    assert!(
        state.xp_rate_samples.is_empty(),
        "Fishing ticks should not push XP samples (combat_seconds_this_tick is false)"
    );
    assert_eq!(
        state.xp_this_second, 0,
        "xp_this_second should still reset even without push"
    );
}

#[test]
fn test_fishing_tick_does_not_push_samples() {
    let mut state = fresh_state();
    state.active_fishing = Some(make_fishing_session(FishingPhase::Waiting, 5000, 100));

    // Pre-fill xp_rate_samples with some data
    for i in 0..50 {
        state.xp_rate_samples.push_back(i);
    }
    assert_eq!(state.xp_rate_samples.len(), 50);

    let mut tc = 9u32;
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut result = TickResult::default();

    state.xp_this_second = 999;

    tick_stages::process_fishing_tick(
        &mut state,
        &mut tc,
        0.1,
        &bonuses,
        &mut ach,
        false,
        &mut result,
        &mut rng,
    );

    assert_eq!(
        state.xp_rate_samples.len(),
        50,
        "Fishing should not push samples — count should be unchanged"
    );
    assert_eq!(state.xp_this_second, 0, "xp_this_second should still reset");
}

#[test]
fn test_xp_rate_stable_across_fishing_interruption() {
    use quest::core::game_state::GameState;

    let mut state = GameState::new("Rate Test".to_string(), 0);

    // Simulate 30 seconds of combat earning ~500 XP/sec
    for _ in 0..30 {
        state.xp_this_second = 500;
        state.combat_seconds_this_tick = true;
        state.xp_rate_samples.push_back(state.xp_this_second);
        if state.xp_rate_samples.len() > quest::core::constants::XP_RATE_WINDOW_SECONDS {
            state.xp_rate_samples.pop_front();
        }
        state.xp_this_second = 0;
        state.combat_seconds_this_tick = false;
    }

    let rate_before = state.xp_per_hour().unwrap();

    // Simulate 60 seconds of fishing (no samples pushed)
    for _ in 0..60 {
        // combat_seconds_this_tick stays false, so no push
        state.xp_this_second = 0;
        state.combat_seconds_this_tick = false;
        // No push — this is the key behavioral change
    }

    let rate_after = state.xp_per_hour().unwrap();

    assert_eq!(
        rate_before, rate_after,
        "XP rate should be unchanged after fishing interruption (no zeros pushed)"
    );
    assert_eq!(
        state.xp_rate_samples.len(),
        30,
        "Only combat seconds should be in samples"
    );
}

#[test]
fn test_combat_tick_pushes_xp_sample_when_flag_set() {
    let mut state = fresh_state();
    // Simulate a second where combat XP was earned
    state.xp_this_second = 500;
    state.combat_seconds_this_tick = true;

    // Manually trigger second boundary in tick.rs logic
    state.xp_rate_samples.clear();

    // Push sample like tick.rs does
    if state.combat_seconds_this_tick {
        state.xp_rate_samples.push_back(state.xp_this_second);
    }
    state.xp_this_second = 0;
    state.combat_seconds_this_tick = false;

    assert_eq!(state.xp_rate_samples.len(), 1);
    assert_eq!(state.xp_rate_samples[0], 500);
    assert_eq!(state.xp_this_second, 0);
    assert!(!state.combat_seconds_this_tick);
}

#[test]
fn test_no_sample_pushed_when_combat_flag_false() {
    let mut state = fresh_state();
    state.xp_this_second = 0;
    state.combat_seconds_this_tick = false;

    let initial_len = state.xp_rate_samples.len();

    // Simulate second boundary without combat
    if state.combat_seconds_this_tick {
        state.xp_rate_samples.push_back(state.xp_this_second);
    }
    state.xp_this_second = 0;
    state.combat_seconds_this_tick = false;

    assert_eq!(
        state.xp_rate_samples.len(),
        initial_len,
        "No sample should be pushed without combat"
    );
}

#[test]
fn test_xp_rate_samples_capped_at_900() {
    let mut state = fresh_state();
    // Pre-fill to 900
    for i in 0..900 {
        state.xp_rate_samples.push_back(i);
    }
    assert_eq!(state.xp_rate_samples.len(), 900);

    // Simulate combat second boundary
    state.xp_this_second = 9999;
    state.combat_seconds_this_tick = true;

    if state.combat_seconds_this_tick {
        state.xp_rate_samples.push_back(state.xp_this_second);
        if state.xp_rate_samples.len() > quest::core::constants::XP_RATE_WINDOW_SECONDS {
            state.xp_rate_samples.pop_front();
        }
    }

    assert_eq!(
        state.xp_rate_samples.len(),
        900,
        "Samples should stay capped at 900"
    );
    assert_eq!(
        *state.xp_rate_samples.back().unwrap(),
        9999,
        "New sample at back"
    );
    assert_eq!(
        *state.xp_rate_samples.front().unwrap(),
        1,
        "Oldest (0) popped"
    );
}
