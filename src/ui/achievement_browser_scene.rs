//! Achievement browser overlay UI.
//!
//! Displays a browsable list of achievements organized by category,
//! with a detail panel showing description and unlock status.

use crate::achievements::{
    get_achievement_def, get_achievements_by_category, AchievementCategory, AchievementId,
    Achievements,
};
use crate::character::prestige::get_prestige_tier;
use crate::fishing::types::fishing_tier_name;
use crate::zones::get_all_zones;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// UI state for the achievement browser overlay.
pub struct AchievementBrowserState {
    pub showing: bool,
    pub selected_category: AchievementCategory,
    pub selected_index: usize,
}

impl AchievementBrowserState {
    pub fn new() -> Self {
        Self {
            showing: false,
            selected_category: AchievementCategory::Combat,
            selected_index: 0,
        }
    }

    pub fn open(&mut self) {
        self.showing = true;
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.showing = false;
    }

    pub fn next_category(&mut self) {
        self.selected_category = match self.selected_category {
            AchievementCategory::Combat => AchievementCategory::Level,
            AchievementCategory::Level => AchievementCategory::Progression,
            AchievementCategory::Progression => AchievementCategory::Challenges,
            AchievementCategory::Challenges => AchievementCategory::Exploration,
            AchievementCategory::Exploration => AchievementCategory::Stats,
            AchievementCategory::Stats => AchievementCategory::Combat,
        };
        self.selected_index = 0;
    }

    pub fn prev_category(&mut self) {
        self.selected_category = match self.selected_category {
            AchievementCategory::Combat => AchievementCategory::Stats,
            AchievementCategory::Stats => AchievementCategory::Exploration,
            AchievementCategory::Level => AchievementCategory::Combat,
            AchievementCategory::Progression => AchievementCategory::Level,
            AchievementCategory::Challenges => AchievementCategory::Progression,
            AchievementCategory::Exploration => AchievementCategory::Challenges,
        };
        self.selected_index = 0;
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self, max_items: usize) {
        if self.selected_index + 1 < max_items {
            self.selected_index += 1;
        }
    }
}

impl Default for AchievementBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the achievement browser overlay.
pub fn render_achievement_browser(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    ui_state: &AchievementBrowserState,
    _ctx: &super::responsive::LayoutContext,
) {
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(
            " Achievements ({:.1}% Complete) ",
            achievements.unlock_percentage()
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout: Category tabs at top, list on left, detail on right, help at bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Category tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Help
        ])
        .split(inner);

    render_category_tabs(frame, chunks[0], achievements, ui_state);

    // Content area: stats view or list+detail
    if ui_state.selected_category == AchievementCategory::Stats {
        render_stats_view(frame, chunks[1], achievements, ui_state.selected_index);
    } else {
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(chunks[1]);

        render_achievement_list(frame, content_chunks[0], achievements, ui_state);
        render_achievement_detail(frame, content_chunks[1], achievements, ui_state);
    }

    let help = Paragraph::new("[</>] Category  [Up/Down] Select  [Esc] Close")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}

