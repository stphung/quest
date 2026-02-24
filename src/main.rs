mod achievements;
mod challenges;
mod character;
mod combat;
mod core;
mod dungeon;
mod enhancement;
mod fishing;
#[allow(dead_code)]
mod god_items;
mod haven;
mod history;
mod input;
mod items;
mod main_helpers;
mod stormglass;
mod tick_events;
mod ui;
mod utils;
mod zones;

use character::manager::CharacterManager;
use chrono::{Local, Utc};
use core::constants::*;
use core::game_state::*;
use input::{GameOverlay, HavenUiState, InputResult, SoulforgeUiState};
use main_helpers::achievements::track_input_achievements;
use main_helpers::character_screens::{
    handle_creation_frame, handle_delete_frame, handle_rename_frame, handle_select_frame,
    ScreenTransition,
};
use main_helpers::input_routing::{route_game_input, InputAction};
use main_helpers::offline::apply_offline_xp;
use main_helpers::overlay::draw_game_overlays;
use main_helpers::persistence::save_all;
use main_helpers::scene::{current_scene_kind, is_realtime_minigame, is_wide_scene};
use main_helpers::update::{
    jittered_update_interval, show_startup_splash_screen, StartupSplashResult,
};
use rand::RngExt;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::crossterm::ExecutableCommand;
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};
use stormglass::types::{ChronoSurgeState, ChronoSurgeSummary};
use tick_events::apply_tick_events;
use ui::achievement_browser_scene::AchievementBrowserState;
use ui::character_creation::CharacterCreationScreen;
use ui::character_delete::CharacterDeleteScreen;
use ui::character_rename::CharacterRenameScreen;
use ui::character_select::CharacterSelectScreen;
use ui::draw_ui_with_update;
use ui::title_browser_scene::TitleBrowserState;
use utils::updater::{UpdateInfo, UpdateInfoStatus};

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    CharacterSelect,
    CharacterCreation,
    CharacterDelete,
    CharacterRename,
    Game,
}

fn play_screen_transition(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    const STEPS: u16 = 6;
    for step in 0..=STEPS {
        let progress = step as f32 / STEPS as f32;
        terminal.draw(|frame| {
            let area = frame.area();
            let band = ((progress * area.height as f32) / 2.0).ceil() as u16;
            if band > 0 && area.width > 0 {
                let row = " ".repeat(area.width as usize);
                for y in 0..band {
                    frame.render_widget(
                        Paragraph::new(row.as_str()).style(Style::default().bg(Color::Black)),
                        Rect::new(area.x, area.y + y, area.width, 1),
                    );
                }
                let bottom_start = area.y + area.height.saturating_sub(band);
                for y in bottom_start..(area.y + area.height) {
                    frame.render_widget(
                        Paragraph::new(row.as_str()).style(Style::default().bg(Color::Black)),
                        Rect::new(area.x, y, area.width, 1),
                    );
                }
            }
            frame.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
                area,
            );
        })?;
        std::thread::sleep(Duration::from_millis(14));
    }
    Ok(())
}

fn tally_chrono_surge_events(surge: &mut ChronoSurgeState, events: &[core::tick::TickEvent]) {
    for event in events {
        match event {
            core::tick::TickEvent::EnemyDefeated { .. }
            | core::tick::TickEvent::SubzoneBossDefeated { .. }
            | core::tick::TickEvent::DungeonBossDefeated { .. }
            | core::tick::TickEvent::DungeonEliteDefeated { .. } => {
                surge.kills += 1;
            }
            core::tick::TickEvent::ItemDropped { equipped: true, .. }
            | core::tick::TickEvent::DungeonTreasureFound { equipped: true, .. } => {
                surge.items_equipped += 1;
            }
            core::tick::TickEvent::LeveledUp { .. } => {
                surge.levels_gained += 1;
            }
            _ => {}
        }
    }
}

/// Extract a meaningful save event from tick events for git history commits.
///
/// Only milestone events trigger git commits — routine combat/XP events do not.
/// Returns the first matching event found.
fn extract_save_event(
    events: &[core::tick::TickEvent],
    _state: &GameState,
) -> Option<history::SaveEvent> {
    use core::tick::TickEvent;
    use history::SaveEvent;
    use zones::BossDefeatResult;

    for event in events {
        match event {
            TickEvent::SubzoneBossDefeated { result, .. } => match result {
                BossDefeatResult::ZoneComplete { old_zone, .. } => {
                    return Some(SaveEvent::ZoneBossDefeated(old_zone.clone()));
                }
                BossDefeatResult::StormsEnd => {
                    return Some(SaveEvent::ZoneBossDefeated("Storm Citadel".to_string()));
                }
                _ => {}
            },
            TickEvent::DungeonCompleted { .. } => {
                return Some(SaveEvent::DungeonCompleted("dungeon".to_string()));
            }
            TickEvent::StormLeviathanCaught => {
                return Some(SaveEvent::StormLeviathanCaught);
            }
            TickEvent::AchievementUnlocked { ref name, .. } => {
                return Some(SaveEvent::AchievementUnlocked(name.clone()));
            }
            _ => {}
        }
    }
    None
}

