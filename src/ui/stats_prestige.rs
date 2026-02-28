//! Prestige and fishing panel rendering helpers for the stats panel.

use crate::achievements::{get_achievements_by_category, AchievementCategory, AchievementId};
use crate::ascension::types::ascension_combat_multiplier;
use crate::character::attributes::AttributeType;
use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::{get_next_prestige_tier, get_prestige_tier};
use crate::core::game_logic::xp_for_next_level;
use crate::core::game_state::GameState;
use crate::fishing::types::{FishingState, RANK_NAMES};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the icon of the highest unlocked prestige achievement, if any.
pub(super) fn highest_prestige_badge(
    achievements: &crate::achievements::Achievements,
) -> Option<&'static str> {
    for def in get_achievements_by_category(AchievementCategory::Prestige)
        .into_iter()
        .rev()
    {
        if achievements.is_unlocked(def.id) {
            return Some(def.icon);
        }
    }
    None
}

/// Returns the icon of the highest unlocked fishing rank achievement, if any.
pub(super) fn highest_fishing_badge(
    achievements: &crate::achievements::Achievements,
) -> Option<&'static str> {
    use crate::achievements::AchievementId;

    let fishing_achievements = [
        AchievementId::FishermanIV,
        AchievementId::FishermanIII,
        AchievementId::FishermanII,
        AchievementId::FishermanI,
    ];

    for id in fishing_achievements {
        if achievements.is_unlocked(id) {
            return crate::achievements::data::get_achievement_def(id).map(|def| def.icon);
        }
    }
    None
}

/// Formats seconds into a human-readable ETA (e.g., "~3m", "~1h 20m", "~2d 5h").
pub(super) fn format_eta(seconds: u64) -> String {
    if seconds < 60 {
        return "~<1m".to_string();
    }
    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("~{}m", minutes);
    }
    let hours = minutes / 60;
    let remaining_mins = minutes % 60;
    if hours < 24 {
        if remaining_mins > 0 {
            return format!("~{}h {}m", hours, remaining_mins);
        }
        return format!("~{}h", hours);
    }
    let days = hours / 24;
    let remaining_hours = hours % 24;
    if remaining_hours > 0 {
        format!("~{}d {}h", days, remaining_hours)
    } else {
        format!("~{}d", days)
    }
}

fn current_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn prestige_ready_gauge_style() -> Style {
    const PULSE_COLORS: [Color; 5] = [
        Color::Rgb(190, 118, 255),
        Color::Rgb(208, 138, 255),
        Color::Rgb(228, 168, 245),
        Color::Rgb(248, 198, 185),
        Color::Rgb(255, 221, 120),
    ];
    let idx = ((current_millis() / 220) % PULSE_COLORS.len() as u128) as usize;
    Style::default()
        .fg(PULSE_COLORS[idx])
        .add_modifier(Modifier::BOLD)
}

