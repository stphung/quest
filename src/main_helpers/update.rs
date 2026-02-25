//! Update check helpers.

use crate::achievements;
use crate::character::input::{process_select_input, SelectInput, SelectResult};
use crate::character::manager::{CharacterInfo, CharacterManager};
use crate::character::prestige::get_prestige_tier;
use crate::core::constants::{UPDATE_CHECK_INTERVAL_SECONDS, UPDATE_CHECK_JITTER_SECONDS};
use crate::core::game_state::GameState;
use crate::enhancement;
use crate::haven;
use crate::history::cloud::{CloudConfig, CloudOpResult, CloudStatus};
use crate::history::HistoryRepo;
use crate::input::time_vault_input::{handle_time_vault_input, TimeVaultAction};
use crate::ui;
use crate::ui::achievement_browser_scene::AchievementBrowserState;
use crate::ui::character_select::CharacterSelectScreen;
use crate::ui::throbber::block_spinner_char;
use crate::ui::time_vault_scene::TimeVaultState;
use crate::ui::title_browser_scene::TitleBrowserState;
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

use super::achievements::log_synced_achievements;
use super::offline::apply_offline_xp;

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
    characters: &[CharacterInfo],
    selected_index: usize,
    achievements: &achievements::Achievements,
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

    // Heroes box
    let hero_border = Style::default().fg(Color::DarkGray);
    text.push(Line::from(vec![Span::styled(
        "  \u{250c}\u{2500} Heroes \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}",
        hero_border,
    )]));

    if characters.is_empty() {
        text.push(Line::from(vec![
            Span::styled("  \u{2502} ", hero_border),
            Span::styled(
                "No heroes yet. Press [N] to create one.",
                Style::default().fg(Color::Gray),
            ),
        ]));
    } else {
        for (i, character) in characters.iter().enumerate() {
            let is_selected = i == selected_index;
            let marker = if is_selected { "> " } else { "  " };

            let display_name = {
                let title_suffix = achievements.selected_title.and_then(|id| {
                    if achievements.is_unlocked(id) {
                        crate::achievements::titles::get_title_text(id)
                    } else {
                        None
                    }
                });
                match title_suffix {
                    Some(title) => format!("{}, {}", character.character_name, title),
                    None => character.character_name.clone(),
                }
            };

            let prestige_name = get_prestige_tier(character.prestige_rank).name;
            let entry = if character.is_corrupted {
                format!("{}{} (CORRUPTED)", marker, character.filename)
            } else {
                format!(
                    "{}{} (Lv {} {})",
                    marker, display_name, character.character_level, prestige_name
                )
            };

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            text.push(Line::from(vec![
                Span::styled("  \u{2502} ", hero_border),
                Span::styled(entry, style),
            ]));
        }
    }

    text.push(Line::from(vec![Span::styled(
        "  \u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}",
        hero_border,
    )]));
    text.push(Line::from(""));

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

    let new_button = if characters.len() >= 3 {
        "[N] New (Max 3)"
    } else {
        "[N] New"
    };

    text.push(Line::from(""));
    text.push(Line::from(vec![
        Span::styled(
            "  [Enter]",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Play    ", Style::default().fg(Color::Gray)),
        Span::styled(
            "[R]",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Rename    ", Style::default().fg(Color::Gray)),
        Span::styled(
            "[D]",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Del    ", Style::default().fg(Color::Gray)),
        Span::styled(new_button, Style::default().fg(Color::Gray)),
    ]));
    text.push(Line::from(vec![
        Span::styled(
            "  [A]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Achv    ", Style::default().fg(Color::Gray)),
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
        Span::styled(" Wiki    ", Style::default().fg(Color::Gray)),
        Span::styled(
            "[!]",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Bug    ", Style::default().fg(Color::Gray)),
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
    Up,
    Down,
    Select,
    New,
    Delete,
    Rename,
    Achievements,
    OpenTimeVault,
    OpenWiki,
    Quit,
    Ignore,
}

fn startup_key_action(key_event: KeyEvent) -> StartupKeyAction {
    if key_event.kind != KeyEventKind::Press {
        return StartupKeyAction::Ignore;
    }

    match key_event.code {
        KeyCode::Up => StartupKeyAction::Up,
        KeyCode::Down => StartupKeyAction::Down,
        KeyCode::Enter => StartupKeyAction::Select,
        KeyCode::Char('n') | KeyCode::Char('N') => StartupKeyAction::New,
        KeyCode::Char('d') | KeyCode::Char('D') => StartupKeyAction::Delete,
        KeyCode::Char('r') | KeyCode::Char('R') => StartupKeyAction::Rename,
        KeyCode::Char('a') | KeyCode::Char('A') => StartupKeyAction::Achievements,
        KeyCode::Char('t') | KeyCode::Char('T') => StartupKeyAction::OpenTimeVault,
        KeyCode::Char('w') | KeyCode::Char('W') => StartupKeyAction::OpenWiki,
        KeyCode::Esc => StartupKeyAction::Quit,
        _ => StartupKeyAction::Ignore,
    }
}

pub enum StartupSplashResult {
    /// Load a character and enter the game.
    LoadCharacter {
        state: Box<GameState>,
        offline_report: Option<crate::core::game_logic::OfflineReport>,
    },
    /// Go to character creation screen.
    GoToCreation,
    /// Go to character delete screen.
    GoToDelete,
    /// Go to character rename screen.
    GoToRename,
    /// Quit the application.
    Quit,
}

/// Show the unified startup splash + character select screen.
/// Handles update checking, character selection, overlays, and cloud sync.
#[allow(clippy::too_many_arguments)]
pub fn show_startup_splash_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    update_check_handle: &mut Option<JoinHandle<UpdateInfoStatus>>,
    history_repo: Option<&HistoryRepo>,
    haven: &mut haven::Haven,
    enhancement: &mut enhancement::EnhancementProgress,
    global_achievements: &mut achievements::Achievements,
    cloud_status: &mut CloudStatus,
    cloud_username: &mut Option<String>,
    quest_dir: &std::path::Path,
    character_manager: &CharacterManager,
    select_screen: &mut CharacterSelectScreen,
    achievement_browser: &mut AchievementBrowserState,
    title_browser: &mut TitleBrowserState,
    cloud_config: &mut Option<CloudConfig>,
    cloud_tx: &std::sync::mpsc::Sender<CloudOpResult>,
    cloud_rx: &std::sync::mpsc::Receiver<CloudOpResult>,
    cloud_op_in_flight: &mut bool,
) -> io::Result<StartupSplashResult> {
    let mut update_status: Option<UpdateInfoStatus> = None;
    let mut time_vault_browser: Option<TimeVaultState> = None;

    let action = loop {
        // Poll update check handle
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

        // Poll cloud sync results
        if *cloud_op_in_flight {
            if let Ok(result) = cloud_rx.try_recv() {
                *cloud_op_in_flight = false;
                let was_cloud_restore = select_screen.cloud_restore_in_flight;
                select_screen.cloud_restore_in_flight = false;
                match result {
                    CloudOpResult::TokenValidated {
                        username,
                        token,
                        repos,
                    } => {
                        *cloud_status = if cloud_config.is_some() {
                            CloudStatus::Linked
                        } else {
                            CloudStatus::Offline
                        };
                        *cloud_username = Some(username);
                        if let Some(ref mut browser) = time_vault_browser {
                            browser.cloud_validated_token = Some(token);
                            browser.cloud_repos = repos;
                            browser.cloud_repo_selected = 0;
                            browser.cloud_repo_input.clear();
                            browser.cloud_username = cloud_username.clone();
                            browser.cloud_current_repo = cloud_config
                                .as_ref()
                                .map(|c| crate::history::cloud::repo_name_from_url(&c.repo_url));
                            browser.mode = crate::ui::time_vault_scene::BrowserMode::SelectingRepo;
                        }
                    }
                    CloudOpResult::Linked(config) => {
                        *cloud_username = Some(config.username.clone());
                        *cloud_config = Some(config);
                        match crate::history::cloud::check_divergence(quest_dir) {
                            Ok(Some(div)) => {
                                *cloud_status = CloudStatus::OutOfSync;
                                if let Some(ref mut browser) = time_vault_browser {
                                    browser.cloud_divergence = Some(div);
                                }
                            }
                            _ => {
                                *cloud_status = CloudStatus::Linked;
                            }
                        }
                        if was_cloud_restore {
                            select_screen.cloud_restore_showing = false;
                            select_screen.cloud_restore_dismissed = true;
                            crate::main_helpers::cloud_ops::reload_account_state(
                                haven,
                                enhancement,
                                global_achievements,
                            );
                        }
                    }
                    CloudOpResult::Pushed => {
                        *cloud_status = CloudStatus::Linked;
                    }
                    CloudOpResult::Pulled => {
                        *cloud_status = CloudStatus::Linked;
                        crate::main_helpers::cloud_ops::reload_account_state(
                            haven,
                            enhancement,
                            global_achievements,
                        );
                    }
                    CloudOpResult::Unlinked => {
                        *cloud_config = None;
                        *cloud_username = None;
                        *cloud_status = CloudStatus::Offline;
                    }
                    CloudOpResult::Diverged(div) => {
                        *cloud_status = CloudStatus::OutOfSync;
                        if let Some(ref mut browser) = time_vault_browser {
                            browser.cloud_divergence = Some(div);
                            browser.mode =
                                crate::ui::time_vault_scene::BrowserMode::DivergenceResolution;
                        }
                    }
                    CloudOpResult::TokenUpdated(new_config) => {
                        *cloud_username = Some(new_config.username.clone());
                        *cloud_status = CloudStatus::Linked;
                        *cloud_config = Some(new_config);
                    }
                    CloudOpResult::Failed(msg) => {
                        if was_cloud_restore {
                            select_screen.cloud_restore_error = Some(msg);
                            *cloud_status = CloudStatus::Offline;
                        } else if crate::history::cloud::is_auth_error(&msg) {
                            *cloud_status = CloudStatus::TokenExpired;
                        } else {
                            *cloud_status = CloudStatus::Error(msg);
                        }
                    }
                }
                if let Some(ref mut browser) = time_vault_browser {
                    browser.cloud_status = cloud_status.clone();
                    browser.cloud_username = cloud_username.clone();
                }
            }
        }

        // Refresh character list
        let characters = character_manager.list_characters()?;

        // Auto-show cloud restore prompt on first launch (no characters, cloud offline, not dismissed)
        if characters.is_empty()
            && matches!(cloud_status, CloudStatus::Offline)
            && !select_screen.cloud_restore_dismissed
            && !select_screen.cloud_restore_showing
            && time_vault_browser.is_none()
        {
            select_screen.cloud_restore_showing = true;
        }

        // Draw
        let update_loading = update_status.is_none() && update_check_handle.is_some();
        let text = build_startup_splash_text(
            update_status.as_ref(),
            update_loading,
            &characters,
            select_screen.selected_index,
            global_achievements,
        );
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

            // Draw overlays
            if achievement_browser.showing {
                let ctx = ui::responsive::LayoutContext::from_frame(f);
                if title_browser.showing {
                    ui::title_browser_scene::render_title_browser(
                        f,
                        area,
                        global_achievements,
                        title_browser,
                        "",
                    );
                } else {
                    ui::achievement_browser_scene::render_achievement_browser(
                        f,
                        area,
                        global_achievements,
                        achievement_browser,
                        enhancement,
                        &ctx,
                    );
                }
            }
            if let Some(ref browser) = time_vault_browser {
                ui::time_vault_scene::draw_time_vault(f, area, browser);
            }
            if select_screen.cloud_restore_showing {
                select_screen.draw_cloud_restore_prompt(f, area);
            }
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let event::Event::Key(key_event) = event::read()? {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                // Cloud restore prompt blocks all other input
                if select_screen.cloud_restore_showing {
                    if !select_screen.cloud_restore_in_flight {
                        match key_event.code {
                            KeyCode::Esc => {
                                select_screen.cloud_restore_showing = false;
                                select_screen.cloud_restore_dismissed = true;
                                select_screen.cloud_restore_input.clear();
                                select_screen.cloud_restore_repo =
                                    crate::history::cloud::DEFAULT_REPO_NAME.to_string();
                                select_screen.cloud_restore_field = 0;
                                select_screen.cloud_restore_error = None;
                            }
                            KeyCode::Tab | KeyCode::BackTab => {
                                select_screen.cloud_restore_field =
                                    1 - select_screen.cloud_restore_field;
                            }
                            KeyCode::Backspace => {
                                if select_screen.cloud_restore_field == 0 {
                                    select_screen.cloud_restore_input.pop();
                                } else {
                                    select_screen.cloud_restore_repo.pop();
                                }
                                select_screen.cloud_restore_error = None;
                            }
                            KeyCode::Enter => {
                                if select_screen.cloud_restore_input.is_empty() {
                                    select_screen.cloud_restore_error =
                                        Some("Token cannot be empty".to_string());
                                } else if select_screen.cloud_restore_repo.is_empty() {
                                    select_screen.cloud_restore_error =
                                        Some("Repo name cannot be empty".to_string());
                                } else if !*cloud_op_in_flight {
                                    let token = select_screen.cloud_restore_input.clone();
                                    let repo_name = select_screen.cloud_restore_repo.clone();
                                    select_screen.cloud_restore_input.clear();
                                    select_screen.cloud_restore_repo =
                                        crate::history::cloud::DEFAULT_REPO_NAME.to_string();
                                    select_screen.cloud_restore_field = 0;
                                    select_screen.cloud_restore_error = None;
                                    select_screen.cloud_restore_in_flight = true;
                                    *cloud_status = CloudStatus::Syncing;
                                    *cloud_op_in_flight = true;
                                    let tx = cloud_tx.clone();
                                    let dir = quest_dir.to_path_buf();
                                    std::thread::spawn(move || {
                                        let tx2 = tx.clone();
                                        let result = std::panic::catch_unwind(
                                            std::panic::AssertUnwindSafe(|| {
                                                let res = match crate::history::cloud::link_and_pull(
                                                    &dir, &token, &repo_name,
                                                ) {
                                                    Ok(config) => CloudOpResult::Linked(config),
                                                    Err(e) => CloudOpResult::Failed(e),
                                                };
                                                let _ = tx.send(res);
                                            }),
                                        );
                                        if result.is_err() {
                                            let _ = tx2.send(CloudOpResult::Failed(
                                                "Cloud operation failed unexpectedly".to_string(),
                                            ));
                                        }
                                    });
                                }
                            }
                            KeyCode::Char(c) => {
                                if select_screen.cloud_restore_field == 0 {
                                    if select_screen.cloud_restore_input.len() < 100 {
                                        select_screen.cloud_restore_input.push(c);
                                        select_screen.cloud_restore_error = None;
                                    }
                                } else {
                                    let c = c.to_ascii_lowercase();
                                    if (c.is_ascii_lowercase()
                                        || c.is_ascii_digit()
                                        || c == '-'
                                        || c == '_')
                                        && select_screen.cloud_restore_repo.len() < 30
                                    {
                                        select_screen.cloud_restore_repo.push(c);
                                        select_screen.cloud_restore_error = None;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    continue;
                }

                // Time Vault overlay blocks other input when open
                let mut close_vault = false;
                if let Some(ref mut browser) = time_vault_browser {
                    let vault_action = handle_time_vault_input(key_event, browser);
                    match vault_action {
                        TimeVaultAction::Close => {
                            close_vault = true;
                        }
                        other => {
                            handle_time_vault_action(
                                other,
                                browser,
                                history_repo,
                                haven,
                                enhancement,
                                global_achievements,
                                quest_dir,
                                cloud_config,
                                cloud_status,
                                cloud_username,
                                cloud_tx,
                                cloud_op_in_flight,
                            );
                        }
                    }
                }
                if close_vault {
                    time_vault_browser = None;
                    continue;
                }
                if time_vault_browser.is_some() {
                    continue;
                }

                // Achievement browser blocks other input
                if achievement_browser.showing {
                    if title_browser.showing {
                        let unlocked =
                            crate::achievements::titles::get_unlocked_titles(global_achievements);
                        match key_event.code {
                            KeyCode::Esc => title_browser.close(),
                            KeyCode::Up => title_browser.move_up(),
                            KeyCode::Down => title_browser.move_down(unlocked.len()),
                            KeyCode::Enter => {
                                if let Some(title_def) = unlocked.get(title_browser.selected_index)
                                {
                                    global_achievements.selected_title =
                                        Some(title_def.achievement_id);
                                    title_browser.close();
                                    let _ = achievements::save_achievements(global_achievements);
                                }
                            }
                            KeyCode::Backspace => {
                                global_achievements.selected_title = None;
                                title_browser.close();
                                let _ = achievements::save_achievements(global_achievements);
                            }
                            _ => {}
                        }
                    } else {
                        let category_achievements = achievements::get_achievements_by_category(
                            achievement_browser.selected_category,
                        );
                        match key_event.code {
                            KeyCode::Up => achievement_browser.move_up(),
                            KeyCode::Down => {
                                achievement_browser.move_down(category_achievements.len())
                            }
                            KeyCode::Left | KeyCode::Char(',') | KeyCode::Char('<') => {
                                achievement_browser.prev_category()
                            }
                            KeyCode::Right | KeyCode::Char('.') | KeyCode::Char('>') => {
                                achievement_browser.next_category()
                            }
                            KeyCode::Esc | KeyCode::Char('a') | KeyCode::Char('A') => {
                                global_achievements.clear_recently_unlocked();
                                achievement_browser.close();
                            }
                            KeyCode::Char('t') | KeyCode::Char('T') => {
                                title_browser.open();
                            }
                            _ => {}
                        }
                    }
                    continue;
                }

                // Normal key dispatch
                match startup_key_action(key_event) {
                    StartupKeyAction::Select => {
                        let input = SelectInput::Select;
                        let result = process_select_input(select_screen, input, &characters);
                        match result {
                            SelectResult::NoCharacters => {
                                break StartupSplashResult::GoToCreation;
                            }
                            SelectResult::LoadCharacter(filename) => {
                                if let Some((state, offline_report)) = load_character_for_game(
                                    character_manager,
                                    &filename,
                                    enhancement,
                                    global_achievements,
                                    haven,
                                ) {
                                    break StartupSplashResult::LoadCharacter {
                                        state,
                                        offline_report,
                                    };
                                }
                            }
                            _ => {}
                        }
                    }
                    StartupKeyAction::Up => {
                        select_screen.move_up(&characters);
                    }
                    StartupKeyAction::Down => {
                        select_screen.move_down(&characters);
                    }
                    StartupKeyAction::New => {
                        if characters.len() < 3 {
                            break StartupSplashResult::GoToCreation;
                        }
                    }
                    StartupKeyAction::Delete => {
                        if !characters.is_empty() {
                            break StartupSplashResult::GoToDelete;
                        }
                    }
                    StartupKeyAction::Rename => {
                        if !characters.is_empty() {
                            break StartupSplashResult::GoToRename;
                        }
                    }
                    StartupKeyAction::Achievements => {
                        global_achievements.clear_pending_notifications();
                        achievement_browser.open();
                    }
                    StartupKeyAction::Quit => break StartupSplashResult::Quit,
                    StartupKeyAction::OpenWiki => {
                        let _ = crate::utils::bug_report::open_browser(
                            &crate::core::constants::wiki_url_for_browser(),
                        );
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
                                if matches!(cloud_status, CloudStatus::OutOfSync) {
                                    if let Ok(Some(div)) =
                                        crate::history::cloud::check_divergence(quest_dir)
                                    {
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

/// Handle a Time Vault action, including all cloud operations.
/// The `Close` action is handled by the caller (not passed here).
#[allow(clippy::too_many_arguments)]
fn handle_time_vault_action(
    action: TimeVaultAction,
    browser: &mut TimeVaultState,
    history_repo: Option<&HistoryRepo>,
    haven: &mut haven::Haven,
    enhancement: &mut enhancement::EnhancementProgress,
    global_achievements: &mut achievements::Achievements,
    quest_dir: &std::path::Path,
    cloud_config: &mut Option<CloudConfig>,
    cloud_status: &mut CloudStatus,
    cloud_username: &mut Option<String>,
    cloud_tx: &std::sync::mpsc::Sender<CloudOpResult>,
    cloud_op_in_flight: &mut bool,
) {
    match action {
        TimeVaultAction::Close => {} // Handled by caller
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
                    crate::main_helpers::cloud_ops::reload_account_state(
                        haven,
                        enhancement,
                        global_achievements,
                    );
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
                    crate::main_helpers::cloud_ops::reload_account_state(
                        haven,
                        enhancement,
                        global_achievements,
                    );
                    refresh_vault_browser(repo, browser, None);
                }
            }
        }
        TimeVaultAction::SwitchBranch { branch_name } => {
            if let Some(repo) = history_repo {
                let _ = repo.commit_raw("Auto-save");
                if repo.switch_timeline(&branch_name).is_ok() {
                    crate::main_helpers::cloud_ops::reload_account_state(
                        haven,
                        enhancement,
                        global_achievements,
                    );
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
        TimeVaultAction::ValidateToken { token } => {
            if !*cloud_op_in_flight {
                *cloud_op_in_flight = true;
                let tx = cloud_tx.clone();
                let tok = token;
                std::thread::spawn(move || {
                    let tx2 = tx.clone();
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        match crate::history::cloud::github_get_username(&tok) {
                            Ok(username) => {
                                let repos = crate::history::cloud::github_list_repos(&tok)
                                    .unwrap_or_default();
                                let _ = tx.send(CloudOpResult::TokenValidated {
                                    username,
                                    token: tok,
                                    repos,
                                });
                            }
                            Err(e) => {
                                let _ = tx.send(CloudOpResult::Failed(e));
                            }
                        }
                    }));
                    if result.is_err() {
                        let _ = tx2.send(CloudOpResult::Failed(
                            "Cloud operation failed unexpectedly".to_string(),
                        ));
                    }
                });
                browser.cloud_status = CloudStatus::Syncing;
            }
        }
        TimeVaultAction::ChangeRepo => {
            if !*cloud_op_in_flight {
                if let Some(ref config) = cloud_config {
                    *cloud_op_in_flight = true;
                    let tx = cloud_tx.clone();
                    let tok = config.token.clone();
                    std::thread::spawn(move || {
                        let tx2 = tx.clone();
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            match crate::history::cloud::github_get_username(&tok) {
                                Ok(username) => {
                                    let repos = crate::history::cloud::github_list_repos(&tok)
                                        .unwrap_or_default();
                                    let _ = tx.send(CloudOpResult::TokenValidated {
                                        username,
                                        token: tok,
                                        repos,
                                    });
                                }
                                Err(e) => {
                                    let _ = tx.send(CloudOpResult::Failed(e));
                                }
                            }
                        }));
                        if result.is_err() {
                            let _ = tx2.send(CloudOpResult::Failed(
                                "Cloud operation failed unexpectedly".to_string(),
                            ));
                        }
                    });
                    browser.cloud_status = CloudStatus::Syncing;
                }
            }
        }
        TimeVaultAction::LinkCloud {
            token,
            repo_name,
            private,
        } => {
            if !*cloud_op_in_flight {
                *cloud_status = CloudStatus::Syncing;
                *cloud_op_in_flight = true;
                browser.cloud_status = cloud_status.clone();
                let tx = cloud_tx.clone();
                let dir = quest_dir.to_path_buf();
                let tok = token;
                let rname = repo_name;
                std::thread::spawn(move || {
                    let tx2 = tx.clone();
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let res =
                            match crate::history::cloud::link_github(&dir, &tok, &rname, private) {
                                Ok(config) => CloudOpResult::Linked(config),
                                Err(e) => CloudOpResult::Failed(e),
                            };
                        let _ = tx.send(res);
                    }));
                    if result.is_err() {
                        let _ = tx2.send(CloudOpResult::Failed(
                            "Cloud operation failed unexpectedly".to_string(),
                        ));
                    }
                });
            }
        }
        TimeVaultAction::PushCloud => {
            if !*cloud_op_in_flight {
                if let Some(ref config) = cloud_config {
                    *cloud_status = CloudStatus::Syncing;
                    *cloud_op_in_flight = true;
                    browser.cloud_status = cloud_status.clone();
                    let tx = cloud_tx.clone();
                    let dir = quest_dir.to_path_buf();
                    let tok = config.token.clone();
                    std::thread::spawn(move || {
                        let tx2 = tx.clone();
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let res = match crate::history::cloud::push_all_branches(&dir, &tok) {
                                Ok(()) => CloudOpResult::Pushed,
                                Err(e) => CloudOpResult::Failed(e),
                            };
                            let _ = tx.send(res);
                        }));
                        if result.is_err() {
                            let _ = tx2.send(CloudOpResult::Failed(
                                "Cloud operation failed unexpectedly".to_string(),
                            ));
                        }
                    });
                }
            }
        }
        TimeVaultAction::PullCloud => {
            if !*cloud_op_in_flight {
                if let Some(ref config) = cloud_config {
                    *cloud_status = CloudStatus::Syncing;
                    *cloud_op_in_flight = true;
                    browser.cloud_status = cloud_status.clone();
                    let tx = cloud_tx.clone();
                    let dir = quest_dir.to_path_buf();
                    let tok = config.token.clone();
                    std::thread::spawn(move || {
                        let tx2 = tx.clone();
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let res = (|| -> CloudOpResult {
                                if let Err(e) = crate::history::cloud::fetch_all(&dir, &tok) {
                                    return CloudOpResult::Failed(e);
                                }
                                match crate::history::cloud::check_divergence(&dir) {
                                    Ok(Some(div)) => CloudOpResult::Diverged(div),
                                    Ok(None) => {
                                        match crate::history::cloud::fast_forward_all(&dir) {
                                            Ok(_) => CloudOpResult::Pulled,
                                            Err(e) => CloudOpResult::Failed(e),
                                        }
                                    }
                                    Err(e) => CloudOpResult::Failed(e),
                                }
                            })();
                            let _ = tx.send(res);
                        }));
                        if result.is_err() {
                            let _ = tx2.send(CloudOpResult::Failed(
                                "Cloud operation failed unexpectedly".to_string(),
                            ));
                        }
                    });
                }
            }
        }
        TimeVaultAction::UnlinkCloud => {
            let _ = crate::history::cloud::unlink(quest_dir);
            *cloud_config = None;
            *cloud_username = None;
            *cloud_status = CloudStatus::Offline;
            browser.cloud_status = cloud_status.clone();
            browser.cloud_username = None;
        }
        TimeVaultAction::ResolveKeepLocal => {
            if !*cloud_op_in_flight {
                if let Some(ref config) = cloud_config {
                    *cloud_status = CloudStatus::Syncing;
                    *cloud_op_in_flight = true;
                    browser.cloud_status = cloud_status.clone();
                    let tx = cloud_tx.clone();
                    let dir = quest_dir.to_path_buf();
                    let tok = config.token.clone();
                    std::thread::spawn(move || {
                        let tx2 = tx.clone();
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let res = match crate::history::cloud::force_push_branch(
                                &dir, "main", &tok,
                            ) {
                                Ok(()) => CloudOpResult::Pushed,
                                Err(e) => CloudOpResult::Failed(e),
                            };
                            let _ = tx.send(res);
                        }));
                        if result.is_err() {
                            let _ = tx2.send(CloudOpResult::Failed(
                                "Cloud operation failed unexpectedly".to_string(),
                            ));
                        }
                    });
                }
            }
        }
        TimeVaultAction::ResolveUseCloud => {
            if let Some(ref config) = cloud_config {
                let _ = crate::history::cloud::fetch_all(quest_dir, &config.token);
                let _ = crate::history::cloud::reset_to_remote(quest_dir, "main");
                crate::main_helpers::cloud_ops::reload_account_state(
                    haven,
                    enhancement,
                    global_achievements,
                );
                *cloud_status = CloudStatus::Linked;
                browser.cloud_status = cloud_status.clone();
                if let Some(repo) = history_repo {
                    refresh_vault_browser(repo, browser, None);
                }
            }
        }
        TimeVaultAction::ResolveKeepBoth => {
            let _ = crate::history::cloud::backup_and_reset(quest_dir, "main");
            crate::main_helpers::cloud_ops::reload_account_state(
                haven,
                enhancement,
                global_achievements,
            );
            *cloud_status = CloudStatus::Linked;
            browser.cloud_status = cloud_status.clone();
            browser.cloud_username = cloud_username.clone();
            if let Some(repo) = history_repo {
                refresh_vault_browser(repo, browser, None);
            }
        }
        TimeVaultAction::UpdateToken { token } => {
            if !*cloud_op_in_flight {
                if let Some(ref config) = cloud_config {
                    *cloud_op_in_flight = true;
                    *cloud_status = CloudStatus::Syncing;
                    browser.cloud_status = cloud_status.clone();
                    let tx = cloud_tx.clone();
                    let dir = quest_dir.to_path_buf();
                    let cfg = config.clone();
                    std::thread::spawn(move || {
                        let tx2 = tx.clone();
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let res = match crate::history::cloud::update_token(&dir, &token, &cfg)
                            {
                                Ok(new_config) => CloudOpResult::TokenUpdated(new_config),
                                Err(e) => {
                                    if crate::history::cloud::is_auth_error(&e) {
                                        CloudOpResult::Failed("invalid token".to_string())
                                    } else {
                                        CloudOpResult::Failed(e)
                                    }
                                }
                            };
                            let _ = tx.send(res);
                        }));
                        if result.is_err() {
                            let _ = tx2.send(CloudOpResult::Failed(
                                "Cloud operation failed unexpectedly".to_string(),
                            ));
                        }
                    });
                }
            }
        }
    }
}

