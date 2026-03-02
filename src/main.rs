mod achievements;
mod ascension;
mod challenges;
mod character;
mod combat;
mod core;
#[allow(unused_imports)]
mod deep;
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
mod power_cores;
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
    handle_creation_frame, handle_delete_frame, handle_rename_frame, ScreenTransition,
};
use main_helpers::chrono_surge::run_chrono_surge_batch;
use main_helpers::input_routing::{route_game_input, InputAction};
use main_helpers::offline::apply_offline_xp;
use main_helpers::overlay::draw_game_overlays;
use main_helpers::persistence::{commit_save, save_files};
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
                Err(e) => {
                    eprintln!("Update failed: {}", e);
                    std::process::exit(1);
                }
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
    let mut cloud = main_helpers::cloud_sync::CloudSyncState::new(&quest_dir);

    // Load account-level Haven state
    let mut haven = haven::load_haven();

    // Load account-level Enhancement (soulforge) state
    let mut enhancement = enhancement::load_enhancement();

    // Load account-level Deep state (mercenary expedition system)
    let mut deep_state = deep::load_deep();

    // Load account-level God Item progress

    // Load global achievements (shared across all characters)
    let mut global_achievements = achievements::load_achievements();
    crate::achievements::titles::validate_selected_title(&mut global_achievements);
    global_achievements.refresh_progress();

    // Auto-fetch from cloud on launch
    if let Some(ref config) = cloud.config {
        if history::cloud::fetch_all(&quest_dir, &config.token).is_ok() {
            match history::cloud::check_divergence(&quest_dir) {
                Ok(Some(_divergence)) => {
                    cloud.status = history::cloud::CloudStatus::OutOfSync;
                    // Divergence dialog will be shown when Time Vault opens
                }
                Ok(None) => {
                    if let Ok(updated) = history::cloud::fast_forward_all(&quest_dir) {
                        if updated {
                            // Reload state from disk since files changed
                            main_helpers::cloud_ops::reload_account_state(
                                &mut haven,
                                &mut enhancement,
                                &mut global_achievements,
                            );
                        }
                    }
                }
                Err(_) => {} // Non-fatal
            }
        }
    }

    // Screen state variables
    let mut select_screen = CharacterSelectScreen::new();
    let mut haven_ui = HavenUiState::new();
    let mut soulforge_ui = SoulforgeUiState::new();
    let mut exchange_ui = stormglass::types::ExchangeUiState::new();
    let mut deep_ui = deep::DeepUiState::new();
    let mut achievement_browser = AchievementBrowserState::new();
    let mut title_browser = TitleBrowserState::new();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // If no characters exist, go straight to creation first
    let initial_characters = character_manager.list_characters()?;
    let mut quit_early = false;
    if initial_characters.is_empty() {
        play_screen_transition(&mut terminal)?;
        terminal.clear()?;
        let mut creation_screen = CharacterCreationScreen::new();
        loop {
            let t = handle_creation_frame(&mut terminal, &mut creation_screen, &character_manager)?;
            match t {
                ScreenTransition::GoToSelect => {
                    select_screen = CharacterSelectScreen::new();
                    break;
                }
                ScreenTransition::Quit => {
                    quit_early = true;
                    break;
                }
                ScreenTransition::Stay => {}
            }
        }
        play_screen_transition(&mut terminal)?;
        terminal.clear()?;
    }

    if quit_early {
        disable_raw_mode()?;
        terminal.backend_mut().execute(LeaveAlternateScreen)?;
        println!("Goodbye!");
        return Ok(());
    }

    // Main loop: splash screen → sub-screens → game → back to splash
    'main_loop: loop {
        let splash_result = show_startup_splash_screen(
            &mut terminal,
            &mut update_available,
            history_repo.as_ref(),
            &mut haven,
            &mut enhancement,
            &mut global_achievements,
            &mut cloud.status,
            &mut cloud.username,
            &quest_dir,
            &character_manager,
            &mut select_screen,
            &mut achievement_browser,
            &mut title_browser,
            &mut cloud.config,
            &cloud.tx,
            &cloud.rx,
            &mut cloud.op_in_flight,
            &mut deep_state,
        )?;

        match splash_result {
            StartupSplashResult::Quit => {
                break 'main_loop;
            }
            StartupSplashResult::GoToCreation => {
                play_screen_transition(&mut terminal)?;
                terminal.clear()?;
                let mut creation_screen = CharacterCreationScreen::new();
                loop {
                    let t = handle_creation_frame(
                        &mut terminal,
                        &mut creation_screen,
                        &character_manager,
                    )?;
                    match t {
                        ScreenTransition::GoToSelect => {
                            select_screen = CharacterSelectScreen::new();
                            break;
                        }
                        ScreenTransition::Quit => {
                            break;
                        }
                        ScreenTransition::Stay => {}
                    }
                }
                play_screen_transition(&mut terminal)?;
                terminal.clear()?;
                continue 'main_loop;
            }
            StartupSplashResult::GoToDelete => {
                play_screen_transition(&mut terminal)?;
                terminal.clear()?;
                let mut delete_screen = CharacterDeleteScreen::new();
                loop {
                    let t = handle_delete_frame(
                        &mut terminal,
                        &mut delete_screen,
                        &select_screen,
                        &character_manager,
                    )?;
                    if let ScreenTransition::GoToSelect = t {
                        select_screen.selected_index = 0;
                        break;
                    }
                }
                play_screen_transition(&mut terminal)?;
                terminal.clear()?;
                continue 'main_loop;
            }
            StartupSplashResult::GoToRename => {
                play_screen_transition(&mut terminal)?;
                terminal.clear()?;
                let mut rename_screen = CharacterRenameScreen::new();
                loop {
                    let t = handle_rename_frame(
                        &mut terminal,
                        &mut rename_screen,
                        &select_screen,
                        &character_manager,
                    )?;
                    if let ScreenTransition::GoToSelect = t {
                        break;
                    }
                }
                play_screen_transition(&mut terminal)?;
                terminal.clear()?;
                continue 'main_loop;
            }
            StartupSplashResult::LoadCharacter {
                state,
                offline_report,
            } => {
                let mut state = *state;

                // Resolve Deep missions that completed while offline
                main_helpers::offline::resolve_deep_offline(
                    &mut deep_state,
                    &mut global_achievements,
                    &state.character_name,
                );
                // Sync Deep achievements from persistent state
                global_achievements.sync_from_deep(
                    deep_state.persistent.discovered,
                    deep_state.persistent.guild_rank.0 as u32,
                    deep_state.persistent.deepest_layer_reached,
                    Some(&state.character_name),
                );

                play_screen_transition(&mut terminal)?;
                terminal.clear()?;

                // Game loop
                {
                    let mut last_tick = Instant::now();
                    let mut last_autosave = Instant::now();
                    let mut last_update_check = Instant::now();
                    let mut next_update_check_interval = jittered_update_interval();
                    let mut tick_counter: u32 = 0;
                    let mut overlay = if let Some(report) = offline_report {
                        GameOverlay::OfflineWelcome { report }
                    } else {
                        GameOverlay::None
                    };
                    let mut debug_menu = utils::debug_menu::DebugMenu::new();
                    let mut pending_overlays: std::collections::VecDeque<GameOverlay> =
                        std::collections::VecDeque::new();
                    let mut chrono_surge: Option<ChronoSurgeState> = None;
                    let mut chrono_summary: Option<ChronoSurgeSummary> = None;
                    let mut last_flappy_frame = Instant::now();
                    let mut prev_overlay_was_fullscreen =
                        matches!(overlay, GameOverlay::Achievements { .. });
                    let mut prev_scene_kind = current_scene_kind(&state);

                    // Save indicator state (for non-debug mode)
                    let mut last_save_instant: Option<Instant> = None;
                    let mut last_save_time: Option<chrono::DateTime<chrono::Local>> = None;
                    let mut last_commit_instant: Option<Instant> = None;
                    let mut last_commit_time: Option<chrono::DateTime<chrono::Local>> = None;
                    let mut last_push_instant: Option<Instant> = None;
                    let mut last_push_time: Option<chrono::DateTime<chrono::Local>> = None;

                    // Update check state - start initial background check immediately
                    let mut update_info: Option<UpdateInfo> = None;
                    let mut update_check_completed = false;
                    let mut update_check_failed = false;
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
                        if let Some(cloud_result) = main_helpers::cloud_sync::poll_cloud_result(
                            &mut cloud,
                            &mut overlay,
                            &quest_dir,
                            &character_manager,
                            &state.character_name,
                            &enhancement,
                        ) {
                            if cloud_result.account_state_reloaded {
                                main_helpers::cloud_ops::reload_account_state(
                                    &mut haven,
                                    &mut enhancement,
                                    &mut global_achievements,
                                );
                            }
                            if let Some(reloaded) = cloud_result.reloaded_state {
                                state = reloaded;
                            }
                            if cloud_result.needs_save_timestamp {
                                state.last_save_time = Utc::now().timestamp();
                            }
                            if cloud_result.pushed {
                                last_push_instant = Some(Instant::now());
                                last_push_time = Some(Local::now());
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
                                update_check_completed,
                                update_check_failed,
                                haven.discovered,
                                enhancement.discovered,
                                state.stormglass_discovered,
                                &deep_state,
                                &global_achievements,
                                &enhancement.levels,
                                &deep_state,
                            );
                            {
                                let game_ctx = main_helpers::game_context::GameContext {
                                    state: &mut state,
                                    haven: &mut haven,
                                    haven_ui: &mut haven_ui,
                                    soulforge_ui: &mut soulforge_ui,
                                    exchange_ui: &mut exchange_ui,
                                    deep_state: &mut deep_state,
                                    deep_ui: &mut deep_ui,
                                    enhancement: &mut enhancement,
                                    overlay: &mut overlay,
                                    debug_menu: &mut debug_menu,
                                    debug_mode,
                                    achievements: &mut global_achievements,
                                };
                                let extras = main_helpers::game_context::OverlayExtras {
                                    last_save_instant,
                                    last_save_time,
                                    last_commit_instant,
                                    last_commit_time,
                                    last_push_instant,
                                    last_push_time,
                                    has_history_repo: history_repo.is_some(),
                                    has_cloud_config: cloud.config.is_some(),
                                    chrono_surge: chrono_surge.as_ref(),
                                    chrono_summary: chrono_summary.as_ref(),
                                    layout_ctx: &ctx,
                                };
                                draw_game_overlays(frame, &game_ctx, &extras);
                            }
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
                                            let mut ctx = core::tick_context::TickContext {
                                                state: &mut state,
                                                tick_counter: &mut tick_counter,
                                                haven: &mut haven,
                                                enhancement: &mut enhancement,
                                                deep: &mut deep_state,
                                                achievements: &mut global_achievements,
                                                debug_mode,
                                            };
                                            let tick_result = core::tick::game_tick_with_context(
                                                &mut ctx, &mut rng,
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
                                            save_files(
                                                &character_manager,
                                                &state,
                                                &global_achievements,
                                                &haven,
                                                &enhancement,
                                                &deep_state,
                                            );
                                            last_save_instant = Some(Instant::now());
                                            last_save_time = Some(Local::now());
                                            if let Some(ref repo) = history_repo {
                                                if commit_save(&state, &surge_event, repo) {
                                                    last_commit_instant = Some(Instant::now());
                                                    last_commit_time = Some(Local::now());
                                                }
                                            }
                                        }
                                    }
                                    continue;
                                }

                                // Track prestige/fishing rank before input to detect changes
                                let prestige_before = state.prestige_rank;
                                let fishing_rank_before = state.fishing.rank;

                                let result = {
                                    let mut ctx = main_helpers::game_context::GameContext {
                                        state: &mut state,
                                        haven: &mut haven,
                                        haven_ui: &mut haven_ui,
                                        soulforge_ui: &mut soulforge_ui,
                                        exchange_ui: &mut exchange_ui,
                                        deep_state: &mut deep_state,
                                        deep_ui: &mut deep_ui,
                                        enhancement: &mut enhancement,
                                        overlay: &mut overlay,
                                        debug_menu: &mut debug_menu,
                                        debug_mode,
                                        achievements: &mut global_achievements,
                                    };
                                    input::handle_game_input(key_event, &mut ctx)
                                };

                                track_input_achievements(
                                    &mut state,
                                    &mut global_achievements,
                                    prestige_before,
                                    fishing_rank_before,
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
                                        stormglass::sigils::SigilBonuses::compute(
                                            &state.storm_sigils,
                                        )
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
                                            vault_state.cloud_status = cloud.status.clone();
                                            vault_state.cloud_username = cloud.username.clone();
                                            vault_state.cloud_current_repo =
                                                cloud.config.as_ref().map(|c| {
                                                    history::cloud::repo_name_from_url(&c.repo_url)
                                                });
                                            // If already out-of-sync, re-check divergence and show resolution dialog
                                            if matches!(
                                                cloud.status,
                                                history::cloud::CloudStatus::OutOfSync
                                            ) {
                                                if let Ok(Some(div)) =
                                                    history::cloud::check_divergence(&quest_dir)
                                                {
                                                    vault_state.cloud_divergence = Some(div);
                                                    vault_state.mode = crate::ui::time_vault_scene::BrowserMode::DivergenceResolution;
                                                }
                                            }
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
                                            if let GameOverlay::TimeVault { ref mut browser } =
                                                overlay
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
                                        save_files(
                                            &character_manager,
                                            &state,
                                            &global_achievements,
                                            &haven,
                                            &enhancement,
                                            &deep_state,
                                        );
                                        last_save_instant = Some(Instant::now());
                                        last_save_time = Some(Local::now());
                                        if commit_save(&state, &history::SaveEvent::AutoSave, repo)
                                        {
                                            last_commit_instant = Some(Instant::now());
                                            last_commit_time = Some(Local::now());
                                        }
                                        if repo.restore_to(commit_id).is_ok() {
                                            // Reload all state from disk (git reset replaced files)
                                            main_helpers::cloud_ops::reload_account_state(
                                                &mut haven,
                                                &mut enhancement,
                                                &mut global_achievements,
                                            );

                                            // Reload character state
                                            let filename = format!("{}.json", state.character_name);
                                            if let Ok(mut reloaded) =
                                                character_manager.load_character(&filename)
                                            {
                                                reloaded
                                                    .recalculate_derived_stats(&enhancement.levels);
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
                                            if let GameOverlay::TimeVault { ref mut browser } =
                                                overlay
                                            {
                                                if let Ok(branches) = repo.list_branches() {
                                                    browser.branches = branches;
                                                    if browser.selected_branch
                                                        >= browser.branches.len()
                                                    {
                                                        browser.selected_branch = browser
                                                            .branches
                                                            .len()
                                                            .saturating_sub(1);
                                                    }
                                                    if let Some(b) = browser
                                                        .branches
                                                        .get(browser.selected_branch)
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
                                        save_files(
                                            &character_manager,
                                            &state,
                                            &global_achievements,
                                            &haven,
                                            &enhancement,
                                            &deep_state,
                                        );
                                        last_save_instant = Some(Instant::now());
                                        last_save_time = Some(Local::now());
                                        if commit_save(&state, &history::SaveEvent::AutoSave, repo)
                                        {
                                            last_commit_instant = Some(Instant::now());
                                            last_commit_time = Some(Local::now());
                                        }
                                        if repo.fork_timeline(branch_name, commit_id).is_ok() {
                                            // Full state reload (fork checks out the new branch)
                                            main_helpers::cloud_ops::reload_account_state(
                                                &mut haven,
                                                &mut enhancement,
                                                &mut global_achievements,
                                            );

                                            let filename = format!("{}.json", state.character_name);
                                            if let Ok(mut reloaded) =
                                                character_manager.load_character(&filename)
                                            {
                                                reloaded
                                                    .recalculate_derived_stats(&enhancement.levels);
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
                                            if let GameOverlay::TimeVault { ref mut browser } =
                                                overlay
                                            {
                                                if let Ok(branches) = repo.list_branches() {
                                                    browser.branches = branches;
                                                    if browser.selected_branch
                                                        >= browser.branches.len()
                                                    {
                                                        browser.selected_branch = browser
                                                            .branches
                                                            .len()
                                                            .saturating_sub(1);
                                                    }
                                                    if let Some(b) = browser
                                                        .branches
                                                        .get(browser.selected_branch)
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
                                        save_files(
                                            &character_manager,
                                            &state,
                                            &global_achievements,
                                            &haven,
                                            &enhancement,
                                            &deep_state,
                                        );
                                        last_save_instant = Some(Instant::now());
                                        last_save_time = Some(Local::now());
                                        if commit_save(&state, &history::SaveEvent::AutoSave, repo)
                                        {
                                            last_commit_instant = Some(Instant::now());
                                            last_commit_time = Some(Local::now());
                                        }
                                        if repo.switch_timeline(branch_name).is_ok() {
                                            // Full state reload (switch checks out the branch)
                                            main_helpers::cloud_ops::reload_account_state(
                                                &mut haven,
                                                &mut enhancement,
                                                &mut global_achievements,
                                            );

                                            let filename = format!("{}.json", state.character_name);
                                            if let Ok(mut reloaded) =
                                                character_manager.load_character(&filename)
                                            {
                                                reloaded
                                                    .recalculate_derived_stats(&enhancement.levels);
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
                                            if let GameOverlay::TimeVault { ref mut browser } =
                                                overlay
                                            {
                                                if let Ok(branches) = repo.list_branches() {
                                                    // Select the switched-to branch (sort order changes)
                                                    browser.selected_branch = branches
                                                        .iter()
                                                        .position(|b| b.name == *branch_name)
                                                        .unwrap_or(0);
                                                    browser.branches = branches;
                                                    if let Some(b) = browser
                                                        .branches
                                                        .get(browser.selected_branch)
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
                                            if let GameOverlay::TimeVault { ref mut browser } =
                                                overlay
                                            {
                                                if let Ok(branches) = repo.list_branches() {
                                                    browser.branches = branches;
                                                    // Clamp selection
                                                    if browser.selected_branch
                                                        >= browser.branches.len()
                                                    {
                                                        browser.selected_branch = browser
                                                            .branches
                                                            .len()
                                                            .saturating_sub(1);
                                                    }
                                                    // Refresh commits for new selection
                                                    if let Some(b) = browser
                                                        .branches
                                                        .get(browser.selected_branch)
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

                                // Handle async cloud sync actions before routing
                                if main_helpers::cloud_sync::dispatch_cloud_action(
                                    &result,
                                    &mut cloud,
                                    &mut overlay,
                                    &quest_dir,
                                ) {
                                    continue;
                                }

                                // Handle blocking resolve cloud actions
                                if let Some(resolve_result) =
                                    main_helpers::cloud_sync::apply_cloud_resolve(
                                        &result,
                                        &mut cloud,
                                        &mut overlay,
                                        &quest_dir,
                                        &character_manager,
                                        &state.character_name,
                                        &enhancement,
                                        history_repo.as_ref(),
                                    )
                                {
                                    if resolve_result.account_state_reloaded {
                                        main_helpers::cloud_ops::reload_account_state(
                                            &mut haven,
                                            &mut enhancement,
                                            &mut global_achievements,
                                        );
                                    }
                                    if let Some(reloaded) = resolve_result.reloaded_state {
                                        state = reloaded;
                                    }
                                    if resolve_result.needs_save_timestamp {
                                        state.last_save_time = Utc::now().timestamp();
                                    }
                                    continue;
                                }

                                let route_action = {
                                    let route_ctx = main_helpers::game_context::GameContext {
                                        state: &mut state,
                                        haven: &mut haven,
                                        haven_ui: &mut haven_ui,
                                        soulforge_ui: &mut soulforge_ui,
                                        exchange_ui: &mut exchange_ui,
                                        deep_state: &mut deep_state,
                                        deep_ui: &mut deep_ui,
                                        enhancement: &mut enhancement,
                                        overlay: &mut overlay,
                                        debug_menu: &mut debug_menu,
                                        debug_mode,
                                        achievements: &mut global_achievements,
                                    };
                                    route_game_input(
                                        result,
                                        &route_ctx,
                                        &character_manager,
                                        &mut last_save_instant,
                                        &mut last_save_time,
                                        &mut last_commit_instant,
                                        &mut last_commit_time,
                                        history_repo.as_ref(),
                                    )
                                };
                                match route_action {
                                    InputAction::QuitToSelect => {
                                        break 'game_loop;
                                    }
                                    InputAction::ContinueAndPush => {
                                        if !cloud.op_in_flight {
                                            if let Some(ref config) = cloud.config {
                                                main_helpers::cloud_ops::spawn_cloud_push(
                                                    &mut cloud.op_in_flight,
                                                    &cloud.tx,
                                                    &quest_dir,
                                                    &config.token,
                                                );
                                            }
                                        }
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
                                    challenges::snake::logic::tick_snake(
                                        game,
                                        dt.as_millis() as u64,
                                    );
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
                                    GameOverlay::OfflineWelcome { .. }
                                        | GameOverlay::TimeVault { .. }
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
                                    save_files(
                                        &character_manager,
                                        &state,
                                        &global_achievements,
                                        &haven,
                                        &enhancement,
                                        &deep_state,
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
                            let batch_result = run_chrono_surge_batch(
                                chrono_surge.as_mut().unwrap(),
                                &mut state,
                                &mut tick_counter,
                                &mut haven,
                                &mut enhancement,
                                &mut deep_state,
                                &mut global_achievements,
                                debug_mode,
                            );

                            if batch_result.needs_save && !debug_mode {
                                save_files(
                                    &character_manager,
                                    &state,
                                    &global_achievements,
                                    &haven,
                                    &enhancement,
                                    &deep_state,
                                );
                            }

                            if let Some((summary, surge_event)) = batch_result.completed {
                                state.chrono_surge_active = false;
                                chrono_surge = None;
                                chrono_summary = Some(summary);
                                if !debug_mode {
                                    save_files(
                                        &character_manager,
                                        &state,
                                        &global_achievements,
                                        &haven,
                                        &enhancement,
                                        &deep_state,
                                    );
                                    last_save_instant = Some(Instant::now());
                                    last_save_time = Some(Local::now());
                                    if let Some(ref repo) = history_repo {
                                        if commit_save(&state, &surge_event, repo) {
                                            last_commit_instant = Some(Instant::now());
                                            last_commit_time = Some(Local::now());
                                        }
                                    }
                                }
                            }

                            last_tick = Instant::now();
                        }

                        // Game tick every 100ms (normal — not during surge)
                        if chrono_surge.is_none()
                            && last_tick.elapsed() >= Duration::from_millis(TICK_INTERVAL_MS)
                        {
                            if !matches!(
                                overlay,
                                GameOverlay::LeviathanEncounter { .. }
                                    | GameOverlay::LeviathanCatchMiss { .. }
                            ) {
                                let mut rng = rand::rng();
                                let mut ctx = core::tick_context::TickContext {
                                    state: &mut state,
                                    tick_counter: &mut tick_counter,
                                    haven: &mut haven,
                                    enhancement: &mut enhancement,
                                    deep: &mut deep_state,
                                    achievements: &mut global_achievements,
                                    debug_mode,
                                };
                                let tick_result =
                                    core::tick::game_tick_with_context(&mut ctx, &mut rng);

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
                                    || tick_result.deep_changed
                                    || save_event.is_some())
                                    && !debug_mode
                                {
                                    save_files(
                                        &character_manager,
                                        &state,
                                        &global_achievements,
                                        &haven,
                                        &enhancement,
                                        &deep_state,
                                    );
                                    last_save_instant = Some(Instant::now());
                                    last_save_time = Some(Local::now());
                                    if let (Some(event), Some(repo)) =
                                        (save_event.as_ref(), history_repo.as_ref())
                                    {
                                        if commit_save(&state, event, repo) {
                                            last_commit_instant = Some(Instant::now());
                                            last_commit_time = Some(Local::now());
                                        }
                                    }

                                    // Push to cloud after milestone commits
                                    if save_event.is_some() && !cloud.op_in_flight {
                                        if let Some(ref config) = cloud.config {
                                            main_helpers::cloud_ops::spawn_cloud_push(
                                                &mut cloud.op_in_flight,
                                                &cloud.tx,
                                                &quest_dir,
                                                &config.token,
                                            );
                                        }
                                    }
                                }

                                // Queue discovery/encounter overlays so they
                                // never clobber each other or get lost behind
                                // an open UI panel (Deep, Haven, etc.).
                                if let Some(encounter_number) = tick_result.leviathan_encounter {
                                    pending_overlays.push_back(GameOverlay::LeviathanEncounter {
                                        encounter_number,
                                        lure_consumed: tick_result.leviathan_lure_consumed,
                                    });
                                } else if tick_result.leviathan_catch_miss {
                                    pending_overlays.push_back(GameOverlay::LeviathanCatchMiss {
                                        lure_consumed: tick_result.leviathan_lure_consumed,
                                    });
                                }
                                if tick_flags.haven_discovered {
                                    pending_overlays.push_back(GameOverlay::HavenDiscovery);
                                }
                                if tick_flags.soulforge_discovered {
                                    pending_overlays.push_back(GameOverlay::SoulforgeDiscovery);
                                }
                                if tick_flags.stormglass_discovered {
                                    pending_overlays.push_back(GameOverlay::StormglassDiscovery);
                                }
                                if tick_flags.deep_discovered {
                                    pending_overlays.push_back(GameOverlay::DeepDiscovery);
                                }
                                if let Some(region) = tick_flags.fracture_region_unlocked {
                                    pending_overlays
                                        .push_back(GameOverlay::FractureRegionUnlock { region });
                                }
                                if !tick_result.achievement_modal_ready.is_empty() {
                                    pending_overlays.push_back(GameOverlay::AchievementUnlocked {
                                        achievements: tick_result.achievement_modal_ready,
                                    });
                                }

                                // Show the next pending overlay when the screen
                                // is clear (no active overlay, no open UI panel).
                                if matches!(overlay, GameOverlay::None)
                                    && !pending_overlays.is_empty()
                                    && !deep_ui.open
                                    && !haven_ui.showing
                                    && !soulforge_ui.open
                                    && !exchange_ui.open
                                {
                                    overlay = pending_overlays.pop_front().unwrap();
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
                                                state
                                                    .recalculate_derived_stats(&enhancement.levels);
                                                state.recalculate_prestige_bonuses();

                                                const SLOTS: [crate::items::types::EquipmentSlot;
                                                    7] = [
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
                                                    save_files(
                                                        &character_manager,
                                                        &state,
                                                        &global_achievements,
                                                        &haven,
                                                        &enhancement,
                                                        &deep_state,
                                                    );
                                                    last_save_instant = Some(Instant::now());
                                                    last_save_time = Some(Local::now());
                                                    if let (Some(event), Some(repo)) = (
                                                        soulforge_event.as_ref(),
                                                        history_repo.as_ref(),
                                                    ) {
                                                        if commit_save(&state, event, repo) {
                                                            last_commit_instant =
                                                                Some(Instant::now());
                                                            last_commit_time = Some(Local::now());
                                                        }
                                                    }
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
                        if last_autosave.elapsed() >= Duration::from_secs(AUTOSAVE_INTERVAL_SECONDS)
                        {
                            // Sync in-memory last_save_time so suspension detection
                            // only counts actual suspension time, not active play time
                            state.last_save_time = Utc::now().timestamp();
                            last_autosave = Instant::now();
                            last_save_time = Some(Local::now());

                            // Skip file I/O in debug mode
                            if !debug_mode {
                                save_files(
                                    &character_manager,
                                    &state,
                                    &global_achievements,
                                    &haven,
                                    &enhancement,
                                    &deep_state,
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
                } // end of game block

                // After game loop exits (QuitToSelect), transition back to splash
                play_screen_transition(&mut terminal)?;
                terminal.clear()?;
                continue 'main_loop;
            }
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;

    println!("Goodbye!");

    Ok(())
}
