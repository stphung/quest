//! Achievement browser overlay UI.
//!
//! Displays a browsable list of achievements organized by category,
//! with a detail panel showing description and unlock status.

use super::game_common::format_number;
use crate::achievements::{
    get_achievement_def, AchievementCategory, AchievementId, Achievements, Act,
};
use crate::enhancement::EnhancementProgress;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// UI state for the achievement browser overlay.
pub struct AchievementBrowserState {
    pub showing: bool,
    pub selected_act: Act,
    pub selected_category: AchievementCategory,
    pub selected_index: usize,
    /// Set on act/section switches; `main.rs` consumes it to force a full
    /// terminal repaint (the wide-scene pattern) — emoji-heavy views like
    /// Stats can desync the terminal's cells from ratatui's diff model,
    /// leaving orphan glyphs when the next view paints fewer rows.
    pub needs_full_repaint: bool,
}

impl AchievementBrowserState {
    pub fn new() -> Self {
        Self {
            showing: false,
            selected_act: Act::ActI,
            selected_category: AchievementCategory::Combat,
            selected_index: 0,
            needs_full_repaint: false,
        }
    }

    /// Consume the full-repaint request (see field docs).
    pub fn take_full_repaint(&mut self) -> bool {
        std::mem::take(&mut self.needs_full_repaint)
    }

    pub fn open(&mut self) {
        self.showing = true;
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.showing = false;
    }

    /// Switch to the other act, landing on its first subsection.
    pub fn toggle_act(&mut self) {
        self.selected_act = match self.selected_act {
            Act::ActI => Act::ActII,
            Act::ActII => Act::ActI,
        };
        self.selected_category = self.selected_act.categories()[0];
        self.selected_index = 0;
        self.needs_full_repaint = true;
    }

    /// Cycle subsections within the selected act (wrapping).
    fn cycle_category(&mut self, delta: isize) {
        let categories = self.selected_act.categories();
        let current = categories
            .iter()
            .position(|cat| *cat == self.selected_category)
            .unwrap_or(0);
        let next = (current as isize + delta).rem_euclid(categories.len() as isize) as usize;
        self.selected_category = categories[next];
        self.selected_index = 0;
        self.needs_full_repaint = true;
    }

    pub fn next_category(&mut self) {
        self.cycle_category(1);
    }

    pub fn prev_category(&mut self) {
        self.cycle_category(-1);
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
    ui_state: &mut AchievementBrowserState,
    enhancement: &EnhancementProgress,
    _ctx: &super::responsive::LayoutContext,
) {
    frame.render_widget(Clear, area);

    let score = achievements.achievement_score();
    let max_score = Achievements::max_achievement_score();
    let score_pct = if max_score > 0 {
        (score as f32 / max_score as f32) * 100.0
    } else {
        0.0
    };
    let block = Block::default()
        .title(format!(
            " Achievements ({}/{} pts, {:.1}%) ",
            format_number(score as u64),
            format_number(max_score as u64),
            score_pct
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Yellow)));
    let inner =
        super::render_themed_block(frame, area, block, Color::Yellow, super::BorderFxContext);

    // Layout: act selector, the act's subsection tabs, list+detail, help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Act selector row
            Constraint::Length(2), // Subsection tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Help
        ])
        .split(inner);

    super::achievement_tabs::render_act_row(frame, chunks[0], achievements, ui_state);
    super::achievement_tabs::render_category_tabs(frame, chunks[1], achievements, ui_state);

    // Content area: stats view or list+detail
    if ui_state.selected_category == AchievementCategory::Stats {
        // Persist the renderer's clamped scroll: the input layer doesn't
        // know the viewport, so without this a held Down runs the offset
        // invisibly past the content and [Up] has to unwind the excess
        // before anything moves again.
        ui_state.selected_index = super::achievement_details::render_stats_view(
            frame,
            chunks[2],
            achievements,
            enhancement,
            ui_state.selected_index,
        );
    } else {
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(chunks[2]);

        super::achievement_list::render_achievement_list(
            frame,
            content_chunks[0],
            achievements,
            ui_state,
        );
        super::achievement_details::render_achievement_detail(
            frame,
            content_chunks[1],
            achievements,
            ui_state,
        );
    }

    let help =
        Paragraph::new("[Tab] Act  [</>] Section  [Up/Down] Select  [T] Titles  [Esc] Close")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);
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
        .border_style(Style::default().fg(super::themed_border_color(Color::Yellow)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        Color::Yellow,
        super::BorderFxContext,
    );

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
            lines.push(Line::from(Span::styled(
                format!("+{} pts", def.points),
                Style::default().fg(Color::Cyan),
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

        let total_pts: u32 = achievements
            .iter()
            .filter_map(|id| get_achievement_def(*id))
            .map(|def| def.points)
            .sum();
        lines.push(Line::from(Span::styled(
            format!("+{} pts", total_pts),
            Style::default().fg(Color::Cyan),
        )));
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
        assert_eq!(state.selected_category, AchievementCategory::Prestige);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Progression);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Challenges);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Exploration);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Deep);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Loom);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Stats);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Combat);

        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Stats);
        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Loom);
        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Deep);
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
        state.next_category(); // Prestige
        state.next_category(); // Progression
        state.next_category(); // Challenges
        state.next_category(); // Exploration
        state.next_category(); // Deep
        state.next_category(); // Loom
        state.next_category(); // Stats
        assert_eq!(state.selected_category, AchievementCategory::Stats);

        // Stats wraps to Combat
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Combat);

        // Backward from Combat goes to Stats
        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Stats);

        // Backward from Stats goes to Loom
        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Loom);

        // Backward from Loom goes to Deep
        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Deep);

        // Backward from Deep goes to Exploration
        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Exploration);
    }

    #[test]
    fn test_format_number() {
        use super::super::game_common::format_number;
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(12847), "12,847");
        assert_eq!(format_number(1000000), "1,000,000");
    }
}