/// Draws prestige information with CHA bonus
pub(super) fn draw_prestige_info(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    achievements: &crate::achievements::Achievements,
) {
    let title = match highest_prestige_badge(achievements) {
        Some(icon) => format!(" Prestige {} ", icon),
        None => " Prestige ".to_string(),
    };
    let prestige_block = Block::default().borders(Borders::ALL).title(title);
    let prestige_block = super::themed_block(prestige_block);

    let inner = prestige_block.inner(area);
    frame.render_widget(prestige_block, area);
    super::apply_themed_border_fx(frame, area, Color::White, super::BorderFxContext);

    let tier = get_prestige_tier(game_state.prestige_rank);
    let cha_mod = game_state.attributes.modifier(AttributeType::Charisma);
    let effective_multiplier =
        DerivedStats::prestige_multiplier(tier.multiplier, &game_state.attributes);

    // Build unlock hint for next prestige rank
    let next_prestige = get_next_prestige_tier(game_state.prestige_rank);
    let mult_delta = next_prestige.multiplier - tier.multiplier;
    let unlock_hint = {
        let mut unlocks = Vec::new();
        // Check which zones unlock at the next prestige rank
        let zones = crate::zones::get_all_zones();
        for zone in zones {
            if zone.prestige_requirement == next_prestige.rank {
                unlocks.push(zone.name);
            }
        }
        // Check for system unlocks at specific prestige gates
        if next_prestige.rank == 10 {
            unlocks.push("Haven");
        }
        if next_prestige.rank == 15 {
            unlocks.push("Soulforge");
            unlocks.push("Stormglass");
        }
        if unlocks.is_empty() {
            format!("\u{1f513} +{:.2}x mult", mult_delta)
        } else {
            format!(
                "\u{1f513} +{:.2}x mult \u{00b7} Unlocks: {}",
                mult_delta,
                unlocks.join(", ")
            )
        }
    };

    let prestige_text = vec![
        Line::from({
            let mut rank_spans = vec![
                Span::styled("🏆 Rank: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{} ({})", game_state.prestige_rank, tier.name),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                Span::styled("🔄 ", Style::default()),
                Span::styled(
                    format!("{}", game_state.total_prestige_count),
                    Style::default().fg(Color::Magenta),
                ),
            ];
            if game_state.ascension_level > 0 {
                let asc_mult = ascension_combat_multiplier(game_state.ascension_level);
                rank_spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
                rank_spans.push(Span::styled(
                    format!(
                        "Asc {} ({:.1}x)",
                        to_roman(game_state.ascension_level),
                        asc_mult
                    ),
                    Style::default()
                        .fg(Color::Rgb(255, 215, 0))
                        .add_modifier(Modifier::BOLD),
                ));
            }
            rank_spans
        }),
        Line::from({
            let mut spans = vec![
                Span::styled("⚡ XP: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{:.2}x", tier.multiplier),
                    Style::default().fg(Color::Cyan),
                ),
            ];
            if cha_mod != 0 {
                spans.push(Span::styled(
                    format!(" +{:.1} CHA", cha_mod as f64 * 0.1),
                    Style::default().fg(Color::Yellow),
                ));
            }
            spans.push(Span::styled(
                " \u{2192} ",
                Style::default().fg(Color::DarkGray),
            ));
            spans.push(Span::styled(
                format!("{:.2}x", effective_multiplier),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ));
            spans
        }),
        Line::from(Span::styled(
            unlock_hint,
            Style::default().fg(Color::DarkGray),
        )),
    ];

    // Prestige level progress bar (next_prestige already computed above for unlock hint)
    let prestige_ratio =
        (game_state.character_level as f64 / next_prestige.required_level as f64).min(1.0);
    let prestige_eta = match game_state.xp_per_hour() {
        Some(rate) if rate > 0 && game_state.character_level < next_prestige.required_level => {
            // Sum XP needed from current level to prestige required level
            let xp_remaining_current = xp_for_next_level(game_state.character_level)
                .saturating_sub(game_state.character_xp);
            let xp_future_levels: u64 = (game_state.character_level + 1
                ..next_prestige.required_level)
                .map(xp_for_next_level)
                .sum();
            let total_xp = xp_remaining_current + xp_future_levels;
            let seconds = (total_xp as f64 / rate as f64 * 3600.0) as u64;
            format!(" ({})", format_eta(seconds))
        }
        _ => String::new(),
    };
    let prestige_label = format!(
        "Lv {}/{} to {} (P{}){}",
        game_state.character_level,
        next_prestige.required_level,
        next_prestige.name,
        next_prestige.rank,
        prestige_eta
    );
    let prestige_ready = game_state.character_level >= next_prestige.required_level;
    let prestige_gauge = Gauge::default()
        .gauge_style(if prestige_ready {
            prestige_ready_gauge_style()
        } else {
            Style::default()
                .fg(Color::Rgb(180, 100, 255))
                .add_modifier(Modifier::BOLD)
        })
        .label(prestige_label)
        .ratio(prestige_ratio);

    if inner.height >= 4 {
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

        frame.render_widget(Paragraph::new(prestige_text[0].clone()), inner_chunks[0]);
        frame.render_widget(Paragraph::new(prestige_text[1].clone()), inner_chunks[1]);
        frame.render_widget(Paragraph::new(prestige_text[2].clone()), inner_chunks[2]);
        frame.render_widget(prestige_gauge, inner_chunks[3]);
    } else if inner.height >= 3 {
        // Fallback: skip unlock hint if not enough space
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

        frame.render_widget(Paragraph::new(prestige_text[0].clone()), inner_chunks[0]);
        frame.render_widget(Paragraph::new(prestige_text[1].clone()), inner_chunks[1]);
        frame.render_widget(prestige_gauge, inner_chunks[2]);
    } else {
        // Show as many text lines as fit, rank first
        let lines_to_show = inner.height as usize;
        let truncated: Vec<Line> = prestige_text.into_iter().take(lines_to_show).collect();
        let prestige_paragraph = Paragraph::new(truncated);
        frame.render_widget(prestige_paragraph, inner);
    }
}

