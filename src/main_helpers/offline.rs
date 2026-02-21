//! Offline XP processing.

use crate::core::game_logic::{process_offline_progression, OfflineReport};
use crate::core::game_state::GameState;
use crate::haven;

/// Process offline XP and add combat log entries. Returns the report if XP was gained.
pub fn apply_offline_xp(state: &mut GameState, haven: &haven::Haven) -> Option<OfflineReport> {
    let haven_offline_bonus = haven.get_bonus(haven::HavenBonusType::OfflineXpPercent);
    let report = process_offline_progression(&mut rand::rng(), state, haven_offline_bonus);
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
