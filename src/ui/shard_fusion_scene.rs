//! Shard Fusion (2048-style) game UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_status_bar, GameResultType,
};
use crate::challenges::menu::DifficultyInfo;
use crate::challenges::shard_fusion::{ShardFusionAnimState, ShardFusionGame, ShardFusionResult};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Map a tile value to its display color.
fn tile_color(value: u32) -> Color {
    match value {
        0 => Color::DarkGray,
        2 | 4 => Color::Gray,
        8 | 16 => Color::Yellow,
        32 | 64 => Color::LightGreen,
        128 | 256 => Color::LightCyan,
        512 | 1024 => Color::LightMagenta,
        _ => Color::LightRed, // 2048+
    }
}

/// Render the Shard Fusion scene.
pub fn render_shard_fusion_scene(
    frame: &mut Frame,
    area: Rect,
    game: &ShardFusionGame,
    ctx: &super::responsive::LayoutContext,
    show_dismiss_hint: bool,
    stormglass_discovered: bool,
) {
    if game.game_result.is_some() {
        render_shard_fusion_game_over(frame, area, game, show_dismiss_hint, stormglass_discovered);
        return;
    }

    // Each cell needs ~3 rows (1 content + top/bottom borders); 4 rows = ~12.
    // Add 2 for padding comfort.
    let layout = create_game_layout(frame, area, " Shard Fusion ", Color::Yellow, 14, 24, ctx);

    render_board(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

/// Render the 4×4 game board.
fn render_board(frame: &mut Frame, area: Rect, game: &ShardFusionGame) {
    // Split into 4 equal rows.
    let row_constraints = [
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
    ];
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);

    // Split each row into 4 equal columns.
    let col_constraints = [
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
    ];

    for r in 0..4usize {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(rows[r]);

        for c in 0..4usize {
            let value = game.board[r][c];

            // During flash phase, merged cells glow white.
            let fg_color = if matches!(game.anim_state, ShardFusionAnimState::Flashing(_))
                && game.merged_cells.contains(&(r, c))
            {
                Color::White
            } else {
                tile_color(value)
            };

            let border_color = if value == 0 {
                Color::DarkGray
            } else {
                fg_color
            };

            let content = if value == 0 {
                String::new()
            } else {
                value.to_string()
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color));

            let para = Paragraph::new(Line::from(Span::styled(
                content,
                Style::default().fg(fg_color),
            )))
            .block(block)
            .alignment(Alignment::Center);

            frame.render_widget(para, cols[c]);
        }
    }
}

/// Render the status bar below the board.
fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &ShardFusionGame) {
    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    render_status_bar(
        frame,
        area,
        "Your move",
        Color::White,
        &[("[Arrows]", "Slide"), ("[Esc]", "Forfeit")],
    );
}

/// Render the info panel on the right side.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &ShardFusionGame) {
    let inner = render_info_panel_frame(frame, area);

    let target = game.difficulty.target_value();
    let highest = game.highest_tile();
    let highest_color = tile_color(highest);

    let lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Score:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.score.to_string(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Target:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(target.to_string(), Style::default().fg(Color::LightCyan)),
        ]),
        Line::from(vec![
            Span::styled("Highest: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if highest == 0 {
                    "-".to_string()
                } else {
                    highest.to_string()
                },
                Style::default().fg(highest_color),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Diff:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::White)),
        ]),
    ];

    let text = Paragraph::new(lines);
    frame.render_widget(text, inner);
}

/// Render the game over overlay.
fn render_shard_fusion_game_over(
    frame: &mut Frame,
    area: Rect,
    game: &ShardFusionGame,
    show_dismiss_hint: bool,
    stormglass_discovered: bool,
) {
    let Some(result) = game.game_result else {
        return;
    };

    let (result_type, title, message) = match result {
        ShardFusionResult::Win => (
            GameResultType::Win,
            "SHARDS FUSED!",
            format!(
                "You merged the crystal shards to {}! Score: {}",
                game.difficulty.target_value(),
                game.score
            ),
        ),
        ShardFusionResult::Loss if game.forfeit_pending => (
            GameResultType::Forfeit,
            "FUSION ABANDONED",
            format!(
                "You stepped away with a highest tile of {}.",
                game.highest_tile()
            ),
        ),
        ShardFusionResult::Loss => (
            GameResultType::Loss,
            "FUSION FAILED",
            format!(
                "No more moves remain. Highest tile: {}.",
                game.highest_tile()
            ),
        ),
    };

    let reward_text = if result == ShardFusionResult::Win {
        game.difficulty.reward().description(stormglass_discovered)
    } else {
        "No reward".to_string()
    };

    render_game_over_overlay(
        frame,
        area,
        result_type,
        title,
        &message,
        &reward_text,
        show_dismiss_hint,
    );
}
