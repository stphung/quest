//! The Deep overlay UI rendering.
//!
//! Renders a centered popup overlay with four tabs: Dashboard, Roster,
//! Missions, and Recruitment. Follows the haven_scene.rs / soulforge_scene.rs
//! overlay pattern.
//!
//! Note: dead_code allowed because this module is not yet wired into the main
//! game loop. Integration is tracked in task #29 (Wire Deep overlay into main).
#![allow(dead_code)]

use crate::deep::clock::{
    format_duration, is_mission_complete, mission_progress_percent, pending_events,
    time_remaining_secs,
};
use crate::deep::infrastructure::total_outpost_daily_income;
use crate::deep::types::{GuildRank, MercStatus, MissionType, TheDeepState};
use crate::deep::ui_state::{
    DeepTab, DeepUiState, EventResolveState, LaunchStep, MissionLaunchState,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, TableState},
    Frame,
};

// =========================================================================
// Color constants
// =========================================================================

/// Accent color for The Deep (deep teal).
const DEEP_ACCENT: Color = Color::Rgb(80, 180, 200);

// =========================================================================
// Guild rank display helpers
// =========================================================================

fn guild_rank_name(rank: GuildRank) -> &'static str {
    match rank {
        GuildRank::Freelancers => "Freelancers",
        GuildRank::Sellswords => "Sellswords",
        GuildRank::Company => "Company",
        GuildRank::Battalion => "Battalion",
        GuildRank::Legion => "Legion",
    }
}

fn guild_rank_color(rank: GuildRank) -> Color {
    match rank {
        GuildRank::Freelancers => Color::White,
        GuildRank::Sellswords => Color::Blue,
        GuildRank::Company => Color::Yellow,
        GuildRank::Battalion => Color::Magenta,
        GuildRank::Legion => Color::Rgb(255, 215, 0),
    }
}

fn mission_type_name(mt: MissionType) -> &'static str {
    match mt {
        MissionType::SupplyRun => "Supply Run",
        MissionType::Recon => "Recon",
        MissionType::Expedition => "Expedition",
        MissionType::Breakthrough => "Breakthrough",
        MissionType::Construction => "Construction",
    }
}

fn mission_type_color(mt: MissionType) -> Color {
    match mt {
        MissionType::SupplyRun => Color::Yellow,
        MissionType::Recon => Color::Cyan,
        MissionType::Expedition => Color::Green,
        MissionType::Breakthrough => Color::Red,
        MissionType::Construction => Color::Blue,
    }
}

fn merc_status_color(status: MercStatus) -> Color {
    match status {
        MercStatus::Ready => Color::Green,
        MercStatus::OnMission => Color::Yellow,
        MercStatus::Injured => Color::Red,
        MercStatus::Lost => Color::DarkGray,
    }
}

fn merc_status_label(status: MercStatus, injury_cooldown: u8) -> String {
    match status {
        MercStatus::Ready => "Ready".to_string(),
        MercStatus::OnMission => "Mission".to_string(),
        MercStatus::Injured => format!("Injured({})", injury_cooldown),
        MercStatus::Lost => "Lost".to_string(),
    }
}

