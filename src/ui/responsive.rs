//! Responsive layout infrastructure.
//!
//! Provides terminal size classification and a `LayoutContext` struct
//! that is created once per frame and threaded through all draw functions.

use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

/// Terminal size tier — determined once per frame, passed everywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SizeTier {
    TooSmall,
    S,  // 40x16+
    M,  // 60x24+
    L,  // 80x30+
    XL, // 120x40+
}

// Threshold constants
const XL_MIN_COLS: u16 = 120;
const XL_MIN_ROWS: u16 = 40;
const L_MIN_COLS: u16 = 80;
const L_MIN_ROWS: u16 = 30;
const M_MIN_COLS: u16 = 60;
const M_MIN_ROWS: u16 = 24;
const S_MIN_COLS: u16 = 40;
const S_MIN_ROWS: u16 = 16;

/// Independent width/height tier — allows "L-width but M-height" combinations.
#[derive(Debug, Clone, Copy)]
pub struct LayoutContext {
    pub width_tier: SizeTier,
    pub height_tier: SizeTier,
    /// The effective tier: min(width_tier, height_tier).
    /// Use this when a single tier value is needed.
    pub tier: SizeTier,
    /// Raw terminal dimensions for fine-grained decisions.
    pub cols: u16,
    pub rows: u16,
}

impl LayoutContext {
    /// Create a LayoutContext from the current frame dimensions.
    pub fn from_frame(frame: &Frame) -> Self {
        let size = frame.size();
        Self::from_size(size.width, size.height)
    }

    /// Create a LayoutContext from explicit dimensions (for testing).
    pub fn from_size(cols: u16, rows: u16) -> Self {
        let width_tier = classify(cols, XL_MIN_COLS, L_MIN_COLS, M_MIN_COLS, S_MIN_COLS);
        let height_tier = classify(rows, XL_MIN_ROWS, L_MIN_ROWS, M_MIN_ROWS, S_MIN_ROWS);
        let tier = width_tier.min(height_tier);

        LayoutContext {
            width_tier,
            height_tier,
            tier,
            cols,
            rows,
        }
    }
}

fn classify(val: u16, xl: u16, l: u16, m: u16, s: u16) -> SizeTier {
    if val >= xl {
        SizeTier::XL
    } else if val >= l {
        SizeTier::L
    } else if val >= m {
        SizeTier::M
    } else if val >= s {
        SizeTier::S
    } else {
        SizeTier::TooSmall
    }
}

/// Render a "terminal too small" message when below minimum size (40x16).
pub fn render_too_small(frame: &mut Frame, ctx: &LayoutContext) {
    let area = frame.size();
    frame.render_widget(Clear, area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Terminal too small",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "Need: {}x{}   Have: {}x{}",
                S_MIN_COLS, S_MIN_ROWS, ctx.cols, ctx.rows
            ),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Please resize your terminal.",
            Style::default().fg(Color::White),
        )),
    ];

    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(text, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xl_classification() {
        let ctx = LayoutContext::from_size(120, 40);
        assert_eq!(ctx.width_tier, SizeTier::XL);
        assert_eq!(ctx.height_tier, SizeTier::XL);
        assert_eq!(ctx.tier, SizeTier::XL);
    }

    #[test]
    fn test_l_classification() {
        let ctx = LayoutContext::from_size(80, 30);
        assert_eq!(ctx.width_tier, SizeTier::L);
        assert_eq!(ctx.height_tier, SizeTier::L);
        assert_eq!(ctx.tier, SizeTier::L);
    }

    #[test]
    fn test_m_classification() {
        let ctx = LayoutContext::from_size(60, 24);
        assert_eq!(ctx.width_tier, SizeTier::M);
        assert_eq!(ctx.height_tier, SizeTier::M);
        assert_eq!(ctx.tier, SizeTier::M);
    }

    #[test]
    fn test_s_classification() {
        let ctx = LayoutContext::from_size(40, 16);
        assert_eq!(ctx.width_tier, SizeTier::S);
        assert_eq!(ctx.height_tier, SizeTier::S);
        assert_eq!(ctx.tier, SizeTier::S);
    }

    #[test]
    fn test_too_small() {
        let ctx = LayoutContext::from_size(39, 20);
        assert_eq!(ctx.width_tier, SizeTier::TooSmall);
        assert_eq!(ctx.height_tier, SizeTier::S);
        assert_eq!(ctx.tier, SizeTier::TooSmall);
    }

    #[test]
    fn test_too_small_height() {
        let ctx = LayoutContext::from_size(100, 15);
        assert_eq!(ctx.width_tier, SizeTier::L);
        assert_eq!(ctx.height_tier, SizeTier::TooSmall);
        assert_eq!(ctx.tier, SizeTier::TooSmall);
    }

    #[test]
    fn test_mixed_tiers() {
        let ctx = LayoutContext::from_size(120, 22);
        assert_eq!(ctx.width_tier, SizeTier::XL);
        assert_eq!(ctx.height_tier, SizeTier::S);
        assert_eq!(ctx.tier, SizeTier::S); // min of both
    }

    #[test]
    fn test_boundary_values() {
        // Exact boundary values
        let ctx = LayoutContext::from_size(119, 39);
        assert_eq!(ctx.width_tier, SizeTier::L);
        assert_eq!(ctx.height_tier, SizeTier::L);
        assert_eq!(ctx.tier, SizeTier::L);

        let ctx = LayoutContext::from_size(79, 29);
        assert_eq!(ctx.width_tier, SizeTier::M);
        assert_eq!(ctx.height_tier, SizeTier::M);
        assert_eq!(ctx.tier, SizeTier::M);

        let ctx = LayoutContext::from_size(59, 23);
        assert_eq!(ctx.width_tier, SizeTier::S);
        assert_eq!(ctx.height_tier, SizeTier::S);
        assert_eq!(ctx.tier, SizeTier::S);
    }

    #[test]
    fn test_zero_dimensions() {
        let ctx = LayoutContext::from_size(0, 0);
        assert_eq!(ctx.tier, SizeTier::TooSmall);
    }

    #[test]
    fn test_raw_dimensions_stored() {
        let ctx = LayoutContext::from_size(100, 35);
        assert_eq!(ctx.cols, 100);
        assert_eq!(ctx.rows, 35);
    }
}
