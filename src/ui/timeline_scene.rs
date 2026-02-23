//! Timeline browser overlay UI.
//!
//! Displays a two-panel overlay for browsing save history branches (timelines)
//! and their commits. Players can restore any previous snapshot.

use crate::history::types::{CommitInfo, TimelineInfo};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Which panel has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    Left,
    Right,
}

/// The current interaction mode of the timeline browser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserMode {
    /// Normal browsing — arrow keys navigate, Tab switches focus.
    Browse,
    /// Waiting for confirmation to restore the selected commit.
    ConfirmRestore,
    /// Waiting for confirmation to delete the selected branch.
    ConfirmDelete,
    /// Typing a name for a new forked timeline.
    NamingFork { commit_id: String },
}

/// UI state for the timeline browser overlay.
pub struct TimelineBrowserState {
    pub branches: Vec<TimelineInfo>,
    pub selected_branch: usize,
    pub commits: Vec<CommitInfo>,
    pub selected_commit: usize,
    pub focus: PanelFocus,
    pub mode: BrowserMode,
    pub fork_name_input: String,
    pub fork_name_error: Option<String>,
}

impl TimelineBrowserState {
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
        self.branches.get(self.selected_branch).map(|b| b.name.as_str())
    }

    /// Short SHA of the currently selected commit, if any.
    pub fn selected_commit_id(&self) -> Option<&str> {
        self.commits.get(self.selected_commit).map(|c| c.id.as_str())
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

/// Render the timeline browser overlay.
pub fn draw_timeline_browser(frame: &mut Frame, area: Rect, state: &TimelineBrowserState) {
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
                " TIMELINE BROWSER ",
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
fn draw_branch_panel(frame: &mut Frame, area: Rect, state: &TimelineBrowserState, focused: bool) {
    let border_color = if focused { Color::Cyan } else { Color::DarkGray };
    let title_color = if focused { Color::Cyan } else { Color::White };
    let block = Block::default()
        .title(Span::styled(
            " Timelines ",
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
fn draw_commit_panel(frame: &mut Frame, area: Rect, state: &TimelineBrowserState, focused: bool) {
    let border_color = if focused { Color::Cyan } else { Color::DarkGray };
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
fn draw_controls(frame: &mut Frame, area: Rect, state: &TimelineBrowserState) {
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
            let name = state
                .selected_branch_name()
                .unwrap_or("?")
                .to_string();
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
                Span::styled("Timelines", Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
                Span::styled("Close", Style::default().fg(Color::DarkGray)),
            ]),
        },
    };

    let paragraph = Paragraph::new(controls).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}
