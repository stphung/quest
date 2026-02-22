//! Achievement browser detail panel and stats view rendering.

use crate::achievements::{
    get_achievements_by_category, AchievementCategory, AchievementId, Achievements,
};
use crate::character::prestige::get_prestige_tier;
use crate::enhancement::EnhancementProgress;
use crate::fishing::types::fishing_tier_name;
use crate::zones::get_all_zones;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::achievement_browser_scene::AchievementBrowserState;

/// Format a number with commas (e.g., 12847 -> "12,847").
pub(super) fn format_number(n: u64) -> String {
    if n < 1000 {
        return n.to_string();
    }
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Create a stat line with dot-leaders: "  Label .... Value"
fn stat_line(
    label: &str,
    value: &str,
    label_style: Style,
    value_style: Style,
    width: u16,
) -> Line<'static> {
    let w = width as usize;
    let label_len = label.len() + 2; // "  Label"
    let value_len = value.len();
    let dots_len = w.saturating_sub(label_len + value_len + 2);
    let dots = ".".repeat(dots_len.max(1));

    Line::from(vec![
        Span::styled(format!("  {label} "), label_style),
        Span::styled(dots, Style::default().fg(Color::DarkGray)),
        Span::styled(format!(" {value}"), value_style),
    ])
}

/// Render a list of lines into an area with a scroll offset.
fn render_lines(frame: &mut Frame, area: Rect, lines: Vec<Line>, scroll: usize) {
    let visible: Vec<Line> = lines.into_iter().skip(scroll).collect();
    let para = Paragraph::new(visible);
    frame.render_widget(para, area);
}

