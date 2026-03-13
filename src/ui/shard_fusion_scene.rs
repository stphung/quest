//! Shard Fusion (2048-style) game UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_minigame_too_small, render_status_bar, GameResultType,
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

/// Fixed tile dimensions (terminal chars are ~2:1 col:row, so 3 rows × 6 cols ≈ square).
const CELL_ROWS: u16 = 3;
const CELL_COLS: u16 = 7; // 7 gives inner width of 5, comfortable for "4096"

/// Total board pixel dimensions.
const BOARD_ROWS: u16 = CELL_ROWS * 4;
const BOARD_COLS: u16 = CELL_COLS * 4;

/// Center a fixed-size rect inside a larger area.
fn center_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

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

    // Board is BOARD_ROWS tall + 2 for status bar + 2 for outer border.
    let min_height = BOARD_ROWS + 4;
    let min_width = BOARD_COLS + 26; // board + info panel
    if area.width < min_width || area.height < min_height {
        render_minigame_too_small(frame, area, "Shard Fusion", min_width, min_height);
        return;
    }

    let layout = create_game_layout(
        frame,
        area,
        " Shard Fusion ",
        Color::Yellow,
        BOARD_ROWS,
        24,
        ctx,
    );

    render_board(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

/// Render the 4×4 game board.
///
/// Tiles are fixed-size squares centered within the content area. The board
/// reflects the post-slide state; slide_moves is reserved for future animation
/// work. The Flashing phase provides merge visual feedback.
fn render_board(frame: &mut Frame, area: Rect, game: &ShardFusionGame) {
    // Center the fixed-size board grid in the available content area.
    let board_area = center_rect(BOARD_COLS, BOARD_ROWS, area);

    let row_constraints = [Constraint::Length(CELL_ROWS); 4];
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(board_area);

    let col_constraints = [Constraint::Length(CELL_COLS); 4];

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

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color));

            // Vertically center the number: inner height = CELL_ROWS - 2 (borders).
            // Pad with blank lines above so the number sits in the middle row.
            let inner_height = (CELL_ROWS.saturating_sub(2)) as usize;
            let text_row = inner_height / 2;
            let mut lines: Vec<Line> = (0..inner_height).map(|_| Line::from("")).collect();
            if value != 0 && text_row < inner_height {
                lines[text_row] = Line::from(Span::styled(
                    value.to_string(),
                    Style::default().fg(fg_color),
                ));
            }

            let para = Paragraph::new(lines)
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
    if area.width < 2 || area.height < 2 {
        return;
    }
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
