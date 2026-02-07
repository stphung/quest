//! Shared board styling utilities for grid-based game UIs.
//!
//! This module provides consistent styling for board game elements like
//! cursors, pieces, last moves, and empty cells across all game scenes.

use ratatui::style::{Color, Modifier, Style};

/// Standard colors used across board games.
#[derive(Clone, Copy)]
pub struct BoardColors {
    /// Color for human player pieces
    pub human: Color,
    /// Color for AI/opponent pieces
    pub ai: Color,
    /// Color for cursor highlight
    pub cursor: Color,
    /// Color for last move indicator
    pub last_move: Color,
    /// Color for empty cells/grid
    pub empty: Color,
    /// Color for winning line highlight
    pub winning: Color,
}

impl Default for BoardColors {
    fn default() -> Self {
        Self {
            human: Color::White,
            ai: Color::LightRed,
            cursor: Color::Yellow,
            last_move: Color::Green,
            empty: Color::DarkGray,
            winning: Color::Magenta,
        }
    }
}

impl BoardColors {
    /// Create a style for a piece at the given position.
    ///
    /// Handles cursor highlighting, last move, and winning line states.
    pub fn piece_style(
        &self,
        is_human: bool,
        is_cursor: bool,
        is_last_move: bool,
        is_winning: bool,
    ) -> Style {
        let base_color = if is_human { self.human } else { self.ai };
        let mut style = Style::default().add_modifier(Modifier::BOLD);

        if is_winning {
            style = style.fg(self.winning);
        } else if is_last_move {
            style = style.fg(self.last_move);
        } else {
            style = style.fg(base_color);
        }

        if is_cursor && !is_winning {
            style = style.bg(Color::DarkGray);
        }

        style
    }

    /// Create a style for an empty cell at the given position.
    pub fn empty_style(&self, is_cursor: bool) -> Style {
        if is_cursor {
            Style::default()
                .fg(self.cursor)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.empty)
        }
    }

    /// Create a style for an empty cell cursor symbol.
    pub fn cursor_style(&self) -> Style {
        Style::default()
            .fg(self.cursor)
            .add_modifier(Modifier::BOLD)
    }
}

/// Calculate centering offsets for a board within an area.
///
/// Returns (x_offset, y_offset) for positioning the board.
pub fn calculate_board_centering(
    area_x: u16,
    area_y: u16,
    area_width: u16,
    area_height: u16,
    board_width: u16,
    board_height: u16,
) -> (u16, u16) {
    let x_offset = area_x + (area_width.saturating_sub(board_width)) / 2;
    let y_offset = area_y + (area_height.saturating_sub(board_height)) / 2;
    (x_offset, y_offset)
}

/// Common symbols used in board games.
pub mod symbols {
    /// Filled circle (used for pieces in Go, Gomoku, etc.)
    pub const FILLED_CIRCLE: &str = "●";
    /// Open circle (used for opponent in Go)
    pub const OPEN_CIRCLE: &str = "○";
    /// Square (used for cursor on empty)
    pub const CURSOR_SQUARE: &str = "□";
    /// Dot (used for empty intersections)
    pub const EMPTY_DOT: &str = "·";
    /// Cross (used for ko point in Go)
    pub const KO_MARKER: &str = "×";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_colors() {
        let colors = BoardColors::default();
        assert_eq!(colors.human, Color::White);
        assert_eq!(colors.ai, Color::LightRed);
        assert_eq!(colors.cursor, Color::Yellow);
    }

    #[test]
    fn test_piece_style_normal() {
        let colors = BoardColors::default();
        let style = colors.piece_style(true, false, false, false);
        // Should have human color and bold
        assert_eq!(style.fg, Some(Color::White));
    }

    #[test]
    fn test_piece_style_cursor() {
        let colors = BoardColors::default();
        let style = colors.piece_style(true, true, false, false);
        // Should have background for cursor
        assert_eq!(style.bg, Some(Color::DarkGray));
    }

    #[test]
    fn test_piece_style_last_move() {
        let colors = BoardColors::default();
        let style = colors.piece_style(false, false, true, false);
        // Should have last_move color
        assert_eq!(style.fg, Some(Color::Green));
    }

    #[test]
    fn test_piece_style_winning() {
        let colors = BoardColors::default();
        let style = colors.piece_style(true, true, false, true);
        // Winning takes precedence, no cursor bg when winning
        assert_eq!(style.fg, Some(Color::Magenta));
        assert_eq!(style.bg, None);
    }

    #[test]
    fn test_board_centering() {
        let (x, y) = calculate_board_centering(0, 0, 40, 20, 20, 10);
        assert_eq!(x, 10); // (40 - 20) / 2
        assert_eq!(y, 5); // (20 - 10) / 2
    }

    #[test]
    fn test_empty_style_cursor() {
        let colors = BoardColors::default();
        let style = colors.empty_style(true);
        assert_eq!(style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_empty_style_normal() {
        let colors = BoardColors::default();
        let style = colors.empty_style(false);
        assert_eq!(style.fg, Some(Color::DarkGray));
    }
}
