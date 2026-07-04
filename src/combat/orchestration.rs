use crate::character::derived_stats::DerivedStats;
use crate::core::constants::*;
use crate::core::game_state::GameState;
use rand::Rng;

use super::attacks::effective_enemy_attack_interval;
use super::events::{CombatBonuses, CombatEvent};

/// Updates combat state, returns events that occurred
/// `bonuses` contains all combined combat bonuses from Haven, god items, prestige, and sigils.
/// `achievements` is used to check for Stormbreaker achievement (Zone 10 boss)
/// `derived` contains pre-computed derived stats (avoids redundant recalculation)
#[allow(clippy::too_many_arguments)]
pub fn update_combat<R: Rng>(
    rng: &mut R,
    state: &mut GameState,
    delta_time: f64,
    bonuses: &CombatBonuses,
    achievements: &mut crate::achievements::Achievements,
    derived: &DerivedStats,
    fracture_zone_cap: u32,
    loom_zone_cap: u32,
) -> Vec<CombatEvent> {
    let mut events = Vec::new();

    // Handle regeneration after enemy death
    if state.combat_state.is_regenerating {
        return super::regen::process_regen(state, delta_time, bonuses, derived);
    }

    // No combat if no enemy
    if state.combat_state.current_enemy.is_none() {
        return events;
    }

    // --- Phase 1: Accumulate timers ---
    state.combat_state.player_attack_timer += delta_time;
    state.combat_state.enemy_attack_timer += delta_time;

    // --- Phase 1b: Boss enrage timer ---
    if state.zone_progression.fighting_boss {
        state.combat_state.boss_fight_timer += delta_time;
        if state.combat_state.boss_fight_timer >= BOSS_ENRAGE_SECONDS {
            events.extend(resolve_boss_enrage(state, achievements));
            return events;
        }
    }

    // --- Phase 1c: Mob fight timeout ---
    if !state.zone_progression.fighting_boss {
        state.combat_state.current_fight_elapsed += delta_time;
        let timeout = if state.active_dungeon.is_some() {
            DUNGEON_FIGHT_TIMEOUT_SECONDS
        } else {
            MOB_FIGHT_TIMEOUT_SECONDS
        };
        if state.combat_state.current_fight_elapsed >= timeout {
            // Stalemate, not a death loop — retreat without frontier backoff
            events.extend(resolve_combat_retreat(state, false));
            return events;
        }
    }

    // Attack speed multiplier: higher = faster attacks
    let player_interval = ATTACK_INTERVAL_SECONDS
        / (derived.attack_speed_multiplier + bonuses.attack_speed_percent / 100.0);
    let enemy_interval = effective_enemy_attack_interval(state);

    // --- Phase 2: Determine who attacks this tick ---
    let player_attacks = state.combat_state.player_attack_timer >= player_interval;
    let enemy_attacks = state.combat_state.enemy_attack_timer >= enemy_interval;

    // --- Phase 3: Player attack (if ready) ---
    if player_attacks {
        let (attack_events, enemy_died) = super::player_attack::resolve_player_attack(
            rng,
            state,
            bonuses,
            achievements,
            derived,
            fracture_zone_cap,
            loom_zone_cap,
        );
        events.extend(attack_events);
        if enemy_died {
            return events;
        }
    }

    // --- Phase 4: Enemy attack (if ready) ---
    if enemy_attacks {
        let attack_events = super::enemy_attack::resolve_enemy_attack(
            rng,
            state,
            bonuses,
            achievements,
            derived,
            fracture_zone_cap,
            loom_zone_cap,
        );
        events.extend(attack_events);
    }

    events
}