#[cfg(test)]
mod stats_scroll_clamp_tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn render_at(ui: &mut AchievementBrowserState, w: u16, h: u16) {
        let ach = Achievements::default();
        let enh = crate::enhancement::EnhancementProgress::default();
        let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
        t.draw(|f| {
            let area = f.area();
            let ctx = crate::ui::responsive::LayoutContext::from_size(area.width, area.height);
            render_achievement_browser(f, area, &ach, ui, &enh, &ctx);
        })
        .unwrap();
    }

    /// Regression (reported 2026-07-14): on the Stats tab a held Down ran
    /// the scroll offset far past the content — the view stopped moving
    /// but [Up] had to unwind the invisible excess before anything
    /// scrolled back. The renderer now persists its clamped offset.
    #[test]
    fn stats_overscroll_is_clamped_back_by_the_renderer() {
        let mut ui = AchievementBrowserState::new();
        ui.selected_category = AchievementCategory::Stats;

        // Way past any real content height (the input layer allows 100).
        ui.selected_index = 100;
        render_at(&mut ui, 120, 30);
        let max_scroll = ui.selected_index;
        assert!(
            max_scroll < 100,
            "the renderer must clamp the persisted offset ({max_scroll})"
        );

        // One Up from the clamp must move immediately — that's the bug.
        ui.move_up();
        assert_eq!(ui.selected_index, max_scroll.saturating_sub(1));

        // The clamp is a fixed point: re-rendering doesn't drift it.
        ui.selected_index = max_scroll;
        render_at(&mut ui, 120, 30);
        assert_eq!(ui.selected_index, max_scroll);

        // On a terminal tall enough to show everything, no scroll at all.
        ui.selected_index = 100;
        render_at(&mut ui, 120, 200);
        assert_eq!(
            ui.selected_index, 0,
            "content that fits entirely on screen has nothing to scroll"
        );
    }
}

#[cfg(test)]
mod act_row_render_tests {
    use super::*;
    use crate::achievements::Act;
    use ratatui::{backend::TestBackend, Terminal};

    fn render_rows(ui: &mut AchievementBrowserState) -> Vec<String> {
        let ach = Achievements::default();
        let enh = crate::enhancement::EnhancementProgress::default();
        let mut t = Terminal::new(TestBackend::new(120, 20)).unwrap();
        t.draw(|f| {
            let area = f.area();
            let ctx = crate::ui::responsive::LayoutContext::from_size(area.width, area.height);
            render_achievement_browser(f, area, &ach, ui, &enh, &ctx);
        })
        .unwrap();
        let buf = t.backend().buffer();
        (0..20u16)
            .map(|y| {
                (0..120u16)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect()
    }

    /// The act row + act-scoped tabs render, and every content row keeps
    /// the panel borders vertically aligned (each border column identical
    /// across rows) — the artifact check for the Option A layout.
    #[test]
    fn act_two_layout_renders_aligned() {
        let mut ui = AchievementBrowserState::new();
        ui.toggle_act();
        assert_eq!(ui.selected_act, Act::ActII);
        let rows = render_rows(&mut ui);
        assert!(rows[1].contains("ACT II \u{00b7} The Crossing"));
        assert!(rows[2].contains("The Voyage"));
        assert!(rows.iter().any(|r| r.contains("The Burn")));
        // Border alignment: find the vertical-border columns on the first
        // list content row and require the same columns on every row of
        // the panel body.
        let body: Vec<&String> = rows[4..16]
            .iter()
            .filter(|r| !r.contains('\u{250c}') && !r.contains('\u{2514}'))
            .collect();
        let border_cols: Vec<usize> = body[0]
            .chars()
            .enumerate()
            .filter(|(_, c)| *c == '\u{2502}')
            .map(|(i, _)| i)
            .collect();
        assert!(
            border_cols.len() >= 4,
            "expected panel borders: {:?}",
            body[0]
        );
        for (i, row) in body.iter().enumerate() {
            let cols: Vec<usize> = row
                .chars()
                .enumerate()
                .filter(|(_, c)| *c == '\u{2502}')
                .map(|(j, _)| j)
                .collect();
            assert_eq!(
                cols, border_cols,
                "row {i} borders misaligned:\n{}\nvs\n{}",
                row, body[0]
            );
        }
    }
}