fn main() -> io::Result<()> {
    // Handle CLI arguments
    let args: Vec<String> = std::env::args().collect();
    let mut debug_mode = false;

    if args.len() > 1 {
        match args[1].as_str() {
            "update" => match utils::updater::run_update_command() {
                Ok(_) => std::process::exit(0),
                Err(_) => std::process::exit(1),
            },
            "--version" | "-v" => {
                println!(
                    "quest {} ({})",
                    utils::build_info::BUILD_DATE,
                    utils::build_info::BUILD_COMMIT
                );
                std::process::exit(0);
            }
            "--help" | "-h" => {
                println!("Quest - Terminal-Based Idle RPG\n");
                println!("Usage: quest [command]\n");
                println!("Commands:");
                println!("  update     Check for and install updates");
                println!("  --debug    Enable debug menu (press ` to toggle)");
                println!("  --version  Show version information");
                println!("  --help     Show this help message");
                std::process::exit(0);
            }
            "--debug" => {
                debug_mode = true;
                eprintln!("=== DEBUG MODE ENABLED - SAVES DISABLED ===");
            }
            other => {
                eprintln!("Unknown command: {}", other);
                eprintln!("Run 'quest --help' for usage.");
                std::process::exit(1);
            }
        }
    }

    // Check for updates in background and feed startup splash when ready.
    let mut update_available = Some(std::thread::spawn(utils::updater::check_update_info));

    // Initialize CharacterManager
    let character_manager = CharacterManager::new()?;

    // Quest data directory
    let quest_dir = dirs::home_dir()
        .map(|d| d.join(".quest"))
        .unwrap_or_default();

    // Initialize Time Vault (non-fatal — game works without it)
    let history_repo = if !debug_mode {
        history::HistoryRepo::init(&quest_dir).ok()
    } else {
        None
    };

    // Cloud sync state
    let mut cloud_config = history::cloud::load_config(&quest_dir);
    let mut cloud_status = if cloud_config.is_some() {
        history::cloud::CloudStatus::Linked
    } else {
        history::cloud::CloudStatus::Offline
    };
    let mut cloud_username = cloud_config.as_ref().map(|c| c.username.clone());
    let (cloud_tx, cloud_rx) = std::sync::mpsc::channel::<history::cloud::CloudOpResult>();
    let mut cloud_op_in_flight = false;

    // Load account-level Haven state
    let mut haven = haven::load_haven();

    // Load account-level Enhancement (soulforge) state
    let mut enhancement = enhancement::load_enhancement();

    // Load account-level God Item progress

    // Load global achievements (shared across all characters)
    let mut global_achievements = achievements::load_achievements();
    crate::achievements::titles::validate_selected_title(&mut global_achievements);
    global_achievements.refresh_progress();

    // Auto-fetch from cloud on launch
    if let Some(ref config) = cloud_config {
        if history::cloud::fetch_all(&quest_dir, &config.token).is_ok() {
            match history::cloud::check_divergence(&quest_dir) {
                Ok(Some(_divergence)) => {
                    cloud_status = history::cloud::CloudStatus::OutOfSync;
                    // Divergence dialog will be shown when Time Vault opens
                }
                Ok(None) => {
                    if let Ok(updated) = history::cloud::fast_forward_all(&quest_dir) {
                        if updated {
                            // Reload state from disk since files changed
                            haven = haven::load_haven();
                            enhancement = enhancement::load_enhancement();
                            global_achievements = achievements::load_achievements();
                            crate::achievements::titles::validate_selected_title(
                                &mut global_achievements,
                            );
                            global_achievements.refresh_progress();
                        }
                    }
                }
                Err(_) => {} // Non-fatal
            }
        }
    }

    // List existing characters
    let characters = character_manager.list_characters()?;

    // Determine initial screen based on whether characters exist
    let mut current_screen = if characters.is_empty() {
        Screen::CharacterCreation
    } else {
        Screen::CharacterSelect
    };

    // Screen state variables
    let mut creation_screen = CharacterCreationScreen::new();
    let mut select_screen = CharacterSelectScreen::new();
    let mut delete_screen = CharacterDeleteScreen::new();
    let mut rename_screen = CharacterRenameScreen::new();
    let mut game_state: Option<GameState> = None;
    let mut pending_offline_report: Option<core::game_logic::OfflineReport> = None;

    let mut haven_ui = HavenUiState::new();
    let mut soulforge_ui = SoulforgeUiState::new();
    let mut exchange_ui = stormglass::types::ExchangeUiState::new();
    let mut achievement_browser = AchievementBrowserState::new();
    let mut title_browser = TitleBrowserState::new();
    let mut help_overlay_showing = false;
    let mut time_vault_browser: Option<ui::time_vault_scene::TimeVaultState> = None;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    if matches!(
        show_startup_splash_screen(
            &mut terminal,
            &mut update_available,
            history_repo.as_ref(),
            &mut haven,
            &mut enhancement,
            &mut global_achievements,
            &cloud_status,
            &cloud_username,
        )?,
        StartupSplashResult::Quit
    ) {
        disable_raw_mode()?;
        terminal.backend_mut().execute(LeaveAlternateScreen)?;
        println!("Goodbye!");
        return Ok(());
    }

    // If update check is still running after splash dismiss, detach it.
    if let Some(handle) = update_available.take() {
        if handle.is_finished() {
            let _ = handle.join();
        }
    }

    // Main loop — clear terminal on screen transitions to prevent
    // stale cells from wide characters (emoji) in the ratatui diff.
    let mut prev_screen = current_screen;
    loop {
        if current_screen != prev_screen {
            play_screen_transition(&mut terminal)?;
            terminal.clear()?;
            prev_screen = current_screen;
        }
        match current_screen {
            Screen::CharacterCreation => {
                let transition =
                    handle_creation_frame(&mut terminal, &mut creation_screen, &character_manager)?;
                if let ScreenTransition::GoToSelect = transition {
                    creation_screen = CharacterCreationScreen::new();
                    select_screen = CharacterSelectScreen::new();
                    current_screen = Screen::CharacterSelect;
                }
            }

            Screen::CharacterSelect => {
                let transition = handle_select_frame(
                    &mut terminal,
                    &mut select_screen,
                    &character_manager,
                    &mut haven,
                    &mut haven_ui,
                    &mut soulforge_ui,
                    &mut enhancement,
                    &mut global_achievements,
                    &mut achievement_browser,
                    &mut title_browser,
                    &mut help_overlay_showing,
                    history_repo.as_ref(),
                    &mut time_vault_browser,
                    &quest_dir,
                    &mut cloud_config,
                    &mut cloud_status,
                    &mut cloud_username,
                    &cloud_tx,
                    &cloud_rx,
                    &mut cloud_op_in_flight,
                )?;
                match transition {
                    ScreenTransition::GoToCreation => {
                        creation_screen = CharacterCreationScreen::new();
                        current_screen = Screen::CharacterCreation;
                    }
                    ScreenTransition::GoToDelete => {
                        delete_screen = CharacterDeleteScreen::new();
                        current_screen = Screen::CharacterDelete;
                    }
                    ScreenTransition::GoToRename => {
                        rename_screen = CharacterRenameScreen::new();
                        current_screen = Screen::CharacterRename;
                    }
                    ScreenTransition::LoadCharacter {
                        state,
                        offline_report,
                    } => {
                        pending_offline_report = offline_report;
                        game_state = Some(*state);
                        current_screen = Screen::Game;
                    }
                    ScreenTransition::Quit => {
                        break;
                    }
                    _ => {}
                }
            }

            Screen::CharacterDelete => {
                let transition = handle_delete_frame(
                    &mut terminal,
                    &mut delete_screen,
                    &select_screen,
                    &character_manager,
                )?;
                if let ScreenTransition::GoToSelect = transition {
                    delete_screen = CharacterDeleteScreen::new();
                    select_screen.selected_index = 0;
                    current_screen = Screen::CharacterSelect;
                }
            }

            Screen::CharacterRename => {
                let transition = handle_rename_frame(
                    &mut terminal,
                    &mut rename_screen,
                    &select_screen,
                    &character_manager,
                )?;
                if let ScreenTransition::GoToSelect = transition {
                    rename_screen = CharacterRenameScreen::new();
                    current_screen = Screen::CharacterSelect;
                }
            }

            Screen::Game => {
                // Take game state (it should always be Some when we're in Game screen)
                let mut state = game_state
                    .take()
                    .expect("Game state should be initialized when entering Game screen");

                // Run the game loop
                let mut last_tick = Instant::now();
                let mut last_autosave = Instant::now();
                let mut last_update_check = Instant::now();
                let mut next_update_check_interval = jittered_update_interval();
                let mut tick_counter: u32 = 0;
                let mut overlay = if let Some(report) = pending_offline_report.take() {
                    GameOverlay::OfflineWelcome { report }
                } else {
                    GameOverlay::None
                };
                let mut debug_menu = utils::debug_menu::DebugMenu::new();
                let mut chrono_surge: Option<ChronoSurgeState> = None;
                let mut chrono_summary: Option<ChronoSurgeSummary> = None;
                let mut last_flappy_frame = Instant::now();
                let mut prev_overlay_was_fullscreen =
                    matches!(overlay, GameOverlay::Achievements { .. });
                let mut prev_scene_kind = current_scene_kind(&state);

                // Save indicator state (for non-debug mode)
                let mut last_save_instant: Option<Instant> = None;
                let mut last_save_time: Option<chrono::DateTime<chrono::Local>> = None;

                // Update check state - start initial background check immediately
                let mut update_info: Option<UpdateInfo> = None;
                let mut update_check_completed = false;
                let mut update_check_failed = false;
                let mut update_expanded = false;
                let mut update_check_handle: Option<std::thread::JoinHandle<UpdateInfoStatus>> =
                    Some(std::thread::spawn(utils::updater::check_update_info));

                'game_loop: loop {
                    // Check if background update check completed
                    if let Some(handle) = update_check_handle.take() {
                        if handle.is_finished() {
                            match handle.join() {
                                Ok(UpdateInfoStatus::UpdateAvailable(info)) => {
                                    update_info = Some(info);
                                    update_check_failed = false;
                                }
                                Ok(UpdateInfoStatus::UpToDate) => {
                                    update_info = None;
                                    update_check_failed = false;
                                }
                                Ok(UpdateInfoStatus::CheckFailed(_)) | Err(_) => {
                                    update_info = None;
                                    update_check_failed = true;
                                }
                            }
                            update_check_completed = true;
                        } else {
                            // Not finished yet, put it back
                            update_check_handle = Some(handle);
                        }
                    }

                    // Poll cloud sync results
                    if cloud_op_in_flight {
                        if let Ok(result) = cloud_rx.try_recv() {
                            cloud_op_in_flight = false;
                            match result {
                                history::cloud::CloudOpResult::TokenValidated {
                                    username,
                                    token,
                                    repos,
                                } => {
                                    // Token is valid — show the repo picker.
                                    // Restore status based on whether we're already linked.
                                    cloud_status = if cloud_config.is_some() {
                                        history::cloud::CloudStatus::Linked
                                    } else {
                                        history::cloud::CloudStatus::Offline
                                    };
                                    if let GameOverlay::TimeVault { ref mut browser } = overlay {
                                        browser.cloud_validated_token = Some(token);
                                        browser.cloud_repos = repos;
                                        browser.cloud_repo_selected = 0;
                                        browser.cloud_repo_input.clear();
                                        browser.cloud_username = Some(username);
                                        browser.cloud_current_repo =
                                            cloud_config.as_ref().map(|c| {
                                                history::cloud::repo_name_from_url(&c.repo_url)
                                            });
                                        browser.mode =
                                            crate::ui::time_vault_scene::BrowserMode::SelectingRepo;
                                    }
                                }
                                history::cloud::CloudOpResult::Linked(config) => {
                                    cloud_username = Some(config.username.clone());
                                    cloud_config = Some(config);
                                    // Check if remote has different data
                                    match history::cloud::check_divergence(&quest_dir) {
                                        Ok(Some(_div)) => {
                                            cloud_status = history::cloud::CloudStatus::OutOfSync;
                                            if let GameOverlay::TimeVault { ref mut browser } =
                                                overlay
                                            {
                                                browser.cloud_divergence = Some(_div);
                                            }
                                        }
                                        _ => {
                                            cloud_status = history::cloud::CloudStatus::Linked;
                                        }
                                    }
                                }
                                history::cloud::CloudOpResult::Pushed => {
                                    cloud_status = history::cloud::CloudStatus::Linked;
                                }
                                history::cloud::CloudOpResult::Pulled => {
                                    cloud_status = history::cloud::CloudStatus::Linked;
                                    // Reload all state from disk
                                    haven = haven::load_haven();
                                    enhancement = enhancement::load_enhancement();
                                    global_achievements = achievements::load_achievements();
                                    crate::achievements::titles::validate_selected_title(
                                        &mut global_achievements,
                                    );
                                    global_achievements.refresh_progress();
                                    // Reload active character from pulled data
                                    let filename = format!("{}.json", state.character_name);
                                    if let Ok(mut reloaded) =
                                        character_manager.load_character(&filename)
                                    {
                                        reloaded.recalculate_derived_stats(&enhancement.levels);
                                        reloaded.recalculate_prestige_bonuses();
                                        state = reloaded;
                                    }
                                    state.last_save_time = Utc::now().timestamp();
                                }
                                history::cloud::CloudOpResult::Unlinked => {
                                    cloud_config = None;
                                    cloud_username = None;
                                    cloud_status = history::cloud::CloudStatus::Offline;
                                }
                                history::cloud::CloudOpResult::Diverged => {
                                    cloud_status = history::cloud::CloudStatus::OutOfSync;
                                }
                                history::cloud::CloudOpResult::Failed(msg) => {
                                    cloud_status = history::cloud::CloudStatus::Error(msg);
                                }
                            }
                            // Update Time Vault overlay if open
                            if let GameOverlay::TimeVault { ref mut browser } = overlay {
                                browser.cloud_status = cloud_status.clone();
                                browser.cloud_username = cloud_username.clone();
                                browser.cloud_current_repo = cloud_config
                                    .as_ref()
                                    .map(|c| history::cloud::repo_name_from_url(&c.repo_url));
                            }
                        }
                    }

                    // Force full terminal redraw when transitioning to/from a
                    // fullscreen overlay. The game UI uses emoji/wide characters
                    // that can desync ratatui's internal buffer from the actual
                    // terminal state; clearing resyncs them.
                    let overlay_is_fullscreen = matches!(
                        overlay,
                        GameOverlay::Achievements { .. } | GameOverlay::TimeVault { .. }
                    );
                    if overlay_is_fullscreen != prev_overlay_was_fullscreen {
                        terminal.clear()?;
                        prev_overlay_was_fullscreen = overlay_is_fullscreen;
                    }

                    // Force a redraw sync when switching between challenge menu
                    // and Sigil Surge scenes. Both use wide glyphs and can leave
                    // stale cells during scene transitions.
                    let scene_kind = current_scene_kind(&state);
                    if scene_kind != prev_scene_kind
                        && (is_wide_scene(scene_kind) || is_wide_scene(prev_scene_kind))
                    {
                        terminal.clear()?;
                    }
                    prev_scene_kind = scene_kind;

                    // Check if sigil rolling animation has timed out
                    crate::input::check_sigil_animation_timeout(&mut exchange_ui);

                    // Draw UI
                    terminal.draw(|frame| {
                        let ctx = ui::responsive::LayoutContext::from_frame(frame);
                        draw_ui_with_update(
                            frame,
                            &state,
                            update_info.as_ref(),
                            update_expanded,
                            update_check_completed,
                            update_check_failed,
                            haven.discovered,
                            enhancement.discovered,
                            state.stormglass_discovered,
                            &global_achievements,
                            &enhancement.levels,
                        );
                        draw_game_overlays(
                            frame,
                            &state,
                            &overlay,
                            &haven,
                            &haven_ui,
                            &soulforge_ui,
                            &exchange_ui,
                            &enhancement,
                            &global_achievements,
                            debug_mode,
                            &debug_menu,
                            last_save_instant,
                            last_save_time,
                            chrono_surge.as_ref(),
                            chrono_summary.as_ref(),
                            &ctx,
                        );
                    })?;

                    // Adaptive polling:
                    // - Realtime minigames: block only until the next frame boundary to avoid
                    //   busy-spinning and burning CPU between updates.
                    // - Normal mode: 50ms block to keep idle CPU low while responsive.
                    let realtime_mode = is_realtime_minigame(&state);
                    let surge_active = chrono_surge.is_some();
                    let mut poll_duration = if surge_active {
                        // During surge: minimal poll to keep rendering fast
                        Duration::ZERO
                    } else if realtime_mode {
                        Duration::from_millis(REALTIME_FRAME_MS)
                            .saturating_sub(last_flappy_frame.elapsed())
                    } else {
                        Duration::from_millis(50)
                    };

                    // Drain available events.
                    // In realtime mode, first poll may block until next frame; subsequent polls
                    // are non-blocking so we can flush queued input quickly.
                    while event::poll(poll_duration)? {
                        if let Event::Key(key_event) = event::read()? {
                            // Only handle key press events (ignore release/repeat)
                            if key_event.kind != KeyEventKind::Press {
                                if !realtime_mode {
                                    break;
                                }
                                continue;
                            }
                            // Chrono Surge summary: any key dismisses
                            if chrono_summary.is_some() {
                                chrono_summary = None;
                                continue;
                            }

                            // Active Chrono Surge: Esc skips animation (runs remaining
                            // ticks headlessly), all other keys ignored during surge.
                            if chrono_surge.is_some() {
                                if key_event.code == ratatui::crossterm::event::KeyCode::Esc {
                                    // Run all remaining ticks headlessly
                                    let mut rng = rand::rng();
                                    let sg_before_skip = state.stormglass;
                                    while chrono_surge.as_ref().unwrap().ticks_remaining > 0 {
                                        let tick_result = core::tick::game_tick(
                                            &mut state,
                                            &mut tick_counter,
                                            &mut haven,
                                            &mut enhancement,
                                            &mut global_achievements,
                                            debug_mode,
                                            &mut rng,
                                        );
                                        let surge = chrono_surge.as_mut().unwrap();
                                        tally_chrono_surge_events(surge, &tick_result.events);
                                        surge.ticks_remaining -= 1;
                                    }
                                    // No SG during surge — restore to pre-skip value
                                    state.stormglass = sg_before_skip;
                                    state.chrono_surge_active = false;
                                    let surge = chrono_surge.take().unwrap();
                                    chrono_summary = Some(ChronoSurgeSummary {
                                        kills: surge.kills,
                                        levels_gained: surge.levels_gained,
                                        items_equipped: surge.items_equipped,
                                        ticks_completed: surge.ticks_total,
                                        ticks_total: surge.ticks_total,
                                        overcharged: surge.overcharged,
                                    });
                                    if !debug_mode {
                                        let surge_event = history::SaveEvent::ChronoSurge {
                                            levels_gained: surge.levels_gained,
                                            kills: surge.kills,
                                            ticks: surge.ticks_total,
                                        };
                                        save_all(
                                            &character_manager,
                                            &state,
                                            &global_achievements,
                                            &haven,
                                            &enhancement,
                                            Some(&surge_event),
                                            history_repo.as_ref(),
                                        );
                                        last_save_instant = Some(Instant::now());
                                        last_save_time = Some(Local::now());
                                    }
                                }
                                continue;
                            }

                            // Track prestige rank before input to detect prestige
                            let prestige_before = state.prestige_rank;

                            let result = input::handle_game_input(
                                key_event,
                                &mut state,
                                &mut haven,
                                &mut haven_ui,
                                &mut soulforge_ui,
                                &mut exchange_ui,
                                &mut enhancement,
                                &mut overlay,
                                &mut debug_menu,
                                debug_mode,
                                &mut global_achievements,
                                update_info.is_some(),
                                update_expanded,
                            );

                            track_input_achievements(
                                &mut state,
                                &mut global_achievements,
                                prestige_before,
                            );

                            // Recalculate caches if prestige or enhancement changed
                            if state.prestige_rank != prestige_before {
                                state.recalculate_prestige_bonuses();
                                state.recalculate_derived_stats(&enhancement.levels);
                            }

                            // Handle StartChronoSurge before routing
                            if let InputResult::StartChronoSurge { ticks } = result {
                                // Roll for overcharge proc from sigil bonus
                                let overcharge_chance =
                                    stormglass::sigils::SigilBonuses::compute(&state.storm_sigils)
                                        .chrono_overcharge_percent;
                                let mut rng = rand::rng();
                                let overcharged = if state.debug_force_overcharge {
                                    state.debug_force_overcharge = false;
                                    true
                                } else {
                                    overcharge_chance > 0.0
                                        && rng.random::<f64>() * 100.0 < overcharge_chance
                                };
                                let actual_ticks = if overcharged {
                                    (ticks as f64 * 1.5) as u64
                                } else {
                                    ticks
                                };
                                chrono_surge =
                                    Some(ChronoSurgeState::new(actual_ticks, overcharged));
                                state.chrono_surge_active = true;
                                continue;
                            }

                            // Handle Time Vault actions before routing
                            if let InputResult::OpenTimeVault = result {
                                if let Some(ref repo) = history_repo {
                                    if let Ok(branches) = repo.list_branches() {
                                        let commits = branches
                                            .first()
                                            .and_then(|b| repo.list_commits(&b.name).ok())
                                            .unwrap_or_default();
                                        let mut vault_state =
                                            crate::ui::time_vault_scene::TimeVaultState::new(
                                                branches, commits,
                                            );
                                        vault_state.cloud_status = cloud_status.clone();
                                        vault_state.cloud_username = cloud_username.clone();
                                        vault_state.cloud_current_repo =
                                            cloud_config.as_ref().map(|c| {
                                                history::cloud::repo_name_from_url(&c.repo_url)
                                            });
                                        overlay = GameOverlay::TimeVault {
                                            browser: Box::new(vault_state),
                                        };
                                    }
                                }
                                continue;
                            }

                            if let InputResult::RefreshSaveHistoryCommits { ref branch_name } =
                                result
                            {
                                if let Some(ref repo) = history_repo {
                                    if let Ok(commits) = repo.list_commits(branch_name) {
                                        if let GameOverlay::TimeVault { ref mut browser } = overlay
                                        {
                                            browser.commits = commits;
                                        }
                                    }
                                }
                                continue;
                            }

                            if let InputResult::RestoreSave { ref commit_id } = result {
                                if let Some(ref repo) = history_repo {
                                    // Auto-save current state before restoring
                                    save_all(
                                        &character_manager,
                                        &state,
                                        &global_achievements,
                                        &haven,
                                        &enhancement,
                                        Some(&history::SaveEvent::AutoSave),
                                        Some(repo),
                                    );
                                    if repo.restore_to(commit_id).is_ok() {
                                        // Reload all state from disk (git reset replaced files)
                                        haven = haven::load_haven();
                                        enhancement = enhancement::load_enhancement();
                                        global_achievements = achievements::load_achievements();
                                        global_achievements.refresh_progress();

                                        // Reload character state
                                        let filename = format!("{}.json", state.character_name);
                                        if let Ok(mut reloaded) =
                                            character_manager.load_character(&filename)
                                        {
                                            reloaded.recalculate_derived_stats(&enhancement.levels);
                                            reloaded.recalculate_prestige_bonuses();
                                            reloaded.combat_state.add_log_entry(
                                                "\u{23F3} Save restored".to_string(),
                                                false,
                                                true,
                                            );
                                            state = reloaded;
                                        }

                                        // Suppress offline XP on the reloaded save
                                        state.last_save_time = Utc::now().timestamp();

                                        // Refresh vault browser in-place (overlay stays open)
                                        if let GameOverlay::TimeVault { ref mut browser } = overlay
                                        {
                                            if let Ok(branches) = repo.list_branches() {
                                                browser.branches = branches;
                                                if browser.selected_branch >= browser.branches.len()
                                                {
                                                    browser.selected_branch =
                                                        browser.branches.len().saturating_sub(1);
                                                }
                                                if let Some(b) =
                                                    browser.branches.get(browser.selected_branch)
                                                {
                                                    browser.commits = repo
                                                        .list_commits(&b.name)
                                                        .unwrap_or_default();
                                                    browser.selected_commit = 0;
                                                }
                                            }
                                        }
                                        if !debug_mode {
                                            last_save_instant = Some(Instant::now());
                                            last_save_time = Some(Local::now());
                                        }
                                    }
                                }
                                continue;
                            }

                            if let InputResult::ForkSave {
                                ref commit_id,
                                ref branch_name,
                            } = result
                            {
                                if let Some(ref repo) = history_repo {
                                    // Auto-save current state before forking
                                    save_all(
                                        &character_manager,
                                        &state,
                                        &global_achievements,
                                        &haven,
                                        &enhancement,
                                        Some(&history::SaveEvent::AutoSave),
                                        Some(repo),
                                    );
                                    if repo.fork_timeline(branch_name, commit_id).is_ok() {
                                        // Full state reload (fork checks out the new branch)
                                        haven = haven::load_haven();
                                        enhancement = enhancement::load_enhancement();
                                        global_achievements = achievements::load_achievements();
                                        global_achievements.refresh_progress();

                                        let filename = format!("{}.json", state.character_name);
                                        if let Ok(mut reloaded) =
                                            character_manager.load_character(&filename)
                                        {
                                            reloaded.recalculate_derived_stats(&enhancement.levels);
                                            reloaded.recalculate_prestige_bonuses();
                                            reloaded.combat_state.add_log_entry(
                                                format!(
                                                    "\u{1F500} Timeline branched: {branch_name}"
                                                ),
                                                false,
                                                true,
                                            );
                                            state = reloaded;
                                        }

                                        // Suppress offline XP on the reloaded save
                                        state.last_save_time = Utc::now().timestamp();

                                        // Refresh vault browser in-place (overlay stays open)
                                        if let GameOverlay::TimeVault { ref mut browser } = overlay
                                        {
                                            if let Ok(branches) = repo.list_branches() {
                                                browser.branches = branches;
                                                if browser.selected_branch >= browser.branches.len()
                                                {
                                                    browser.selected_branch =
                                                        browser.branches.len().saturating_sub(1);
                                                }
                                                if let Some(b) =
                                                    browser.branches.get(browser.selected_branch)
                                                {
                                                    browser.commits = repo
                                                        .list_commits(&b.name)
                                                        .unwrap_or_default();
                                                    browser.selected_commit = 0;
                                                }
                                            }
                                        }
                                        if !debug_mode {
                                            last_save_instant = Some(Instant::now());
                                            last_save_time = Some(Local::now());
                                        }
                                    }
                                }
                                continue;
                            }

                            if let InputResult::SwitchSaveBranch { ref branch_name } = result {
                                if let Some(ref repo) = history_repo {
                                    // Auto-save current state before switching
                                    save_all(
                                        &character_manager,
                                        &state,
                                        &global_achievements,
                                        &haven,
                                        &enhancement,
                                        Some(&history::SaveEvent::AutoSave),
                                        Some(repo),
                                    );
                                    if repo.switch_timeline(branch_name).is_ok() {
                                        // Full state reload (switch checks out the branch)
                                        haven = haven::load_haven();
                                        enhancement = enhancement::load_enhancement();
                                        global_achievements = achievements::load_achievements();
                                        global_achievements.refresh_progress();

                                        let filename = format!("{}.json", state.character_name);
                                        if let Ok(mut reloaded) =
                                            character_manager.load_character(&filename)
                                        {
                                            reloaded.recalculate_derived_stats(&enhancement.levels);
                                            reloaded.recalculate_prestige_bonuses();
                                            reloaded.combat_state.add_log_entry(
                                                format!(
                                                    "\u{1F500} Timeline switched: {branch_name}"
                                                ),
                                                false,
                                                true,
                                            );
                                            state = reloaded;
                                        }

                                        // Suppress offline XP on the reloaded save
                                        state.last_save_time = Utc::now().timestamp();

                                        // Refresh vault browser in-place (overlay stays open)
                                        if let GameOverlay::TimeVault { ref mut browser } = overlay
                                        {
                                            if let Ok(branches) = repo.list_branches() {
                                                // Select the switched-to branch (sort order changes)
                                                browser.selected_branch = branches
                                                    .iter()
                                                    .position(|b| b.name == *branch_name)
                                                    .unwrap_or(0);
                                                browser.branches = branches;
                                                if let Some(b) =
                                                    browser.branches.get(browser.selected_branch)
                                                {
                                                    browser.commits = repo
                                                        .list_commits(&b.name)
                                                        .unwrap_or_default();
                                                    browser.selected_commit = 0;
                                                }
                                            }
                                        }
                                        if !debug_mode {
                                            last_save_instant = Some(Instant::now());
                                            last_save_time = Some(Local::now());
                                        }
                                    }
                                }
                                continue;
                            }

                            if let InputResult::DeleteSaveBranch { ref branch_name } = result {
                                if let Some(ref repo) = history_repo {
                                    if repo.delete_timeline(branch_name).is_ok() {
                                        // Refresh browser in-place (overlay stays open)
                                        if let GameOverlay::TimeVault { ref mut browser } = overlay
                                        {
                                            if let Ok(branches) = repo.list_branches() {
                                                browser.branches = branches;
                                                // Clamp selection
                                                if browser.selected_branch >= browser.branches.len()
                                                {
                                                    browser.selected_branch =
                                                        browser.branches.len().saturating_sub(1);
                                                }
                                                // Refresh commits for new selection
                                                if let Some(b) =
                                                    browser.branches.get(browser.selected_branch)
                                                {
                                                    browser.commits = repo
                                                        .list_commits(&b.name)
                                                        .unwrap_or_default();
                                                    browser.selected_commit = 0;
                                                }
                                            }
                                        }
                                    }
                                }
                                continue;
                            }

                            // Handle cloud sync actions before routing
                            if let InputResult::ValidateToken { ref token } = result {
                                if !cloud_op_in_flight {
                                    cloud_op_in_flight = true;
                                    let tx = cloud_tx.clone();
                                    let tok = token.clone();
                                    std::thread::spawn(move || {
                                        match history::cloud::github_get_username(&tok) {
                                            Ok(username) => {
                                                let repos = history::cloud::github_list_repos(&tok)
                                                    .unwrap_or_default();
                                                let _ = tx.send(
                                                    history::cloud::CloudOpResult::TokenValidated {
                                                        username,
                                                        token: tok,
                                                        repos,
                                                    },
                                                );
                                            }
                                            Err(e) => {
                                                let _ = tx
                                                    .send(history::cloud::CloudOpResult::Failed(e));
                                            }
                                        }
                                    });
                                    if let GameOverlay::TimeVault { ref mut browser } = overlay {
                                        browser.cloud_status = history::cloud::CloudStatus::Syncing;
                                    }
                                }
                                continue;
                            }

                            // Change repo: re-use existing PAT from config
                            if matches!(result, InputResult::ChangeRepo) {
                                if !cloud_op_in_flight {
                                    if let Some(ref config) = cloud_config {
                                        cloud_op_in_flight = true;
                                        let tx = cloud_tx.clone();
                                        let tok = config.token.clone();
                                        std::thread::spawn(move || {
                                            match history::cloud::github_get_username(&tok) {
                                                Ok(username) => {
                                                    let repos =
                                                        history::cloud::github_list_repos(&tok)
                                                            .unwrap_or_default();
                                                    let _ = tx.send(
                                                        history::cloud::CloudOpResult::TokenValidated {
                                                            username,
                                                            token: tok,
                                                            repos,
                                                        },
                                                    );
                                                }
                                                Err(e) => {
                                                    let _ = tx.send(
                                                        history::cloud::CloudOpResult::Failed(e),
                                                    );
                                                }
                                            }
                                        });
                                        if let GameOverlay::TimeVault { ref mut browser } = overlay
                                        {
                                            browser.cloud_status =
                                                history::cloud::CloudStatus::Syncing;
                                        }
                                    }
                                }
                                continue;
                            }

                            if let InputResult::LinkCloud {
                                ref token,
                                ref repo_name,
                            } = result
                            {
                                if !cloud_op_in_flight {
                                    cloud_status = history::cloud::CloudStatus::Syncing;
                                    cloud_op_in_flight = true;
                                    let tx = cloud_tx.clone();
                                    let dir = quest_dir.clone();
                                    let tok = token.clone();
                                    let rname = repo_name.clone();
                                    std::thread::spawn(move || {
                                        let res =
                                            match history::cloud::link_github(&dir, &tok, &rname) {
                                                Ok(config) => {
                                                    history::cloud::CloudOpResult::Linked(config)
                                                }
                                                Err(e) => history::cloud::CloudOpResult::Failed(e),
                                            };
                                        let _ = tx.send(res);
                                    });
                                    if let GameOverlay::TimeVault { ref mut browser } = overlay {
                                        browser.cloud_status = cloud_status.clone();
                                    }
                                }
                                continue;
                            }

                            if matches!(result, InputResult::PushCloud) {
                                if !cloud_op_in_flight {
                                    if let Some(ref config) = cloud_config {
                                        cloud_status = history::cloud::CloudStatus::Syncing;
                                        cloud_op_in_flight = true;
                                        let tx = cloud_tx.clone();
                                        let dir = quest_dir.clone();
                                        let tok = config.token.clone();
                                        std::thread::spawn(move || {
                                            let res =
                                                match history::cloud::push_all_branches(&dir, &tok)
                                                {
                                                    Ok(()) => history::cloud::CloudOpResult::Pushed,
                                                    Err(e) => {
                                                        history::cloud::CloudOpResult::Failed(e)
                                                    }
                                                };
                                            let _ = tx.send(res);
                                        });
                                        if let GameOverlay::TimeVault { ref mut browser } = overlay
                                        {
                                            browser.cloud_status = cloud_status.clone();
                                        }
                                    }
                                }
                                continue;
                            }

                            if matches!(result, InputResult::PullCloud) {
                                if !cloud_op_in_flight {
                                    if let Some(ref config) = cloud_config {
                                        cloud_status = history::cloud::CloudStatus::Syncing;
                                        cloud_op_in_flight = true;
                                        let tx = cloud_tx.clone();
                                        let dir = quest_dir.clone();
                                        let tok = config.token.clone();
                                        std::thread::spawn(move || {
                                            let res = (|| -> history::cloud::CloudOpResult {
                                                if let Err(e) =
                                                    history::cloud::fetch_all(&dir, &tok)
                                                {
                                                    return history::cloud::CloudOpResult::Failed(
                                                        e,
                                                    );
                                                }
                                                match history::cloud::check_divergence(&dir) {
                                                    Ok(Some(_)) => {
                                                        history::cloud::CloudOpResult::Diverged
                                                    }
                                                    Ok(None) => {
                                                        match history::cloud::fast_forward_all(&dir)
                                                        {
                                                            Ok(_) => {
                                                                history::cloud::CloudOpResult::Pulled
                                                            }
                                                            Err(e) => {
                                                                history::cloud::CloudOpResult::Failed(e)
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        history::cloud::CloudOpResult::Failed(e)
                                                    }
                                                }
                                            })(
                                            );
                                            let _ = tx.send(res);
                                        });
                                        if let GameOverlay::TimeVault { ref mut browser } = overlay
                                        {
                                            browser.cloud_status = cloud_status.clone();
                                        }
                                    }
                                }
                                continue;
                            }

                            if matches!(result, InputResult::UnlinkCloud) {
                                let _ = history::cloud::unlink(&quest_dir);
                                cloud_config = None;
                                cloud_username = None;
                                cloud_status = history::cloud::CloudStatus::Offline;
                                if let GameOverlay::TimeVault { ref mut browser } = overlay {
                                    browser.cloud_status = cloud_status.clone();
                                    browser.cloud_username = None;
                                }
                                continue;
                            }

                            if matches!(result, InputResult::ResolveKeepLocal) {
                                if !cloud_op_in_flight {
                                    if let Some(ref config) = cloud_config {
                                        cloud_status = history::cloud::CloudStatus::Syncing;
                                        cloud_op_in_flight = true;
                                        let tx = cloud_tx.clone();
                                        let dir = quest_dir.clone();
                                        let tok = config.token.clone();
                                        std::thread::spawn(move || {
                                            let res = match history::cloud::force_push_branch(
                                                &dir, "main", &tok,
                                            ) {
                                                Ok(()) => history::cloud::CloudOpResult::Pushed,
                                                Err(e) => history::cloud::CloudOpResult::Failed(e),
                                            };
                                            let _ = tx.send(res);
                                        });
                                        if let GameOverlay::TimeVault { ref mut browser } = overlay
                                        {
                                            browser.cloud_status = cloud_status.clone();
                                        }
                                    }
                                }
                                continue;
                            }

                            if matches!(result, InputResult::ResolveUseCloud) {
                                // Blocking: fetch + fast-forward, then reload
                                if let Some(ref config) = cloud_config {
                                    let _ = history::cloud::fetch_all(&quest_dir, &config.token);
                                    let _ = history::cloud::fast_forward_all(&quest_dir);
                                    haven = haven::load_haven();
                                    enhancement = enhancement::load_enhancement();
                                    global_achievements = achievements::load_achievements();
                                    crate::achievements::titles::validate_selected_title(
                                        &mut global_achievements,
                                    );
                                    global_achievements.refresh_progress();

                                    // Reload character state
                                    let filename = format!("{}.json", state.character_name);
                                    if let Ok(mut reloaded) =
                                        character_manager.load_character(&filename)
                                    {
                                        reloaded.recalculate_derived_stats(&enhancement.levels);
                                        reloaded.recalculate_prestige_bonuses();
                                        state = reloaded;
                                    }
                                    state.last_save_time = Utc::now().timestamp();

                                    cloud_status = history::cloud::CloudStatus::Linked;
                                    if let GameOverlay::TimeVault { ref mut browser } = overlay {
                                        browser.cloud_status = cloud_status.clone();
                                        if let Some(ref repo) = history_repo {
                                            if let Ok(branches) = repo.list_branches() {
                                                browser.branches = branches;
                                                if browser.selected_branch >= browser.branches.len()
                                                {
                                                    browser.selected_branch =
                                                        browser.branches.len().saturating_sub(1);
                                                }
                                                if let Some(b) =
                                                    browser.branches.get(browser.selected_branch)
                                                {
                                                    browser.commits = repo
                                                        .list_commits(&b.name)
                                                        .unwrap_or_default();
                                                    browser.selected_commit = 0;
                                                }
                                            }
                                        }
                                    }
                                }
                                continue;
                            }

                            if matches!(result, InputResult::ResolveKeepBoth) {
                                let _ = history::cloud::backup_and_reset(&quest_dir, "main");
                                haven = haven::load_haven();
                                enhancement = enhancement::load_enhancement();
                                global_achievements = achievements::load_achievements();
                                crate::achievements::titles::validate_selected_title(
                                    &mut global_achievements,
                                );
                                global_achievements.refresh_progress();

                                // Reload character state
                                let filename = format!("{}.json", state.character_name);
                                if let Ok(mut reloaded) =
                                    character_manager.load_character(&filename)
                                {
                                    reloaded.recalculate_derived_stats(&enhancement.levels);
                                    reloaded.recalculate_prestige_bonuses();
                                    state = reloaded;
                                }
                                state.last_save_time = Utc::now().timestamp();

                                cloud_status = history::cloud::CloudStatus::Linked;
                                if let GameOverlay::TimeVault { ref mut browser } = overlay {
                                    browser.cloud_status = cloud_status.clone();
                                    browser.cloud_username = cloud_username.clone();
                                    if let Some(ref repo) = history_repo {
                                        if let Ok(branches) = repo.list_branches() {
                                            browser.branches = branches;
                                            if browser.selected_branch >= browser.branches.len() {
                                                browser.selected_branch =
                                                    browser.branches.len().saturating_sub(1);
                                            }
                                            if let Some(b) =
                                                browser.branches.get(browser.selected_branch)
                                            {
                                                browser.commits =
                                                    repo.list_commits(&b.name).unwrap_or_default();
                                                browser.selected_commit = 0;
                                            }
                                        }
                                    }
                                }
                                continue;
                            }

                            match route_game_input(
                                result,
                                &state,
                                &character_manager,
                                &global_achievements,
                                &haven,
                                &enhancement,
                                debug_mode,
                                &mut update_expanded,
                                &mut last_save_instant,
                                &mut last_save_time,
                                history_repo.as_ref(),
                            ) {
                                InputAction::QuitToSelect => {
                                    game_state = None;
                                    current_screen = Screen::CharacterSelect;
                                    break 'game_loop;
                                }
                                InputAction::Continue => {}
                            }
                        }
                        // Normal mode: process one event per frame. Realtime: drain all.
                        if !realtime_mode {
                            break;
                        }
                        poll_duration = Duration::ZERO;
                    }

                    // Flappy Bird real-time tick (~30 FPS)
                    if realtime_mode {
                        let dt = last_flappy_frame.elapsed();
                        if dt >= Duration::from_millis(REALTIME_FRAME_MS) {
                            if let Some(challenges::ActiveMinigame::FlappyBird(ref mut game)) =
                                state.active_minigame
                            {
                                challenges::flappy::logic::tick_flappy_bird(
                                    game,
                                    dt.as_millis() as u64,
                                );
                            }
                            if let Some(challenges::ActiveMinigame::Snake(ref mut game)) =
                                state.active_minigame
                            {
                                challenges::snake::logic::tick_snake(game, dt.as_millis() as u64);
                            }
                            if let Some(challenges::ActiveMinigame::RunicShift(ref mut game)) =
                                state.active_minigame
                            {
                                challenges::runic_shift::logic::tick_runic_shift(
                                    game,
                                    dt.as_millis() as u64,
                                );
                            }
                            if let Some(challenges::ActiveMinigame::Jezzball(ref mut game)) =
                                state.active_minigame
                            {
                                challenges::jezzball::logic::tick_jezzball(
                                    game,
                                    dt.as_millis() as u64,
                                );
                            }
                            last_flappy_frame = Instant::now();
                        }
                    }

                    // Detect process suspension (laptop lid close/open).
                    // Compare wall-clock time against last_save_time to detect
                    // time gaps from OS-level process suspension (SIGTSTP/SIGSTOP).
                    // Autosave runs every 30s and syncs last_save_time, so a gap
                    // > 60s means the process was suspended.
                    {
                        let elapsed_since_save = Utc::now().timestamp() - state.last_save_time;
                        if elapsed_since_save > 60
                            && !matches!(
                                overlay,
                                GameOverlay::OfflineWelcome { .. } | GameOverlay::TimeVault { .. }
                            )
                        {
                            if let Some(report) = apply_offline_xp(&mut state, &haven) {
                                overlay = GameOverlay::OfflineWelcome { report };
                            }
                            // Reset tick timers to prevent stale Instant from
                            // causing a burst of catch-up ticks or immediate autosave
                            last_tick = Instant::now();
                            last_autosave = Instant::now();
                            // Immediate save with updated last_save_time
                            if !debug_mode {
                                save_all(
                                    &character_manager,
                                    &state,
                                    &global_achievements,
                                    &haven,
                                    &enhancement,
                                    None,
                                    history_repo.as_ref(),
                                );
                                last_save_instant = Some(Instant::now());
                                last_save_time = Some(Local::now());
                            }
                        }
                    }

                    // ── Chrono Surge: batched tick execution ──────────────
                    if chrono_surge.is_some()
                        && last_tick.elapsed() >= Duration::from_millis(TICK_INTERVAL_MS)
                    {
                        let batch = {
                            let surge = chrono_surge.as_ref().unwrap();
                            surge.current_batch_size().min(surge.ticks_remaining)
                        };

                        let mut rng = rand::rng();
                        let mut needs_save = false;
                        let sg_before_batch = state.stormglass;

                        for _ in 0..batch {
                            let tick_result = core::tick::game_tick(
                                &mut state,
                                &mut tick_counter,
                                &mut haven,
                                &mut enhancement,
                                &mut global_achievements,
                                debug_mode,
                                &mut rng,
                            );

                            // Keep surge path headless (same semantics as [Esc] skip):
                            // collect summary counters only and avoid per-tick UI work.
                            let surge = chrono_surge.as_mut().unwrap();
                            tally_chrono_surge_events(surge, &tick_result.events);

                            if tick_result.achievements_changed
                                || tick_result.haven_changed
                                || tick_result.enhancement_changed
                                || tick_result.god_items_changed
                            {
                                needs_save = true;
                            }

                            surge.ticks_remaining -= 1;
                        }

                        // No SG earned during Chrono Surge — temporal displacement
                        // prevents salvage from materializing
                        state.stormglass = sg_before_batch;

                        if needs_save && !debug_mode {
                            save_all(
                                &character_manager,
                                &state,
                                &global_achievements,
                                &haven,
                                &enhancement,
                                None,
                                history_repo.as_ref(),
                            );
                        }

                        // Check if surge is complete
                        let done = chrono_surge.as_ref().unwrap().ticks_remaining == 0;
                        if done {
                            state.chrono_surge_active = false;
                            let surge = chrono_surge.take().unwrap();
                            chrono_summary = Some(ChronoSurgeSummary {
                                kills: surge.kills,
                                levels_gained: surge.levels_gained,
                                items_equipped: surge.items_equipped,
                                ticks_completed: surge.ticks_total,
                                ticks_total: surge.ticks_total,
                                overcharged: surge.overcharged,
                            });
                            if !debug_mode {
                                let surge_event = history::SaveEvent::ChronoSurge {
                                    levels_gained: surge.levels_gained,
                                    kills: surge.kills,
                                    ticks: surge.ticks_total,
                                };
                                save_all(
                                    &character_manager,
                                    &state,
                                    &global_achievements,
                                    &haven,
                                    &enhancement,
                                    Some(&surge_event),
                                    history_repo.as_ref(),
                                );
                                last_save_instant = Some(Instant::now());
                                last_save_time = Some(Local::now());
                            }
                        }

                        last_tick = Instant::now();
                    }

                    // Game tick every 100ms (normal — not during surge)
                    if chrono_surge.is_none()
                        && last_tick.elapsed() >= Duration::from_millis(TICK_INTERVAL_MS)
                    {
                        if !matches!(overlay, GameOverlay::LeviathanEncounter { .. }) {
                            let mut rng = rand::rng();
                            let tick_result = core::tick::game_tick(
                                &mut state,
                                &mut tick_counter,
                                &mut haven,
                                &mut enhancement,
                                &mut global_achievements,
                                debug_mode,
                                &mut rng,
                            );

                            let tick_flags = apply_tick_events(&mut state, &tick_result.events);

                            // Advance loot ticker scroll (update viewport width for cleanup)
                            if let Ok((cols, _)) = ratatui::crossterm::terminal::size() {
                                state.ticker.viewport_width = cols as usize;
                            }
                            state.ticker.tick();

                            // Update visual effect lifetimes
                            let delta_time = TICK_INTERVAL_MS as f64 / 1000.0;
                            state
                                .combat_state
                                .visual_effects
                                .retain_mut(|effect| effect.update(delta_time));

                            // Extract milestone save event for git history
                            let save_event = extract_save_event(&tick_result.events, &state);

                            // Persist all state if anything changed
                            if (tick_result.achievements_changed
                                || tick_result.haven_changed
                                || tick_result.enhancement_changed
                                || tick_result.god_items_changed
                                || save_event.is_some())
                                && !debug_mode
                            {
                                save_all(
                                    &character_manager,
                                    &state,
                                    &global_achievements,
                                    &haven,
                                    &enhancement,
                                    save_event.as_ref(),
                                    history_repo.as_ref(),
                                );

                                // Push to cloud after milestone commits
                                if save_event.is_some()
                                    && cloud_config.is_some()
                                    && !cloud_op_in_flight
                                {
                                    if let Some(ref config) = cloud_config {
                                        cloud_op_in_flight = true;
                                        let tx = cloud_tx.clone();
                                        let dir = quest_dir.clone();
                                        let tok = config.token.clone();
                                        std::thread::spawn(move || {
                                            let res =
                                                match history::cloud::push_all_branches(&dir, &tok)
                                                {
                                                    Ok(()) => history::cloud::CloudOpResult::Pushed,
                                                    Err(e) => {
                                                        history::cloud::CloudOpResult::Failed(e)
                                                    }
                                                };
                                            let _ = tx.send(res);
                                        });
                                    }
                                }
                            }

                            // Discovery/encounter overlays should not clobber
                            // an open Time Vault; they will fire on a later tick.
                            let vault_open = matches!(overlay, GameOverlay::TimeVault { .. });
                            if !vault_open {
                                if let Some(encounter_number) = tick_result.leviathan_encounter {
                                    overlay = GameOverlay::LeviathanEncounter { encounter_number };
                                }
                                if tick_flags.haven_discovered {
                                    overlay = GameOverlay::HavenDiscovery;
                                }
                                if tick_flags.soulforge_discovered {
                                    overlay = GameOverlay::SoulforgeDiscovery;
                                }
                                if tick_flags.stormglass_discovered {
                                    overlay = GameOverlay::StormglassDiscovery;
                                }
                            }

                            if matches!(overlay, GameOverlay::None)
                                && !tick_result.achievement_modal_ready.is_empty()
                            {
                                overlay = GameOverlay::AchievementUnlocked {
                                    achievements: tick_result.achievement_modal_ready,
                                };
                            }
                        }
                        last_tick = Instant::now();

                        // Advance soulforge animation
                        if soulforge_ui.open {
                            match soulforge_ui.phase {
                                input::SoulforgePhase::Hammering => {
                                    soulforge_ui.animation_tick =
                                        soulforge_ui.animation_tick.saturating_add(1);
                                    if soulforge_ui.animation_tick >= 50 {
                                        if let Some(ref result) = soulforge_ui.last_result {
                                            // Apply cost and level change after animation
                                            state.prestige_rank -= result.cost;
                                            enhancement::apply_enhancement_result(
                                                &mut enhancement,
                                                result.slot_index,
                                                result.new_level,
                                                result.success,
                                            );
                                            global_achievements.on_enhancement_upgraded(
                                                result.new_level,
                                                &enhancement.levels,
                                                enhancement.total_attempts,
                                                Some(&state.character_name),
                                            );

                                            // Recalculate cached derived stats after enhancement change
                                            state.recalculate_derived_stats(&enhancement.levels);
                                            state.recalculate_prestige_bonuses();

                                            const SLOTS: [crate::items::types::EquipmentSlot; 7] = [
                                                crate::items::types::EquipmentSlot::Weapon,
                                                crate::items::types::EquipmentSlot::Armor,
                                                crate::items::types::EquipmentSlot::Helmet,
                                                crate::items::types::EquipmentSlot::Gloves,
                                                crate::items::types::EquipmentSlot::Boots,
                                                crate::items::types::EquipmentSlot::Amulet,
                                                crate::items::types::EquipmentSlot::Ring,
                                            ];
                                            let slot_name = SLOTS
                                                .get(result.slot_index)
                                                .map(|s| s.name())
                                                .unwrap_or("Unknown");
                                            if result.success {
                                                state.combat_state.add_log_entry(
                                                    format!(
                                                        "\u{2692} {} enhanced to +{}!",
                                                        slot_name, result.new_level
                                                    ),
                                                    false,
                                                    true,
                                                );
                                            } else {
                                                state.combat_state.add_log_entry(
                                                    format!("\u{2692} Enhancement failed! {} dropped to +{}.", slot_name, result.new_level),
                                                    false,
                                                    true,
                                                );

                                                // Stormglass consolation for failed enhancements (target +5 and above)
                                                if state.stormglass_discovered {
                                                    let target_level = result.old_level + 1;
                                                    let consolation =
                                                        stormglass::earning::soulforge_consolation(
                                                            target_level,
                                                        );
                                                    if consolation > 0 {
                                                        state.stormglass += consolation;
                                                        state.combat_state.add_log_entry(
                                                            format!("\u{1F48E} +{} Stormglass recovered from failed enhancement", consolation),
                                                            false,
                                                            true,
                                                        );
                                                    }
                                                }
                                            }

                                            soulforge_ui.phase = if result.success {
                                                input::SoulforgePhase::ResultSuccess
                                            } else {
                                                input::SoulforgePhase::ResultFailure
                                            };
                                            soulforge_ui.animation_tick = 0;

                                            // Persist changes (only commit on success)
                                            let soulforge_event = if result.success {
                                                Some(history::SaveEvent::SoulforgeEnhanced(
                                                    slot_name.to_string(),
                                                    result.new_level,
                                                ))
                                            } else {
                                                None
                                            };
                                            if !debug_mode {
                                                save_all(
                                                    &character_manager,
                                                    &state,
                                                    &global_achievements,
                                                    &haven,
                                                    &enhancement,
                                                    soulforge_event.as_ref(),
                                                    history_repo.as_ref(),
                                                );
                                            }
                                        }
                                    }
                                }
                                input::SoulforgePhase::ResultSuccess => {
                                    if soulforge_ui.animation_tick < 20 {
                                        soulforge_ui.animation_tick += 1;
                                    }
                                }
                                input::SoulforgePhase::ResultFailure => {
                                    if soulforge_ui.animation_tick < 15 {
                                        soulforge_ui.animation_tick += 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    // Auto-save every 30 seconds
                    if last_autosave.elapsed() >= Duration::from_secs(AUTOSAVE_INTERVAL_SECONDS) {
                        // Sync in-memory last_save_time so suspension detection
                        // only counts actual suspension time, not active play time
                        state.last_save_time = Utc::now().timestamp();
                        last_autosave = Instant::now();
                        last_save_time = Some(Local::now());

                        // Skip file I/O in debug mode
                        if !debug_mode {
                            save_all(
                                &character_manager,
                                &state,
                                &global_achievements,
                                &haven,
                                &enhancement,
                                None,
                                history_repo.as_ref(),
                            );
                            last_save_instant = Some(Instant::now());
                        }
                    }

                    // Periodic update check (every ~30 minutes with jitter)
                    // Only start a new check if we don't have one running and haven't found an update
                    if update_info.is_none()
                        && update_check_handle.is_none()
                        && last_update_check.elapsed() >= next_update_check_interval
                    {
                        update_check_handle =
                            Some(std::thread::spawn(utils::updater::check_update_info));
                        update_check_completed = false; // Reset to show "Checking..." again
                        update_check_failed = false;
                        last_update_check = Instant::now();
                        next_update_check_interval = jittered_update_interval();
                    }
                }
            }
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;

    println!("Goodbye!");

    Ok(())
}
