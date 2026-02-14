//! Containment Breach scene rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_minigame_too_small, render_status_bar, GameResultType,
};
use crate::challenges::jezzball::types::{
    ActiveWall, JezzballGame, JezzballResult, Position, WallOrientation,
};
use crate::challenges::menu::DifficultyInfo;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

const BORDER_H: char = '\u{2500}'; // ─
const BORDER_V: char = '\u{2502}'; // │
const BORDER_TL: char = '\u{250C}'; // ┌
const BORDER_TR: char = '\u{2510}'; // ┐
const BORDER_BL: char = '\u{2514}'; // └
const BORDER_BR: char = '\u{2518}'; // ┘

fn axis_glyph(orientation: WallOrientation) -> &'static str {
    match orientation {
        WallOrientation::Horizontal => "━",
        WallOrientation::Vertical => "┃",
    }
}

fn axis_preview_glyph(orientation: WallOrientation) -> &'static str {
    match orientation {
        WallOrientation::Horizontal => "─",
        WallOrientation::Vertical => "│",
    }
}

fn axis_color(orientation: WallOrientation) -> Color {
    match orientation {
        WallOrientation::Horizontal => Color::Yellow,
        WallOrientation::Vertical => Color::Cyan,
    }
}

/// Render Containment Breach scene.
pub fn render_jezzball_scene(
    frame: &mut Frame,
    area: Rect,
    game: &JezzballGame,
    ctx: &super::responsive::LayoutContext,
) {
    if game.game_result.is_some() {
        render_jezzball_game_over(frame, area, game);
        return;
    }

    const MIN_WIDTH: u16 = 34;
    const MIN_HEIGHT: u16 = 18;
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_minigame_too_small(frame, area, "Containment Breach", MIN_WIDTH, MIN_HEIGHT);
        return;
    }

    let layout = create_game_layout(
        frame,
        area,
        " Containment Breach ",
        Color::Cyan,
        18,
        22,
        ctx,
    );

    render_board(frame, layout.content, game);

    if game.waiting_to_start {
        render_start_prompt(frame, layout.content);
    }

    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

fn render_board(frame: &mut Frame, area: Rect, game: &JezzballGame) {
    if area.width < 5 || area.height < 5 {
        return;
    }

    let grid_w = game.grid_width as u16;
    let grid_h = game.grid_height as u16;

    let render_w = (grid_w + 2).min(area.width);
    let render_h = (grid_h + 2).min(area.height);

    if render_w < 4 || render_h < 4 {
        return;
    }

    let visible_w = render_w - 2;
    let visible_h = render_h - 2;

    let x_off = area.x + (area.width.saturating_sub(render_w)) / 2;
    let y_off = area.y + (area.height.saturating_sub(render_h)) / 2;

    // Top border
    {
        let mut top = String::new();
        top.push(BORDER_TL);
        for _ in 0..visible_w {
            top.push(BORDER_H);
        }
        top.push(BORDER_TR);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                top,
                Style::default().fg(Color::DarkGray),
            ))),
            Rect::new(x_off, y_off, render_w, 1),
        );
    }

    let active_wall = game.active_wall;

    // Cells
    for y in 0..visible_h {
        let gy = y as i16;
        let mut spans = vec![Span::styled(
            BORDER_V.to_string(),
            Style::default().fg(Color::DarkGray),
        )];

        for x in 0..visible_w {
            let gx = x as i16;
            let pos = Position { x: gx, y: gy };
            let cell_style = cell_style(game, pos, active_wall);
            spans.push(Span::styled(cell_style.0, cell_style.1));
        }

        spans.push(Span::styled(
            BORDER_V.to_string(),
            Style::default().fg(Color::DarkGray),
        ));

        frame.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(x_off, y_off + 1 + y, render_w, 1),
        );
    }

    // Bottom border
    {
        let mut bot = String::new();
        bot.push(BORDER_BL);
        for _ in 0..visible_w {
            bot.push(BORDER_H);
        }
        bot.push(BORDER_BR);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                bot,
                Style::default().fg(Color::DarkGray),
            ))),
            Rect::new(x_off, y_off + render_h - 1, render_w, 1),
        );
    }
}