/// Render the achievement detail panel for a selected achievement.
pub(super) fn render_achievement_detail(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    ui_state: &AchievementBrowserState,
) {
    let category_achievements = get_achievements_by_category(ui_state.selected_category);

    let Some(def) = category_achievements.get(ui_state.selected_index) else {
        return;
    };

    let is_unlocked = achievements.is_unlocked(def.id);
    let border_color = if is_unlocked {
        Color::Green
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .title(format!(" {} ", def.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(border_color)));
    let inner =
        super::render_themed_block(frame, area, block, border_color, super::BorderFxContext);

    let mut lines = Vec::new();

    // Icon and name
    lines.push(Line::from(Span::styled(
        format!("{} {}", def.icon, def.name),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Description
    lines.push(Line::from(Span::styled(
        def.description,
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(""));

    // Unlock status
    if is_unlocked {
        if let Some(record) = achievements.unlocked.get(&def.id) {
            let timestamp = chrono::DateTime::from_timestamp(record.unlocked_at, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            lines.push(Line::from(Span::styled(
                format!("[X] Unlocked: {}", timestamp),
                Style::default().fg(Color::Green),
            )));

            if let Some(ref char_name) = record.character_name {
                lines.push(Line::from(Span::styled(
                    format!("    By: {}", char_name),
                    Style::default().fg(Color::DarkGray),
                )));
            }

            // Show completed progress bar for milestone achievements
            if let Some(progress) = achievements.get_progress(def.id) {
                let display_current = progress.target;
                lines.push(Line::from(vec![
                    Span::styled("    [", Style::default().fg(Color::DarkGray)),
                    Span::styled("\u{2588}".repeat(20), Style::default().fg(Color::Green)),
                    Span::styled("] ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!(
                            "{}/{}",
                            format_number(display_current),
                            format_number(progress.target)
                        ),
                        Style::default().fg(Color::Green),
                    ),
                ]));
            }

            if achievements.is_recently_unlocked(def.id) {
                lines.push(Line::from(Span::styled(
                    "[NEW] Recently unlocked!",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            }
        }
    } else {
        lines.push(Line::from(Span::styled(
            "[ ] Not yet unlocked",
            Style::default().fg(Color::Red),
        )));

        // Show progress if applicable
        if let Some(progress) = achievements.get_progress(def.id) {
            let percent = if progress.target > 0 {
                (progress.current as f64 / progress.target as f64 * 100.0) as u32
            } else {
                0
            };
            lines.push(Line::from(Span::styled(
                format!(
                    "    Progress: {}/{}",
                    format_number(progress.current),
                    format_number(progress.target)
                ),
                Style::default().fg(Color::Yellow),
            )));

            // Progress bar
            let bar_width = 20usize;
            let filled = if progress.target > 0 {
                (progress.current as usize * bar_width / progress.target as usize).min(bar_width)
            } else {
                0
            };
            let empty = bar_width - filled;
            lines.push(Line::from(vec![
                Span::styled("    [", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "\u{2588}".repeat(filled),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    "\u{2591}".repeat(empty),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled("] ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}%", percent), Style::default().fg(Color::Yellow)),
            ]));
        }
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(para, inner);
}

/// Render the stats view (full-width, two columns).
pub(super) fn render_stats_view(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    enhancement: &EnhancementProgress,
    scroll_offset: usize,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::DarkGray)));
    let inner =
        super::render_themed_block(frame, area, block, Color::DarkGray, super::BorderFxContext);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(44),
            Constraint::Length(3),
            Constraint::Percentage(56),
        ])
        .split(inner);

    let left_lines = build_stats_left_lines(achievements, enhancement, columns[0].width);
    let right_lines = build_stats_right_lines(achievements, enhancement, columns[2].width);

    let max_content = left_lines.len().max(right_lines.len());
    let visible = inner.height as usize;
    let max_scroll = max_content.saturating_sub(visible);
    let scroll = scroll_offset.min(max_scroll);

    render_lines(frame, columns[0], left_lines, scroll);
    render_lines(frame, columns[2], right_lines, scroll);
}

/// Build the left column lines: raw stats with dot-leaders.
fn build_stats_left_lines(
    achievements: &Achievements,
    enhancement: &EnhancementProgress,
    width: u16,
) -> Vec<Line<'static>> {
    let section_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let separator_style = Style::default().fg(Color::DarkGray);
    let label_style = Style::default().fg(Color::DarkGray);
    let value_style = Style::default().fg(Color::Cyan);

    let kill_boss_ratio = if achievements.total_bosses_defeated > 0 {
        format!(
            "{}:1",
            achievements.total_kills / achievements.total_bosses_defeated
        )
    } else {
        "N/A".to_string()
    };

    let prestige_tier = get_prestige_tier(achievements.highest_prestige_rank).name;
    let fishing_tier = fishing_tier_name(achievements.highest_fishing_rank);

    let w = width as usize;

    let total_kills_str = format_number(achievements.total_kills);
    let boss_kills_str = format_number(achievements.total_bosses_defeated);
    let highest_level_str = format_number(achievements.highest_level as u64);
    let highest_prestige_str = format_number(achievements.highest_prestige_rank as u64);
    let expanse_cycles_str = format_number(achievements.expanse_cycles_completed);
    let total_fish_str = format_number(achievements.total_fish_caught);
    let highest_fishing_rank_str = format_number(achievements.highest_fishing_rank as u64);
    let dungeons_completed_str = format_number(achievements.total_dungeons_completed);
    let minigame_wins_str = format_number(achievements.total_minigame_wins);

    let total_enhancement_attempts_str = format_number(enhancement.total_attempts as u64);
    let total_enhancement_successes_str = format_number(enhancement.total_successes as u64);
    let total_enhancement_failures_str = format_number(enhancement.total_failures as u64);
    let highest_enhancement_str = format!("+{}", enhancement.highest_level_reached);

    let mut lines = vec![
        Line::from(Span::styled("COMBAT", section_style)),
        Line::from(Span::styled("\u{2500}".repeat(w), separator_style)),
        stat_line(
            "Total Kills",
            &total_kills_str,
            label_style,
            value_style,
            width,
        ),
        stat_line(
            "Boss Kills",
            &boss_kills_str,
            label_style,
            value_style,
            width,
        ),
        stat_line(
            "Kill/Boss Ratio",
            &kill_boss_ratio,
            label_style,
            value_style,
            width,
        ),
        Line::from(""),
        Line::from(Span::styled("PROGRESSION", section_style)),
        Line::from(Span::styled("\u{2500}".repeat(w), separator_style)),
        stat_line(
            "Highest Level",
            &highest_level_str,
            label_style,
            value_style,
            width,
        ),
        stat_line(
            "Highest Prestige",
            &highest_prestige_str,
            label_style,
            value_style,
            width,
        ),
        stat_line(
            "Prestige Tier",
            prestige_tier,
            label_style,
            value_style,
            width,
        ),
        stat_line(
            "Expanse Cycles",
            &expanse_cycles_str,
            label_style,
            value_style,
            width,
        ),
        Line::from(""),
        Line::from(Span::styled("FISHING", section_style)),
        Line::from(Span::styled("\u{2500}".repeat(w), separator_style)),
        stat_line(
            "Total Fish Caught",
            &total_fish_str,
            label_style,
            value_style,
            width,
        ),
        stat_line(
            "Highest Rank",
            &highest_fishing_rank_str,
            label_style,
            value_style,
            width,
        ),
        stat_line("Rank Tier", fishing_tier, label_style, value_style, width),
        Line::from(""),
        Line::from(Span::styled("DUNGEONS & CHALLENGES", section_style)),
        Line::from(Span::styled("\u{2500}".repeat(w), separator_style)),
        stat_line(
            "Dungeons Completed",
            &dungeons_completed_str,
            label_style,
            value_style,
            width,
        ),
        stat_line(
            "Minigame Wins",
            &minigame_wins_str,
            label_style,
            value_style,
            width,
        ),
    ];

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("ENHANCEMENT", section_style)));
    lines.push(Line::from(Span::styled(
        "\u{2500}".repeat(w),
        separator_style,
    )));
    lines.push(stat_line(
        "Attempts",
        &total_enhancement_attempts_str,
        label_style,
        value_style,
        width,
    ));
    lines.push(stat_line(
        "Successes",
        &total_enhancement_successes_str,
        label_style,
        value_style,
        width,
    ));
    lines.push(stat_line(
        "Failures",
        &total_enhancement_failures_str,
        label_style,
        value_style,
        width,
    ));
    lines.push(stat_line(
        "Highest Level",
        &highest_enhancement_str,
        label_style,
        value_style,
        width,
    ));

    lines
}

/// Build the right column lines: zone checklist, challenge grid, achievement summary.
fn build_stats_right_lines(
    achievements: &Achievements,
    enhancement: &EnhancementProgress,
    width: u16,
) -> Vec<Line<'static>> {
    let section_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let separator_style = Style::default().fg(Color::DarkGray);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled("ZONES CLEARED", section_style)));
    lines.push(Line::from(Span::styled(
        "\u{2500}".repeat(width as usize),
        separator_style,
    )));

    let zones = get_all_zones();
    let zone_achievements = [
        (1, AchievementId::Zone1Complete),
        (2, AchievementId::Zone2Complete),
        (3, AchievementId::Zone3Complete),
        (4, AchievementId::Zone4Complete),
        (5, AchievementId::Zone5Complete),
        (6, AchievementId::Zone6Complete),
        (7, AchievementId::Zone7Complete),
        (8, AchievementId::Zone8Complete),
        (9, AchievementId::Zone9Complete),
        (10, AchievementId::Zone10Complete),
    ];

    for pair in zone_achievements.chunks(2) {
        let mut spans = Vec::new();
        for (zone_id, achievement_id) in pair {
            let zone_name = zones
                .iter()
                .find(|z| z.id == *zone_id)
                .map(|z| z.name)
                .unwrap_or("???");
            let cleared = achievements.is_unlocked(*achievement_id);
            let (check, style) = if cleared {
                ("[X]", Style::default().fg(Color::Green))
            } else {
                ("[ ]", Style::default().fg(Color::DarkGray))
            };
            spans.push(Span::styled(format!("  {check} {zone_name:<16}"), style));
        }
        lines.push(Line::from(spans));
    }

    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "CHALLENGES MASTERED",
        section_style,
    )));
    lines.push(Line::from(Span::styled(
        "\u{2500}".repeat(width as usize),
        separator_style,
    )));

    let challenge_games: &[(&str, [AchievementId; 4])] = &[
        (
            "Chess",
            [
                AchievementId::ChessNovice,
                AchievementId::ChessApprentice,
                AchievementId::ChessJourneyman,
                AchievementId::ChessMaster,
            ],
        ),
        (
            "Morris",
            [
                AchievementId::MorrisNovice,
                AchievementId::MorrisApprentice,
                AchievementId::MorrisJourneyman,
                AchievementId::MorrisMaster,
            ],
        ),
        (
            "Gomoku",
            [
                AchievementId::GomokuNovice,
                AchievementId::GomokuApprentice,
                AchievementId::GomokuJourneyman,
                AchievementId::GomokuMaster,
            ],
        ),
        (
            "Minesweeper",
            [
                AchievementId::MinesweeperNovice,
                AchievementId::MinesweeperApprentice,
                AchievementId::MinesweeperJourneyman,
                AchievementId::MinesweeperMaster,
            ],
        ),
        (
            "Rune",
            [
                AchievementId::RuneNovice,
                AchievementId::RuneApprentice,
                AchievementId::RuneJourneyman,
                AchievementId::RuneMaster,
            ],
        ),
        (
            "Go",
            [
                AchievementId::GoNovice,
                AchievementId::GoApprentice,
                AchievementId::GoJourneyman,
                AchievementId::GoMaster,
            ],
        ),
        (
            "Skyward",
            [
                AchievementId::FlappyNovice,
                AchievementId::FlappyApprentice,
                AchievementId::FlappyJourneyman,
                AchievementId::FlappyMaster,
            ],
        ),
        (
            "Serpent",
            [
                AchievementId::SnakeNovice,
                AchievementId::SnakeApprentice,
                AchievementId::SnakeJourneyman,
                AchievementId::SnakeMaster,
            ],
        ),
        (
            "Breach",
            [
                AchievementId::ContainmentBreachNovice,
                AchievementId::ContainmentBreachApprentice,
                AchievementId::ContainmentBreachJourneyman,
                AchievementId::ContainmentBreachMaster,
            ],
        ),
        (
            "Sigil",
            [
                AchievementId::SigilSurgeNovice,
                AchievementId::SigilSurgeApprentice,
                AchievementId::SigilSurgeJourneyman,
                AchievementId::SigilSurgeMaster,
            ],
        ),
    ];

    let diff_labels = ["Nov", "App", "Jou", "Mas"];

    for (name, ids) in challenge_games {
        let mut spans = vec![Span::styled(
            format!("  {name:<12}"),
            Style::default().fg(Color::DarkGray),
        )];
        for (i, id) in ids.iter().enumerate() {
            let unlocked = achievements.is_unlocked(*id);
            let (text, style) = if unlocked {
                (diff_labels[i], Style::default().fg(Color::Green))
            } else {
                ("---", Style::default().fg(Color::DarkGray))
            };
            spans.push(Span::styled(format!(" {text}"), style));
        }
        lines.push(Line::from(spans));
    }

    lines.push(Line::from(vec![
        Span::styled(
            format!("  {:<12}", "Total Wins"),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!(" {}", format_number(achievements.total_minigame_wins)),
            Style::default().fg(Color::Cyan),
        ),
    ]));

    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled("ENHANCEMENT", section_style)));
    lines.push(Line::from(Span::styled(
        "\u{2500}".repeat(width as usize),
        separator_style,
    )));
    let slot_names = [
        "Weapon", "Armor", "Helmet", "Gloves", "Boots", "Amulet", "Ring",
    ];
    for pair in slot_names.iter().enumerate().collect::<Vec<_>>().chunks(2) {
        let mut spans = Vec::new();
        for &(slot_idx, name) in pair {
            let level = enhancement.levels[slot_idx];
            let (r, g, b) = crate::enhancement::enhancement_color_rgb(level);
            let color = Color::Rgb(r, g, b);
            spans.push(Span::styled(
                format!("  {:<8}+{:<3}", name, level),
                Style::default().fg(color),
            ));
        }
        lines.push(Line::from(spans));
    }
    lines.push(Line::from(""));

    let total_unlocked = achievements.unlocked_count();
    let total_count = achievements.total_count();
    let pct = achievements.unlock_percentage();

    lines.push(Line::from(vec![
        Span::styled("ACHIEVEMENTS", section_style),
        Span::styled(
            format!("    {}/{} {:.1}%", total_unlocked, total_count, pct),
            Style::default().fg(Color::Cyan),
        ),
    ]));
    lines.push(Line::from(Span::styled(
        "\u{2500}".repeat(width as usize),
        separator_style,
    )));

    for cat in &[
        AchievementCategory::Combat,
        AchievementCategory::Level,
        AchievementCategory::Progression,
        AchievementCategory::Challenges,
        AchievementCategory::Exploration,
    ] {
        let (unlocked, total) = achievements.count_by_category(*cat);
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<16}", cat.name()),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{unlocked}/{total}"),
                Style::default().fg(Color::Cyan),
            ),
        ]));
    }

    lines
}
