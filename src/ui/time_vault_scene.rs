//! Time Vault overlay UI.
//!
//! Displays a two-panel overlay for browsing save branches
//! and their snapshots. Players can restore, fork, and manage saves.

use crate::history::types::{CommitInfo, TimelineInfo};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::scene_fx::{
    current_millis, hash2d, lerp_rgb, put_cell, put_text, render_buffer, SceneCell,
};

/// Temporal backdrop: dark navy top.
const VAULT_TOP_RGB: (u8, u8, u8) = (8, 12, 35);
/// Temporal backdrop: near-black bottom.
const VAULT_BOTTOM_RGB: (u8, u8, u8) = (3, 5, 15);
/// Dim cyan for timeline graph lines.
const TIMELINE_DIM: Color = Color::Rgb(40, 80, 120);
/// Number of drifting particles.
const PARTICLE_COUNT: usize = 5;
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
    /// Waiting for confirmation to switch to the selected branch.
    ConfirmSwitch,
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

    // Render bordered block first to get inner area
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

    // Buffer dimensions (inner minus 1 row for controls bar)
    let buf_w = inner.width as usize;
    let buf_h = inner.height.saturating_sub(1) as usize;
    if buf_w < 10 || buf_h < 5 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); buf_w]; buf_h];
    let millis = current_millis();
    paint_vault_backdrop(&mut buffer, millis);

    // Layout: branch panel on the left, snapshot panel on the right
    let branch_width = 20usize.min(buf_w / 3);
    let snap_x = branch_width + 1; // 1 col gap
    let snap_w = buf_w.saturating_sub(snap_x);

    paint_branch_panel(&mut buffer, state, branch_width);
    paint_snapshot_panel(&mut buffer, state, snap_x, snap_w);

    // Overlay confirmation dialog when not browsing
    if state.mode != BrowserMode::Browse {
        paint_confirm_dialog(&mut buffer, state);
    }

    // Render the scene buffer into the inner area (above controls)
    let buffer_area = Rect::new(inner.x, inner.y, inner.width, inner.height.saturating_sub(1));
    render_buffer(frame, buffer_area, &buffer);

    // Controls bar at the bottom
    let controls_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(1),
        inner.width,
        1,
    );
    draw_controls(frame, controls_area, state);
}

/// Paint the branch list into the scene buffer.
fn paint_branch_panel(buffer: &mut [Vec<SceneCell>], state: &TimeVaultState, width: usize) {
    let height = buffer.len();
    let focused = state.focus == PanelFocus::Left;

    // Panel title
    let title_color = if focused { Color::Cyan } else { Color::White };
    put_text(buffer, 0, 1, "Branches", title_color);

    // Thin separator
    let sep_color = if focused {
        Color::Rgb(40, 80, 120)
    } else {
        Color::DarkGray
    };
    let sep: String = "\u{2500}".repeat(width.saturating_sub(1));
    put_text(buffer, 1, 0, &sep, sep_color);

    // Branch list
    for (i, branch) in state.branches.iter().enumerate() {
        let row = 2 + i as i32;
        if row >= height as i32 {
            break;
        }

        let is_selected = i == state.selected_branch;

        // Highlight background for selected row
        if is_selected && focused {
            let r = row as usize;
            if r < height {
                let highlight_bg = Color::Rgb(25, 40, 80);
                for col in 0..width {
                    if col < buffer[r].len() {
                        buffer[r][col].bg = highlight_bg;
                    }
                }
            }
        }

        let marker = if branch.is_active {
            "\u{25cf}" // ● active
        } else {
            "\u{25cb}" // ○ inactive
        };

        let marker_color = if branch.is_active {
            Color::Green
        } else if is_selected {
            Color::Yellow
        } else {
            Color::DarkGray
        };
        let name_style = if is_selected {
            Color::Yellow
        } else if branch.is_active {
            Color::Green
        } else {
            Color::White
        };

        put_text(buffer, row, 1, marker, marker_color);
        let label = format!(" {}", branch.name);
        put_text(buffer, row, 3, &label, name_style);
    }
}