/// Load a character file and prepare it for entering the game.
fn load_character_for_game(
    character_manager: &CharacterManager,
    filename: &str,
    enhancement: &mut enhancement::EnhancementProgress,
    global_achievements: &mut achievements::Achievements,
    haven: &mut haven::Haven,
) -> Option<(
    Box<GameState>,
    Option<crate::core::game_logic::OfflineReport>,
)> {
    match character_manager.load_character(filename) {
        Ok(mut state) => {
            // Initialize cached derived stats and prestige bonuses
            state.recalculate_derived_stats(&enhancement.levels);
            state.recalculate_prestige_bonuses();

            // Sanity check: clear stale enemy if HP is impossibly high
            let derived = state.cached_derived_stats;
            if let Some(enemy) = &state.combat_state.current_enemy {
                if enemy.max_hp > (derived.max_hp as f64 * 2.5) as u32 {
                    state.combat_state.current_enemy = None;
                }
            }

            // Sync achievements from character state (retroactive unlocks)
            let defeated_bosses = state.zone_progression.defeated_bosses.to_vec();
            global_achievements.sync_from_game_state(
                state.character_level,
                state.prestige_rank,
                state.fishing.rank,
                state.fishing.total_fish_caught,
                &defeated_bosses,
                Some(&state.character_name),
            );
            global_achievements.sync_from_haven(
                haven.discovered,
                &haven.rooms,
                Some(&state.character_name),
            );

            // Retroactive enhancement/soulforge achievement sync
            if enhancement.discovered {
                global_achievements.on_soulforge_discovered(Some(&state.character_name));
            }
            global_achievements.on_enhancement_upgraded(
                enhancement.highest_level_reached,
                &enhancement.levels,
                enhancement.total_attempts,
                Some(&state.character_name),
            );

            log_synced_achievements(&mut state, global_achievements);

            // Process offline progression
            let current_time = chrono::Utc::now().timestamp();
            let elapsed_seconds = current_time - state.last_save_time;

            let offline_report = if elapsed_seconds > 60 {
                apply_offline_xp(&mut state, haven)
            } else {
                None
            };
            // Always sync last_save_time on load
            state.last_save_time = chrono::Utc::now().timestamp();

            Some((Box::new(state), offline_report))
        }
        Err(e) => {
            eprintln!("Failed to load character: {}", e);
            None
        }
    }
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
    fn test_startup_key_action_select_on_enter() {
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(matches!(startup_key_action(key), StartupKeyAction::Select));
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
        let url = crate::core::constants::wiki_url_for_browser();
        assert!(url.starts_with("https://") || url.starts_with("http://"));
        assert!(url.ends_with(crate::core::constants::WIKI_URL));
    }
}
