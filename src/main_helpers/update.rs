//! Update check helpers.

use crate::achievements;
use crate::core::constants::{
    UPDATE_CHECK_INTERVAL_SECONDS, UPDATE_CHECK_JITTER_SECONDS, WIKI_URL,
};
use crate::enhancement;
use crate::haven;
use crate::history::HistoryRepo;
use crate::input::time_vault_input::{handle_time_vault_input, TimeVaultAction};
use crate::ui;
use crate::ui::throbber::block_spinner_char;
use crate::ui::time_vault_scene::TimeVaultState;
use crate::utils::updater::UpdateInfoStatus;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use rand::RngExt;
use ratatui::crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
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

fn parse_timestamp_utc(timestamp: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
        return Some(dt.with_timezone(&Utc));
    }

    let date = NaiveDate::parse_from_str(timestamp, "%Y-%m-%d").ok()?;
    let naive = date.and_hms_opt(0, 0, 0)?;
    Some(Utc.from_utc_datetime(&naive))
}

fn compact_relative_label(timestamp: &str, sign: char, now: &DateTime<Utc>) -> Option<String> {
    const MINUTE: i64 = 60;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    const WEEK: i64 = 7 * DAY;
    const MONTH: i64 = 30 * DAY;
    const YEAR: i64 = 365 * DAY;

    let then = parse_timestamp_utc(timestamp)?;
    let seconds = now.signed_duration_since(then).num_seconds().abs();
    let (value, unit) = if seconds >= YEAR {
        (seconds / YEAR, "y")
    } else if seconds >= MONTH {
        (seconds / MONTH, "m")
    } else if seconds >= WEEK {
        (seconds / WEEK, "w")
    } else if seconds >= DAY {
        (seconds / DAY, "d")
    } else if seconds >= HOUR {
        (seconds / HOUR, "h")
    } else {
        (seconds / MINUTE, "min")
    };

    Some(format!("[{}{}{}]", sign, value, unit))
}

fn with_relative_prefix(
    item: &str,
    timestamp: Option<&str>,
    sign: char,
    now: &DateTime<Utc>,
) -> String {
    if let Some(label) = timestamp.and_then(|ts| compact_relative_label(ts, sign, now)) {
        format!("{} {}", label, item)
    } else {
        item.to_string()
    }
}