fn render_category_tabs(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    ui_state: &AchievementBrowserState,
) {
    let mut spans = Vec::new();

    for cat in AchievementCategory::ALL {
        let style = if cat == ui_state.selected_category {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        if cat == AchievementCategory::Stats {
            spans.push(Span::styled(" Stats ", style));
        } else {
            let (unlocked, total) = achievements.count_by_category(cat);
            let new_count = achievements.count_recently_unlocked_by_category(cat);
            if new_count > 0 {
                spans.push(Span::styled(
                    format!(" {} ({}/{}) +{} ", cat.name(), unlocked, total, new_count),
                    style,
                ));
            } else {
                spans.push(Span::styled(
                    format!(" {} ({}/{}) ", cat.name(), unlocked, total),
                    style,
                ));
            }
        }
    }

    let tabs = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    frame.render_widget(tabs, area);
}

fn render_achievement_list(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    ui_state: &AchievementBrowserState,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let category_achievements = get_achievements_by_category(ui_state.selected_category);

    let items: Vec<ListItem> = category_achievements
        .iter()
        .enumerate()
        .map(|(i, def)| {
            let is_unlocked = achievements.is_unlocked(def.id);
            let is_selected = i == ui_state.selected_index;
            let is_new = achievements.is_recently_unlocked(def.id);

            let prefix = if is_selected { "> " } else { "  " };
            let checkmark = if is_unlocked { "[X] " } else { "[ ] " };

            let style = if is_unlocked {
                Style::default().fg(Color::Green)
            } else if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let mut spans = vec![
                Span::styled(prefix, style),
                Span::styled(
                    checkmark,
                    if is_unlocked {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::raw(format!("{} ", def.icon)),
                Span::styled(def.name, style),
            ];

            if is_new {
                spans.push(Span::styled(
                    " [NEW]",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_achievement_detail(
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
    let block = Block::default()
        .title(format!(" {} ", def.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_unlocked {
            Color::Green
        } else {
            Color::DarkGray
        }));

    let inner = block.inner(area);
    frame.render_widget(block, area);

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
                        format!("{}/{}", display_current, progress.target),
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
                format!("    Progress: {}/{}", progress.current, progress.target),
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

/// Format a number with commas (e.g., 12847 -> "12,847").
fn format_number(n: u64) -> String {
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
fn stat_line<'a>(
    label: &'a str,
    value: &'a str,
    label_style: Style,
    value_style: Style,
    width: u16,
) -> Line<'a> {
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

/// Render the stats view (full-width, two columns).
fn render_stats_view(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    scroll_offset: usize,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split into two columns: 45% left (raw stats), 55% right (grids)
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(inner);

    render_stats_left_column(frame, columns[0], achievements, scroll_offset);
    render_stats_right_column(frame, columns[1], achievements, scroll_offset);
}

/// Render the left column: raw stats with dot-leaders.
fn render_stats_left_column(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    scroll_offset: usize,
) {
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

    let w = area.width as usize;

    // Pre-compute formatted numbers to extend their lifetimes
    let total_kills_str = format_number(achievements.total_kills);
    let boss_kills_str = format_number(achievements.total_bosses_defeated);
    let highest_level_str = format_number(achievements.highest_level as u64);
    let highest_prestige_str = format_number(achievements.highest_prestige_rank as u64);
    let expanse_cycles_str = format_number(achievements.expanse_cycles_completed);
    let total_fish_str = format_number(achievements.total_fish_caught);
    let highest_fishing_rank_str = format_number(achievements.highest_fishing_rank as u64);
    let dungeons_completed_str = format_number(achievements.total_dungeons_completed);
    let minigame_wins_str = format_number(achievements.total_minigame_wins);

    let mut lines: Vec<Line> = vec![
        // COMBAT section
        Line::from(Span::styled("COMBAT", section_style)),
        Line::from(Span::styled("\u{2500}".repeat(w), separator_style)),
        stat_line(
            "Total Kills",
            &total_kills_str,
            label_style,
            value_style,
            area.width,
        ),
        stat_line(
            "Boss Kills",
            &boss_kills_str,
            label_style,
            value_style,
            area.width,
        ),
        stat_line(
            "Kill/Boss Ratio",
            &kill_boss_ratio,
            label_style,
            value_style,
            area.width,
        ),
        Line::from(""),
        // PROGRESSION section
        Line::from(Span::styled("PROGRESSION", section_style)),
        Line::from(Span::styled("\u{2500}".repeat(w), separator_style)),
        stat_line(
            "Highest Level",
            &highest_level_str,
            label_style,
            value_style,
            area.width,
        ),
        stat_line(
            "Highest Prestige",
            &highest_prestige_str,
            label_style,
            value_style,
            area.width,
        ),
        stat_line(
            "Prestige Tier",
            prestige_tier,
            label_style,
            value_style,
            area.width,
        ),
        stat_line(
            "Expanse Cycles",
            &expanse_cycles_str,
            label_style,
            value_style,
            area.width,
        ),
        Line::from(""),
        // FISHING section
        Line::from(Span::styled("FISHING", section_style)),
        Line::from(Span::styled("\u{2500}".repeat(w), separator_style)),
        stat_line(
            "Total Fish Caught",
            &total_fish_str,
            label_style,
            value_style,
            area.width,
        ),
        stat_line(
            "Highest Rank",
            &highest_fishing_rank_str,
            label_style,
            value_style,
            area.width,
        ),
        stat_line(
            "Rank Tier",
            fishing_tier,
            label_style,
            value_style,
            area.width,
        ),
        Line::from(""),
        // DUNGEONS & CHALLENGES section
        Line::from(Span::styled("DUNGEONS & CHALLENGES", section_style)),
        Line::from(Span::styled("\u{2500}".repeat(w), separator_style)),
        stat_line(
            "Dungeons Completed",
            &dungeons_completed_str,
            label_style,
            value_style,
            area.width,
        ),
        stat_line(
            "Minigame Wins",
            &minigame_wins_str,
            label_style,
            value_style,
            area.width,
        ),
    ];

    // Apply scroll offset
    if scroll_offset < lines.len() {
        lines = lines.into_iter().skip(scroll_offset).collect();
    } else {
        lines.clear();
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}

/// Render the right column: zone checklist, challenge grid, achievement summary.
fn render_stats_right_column(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    scroll_offset: usize,
) {
    let section_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let separator_style = Style::default().fg(Color::DarkGray);

    let mut lines: Vec<Line> = Vec::new();

    // ZONES CLEARED section
    lines.push(Line::from(Span::styled("ZONES CLEARED", section_style)));
    lines.push(Line::from(Span::styled(
        "\u{2500}".repeat(area.width as usize),
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

    // Two zones per row
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

    // CHALLENGES MASTERED section
    lines.push(Line::from(Span::styled(
        "CHALLENGES MASTERED",
        section_style,
    )));
    lines.push(Line::from(Span::styled(
        "\u{2500}".repeat(area.width as usize),
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

    // Total wins line
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

    // ACHIEVEMENTS section
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
        "\u{2500}".repeat(area.width as usize),
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

    // Apply scroll offset
    if scroll_offset < lines.len() {
        lines = lines.into_iter().skip(scroll_offset).collect();
    } else {
        lines.clear();
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}

/// Render the achievement unlocked celebration modal.
pub fn render_achievement_unlocked_modal(
    frame: &mut Frame,
    area: Rect,
    achievements: &[AchievementId],
    _ctx: &super::responsive::LayoutContext,
) {
    if achievements.is_empty() {
        return;
    }

    let is_single = achievements.len() == 1;
    let modal_height = if is_single {
        9u16.min(area.height.saturating_sub(4))
    } else {
        ((6 + achievements.len()).min(20) as u16).min(area.height.saturating_sub(4))
    };
    let modal_width = 50u16.min(area.width.saturating_sub(4));

    // Center the modal
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let title = if is_single {
        " Achievement Unlocked! "
    } else {
        " Achievements Unlocked! "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let mut lines = vec![Line::from("")];

    if is_single {
        // Single achievement: show icon, name, and description
        if let Some(def) = get_achievement_def(achievements[0]) {
            lines.push(Line::from(Span::styled(
                format!("{}  {}", def.icon, def.name),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                def.description,
                Style::default().fg(Color::White),
            )));
        }
    } else {
        // Multiple achievements: show count and list
        lines.push(Line::from(Span::styled(
            format!("🏆  {} achievements!", achievements.len()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for id in achievements.iter().take(12) {
            if let Some(def) = get_achievement_def(*id) {
                lines.push(Line::from(Span::styled(
                    format!("  {}  {}", def.icon, def.name),
                    Style::default().fg(Color::White),
                )));
            }
        }

        if achievements.len() > 12 {
            lines.push(Line::from(Span::styled(
                format!("  ...and {} more", achievements.len() - 12),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("[Enter] to continue", Style::default().fg(Color::DarkGray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("A = Achievements", Style::default().fg(Color::Magenta)),
    ]));

    let para = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(para, inner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_browser_state_navigation() {
        let mut state = AchievementBrowserState::new();

        // Initial state
        assert!(!state.showing);
        assert_eq!(state.selected_category, AchievementCategory::Combat);
        assert_eq!(state.selected_index, 0);

        // Open
        state.open();
        assert!(state.showing);
        assert_eq!(state.selected_index, 0);

        // Navigate categories
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Level);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Progression);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Challenges);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Exploration);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Stats);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Combat);

        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Stats);
        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Exploration);

        // Navigate items
        state.move_down(10);
        assert_eq!(state.selected_index, 1);
        state.move_up();
        assert_eq!(state.selected_index, 0);
        state.move_up();
        assert_eq!(state.selected_index, 0); // Can't go below 0

        // Close
        state.close();
        assert!(!state.showing);
    }

    #[test]
    fn test_stats_tab_navigation() {
        let mut state = AchievementBrowserState::new();

        // Navigate to Stats tab
        state.next_category(); // Level
        state.next_category(); // Progression
        state.next_category(); // Challenges
        state.next_category(); // Exploration
        state.next_category(); // Stats
        assert_eq!(state.selected_category, AchievementCategory::Stats);

        // Stats wraps to Combat
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Combat);

        // Backward from Combat goes to Stats
        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Stats);

        // Backward from Stats goes to Exploration
        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Exploration);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(12847), "12,847");
        assert_eq!(format_number(1000000), "1,000,000");
    }
}
