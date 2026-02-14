mod achievements;
mod challenges;
mod character;
mod combat;
mod core;
mod dungeon;
mod fishing;
mod haven;
mod input;
mod items;
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
use core::game_logic::*;
use core::game_state::*;
use input::{GameOverlay, HavenUiState, InputResult};
use rand::RngExt;
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

/// Process offline XP and add combat log entries. Returns the report if XP was gained.
fn apply_offline_xp(state: &mut GameState, haven: &haven::Haven) -> Option<OfflineReport> {
    let haven_offline_bonus = haven.get_bonus(haven::HavenBonusType::OfflineXpPercent);
    let report = process_offline_progression(state, haven_offline_bonus);
    if report.xp_gained > 0 {
        let hours = report.elapsed_seconds / 3600;
        let minutes = (report.elapsed_seconds % 3600) / 60;
        let away_str = if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        };
        state.combat_state.add_log_entry(
            format!("☀️ Welcome back! ({} away)", away_str),
            false,
            true,
        );
        state.combat_state.add_log_entry(
            format!(
                "⚔️ +{} XP gained offline",
                ui::game_common::format_number_short(report.xp_gained)
            ),
            false,
            true,
        );
        if report.total_level_ups > 0 {
            state.combat_state.add_log_entry(
                format!(
                    "📈 Leveled up {} times! ({} → {})",
                    report.total_level_ups, report.level_before, report.level_after,
                ),
                false,
                true,
            );
        }
        Some(report)
    } else {
        None
    }
}

enum Screen {
    CharacterSelect,
    CharacterCreation,
    CharacterDelete,
    CharacterRename,
    Game,
}

/// Returns the update check interval with random jitter applied.
/// Jitter spreads checks across [base - jitter, base + jitter] to avoid
/// simultaneous API requests from many clients.
fn jittered_update_interval() -> Duration {
    let mut rng = rand::rng();
    let jitter = rng.random_range(0..=2 * UPDATE_CHECK_JITTER_SECONDS);
    let interval = UPDATE_CHECK_INTERVAL_SECONDS - UPDATE_CHECK_JITTER_SECONDS + jitter;
    Duration::from_secs(interval)
}

