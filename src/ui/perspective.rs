use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

/// Renders the dungeon ceiling with gradient
pub fn render_ceiling(width: usize, height: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for i in 0..height {
        // Darker at top, lighter at bottom
        let darkness = 1.0 - (i as f64 / height as f64);
        let char_density = if darkness > 0.75 {
            '░'
        } else if darkness > 0.5 {
            '▒'
        } else if darkness > 0.25 {
            '▓'
        } else {
            '█'
        };

        let line_str = char_density.to_string().repeat(width);
        lines.push(Line::from(Span::styled(
            line_str,
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines
}

/// Renders perspective floor with grid
pub fn render_floor(width: usize, height: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for i in 0..height {
        // Perspective: wider at bottom, narrower at top
        let perspective = (i as f64 / height as f64).powi(2);
        let line_width = (width as f64 * (0.4 + perspective * 0.6)) as usize;
        let padding = (width - line_width) / 2;

        // Create floor line with grid pattern
        let mut line_str = " ".repeat(padding);

        for j in 0..line_width {
            // Grid lines every 4 chars
            if j % 4 == 0 {
                line_str.push('═');
            } else if i % 2 == 0 && j % 2 == 0 {
                line_str.push('▓');
            } else {
                line_str.push('░');
            }
        }

        line_str.push_str(&" ".repeat(width - padding - line_width));

        lines.push(Line::from(Span::styled(
            line_str,
            Style::default().fg(Color::Gray),
        )));
    }

    lines
}

/// Renders stone walls with perspective
pub fn render_walls(width: usize, height: usize) -> (Vec<String>, Vec<String>) {
    let mut left_wall = Vec::new();
    let mut right_wall = Vec::new();

    for i in 0..height {
        // Wall width increases toward bottom (perspective)
        let perspective = (i as f64 / height as f64).powi(2);
        let wall_width = (width as f64 * 0.15 * (0.5 + perspective * 0.5)) as usize;

        // Left wall (stone texture)
        let mut left_line = String::new();
        for j in 0..wall_width {
            let char = match (i + j) % 4 {
                0 => '█',
                1 => '▓',
                2 => '▒',
                _ => '░',
            };
            left_line.push(char);
        }

        // Right wall (mirror of left)
        let right_line = left_line.chars().rev().collect();

        left_wall.push(left_line);
        right_wall.push(right_line);
    }

    (left_wall, right_wall)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_ceiling() {
        let ceiling = render_ceiling(40, 5);
        assert_eq!(ceiling.len(), 5);
        // Top should be lighter (░)
        assert!(ceiling[0].to_string().contains('░'));
    }

    #[test]
    fn test_render_floor() {
        let floor = render_floor(40, 5);
        assert_eq!(floor.len(), 5);
        // Should contain grid lines
        assert!(floor[4].to_string().contains('═'));
    }

    #[test]
    fn test_render_walls() {
        let (left, right) = render_walls(40, 5);
        assert_eq!(left.len(), 5);
        assert_eq!(right.len(), 5);
        // Walls should get wider toward bottom
        assert!(left[4].len() > left[0].len());
    }

    #[test]
    fn test_wall_perspective() {
        let (left, _) = render_walls(40, 10);
        // Bottom wall should be wider than top
        assert!(left[9].len() > left[0].len());
    }
}
