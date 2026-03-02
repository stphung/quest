//! Chrono Surge batched tick execution.

use crate::achievements::Achievements;
use crate::core::game_state::GameState;
use crate::core::tick_context::TickContext;
use crate::deep::DeepState;
use crate::enhancement::types::EnhancementProgress;
use crate::haven::types::Haven;
use crate::history::SaveEvent;
use crate::stormglass::types::{ChronoSurgeState, ChronoSurgeSummary};

/// Result returned from a single Chrono Surge batch execution.
pub struct ChronoSurgeBatchResult {
    /// Whether state changes occurred that require saving (mid-surge save).
    pub needs_save: bool,
    /// Present if the surge completed this batch. Contains the final summary
    /// and a pre-built `SaveEvent` for the completion git commit.
    pub completed: Option<(ChronoSurgeSummary, SaveEvent)>,
}

/// Execute one batch of Chrono Surge ticks.
///
/// Runs `surge.current_batch_size()` ticks (capped to `ticks_remaining`),
/// suppresses stormglass salvage earned during the batch (temporal
/// displacement), tallies summary counters, and decrements `ticks_remaining`.
///
/// Returns a `ChronoSurgeBatchResult` describing whether a mid-surge save is
/// needed and whether the surge finished. The caller is responsible for:
/// - Calling `save_all()` when `needs_save` is true.
/// - Setting `state.chrono_surge_active = false` and storing the summary when
///   `completed` is `Some`.
/// - Calling `save_all()` with the returned `SaveEvent` on completion (unless
///   in debug mode).
#[allow(clippy::too_many_arguments)]
pub fn run_chrono_surge_batch(
    surge: &mut ChronoSurgeState,
    state: &mut GameState,
    tick_counter: &mut u32,
    haven: &mut Haven,
    enhancement: &mut EnhancementProgress,
    deep_state: &mut DeepState,
    global_achievements: &mut Achievements,
    debug_mode: bool,
) -> ChronoSurgeBatchResult {
    let batch = surge.current_batch_size().min(surge.ticks_remaining);

    let mut rng = rand::rng();
    let mut needs_save = false;
    let sg_before_batch = state.stormglass;

    for _ in 0..batch {
        let mut ctx = TickContext {
            state,
            tick_counter,
            haven,
            enhancement,
            deep: deep_state,
            achievements: global_achievements,
            debug_mode,
        };
        let tick_result = crate::core::tick::game_tick_with_context(&mut ctx, &mut rng);

        // Keep surge path headless (same semantics as [Esc] skip):
        // collect summary counters only and avoid per-tick UI work.
        tally_chrono_surge_events(surge, &tick_result.events);

        if tick_result.achievements_changed
            || tick_result.haven_changed
            || tick_result.enhancement_changed
            || tick_result.god_items_changed
            || tick_result.deep_changed
        {
            needs_save = true;
        }

        surge.ticks_remaining -= 1;
    }

    // No SG earned during Chrono Surge — temporal displacement
    // prevents salvage from materializing
    state.stormglass = sg_before_batch;

    let completed = if surge.ticks_remaining == 0 {
        let summary = ChronoSurgeSummary {
            kills: surge.kills,
            levels_gained: surge.levels_gained,
            items_equipped: surge.items_equipped,
            ticks_completed: surge.ticks_total,
            ticks_total: surge.ticks_total,
            overcharged: surge.overcharged,
        };
        let save_event = SaveEvent::ChronoSurge {
            levels_gained: surge.levels_gained,
            kills: surge.kills,
            ticks: surge.ticks_total,
        };
        Some((summary, save_event))
    } else {
        None
    };

    ChronoSurgeBatchResult {
        needs_save,
        completed,
    }
}

fn tally_chrono_surge_events(surge: &mut ChronoSurgeState, events: &[crate::core::tick::TickEvent]) {
    for event in events {
        match event {
            crate::core::tick::TickEvent::EnemyDefeated { .. }
            | crate::core::tick::TickEvent::SubzoneBossDefeated { .. }
            | crate::core::tick::TickEvent::DungeonBossDefeated { .. }
            | crate::core::tick::TickEvent::DungeonEliteDefeated { .. } => {
                surge.kills += 1;
            }
            crate::core::tick::TickEvent::ItemDropped { equipped: true, .. }
            | crate::core::tick::TickEvent::DungeonTreasureFound { equipped: true, .. } => {
                surge.items_equipped += 1;
            }
            crate::core::tick::TickEvent::LeveledUp { .. } => {
                surge.levels_gained += 1;
            }
            _ => {}
        }
    }
}