/// Paint the snapshot timeline into the scene buffer.
fn paint_snapshot_panel(
    buffer: &mut [Vec<SceneCell>],
    state: &TimeVaultState,
    x_offset: usize,
    width: usize,
) {
    let height = buffer.len();
    let focused = state.focus == PanelFocus::Right;

    // Panel title — shows which branch is being viewed
    let title_color = if focused { Color::Cyan } else { Color::White };
    let branch_name = state.selected_branch_name().unwrap_or("?");
    let title = if state.selected_branch_is_active() {
        format!("Snapshots ({})", branch_name)
    } else {
        format!("Snapshots (viewing: {})", branch_name)
    };
    put_text(buffer, 0, x_offset as i32 + 1, &title, title_color);

    // Thin separator
    let sep_color = if focused {
        Color::Rgb(40, 80, 120)
    } else {
        Color::DarkGray
    };
    let sep: String = "\u{2500}".repeat(width.saturating_sub(1));
    put_text(buffer, 1, x_offset as i32, &sep, sep_color);

    if state.commits.is_empty() {
        put_text(
            buffer,
            3,
            x_offset as i32 + 2,
            "No snapshots yet",
            Color::DarkGray,
        );
        return;
    }

    // Each card: 4 rows (description, date, stats, separator)
    let card_height = 4usize;
    let available_rows = height.saturating_sub(2); // below title+sep
    let visible_cards = (available_rows / card_height).max(1);

    // Scroll so selected commit is visible
    let scroll_offset = if state.selected_commit >= visible_cards {
        state.selected_commit - visible_cards + 1
    } else {
        0
    };

    let x = x_offset as i32;
    let mut row = 2i32; // start below title + separator
    let total_visible = state.commits.len().saturating_sub(scroll_offset);

    for (vi, (i, commit)) in state
        .commits
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .enumerate()
    {
        if row + 3 > height as i32 {
            break;
        }

        let is_selected = i == state.selected_commit;
        let is_last = vi == total_visible - 1 || row + 4 + 3 > height as i32;
        let (icon, icon_color) = event_icon_color(&commit.message);

        // Highlight background for selected card
        if is_selected && focused {
            let highlight_bg = Color::Rgb(25, 40, 80);
            for r in row..row + 3 {
                let r = r as usize;
                if r < height {
                    for col in x_offset..x_offset + width {
                        if col < buffer[r].len() {
                            buffer[r][col].bg = highlight_bg;
                        }
                    }
                }
            }
        }

        // Timeline node
        let node = if is_selected {
            "\u{25cf}" // filled circle
        } else {
            "\u{25cb}" // open circle
        };
        let node_color = if is_selected {
            Color::Yellow
        } else {
            Color::Cyan
        };
        put_text(buffer, row, x + 2, node, node_color);

        // Icon
        put_text(buffer, row, x + 4, icon, icon_color);

        // Description
        let desc = commit.message.split(" | ").next().unwrap_or(&commit.message);
        let desc_color = if is_selected {
            Color::Yellow
        } else {
            Color::White
        };
        let icon_width = super::scene_fx::display_width(icon);
        put_text(
            buffer,
            row,
            x + 4 + icon_width as i32 + 1,
            desc,
            desc_color,
        );

        // Timeline connector for rows below
        let dim = if is_selected {
            Color::Yellow
        } else {
            Color::DarkGray
        };

        // Date line
        let connector = if is_last { " " } else { "\u{2502}" }; // vertical line
        put_text(buffer, row + 1, x + 2, connector, TIMELINE_DIM);
        let datetime = chrono::DateTime::from_timestamp(commit.timestamp, 0)
            .map(|dt| {
                dt.with_timezone(&chrono::Local)
                    .format("%b %d, %Y  %l:%M %p")
                    .to_string()
            })
            .unwrap_or_else(|| "Unknown".to_string());
        put_text(buffer, row + 1, x + 6, &datetime, dim);

        // Stats line
        let connector2 = if is_last { " " } else { "\u{2502}" }; // vertical line
        put_text(buffer, row + 2, x + 2, connector2, TIMELINE_DIM);
        let hours = commit.playtime / 3600;
        let minutes = (commit.playtime % 3600) / 60;
        let stats = format!(
            "Lv{} \u{00b7} P{} \u{00b7} Zone {} \u{00b7} {}h {:02}m",
            commit.level, commit.prestige, commit.zone, hours, minutes
        );
        put_text(buffer, row + 2, x + 6, &stats, dim);

        // Separator line (thin line with connector)
        if !is_last {
            put_text(buffer, row + 3, x + 2, "\u{2502}", TIMELINE_DIM); // vertical line
            let card_sep: String = "\u{2500}".repeat(width.saturating_sub(8));
            put_text(buffer, row + 3, x + 4, &card_sep, Color::Rgb(30, 50, 80));
        }

        row += card_height as i32;
    }
}

