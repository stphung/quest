#![allow(dead_code)]
use ratatui::layout::Rect;

/// Layout for a centered overlay with title and footer regions.
pub struct OverlayLayout {
    pub outer: Rect,
    pub title_bar: Rect,
    pub content: Rect,
    pub footer: Rect,
}

/// Compute a centered overlay layout.
/// `max_width` and `max_height` cap the overlay dimensions.
/// `title_height` rows for the title bar (typically 1-3).
/// `footer_height` rows for the footer (typically 1-2).
pub fn centered_overlay(
    area: Rect,
    max_width: u16,
    max_height: u16,
    title_height: u16,
    footer_height: u16,
) -> OverlayLayout {
    let w = max_width.min(area.width);
    let h = max_height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let outer = Rect::new(x, y, w, h);
    let title_bar = Rect::new(x, y, w, title_height);
    let content_y = y + title_height;
    let content_h = h.saturating_sub(title_height + footer_height);
    let content = Rect::new(x, content_y, w, content_h);
    let footer = Rect::new(x, y + h.saturating_sub(footer_height), w, footer_height);
    OverlayLayout {
        outer,
        title_bar,
        content,
        footer,
    }
}

/// Layout for a two-panel split (list on left, detail on right).
pub struct TwoPanelLayout {
    pub left: Rect,
    pub right: Rect,
}

/// Split an area into two panels by percentage.
/// `left_pct` is 0-100, the percentage of width for the left panel.
pub fn two_panel_split(area: Rect, left_pct: u16) -> TwoPanelLayout {
    let left_w = (area.width as u32 * left_pct as u32 / 100) as u16;
    let right_w = area.width.saturating_sub(left_w);
    TwoPanelLayout {
        left: Rect::new(area.x, area.y, left_w, area.height),
        right: Rect::new(area.x + left_w, area.y, right_w, area.height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centered_overlay_fits_within_area() {
        let area = Rect::new(0, 0, 120, 40);
        let layout = centered_overlay(area, 80, 30, 2, 1);
        assert_eq!(layout.outer.width, 80);
        assert_eq!(layout.outer.height, 30);
        assert_eq!(layout.outer.x, 20); // centered: (120 - 80) / 2
        assert_eq!(layout.outer.y, 5); // centered: (40 - 30) / 2
        assert_eq!(layout.title_bar.height, 2);
        assert_eq!(layout.footer.height, 1);
        assert_eq!(layout.content.height, 27); // 30 - 2 - 1
    }

    #[test]
    fn test_centered_overlay_clamps_to_area() {
        let area = Rect::new(0, 0, 40, 20);
        let layout = centered_overlay(area, 80, 30, 2, 1);
        assert_eq!(layout.outer.width, 40); // clamped to area
        assert_eq!(layout.outer.height, 20); // clamped to area
    }

    #[test]
    fn test_two_panel_split_50_50() {
        let area = Rect::new(10, 5, 100, 30);
        let layout = two_panel_split(area, 50);
        assert_eq!(layout.left.width, 50);
        assert_eq!(layout.right.width, 50);
        assert_eq!(layout.left.x, 10);
        assert_eq!(layout.right.x, 60);
    }

    #[test]
    fn test_two_panel_split_preserves_area() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = two_panel_split(area, 30);
        assert_eq!(layout.left.width + layout.right.width, 80);
        assert_eq!(layout.left.height, 24);
        assert_eq!(layout.right.height, 24);
    }
}