/// Draws the fishing panel with rank and progress bar.
pub(super) fn draw_fishing_panel(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    achievements: &crate::achievements::Achievements,
) {
    let title = match highest_fishing_badge(achievements) {
        Some(icon) => format!(" Fishing {} ", icon),
        None => " Fishing ".to_string(),
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let block = super::themed_block(block);

    let inner = block.inner(area);
    frame.render_widget(block, area);
    super::apply_themed_border_fx(frame, area, Color::White, super::BorderFxContext);

    let fish_required = FishingState::fish_required_for_rank(game_state.fishing.rank);
    let fish_progress = game_state.fishing.fish_toward_next_rank;
    let fish_ratio = if fish_required > 0 {
        (fish_progress as f64 / fish_required as f64).min(1.0)
    } else {
        0.0
    };

    let rank_line = Line::from(vec![
        Span::styled("🎣 Rank: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            format!(
                "{} ({})",
                game_state.fishing.rank_name(),
                game_state.fishing.rank
            ),
            Style::default().fg(Color::Cyan),
        ),
    ]);

    let is_max_rank = game_state.fishing.rank as usize >= RANK_NAMES.len();
    let leviathan_caught = achievements.is_unlocked(AchievementId::StormLeviathan);
    let show_leviathan_hunt = is_max_rank && game_state.fishing.rank >= 40 && !leviathan_caught;
    let show_leviathan_trophy = is_max_rank && game_state.fishing.rank >= 40 && leviathan_caught;

    if inner.height >= 2 {
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

        if show_leviathan_hunt {
            // Row 1: "Storm Leviathan Hunt" instead of rank
            let hunt_line = Line::from(vec![Span::styled(
                "🐋 Storm Leviathan Hunt",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]);
            frame.render_widget(Paragraph::new(hunt_line), inner_chunks[0]);
            // Row 2: tracker with dots + charge bar
            let tracker_line = build_leviathan_tracker_line(&game_state.fishing);
            frame.render_widget(Paragraph::new(tracker_line), inner_chunks[1]);
        } else if show_leviathan_trophy {
            // Row 1: rank name
            frame.render_widget(Paragraph::new(rank_line), inner_chunks[0]);
            // Row 2: all dots filled + "Storm Leviathan ✓"
            let trophy_line = build_leviathan_trophy_line();
            frame.render_widget(Paragraph::new(trophy_line), inner_chunks[1]);
        } else {
            let fish_label = if is_max_rank {
                "Max Rank".to_string()
            } else {
                let next_rank = game_state.fishing.rank + 1;
                let next_rank_name = RANK_NAMES[next_rank as usize - 1];
                format!(
                    "{}/{} to {} ({})",
                    fish_progress, fish_required, next_rank_name, next_rank
                )
            };
            let fish_ratio = if is_max_rank { 1.0 } else { fish_ratio };
            let fish_gauge = Gauge::default()
                .gauge_style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
                .label(fish_label)
                .ratio(fish_ratio);
            frame.render_widget(fish_gauge, inner_chunks[1]);
        }
    } else if inner.height >= 1 {
        // Only room for one line — show rank
        let rank_paragraph = Paragraph::new(rank_line);
        frame.render_widget(rank_paragraph, inner);
    }
}

/// Builds the Leviathan hunt tracker line for the fishing panel at rank 40.
///
/// Layout: `🐋 ● ● ● ● ○ ○ ○ ○ ○ ○   ⚡ ▰▰▰▱▱▱▱ +6.5%`
fn build_leviathan_tracker_line(fishing: &FishingState) -> Line<'static> {
    let encounters = fishing.leviathan_encounters.min(10) as usize;
    let in_catch_phase = encounters >= 10;

    let mut spans: Vec<Span<'static>> = Vec::new();

    // Whale prefix
    spans.push(Span::styled(
        "\u{1F40B} ",
        Style::default().add_modifier(Modifier::BOLD),
    ));

    // Encounter progress dots
    for i in 0..10 {
        if i < encounters {
            spans.push(Span::styled("\u{25CF} ", Style::default().fg(Color::Cyan)));
        } else {
            spans.push(Span::styled(
                "\u{25CB} ",
                Style::default().fg(Color::DarkGray),
            ));
        }
    }

    // Separator
    spans.push(Span::raw("  "));

    // Lightning bolt
    spans.push(Span::styled(
        "\u{26A1} ",
        Style::default()
            .fg(Color::Rgb(100, 180, 255))
            .add_modifier(Modifier::BOLD),
    ));

    if !fishing.storm_lure_active {
        spans.push(Span::styled("--", Style::default().fg(Color::DarkGray)));
    } else {
        // Charge bar: 7 ticks representing miss_ramp 0-10%
        let filled = (fishing.lure_miss_ramp / 0.10 * 7.0).round() as usize;
        let filled = filled.min(7);
        for i in 0..7 {
            if i < filled {
                spans.push(Span::styled(
                    "\u{25B0}",
                    Style::default().fg(Color::Rgb(100, 180, 255)),
                ));
            } else {
                spans.push(Span::styled(
                    "\u{25B1}",
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }
        spans.push(Span::raw(" "));

        // Percentage
        let bonus = fishing.lure_tracking_bonus + fishing.lure_miss_ramp;
        if in_catch_phase {
            // Show total catch chance (base 25% + bonus)
            let catch_pct = (0.25 + bonus) * 100.0;
            spans.push(Span::styled(
                format!("{:.1}%", catch_pct),
                Style::default()
                    .fg(Color::Rgb(100, 180, 255))
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            // Show lure bonus added to encounter chance
            spans.push(Span::styled(
                format!("+{:.1}%", bonus * 100.0),
                Style::default().fg(Color::Rgb(100, 180, 255)),
            ));
        }
    }

    Line::from(spans)
}

/// Builds the Leviathan trophy line shown after the Leviathan has been caught.
///
/// Layout: `🐋 ● ● ● ● ● ● ● ● ● ●   Storm Leviathan ✓`
fn build_leviathan_trophy_line() -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();

    // Whale prefix
    spans.push(Span::styled(
        "\u{1F40B} ",
        Style::default().add_modifier(Modifier::BOLD),
    ));

    // All 10 dots filled
    for _ in 0..10 {
        spans.push(Span::styled("\u{25CF} ", Style::default().fg(Color::Cyan)));
    }

    spans.push(Span::raw("  "));
    spans.push(Span::styled(
        "Storm Leviathan \u{2713}",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    ));

    Line::from(spans)
}

/// Converts a small positive integer to a Roman numeral string.
pub(crate) fn to_roman(n: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    const VALS: [(u32, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut result = String::new();
    let mut remaining = n;
    for &(val, sym) in &VALS {
        while remaining >= val {
            result.push_str(sym);
            remaining -= val;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::Achievements;

    #[test]
    fn test_highest_prestige_badge_uses_existing_top_rank() {
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::PrestigeX, Some("Hero".to_string()));
        achievements.unlock(AchievementId::Eternal, Some("Hero".to_string()));

        assert_eq!(highest_prestige_badge(&achievements), Some("♾️"));
    }

    #[test]
    fn test_highest_prestige_badge_uses_new_capstone() {
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::Eternal, Some("Hero".to_string()));
        achievements.unlock(AchievementId::Prestige10000, Some("Hero".to_string()));

        assert_eq!(highest_prestige_badge(&achievements), Some("🔱"));
    }

    #[test]
    fn test_to_roman() {
        assert_eq!(to_roman(1), "I");
        assert_eq!(to_roman(3), "III");
        assert_eq!(to_roman(4), "IV");
        assert_eq!(to_roman(6), "VI");
        assert_eq!(to_roman(7), "VII");
        assert_eq!(to_roman(9), "IX");
        assert_eq!(to_roman(10), "X");
    }

    #[test]
    fn test_to_roman_zero() {
        assert_eq!(to_roman(0), "0");
    }
}
