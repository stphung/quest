//! Time Vault overlay UI.
//!
//! Displays a two-panel overlay for browsing save branches
//! and their snapshots. Players can restore, fork, and manage saves.

use crate::history::types::{CommitInfo, TimelineInfo};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

#[allow(unused_imports)]
use super::scene_fx::{
    current_millis, hash2d, lerp_rgb, put_cell, put_text, render_buffer, SceneCell,
};

#[allow(dead_code)]
/// Temporal backdrop: dark navy top.
const VAULT_TOP_RGB: (u8, u8, u8) = (8, 12, 35);
#[allow(dead_code)]
/// Temporal backdrop: near-black bottom.
const VAULT_BOTTOM_RGB: (u8, u8, u8) = (3, 5, 15);
#[allow(dead_code)]
/// Dim cyan for timeline graph lines.
const TIMELINE_DIM: Color = Color::Rgb(40, 80, 120);
#[allow(dead_code)]
/// Number of drifting particles.
const PARTICLE_COUNT: usize = 5;
#[allow(dead_code)]
/// Particle drift speed (lower = slower).
const PARTICLE_SPEED: f64 = 0.8;

/// Which panel has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    Left,
    Right,
}

/// The current interaction mode of the Time Vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserMode {
    /// Normal browsing — arrow keys navigate, Tab switches focus.
    Browse,
    /// Waiting for confirmation to restore the selected commit.
    ConfirmRestore,
    /// Waiting for confirmation to delete the selected branch.
    ConfirmDelete,
    /// Typing a name for a new forked branch.
    NamingFork { commit_id: String },
}

/// UI state for the Time Vault overlay.
pub struct TimeVaultState {
    pub branches: Vec<TimelineInfo>,
    pub selected_branch: usize,
    pub commits: Vec<CommitInfo>,
    pub selected_commit: usize,
    pub focus: PanelFocus,
    pub mode: BrowserMode,
    pub fork_name_input: String,
    pub fork_name_error: Option<String>,
}

impl TimeVaultState {
    pub fn new(branches: Vec<TimelineInfo>, commits: Vec<CommitInfo>) -> Self {
        Self {
            branches,
            selected_branch: 0,
            commits,
            selected_commit: 0,
            focus: PanelFocus::Right,
            mode: BrowserMode::Browse,
            fork_name_input: String::new(),
            fork_name_error: None,
        }
    }

    /// Name of the currently selected branch, if any.
    pub fn selected_branch_name(&self) -> Option<&str> {
        self.branches
            .get(self.selected_branch)
            .map(|b| b.name.as_str())
    }

    /// Short SHA of the currently selected commit, if any.
    pub fn selected_commit_id(&self) -> Option<&str> {
        self.commits
            .get(self.selected_commit)
            .map(|c| c.id.as_str())
    }

    /// Whether the selected branch is "main".
    pub fn selected_branch_is_main(&self) -> bool {
        self.selected_branch_name() == Some("main")
    }

    /// Whether the selected branch is the currently active branch.
    pub fn selected_branch_is_active(&self) -> bool {
        self.branches
            .get(self.selected_branch)
            .is_some_and(|b| b.is_active)
    }
}

#[allow(dead_code)]
/// Paint the temporal vault backdrop: dark gradient with slow-drifting cyan particles.
fn paint_vault_backdrop(buffer: &mut [Vec<SceneCell>], millis: u128) {
    let height = buffer.len();
    if height == 0 {
        return;
    }
    let width = buffer[0].len();

    // 1. Background gradient (top to bottom)
    for (row, row_cells) in buffer.iter_mut().enumerate() {
        let t = if height <= 1 {
            0.0
        } else {
            row as f64 / (height - 1) as f64
        };
        let rgb = lerp_rgb(VAULT_TOP_RGB, VAULT_BOTTOM_RGB, t);
        let bg = Color::Rgb(rgb.0, rgb.1, rgb.2);
        for cell in row_cells.iter_mut() {
            cell.bg = bg;
        }
    }

    // 2. Subtle particles drifting downward
    let particle_chars: &[char] = &['\u{00b7}', '\u{2022}', '\u{2726}'];
    let particle_hot: (u8, u8, u8) = (80, 160, 220);
    let particle_cool: (u8, u8, u8) = (20, 40, 80);
    for i in 0..PARTICLE_COUNT {
        let seed = hash2d(i, 0);
        let col = (seed as usize) % width;
        let ch = particle_chars[(hash2d(i, 1) as usize) % particle_chars.len()];

        let phase_offset = (seed as f64) * 0.73;
        let pos = (phase_offset + millis as f64 * PARTICLE_SPEED / 1000.0) % height as f64;
        let row = pos as i32;

        let t = pos / height.max(1) as f64;
        let rgb = lerp_rgb(particle_hot, particle_cool, t);
        put_cell(buffer, row, col as i32, ch, Color::Rgb(rgb.0, rgb.1, rgb.2));
    }

    // 3. Faint temporal shimmer
    let flash_phase = (millis / 120) as usize;
    for i in 0..2 {
        let seed = hash2d(flash_phase.wrapping_add(i), 99);
        let row = (seed as usize) % height;
        let col = (hash2d(flash_phase.wrapping_add(i), 111) as usize) % width;
        let brightness = 40 + ((seed % 30) as u8);
        put_cell(
            buffer,
            row as i32,
            col as i32,
            '\u{00b7}',
            Color::Rgb(brightness, brightness + 30, brightness + 80),
        );
    }
}

