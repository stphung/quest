//! Achievement tracking helpers for input-driven events.

use crate::achievements;
use crate::core::game_state::GameState;

/// Log synced achievements to the combat log after loading a character.
pub fn log_synced_achievements(
    state: &mut GameState,
    global_achievements: &mut achievements::Achievements,
) {
    let synced_count = global_achievements.pending_count();
    if synced_count > 0 {
        if synced_count == 1 {
            if let Some(id) = global_achievements.pending_notifications.first() {
                if let Some(def) = achievements::get_achievement_def(*id) {
                    state.combat_state.add_log_entry(
                        format!("\u{1f3c6} Achievement Unlocked: {}", def.name),
                        false,
                        true,
                    );
                }
            }
        } else {
            state.combat_state.add_log_entry(
                format!(
                    "\u{1f3c6} {} achievements synced from progress!",
                    synced_count
                ),
                false,
                true,
            );
        }
        global_achievements.newly_unlocked.clear();
    }
}

/// Track achievements that may have changed from input handling (prestige, minigame wins).
/// Saving is handled by the caller via save_all().
///
/// # Why this lives outside tick.rs (R4 skip rationale)
///
/// A previous refactoring pass (R4) considered moving these two triggers into
/// `tick.rs` or the relevant domain modules so that all achievement tracking is
/// co-located with game logic.  That was deferred for the following reasons:
///
/// **Prestige**: Prestige fires from user input (Enter on the prestige dialog),
/// not from `game_tick()`.  Moving it to `tick.rs` would require either:
/// (a) adding a `prestige_changed` flag to `TickResult`, or
/// (b) having `tick.rs` compare the current and previous prestige rank each
///     tick -- introducing a data-dependency the tick engine doesn't currently
///     carry.  Both options expand `TickResult`'s interface for minimal gain.
///
/// **Minigame wins**: `state.last_minigame_win` is set inside input handlers.
/// `tick.rs` *could* drain it, but that would delay the achievement by one tick
/// (100 ms) and place input-state management inside a pure-logic function.
///
/// The current design is a clean two-phase split: `game_tick()` handles all
/// autonomous game events; `track_input_achievements()` handles the two cases
/// where achievement progress is driven directly by player input.  Revisit if
/// the number of input-driven achievements grows significantly.
pub fn track_input_achievements(
    state: &mut GameState,
    global_achievements: &mut achievements::Achievements,
    prestige_before: u32,
) {
    if state.prestige_rank > prestige_before {
        global_achievements.on_prestige(state.prestige_rank, Some(&state.character_name));
    }

    if let Some(ref win_info) = state.last_minigame_win {
        global_achievements.on_minigame_won(
            win_info.game_type,
            win_info.difficulty,
            Some(&state.character_name),
        );
        state.last_minigame_win = None;
    }
}