fn progress_color(percent: u8) -> Color {
    if percent >= 75 {
        Color::Green
    } else if percent >= 25 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn render_progress_bar(percent: u8, width: usize) -> String {
    if width < 5 {
        return format!("{}%", percent);
    }
    let bar_width = width.saturating_sub(5); // leave room for " xx% "
    let filled = (bar_width * percent as usize / 100).min(bar_width);
    let empty = bar_width - filled;
    format!(
        "[{}{}] {:>3}%",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
        percent
    )
}

// =========================================================================
// Main entry point
// =========================================================================

/// Render The Deep overlay.
///
/// Renders a centered popup (approximately 80% of terminal area) with tab
/// navigation and content appropriate for the active tab.
pub fn draw_deep_overlay(
    frame: &mut Frame,
    area: Rect,
    deep: &TheDeepState,
    ui: &DeepUiState,
    now_unix: i64,
) {
    // Size the overlay: ~85% wide, ~85% tall, centered.
    let overlay_width = (area.width as f32 * 0.85) as u16;
    let overlay_height = (area.height as f32 * 0.85) as u16;
    let overlay_width = overlay_width.max(60).min(area.width.saturating_sub(2));
    let overlay_height = overlay_height.max(20).min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    // Outer border with title showing guild name and marks.
    let guild_name = guild_rank_name(deep.account.guild_rank);
    let marks = deep.run.warband_marks;
    let title = format!(" The Deep  [{}]  {} Marks ", guild_name, marks);
    let border_color = if deep.discovered {
        DEEP_ACCENT
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    if inner.height < 4 || inner.width < 20 {
        return;
    }

    // Split inner: tab bar (1 row) + divider (1 row) + content + help bar (1 row).
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Length(1), // Divider line
            Constraint::Min(1),    // Content
            Constraint::Length(1), // Help bar
        ])
        .split(inner);

    render_tab_bar(frame, chunks[0], ui);
    render_divider(frame, chunks[1], inner.width);
    render_content(frame, chunks[2], deep, ui, now_unix);
    render_help_bar(frame, chunks[3], ui);

    // Sub-overlays rendered on top.
    if let Some(ref launch) = ui.mission_launch {
        render_mission_launch_overlay(frame, overlay_area, deep, launch, now_unix);
    } else if let Some(ref resolve) = ui.event_resolve {
        render_event_resolve_overlay(frame, overlay_area, deep, resolve, now_unix);
    }
}

// =========================================================================
// Tab bar
// =========================================================================

fn render_tab_bar(frame: &mut Frame, area: Rect, ui: &DeepUiState) {
    let tabs = DeepTab::all();
    let mut spans: Vec<Span> = Vec::new();
    for (i, tab) in tabs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", Style::default().fg(Color::DarkGray)));
        }
        let is_active = *tab == ui.active_tab;
        let style = if is_active {
            Style::default()
                .fg(DEEP_ACCENT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let label = if is_active {
            format!("[{}]", tab.label())
        } else {
            format!(" {} ", tab.label())
        };
        spans.push(Span::styled(label, style));
    }
    let tab_line = Paragraph::new(Line::from(spans));
    frame.render_widget(tab_line, area);
}

// =========================================================================
// Divider
// =========================================================================

fn render_divider(frame: &mut Frame, area: Rect, width: u16) {
    let line = "\u{2500}".repeat(width as usize);
    let divider = Paragraph::new(Span::styled(
        line,
        Style::default().fg(Color::Rgb(40, 50, 70)),
    ));
    frame.render_widget(divider, area);
}

// =========================================================================
// Content dispatch
// =========================================================================

fn render_content(
    frame: &mut Frame,
    area: Rect,
    deep: &TheDeepState,
    ui: &DeepUiState,
    now_unix: i64,
) {
    // Content area: main body + optional status message line.
    let (content_area, msg_area) = if ui.message.is_some() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
    };

    match ui.active_tab {
        DeepTab::Dashboard => render_dashboard(frame, content_area, deep, now_unix),
        DeepTab::Roster => render_roster(frame, content_area, deep, ui),
        DeepTab::Missions => render_missions(frame, content_area, deep, ui, now_unix),
        DeepTab::Recruitment => render_recruitment(frame, content_area, deep, ui),
    }

    if let (Some(area), Some(ref msg)) = (msg_area, &ui.message) {
        let msg_para = Paragraph::new(Span::styled(
            msg.as_str(),
            Style::default().fg(Color::Yellow),
        ));
        frame.render_widget(msg_para, area);
    }
}

// =========================================================================
// Dashboard tab
// =========================================================================

