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
        &mut deep.session,
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
        &mut deep.session,
        &deep.persistent,
        now,
        &mut rng,
    );

    crate::deep::missions::run_softlock_safeguards(
        &mut deep.session,
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

    // Set pending milestones for tick pipeline consumption (mirrors Deep's pending_fracture_region_unlock)
    let milestones: Vec<loom::PatternMilestone> = ((patterns_before + 1)..=patterns_after)
        .filter_map(loom::PatternMilestone::from_count)
        .collect();
    loom_state.persistent.pending_pattern_milestones = milestones;

    // Force UI graph rebuild after offline changes (neighbor unlocks, construction completions).
    loom_state.graph_dirty = true;

    Some(LoomOfflineReport { patterns_completed })
}

/// Run one simulation step of the Loom production chain.
fn simulate_loom_step(loom_state: &mut loom::LoomState, delta_seconds: f64) {
    // Shuttle construction.
    loom::tick_shuttle_construction(loom_state, delta_seconds);

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

    // Compute instantaneous rates directly for pattern sustain.
    // RateTracker assumes each push() = one 100ms tick (TICKS_PER_HOUR = 36,000),
    // but offline steps are 10 seconds — pushing raw amounts would inflate rates 100x.
    // Instead, derive rates from production amounts and step duration.
    let mut rates = std::collections::HashMap::new();
    if delta_seconds > 0.0 {
        for (resource, &amount) in &produced {
            rates.insert(*resource, amount / delta_seconds * 3600.0);
        }
    }
    loom::tick_pattern_sustain(&mut loom_state.persistent, &rates, delta_seconds);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that offline Loom simulation computes correct production rates.
    ///
    /// Before the fix, RateTracker was used with 10-second steps but assumed 100ms
    /// ticks, inflating rates 100x and causing patterns to complete instantly.
    #[test]
    fn test_offline_loom_rates_not_inflated() {
        let mut loom_state = loom::LoomState::new();
        loom::discovery::complete_discovery(&mut loom_state);

        // First pattern: Ember at 25.0/hr sustained for 2 hours (7200 seconds).
        // Extractors start at base rate 25/hr at level 1.
        assert!(!loom_state.persistent.patterns.is_empty());
        let sustain_secs = loom_state.persistent.patterns[0].requirements[0].sustain_duration_secs;
        assert!(
            sustain_secs > 3600.0,
            "First pattern requires hours of sustained production, got {sustain_secs}s"
        );
        assert!(!loom_state.persistent.patterns[0].completed);

        // Simulate 60 seconds offline — far too short to complete a 2-hour sustain.
        // Before the fix, inflated rates would cause instant completion.
        let report = resolve_loom_offline(&mut loom_state, 60);
        assert!(report.is_some());

        // Pattern should NOT complete after only 60 seconds (needs 2 hours).
        assert!(
            !loom_state.persistent.patterns[0].completed,
            "Pattern should not complete after only 60s offline (needs {}s)",
            sustain_secs
        );
        assert_eq!(report.unwrap().patterns_completed, 0);

        // But the sustain timer should have advanced by ~60 seconds.
        let sustained = loom_state.persistent.patterns[0].requirements[0].sustained_secs;
        assert!(
            sustained > 50.0 && sustained <= 60.0,
            "Sustain timer should advance ~60s, got {sustained}s"
        );
    }

    /// Verify that offline Loom simulation completes patterns given enough time.
    #[test]
    fn test_offline_loom_completes_with_enough_time() {
        let mut loom_state = loom::LoomState::new();
        loom::discovery::complete_discovery(&mut loom_state);

        // Simulate 3 hours offline — enough to complete the first pattern (2h sustain).
        let report = resolve_loom_offline(&mut loom_state, 3 * 3600);
        assert!(report.is_some());
        assert!(
            loom_state.persistent.patterns[0].completed,
            "First pattern should complete after 3 hours offline"
        );
        assert!(report.unwrap().patterns_completed >= 1);
    }

    /// Verify that offline Loom simulation does NOT complete patterns when production
    /// rate is below the threshold.
    #[test]
    fn test_offline_loom_no_false_completion() {
        let mut loom_state = loom::LoomState::new();
        loom::discovery::complete_discovery(&mut loom_state);

        // Lock the Ember extractor (EmberSpindle = index 0) so it doesn't produce.
        // The first pattern requires Ember at 25/hr, so it should NOT complete.
        loom_state.persistent.nodes[0].upgrading = true;

        // Even with 3 hours, pattern should not complete without Ember production.
        let report = resolve_loom_offline(&mut loom_state, 3 * 3600);
        assert!(report.is_some());

        assert!(
            !loom_state.persistent.patterns[0].completed,
            "Pattern should not complete when required extractor is locked"
        );
        assert_eq!(report.unwrap().patterns_completed, 0);
    }
}
