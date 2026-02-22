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
mod input;
mod items;
mod main_helpers;
mod stormglass;
mod tick_events;
mod ui;
mod utils;
mod zones;

use character::manager::CharacterManager;
use character::prestige::can_prestige;
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
use utils::updater::UpdateInfo;

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

    // Load account-level Haven state
    let mut haven = haven::load_haven();

    // Load account-level Enhancement (soulforge) state
    let mut enhancement = enhancement::load_enhancement();

    // Load account-level God Item progress

    // Load global achievements (shared across all characters)
    let mut global_achievements = achievements::load_achievements();
    global_achievements.refresh_progress();

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
    let mut help_overlay_showing = false;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    if matches!(
        show_startup_splash_screen(&mut terminal, &mut update_available)?,
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
                    &mut help_overlay_showing,
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
                let mut was_prestige_ready = can_prestige(&state);
                let mut prestige_ready_sound_enabled = false;

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
                let mut update_expanded = false;
                let mut update_check_handle: Option<std::thread::JoinHandle<Option<UpdateInfo>>> =
                    Some(std::thread::spawn(utils::updater::check_update_info));

                'game_loop: loop {
                    // Check if background update check completed
                    if let Some(handle) = update_check_handle.take() {
                        if handle.is_finished() {
                            if let Ok(info) = handle.join() {
                                update_info = info;
                            }
                            update_check_completed = true;
                        } else {
                            // Not finished yet, put it back
                            update_check_handle = Some(handle);
                        }
                    }

                    // Force full terminal redraw when transitioning to/from a
                    // fullscreen overlay. The game UI uses emoji/wide characters
                    // that can desync ratatui's internal buffer from the actual
                    // terminal state; clearing resyncs them.
                    let overlay_is_fullscreen = matches!(overlay, GameOverlay::Achievements { .. });
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
                            haven.discovered,
                            enhancement.discovered,
                            state.stormglass_discovered,
                            &global_achievements,
                            &enhancement.levels,
                            prestige_ready_sound_enabled,
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
                                        for event in &tick_result.events {
                                            match event {
                                                core::tick::TickEvent::EnemyDefeated { .. }
                                                | core::tick::TickEvent::SubzoneBossDefeated {
                                                    ..
                                                }
                                                | core::tick::TickEvent::DungeonBossDefeated {
                                                    ..
                                                }
                                                | core::tick::TickEvent::DungeonEliteDefeated {
                                                    ..
                                                } => {
                                                    chrono_surge.as_mut().unwrap().kills += 1;
                                                }
                                                core::tick::TickEvent::ItemDropped {
                                                    equipped: true,
                                                    ..
                                                }
                                                | core::tick::TickEvent::DungeonTreasureFound {
                                                    equipped: true,
                                                    ..
                                                } => {
                                                    chrono_surge
                                                        .as_mut()
                                                        .unwrap()
                                                        .items_equipped += 1;
                                                }
                                                core::tick::TickEvent::LeveledUp { .. } => {
                                                    chrono_surge.as_mut().unwrap().levels_gained +=
                                                        1;
                                                }
                                                _ => {}
                                            }
                                        }
                                        chrono_surge.as_mut().unwrap().ticks_remaining -= 1;
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
                                    });
                                    if !debug_mode {
                                        save_all(
                                            &character_manager,
                                            &state,
                                            &global_achievements,
                                            &haven,
                                            &enhancement,
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
                                &mut prestige_ready_sound_enabled,
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
                                chrono_surge = Some(ChronoSurgeState::new(ticks));
                                state.chrono_surge_active = true;
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
                            && !matches!(overlay, GameOverlay::OfflineWelcome { .. })
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

                            // Count surge stats from events
                            for event in &tick_result.events {
                                match event {
                                    core::tick::TickEvent::EnemyDefeated { .. }
                                    | core::tick::TickEvent::SubzoneBossDefeated { .. }
                                    | core::tick::TickEvent::DungeonBossDefeated { .. }
                                    | core::tick::TickEvent::DungeonEliteDefeated { .. } => {
                                        chrono_surge.as_mut().unwrap().kills += 1;
                                    }
                                    core::tick::TickEvent::ItemDropped {
                                        equipped: true, ..
                                    }
                                    | core::tick::TickEvent::DungeonTreasureFound {
                                        equipped: true,
                                        ..
                                    } => {
                                        chrono_surge.as_mut().unwrap().items_equipped += 1;
                                    }
                                    core::tick::TickEvent::LeveledUp { .. } => {
                                        chrono_surge.as_mut().unwrap().levels_gained += 1;
                                    }
                                    _ => {}
                                }
                            }

                            // Apply events to UI state — ticker zooms at surge speed
                            apply_tick_events(&mut state, &tick_result.events);
                            state.ticker.tick();

                            if tick_result.achievements_changed
                                || tick_result.haven_changed
                                || tick_result.enhancement_changed
                                || tick_result.god_items_changed
                            {
                                needs_save = true;
                            }

                            chrono_surge.as_mut().unwrap().ticks_remaining -= 1;
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
                            });
                            if !debug_mode {
                                save_all(
                                    &character_manager,
                                    &state,
                                    &global_achievements,
                                    &haven,
                                    &enhancement,
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

                            // Persist all state if anything changed
                            if (tick_result.achievements_changed
                                || tick_result.haven_changed
                                || tick_result.enhancement_changed
                                || tick_result.god_items_changed)
                                && !debug_mode
                            {
                                save_all(
                                    &character_manager,
                                    &state,
                                    &global_achievements,
                                    &haven,
                                    &enhancement,
                                );
                            }

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

                                            // Persist changes
                                            if !debug_mode {
                                                save_all(
                                                    &character_manager,
                                                    &state,
                                                    &global_achievements,
                                                    &haven,
                                                    &enhancement,
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

                    let prestige_ready = can_prestige(&state);
                    if prestige_ready_sound_enabled && !was_prestige_ready && prestige_ready {
                        utils::audio::play_prestige_ready_sound();
                    }
                    was_prestige_ready = prestige_ready;

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