/// Show update notification with changelog at startup, then wait for keypress.
fn show_startup_update_notification(
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
                text.push(ratatui::text::Line::from(format!("    • {}", entry)));
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

/// Draw all game overlays on top of the main game UI.
#[allow(clippy::too_many_arguments)]
fn draw_game_overlays(
    frame: &mut ratatui::Frame,
    state: &GameState,
    overlay: &GameOverlay,
    haven: &haven::Haven,
    haven_ui: &HavenUiState,
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
        } => {
            ui::haven_scene::render_vault_selection(
                frame,
                area,
                state,
                haven.get_bonus(haven::HavenBonusType::VaultSlots) as u8,
                *selected_index,
                selected_slots,
                ctx,
            );
        }
        GameOverlay::Achievements { browser } => {
            ui::achievement_browser_scene::render_achievement_browser(
                frame,
                area,
                global_achievements,
                browser,
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

/// Log synced achievements to the combat log after loading a character.
fn log_synced_achievements(
    state: &mut GameState,
    global_achievements: &mut achievements::Achievements,
) {
    let synced_count = global_achievements.pending_count();
    if synced_count > 0 {
        if synced_count == 1 {
            if let Some(id) = global_achievements.pending_notifications.first() {
                if let Some(def) = achievements::get_achievement_def(*id) {
                    state.combat_state.add_log_entry(
                        format!("\u{1f3c6} Achievement Unlocked: {}", def.name),
                        false,
                        true,
                    );
                }
            }
        } else {
            state.combat_state.add_log_entry(
                format!(
                    "\u{1f3c6} {} achievements synced from progress!",
                    synced_count
                ),
                false,
                true,
            );
        }
        global_achievements.newly_unlocked.clear();
    }
}

/// Track achievements that may have changed from input handling (prestige, minigame wins).
fn track_input_achievements(
    state: &mut GameState,
    global_achievements: &mut achievements::Achievements,
    prestige_before: u32,
    debug_mode: bool,
) {
    let mut achievements_changed = false;

    if state.prestige_rank > prestige_before {
        global_achievements.on_prestige(state.prestige_rank, Some(&state.character_name));
        achievements_changed = true;
    }

    if let Some(ref win_info) = state.last_minigame_win {
        global_achievements.on_minigame_won(
            win_info.game_type,
            win_info.difficulty,
            Some(&state.character_name),
        );
        achievements_changed = true;
        state.last_minigame_win = None;
    }

    if achievements_changed && !debug_mode {
        if let Err(e) = achievements::save_achievements(global_achievements) {
            eprintln!("Failed to save achievements: {}", e);
        }
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

    // Load global achievements (shared across all characters)
    let mut global_achievements = achievements::load_achievements();

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
    let mut achievement_browser = AchievementBrowserState::new();

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

    // Main loop
    loop {
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
                    select_screen.draw(f, area, &characters, &haven, &ctx);
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
                            &ctx,
                        );
                    }
                })?;

                // Handle input
                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key_event) = event::read()? {
                        if key_event.kind != KeyEventKind::Press {
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

                        // Handle Haven screen (blocks other input when open)
                        if haven_ui.showing {
                            if haven_ui.confirmation == input::HavenConfirmation::Build {
                                match key_event.code {
                                    KeyCode::Enter => {
                                        // Note: Can't build from character select (no active character)
                                        // Just close the confirmation
                                        haven_ui.confirmation = input::HavenConfirmation::None;
                                    }
                                    KeyCode::Esc => {
                                        haven_ui.confirmation = input::HavenConfirmation::None;
                                    }
                                    _ => {}
                                }
                            } else {
                                match key_event.code {
                                    KeyCode::Up => {
                                        haven_ui.selected_room =
                                            haven_ui.selected_room.saturating_sub(1);
                                    }
                                    KeyCode::Down => {
                                        if haven_ui.selected_room + 1
                                            < haven::HavenRoomId::ALL.len()
                                        {
                                            haven_ui.selected_room += 1;
                                        }
                                    }
                                    KeyCode::Esc => {
                                        haven_ui.close();
                                    }
                                    _ => {}
                                }
                            }
                            continue;
                        }

                        // Handle achievement browser shortcut
                        if matches!(key_event.code, KeyCode::Char('a') | KeyCode::Char('A')) {
                            global_achievements.clear_pending_notifications();
                            achievement_browser.open();
                            continue;
                        }

                        // Handle Haven shortcut (if discovered)
                        if matches!(key_event.code, KeyCode::Char('h') | KeyCode::Char('H'))
                            && haven.discovered
                        {
                            haven_ui.open();
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
                                        // Sanity check: clear stale enemy if HP is impossibly high
                                        // (can happen if save was from before prestige reset)
                                        let derived = character::derived_stats::DerivedStats::calculate_derived_stats(
                                            &state.attributes,
                                            &state.equipment,
                                        );
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
                            &global_achievements,
                        );
                        draw_game_overlays(
                            frame,
                            &state,
                            &overlay,
                            &haven,
                            &haven_ui,
                            &global_achievements,
                            debug_mode,
                            &debug_menu,
                            last_save_instant,
                            last_save_time,
                            &ctx,
                        );
                    })?;

                    // Adaptive polling: non-blocking drain in realtime mode, 50ms block otherwise
                    let realtime_mode = is_realtime_minigame(&state);
                    let poll_duration = if realtime_mode {
                        Duration::ZERO
                    } else {
                        Duration::from_millis(50)
                    };

                    // Drain all available events (critical for responsive input at 30+ FPS)
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
                                debug_mode,
                            );

                            match result {
                                InputResult::Continue => {}
                                InputResult::QuitToSelect => {
                                    if !debug_mode {
                                        character_manager.save_character(&state)?;
                                        // Save achievements when quitting to character select
                                        achievements::save_achievements(&global_achievements)?;
                                    }
                                    game_state = None;
                                    current_screen = Screen::CharacterSelect;
                                    break 'game_loop;
                                }
                                InputResult::NeedsSave => {
                                    if !debug_mode {
                                        let _ = character_manager.save_character(&state);
                                        last_save_instant = Some(Instant::now());
                                        last_save_time = Some(Local::now());
                                    }
                                }
                                InputResult::NeedsSaveAll => {
                                    if !debug_mode {
                                        let _ = character_manager.save_character(&state);
                                        // Only save Haven if it has been discovered
                                        if haven.discovered {
                                            haven::save_haven(&haven).ok();
                                        }
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
                                character_manager.save_character(&state)?;
                                if haven.discovered {
                                    haven::save_haven(&haven).ok();
                                }
                                achievements::save_achievements(&global_achievements).ok();
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
                                &mut global_achievements,
                                debug_mode,
                                &mut rng,
                            );

                            let haven_discovered =
                                apply_tick_events(&mut state, &tick_result.events);

                            // Update visual effect lifetimes
                            let delta_time = TICK_INTERVAL_MS as f64 / 1000.0;
                            state
                                .combat_state
                                .visual_effects
                                .retain_mut(|effect| effect.update(delta_time));

                            // Persist achievements if changed
                            if tick_result.achievements_changed && !debug_mode {
                                if let Err(e) =
                                    achievements::save_achievements(&global_achievements)
                                {
                                    eprintln!("Failed to save achievements: {}", e);
                                }
                            }

                            if let Some(encounter_number) = tick_result.leviathan_encounter {
                                overlay = GameOverlay::LeviathanEncounter { encounter_number };
                            }

                            if tick_result.haven_changed && !debug_mode {
                                haven::save_haven(&haven).ok();
                            }
                            if haven_discovered {
                                overlay = GameOverlay::HavenDiscovery;
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
                            character_manager.save_character(&state)?;
                            if haven.discovered {
                                haven::save_haven(&haven)?;
                            }
                            achievements::save_achievements(&global_achievements)?;
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

/// Returns true if the active minigame requires real-time (high FPS) updates.
fn is_realtime_minigame(state: &GameState) -> bool {
    matches!(
        state.active_minigame,
        Some(challenges::ActiveMinigame::FlappyBird(_))
            | Some(challenges::ActiveMinigame::Snake(_))
    )
}
