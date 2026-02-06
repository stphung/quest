mod challenges;
mod character;
mod combat;
mod core;
mod dungeon;
mod fishing;
mod haven;
mod input;
mod items;
mod ui;
mod utils;
mod zones;

use challenges::chess::logic::process_ai_thinking;
use challenges::go::process_go_ai;
use challenges::gomoku::logic::process_ai_thinking as process_gomoku_ai;
use challenges::menu::try_discover_challenge_with_haven;
use challenges::morris::logic::process_ai_thinking as process_morris_ai;
use challenges::ActiveMinigame;
use character::input::{
    process_creation_input, process_delete_input, process_rename_input, process_select_input,
    CreationInput, CreationResult, DeleteInput, DeleteResult, RenameInput, RenameResult,
    SelectInput, SelectResult,
};
use character::manager::CharacterManager;
use character::save::SaveManager;
use chrono::{Local, Utc};
use core::constants::*;
use core::game_logic::*;
use core::game_state::*;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use input::{GameOverlay, HavenUiState, InputResult};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};
use ui::character_creation::CharacterCreationScreen;
use ui::character_delete::CharacterDeleteScreen;
use ui::character_rename::CharacterRenameScreen;
use ui::character_select::CharacterSelectScreen;
use ui::draw_ui_with_update;
use utils::updater::UpdateInfo;

enum Screen {
    CharacterSelect,
    CharacterCreation,
    CharacterDelete,
    CharacterRename,
    Game,
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

