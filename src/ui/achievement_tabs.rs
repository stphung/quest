//! Achievement browser category tab rendering.

use crate::achievements::{AchievementCategory, Achievements, Act};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::achievement_browser_scene::AchievementBrowserState;

/// Build the label string for a single category tab.
fn tab_label(cat: AchievementCategory, achievements: &Achievements) -> String {
    if cat == AchievementCategory::Stats {
        " Stats ".to_string()
    } else {
        let (unlocked, total) = achievements.count_by_category(cat);
        let new_count = achievements.count_recently_unlocked_by_category(cat);
        if new_count > 0 {
            format!(" {} ({}/{}) +{} ", cat.name(), unlocked, total, new_count)
        } else {
            format!(" {} ({}/{}) ", cat.name(), unlocked, total)
        }
    }
}

/// Build one act's selector label: name + unlocked/total across its
/// categories.
fn act_label(act: Act, achievements: &Achievements) -> String {
    let (unlocked, total) = act
        .categories()
        .iter()
        .map(|c| achievements.count_by_category(*c))
        .fold((0usize, 0usize), |(au, at), (u, t)| (au + u, at + t));
    format!(" {} ({}/{}) ", act.name(), unlocked, total)
}

/// Render the act selector row above the subsection tabs. Act II's label
/// dims while the act is dark-shipped (its rows stay browsable — the
/// visible-but-unearnable teaser ruling, docs/decisions.md 2026-07-13).
pub(super) fn render_act_row(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    achievements: &Achievements,
    ui_state: &AchievementBrowserState,
) {
    let mut spans: Vec<Span> = Vec::new();
    for act in [Act::ActI, Act::ActII] {
        let selected = act == ui_state.selected_act;
        let dark = act == Act::ActII && !crate::vessel::act2_enabled();
        let style = if selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if dark {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(if selected { "\u{25B6}" } else { " " }, style));
        spans.push(Span::styled(act_label(act, achievements), style));
        spans.push(Span::raw("  "));
    }
    let row = Paragraph::new(Line::from(spans)).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(row, area);
}

/// Render category tabs at the top of the achievement browser.
/// Scrolls horizontally when tabs exceed available width, keeping the
/// selected category visible.
pub(super) fn render_category_tabs(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    achievements: &Achievements,
    ui_state: &AchievementBrowserState,
) {
    let categories = ui_state.selected_act.categories();
    let available = area.width as usize;

    // Build labels and measure widths.
    let labels: Vec<String> = categories
        .iter()
        .map(|c| tab_label(*c, achievements))
        .collect();
    let widths: Vec<usize> = labels.iter().map(|l| l.len()).collect();
    let total_width: usize = widths.iter().sum();

    let selected_idx = categories
        .iter()
        .position(|c| *c == ui_state.selected_category)
        .unwrap_or(0);

    if total_width <= available {
        // Everything fits — render centered.
        let spans: Vec<Span> = categories
            .iter()
            .zip(labels.iter())
            .map(|(cat, label)| {
                Span::styled(label.clone(), tab_style(*cat == ui_state.selected_category))
            })
            .collect();
        let tabs = Paragraph::new(Line::from(spans)).alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(tabs, area);
    } else {
        // Scroll so the selected tab is visible.
        // Compute the pixel start of each tab and centre the selected one.
        let mut starts: Vec<usize> = Vec::with_capacity(categories.len());
        let mut acc = 0usize;
        for w in &widths {
            starts.push(acc);
            acc += w;
        }

        // Target: centre the selected tab in the viewport.
        let sel_mid = starts[selected_idx] + widths[selected_idx] / 2;
        let scroll = if sel_mid > available / 2 {
            (sel_mid - available / 2).min(total_width.saturating_sub(available))
        } else {
            0
        };

        // Render with scroll indicator arrows.
        let mut spans: Vec<Span> = Vec::new();
        if scroll > 0 {
            spans.push(Span::styled(
                "\u{25C0} ",
                Style::default().fg(Color::DarkGray),
            ));
        }

        // Walk tabs and emit only the visible portion.
        for (i, (cat, label)) in categories.iter().zip(labels.iter()).enumerate() {
            let tab_start = starts[i];
            let tab_end = tab_start + widths[i];

            // Skip tabs entirely before the viewport.
            if tab_end <= scroll {
                continue;
            }
            // Stop if we've gone past the viewport.
            let arrow_reserve = if scroll > 0 { 2 } else { 0 }
                + if scroll + available < total_width {
                    2
                } else {
                    0
                };
            let viewport = available.saturating_sub(arrow_reserve);
            let emitted: usize = spans
                .iter()
                .skip(if scroll > 0 { 1 } else { 0 })
                .map(|s| s.width())
                .sum();
            if emitted >= viewport {
                break;
            }

            let style = tab_style(*cat == ui_state.selected_category);

            // Clip left edge if tab is partially scrolled.
            if tab_start < scroll {
                let clip = scroll - tab_start;
                let visible: String = label.chars().skip(clip).collect();
                let max_chars = viewport.saturating_sub(emitted);
                let truncated: String = visible.chars().take(max_chars).collect();
                spans.push(Span::styled(truncated, style));
            } else {
                let max_chars = viewport.saturating_sub(emitted);
                if label.len() <= max_chars {
                    spans.push(Span::styled(label.clone(), style));
                } else {
                    let truncated: String = label.chars().take(max_chars).collect();
                    spans.push(Span::styled(truncated, style));
                }
            }
        }

        if scroll + available < total_width {
            spans.push(Span::styled(
                " \u{25B6}",
                Style::default().fg(Color::DarkGray),
            ));
        }

        let tabs = Paragraph::new(Line::from(spans));
        frame.render_widget(tabs, area);
    }
}

fn tab_style(selected: bool) -> Style {
    if selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_label_stats() {
        let achievements = Achievements::default();
        let label = tab_label(AchievementCategory::Stats, &achievements);
        assert_eq!(label, " Stats ");
    }

    #[test]
    fn test_tab_label_combat() {
        let achievements = Achievements::default();
        let label = tab_label(AchievementCategory::Combat, &achievements);
        assert!(label.contains("Combat"));
        assert!(label.contains("(0/"));
    }

    #[test]
    fn test_total_tab_width_exceeds_narrow_terminal() {
        let achievements = Achievements::default();
        let labels: Vec<String> = AchievementCategory::ALL
            .iter()
            .map(|c| tab_label(*c, &achievements))
            .collect();
        let total: usize = labels.iter().map(|l| l.len()).sum();
        // With 9 categories the tabs should be wide enough to need scrolling
        // on a typical 80-column half-panel (~40 cols).
        assert!(total > 40, "total tab width {} should exceed 40", total);
    }
}