#[allow(dead_code)]
/// Map a commit message to an event-type icon and color.
fn event_icon_color(message: &str) -> (&'static str, Color) {
    let desc = message.split(" | ").next().unwrap_or(message);
    if desc.starts_with("Defeated") {
        ("\u{2694}", Color::LightRed)          // ⚔
    } else if desc.starts_with("Prestige") {
        ("\u{2605}", Color::Rgb(255, 215, 0))  // ★ gold
    } else if desc.starts_with("Won ") {
        ("\u{265f}", Color::Magenta)           // ♟
    } else if desc.starts_with("Completed") {
        ("\u{25c6}", Color::Green)             // ◆
    } else if desc.starts_with("Caught") || desc.starts_with("Fishing") {
        ("~", Color::Blue)
    } else if desc.starts_with("Built") || desc.starts_with("Upgraded") {
        ("\u{2302}", Color::Yellow)            // ⌂
    } else if desc.starts_with("Enhanced") {
        ("\u{2692}", Color::Cyan)              // ⚒
    } else if desc.starts_with("Achievement") {
        ("\u{2726}", Color::White)             // ✦
    } else if desc.starts_with("Chrono Surge") {
        ("\u{23e9}", Color::Cyan)              // ⏩
    } else {
        ("\u{00b7}", Color::DarkGray)          // ·
    }
}

/// Render the Time Vault overlay.
pub fn draw_time_vault(frame: &mut Frame, area: Rect, state: &TimeVaultState) {
    // Full-screen overlay with padding
    let w = area.width.saturating_sub(4).min(90);
    let h = area.height.saturating_sub(4);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay_area = Rect::new(x, y, w, h);

    frame.render_widget(Clear, overlay_area);

    let outer_block = Block::default()
        .title(
            Line::from(Span::styled(
                " TIME VAULT ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
        )
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Double)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = outer_block.inner(overlay_area);
    frame.render_widget(outer_block, overlay_area);

    // Split inner into: content area + controls bar
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Content
            Constraint::Length(1), // Controls
        ])
        .split(inner);

    let content_area = v_chunks[0];
    let controls_area = v_chunks[1];

    // Split content into left branch panel + right commit panel
    let branch_width = 22u16.min(content_area.width / 3);
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(branch_width), Constraint::Min(10)])
        .split(content_area);

    draw_branch_panel(frame, h_chunks[0], state, state.focus == PanelFocus::Left);
    draw_commit_panel(frame, h_chunks[1], state, state.focus == PanelFocus::Right);
    draw_controls(frame, controls_area, state);
}

