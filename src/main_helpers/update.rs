//! Update check helpers.

use crate::core::constants::{UPDATE_CHECK_INTERVAL_SECONDS, UPDATE_CHECK_JITTER_SECONDS};
use crate::utils::updater::UpdateInfo;
use rand::RngExt;
use ratatui::crossterm::event;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

/// Returns the update check interval with random jitter applied.
/// Jitter spreads checks across [base - jitter, base + jitter] to avoid
/// simultaneous API requests from many clients.
pub fn jittered_update_interval() -> Duration {
    let mut rng = rand::rng();
    let jitter = rng.random_range(0..=2 * UPDATE_CHECK_JITTER_SECONDS);
    let interval = UPDATE_CHECK_INTERVAL_SECONDS - UPDATE_CHECK_JITTER_SECONDS + jitter;
    Duration::from_secs(interval)
}

/// Show update notification with changelog at startup, then wait for keypress.
pub fn show_startup_update_notification(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    update_info: &UpdateInfo,
) -> io::Result<()> {
    terminal.draw(|frame| {
        let area = frame.area();
        let block = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
            .title(" Update Available ");

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut text = vec![
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(format!(
                "  New version: {} ({})",
                update_info.new_version, update_info.new_commit
            )),
            ratatui::text::Line::from(""),
        ];

        if !update_info.changelog.is_empty() {
            text.push(ratatui::text::Line::from("  What's new:"));
            for entry in update_info.changelog.iter().take(15) {
                text.push(ratatui::text::Line::from(format!("    \u{2022} {}", entry)));
            }
            if update_info.changelog.len() > 15 {
                text.push(ratatui::text::Line::from(format!(
                    "    ...and {} more",
                    update_info.changelog.len() - 15
                )));
            }
            text.push(ratatui::text::Line::from(""));
        }

        text.push(ratatui::text::Line::from(
            "  Run 'quest update' to install.",
        ));
        text.push(ratatui::text::Line::from(""));
        text.push(ratatui::text::Line::from("  Press any key to continue..."));

        let paragraph =
            ratatui::widgets::Paragraph::new(text).alignment(ratatui::layout::Alignment::Left);

        frame.render_widget(paragraph, inner);
    })?;

    // Wait for keypress (max 5 seconds)
    let _ = event::poll(Duration::from_secs(5));
    if event::poll(Duration::from_millis(0))? {
        let _ = event::read()?;
    }
    Ok(())
}
