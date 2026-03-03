//! Prestige and fishing panel rendering helpers for the stats panel.

use crate::achievements::{get_achievements_by_category, AchievementCategory};
use crate::deep::DeepState;
use crate::power_cores::{fill_duration_secs, fill_ratio, ALL_POWER_CORES};
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

pub(super) fn prestige_ready_gauge_style() -> Style {
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
        .bg(Color::Rgb(25, 15, 35))
        .add_modifier(Modifier::BOLD)
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

/// Draws the unified Deep panel: guild rank, missions, crew, and power core status.
///
/// Shows when The Deep is discovered. 8 rows total (6 content + 2 border).
pub(super) fn draw_deep_panel(
    frame: &mut Frame,
    area: Rect,
    achievements: &crate::achievements::Achievements,
    deep: &DeepState,
) {
    const AMBER: Color = Color::Rgb(220, 180, 60);
    const CORE_AMBER: Color = Color::Rgb(255, 165, 0);

    if !deep.persistent.discovered {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" The Deep ")
        .border_style(Style::default().fg(super::themed_border_color(CORE_AMBER)));
    let inner = super::render_themed_block(frame, area, block, CORE_AMBER, super::BorderFxContext);

    let mut lines: Vec<Line> = Vec::new();
    let width = inner.width as usize;

    // Row 1: Guild rank + Warband Marks
    {
        let dw = super::scene_fx::display_width;
        let rank_name = deep.persistent.guild_rank.display_name();
        let marks = deep.prestige.warband_marks;
        let marks_str = format!("\u{25c6} {} Warband Marks", marks);
        let rank_part = format!("\u{2b21} {}", rank_name);
        let padding = width.saturating_sub(dw(&rank_part) + dw(&marks_str));

        lines.push(Line::from(vec![
            Span::styled("\u{2b21} ", Style::default().fg(Color::White)),
            Span::styled(rank_name.to_string(), Style::default().fg(Color::White)),
            Span::raw(" ".repeat(padding)),
            Span::styled("\u{25c6} ", Style::default().fg(AMBER)),
            Span::styled(
                format!("{} Warband Marks", marks),
                Style::default().fg(AMBER),
            ),
        ]));
    }

    // Row 2: Missions + progress bar + ETA + events badge
    {
        let dw = super::scene_fx::display_width;
        let active = deep.prestige.active_mission_count();
        let max_concurrent = crate::deep::effective_concurrent_missions(
            deep.persistent.guild_rank,
            deep.persistent.deepest_layer_reached,
        );
        let events = pending_event_count(&deep.prestige);

        let mut left_spans: Vec<Span> = Vec::new();
        let mut right_spans: Vec<Span> = Vec::new();

        let msn_str = format!("Missions {}/{} ", active, max_concurrent);
        left_spans.push(Span::styled(msn_str, Style::default().fg(Color::Cyan)));

        // Find the nearest active mission for the progress bar
        let now_chrono = chrono::Utc::now();
        let nearest_mission = deep
            .prestige
            .active_missions
            .iter()
            .filter(|m| {
                matches!(
                    m.status,
                    crate::deep::MissionStatus::Active | crate::deep::MissionStatus::EventPending
                )
            })
            .min_by_key(|m| m.ends_at);

        let eta = next_mission_eta_secs(&deep.prestige);

        if let Some(mission) = nearest_mission {
            // Build 12-char progress bar [████████░░░░]
            let progress = mission.progress(now_chrono);
            let filled = (progress * 12.0).round() as usize;
            let empty = 12 - filled;
            left_spans.push(Span::styled("[", Style::default().fg(Color::DarkGray)));
            if filled > 0 {
                left_spans.push(Span::styled(
                    "\u{2588}".repeat(filled),
                    Style::default().fg(CORE_AMBER),
                ));
            }
            if empty > 0 {
                left_spans.push(Span::styled(
                    "\u{2591}".repeat(empty),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            left_spans.push(Span::styled("]", Style::default().fg(Color::DarkGray)));

            // ETA text (right-aligned)
            if let Some(secs) = eta {
                let eta_color = if secs < 900 {
                    Color::Yellow
                } else {
                    Color::DarkGray
                };
                right_spans.push(Span::styled(
                    format_eta(secs as u64),
                    Style::default().fg(eta_color),
                ));
            }
        } else {
            // No active missions: idle state (right-aligned)
            right_spans.push(Span::styled(
                "\u{25f7} idle",
                Style::default().fg(Color::DarkGray),
            ));
        }

        // Events badge (right-aligned, after ETA/idle)
        if events > 0 {
            right_spans.push(Span::styled(
                format!("  \u{26a1}{}", events),
                Style::default().fg(Color::Yellow),
            ));
        }

        // Combine with padding
        let left_width: usize = left_spans.iter().map(|s| dw(&s.content)).sum();
        let right_width: usize = right_spans.iter().map(|s| dw(&s.content)).sum();
        let padding = width.saturating_sub(left_width + right_width);

        let mut spans = left_spans;
        spans.push(Span::raw(" ".repeat(padding)));
        spans.extend(right_spans);

        lines.push(Line::from(spans));
    }

    // Row 3: Team glyphs + Frontier
    {
        let mut team_spans: Vec<Span> = Vec::new();
        let mut available_count: usize = 0;
        let mut on_mission_count: usize = 0;
        let mut injured_count: usize = 0;

        for merc in &deep.prestige.roster {
            match merc.status {
                crate::deep::MercStatus::Available => available_count += 1,
                crate::deep::MercStatus::OnMission(_) => on_mission_count += 1,
                crate::deep::MercStatus::Injured { .. } => injured_count += 1,
                crate::deep::MercStatus::Lost => {} // skip
            }
        }

        let has_team = available_count > 0 || on_mission_count > 0 || injured_count > 0;
        if has_team {
            team_spans.push(Span::styled("Team ", Style::default().fg(Color::DarkGray)));
        }

        // Available mercs: ♦ (green)
        if available_count > 0 {
            team_spans.push(Span::styled(
                "\u{2666}".repeat(available_count),
                Style::default().fg(Color::Green),
            ));
        }
        // Space between groups
        if available_count > 0 && (on_mission_count > 0 || injured_count > 0) {
            team_spans.push(Span::raw(" "));
        }
        // On mission: ♢ (cyan)
        if on_mission_count > 0 {
            team_spans.push(Span::styled(
                "\u{2662}".repeat(on_mission_count),
                Style::default().fg(Color::Cyan),
            ));
        }
        if on_mission_count > 0 && injured_count > 0 {
            team_spans.push(Span::raw(" "));
        }
        // Injured: ✝ (red)
        if injured_count > 0 {
            team_spans.push(Span::styled(
                "\u{271d}".repeat(injured_count),
                Style::default().fg(Color::Red),
            ));
        }

        let team_width: usize = if has_team { 5 } else { 0 } // "Team "
            + available_count
            + on_mission_count
            + injured_count
            + if available_count > 0 && (on_mission_count > 0 || injured_count > 0) {
                1
            } else {
                0
            }
            + if on_mission_count > 0 && injured_count > 0 {
                1
            } else {
                0
            };

        // Right side: Frontier only
        let frontier = deep.persistent.frontier_layer();
        let frontier_str = format!("Frontier L{}", frontier);
        let right_str_len = frontier_str.len();

        let padding = width.saturating_sub(team_width + right_str_len);
        team_spans.push(Span::raw(" ".repeat(padding)));
        team_spans.push(Span::styled(
            frontier_str,
            Style::default().fg(Color::Rgb(120, 140, 170)),
        ));

        lines.push(Line::from(team_spans));
    }

    // Render header lines (rows 1-3) as a Paragraph in the top section,
    // then render cores in a bordered sub-block below.
    let summary = core_summary(achievements, deep);
    let core_count = summary.cores.len() as u16;

    // Split inner into: header (3 lines) + cores sub-block (remaining)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(inner);

    // Render header lines
    let para = Paragraph::new(lines);
    frame.render_widget(para, chunks[0]);

    // Render "Power Cores" bordered sub-block
    let total_pr_per_day: u32 = summary
        .cores
        .iter()
        .filter(|c| c.unlocked)
        .map(|c| c.pr_per_day)
        .sum();
    let pr_title = format!(" +{} PR/d ", total_pr_per_day);
    let cores_block = Block::default()
        .borders(Borders::ALL)
        .title(" Power Cores ")
        .border_style(Style::default().fg(CORE_AMBER))
        .title_bottom(Line::from(pr_title).right_aligned());
    let cores_inner = cores_block.inner(chunks[1]);
    frame.render_widget(cores_block, chunks[1]);

    // Split cores_inner into N gauge rows
    let mut gauge_constraints = Vec::new();
    for _ in 0..core_count {
        gauge_constraints.push(Constraint::Length(1));
    }
    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(gauge_constraints)
        .split(cores_inner);

    // Render each core as a Gauge
    for (i, badge) in summary.cores.iter().enumerate() {
        let gauge_area = gauge_chunks[i];
        if badge.unlocked {
            if badge.ready {
                let label = format!(
                    "{} Power Core (+{} PR/d) \u{00b7} \u{2713} Ready",
                    badge.name, badge.pr_per_day
                );
                let gauge = Gauge::default()
                    .gauge_style(
                        Style::default()
                            .fg(Color::Green)
                            .bg(Color::Rgb(8, 28, 8))
                            .add_modifier(Modifier::BOLD),
                    )
                    .label(label)
                    .ratio(1.0);
                frame.render_widget(gauge, gauge_area);
            } else {
                let time_str = format_core_time_short(badge.remaining_secs);
                let label = format!(
                    "{} Power Core (+{} PR/d) \u{00b7} {}",
                    badge.name, badge.pr_per_day, time_str
                );
                let gauge = Gauge::default()
                    .gauge_style(
                        Style::default()
                            .fg(CORE_AMBER)
                            .bg(Color::Rgb(28, 18, 0))
                            .add_modifier(Modifier::BOLD),
                    )
                    .label(label)
                    .ratio(badge.fill_ratio);
                frame.render_widget(gauge, gauge_area);
            }
        } else {
            let label = format!(
                "{} Power Core (+{} PR/d) \u{00b7} Locked (L{})",
                badge.name, badge.pr_per_day, badge.required_layer
            );
            let gauge = Gauge::default()
                .gauge_style(
                    Style::default()
                        .fg(Color::DarkGray)
                        .bg(Color::Rgb(15, 15, 15))
                        .add_modifier(Modifier::BOLD),
                )
                .label(label)
                .ratio(0.0);
            frame.render_widget(gauge, gauge_area);
        }
    }
}

/// Returns seconds until the next active mission completes, or None if no active missions.
fn next_mission_eta_secs(prestige: &crate::deep::DeepPrestige) -> Option<i64> {
    let now = chrono::Utc::now();
    prestige
        .active_missions
        .iter()
        .filter(|m| {
            matches!(
                m.status,
                crate::deep::MissionStatus::Active | crate::deep::MissionStatus::EventPending
            )
        })
        .map(|m| (m.ends_at - now).num_seconds().max(0))
        .min()
}

/// Count of active missions with pending events needing player response.
fn pending_event_count(prestige: &crate::deep::DeepPrestige) -> usize {
    prestige
        .active_missions
        .iter()
        .filter(|m| m.has_pending_event())
        .count()
}

struct CoreSummary {
    ready_count: usize,
    ready_pr: u32,
    unlocked_count: usize,
    total_pr_per_day: u32,
    next_ready_secs: Option<i64>,
    next_ready_ratio: Option<f64>,
    cores: Vec<CoreBadge>,
}

struct CoreBadge {
    name: &'static str,
    unlocked: bool,
    ready: bool,
    fill_ratio: f64,
    required_layer: u32,
    pr_per_day: u32,
    remaining_secs: i64,
}

fn format_core_time_short(secs: i64) -> String {
    let secs = secs.max(0) as u64;
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if hours > 0 {
        format!("{}h {:02}m {:02}s", hours, minutes, seconds)
    } else {
        format!("{}m {:02}s", minutes.max(1), seconds)
    }
}

fn core_summary(
    achievements: &crate::achievements::Achievements,
    deep: &crate::deep::DeepState,
) -> CoreSummary {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let mut summary = CoreSummary {
        ready_count: 0,
        ready_pr: 0,
        unlocked_count: 0,
        total_pr_per_day: 0,
        next_ready_secs: None,
        next_ready_ratio: None,
        cores: Vec::new(),
    };

    for core in ALL_POWER_CORES {
        let is_unlocked = achievements.is_unlocked(core.achievement_id);

        if is_unlocked {
            summary.unlocked_count += 1;
            summary.total_pr_per_day += core.pr_per_day;

            let fill_secs = fill_duration_secs(core.pr_per_day);
            let last_granted = deep
                .persistent
                .power_core_last_granted
                .get(&core.achievement_id)
                .copied()
                .unwrap_or(0);
            let elapsed = (now - last_granted).max(0);
            let ratio = fill_ratio(elapsed, fill_secs);
            let remaining = (fill_secs - elapsed).max(0);

            if ratio >= 1.0 {
                summary.ready_count += 1;
                summary.ready_pr += core.pr_per_day;
                summary.cores.push(CoreBadge {
                    name: core.name,
                    unlocked: true,
                    ready: true,
                    fill_ratio: 1.0,
                    required_layer: core.required_layer,
                    pr_per_day: core.pr_per_day,
                    remaining_secs: 0,
                });
            } else {
                if let Some(current_next) = summary.next_ready_secs {
                    if remaining < current_next {
                        summary.next_ready_secs = Some(remaining);
                        summary.next_ready_ratio = Some(ratio);
                    }
                } else {
                    summary.next_ready_secs = Some(remaining);
                    summary.next_ready_ratio = Some(ratio);
                }
                summary.cores.push(CoreBadge {
                    name: core.name,
                    unlocked: true,
                    ready: false,
                    fill_ratio: ratio,
                    required_layer: core.required_layer,
                    pr_per_day: core.pr_per_day,
                    remaining_secs: remaining,
                });
            }
        } else {
            summary.cores.push(CoreBadge {
                name: core.name,
                unlocked: false,
                ready: false,
                fill_ratio: 0.0,
                required_layer: core.required_layer,
                pr_per_day: core.pr_per_day,
                remaining_secs: 0,
            });
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::{AchievementId, Achievements};

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

    #[test]
    fn test_next_mission_eta_no_missions() {
        let prestige = crate::deep::DeepPrestige::default();
        assert_eq!(next_mission_eta_secs(&prestige), None);
    }

    #[test]
    fn test_pending_event_count_no_events() {
        let prestige = crate::deep::DeepPrestige::default();
        assert_eq!(pending_event_count(&prestige), 0);
    }

    #[test]
    fn test_core_summary_no_cores() {
        let achievements = Achievements::default();
        let deep = crate::deep::DeepState::default();
        let summary = core_summary(&achievements, &deep);
        assert_eq!(summary.ready_count, 0);
        assert_eq!(summary.ready_pr, 0);
        assert_eq!(summary.unlocked_count, 0);
        assert_eq!(summary.total_pr_per_day, 0);
        assert!(summary.next_ready_secs.is_none());
        assert!(summary.next_ready_ratio.is_none());
    }
}