/// Resolves combat retreat: player is overwhelmed and retreats to the last safe zone.
///
/// Called when mob fight timeout or death loop threshold is reached.
/// Finds the highest zone with a defeated boss and travels there.
///
/// `triggered_by_death` marks death-loop retreats: those record the zone for
/// frontier backoff so boss-defeat advancement stops bouncing the player back
/// into a zone that keeps killing them (#576). Stalemate (timeout) retreats
/// don't — the player survives those and retrying is cheap.
pub fn resolve_combat_retreat(state: &mut GameState, triggered_by_death: bool) -> Vec<CombatEvent> {
    let mut events = Vec::new();

    if triggered_by_death {
        state.zone_progression.record_death_retreat();
    }

    // Retreating while inside a dungeon abandons it entirely (safe exit, no
    // prestige loss). Otherwise the uncleared room respawns its enemy at full
    // HP and the fight timeout loops forever.
    if state.active_dungeon.is_some() {
        state.active_dungeon = None;
        events.push(CombatEvent::DungeonRetreat);
    }

    // Find last safe zone: highest zone_id with a defeated boss
    let safe_zone_id = state
        .zone_progression
        .defeated_bosses
        .iter()
        .map(|(zone_id, _)| *zone_id)
        .max()
        .unwrap_or(1); // Fall back to Zone 1

    let zone_name = crate::zones::get_zone(safe_zone_id)
        .map(|z| z.name.to_string())
        .unwrap_or_else(|| "Meadow".to_string());

    // Reset combat state
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;
    state.combat_state.player_attack_timer = 0.0;
    state.combat_state.enemy_attack_timer = 0.0;
    state.combat_state.current_fight_elapsed = 0.0;
    state.combat_state.current_enemy = None;

    // Move to safe zone
    state.zone_progression.current_zone_id = safe_zone_id;
    state.zone_progression.current_subzone_id = 1;
    state.zone_progression.kills_in_subzone = 0;
    state.zone_progression.fighting_boss = false;

    // Reset death counter
    state.consecutive_deaths = 0;

    events.push(CombatEvent::CombatRetreat { zone_name });
    events
}