/// Render the left branch list panel.
fn draw_branch_panel(frame: &mut Frame, area: Rect, state: &TimeVaultState, focused: bool) {
    let border_color = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let title_color = if focused { Color::Cyan } else { Color::White };
    let block = Block::default()
        .title(Span::styled(
            " Branches ",
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = state
        .branches
        .iter()
        .enumerate()
        .map(|(i, branch)| {
            let prefix = if branch.is_active { "* " } else { "  " };
            let name = format!("{}{}", prefix, branch.name);
            let style = if i == state.selected_branch {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if branch.is_active {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Span::styled(name, style))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

/// Render the right commit list panel.
fn draw_commit_panel(frame: &mut Frame, area: Rect, state: &TimeVaultState, focused: bool) {
    let border_color = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let title_color = if focused { Color::Cyan } else { Color::White };
    let block = Block::default()
        .title(Span::styled(
            " Saves ",
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.commits.is_empty() {
        let empty_msg = Paragraph::new(Span::styled(
            "  No commits on this branch",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(empty_msg, inner);
        return;
    }

    // Each commit card takes 3 lines + 1 blank separator
    let card_height = 4u16;
    let visible_cards = (inner.height / card_height).max(1) as usize;

    // Scroll so selected commit is visible
    let scroll_offset = if state.selected_commit >= visible_cards {
        state.selected_commit - visible_cards + 1
    } else {
        0
    };

    let mut y = inner.y;
    for (i, commit) in state.commits.iter().enumerate().skip(scroll_offset) {
        if y + 3 > inner.y + inner.height {
            break;
        }

        let is_selected = i == state.selected_commit;
        let highlight = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let dim = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Line 1: Event description (part before " | ")
        let description = commit
            .message
            .split(" | ")
            .next()
            .unwrap_or(&commit.message);
        let selector = if is_selected { "> " } else { "  " };
        let line1 = Line::from(Span::styled(
            format!("{}{}", selector, description),
            highlight,
        ));
        frame.render_widget(Paragraph::new(line1), Rect::new(inner.x, y, inner.width, 1));

        // Line 2: Formatted date/time
        let datetime = chrono::DateTime::from_timestamp(commit.timestamp, 0)
            .map(|dt| {
                dt.with_timezone(&chrono::Local)
                    .format("%b %d, %Y  %l:%M %p")
                    .to_string()
            })
            .unwrap_or_else(|| "Unknown".to_string());
        let line2 = Line::from(Span::styled(format!("    {}", datetime), dim));
        frame.render_widget(
            Paragraph::new(line2),
            Rect::new(inner.x, y + 1, inner.width, 1),
        );

        // Line 3: Status line
        let hours = commit.playtime / 3600;
        let minutes = (commit.playtime % 3600) / 60;
        let playtime_str = format!("{}h {:02}m", hours, minutes);
        let status = format!(
            "    Lv{} | P{} | Zone {} | {}",
            commit.level, commit.prestige, commit.zone, playtime_str
        );
        let line3 = Line::from(Span::styled(status, dim));
        frame.render_widget(
            Paragraph::new(line3),
            Rect::new(inner.x, y + 2, inner.width, 1),
        );

        y += card_height;
    }
}

/// Render the bottom controls bar.
fn draw_controls(frame: &mut Frame, area: Rect, state: &TimeVaultState) {
    let controls = match &state.mode {
        BrowserMode::ConfirmRestore => Line::from(vec![
            Span::styled(
                " [Enter] Confirm Restore ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "[Esc] Cancel ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        BrowserMode::ConfirmDelete => {
            let name = state.selected_branch_name().unwrap_or("?").to_string();
            Line::from(vec![
                Span::styled(
                    format!(" [Enter] Delete '{name}' "),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    "[Esc] Cancel ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        }
        BrowserMode::NamingFork { .. } => {
            let input_display = format!("Name: {}_ ", state.fork_name_input);
            if let Some(err) = &state.fork_name_error {
                Line::from(vec![
                    Span::styled(
                        format!(" {input_display}"),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw("  "),
                    Span::styled(err.clone(), Style::default().fg(Color::Red)),
                    Span::raw("  "),
                    Span::styled("[Esc] Cancel ", Style::default().fg(Color::Green)),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        format!(" {input_display}"),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw("  "),
                    Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
                    Span::styled("Create", Style::default().fg(Color::DarkGray)),
                    Span::raw("  "),
                    Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
                    Span::styled("Cancel", Style::default().fg(Color::DarkGray)),
                ])
            }
        }
        BrowserMode::Browse => match state.focus {
            PanelFocus::Left => {
                let mut spans = vec![
                    Span::styled(" [Enter] ", Style::default().fg(Color::Cyan)),
                    Span::styled("Switch", Style::default().fg(Color::DarkGray)),
                ];
                // Only show Delete if branch is deletable (not main, not active)
                if !state.selected_branch_is_main() && !state.selected_branch_is_active() {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled("[D] ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled("Delete", Style::default().fg(Color::DarkGray)));
                }
                spans.push(Span::raw("  "));
                spans.push(Span::styled("[Tab] ", Style::default().fg(Color::Cyan)));
                spans.push(Span::styled("Saves", Style::default().fg(Color::DarkGray)));
                spans.push(Span::raw("  "));
                spans.push(Span::styled("[Esc] ", Style::default().fg(Color::Cyan)));
                spans.push(Span::styled("Close", Style::default().fg(Color::DarkGray)));
                Line::from(spans)
            }
            PanelFocus::Right => Line::from(vec![
                Span::styled(" [Enter] ", Style::default().fg(Color::Cyan)),
                Span::styled("Restore", Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::styled("[F] ", Style::default().fg(Color::Cyan)),
                Span::styled("Fork", Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::styled("[Tab] ", Style::default().fg(Color::Cyan)),
                Span::styled("Branches", Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
                Span::styled("Close", Style::default().fg(Color::DarkGray)),
            ]),
        },
    };

    let paragraph = Paragraph::new(controls).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}