fn render_dashboard(frame: &mut Frame, area: Rect, deep: &TheDeepState, now_unix: i64) {
    let rank = deep.account.guild_rank;
    let roster_count = deep.run.mercenaries.len();
    let roster_cap = rank.roster_cap();
    let active_count = deep.run.active_missions.len();
    let mission_slots = rank.mission_slots();
    let completed_count = deep
        .run
        .completed_missions
        .iter()
        .filter(|m| !m.collected)
        .count();
    let total_income = total_outpost_daily_income(deep);
    let deepest_layer = deep
        .account
        .layers
        .iter()
        .filter(|l| l.cleared)
        .map(|l| l.layer_number)
        .max()
        .unwrap_or(0);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Guild Rank:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                guild_rank_name(rank),
                Style::default()
                    .fg(guild_rank_color(rank))
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Warband Marks: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", deep.run.warband_marks),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Roster:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", roster_count, roster_cap),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Missions:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{} active", active_count, mission_slots),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    if completed_count > 0 {
        lines.push(Line::from(vec![
            Span::styled("  Awaiting:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} completed", completed_count),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  [c] Collect", Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Deepest Layer: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            if deepest_layer == 0 {
                "None yet".to_string()
            } else {
                format!("Layer {}", deepest_layer)
            },
            Style::default().fg(Color::Cyan),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Outpost Income: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{} Marks/day", total_income),
            Style::default().fg(Color::Yellow),
        ),
    ]));

    // Active mission summary.
    if !deep.run.active_missions.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Active Missions:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        for mission in &deep.run.active_missions {
            let progress = mission_progress_percent(mission, now_unix);
            let remaining = time_remaining_secs(mission, now_unix);
            let time_str = format_duration(remaining);
            let pending = pending_events(mission, now_unix).len();
            let event_indicator = if pending > 0 {
                format!(
                    " [{} event{}]",
                    pending,
                    if pending == 1 { "" } else { "s" }
                )
            } else {
                String::new()
            };
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(
                    mission_type_name(mission.mission_type),
                    Style::default().fg(mission_type_color(mission.mission_type)),
                ),
                Span::styled(
                    format!(" L{}  {}%  {}", mission.layer, progress, time_str),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(event_indicator, Style::default().fg(Color::Yellow)),
            ]));
        }
    }

    // Guild upgrade hint.
    if let Some(cost) = rank.upgrade_cost() {
        if let Some(next) = rank.next() {
            lines.push(Line::from(""));
            let can_afford = deep.run.warband_marks >= cost;
            let color = if can_afford {
                Color::Green
            } else {
                Color::DarkGray
            };
            lines.push(Line::from(vec![
                Span::styled("  Upgrade to ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    guild_rank_name(next),
                    Style::default().fg(guild_rank_color(next)),
                ),
                Span::styled(format!(": {} Marks", cost), Style::default().fg(color)),
            ]));
        }
    }

    let _ = now_unix; // suppress unused warning if missions is empty
    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}

// =========================================================================
// Roster tab
// =========================================================================