/// Resolves boss enrage: instant kills the player and resets combat state.
///
/// Always retreats the player to subzone 1 of the current zone.
/// For weapon-gated bosses this prevents an infinite loop; for normal bosses
/// it signals the player isn't strong enough yet.
fn resolve_boss_enrage(
    state: &mut GameState,
    achievements: &crate::achievements::Achievements,
) -> Vec<CombatEvent> {
    let weapon_blocked = state
        .zone_progression
        .boss_weapon_blocked(achievements)
        .is_some();
    let enemy_name = state
        .combat_state
        .current_enemy
        .as_ref()
        .map(|e| e.name.clone())
        .unwrap_or_else(|| "Boss".to_string());

    // Reset combat state
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;
    state.combat_state.player_attack_timer = 0.0;
    state.combat_state.enemy_attack_timer = 0.0;
    state.combat_state.boss_fight_timer = 0.0;
    state.combat_state.current_enemy = None;

    // Retreat to first subzone of current zone
    state.zone_progression.fighting_boss = false;
    state.zone_progression.kills_in_subzone = 0;
    state.zone_progression.current_subzone_id = 1;

    vec![CombatEvent::BossEnrage {
        weapon_blocked,
        enemy_name,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::Achievements;
    use crate::combat::Enemy;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn state_with_enemy(enemy_name: &str) -> GameState {
        let mut state = GameState::new("Hero".to_string(), 0);
        state.combat_state.current_enemy = Some(Enemy::new(enemy_name.to_string(), 50, 5));
        state.combat_state.player_max_hp = 120;
        state.combat_state.player_current_hp = 17;
        state
    }

    #[test]
    fn resolve_combat_retreat_falls_back_to_zone_one() {
        let mut state = state_with_enemy("Dire Wolf");
        state.zone_progression.current_zone_id = 6;
        state.zone_progression.current_subzone_id = 3;
        state.zone_progression.kills_in_subzone = 8;
        state.combat_state.player_attack_timer = 1.0;
        state.combat_state.enemy_attack_timer = 2.0;
        state.combat_state.current_fight_elapsed = 12.0;
        state.consecutive_deaths = 4;

        let events = resolve_combat_retreat(&mut state, true);

        assert!(matches!(
            events.as_slice(),
            [CombatEvent::CombatRetreat { zone_name }] if zone_name == "Meadow"
        ));
        assert_eq!(state.zone_progression.current_zone_id, 1);
        assert_eq!(state.zone_progression.current_subzone_id, 1);
        assert_eq!(state.zone_progression.kills_in_subzone, 0);
        assert!(!state.zone_progression.fighting_boss);
        assert_eq!(
            state.combat_state.player_current_hp,
            state.combat_state.player_max_hp
        );
        assert!(state.combat_state.current_enemy.is_none());
        assert_eq!(state.consecutive_deaths, 0);
        // Death-triggered retreat records the zone for frontier backoff
        assert_eq!(state.zone_progression.death_retreat_zone, Some(6));
        assert_eq!(state.zone_progression.death_retreat_count, 1);
        assert_eq!(state.zone_progression.frontier_cooldown_cycles, 1);
    }

    #[test]
    fn resolve_combat_retreat_stalemate_does_not_record_backoff() {
        let mut state = state_with_enemy("Dire Wolf");
        state.zone_progression.current_zone_id = 6;

        let events = resolve_combat_retreat(&mut state, false);

        assert!(matches!(
            events.as_slice(),
            [CombatEvent::CombatRetreat { .. }]
        ));
        assert_eq!(state.zone_progression.death_retreat_zone, None);
        assert_eq!(state.zone_progression.frontier_cooldown_cycles, 0);
    }

    #[test]
    fn resolve_combat_retreat_uses_highest_defeated_zone() {
        let mut state = state_with_enemy("Dire Wolf");
        state.zone_progression.defeated_bosses.insert((3, 3));
        state.zone_progression.defeated_bosses.insert((5, 2));

        let events = resolve_combat_retreat(&mut state, true);
        let expected_zone = crate::zones::get_zone(5).unwrap().name.to_string();

        assert!(matches!(
            events.as_slice(),
            [CombatEvent::CombatRetreat { zone_name }] if zone_name == &expected_zone
        ));
        assert_eq!(state.zone_progression.current_zone_id, 5);
    }

    #[test]
    fn resolve_combat_retreat_falls_back_to_meadow_name_for_unknown_zone() {
        // Defensive fallback: if defeated_bosses somehow references a zone_id
        // with no zone data, the zone_name should fall back to "Meadow"
        // rather than panicking (get_zone() returns None for unknown ids).
        let mut state = state_with_enemy("Dire Wolf");
        state.zone_progression.defeated_bosses.insert((9999, 1));

        let events = resolve_combat_retreat(&mut state, true);

        assert!(matches!(
            events.as_slice(),
            [CombatEvent::CombatRetreat { zone_name }] if zone_name == "Meadow"
        ));
        assert_eq!(state.zone_progression.current_zone_id, 9999);
    }

    #[test]
    fn resolve_boss_enrage_falls_back_to_boss_name_without_current_enemy() {
        // Defensive fallback: resolve_boss_enrage is only called by
        // update_combat while current_enemy is Some, but it shouldn't panic
        // if called with no enemy present — enemy_name should fall back to
        // "Boss".
        let mut state = GameState::new("Hero".to_string(), 0);
        state.combat_state.current_enemy = None;
        let achievements = Achievements::default();

        let events = resolve_boss_enrage(&mut state, &achievements);

        assert!(matches!(
            events.as_slice(),
            [CombatEvent::BossEnrage {
                weapon_blocked: false,
                enemy_name
            }] if enemy_name == "Boss"
        ));
    }

    #[test]
    fn update_combat_neither_timer_ready_produces_no_attack_events() {
        // Both timers start below their intervals; a small delta_time tick
        // should just accumulate time without any attack resolving.
        let mut rng = ChaCha8Rng::seed_from_u64(11);
        let mut state = state_with_enemy("Dire Wolf");
        state.combat_state.player_attack_timer = 0.0;
        state.combat_state.enemy_attack_timer = 0.0;

        let events = update_combat(
            &mut rng,
            &mut state,
            0.1,
            &CombatBonuses::default(),
            &mut Achievements::default(),
            &DerivedStats::default(),
            11,
            30,
        );

        assert!(events.is_empty());
        assert!((state.combat_state.player_attack_timer - 0.1).abs() < f64::EPSILON);
        assert!((state.combat_state.enemy_attack_timer - 0.1).abs() < f64::EPSILON);
        assert!(state.combat_state.current_enemy.is_some());
    }

    #[test]
    fn resolve_combat_retreat_abandons_active_dungeon() {
        let mut state = state_with_enemy("Boss Gorehowl");
        state.active_dungeon = Some(crate::dungeon::generation::generate_dungeon(10, 0, 1));

        let events = resolve_combat_retreat(&mut state, false);

        assert!(
            state.active_dungeon.is_none(),
            "Retreat should abandon the dungeon so its room can't respawn the enemy"
        );
        assert!(matches!(
            events.as_slice(),
            [
                CombatEvent::DungeonRetreat,
                CombatEvent::CombatRetreat { .. }
            ]
        ));
        assert!(state.combat_state.current_enemy.is_none());
    }

    #[test]
    fn update_combat_dungeon_fight_survives_mob_timeout() {
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let mut state = state_with_enemy("Boss Gorehowl");
        state.active_dungeon = Some(crate::dungeon::generation::generate_dungeon(10, 0, 1));
        state.combat_state.current_fight_elapsed = MOB_FIGHT_TIMEOUT_SECONDS + 5.0;

        let events = update_combat(
            &mut rng,
            &mut state,
            0.1,
            &CombatBonuses::default(),
            &mut Achievements::default(),
            &DerivedStats::default(),
            11,
            30,
        );

        assert!(
            !events
                .iter()
                .any(|e| matches!(e, CombatEvent::CombatRetreat { .. })),
            "Dungeon fights should use the longer dungeon timeout, not the mob timeout"
        );
        assert!(state.active_dungeon.is_some());
    }

    #[test]
    fn update_combat_dungeon_timeout_exits_dungeon() {
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let mut state = state_with_enemy("Boss Gorehowl");
        state.active_dungeon = Some(crate::dungeon::generation::generate_dungeon(10, 0, 1));
        state.combat_state.current_fight_elapsed = DUNGEON_FIGHT_TIMEOUT_SECONDS - 0.05;

        let events = update_combat(
            &mut rng,
            &mut state,
            0.1,
            &CombatBonuses::default(),
            &mut Achievements::default(),
            &DerivedStats::default(),
            11,
            30,
        );

        assert!(matches!(
            events.as_slice(),
            [
                CombatEvent::DungeonRetreat,
                CombatEvent::CombatRetreat { .. }
            ]
        ));
        assert!(
            state.active_dungeon.is_none(),
            "Timing out against a dungeon enemy should exit the dungeon"
        );
        assert!(state.combat_state.current_enemy.is_none());
        assert_eq!(
            state.combat_state.player_current_hp,
            state.combat_state.player_max_hp
        );
    }

    #[test]
    fn update_combat_triggers_boss_enrage_before_attack_resolution() {
        let mut rng = ChaCha8Rng::seed_from_u64(5);
        let mut state = state_with_enemy("Warden");
        state.zone_progression.fighting_boss = true;
        state.zone_progression.current_zone_id = 1;
        state.zone_progression.current_subzone_id = 4;
        state.zone_progression.kills_in_subzone = 9;
        state.combat_state.player_attack_timer = 10.0;
        state.combat_state.enemy_attack_timer = 10.0;
        state.combat_state.boss_fight_timer = BOSS_ENRAGE_SECONDS - 0.1;

        let events = update_combat(
            &mut rng,
            &mut state,
            0.1,
            &CombatBonuses::default(),
            &mut Achievements::default(),
            &DerivedStats::default(),
            11,
            30,
        );

        assert!(matches!(
            events.as_slice(),
            [CombatEvent::BossEnrage {
                weapon_blocked: false,
                enemy_name
            }] if enemy_name == "Warden"
        ));
        assert!(state.combat_state.current_enemy.is_none());
        assert_eq!(state.combat_state.boss_fight_timer, 0.0);
        assert_eq!(
            state.combat_state.player_current_hp,
            state.combat_state.player_max_hp
        );
        assert!(!state.zone_progression.fighting_boss);
        assert_eq!(state.zone_progression.current_subzone_id, 1);
        assert_eq!(state.zone_progression.kills_in_subzone, 0);
    }

    #[test]
    fn update_combat_retreats_to_safe_zone_after_mob_timeout() {
        let mut rng = ChaCha8Rng::seed_from_u64(9);
        let mut state = state_with_enemy("Bandit");
        state.zone_progression.current_zone_id = 6;
        state.zone_progression.current_subzone_id = 2;
        state.zone_progression.defeated_bosses.insert((4, 3));
        state.combat_state.current_fight_elapsed = MOB_FIGHT_TIMEOUT_SECONDS - 0.05;

        let events = update_combat(
            &mut rng,
            &mut state,
            0.05,
            &CombatBonuses::default(),
            &mut Achievements::default(),
            &DerivedStats::default(),
            11,
            30,
        );

        let expected_zone = crate::zones::get_zone(4).unwrap().name.to_string();
        assert!(matches!(
            events.as_slice(),
            [CombatEvent::CombatRetreat { zone_name }] if zone_name == &expected_zone
        ));
        assert_eq!(state.zone_progression.current_zone_id, 4);
        assert!(state.combat_state.current_enemy.is_none());
    }
}