fn build_startup_splash_text(
    update_status: Option<&UpdateInfoStatus>,
    update_loading: bool,
) -> Vec<Line<'static>> {
    use crate::utils::build_info::{BUILD_COMMIT, BUILD_DATE};
    let now = Utc::now();

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

    let upstream_title = match update_status {
        Some(UpdateInfoStatus::UpdateAvailable(info)) => {
            format!("  Upstream (v{} ({}))", info.new_version, info.new_commit)
        }
        _ => "  Upstream".to_string(),
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
    } else {
        match update_status {
            Some(UpdateInfoStatus::UpdateAvailable(info)) => {
                if info.changelog.is_empty() {
                    text.push(Line::from(vec![Span::styled(
                        "    You're already up to date.",
                        Style::default().fg(Color::Gray),
                    )]));
                } else {
                    let max_upstream_items = 5;
                    for (idx, item) in info.changelog.iter().take(max_upstream_items).enumerate() {
                        let item_text = with_relative_prefix(
                            item,
                            info.changelog_times.get(idx).and_then(|s| s.as_deref()),
                            '-',
                            &now,
                        );
                        text.push(Line::from(vec![
                            Span::styled("    • ", Style::default().fg(Color::DarkGray)),
                            Span::styled(item_text, Style::default().fg(Color::White)),
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
            }
            Some(UpdateInfoStatus::UpToDate) => {
                text.push(Line::from(vec![
                    Span::styled("    ✓ ", Style::default().fg(Color::Green)),
                    Span::styled(
                        "You're running the latest version.",
                        Style::default().fg(Color::Gray),
                    ),
                ]));
            }
            Some(UpdateInfoStatus::CheckFailed(err)) => {
                let error = err
                    .lines()
                    .next()
                    .unwrap_or("unknown error")
                    .chars()
                    .take(72)
                    .collect::<String>();
                text.push(Line::from(vec![Span::styled(
                    format!("    Could not check for updates: {}", error),
                    Style::default().fg(Color::DarkGray),
                )]));
            }
            None => {
                text.push(Line::from(vec![Span::styled(
                    "    Could not load update details right now.",
                    Style::default().fg(Color::Gray),
                )]));
            }
        }
    }

    let installed_title = match update_status {
        Some(UpdateInfoStatus::UpdateAvailable(info)) => {
            format!(
                "  Installed (v{} ({}))",
                info.current_version, info.current_commit
            )
        }
        _ => format!("  Installed (v{} ({}))", BUILD_DATE, BUILD_COMMIT),
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
    } else {
        match update_status {
            Some(UpdateInfoStatus::UpdateAvailable(info)) => {
                for (idx, item) in info.current_and_previous.iter().enumerate() {
                    let item_text = with_relative_prefix(
                        item,
                        info.current_and_previous_times
                            .get(idx)
                            .and_then(|s| s.as_deref()),
                        '-',
                        &now,
                    );
                    text.push(Line::from(vec![
                        Span::styled("    • ", Style::default().fg(Color::DarkGray)),
                        Span::styled(item_text, Style::default().fg(Color::White)),
                    ]));
                }
                if info.current_and_previous.is_empty() {
                    text.push(Line::from(vec![Span::styled(
                        "    No version history available.",
                        Style::default().fg(Color::Gray),
                    )]));
                }
            }
            Some(UpdateInfoStatus::UpToDate) => {
                text.push(Line::from(vec![Span::styled(
                    "    Current build is already on the latest release.",
                    Style::default().fg(Color::Gray),
                )]));
            }
            Some(UpdateInfoStatus::CheckFailed(_)) => {
                text.push(Line::from(vec![Span::styled(
                    "    Current build loaded; upstream status unavailable.",
                    Style::default().fg(Color::Gray),
                )]));
            }
            None => {
                text.push(Line::from(vec![Span::styled(
                    "    Version history unavailable.",
                    Style::default().fg(Color::Gray),
                )]));
            }
        }
    }

    text.push(Line::from(""));
    text.push(Line::from(vec![
        Span::styled(
            "  [Enter]",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Continue    ", Style::default().fg(Color::Gray)),
        Span::styled(
            "[T]",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Time Vault    ", Style::default().fg(Color::Gray)),
        Span::styled(
            "[W]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Quest Wiki    ", Style::default().fg(Color::Gray)),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Quit", Style::default().fg(Color::Gray)),
    ]));

    text.push(Line::from(""));
    text.push(Line::from(vec![Span::styled(
        "  Contributors: @stphung \u{00b7} @dhsu",
        Style::default().fg(Color::DarkGray),
    )]));

    text
}

enum StartupKeyAction {
    Continue,
    Quit,
    OpenWiki,
    OpenTimeVault,
    Ignore,
}

fn startup_key_action(key_event: KeyEvent) -> StartupKeyAction {
    if key_event.kind != KeyEventKind::Press {
        return StartupKeyAction::Ignore;
    }

    match key_event.code {
        KeyCode::Enter => StartupKeyAction::Continue,
        KeyCode::Esc => StartupKeyAction::Quit,
        KeyCode::Char('w') | KeyCode::Char('W') => StartupKeyAction::OpenWiki,
        KeyCode::Char('t') | KeyCode::Char('T') => StartupKeyAction::OpenTimeVault,
        _ => StartupKeyAction::Ignore,
    }
}

fn wiki_url_for_browser() -> String {
    if WIKI_URL.starts_with("http://") || WIKI_URL.starts_with("https://") {
        WIKI_URL.to_string()
    } else {
        format!("https://{WIKI_URL}")
    }
}

pub enum StartupSplashResult {
    Continue,
    Quit,
}

/// Show the startup splash screen while update data loads in the background.
/// Pressing Enter continues immediately; Esc quits from startup.
#[allow(clippy::too_many_arguments)]
pub fn show_startup_splash_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    update_check_handle: &mut Option<JoinHandle<UpdateInfoStatus>>,
    history_repo: Option<&HistoryRepo>,
    haven: &mut haven::Haven,
    enhancement: &mut enhancement::EnhancementProgress,
    global_achievements: &mut achievements::Achievements,
    cloud_status: &crate::history::cloud::CloudStatus,
    cloud_username: &Option<String>,
    quest_dir: &std::path::Path,
) -> io::Result<StartupSplashResult> {
    let mut update_status: Option<UpdateInfoStatus> = None;
    let mut time_vault_browser: Option<TimeVaultState> = None;

    let action = loop {
        if update_status.is_none() {
            let finished = update_check_handle
                .as_ref()
                .is_some_and(std::thread::JoinHandle::is_finished);
            if finished {
                if let Some(handle) = update_check_handle.take() {
                    update_status = Some(handle.join().unwrap_or_else(|_| {
                        UpdateInfoStatus::CheckFailed("update check thread panicked".to_string())
                    }));
                }
            }
        }

        let update_loading = update_status.is_none() && update_check_handle.is_some();
        let text = build_startup_splash_text(update_status.as_ref(), update_loading);
        // Draw splash, then Time Vault overlay on top if open.
        terminal.draw(|f| {
            let area = f.area();
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Quest ");
            let inner = block.inner(area);
            f.render_widget(block, area);
            let paragraph = Paragraph::new(text).alignment(ratatui::layout::Alignment::Left);
            f.render_widget(paragraph, inner);

            if let Some(ref browser) = time_vault_browser {
                ui::time_vault_scene::draw_time_vault(f, area, browser);
            }
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let event::Event::Key(key_event) = event::read()? {
                // Handle Time Vault overlay when open.
                if let Some(ref mut browser) = time_vault_browser {
                    match handle_time_vault_input(key_event, browser) {
                        TimeVaultAction::Close => {
                            time_vault_browser = None;
                        }
                        TimeVaultAction::RefreshCommits { branch_name } => {
                            if let Some(repo) = history_repo {
                                if let Ok(commits) = repo.list_commits(&branch_name) {
                                    browser.commits = commits;
                                }
                            }
                        }
                        TimeVaultAction::Restore { commit_id } => {
                            if let Some(repo) = history_repo {
                                let _ = repo.commit_raw("Auto-save");
                                if repo.restore_to(&commit_id).is_ok() {
                                    *haven = haven::load_haven();
                                    *enhancement = enhancement::load_enhancement();
                                    *global_achievements = achievements::load_achievements();
                                    global_achievements.refresh_progress();
                                    refresh_vault_browser(repo, browser, None);
                                }
                            }
                        }
                        TimeVaultAction::Fork {
                            commit_id,
                            branch_name,
                        } => {
                            if let Some(repo) = history_repo {
                                let _ = repo.commit_raw("Auto-save");
                                if repo.fork_timeline(&branch_name, &commit_id).is_ok() {
                                    *haven = haven::load_haven();
                                    *enhancement = enhancement::load_enhancement();
                                    *global_achievements = achievements::load_achievements();
                                    global_achievements.refresh_progress();
                                    refresh_vault_browser(repo, browser, None);
                                }
                            }
                        }
                        TimeVaultAction::SwitchBranch { branch_name } => {
                            if let Some(repo) = history_repo {
                                let _ = repo.commit_raw("Auto-save");
                                if repo.switch_timeline(&branch_name).is_ok() {
                                    *haven = haven::load_haven();
                                    *enhancement = enhancement::load_enhancement();
                                    *global_achievements = achievements::load_achievements();
                                    global_achievements.refresh_progress();
                                    refresh_vault_browser(repo, browser, Some(&branch_name));
                                }
                            }
                        }
                        TimeVaultAction::DeleteBranch { branch_name } => {
                            if let Some(repo) = history_repo {
                                if repo.delete_timeline(&branch_name).is_ok() {
                                    refresh_vault_browser(repo, browser, None);
                                }
                            }
                        }
                        TimeVaultAction::Continue => {}
                        // Cloud operations are handled by the main loop (Task 8).
                        TimeVaultAction::ValidateToken { .. }
                        | TimeVaultAction::ChangeRepo
                        | TimeVaultAction::LinkCloud { .. }
                        | TimeVaultAction::PushCloud
                        | TimeVaultAction::PullCloud
                        | TimeVaultAction::UnlinkCloud
                        | TimeVaultAction::ResolveKeepLocal
                        | TimeVaultAction::ResolveUseCloud
                        | TimeVaultAction::ResolveKeepBoth => {}
                    }
                    continue;
                }

                match startup_key_action(key_event) {
                    StartupKeyAction::Continue => break StartupSplashResult::Continue,
                    StartupKeyAction::Quit => break StartupSplashResult::Quit,
                    StartupKeyAction::OpenWiki => {
                        let _ = crate::utils::bug_report::open_browser(&wiki_url_for_browser());
                    }
                    StartupKeyAction::OpenTimeVault => {
                        if let Some(repo) = history_repo {
                            if let Ok(branches) = repo.list_branches() {
                                let commits = branches
                                    .first()
                                    .and_then(|b| repo.list_commits(&b.name).ok())
                                    .unwrap_or_default();
                                let mut vault_state = TimeVaultState::new(branches, commits);
                                vault_state.cloud_status = cloud_status.clone();
                                vault_state.cloud_username = cloud_username.clone();
                                if matches!(cloud_status, crate::history::cloud::CloudStatus::OutOfSync) {
                                    if let Ok(Some(div)) = crate::history::cloud::check_divergence(quest_dir) {
                                        vault_state.cloud_divergence = Some(div);
                                        vault_state.mode = crate::ui::time_vault_scene::BrowserMode::DivergenceResolution;
                                    }
                                }
                                time_vault_browser = Some(vault_state);
                            }
                        }
                    }
                    StartupKeyAction::Ignore => {}
                }
            }
        }
    };

    Ok(action)
}

/// Refresh the Time Vault browser after a branch operation.
/// If `select_branch` is provided, finds it by name; otherwise clamps selection.
fn refresh_vault_browser(
    repo: &HistoryRepo,
    browser: &mut TimeVaultState,
    select_branch: Option<&str>,
) {
    if let Ok(branches) = repo.list_branches() {
        if let Some(name) = select_branch {
            browser.selected_branch = branches.iter().position(|b| b.name == name).unwrap_or(0);
        }
        browser.branches = branches;
        if browser.selected_branch >= browser.branches.len() {
            browser.selected_branch = browser.branches.len().saturating_sub(1);
        }
        if let Some(b) = browser.branches.get(browser.selected_branch) {
            browser.commits = repo.list_commits(&b.name).unwrap_or_default();
            browser.selected_commit = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyModifiers;

    #[test]
    fn test_startup_key_action_continue_on_enter() {
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(matches!(
            startup_key_action(key),
            StartupKeyAction::Continue
        ));
    }

    #[test]
    fn test_startup_key_action_open_wiki_on_w() {
        let key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(matches!(
            startup_key_action(key),
            StartupKeyAction::OpenWiki
        ));
    }

    #[test]
    fn test_startup_key_action_time_vault_on_t() {
        let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
        assert!(matches!(
            startup_key_action(key),
            StartupKeyAction::OpenTimeVault
        ));
    }

    #[test]
    fn test_startup_key_action_quit_on_escape() {
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(matches!(startup_key_action(key), StartupKeyAction::Quit));
    }

    #[test]
    fn test_startup_key_action_ignore_other_keys() {
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(matches!(startup_key_action(key), StartupKeyAction::Ignore));
    }

    #[test]
    fn test_wiki_url_for_browser_has_scheme() {
        let url = wiki_url_for_browser();
        assert!(url.starts_with("https://") || url.starts_with("http://"));
        assert!(url.ends_with(WIKI_URL));
    }
}