fn render_roster(frame: &mut Frame, area: Rect, deep: &TheDeepState, ui: &DeepUiState) {
    if deep.run.mercenaries.is_empty() {
        let para = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No mercenaries in your roster.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  Visit the Recruitment tab to hire some.",
                Style::default().fg(Color::DarkGray),
            )),
        ]);
        frame.render_widget(para, area);
        return;
    }

    let header = Row::new(vec!["Name", "Archetype", "Lv", "Power", "Resil", "Status"])
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

    let rows: Vec<Row> = deep
        .run
        .mercenaries
        .iter()
        .enumerate()
        .map(|(i, merc)| {
            let is_selected = i == ui.selected_index;
            let status_label = merc_status_label(merc.status, merc.injury_cooldown);
            let row = Row::new(vec![
                merc.name.clone(),
                merc.archetype.to_string(),
                merc.level.to_string(),
                merc.power.to_string(),
                merc.resilience.to_string(),
                status_label,
            ])
            .style(if is_selected {
                Style::default().fg(Color::White).bg(Color::Rgb(30, 40, 60))
            } else {
                Style::default().fg(merc_status_color(merc.status))
            });
            row
        })
        .collect();

    let widths = [
        Constraint::Min(14),
        Constraint::Length(10),
        Constraint::Length(3),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Min(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(1)
        .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut state = TableState::default();
    state.select(Some(
        ui.selected_index
            .min(deep.run.mercenaries.len().saturating_sub(1)),
    ));
    frame.render_stateful_widget(table, area, &mut state);
}

// =========================================================================
// Missions tab
// =========================================================================

fn render_missions(
    frame: &mut Frame,
    area: Rect,
    deep: &TheDeepState,
    ui: &DeepUiState,
    now_unix: i64,
) {
    let active = &deep.run.active_missions;
    let completed: Vec<_> = deep
        .run
        .completed_missions
        .iter()
        .filter(|m| !m.collected)
        .collect();

    if active.is_empty() && completed.is_empty() {
        let rank = deep.account.guild_rank;
        let slots = rank.mission_slots();
        let para = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No active missions.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  Mission slots: {}", slots),
                Style::default().fg(Color::White),
            )]),
            Line::from(Span::styled(
                "  Press [Enter] to launch a mission.",
                Style::default().fg(Color::DarkGray),
            )),
        ]);
        frame.render_widget(para, area);
        return;
    }

    let show_completed = !completed.is_empty();
    let (active_area, completed_area) = if show_completed && area.height >= 8 {
        let half = area.height / 2;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(half), Constraint::Min(1)])
            .split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
    };

    // Active missions panel.
    {
        let mut lines = vec![Line::from(Span::styled(
            "  Active Missions",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))];

        if active.is_empty() {
            lines.push(Line::from(Span::styled(
                "    (none)",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for (idx, mission) in active.iter().enumerate() {
                let progress = mission_progress_percent(mission, now_unix);
                let remaining = time_remaining_secs(mission, now_unix);
                let time_str = format_duration(remaining);
                let is_complete = is_mission_complete(mission, now_unix);
                let pending = pending_events(mission, now_unix).len();
                let is_selected = idx == ui.selected_index && ui.active_tab == DeepTab::Missions;

                let bar_width = active_area.width.saturating_sub(10) as usize;
                let bar = render_progress_bar(progress, bar_width.min(30));

                let prefix = if is_selected { " > " } else { "   " };
                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(
                        mission_type_name(mission.mission_type),
                        Style::default()
                            .fg(mission_type_color(mission.mission_type))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" L{}", mission.layer),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("     ", Style::default()),
                    Span::styled(bar, Style::default().fg(progress_color(progress))),
                    Span::styled(
                        format!("  {}", time_str),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
                if is_complete {
                    lines.push(Line::from(Span::styled(
                        "     [Complete — collecting on next tick]",
                        Style::default().fg(Color::Green),
                    )));
                } else if pending > 0 {
                    lines.push(Line::from(Span::styled(
                        format!(
                            "     {} event{} pending — [Enter] to resolve",
                            pending,
                            if pending == 1 { "" } else { "s" }
                        ),
                        Style::default().fg(Color::Yellow),
                    )));
                }
            }
        }

        let para = Paragraph::new(lines);
        frame.render_widget(para, active_area);
    }

    // Completed missions panel.
    if let Some(c_area) = completed_area {
        let mut lines = vec![Line::from(Span::styled(
            "  Completed (uncollected)",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ))];
        for cm in &completed {
            let result_color = match cm.result {
                crate::deep::types::MissionResult::Success => Color::Green,
                crate::deep::types::MissionResult::PartialSuccess => Color::Yellow,
                crate::deep::types::MissionResult::Failure => Color::Red,
            };
            let result_str = match cm.result {
                crate::deep::types::MissionResult::Success => "Success",
                crate::deep::types::MissionResult::PartialSuccess => "Partial",
                crate::deep::types::MissionResult::Failure => "Failure",
            };
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(
                    mission_type_name(cm.mission_type),
                    Style::default().fg(mission_type_color(cm.mission_type)),
                ),
                Span::styled(
                    format!(" L{}  ", cm.layer),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(result_str, Style::default().fg(result_color)),
                Span::styled(
                    format!("  +{} Marks", cm.marks_earned),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled("  [c] Collect", Style::default().fg(Color::DarkGray)),
            ]));
        }
        let para = Paragraph::new(lines);
        frame.render_widget(para, c_area);
    }
}

// =========================================================================
// Recruitment tab
// =========================================================================

fn render_recruitment(frame: &mut Frame, area: Rect, deep: &TheDeepState, ui: &DeepUiState) {
    let roster_count = deep.run.mercenaries.len();
    let roster_cap = deep.account.guild_rank.roster_cap();

    if deep.run.recruitment_pool.is_empty() {
        let para = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Recruitment pool is empty.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  Pool refreshes daily.",
                Style::default().fg(Color::DarkGray),
            )),
        ]);
        frame.render_widget(para, area);
        return;
    }

    let cap_line = Line::from(vec![
        Span::styled("  Roster: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}/{}", roster_count, roster_cap),
            Style::default().fg(if roster_count >= roster_cap {
                Color::Red
            } else {
                Color::White
            }),
        ),
        Span::styled(
            "   (cost per hire: 40 Marks)",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let header = Row::new(vec!["Name", "Archetype", "Power", "Resil", "Cost"])
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

    let rows: Vec<Row> = deep
        .run
        .recruitment_pool
        .iter()
        .enumerate()
        .map(|(i, merc)| {
            let is_selected = i == ui.selected_index;
            Row::new(vec![
                merc.name.clone(),
                merc.archetype.to_string(),
                merc.power.to_string(),
                merc.resilience.to_string(),
                "40 M".to_string(),
            ])
            .style(if is_selected {
                Style::default().fg(Color::White).bg(Color::Rgb(30, 40, 60))
            } else {
                Style::default().fg(Color::White)
            })
        })
        .collect();

    let widths = [
        Constraint::Min(14),
        Constraint::Length(10),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(8),
    ];

    let table = Table::new(rows, widths).header(header).column_spacing(1);

    // Layout: cap line (1) + table (rest).
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    let cap_para = Paragraph::new(cap_line);
    frame.render_widget(cap_para, chunks[0]);

    let mut state = TableState::default();
    state.select(Some(
        ui.selected_index
            .min(deep.run.recruitment_pool.len().saturating_sub(1)),
    ));
    frame.render_stateful_widget(table, chunks[1], &mut state);
}

// =========================================================================
// Help bar
// =========================================================================

fn render_help_bar(frame: &mut Frame, area: Rect, ui: &DeepUiState) {
    let text = match ui.active_tab {
        DeepTab::Dashboard => {
            "[\u{2190}\u{2192}] Tab  [c] Collect  [Esc] Close"
        }
        DeepTab::Roster => {
            "[\u{2190}\u{2192}] Tab  [\u{2191}\u{2193}] Select  [Esc] Close"
        }
        DeepTab::Missions => {
            "[\u{2190}\u{2192}] Tab  [\u{2191}\u{2193}] Select  [Enter] Resolve/Launch  [c] Collect  [Esc] Close"
        }
        DeepTab::Recruitment => {
            "[\u{2190}\u{2192}] Tab  [\u{2191}\u{2193}] Select  [Enter] Hire  [Esc] Close"
        }
    };
    let help = Paragraph::new(Span::styled(text, Style::default().fg(Color::DarkGray)));
    frame.render_widget(help, area);
}

// =========================================================================
// Mission launch overlay
// =========================================================================

fn render_mission_launch_overlay(
    frame: &mut Frame,
    parent: Rect,
    deep: &TheDeepState,
    launch: &MissionLaunchState,
    _now_unix: i64,
) {
    let modal_width = 50u16.min(parent.width.saturating_sub(4));
    let modal_height = 16u16.min(parent.height.saturating_sub(4));
    let x = parent.x + (parent.width.saturating_sub(modal_width)) / 2;
    let y = parent.y + (parent.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let title = match launch.step {
        LaunchStep::SelectMission => " Launch Mission ",
        LaunchStep::SelectSquad => " Select Squad ",
        LaunchStep::Confirm => " Confirm Launch ",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DEEP_ACCENT));
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    match launch.step {
        LaunchStep::SelectMission => {
            render_launch_select_mission(frame, inner, deep, launch);
        }
        LaunchStep::SelectSquad => {
            render_launch_select_squad(frame, inner, deep, launch);
        }
        LaunchStep::Confirm => {
            render_launch_confirm(frame, inner, deep, launch);
        }
    }
}

fn render_launch_select_mission(
    frame: &mut Frame,
    area: Rect,
    _deep: &TheDeepState,
    launch: &MissionLaunchState,
) {
    let mission_types = [
        (MissionType::SupplyRun, "2-4h, no risk, resource farming"),
        (MissionType::Recon, "4-8h, low risk, gather intel"),
        (MissionType::Expedition, "8-16h, medium risk, progression"),
        (MissionType::Breakthrough, "18-24h, high risk, clear layer"),
        (MissionType::Construction, "4-8h, no risk, build infra"),
    ];

    let items: Vec<ListItem> = mission_types
        .iter()
        .enumerate()
        .map(|(i, (mt, desc))| {
            let is_sel = i == launch.mission_index;
            let prefix = if is_sel { "> " } else { "  " };
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(
                        mission_type_name(*mt),
                        Style::default()
                            .fg(mission_type_color(*mt))
                            .add_modifier(if is_sel {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            }),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled(*desc, Style::default().fg(Color::DarkGray)),
                ]),
            ])
        })
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let list = List::new(items);
    frame.render_widget(list, chunks[0]);

    let help = Paragraph::new(Span::styled(
        "[\u{2191}\u{2193}] Select  [Enter] Next  [Esc] Cancel",
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(help, chunks[1]);
}

fn render_launch_select_squad(
    frame: &mut Frame,
    area: Rect,
    deep: &TheDeepState,
    launch: &MissionLaunchState,
) {
    let ready_mercs: Vec<_> = deep
        .run
        .mercenaries
        .iter()
        .filter(|m| m.status == MercStatus::Ready)
        .collect();

    let items: Vec<ListItem> = ready_mercs
        .iter()
        .enumerate()
        .map(|(i, merc)| {
            let is_cursor = i == launch.merc_cursor;
            let is_selected = launch.selected_mercs.contains(&merc.id);
            let prefix = if is_cursor { "> " } else { "  " };
            let check = if is_selected { "[x] " } else { "[ ] " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled(
                    check,
                    Style::default().fg(if is_selected {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::styled(merc.name.as_str(), Style::default().fg(Color::White)),
                Span::styled(
                    format!(" ({}) Pw:{}", merc.archetype, merc.power),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(area);

    if ready_mercs.is_empty() {
        let para = Paragraph::new(Span::styled(
            "No ready mercenaries available.",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(para, chunks[0]);
    } else {
        let list = List::new(items);
        frame.render_widget(list, chunks[0]);
    }

    let selected_count = launch.selected_mercs.len();
    let help = Paragraph::new(vec![
        Line::from(Span::styled(
            format!("Selected: {}", selected_count),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            "[\u{2191}\u{2193}] Move  [Space] Toggle  [Enter] Next  [Esc] Back",
            Style::default().fg(Color::DarkGray),
        )),
    ]);
    frame.render_widget(help, chunks[1]);
}

fn render_launch_confirm(
    frame: &mut Frame,
    area: Rect,
    deep: &TheDeepState,
    launch: &MissionLaunchState,
) {
    let mission_types = [
        MissionType::SupplyRun,
        MissionType::Recon,
        MissionType::Expedition,
        MissionType::Breakthrough,
        MissionType::Construction,
    ];
    let mt = mission_types
        .get(launch.mission_index)
        .copied()
        .unwrap_or(MissionType::SupplyRun);

    let squad_names: Vec<_> = launch
        .selected_mercs
        .iter()
        .filter_map(|id| deep.run.mercenaries.iter().find(|m| m.id == *id))
        .map(|m| m.name.as_str())
        .collect();

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Mission: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                mission_type_name(mt),
                Style::default()
                    .fg(mission_type_color(mt))
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Squad:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if squad_names.is_empty() {
                    "(none)".to_string()
                } else {
                    squad_names.join(", ")
                },
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  [Enter] Launch  [Esc] Back",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}

// =========================================================================
// Event resolve overlay
// =========================================================================

fn render_event_resolve_overlay(
    frame: &mut Frame,
    parent: Rect,
    deep: &TheDeepState,
    resolve: &EventResolveState,
    now_unix: i64,
) {
    let modal_width = 52u16.min(parent.width.saturating_sub(4));
    let modal_height = 14u16.min(parent.height.saturating_sub(4));
    let x = parent.x + (parent.width.saturating_sub(modal_width)) / 2;
    let y = parent.y + (parent.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" Resolve Event ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let mission = match deep.run.active_missions.get(resolve.mission_index) {
        Some(m) => m,
        None => return,
    };
    let event = match mission.events.get(resolve.event_index) {
        Some(e) => e,
        None => return,
    };

    let event_name = match event.event_type {
        crate::deep::types::EventType::CaveIn => "Cave-In",
        crate::deep::types::EventType::Ambush => "Ambush",
        crate::deep::types::EventType::FloodedPassage => "Flooded Passage",
        crate::deep::types::EventType::AncientDoor => "Ancient Door",
        crate::deep::types::EventType::Tremor => "Tremor",
    };

    let options = [
        ("Safe", Color::Green),
        ("Archetype", Color::Cyan),
        ("Risky", Color::Red),
    ];

    let progress = mission_progress_percent(mission, now_unix);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Event: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                event_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({}% through)", progress),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Choose a resolution:",
            Style::default().fg(Color::White),
        )),
    ];

    let option_descs = [
        "No risk, minimal reward.",
        "Archetype ability — needs specialist in squad.",
        "Higher reward, chance of injury.",
    ];
    for (i, ((label, color), desc)) in options.iter().zip(option_descs.iter()).enumerate() {
        let is_sel = i == resolve.selected_resolution;
        let prefix = if is_sel { "  > " } else { "    " };
        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(Color::Cyan)),
            Span::styled(
                *label,
                Style::default().fg(*color).add_modifier(if is_sel {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
            ),
            Span::styled(format!(" — {}", desc), Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [\u{2191}\u{2193}] Select  [Enter] Confirm  [Esc] Cancel",
        Style::default().fg(Color::DarkGray),
    )));

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

// =========================================================================
// Discovery modal
// =========================================================================

/// Render the Deep discovery modal.
pub fn render_deep_discovery_modal(frame: &mut Frame, area: Rect) {
    let modal_width = 54u16.min(area.width.saturating_sub(4));
    let modal_height = 10u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" \u{25b6} New System Unlocked \u{25c0} ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "You have discovered The Deep!",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "A vast underground structure awaits.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "Recruit mercenaries and send them on missions.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press [D] to visit. [Enter] to dismiss.",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}