/// Paint a centered confirmation dialog into the scene buffer.
fn paint_confirm_dialog(buffer: &mut [Vec<SceneCell>], state: &TimeVaultState) {
    let buf_h = buffer.len();
    if buf_h == 0 {
        return;
    }
    let buf_w = buffer[0].len();

    // Dialog dimensions vary by mode
    let dialog_w = 44usize.min(buf_w.saturating_sub(4));
    let dialog_h = match &state.mode {
        BrowserMode::ConfirmRestore => 10usize,
        BrowserMode::ConfirmSwitch | BrowserMode::ConfirmDelete => 7,
        BrowserMode::NamingFork { .. } => {
            if state.fork_name_error.is_some() {
                9
            } else {
                8
            }
        }
        BrowserMode::Browse => return,
    }
    .min(buf_h.saturating_sub(2));

    // Center in buffer
    let dx = (buf_w.saturating_sub(dialog_w)) / 2;
    let dy = (buf_h.saturating_sub(dialog_h)) / 2;

    let bg = Color::Rgb(12, 16, 40);
    let border_color = Color::Cyan;

    // Fill background
    for row_cells in buffer.iter_mut().skip(dy).take(dialog_h) {
        for cell in row_cells.iter_mut().skip(dx).take(dialog_w) {
            *cell = SceneCell::new(' ', Color::Reset, bg);
        }
    }

    // Border (box-drawing characters)
    let top = dy as i32;
    let bottom = (dy + dialog_h - 1) as i32;
    let left = dx as i32;
    let right = (dx + dialog_w - 1) as i32;

    put_cell(buffer, top, left, '\u{250c}', border_color);
    put_cell(buffer, top, right, '\u{2510}', border_color);
    put_cell(buffer, bottom, left, '\u{2514}', border_color);
    put_cell(buffer, bottom, right, '\u{2518}', border_color);
    for col in (dx + 1)..(dx + dialog_w - 1) {
        put_cell(buffer, top, col as i32, '\u{2500}', border_color);
        put_cell(buffer, bottom, col as i32, '\u{2500}', border_color);
    }
    for row in (dy + 1)..(dy + dialog_h - 1) {
        put_cell(buffer, row as i32, left, '\u{2502}', border_color);
        put_cell(buffer, row as i32, right, '\u{2502}', border_color);
    }

    // Content area starts inside border + padding
    let cx = left + 3;
    let cy = top + 2;

    match &state.mode {
        BrowserMode::ConfirmRestore => {
            put_text(buffer, cy, cx, "Restore to this save?", Color::White);

            if let Some(commit) = state.commits.get(state.selected_commit) {
                let (icon, icon_color) = event_icon_color(&commit.message);
                let desc = commit.message.split(" | ").next().unwrap_or(&commit.message);

                put_text(buffer, cy + 2, cx, icon, icon_color);
                let iw = super::scene_fx::display_width(icon);
                put_text(buffer, cy + 2, cx + iw as i32 + 1, desc, Color::Yellow);

                let datetime = chrono::DateTime::from_timestamp(commit.timestamp, 0)
                    .map(|dt| {
                        dt.with_timezone(&chrono::Local)
                            .format("%b %d, %Y  %l:%M %p")
                            .to_string()
                    })
                    .unwrap_or_else(|| "Unknown".to_string());
                put_text(buffer, cy + 3, cx, &datetime, Color::DarkGray);

                let hours = commit.playtime / 3600;
                let minutes = (commit.playtime % 3600) / 60;
                let stats = format!(
                    "Lv{} \u{00b7} P{} \u{00b7} Zone {} \u{00b7} {}h {:02}m",
                    commit.level, commit.prestige, commit.zone, hours, minutes
                );
                put_text(buffer, cy + 4, cx, &stats, Color::DarkGray);
            }

            put_text(buffer, cy + 6, cx, "[Enter]", Color::Red);
            put_text(buffer, cy + 6, cx + 8, "Confirm", Color::DarkGray);
            put_text(buffer, cy + 6, cx + 18, "[Esc]", Color::Green);
            put_text(buffer, cy + 6, cx + 24, "Cancel", Color::DarkGray);
        }
        BrowserMode::ConfirmSwitch => {
            let name = state.selected_branch_name().unwrap_or("?");
            let title = format!("Switch to '{}'?", name);
            put_text(buffer, cy, cx, &title, Color::Yellow);

            put_text(buffer, cy + 3, cx, "[Enter]", Color::Red);
            put_text(buffer, cy + 3, cx + 8, "Confirm", Color::DarkGray);
            put_text(buffer, cy + 3, cx + 18, "[Esc]", Color::Green);
            put_text(buffer, cy + 3, cx + 24, "Cancel", Color::DarkGray);
        }
        BrowserMode::ConfirmDelete => {
            let name = state.selected_branch_name().unwrap_or("?");
            let title = format!("Delete branch '{}'?", name);
            put_text(buffer, cy, cx, &title, Color::Red);
            put_text(buffer, cy + 1, cx, "This cannot be undone.", Color::DarkGray);

            put_text(buffer, cy + 3, cx, "[Enter]", Color::Red);
            put_text(buffer, cy + 3, cx + 8, "Delete", Color::DarkGray);
            put_text(buffer, cy + 3, cx + 17, "[Esc]", Color::Green);
            put_text(buffer, cy + 3, cx + 23, "Cancel", Color::DarkGray);
        }
        BrowserMode::NamingFork { .. } => {
            put_text(buffer, cy, cx, "Fork new branch", Color::White);

            let input_display = format!("Name: {}_", state.fork_name_input);
            put_text(buffer, cy + 2, cx, &input_display, Color::Yellow);

            let mut ctrl_row = cy + 4;
            if let Some(err) = &state.fork_name_error {
                put_text(buffer, cy + 3, cx, err, Color::Red);
                ctrl_row = cy + 5;
            }

            put_text(buffer, ctrl_row, cx, "[Enter]", Color::Cyan);
            put_text(buffer, ctrl_row, cx + 8, "Create", Color::DarkGray);
            put_text(buffer, ctrl_row, cx + 17, "[Esc]", Color::Cyan);
            put_text(buffer, ctrl_row, cx + 23, "Cancel", Color::DarkGray);
        }
        BrowserMode::Browse => {}
    }
}

