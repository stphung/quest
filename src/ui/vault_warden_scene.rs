//! Vault Warden minigame UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_minigame_too_small, render_status_bar, GameResultType,
};
use crate::challenges::menu::DifficultyInfo;
use crate::challenges::vault_warden::logic::is_crate_deadlocked;
use crate::challenges::vault_warden::types::Cell;
use crate::challenges::vault_warden::{VaultWardenGame, VaultWardenResult};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

const BORDER_COLOR: Color = Color::Rgb(180, 140, 40);

pub fn render_vault_warden(
    frame: &mut Frame,
    area: Rect,
    game: &VaultWardenGame,
    ctx: &super::responsive::LayoutContext,
    show_dismiss_hint: bool,
) {
    if game.game_result.is_some() {
        render_game_over(frame, area, game, show_dismiss_hint);
        return;
    }

    let grid_display_width = game.width as u16 * 2;
    let grid_display_height = game.height as u16;
    let min_width = grid_display_width + 6;
    let min_height = grid_display_height + 6;

    if area.width < min_width || area.height < min_height {
        render_minigame_too_small(frame, area, "Vault Warden", min_width, min_height);
        return;
    }

    let layout = create_game_layout(
        frame,
        area,
        " Vault Warden ",
        BORDER_COLOR,
        grid_display_height,
        22,
        ctx,
    );

    render_grid(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

fn render_grid(frame: &mut Frame, area: Rect, game: &VaultWardenGame) {
    let grid_display_width = game.width as u16 * 2;
    let grid_display_height = game.height as u16;

    let x_offset = area.x + (area.width.saturating_sub(grid_display_width)) / 2;
    let y_offset = area.y + (area.height.saturating_sub(grid_display_height)) / 2;

    for row in 0..game.height {
        let mut spans = Vec::new();

        for col in 0..game.width {
            let pos = (row, col);
            let emoji = if pos == game.player_pos {
                "\u{1F9D9}" // 🧙
            } else if game.has_crate_at(pos) {
                if game.is_goal(pos) {
                    "\u{2705}" // ✅ crate on goal
                } else if is_crate_deadlocked(game, pos) {
                    "\u{1F7E5}" // 🟥 deadlocked
                } else {
                    "\u{1F4E6}" // 📦 crate
                }
            } else if game.is_goal(pos) {
                "\u{2B50}" // ⭐ goal
            } else if game.grid[row][col] == Cell::Wall {
                "\u{2B1C}" // ⬜ wall
            } else {
                "\u{2B1B}" // ⬛ floor
            };

            spans.push(Span::raw(emoji));
        }

        let line = Line::from(spans);
        let y = y_offset + row as u16;
        if y < area.y + area.height {
            frame.render_widget(
                Paragraph::new(vec![line]),
                Rect::new(x_offset, y, grid_display_width, 1),
            );
        }
    }
}

fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &VaultWardenGame) {
    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    render_status_bar(
        frame,
        area,
        "Arranging relics...",
        BORDER_COLOR,
        &[
            ("[Arrows]", "Move"),
            ("[Z]", "Undo"),
            ("[R]", "Restart"),
            ("[Esc]", "Forfeit"),
        ],
    );
}

fn render_info_panel(frame: &mut Frame, area: Rect, game: &VaultWardenGame) {
    let inner = render_info_panel_frame(frame, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Difficulty ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(BORDER_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Grid    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}×{}", game.width, game.height),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Placed  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.crates_on_goals(), game.total_crates()),
                Style::default().fg(if game.crates_on_goals() == game.total_crates() {
                    Color::Green
                } else {
                    Color::White
                }),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Moves   ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", game.moves), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Restarts", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(" {}/{}", game.attempts_remaining, game.attempts_max),
                Style::default().fg(if game.attempts_remaining == 0 {
                    Color::Red
                } else if game.attempts_remaining <= 1 {
                    Color::Yellow
                } else {
                    Color::White
                }),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Undos   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(" {}/{}", game.undos_remaining, game.undos_max),
                Style::default().fg(if game.undos_remaining == 0 {
                    Color::Red
                } else {
                    Color::White
                }),
            ),
        ]),
    ];

    lines.truncate(inner.height as usize);
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_game_over(
    frame: &mut Frame,
    area: Rect,
    game: &VaultWardenGame,
    show_dismiss_hint: bool,
) {
    let (result_type, title, message, reward) = match game.game_result {
        Some(VaultWardenResult::Win) => {
            let r = game.difficulty.reward();
            let reward_text = if r.prestige_ranks > 0 {
                format!(
                    "+{} Prestige Ranks, +{} Stormglass",
                    r.prestige_ranks, r.stormglass
                )
            } else {
                format!("+{} Stormglass", r.stormglass)
            };
            (
                GameResultType::Win,
                "VAULT SEALED!".to_string(),
                format!("Solved in {} moves", game.moves),
                reward_text,
            )
        }
        _ => (
            GameResultType::Loss,
            "VAULT BREACHED!".to_string(),
            "Out of restart attempts!".to_string(),
            "No penalty incurred.".to_string(),
        ),
    };
    render_game_over_overlay(
        frame,
        area,
        result_type,
        &title,
        &message,
        &reward,
        show_dismiss_hint,
    );
}
