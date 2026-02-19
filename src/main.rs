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
mod tick_events;
mod ui;
mod utils;
mod zones;

use character::input::{
    process_creation_input, process_delete_input, process_rename_input, process_select_input,
    CreationInput, CreationResult, DeleteInput, DeleteResult, RenameInput, RenameResult,
    SelectInput, SelectResult,
};
use character::manager::CharacterManager;
use chrono::{Local, Utc};
use core::constants::*;
use core::game_state::*;
use input::{GameOverlay, HavenUiState, InputResult, SoulforgeUiState};
use main_helpers::achievements::{log_synced_achievements, track_input_achievements};
use main_helpers::offline::apply_offline_xp;
use main_helpers::persistence::save_all;
use main_helpers::scene::{current_scene_kind, is_realtime_minigame, is_wide_scene};
use main_helpers::update::{jittered_update_interval, show_startup_update_notification};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::crossterm::ExecutableCommand;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};
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

/// Draws the quit confirmation dialog when pending challenges exist.
fn draw_quit_confirm(frame: &mut ratatui::Frame, pending_count: usize) {
    use ratatui::{
        layout::{Alignment, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Clear, Paragraph},
    };

    let size = frame.area();
    let w = 42.min(size.width.saturating_sub(4));
    let h = 8.min(size.height.saturating_sub(4));
    let x = (size.width.saturating_sub(w)) / 2;
    let y = (size.height.saturating_sub(h)) / 2;
    let area = Rect::new(x, y, w, h);

    frame.render_widget(Clear, area);

    let challenge_word = if pending_count == 1 {
        "challenge"
    } else {
        "challenges"
    };

    let lines = vec![
        Line::from(""),
        Line::from(format!(
            "  {} pending {} will be lost.",
            pending_count, challenge_word
        )),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "[Enter] Leave",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled(
                "[Esc] Stay",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(
                Line::from(Span::styled(
                    " Unsaved Challenges ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(paragraph, area);
}

/// Draw all game overlays on top of the main game UI.
#[allow(clippy::too_many_arguments)]
fn draw_game_overlays(
    frame: &mut ratatui::Frame,
    state: &GameState,
    overlay: &GameOverlay,
    haven: &haven::Haven,
    haven_ui: &HavenUiState,
    soulforge_ui: &SoulforgeUiState,
    enhancement: &enhancement::EnhancementProgress,
    global_achievements: &achievements::Achievements,
    debug_mode: bool,
    debug_menu: &utils::debug_menu::DebugMenu,
    last_save_instant: Option<Instant>,
    last_save_time: Option<chrono::DateTime<chrono::Local>>,
    ctx: &ui::responsive::LayoutContext,
) {
    let area = frame.area();
    match overlay {
        GameOverlay::OfflineWelcome { report } => {
            ui::game_common::render_offline_welcome(frame, area, report, ctx);
        }
        GameOverlay::PrestigeConfirm => {
            ui::prestige_confirm::draw_prestige_confirm(frame, state, ctx);
        }
        GameOverlay::HavenDiscovery => {
            ui::haven_scene::render_haven_discovery_modal(frame, area, ctx);
        }
        GameOverlay::SoulforgeDiscovery => {
            ui::soulforge_scene::render_soulforge_discovery_modal(frame, area, ctx);
        }
        GameOverlay::AchievementUnlocked { ref achievements } => {
            ui::achievement_browser_scene::render_achievement_unlocked_modal(
                frame,
                area,
                achievements,
                ctx,
            );
        }
        GameOverlay::VaultSelection {
            selected_index,
            ref selected_slots,
            confirm_pending,
        } => {
            ui::haven_scene::render_vault_selection(
                frame,
                area,
                state,
                haven.get_bonus(haven::HavenBonusType::VaultSlots) as u8,
                *selected_index,
                selected_slots,
                &enhancement.levels,
                *confirm_pending,
                ctx,
            );
        }
        GameOverlay::Achievements { browser } => {
            ui::achievement_browser_scene::render_achievement_browser(
                frame,
                area,
                global_achievements,
                browser,
                enhancement,
                ctx,
            );
        }
        GameOverlay::LeviathanEncounter { encounter_number } => {
            ui::fishing_scene::render_leviathan_encounter_modal(
                frame,
                area,
                *encounter_number,
                ctx,
            );
        }
        GameOverlay::QuitConfirm => {
            draw_quit_confirm(frame, state.challenge_menu.challenges.len());
        }
        GameOverlay::Help => {
            ui::help_overlay::draw_help_overlay(frame);
        }
        GameOverlay::None => {}
    }

    // Haven screen overlay
    if haven_ui.showing {
        ui::haven_scene::render_haven_tree(
            frame,
            area,
            haven,
            haven_ui.selected_room,
            state.prestige_rank,
            global_achievements,
            ctx,
        );
        match haven_ui.confirmation {
            input::HavenConfirmation::Build => {
                let room = haven::HavenRoomId::ALL[haven_ui.selected_room];
                ui::haven_scene::render_build_confirmation(
                    frame,
                    area,
                    room,
                    haven,
                    state.prestige_rank,
                    ctx,
                );
            }
            input::HavenConfirmation::Forge => {
                ui::haven_scene::render_forge_confirmation(
                    frame,
                    area,
                    global_achievements,
                    state.prestige_rank,
                    ctx,
                );
            }
            input::HavenConfirmation::None => {}
        }
    }

    // Soulforge overlay
    if soulforge_ui.open {
        ui::soulforge_scene::render_soulforge(
            frame,
            area,
            soulforge_ui,
            enhancement,
            state.prestige_rank,
            ctx,
        );
    }

    // Debug indicator / save indicator
    if debug_mode {
        ui::debug_menu_scene::render_debug_indicator(frame, area, ctx);
        if debug_menu.is_open {
            ui::debug_menu_scene::render_debug_menu(frame, area, debug_menu, ctx);
        }
    } else {
        let is_saving = last_save_instant
            .map(|t| t.elapsed() < Duration::from_secs(1))
            .unwrap_or(false);
        ui::debug_menu_scene::render_save_indicator(frame, area, is_saving, last_save_time, ctx);
    }
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

    // Check for updates in background (non-blocking notification)
    let update_available = std::thread::spawn(utils::updater::check_update_info);

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
    let mut achievement_browser = AchievementBrowserState::new();
    let mut help_overlay_showing = false;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Show update notification if available
    if let Ok(Some(update_info)) = update_available.join() {
        show_startup_update_notification(&mut terminal, &update_info)?;
    }

    // Main loop — clear terminal on screen transitions to prevent
    // stale cells from wide characters (emoji) in the ratatui diff.
    let mut prev_screen = current_screen;
    loop {
        if current_screen != prev_screen {
            terminal.clear()?;
            prev_screen = current_screen;
        }
        match current_screen {
            Screen::CharacterCreation => {
                // Draw character creation screen
                terminal.draw(|f| {
                    let area = f.area();
                    let ctx = ui::responsive::LayoutContext::from_frame(f);
                    creation_screen.draw(f, area, &ctx);
                })?;

                // Handle input
                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key_event) = event::read()? {
                        if key_event.kind != KeyEventKind::Press {
                            continue;
                        }
                        let input = match key_event.code {
                            KeyCode::Char(c) => CreationInput::Char(c),
                            KeyCode::Backspace => CreationInput::Backspace,
                            KeyCode::Enter => CreationInput::Submit,
                            KeyCode::Esc => CreationInput::Cancel,
                            _ => CreationInput::Other,
                        };

                        let has_existing = !character_manager.list_characters()?.is_empty();
                        let result = process_creation_input(
                            &mut creation_screen,
                            input,
                            &character_manager,
                            has_existing,
                        );

                        match result {
                            CreationResult::Created | CreationResult::Cancelled => {
                                creation_screen = CharacterCreationScreen::new();
                                select_screen = CharacterSelectScreen::new();
                                current_screen = Screen::CharacterSelect;
                            }
                            CreationResult::Continue | CreationResult::SaveFailed(_) => {}
                        }
                    }
                }
            }

            Screen::CharacterSelect => {
                // Refresh character list
                let characters = character_manager.list_characters()?;

                // Draw character select screen (includes Haven tree visualization)
                terminal.draw(|f| {
                    let area = f.area();
                    let ctx = ui::responsive::LayoutContext::from_frame(f);
                    select_screen.draw(f, area, &characters, &haven, &enhancement, &ctx);
                    // Draw Haven management overlay if open
                    if haven_ui.showing {
                        ui::haven_scene::render_haven_tree(
                            f,
                            area,
                            &haven,
                            haven_ui.selected_room,
                            0, // No character selected, so prestige rank = 0
                            &global_achievements,
                            &ctx,
                        );
                    }
                    // Draw achievement browser overlay if open
                    if achievement_browser.showing {
                        ui::achievement_browser_scene::render_achievement_browser(
                            f,
                            area,
                            &global_achievements,
                            &achievement_browser,
                            &enhancement,
                            &ctx,
                        );
                    }
                    // Draw Soulforge overlay if open
                    if soulforge_ui.open {
                        ui::soulforge_scene::render_soulforge(
                            f,
                            area,
                            &soulforge_ui,
                            &enhancement,
                            0,
                            &ctx,
                        );
                    }
                    // Draw Help overlay if open
                    if help_overlay_showing {
                        ui::help_overlay::draw_help_overlay(f);
                    }
                })?;

                // Handle input
                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key_event) = event::read()? {
                        if key_event.kind != KeyEventKind::Press {
                            continue;
                        }
                        // Handle Soulforge overlay (blocks other input when open)
                        if soulforge_ui.open {
                            match key_event.code {
                                KeyCode::Up => {
                                    if soulforge_ui.selected_slot > 0 {
                                        soulforge_ui.selected_slot -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    if soulforge_ui.selected_slot < 6 {
                                        soulforge_ui.selected_slot += 1;
                                    }
                                }
                                KeyCode::Esc => soulforge_ui.close(),
                                _ => {}
                            }
                            continue;
                        }

                        // Handle Help overlay (blocks other input when open)
                        if help_overlay_showing {
                            if matches!(key_event.code, KeyCode::Esc | KeyCode::Char('?')) {
                                help_overlay_showing = false;
                            }
                            continue;
                        }

                        // Handle achievement browser (blocks other input when open)
                        if achievement_browser.showing {
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
                                KeyCode::Esc => {
                                    global_achievements.clear_recently_unlocked();
                                    achievement_browser.close();
                                }
                                _ => {}
                            }
                            continue;
                        }

                        // Handle achievement browser shortcut
                        if matches!(key_event.code, KeyCode::Char('a') | KeyCode::Char('A')) {
                            global_achievements.clear_pending_notifications();
                            achievement_browser.open();
                            continue;
                        }

                        // Help overlay shortcut
                        if key_event.code == KeyCode::Char('?') {
                            help_overlay_showing = true;
                            continue;
                        }

                        let input = match key_event.code {
                            KeyCode::Up => SelectInput::Up,
                            KeyCode::Down => SelectInput::Down,
                            KeyCode::Enter => SelectInput::Select,
                            KeyCode::Char('n') | KeyCode::Char('N') => SelectInput::New,
                            KeyCode::Char('d') | KeyCode::Char('D') => SelectInput::Delete,
                            KeyCode::Char('r') | KeyCode::Char('R') => SelectInput::Rename,
                            KeyCode::Esc => SelectInput::Quit,
                            _ => SelectInput::Other,
                        };

                        let result = process_select_input(&mut select_screen, input, &characters);

                        match result {
                            SelectResult::NoCharacters => {
                                current_screen = Screen::CharacterCreation;
                            }
                            SelectResult::LoadCharacter(filename) => {
                                match character_manager.load_character(&filename) {
                                    Ok(mut state) => {
                                        // Initialize cached derived stats and prestige bonuses
                                        state.recalculate_derived_stats(&enhancement.levels);
                                        state.recalculate_prestige_bonuses();

                                        // Sanity check: clear stale enemy if HP is impossibly high
                                        // (can happen if save was from before prestige reset)
                                        let derived = state.cached_derived_stats;
                                        if let Some(enemy) = &state.combat_state.current_enemy {
                                            // Max possible enemy HP is 2.4x player HP (boss with max variance)
                                            // If enemy HP is > 2.5x, it's stale from before a stat reset
                                            if enemy.max_hp > (derived.max_hp as f64 * 2.5) as u32 {
                                                state.combat_state.current_enemy = None;
                                            }
                                        }

                                        // Sync achievements from character state (retroactive unlocks)
                                        let defeated_bosses =
                                            state.zone_progression.defeated_bosses.to_vec();
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
                                            global_achievements.on_soulforge_discovered(Some(
                                                &state.character_name,
                                            ));
                                        }
                                        global_achievements.on_enhancement_upgraded(
                                            enhancement.highest_level_reached,
                                            &enhancement.levels,
                                            enhancement.total_attempts,
                                            Some(&state.character_name),
                                        );

                                        log_synced_achievements(
                                            &mut state,
                                            &mut global_achievements,
                                        );

                                        // Process offline progression
                                        let current_time = Utc::now().timestamp();
                                        let elapsed_seconds = current_time - state.last_save_time;

                                        if elapsed_seconds > 60 {
                                            if let Some(report) =
                                                apply_offline_xp(&mut state, &haven)
                                            {
                                                pending_offline_report = Some(report);
                                            }
                                        }
                                        // Always sync last_save_time on load so suspension
                                        // detection doesn't false-trigger from a stale value
                                        state.last_save_time = Utc::now().timestamp();

                                        game_state = Some(state);
                                        current_screen = Screen::Game;
                                    }
                                    Err(e) => {
                                        // Could show error message, for now just stay on select
                                        eprintln!("Failed to load character: {}", e);
                                    }
                                }
                            }
                            SelectResult::GoToCreation => {
                                creation_screen = CharacterCreationScreen::new();
                                current_screen = Screen::CharacterCreation;
                            }
                            SelectResult::GoToDelete => {
                                delete_screen = CharacterDeleteScreen::new();
                                current_screen = Screen::CharacterDelete;
                            }
                            SelectResult::GoToRename => {
                                rename_screen = CharacterRenameScreen::new();
                                current_screen = Screen::CharacterRename;
                            }
                            SelectResult::Quit => {
                                break;
                            }
                            SelectResult::Continue | SelectResult::LoadFailed(_) => {}
                        }
                    }
                }
            }

            Screen::CharacterDelete => {
                // Get current character list and selected character
                let characters = character_manager.list_characters()?;
                if characters.is_empty() || select_screen.selected_index >= characters.len() {
                    current_screen = Screen::CharacterSelect;
                    continue;
                }
                let selected_character = &characters[select_screen.selected_index];

                // Draw delete confirmation screen
                terminal.draw(|f| {
                    let area = f.area();
                    let ctx = ui::responsive::LayoutContext::from_frame(f);
                    delete_screen.draw(f, area, selected_character, &ctx);
                })?;

                // Handle input
                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key_event) = event::read()? {
                        if key_event.kind != KeyEventKind::Press {
                            continue;
                        }
                        let input = match key_event.code {
                            KeyCode::Char(c) => DeleteInput::Char(c),
                            KeyCode::Backspace => DeleteInput::Backspace,
                            KeyCode::Enter => DeleteInput::Submit,
                            KeyCode::Esc => DeleteInput::Cancel,
                            _ => DeleteInput::Other,
                        };

                        let result = process_delete_input(
                            &mut delete_screen,
                            input,
                            &character_manager,
                            selected_character,
                        );

                        match result {
                            DeleteResult::Deleted | DeleteResult::Cancelled => {
                                delete_screen = CharacterDeleteScreen::new();
                                select_screen.selected_index = 0;
                                current_screen = Screen::CharacterSelect;
                            }
                            DeleteResult::DeleteFailed(e) => {
                                eprintln!("Failed to delete character: {}", e);
                            }
                            DeleteResult::Continue => {}
                        }
                    }
                }
            }

            Screen::CharacterRename => {
                // Get current character list and selected character
                let characters = character_manager.list_characters()?;
                if characters.is_empty() || select_screen.selected_index >= characters.len() {
                    current_screen = Screen::CharacterSelect;
                    continue;
                }
                let selected_character = &characters[select_screen.selected_index];

                // Draw rename screen
                terminal.draw(|f| {
                    let area = f.area();
                    let ctx = ui::responsive::LayoutContext::from_frame(f);
                    rename_screen.draw(f, area, selected_character, &ctx);
                })?;

                // Handle input
                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key_event) = event::read()? {
                        if key_event.kind != KeyEventKind::Press {
                            continue;
                        }
                        let input = match key_event.code {
                            KeyCode::Char(c) => RenameInput::Char(c),
                            KeyCode::Backspace => RenameInput::Backspace,
                            KeyCode::Enter => RenameInput::Submit,
                            KeyCode::Esc => RenameInput::Cancel,
                            _ => RenameInput::Other,
                        };

                        let result = process_rename_input(
                            &mut rename_screen,
                            input,
                            &character_manager,
                            selected_character,
                        );

                        match result {
                            RenameResult::Renamed | RenameResult::Cancelled => {
                                rename_screen = CharacterRenameScreen::new();
                                current_screen = Screen::CharacterSelect;
                            }
                            RenameResult::RenameFailed(_) | RenameResult::Continue => {}
                        }
                    }
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
                            &enhancement,
                            &global_achievements,
                            debug_mode,
                            &debug_menu,
                            last_save_instant,
                            last_save_time,
                            &ctx,
                        );
                    })?;

                    // Adaptive polling:
                    // - Realtime minigames: block only until the next frame boundary to avoid
                    //   busy-spinning and burning CPU between updates.
                    // - Normal mode: 50ms block to keep idle CPU low while responsive.
                    let realtime_mode = is_realtime_minigame(&state);
                    let mut poll_duration = if realtime_mode {
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
                            // Track prestige rank before input to detect prestige
                            let prestige_before = state.prestige_rank;

                            let result = input::handle_game_input(
                                key_event,
                                &mut state,
                                &mut haven,
                                &mut haven_ui,
                                &mut soulforge_ui,
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

                            match result {
                                InputResult::Continue => {}
                                InputResult::QuitToSelect => {
                                    if !debug_mode {
                                        save_all(
                                            &character_manager,
                                            &state,
                                            &global_achievements,
                                            &haven,
                                            &enhancement,
                                        );
                                    }
                                    game_state = None;
                                    current_screen = Screen::CharacterSelect;
                                    break 'game_loop;
                                }
                                InputResult::NeedsSave => {
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
                                InputResult::NeedsSaveAll => {
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
                                InputResult::ToggleUpdateDetails => {
                                    update_expanded = !update_expanded;
                                }
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

                    // Game tick every 100ms
                    if last_tick.elapsed() >= Duration::from_millis(TICK_INTERVAL_MS) {
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