/// Render the bottom controls bar.
fn draw_controls(frame: &mut Frame, area: Rect, state: &TimeVaultState) {
    // Non-Browse modes show the dialog overlay — no footer controls needed.
    let controls = match &state.mode {
        BrowserMode::ConfirmRestore
        | BrowserMode::ConfirmSwitch
        | BrowserMode::ConfirmDelete
        | BrowserMode::NamingFork { .. } => return,
        BrowserMode::Browse => {
            let dot = Span::styled(
                "  \u{00b7}  ",
                Style::default().fg(Color::Rgb(40, 80, 120)),
            );
            match state.focus {
                PanelFocus::Left => {
                    let mut spans = vec![
                        Span::styled(" [Enter] ", Style::default().fg(Color::Cyan)),
                        Span::styled("Switch", Style::default().fg(Color::DarkGray)),
                        dot.clone(),
                        Span::styled("[F] ", Style::default().fg(Color::Cyan)),
                        Span::styled("Fork", Style::default().fg(Color::DarkGray)),
                    ];
                    if !state.selected_branch_is_main() && !state.selected_branch_is_active() {
                        spans.push(dot.clone());
                        spans.push(Span::styled("[D] ", Style::default().fg(Color::Cyan)));
                        spans.push(Span::styled(
                            "Delete",
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                    spans.push(dot.clone());
                    spans.push(Span::styled("[Tab] ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled("Saves", Style::default().fg(Color::DarkGray)));
                    spans.push(dot);
                    spans.push(Span::styled("[Esc] ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled("Close", Style::default().fg(Color::DarkGray)));
                    Line::from(spans)
                }
                PanelFocus::Right => {
                    let enter_label = if state.selected_branch_is_active() {
                        "Restore"
                    } else {
                        "Switch to branch"
                    };
                    Line::from(vec![
                        Span::styled(" [Enter] ", Style::default().fg(Color::Cyan)),
                        Span::styled(enter_label, Style::default().fg(Color::DarkGray)),
                        dot.clone(),
                        Span::styled("[F] ", Style::default().fg(Color::Cyan)),
                        Span::styled("Fork", Style::default().fg(Color::DarkGray)),
                        dot.clone(),
                        Span::styled("[Tab] ", Style::default().fg(Color::Cyan)),
                        Span::styled("Branches", Style::default().fg(Color::DarkGray)),
                        dot,
                        Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
                        Span::styled("Close", Style::default().fg(Color::DarkGray)),
                    ])
                }
            }
        }
    };

    let paragraph = Paragraph::new(controls).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}