fn cell_style(
    game: &JezzballGame,
    pos: Position,
    active_wall: Option<ActiveWall>,
) -> (&'static str, Style) {
    if game
        .balls
        .iter()
        .any(|ball| ball.x.floor() as i16 == pos.x && ball.y.floor() as i16 == pos.y)
    {
        return (
            "●",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        );
    }

    if let Some(wall) = active_wall {
        if wall_contains_cell(wall, pos) {
            return (
                axis_glyph(wall.orientation),
                Style::default()
                    .fg(axis_color(wall.orientation))
                    .add_modifier(Modifier::BOLD),
            );
        }
    }

    if pos.y >= 0
        && pos.y < game.grid_height
        && pos.x >= 0
        && pos.x < game.grid_width
        && game.blocked[pos.y as usize][pos.x as usize]
    {
        return (
            "█",
            Style::default()
                .fg(Color::Rgb(60, 140, 190))
                .add_modifier(Modifier::BOLD),
        );
    }

    if game.active_wall.is_none() && game.cursor == pos {
        return (
            axis_glyph(game.orientation),
            Style::default()
                .fg(axis_color(game.orientation))
                .add_modifier(Modifier::BOLD),
        );
    }

    if game.active_wall.is_none() {
        let on_axis = match game.orientation {
            WallOrientation::Horizontal => pos.y == game.cursor.y,
            WallOrientation::Vertical => pos.x == game.cursor.x,
        };
        if on_axis {
            return (
                axis_preview_glyph(game.orientation),
                Style::default().fg(Color::Rgb(72, 72, 90)),
            );
        }
    }

    ("·", Style::default().fg(Color::Rgb(28, 28, 36)))
}

fn wall_contains_cell(wall: ActiveWall, pos: Position) -> bool {
    match wall.orientation {
        WallOrientation::Horizontal => {
            pos.y == wall.pivot.y
                && pos.x >= wall.pivot.x - wall.neg_extent
                && pos.x <= wall.pivot.x + wall.pos_extent
        }
        WallOrientation::Vertical => {
            pos.x == wall.pivot.x
                && pos.y >= wall.pivot.y - wall.neg_extent
                && pos.y <= wall.pivot.y + wall.pos_extent
        }
    }
}

fn render_start_prompt(frame: &mut Frame, area: Rect) {
    if area.height < 5 || area.width < 20 {
        return;
    }

    let prompt = "[ Press Space to Start ]";
    let x = area.x + area.width.saturating_sub(prompt.len() as u16) / 2;
    let y = area.y + area.height / 2;

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            prompt,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))),
        Rect::new(x, y, prompt.len() as u16, 1),
    );
}

fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &JezzballGame) {
    let axis_label = format!(
        "Axis: {} {}",
        game.orientation.name(),
        axis_glyph(game.orientation)
    );

    if game.waiting_to_start {
        render_status_bar(
            frame,
            area,
            &format!("Containment ready - {}", axis_label),
            Color::Cyan,
            &[
                ("[Arrows]", "Move"),
                ("[X]", "Axis"),
                ("[Space]", "Start"),
                ("[Esc]", "Forfeit"),
            ],
        );
        return;
    }

    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    if game.active_wall.is_some() {
        render_status_bar(
            frame,
            area,
            "Wall extending... avoid contact",
            Color::Yellow,
            &[("[Esc]", "Forfeit")],
        );
        return;
    }

    render_status_bar(
        frame,
        area,
        &format!("Capture territory - {}", axis_label),
        Color::LightCyan,
        &[
            ("[Arrows]", "Move"),
            ("[X]", "Axis"),
            ("[Space/Enter]", "Build"),
            ("[Esc]", "Forfeit"),
        ],
    );
}

fn render_info_panel(frame: &mut Frame, area: Rect, game: &JezzballGame) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let inner = render_info_panel_frame(frame, area);

    let lines = vec![
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Captured: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:>5.1}%", game.captured_percent),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Target: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}%", game.target_percent),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("Hazards: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.balls.len().to_string(),
                Style::default().fg(Color::LightRed),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Axis: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{} {}",
                    game.orientation.name(),
                    axis_glyph(game.orientation)
                ),
                Style::default()
                    .fg(axis_color(game.orientation))
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Cursor: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}, {}", game.cursor.x, game.cursor.y),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Legend",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(" ● ", Style::default().fg(Color::LightRed)),
            Span::styled("Hazard", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {} ", axis_glyph(game.orientation)),
                Style::default().fg(axis_color(game.orientation)),
            ),
            Span::styled("Building", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(" █ ", Style::default().fg(Color::Rgb(60, 140, 190))),
            Span::styled("Sealed", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {} ", axis_glyph(game.orientation)),
                Style::default().fg(axis_color(game.orientation)),
            ),
            Span::styled("Axis cursor", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_jezzball_game_over(frame: &mut Frame, area: Rect, game: &JezzballGame) {
    let result = game.game_result.expect("game_result checked before call");

    let (result_type, title, message, reward) = match result {
        JezzballResult::Win => {
            let reward_text = game.difficulty.reward().description();
            (
                GameResultType::Win,
                ":: CONTAINMENT ACHIEVED ::",
                format!(
                    "Arena secured at {:.1}% (target {}%).",
                    game.captured_percent, game.target_percent
                ),
                reward_text,
            )
        }
        JezzballResult::Loss => {
            let result_type = if game.forfeit_pending {
                GameResultType::Forfeit
            } else {
                GameResultType::Loss
            };
            (
                result_type,
                "CONTAINMENT FAILURE",
                format!(
                    "The field destabilized at {:.1}% captured.",
                    game.captured_percent
                ),
                "No penalty incurred.".to_string(),
            )
        }
    };

    render_game_over_overlay(frame, area, result_type, title, &message, &reward);
}
