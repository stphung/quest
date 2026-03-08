//! Offline progression processing.

use crate::core::game_logic::{process_offline_progression, OfflineReport};
use crate::core::game_state::GameState;
use crate::haven;
use crate::loom;

/// Resolve Deep missions that completed while the game was closed.
pub fn resolve_deep_offline(
    deep: &mut crate::deep::DeepState,
    achievements: &mut crate::achievements::Achievements,
    character_name: &str,
) {
    if !deep.persistent.discovered {
        return;
    }

    let mut rng = rand::rng();

    let now = chrono::Utc::now();
    let summary = crate::deep::missions::tick_all_missions(
        &mut deep.prestige,
        &mut deep.persistent,
        now,
        &mut rng,
    );

    // Fire achievement handlers
    for _ in 0..summary.missions_completed {
        achievements.on_deep_mission_complete(Some(character_name));
    }
    for layer in &summary.breakthroughs {
        achievements.on_deep_breakthrough(*layer, Some(character_name));
        // Check if this breakthrough unlocks a fracture region (mirrors tick_stages.rs)
        if let Some(region) = crate::zones::FractureRegion::from_layer(*layer) {
            let new_cap = region.end_zone_id();
            if new_cap > deep.persistent.fracture_zone_cap {
                deep.persistent.fracture_zone_cap = new_cap;
                deep.persistent.pending_fracture_region_unlock = Some(region);
            }
        }
    }
    for _ in 0..summary.mercs_lost {
        achievements.on_deep_merc_lost(Some(character_name));
    }
    if summary.gateway_opened {
        achievements.on_deep_gateway_opened(Some(character_name));
    }

    // After offline resolution, refresh the mission pool if stale or empty.
    // This ensures players always have missions available when they return.
    crate::deep::missions::maybe_refresh_mission_pool(
        &mut deep.prestige,
        &deep.persistent,
        now,
        &mut rng,
    );

    crate::deep::missions::run_softlock_safeguards(
        &mut deep.prestige,
        &mut deep.persistent,
        now,
        &mut rng,
    );
}

/// Process offline XP and add combat log entries. Returns the report if XP was gained.
pub fn apply_offline_xp(state: &mut GameState, haven: &haven::Haven) -> Option<OfflineReport> {
    let haven_offline_bonus = haven.get_bonus(haven::HavenBonusType::OfflineXpPercent);
    let sigil_offline_bonus =
        crate::stormglass::sigils::SigilBonuses::compute(&state.storm_sigils).offline_xp_percent;
    let report = process_offline_progression(
        &mut rand::rng(),
        state,
        haven_offline_bonus + sigil_offline_bonus,
    );
    if report.xp_gained > 0 {
        let hours = report.elapsed_seconds / 3600;
        let minutes = (report.elapsed_seconds % 3600) / 60;
        let away_str = if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        };
        state.combat_state.add_log_entry(
            format!("\u{2600}\u{fe0f} Welcome back! ({} away)", away_str),
            false,
            true,
        );
        state.combat_state.add_log_entry(
            format!(
                "\u{2694}\u{fe0f} +{} XP gained offline",
                crate::ui::game_common::format_number_short(report.xp_gained)
            ),
            false,
            true,
        );
        if report.total_level_ups > 0 {
            state.combat_state.add_log_entry(
                format!(
                    "\u{1f4c8} Leveled up {} times! ({} \u{2192} {})",
                    report.total_level_ups, report.level_before, report.level_after,
                ),
                false,
                true,
            );
        }
        state.ticker.push(crate::core::game_state::TickerEntry {
            icon: "\u{2600}",
            text: format!(
                "+{} XP offline",
                crate::ui::game_common::format_number_short(report.xp_gained)
            ),
            color: ratatui::style::Color::Green,
            bold: false,
            segments: None,
        });
        Some(report)
    } else {
        None
    }
}

/// Result of offline Loom simulation.
pub struct LoomOfflineReport {
    /// Number of Woven Patterns completed during offline time.
    pub patterns_completed: u32,
}

/// Simulate Loom production for time elapsed while offline.
///
/// Runs extractor production, shuttle processing, neighbor unlocking,
/// and pattern sustain in 10-second steps for the offline duration
/// (capped at 7 days). Rate trackers are rebuilt from scratch since
/// they are transient.
pub fn resolve_loom_offline(
    loom_state: &mut loom::LoomState,
    elapsed_seconds: i64,
) -> Option<LoomOfflineReport> {
    if !loom_state.persistent.discovered || elapsed_seconds <= 0 {
        return None;
    }

    let patterns_before = loom_state.persistent.completed_pattern_count();

    // Cap at 7 days, same as offline XP.
    let max_offline: i64 = 7 * 24 * 3600;
    let total_seconds = elapsed_seconds.min(max_offline) as f64;

    // Simulate in 10-second steps for reasonable accuracy + speed.
    // 7 days = ~60,480 steps — completes in milliseconds.
    const STEP_SECONDS: f64 = 10.0;
    let steps = (total_seconds / STEP_SECONDS) as u64;
    let remainder = total_seconds - (steps as f64 * STEP_SECONDS);

    for _ in 0..steps {
        simulate_loom_step(loom_state, STEP_SECONDS);
    }
    if remainder > 0.0 {
        simulate_loom_step(loom_state, remainder);
    }

    let patterns_after = loom_state.persistent.completed_pattern_count();
    let patterns_completed = (patterns_after - patterns_before) as u32;

    Some(LoomOfflineReport { patterns_completed })
}

/// Run one simulation step of the Loom production chain.
fn simulate_loom_step(loom_state: &mut loom::LoomState, delta_seconds: f64) {
    // Staggered unlock (legacy, currently no-op).
    if loom_state.persistent.second_node_unlock_elapsed.is_some() {
        loom::tick_loom_staggered_unlock(loom_state, delta_seconds);
    }

    // Shuttle construction.
    loom::tick_shuttle_construction(loom_state);

    // Shuttle direct-pull processing.
    let shuttle_produced = loom::tick_shuttle_pull(loom_state, delta_seconds);

    // Base extractor production.
    let mut produced = loom::tick_base_production(loom_state, delta_seconds);

    // Merge shuttle output.
    for (resource, amount) in shuttle_produced {
        *produced.entry(resource).or_insert(0.0) += amount;
    }

    // Neighbor unlocking.
    loom::tick_neighbor_unlocking(loom_state, delta_seconds);

    // Push production into rate trackers for pattern sustain.
    for (resource, &amount) in &produced {
        loom_state
            .rate_trackers
            .entry(*resource)
            .or_default()
            .push(amount);
    }

    // Read measured rates and advance pattern sustain.
    let rates: std::collections::HashMap<loom::Resource, f64> = loom_state
        .rate_trackers
        .iter()
        .map(|(resource, tracker)| (*resource, tracker.rate_per_hour()))
        .collect();
    loom::tick_pattern_sustain(&mut loom_state.persistent, &rates, delta_seconds);
}
