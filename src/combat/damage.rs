use crate::core::game_state::GameState;
use crate::dungeon::types::RoomType;

use super::events::CombatEvent;

/// Handles the common logic when an enemy is killed (by player attack or reflection).
///
/// Calculates XP, emits the appropriate kill event based on dungeon/zone context,
/// updates achievements, and cleans up combat state (removes enemy, starts regen).
///
/// Returns `(events, is_boss_kill)`.
pub(crate) fn handle_enemy_death(
    state: &mut GameState,
    achievements: &mut crate::achievements::Achievements,
    haven_xp_gain_percent: f64,
) -> (Vec<CombatEvent>, bool) {
    let mut events = Vec::new();

    let wis_mod = state
        .attributes
        .modifier(crate::character::attributes::AttributeType::Wisdom);
    let cha_mod = state
        .attributes
        .modifier(crate::character::attributes::AttributeType::Charisma);
    let xp_gained = crate::core::game_logic::combat_kill_xp(
        crate::core::game_logic::xp_gain_per_tick(state.prestige_rank, wis_mod, cha_mod),
        haven_xp_gain_percent,
    );

    // Check if we're in a dungeon and what type of room
    let dungeon_room_type = state
        .active_dungeon
        .as_ref()
        .and_then(|d| d.current_room())
        .map(|r| r.room_type);

    // Track if this was a boss-level kill for achievements
    let is_boss_kill = matches!(
        dungeon_room_type,
        Some(RoomType::Elite) | Some(RoomType::Boss)
    ) || (state.active_dungeon.is_none()
        && state.zone_progression.fighting_boss);

    match dungeon_room_type {
        Some(RoomType::Elite) => {
            events.push(CombatEvent::EliteDefeated { xp_gained });
        }
        Some(RoomType::Boss) => {
            events.push(CombatEvent::BossDefeated { xp_gained });
        }
        _ => {
            if state.active_dungeon.is_some() {
                // Dungeon Combat room kill — don't affect zone progression
                events.push(CombatEvent::EnemyDied { xp_gained });
            } else if state.zone_progression.fighting_boss {
                // Overworld boss defeated
                let result = state
                    .zone_progression
                    .on_boss_defeated(state.prestige_rank, achievements);
                events.push(CombatEvent::SubzoneBossDefeated { xp_gained, result });
            } else {
                // Record the kill for boss spawn tracking (boss flag set if threshold reached)
                state.zone_progression.record_kill();
                events.push(CombatEvent::EnemyDied { xp_gained });
            }
        }
    }

    // Track kill for achievements
    achievements.on_enemy_killed(is_boss_kill, Some(&state.character_name));

    // Remove enemy and start regeneration
    state.combat_state.current_enemy = None;
    state.combat_state.enemy_attack_timer = 0.0;
    state.combat_state.is_regenerating = true;
    state.combat_state.regen_timer = 0.0;
    state.combat_state.regen_start_hp = state.combat_state.player_current_hp;

    (events, is_boss_kill)
}
