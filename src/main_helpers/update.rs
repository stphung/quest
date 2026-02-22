//! Update check helpers.

use crate::core::constants::{UPDATE_CHECK_INTERVAL_SECONDS, UPDATE_CHECK_JITTER_SECONDS};
use crate::ui::throbber::block_spinner_char;
use crate::utils::updater::UpdateInfo;
use rand::RngExt;
use ratatui::crossterm::event;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::thread::JoinHandle;
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

fn draw_startup_modal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    text: Vec<Line<'static>>,
) -> io::Result<()> {
    terminal
        .draw(|frame| {
            let area = frame.area();
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Quest ");

            let inner = block.inner(area);
            frame.render_widget(block, area);

            let paragraph = Paragraph::new(text).alignment(ratatui::layout::Alignment::Left);
            frame.render_widget(paragraph, inner);
        })
        .map(|_| ())
}

fn build_startup_splash_text(
    update_info: Option<&UpdateInfo>,
    update_loading: bool,
) -> Vec<Line<'static>> {
    let q = Style::default()
        .fg(Color::Rgb(78, 217, 255))
        .add_modifier(Modifier::BOLD);
    let u = Style::default()
        .fg(Color::Rgb(102, 170, 255))
        .add_modifier(Modifier::BOLD);
    let e = Style::default()
        .fg(Color::Rgb(196, 132, 255))
        .add_modifier(Modifier::BOLD);
    let s = Style::default()
        .fg(Color::Rgb(255, 170, 102))
        .add_modifier(Modifier::BOLD);
    let t = Style::default()
        .fg(Color::Rgb(255, 222, 102))
        .add_modifier(Modifier::BOLD);

    let accent = Style::default().fg(Color::DarkGray);
    let mut text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  █████   ", q),
            Span::styled("██   ██  ", u),
            Span::styled("███████  ", e),
            Span::styled(" ██████  ", s),
            Span::styled("███████", t),
        ]),
        Line::from(vec![
            Span::styled(" ██   ██  ", q),
            Span::styled("██   ██  ", u),
            Span::styled("██       ", e),
            Span::styled("██       ", s),
            Span::styled("  ██   ", t),
        ]),
        Line::from(vec![
            Span::styled(" ██   ██  ", q),
            Span::styled("██   ██  ", u),
            Span::styled("█████    ", e),
            Span::styled(" █████   ", s),
            Span::styled("  ██   ", t),
        ]),
        Line::from(vec![
            Span::styled(" ██  ███  ", q),
            Span::styled("██   ██  ", u),
            Span::styled("██       ", e),
            Span::styled("     ██  ", s),
            Span::styled("  ██   ", t),
        ]),
        Line::from(vec![
            Span::styled("  ████ ██  ", q),
            Span::styled(" █████   ", u),
            Span::styled("███████  ", e),
            Span::styled("██████   ", s),
            Span::styled("  ██   ", t),
        ]),
        Line::from(vec![
            Span::styled("  ▀▀▀▀▀▀▀  ", q),
            Span::styled("▀▀▀▀▀▀▀  ", u),
            Span::styled("▀▀▀▀▀▀▀  ", e),
            Span::styled("▀▀▀▀▀▀  ", s),
            Span::styled("▀▀▀▀▀▀ ", t),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", accent),
            Span::styled("▀▄", Style::default().fg(Color::Rgb(78, 217, 255))),
            Span::styled(
                "  Welcome back, adventurer.  ",
                Style::default().fg(Color::White),
            ),
            Span::styled("▄▀", Style::default().fg(Color::Rgb(255, 170, 102))),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Sharpen your blade. Loot awaits.",
            Style::default().fg(Color::Gray),
        )]),
        Line::from(""),
    ];

    let upstream_title = if let Some(info) = update_info {
        format!("  Upstream (v{} ({}))", info.new_version, info.new_commit)
    } else {
        "  Upstream".to_string()
    };
    text.push(Line::from(vec![Span::styled(
        upstream_title,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));

    if update_loading {
        text.push(Line::from(vec![Span::styled(
            format!(
                "    {} Checking for update details...",
                block_spinner_char()
            ),
            Style::default().fg(Color::DarkGray),
        )]));
    } else if let Some(info) = update_info {
        if info.changelog.is_empty() {
            text.push(Line::from(vec![Span::styled(
                "    You're already up to date.",
                Style::default().fg(Color::Gray),
            )]));
        } else {
            let max_upstream_items = 5;
            for item in info.changelog.iter().take(max_upstream_items) {
                text.push(Line::from(vec![
                    Span::styled("    • ", Style::default().fg(Color::DarkGray)),
                    Span::styled(item.clone(), Style::default().fg(Color::White)),
                ]));
            }
            if info.changelog_total > max_upstream_items {
                text.push(Line::from(vec![Span::styled(
                    "    ...and more",
                    Style::default().fg(Color::DarkGray),
                )]));
            }
        }

        text.push(Line::from(""));
        text.push(Line::from(vec![Span::styled(
            "  Run 'quest update' when you're ready.",
            Style::default().fg(Color::Green),
        )]));
    } else {
        text.push(Line::from(vec![Span::styled(
            "    Could not load update details right now.",
            Style::default().fg(Color::Gray),
        )]));
    }

    let installed_title = if let Some(info) = update_info {
        format!(
            "  Installed (v{} ({}))",
            info.current_version, info.current_commit
        )
    } else {
        "  Installed".to_string()
    };
    text.push(Line::from(""));
    text.push(Line::from(vec![Span::styled(
        installed_title,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));

    if update_loading {
        text.push(Line::from(vec![Span::styled(
            format!("    {} Loading version history...", block_spinner_char()),
            Style::default().fg(Color::DarkGray),
        )]));
    } else if let Some(info) = update_info {
        for item in &info.current_and_previous {
            text.push(Line::from(vec![
                Span::styled("    • ", Style::default().fg(Color::DarkGray)),
                Span::styled(item.clone(), Style::default().fg(Color::White)),
            ]));
        }
        if info.current_and_previous.is_empty() {
            text.push(Line::from(vec![Span::styled(
                "    No version history available.",
                Style::default().fg(Color::Gray),
            )]));
        }
    } else {
        text.push(Line::from(vec![Span::styled(
            "    Version history unavailable.",
            Style::default().fg(Color::Gray),
        )]));
    }

    text.push(Line::from(""));
    text.push(Line::from(vec![Span::styled(
        "  Press any key to continue...",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));

    text
}

/// Show the startup splash screen while update data loads in the background.
/// Pressing any key continues immediately; update loading never blocks launch.
pub fn show_startup_splash_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    update_check_handle: &mut Option<JoinHandle<Option<UpdateInfo>>>,
) -> io::Result<()> {
    let mut update_info: Option<UpdateInfo> = None;

    loop {
        if update_info.is_none() {
            let finished = update_check_handle
                .as_ref()
                .is_some_and(std::thread::JoinHandle::is_finished);
            if finished {
                if let Some(handle) = update_check_handle.take() {
                    update_info = handle.join().ok().flatten();
                }
            }
        }

        let update_loading = update_info.is_none() && update_check_handle.is_some();
        let text = build_startup_splash_text(update_info.as_ref(), update_loading);
        draw_startup_modal(terminal, text)?;

        if event::poll(Duration::from_millis(100))?
            && matches!(event::read()?, event::Event::Key(_))
        {
            break;
        }
    }

    Ok(())
}