    // Check for old save file to migrate
    let old_save_manager = SaveManager::new()?;
    if old_save_manager.save_exists() {
        println!("Old save file detected. Importing as 'Imported Character'...");

        match old_save_manager.load() {
            Ok(old_state) => {
                // Save as new character
                character_manager.save_character(&old_state)?;
                println!("Import successful! Character available in character select.");
                println!("Old save file left at original location (you can delete it manually).");
            }
            Err(e) => {
                println!("Warning: Could not import old save: {}", e);
                println!("You can still create new characters.");
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
    let mut pending_haven_offline_bonus: Option<f64> = None;

    let mut haven_ui = HavenUiState::new();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Show update notification if available
    if let Ok(Some(update_info)) = update_available.join() {
        // Draw notification with changelog
        terminal.draw(|frame| {
            let area = frame.size();
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

            // Add changelog if available (max 15 entries)
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
    }

    // Main loop
    loop {
        match current_screen {
            Screen::CharacterCreation => {
                // Draw character creation screen
                terminal.draw(|f| {
                    let area = f.size();
                    creation_screen.draw(f, area);
                })?;

                // Handle input
                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key_event) = event::read()? {
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
                    let area = f.size();
                    select_screen.draw(f, area, &characters, &haven);
                    // Draw Haven management overlay if open
                    if haven_ui.showing {
                        ui::haven_scene::render_haven_tree(
                            f,
                            area,
                            &haven,
                            haven_ui.selected_room,
                            0, // No character selected, so prestige rank = 0
                        );
                    }
                })?;

                // Handle input
                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key_event) = event::read()? {
                        // Handle Haven screen (blocks other input when open)
                        if haven_ui.showing {
                            if haven_ui.confirming_build {
                                match key_event.code {
                                    KeyCode::Enter => {
                                        // Note: Can't build from character select (no active character)
                                        // Just close the confirmation
                                        haven_ui.confirming_build = false;
                                    }
                                    KeyCode::Esc => {
                                        haven_ui.confirming_build = false;
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
                            KeyCode::Char('q') | KeyCode::Char('Q') => SelectInput::Quit,
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

                                        // Process offline progression
                                        let current_time = Utc::now().timestamp();
                                        let elapsed_seconds = current_time - state.last_save_time;

                                        if elapsed_seconds > 60 {
                                            let haven_offline_bonus = haven
                                                .get_bonus(haven::HavenBonusType::OfflineXpPercent);
                                            let report = process_offline_progression(
                                                &mut state,
                                                haven_offline_bonus,
                                            );
                                            if report.xp_gained > 0 {
                                                // Enhanced combat log entries
                                                let hours = report.elapsed_seconds / 3600;
                                                let minutes = (report.elapsed_seconds % 3600) / 60;
                                                let away_str = if hours > 0 {
                                                    format!("{}h {}m", hours, minutes)
                                                } else {
                                                    format!("{}m", minutes)
                                                };
                                                state.combat_state.add_log_entry(
                                                    format!("☀️ Welcome back! ({} away)", away_str),
                                                    false, true,
                                                );
                                                state.combat_state.add_log_entry(
                                                    format!("⚔️ +{} XP gained offline", ui::game_common::format_number_short(report.xp_gained)),
                                                    false, true,
                                                );
                                                if report.total_level_ups > 0 {
                                                    state.combat_state.add_log_entry(
                                                        format!(
                                                            "📈 Leveled up {} times! ({} → {})",
                                                            report.total_level_ups,
                                                            report.level_before,
                                                            report.level_after,
                                                        ),
                                                        false, true,
                                                    );
                                                }

                                                // Store report and haven bonus for welcome overlay
                                                pending_haven_offline_bonus = Some(haven_offline_bonus);
                                                pending_offline_report = Some(report);
                                            }
                                        }

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
                    let area = f.size();
                    delete_screen.draw(f, area, selected_character);
                })?;

                // Handle input
                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key_event) = event::read()? {
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
                    let area = f.size();
                    rename_screen.draw(f, area, selected_character);
                })?;

                // Handle input
                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key_event) = event::read()? {
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
                let mut tick_counter: u32 = 0;
                let mut overlay = if let Some(report) = pending_offline_report.take() {
                    let haven_bonus = pending_haven_offline_bonus.take().unwrap_or(0.0);
                    GameOverlay::OfflineWelcome {
                        elapsed_seconds: report.elapsed_seconds,
                        xp_gained: report.xp_gained,
                        level_before: report.level_before,
                        level_after: report.level_after,
                        offline_rate_percent: report.offline_rate_percent,
                        haven_bonus_percent: haven_bonus,
                    }
                } else {
                    GameOverlay::None
                };
                let mut debug_menu = utils::debug_menu::DebugMenu::new();

                // Save indicator state (for non-debug mode)
                let mut last_save_instant: Option<Instant> = None;
                let mut last_save_time: Option<chrono::DateTime<chrono::Local>> = None;

                // Update check state - start initial background check immediately
                let mut update_info: Option<UpdateInfo> = None;
                let mut update_check_completed = false;
                let mut update_check_handle: Option<std::thread::JoinHandle<Option<UpdateInfo>>> =
                    Some(std::thread::spawn(utils::updater::check_update_info));

                loop {
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
                        draw_ui_with_update(
                            frame,
                            &state,
                            update_info.as_ref(),
                            update_check_completed,
                            haven.discovered,
                        );
                        // Draw offline welcome overlay if active
                        if let GameOverlay::OfflineWelcome {
                            elapsed_seconds,
                            xp_gained,
                            level_before,
                            level_after,
                            offline_rate_percent,
                            haven_bonus_percent,
                        } = &overlay
                        {
                            ui::game_common::render_offline_welcome(
                                frame,
                                frame.size(),
                                *elapsed_seconds,
                                *xp_gained,
                                *level_before,
                                *level_after,
                                *offline_rate_percent,
                                *haven_bonus_percent,
                            );
                        }
                        // Draw prestige confirmation overlay if active
                        if matches!(overlay, GameOverlay::PrestigeConfirm) {
                            ui::prestige_confirm::draw_prestige_confirm(frame, &state);
                        }
                        // Draw Haven discovery modal if active
                        if matches!(overlay, GameOverlay::HavenDiscovery) {
                            ui::haven_scene::render_haven_discovery_modal(frame, frame.size());
                        }
                        // Draw Vault selection screen if active
                        if let GameOverlay::VaultSelection {
                            selected_index,
                            ref selected_slots,
                        } = overlay
                        {
                            ui::haven_scene::render_vault_selection(
                                frame,
                                frame.size(),
                                &state,
                                haven.vault_tier(),
                                selected_index,
                                selected_slots,
                            );
                        }
                        // Draw Haven screen if active
                        if haven_ui.showing {
                            ui::haven_scene::render_haven_tree(
                                frame,
                                frame.size(),
                                &haven,
                                haven_ui.selected_room,
                                state.prestige_rank,
                            );
                            if haven_ui.confirming_build {
                                let room = haven::HavenRoomId::ALL[haven_ui.selected_room];
                                ui::haven_scene::render_build_confirmation(
                                    frame,
                                    frame.size(),
                                    room,
                                    &haven,
                                    state.prestige_rank,
                                );
                            }
                        }
                        // Draw debug indicator and menu if in debug mode, otherwise save indicator
                        if debug_mode {
                            ui::debug_menu_scene::render_debug_indicator(frame, frame.size());
                            if debug_menu.is_open {
                                ui::debug_menu_scene::render_debug_menu(
                                    frame,
                                    frame.size(),
                                    &debug_menu,
                                );
                            }
                        } else {
                            // Show save indicator (spinner for 1s after save, then timestamp)
                            let is_saving = last_save_instant
                                .map(|t| t.elapsed() < Duration::from_secs(1))
                                .unwrap_or(false);
                            ui::debug_menu_scene::render_save_indicator(
                                frame,
                                frame.size(),
                                is_saving,
                                last_save_time,
                            );
                        }
                    })?;

                    // Poll for input (50ms non-blocking)
                    if event::poll(Duration::from_millis(50))? {
                        if let Event::Key(key_event) = event::read()? {
                            let result = input::handle_game_input(
                                key_event,
                                &mut state,
                                &mut haven,
                                &mut haven_ui,
                                &mut overlay,
                                &mut debug_menu,
                                debug_mode,
                            );
                            match result {
                                InputResult::Continue => {}
                                InputResult::QuitToSelect => {
                                    if !debug_mode {
                                        character_manager.save_character(&state)?;
                                    }
                                    game_state = None;
                                    current_screen = Screen::CharacterSelect;
                                    break;
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
                            }
                        }
                    }

                    // Game tick every 100ms
                    if last_tick.elapsed() >= Duration::from_millis(TICK_INTERVAL_MS) {
                        game_tick(&mut state, &mut tick_counter, &haven);
                        last_tick = Instant::now();

                        // Haven discovery check (independent roll, once per tick)
                        if !haven.discovered
                            && state.prestige_rank >= 10
                            && state.active_dungeon.is_none()
                            && state.active_fishing.is_none()
                            && state.active_minigame.is_none()
                        {
                            let mut rng = rand::thread_rng();
                            if haven::try_discover_haven(&mut haven, state.prestige_rank, &mut rng)
                            {
                                if !debug_mode {
                                    haven::save_haven(&haven).ok();
                                }
                                overlay = GameOverlay::HavenDiscovery;
                            }
                        }
                    }

                    // Auto-save every 30 seconds (skip in debug mode)
                    if !debug_mode
                        && last_autosave.elapsed() >= Duration::from_secs(AUTOSAVE_INTERVAL_SECONDS)
                    {
                        character_manager.save_character(&state)?;
                        // Only save Haven if it has been discovered
                        if haven.discovered {
                            haven::save_haven(&haven)?;
                        }
                        last_autosave = Instant::now();
                        last_save_instant = Some(Instant::now());
                        last_save_time = Some(Local::now());
                    }

                    // Periodic update check (every 30 minutes)
                    // Only start a new check if we don't have one running and haven't found an update
                    if update_info.is_none()
                        && update_check_handle.is_none()
                        && last_update_check.elapsed()
                            >= Duration::from_secs(UPDATE_CHECK_INTERVAL_SECONDS)
                    {
                        update_check_handle =
                            Some(std::thread::spawn(utils::updater::check_update_info));
                        update_check_completed = false; // Reset to show "Checking..." again
                        last_update_check = Instant::now();
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

/// Processes a single game tick, updating combat and stats
fn game_tick(game_state: &mut GameState, tick_counter: &mut u32, haven: &haven::Haven) {
    use combat::logic::update_combat;
    use dungeon::logic::{
        on_boss_defeated, on_elite_defeated, on_treasure_room_entered, update_dungeon,
    };

    // Each tick is 100ms = 0.1 seconds
    let delta_time = TICK_INTERVAL_MS as f64 / 1000.0;

    // Process chess AI thinking
    // Process AI thinking for active minigame
    match &mut game_state.active_minigame {
        Some(ActiveMinigame::Chess(chess_game)) => {
            let mut rng = rand::thread_rng();
            process_ai_thinking(chess_game, &mut rng);
        }
        Some(ActiveMinigame::Morris(morris_game)) => {
            let mut rng = rand::thread_rng();
            process_morris_ai(morris_game, &mut rng);
        }
        Some(ActiveMinigame::Gomoku(gomoku_game)) => {
            let mut rng = rand::thread_rng();
            process_gomoku_ai(gomoku_game, &mut rng);
        }
        Some(ActiveMinigame::Go(go_game)) => {
            let mut rng = rand::thread_rng();
            process_go_ai(go_game, &mut rng);
        }
        _ => {}
    }

    // Try challenge discovery (single roll, weighted table picks which type)
    {
        let mut rng = rand::thread_rng();
        let haven_discovery = haven.get_bonus(haven::HavenBonusType::ChallengeDiscoveryPercent);
        if let Some(challenge_type) =
            try_discover_challenge_with_haven(game_state, &mut rng, haven_discovery)
        {
            let icon = challenge_type.icon();
            let flavor = challenge_type.discovery_flavor();
            game_state
                .combat_state
                .add_log_entry(format!("{} {}", icon, flavor), false, true);
            game_state.combat_state.add_log_entry(
                format!("{} Press [Tab] to view pending challenges", icon),
                false,
                true,
            );
        }
    }

    // Sync player max HP with derived stats (ensures equipment changes are reflected)
    let derived = character::derived_stats::DerivedStats::calculate_derived_stats(
        &game_state.attributes,
        &game_state.equipment,
    );
    game_state.combat_state.update_max_hp(derived.max_hp);

    // Update dungeon exploration if in a dungeon
    if game_state.active_dungeon.is_some() {
        let dungeon_events = update_dungeon(game_state, delta_time);
        for event in dungeon_events {
            use dungeon::logic::DungeonEvent;
            match event {
                DungeonEvent::EnteredRoom { room_type, .. } => {
                    let message = format!("🚪 Entered {:?} room", room_type);
                    game_state.combat_state.add_log_entry(message, false, true);

                    // Handle treasure room
                    if room_type == crate::dungeon::RoomType::Treasure {
                        if let Some((item, equipped)) = on_treasure_room_entered(game_state) {
                            let status = if equipped {
                                "Equipped!"
                            } else {
                                "Kept current gear"
                            };
                            let message = format!("💎 Found: {} [{}]", item.display_name, status);
                            game_state.combat_state.add_log_entry(message, false, true);
                        }
                    }
                }
                DungeonEvent::FoundKey => {
                    game_state.combat_state.add_log_entry(
                        "🗝️ Found the dungeon key!".to_string(),
                        false,
                        true,
                    );
                }
                DungeonEvent::BossUnlocked => {
                    game_state.combat_state.add_log_entry(
                        "👹 The boss room is now unlocked!".to_string(),
                        false,
                        true,
                    );
                }
                DungeonEvent::DungeonComplete {
                    xp_earned,
                    items_collected,
                } => {
                    let message = format!(
                        "🏆 Dungeon Complete! +{} XP, {} items found",
                        xp_earned, items_collected
                    );
                    game_state.combat_state.add_log_entry(message, false, true);
                }
                DungeonEvent::DungeonFailed => {
                    game_state.combat_state.add_log_entry(
                        "💀 Escaped the dungeon... (no prestige lost)".to_string(),
                        false,
                        false,
                    );
                }
                _ => {}
            }
        }
    }

    // Update fishing if active (mutually exclusive with combat)
    if game_state.active_fishing.is_some() {
        let mut rng = rand::thread_rng();
        let haven_fishing = fishing::logic::HavenFishingBonuses {
            timer_reduction_percent: haven.get_bonus(haven::HavenBonusType::FishingTimerReduction),
            double_fish_chance_percent: haven.get_bonus(haven::HavenBonusType::DoubleFishChance),
        };
        let fishing_messages =
            fishing::logic::tick_fishing_with_haven(game_state, &mut rng, &haven_fishing);
        for message in &fishing_messages {
            game_state
                .combat_state
                .add_log_entry(format!("🎣 {}", message), false, true);

            // Track fish catches and fishing item finds in recent gains
            if message.contains("Caught") {
                // Parse rarity from "[Rarity]" in message
                let rarity = if message.contains("[Legendary]") {
                    items::types::Rarity::Legendary
                } else if message.contains("[Epic]") {
                    items::types::Rarity::Epic
                } else if message.contains("[Rare]") {
                    items::types::Rarity::Rare
                } else if message.contains("[Uncommon]") {
                    items::types::Rarity::Magic
                } else {
                    items::types::Rarity::Common
                };
                // Extract fish name (between "Caught " and " [")
                let fish_name = message
                    .split("Caught ")
                    .nth(1)
                    .and_then(|s| s.split(" [").next())
                    .unwrap_or("Fish")
                    .to_string();
                game_state.add_recent_drop(fish_name, rarity, false, "🐟", String::new(), String::new());
            } else if message.contains("Found item:") {
                let item_name = message
                    .split("Found item: ")
                    .nth(1)
                    .map(|s| s.trim_end_matches('!'))
                    .unwrap_or("Item")
                    .to_string();
                game_state.add_recent_drop(item_name, items::types::Rarity::Rare, false, "📦", String::new(), String::new());
            }
        }

        // Check for fishing rank up
        if let Some(rank_msg) = fishing::logic::check_rank_up(&mut game_state.fishing) {
            game_state
                .combat_state
                .add_log_entry(format!("🎣 {}", rank_msg), false, true);
        }

        // Update play_time_seconds and last_save_time (still needed while fishing)
        *tick_counter += 1;
        if *tick_counter >= 10 {
            game_state.play_time_seconds += 1;
            *tick_counter = 0;
        }
        game_state.last_save_time = Utc::now().timestamp();

        return; // Skip combat processing while fishing
    }

    // Build Haven combat bonuses
    let haven_combat = combat::logic::HavenCombatBonuses {
        hp_regen_percent: haven.get_bonus(haven::HavenBonusType::HpRegenPercent),
        hp_regen_delay_reduction: haven.get_bonus(haven::HavenBonusType::HpRegenDelayReduction),
        damage_percent: haven.get_bonus(haven::HavenBonusType::DamagePercent),
        crit_chance_percent: haven.get_bonus(haven::HavenBonusType::CritChancePercent),
        double_strike_chance: haven.get_bonus(haven::HavenBonusType::DoubleStrikeChance),
        xp_gain_percent: haven.get_bonus(haven::HavenBonusType::XpGainPercent),
    };
    let combat_events = update_combat(game_state, delta_time, &haven_combat);

    // Process combat events
    for event in combat_events {
        use combat::logic::CombatEvent;
        match event {
            CombatEvent::PlayerAttackBlocked { weapon_needed } => {
                // Attack blocked - boss requires legendary weapon
                let message = format!("🚫 {} required to damage this foe!", weapon_needed);
                game_state.combat_state.add_log_entry(message, false, true);
            }
            CombatEvent::PlayerAttack { damage, was_crit } => {
                // Add to combat log
                let message = if was_crit {
                    format!("💥 CRITICAL HIT for {} damage!", damage)
                } else {
                    format!("⚔ You hit for {} damage", damage)
                };
                game_state
                    .combat_state
                    .add_log_entry(message, was_crit, true);

                // Spawn damage number effect
                let damage_effect = ui::combat_effects::VisualEffect::new(
                    ui::combat_effects::EffectType::DamageNumber {
                        value: damage,
                        is_crit: was_crit,
                    },
                    0.8,
                );
                game_state.combat_state.visual_effects.push(damage_effect);

                // Spawn attack flash
                let flash_effect = ui::combat_effects::VisualEffect::new(
                    ui::combat_effects::EffectType::AttackFlash,
                    0.2,
                );
                game_state.combat_state.visual_effects.push(flash_effect);

                // Spawn impact effect
                let impact_effect = ui::combat_effects::VisualEffect::new(
                    ui::combat_effects::EffectType::HitImpact,
                    0.3,
                );
                game_state.combat_state.visual_effects.push(impact_effect);
            }
            CombatEvent::EnemyAttack { damage } => {
                // Add enemy attack to combat log
                if let Some(enemy) = &game_state.combat_state.current_enemy {
                    let message = format!("🛡 {} hits you for {} damage", enemy.name, damage);
                    game_state.combat_state.add_log_entry(message, false, false);
                }
            }
            CombatEvent::EnemyDied { xp_gained } => {
                // Add to combat log
                if let Some(enemy) = &game_state.combat_state.current_enemy {
                    let message = format!("✨ {} defeated! +{} XP", enemy.name, xp_gained);
                    game_state.combat_state.add_log_entry(message, false, true);
                }
                apply_tick_xp(game_state, xp_gained as f64);
                game_state.session_kills += 1;

                // Track XP in dungeon if active and mark room cleared
                dungeon::logic::add_dungeon_xp(game_state, xp_gained);
                if let Some(dungeon) = &mut game_state.active_dungeon {
                    dungeon::logic::on_room_enemy_defeated(dungeon);
                }

                // Try to drop item
                use items::drops::try_drop_item_with_haven;
                use items::scoring::auto_equip_if_better;

                let haven_drop_rate = haven.get_bonus(haven::HavenBonusType::DropRatePercent);
                let haven_rarity = haven.get_bonus(haven::HavenBonusType::ItemRarityPercent);
                if let Some(item) =
                    try_drop_item_with_haven(game_state, haven_drop_rate, haven_rarity)
                {
                    let item_name = item.display_name.clone();
                    let rarity = item.rarity;
                    let slot = item.slot_name().to_string();
                    let stats = item.stat_summary();
                    let equipped = auto_equip_if_better(item, game_state);
                    game_state.add_recent_drop(item_name, rarity, equipped, "🎁", slot, stats);
                }

                // Try to discover dungeon (only when not in a dungeon)
                let discovered_dungeon =
                    game_state.active_dungeon.is_none() && try_discover_dungeon(game_state);
                if discovered_dungeon {
                    game_state.combat_state.add_log_entry(
                        "🌀 You notice a dark passage leading underground...".to_string(),
                        false,
                        true,
                    );
                }

                // Try to discover fishing spot (only when not in dungeon and not already fishing)
                if !discovered_dungeon
                    && game_state.active_dungeon.is_none()
                    && game_state.active_fishing.is_none()
                {
                    let mut rng = rand::thread_rng();
                    if let Some(message) =
                        fishing::logic::try_discover_fishing(game_state, &mut rng)
                    {
                        game_state.combat_state.add_log_entry(
                            format!("🎣 {}", message),
                            false,
                            true,
                        );
                    }
                }
            }
            CombatEvent::EliteDefeated { xp_gained } => {
                // Elite defeated - give key
                if let Some(enemy) = &game_state.combat_state.current_enemy {
                    let message = format!("⚔️ {} defeated! +{} XP", enemy.name, xp_gained);
                    game_state.combat_state.add_log_entry(message, false, true);
                }
                apply_tick_xp(game_state, xp_gained as f64);
                dungeon::logic::add_dungeon_xp(game_state, xp_gained);

                // Give key
                if let Some(dungeon) = &mut game_state.active_dungeon {
                    let events = on_elite_defeated(dungeon);
                    for event in events {
                        if matches!(event, dungeon::logic::DungeonEvent::FoundKey) {
                            game_state.combat_state.add_log_entry(
                                "🗝️ Found the dungeon key!".to_string(),
                                false,
                                true,
                            );
                        }
                    }
                }
            }
            CombatEvent::BossDefeated { xp_gained } => {
                // Boss defeated - complete dungeon
                if let Some(enemy) = &game_state.combat_state.current_enemy {
                    let message = format!("👑 {} vanquished! +{} XP", enemy.name, xp_gained);
                    game_state.combat_state.add_log_entry(message, false, true);
                }
                apply_tick_xp(game_state, xp_gained as f64);

                // Calculate boss bonus XP (copy values before mutable borrow)
                let (bonus_xp, total_xp, items) = if let Some(dungeon) = &game_state.active_dungeon
                {
                    let bonus = dungeon::logic::calculate_boss_xp_reward(dungeon.size);
                    let total = dungeon.xp_earned + xp_gained + bonus;
                    let item_count = dungeon.collected_items.len();
                    (bonus, total, item_count)
                } else {
                    (0, xp_gained, 0)
                };

                apply_tick_xp(game_state, bonus_xp as f64);

                let message = format!(
                    "🏆 Dungeon Complete! +{} bonus XP ({} total, {} items)",
                    bonus_xp, total_xp, items
                );
                game_state.combat_state.add_log_entry(message, false, true);

                // Clear dungeon
                let _events = on_boss_defeated(game_state);
            }
            CombatEvent::PlayerDiedInDungeon => {
                // Died in dungeon - exit without prestige loss
                game_state.combat_state.add_log_entry(
                    "💀 You fell in the dungeon... (escaped without prestige loss)".to_string(),
                    false,
                    false,
                );
            }
            CombatEvent::PlayerDied => {
                // Add to combat log
                game_state.combat_state.add_log_entry(
                    "💀 You died! Boss encounter reset.".to_string(),
                    false,
                    false,
                );
            }
            CombatEvent::SubzoneBossDefeated { xp_gained, result } => {
                use zones::BossDefeatResult;
                // Apply XP from boss kill
                apply_tick_xp(game_state, xp_gained as f64);
                game_state.session_kills += 1;

                // Log based on result
                let message = match &result {
                    BossDefeatResult::SubzoneComplete { .. } => {
                        format!("👑 Boss defeated! +{} XP — Moving to next area.", xp_gained)
                    }
                    BossDefeatResult::ZoneComplete {
                        old_zone,
                        new_zone_id,
                    } => {
                        let new_zone = zones::get_zone(*new_zone_id)
                            .map(|z| z.name)
                            .unwrap_or("???");
                        format!(
                            "👑 {} conquered! +{} XP — Advancing to {}!",
                            old_zone, xp_gained, new_zone
                        )
                    }
                    BossDefeatResult::ZoneCompleteButGated {
                        zone_name,
                        required_prestige,
                    } => {
                        format!(
                            "👑 {} conquered! +{} XP — Next zone requires Prestige {}.",
                            zone_name, xp_gained, required_prestige
                        )
                    }
                    BossDefeatResult::GameComplete => {
                        format!(
                            "👑 All zones conquered! +{} XP — You have completed the game!",
                            xp_gained
                        )
                    }
                    BossDefeatResult::WeaponRequired { .. } => {
                        // Already handled by PlayerAttackBlocked
                        continue;
                    }
                };
                game_state.combat_state.add_log_entry(message, false, true);
            }
            _ => {}
        }
    }

    // Update visual effects
    let delta_time = TICK_INTERVAL_MS as f64 / 1000.0;
    game_state
        .combat_state
        .visual_effects
        .retain_mut(|effect| effect.update(delta_time));

    // Spawn enemy if needed
    spawn_enemy_if_needed(game_state);

    // Update play_time_seconds
    // Each tick is 100ms (TICK_INTERVAL_MS), so 10 ticks = 1 second
    *tick_counter += 1;
    if *tick_counter >= 10 {
        game_state.play_time_seconds += 1;
        *tick_counter = 0;
    }

    // Update last_save_time to current time for tracking
    game_state.last_save_time = Utc::now().timestamp();
}
