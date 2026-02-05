mod challenges;
mod character;
mod combat;
mod core;
mod dungeon;
mod fishing;
mod items;
mod ui;
mod utils;
mod zones;

use challenges::chess::logic::{
    apply_game_result, process_ai_thinking, process_input as process_chess_input, ChessInput,
};
use challenges::gomoku::logic::{
    apply_game_result as apply_gomoku_result, process_ai_thinking as process_gomoku_ai,
    process_input as process_gomoku_input, GomokuInput,
};
use challenges::menu::{process_input as process_menu_input, try_discover_challenge, MenuInput};
use challenges::minesweeper::logic::{
    apply_game_result as apply_minesweeper_result, process_input as process_minesweeper_input,
    MinesweeperInput,
};
use challenges::morris::logic::{
    apply_game_result as apply_morris_result, process_ai_thinking as process_morris_ai,
    process_input as process_morris_input, MorrisInput,
};
use challenges::rune::logic::{
    apply_game_result as apply_rune_result, process_input as process_rune_input, RuneInput,
};
use character::input::{
    process_creation_input, process_delete_input, process_rename_input, process_select_input,
    CreationInput, CreationResult, DeleteInput, DeleteResult, RenameInput, RenameResult,
    SelectInput, SelectResult,
};
use character::manager::CharacterManager;
use character::prestige::*;
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

                // Draw character select screen
                terminal.draw(|f| {
                    let area = f.size();
                    select_screen.draw(f, area, &characters);
                })?;

                // Handle input
                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key_event) = event::read()? {
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
                                            let report = process_offline_progression(&mut state);
                                            // Always show offline progress in combat log
                                            if report.xp_gained > 0 {
                                                let message = if report.total_level_ups > 0 {
                                                    format!(
                                                        "Offline: +{} XP, +{} levels",
                                                        report.xp_gained, report.total_level_ups
                                                    )
                                                } else {
                                                    format!("Offline: +{} XP", report.xp_gained)
                                                };
                                                state
                                                    .combat_state
                                                    .add_log_entry(message, false, true);
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
                let mut showing_prestige_confirm = false;
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
                        );
                        // Draw prestige confirmation overlay if active
                        if showing_prestige_confirm {
                            ui::prestige_confirm::draw_prestige_confirm(frame, &state);
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
                            // Handle prestige confirmation dialog
                            if showing_prestige_confirm {
                                match key_event.code {
                                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                                        perform_prestige(&mut state);
                                        showing_prestige_confirm = false;
                                        // Save immediately after prestige to prevent stale enemy on reload
                                        if !debug_mode {
                                            let _ = character_manager.save_character(&state);
                                            last_save_instant = Some(Instant::now());
                                            last_save_time = Some(Local::now());
                                        }
                                        state.combat_state.add_log_entry(
                                            format!(
                                                "Prestiged to {}!",
                                                get_prestige_tier(state.prestige_rank).name
                                            ),
                                            false,
                                            true,
                                        );
                                    }
                                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                        showing_prestige_confirm = false;
                                    }
                                    _ => {}
                                }
                                continue;
                            }

                            // Handle debug menu (if debug mode enabled)
                            if debug_mode {
                                // Backtick toggles debug menu
                                if key_event.code == KeyCode::Char('`') {
                                    debug_menu.toggle();
                                    continue;
                                }

                                // Handle debug menu navigation when open
                                if debug_menu.is_open {
                                    match key_event.code {
                                        KeyCode::Up => debug_menu.navigate_up(),
                                        KeyCode::Down => debug_menu.navigate_down(),
                                        KeyCode::Enter => {
                                            let msg = debug_menu.trigger_selected(&mut state);
                                            state.combat_state.add_log_entry(
                                                format!("[DEBUG] {}", msg),
                                                false,
                                                true,
                                            );
                                        }
                                        KeyCode::Esc => debug_menu.close(),
                                        _ => {}
                                    }
                                    continue;
                                }
                            }

                            // Handle active rune game input
                            if let Some(ref mut rune_game) = state.active_rune {
                                if rune_game.game_result.is_some() {
                                    // Any key dismisses result and applies rewards
                                    apply_rune_result(&mut state);
                                    continue;
                                }

                                let input = match key_event.code {
                                    KeyCode::Left => RuneInput::Left,
                                    KeyCode::Right => RuneInput::Right,
                                    KeyCode::Up => RuneInput::Up,
                                    KeyCode::Down => RuneInput::Down,
                                    KeyCode::Enter => RuneInput::Submit,
                                    KeyCode::Char('f') | KeyCode::Char('F') => {
                                        RuneInput::ClearGuess
                                    }
                                    KeyCode::Esc => RuneInput::Forfeit,
                                    _ => RuneInput::Other,
                                };
                                let mut rng = rand::thread_rng();
                                process_rune_input(rune_game, input, &mut rng);
                                continue;
                            }

                            // Handle active minesweeper game input
                            if let Some(ref mut minesweeper_game) = state.active_minesweeper {
                                if minesweeper_game.game_result.is_some() {
                                    // Any key dismisses result and applies rewards
                                    apply_minesweeper_result(&mut state);
                                    continue;
                                }

                                let input = match key_event.code {
                                    KeyCode::Up => MinesweeperInput::Up,
                                    KeyCode::Down => MinesweeperInput::Down,
                                    KeyCode::Left => MinesweeperInput::Left,
                                    KeyCode::Right => MinesweeperInput::Right,
                                    KeyCode::Enter => MinesweeperInput::Reveal,
                                    KeyCode::Char('f') | KeyCode::Char('F') => {
                                        MinesweeperInput::ToggleFlag
                                    }
                                    KeyCode::Esc => MinesweeperInput::Forfeit,
                                    _ => MinesweeperInput::Other,
                                };
                                let mut rng = rand::thread_rng();
                                process_minesweeper_input(minesweeper_game, input, &mut rng);
                                continue;
                            }

                            // Handle active Gomoku game input
                            if let Some(ref mut gomoku_game) = state.active_gomoku {
                                if gomoku_game.game_result.is_some() {
                                    // Any key dismisses result and applies rewards
                                    apply_gomoku_result(&mut state);
                                    continue;
                                }

                                let input = match key_event.code {
                                    KeyCode::Up => GomokuInput::Up,
                                    KeyCode::Down => GomokuInput::Down,
                                    KeyCode::Left => GomokuInput::Left,
                                    KeyCode::Right => GomokuInput::Right,
                                    KeyCode::Enter => GomokuInput::PlaceStone,
                                    KeyCode::Esc => GomokuInput::Forfeit,
                                    _ => GomokuInput::Other,
                                };
                                process_gomoku_input(gomoku_game, input);
                                continue;
                            }

                            // Handle active chess game input (highest priority)
                            if let Some(ref mut chess_game) = state.active_chess {
                                if chess_game.game_result.is_some() {
                                    // Any key dismisses result and applies rewards
                                    apply_game_result(&mut state);
                                    continue;
                                }
                                let input = match key_event.code {
                                    KeyCode::Up => ChessInput::Up,
                                    KeyCode::Down => ChessInput::Down,
                                    KeyCode::Left => ChessInput::Left,
                                    KeyCode::Right => ChessInput::Right,
                                    KeyCode::Enter => ChessInput::Select,
                                    KeyCode::Esc => ChessInput::Cancel,
                                    _ => ChessInput::Other,
                                };
                                process_chess_input(chess_game, input);
                                continue;
                            }

                            // Handle active Morris game input
                            if let Some(ref mut morris_game) = state.active_morris {
                                if morris_game.game_result.is_some() {
                                    // Any key dismisses result and applies rewards
                                    apply_morris_result(&mut state);
                                    continue;
                                }
                                let input = match key_event.code {
                                    KeyCode::Up => MorrisInput::Up,
                                    KeyCode::Down => MorrisInput::Down,
                                    KeyCode::Left => MorrisInput::Left,
                                    KeyCode::Right => MorrisInput::Right,
                                    KeyCode::Enter => MorrisInput::Select,
                                    KeyCode::Esc => MorrisInput::Cancel,
                                    _ => MorrisInput::Other,
                                };
                                process_morris_input(morris_game, input);
                                continue;
                            }

                            // Handle challenge menu input
                            if state.challenge_menu.is_open {
                                let input = match key_event.code {
                                    KeyCode::Up => MenuInput::Up,
                                    KeyCode::Down => MenuInput::Down,
                                    KeyCode::Enter => MenuInput::Select,
                                    KeyCode::Char('d') | KeyCode::Char('D') => MenuInput::Decline,
                                    KeyCode::Esc | KeyCode::Tab => MenuInput::Cancel,
                                    _ => MenuInput::Other,
                                };
                                process_menu_input(&mut state, input);
                                continue;
                            }

                            // Tab to open challenge menu
                            if key_event.code == KeyCode::Tab
                                && !state.challenge_menu.challenges.is_empty()
                            {
                                state.challenge_menu.open();
                                continue;
                            }

                            match key_event.code {
                                // Handle 'q'/'Q' to quit
                                KeyCode::Char('q') | KeyCode::Char('Q') => {
                                    // Save character before returning to select (skip in debug mode)
                                    if !debug_mode {
                                        character_manager.save_character(&state)?;
                                    }
                                    game_state = None;
                                    current_screen = Screen::CharacterSelect;
                                    break;
                                }
                                // Handle 'p'/'P' to show prestige confirmation
                                KeyCode::Char('p') | KeyCode::Char('P') => {
                                    if can_prestige(&state) {
                                        showing_prestige_confirm = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    // Game tick every 100ms
                    if last_tick.elapsed() >= Duration::from_millis(TICK_INTERVAL_MS) {
                        game_tick(&mut state, &mut tick_counter);
                        last_tick = Instant::now();
                    }

                    // Auto-save every 30 seconds (skip in debug mode)
                    if !debug_mode
                        && last_autosave.elapsed() >= Duration::from_secs(AUTOSAVE_INTERVAL_SECONDS)
                    {
                        character_manager.save_character(&state)?;
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
fn game_tick(game_state: &mut GameState, tick_counter: &mut u32) {
    use combat::logic::update_combat;
    use dungeon::logic::{
        on_boss_defeated, on_elite_defeated, on_treasure_room_entered, update_dungeon,
    };
    use fishing::logic::tick_fishing;

    // Each tick is 100ms = 0.1 seconds
    let delta_time = TICK_INTERVAL_MS as f64 / 1000.0;

    // Process chess AI thinking
    if let Some(ref mut chess_game) = game_state.active_chess {
        let mut rng = rand::thread_rng();
        process_ai_thinking(chess_game, &mut rng);
    }

    // Process Morris AI thinking
    if let Some(ref mut morris_game) = game_state.active_morris {
        let mut rng = rand::thread_rng();
        process_morris_ai(morris_game, &mut rng);
    }

    // Process Gomoku AI thinking
    if let Some(ref mut gomoku_game) = game_state.active_gomoku {
        let mut rng = rand::thread_rng();
        process_gomoku_ai(gomoku_game, &mut rng);
    }

    // Try challenge discovery (single roll, weighted table picks which type)
    {
        let mut rng = rand::thread_rng();
        if let Some(challenge_type) = try_discover_challenge(game_state, &mut rng) {
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
        let fishing_messages = tick_fishing(game_state, &mut rng);
        for message in fishing_messages {
            game_state
                .combat_state
                .add_log_entry(format!("🎣 {}", message), false, true);
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

    // Update combat state
    let combat_events = update_combat(game_state, delta_time);

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

                // Track XP in dungeon if active and mark room cleared
                dungeon::logic::add_dungeon_xp(game_state, xp_gained);
                if let Some(dungeon) = &mut game_state.active_dungeon {
                    dungeon::logic::on_room_enemy_defeated(dungeon);
                }

                // Try to drop item
                use items::drops::try_drop_item;
                use items::scoring::auto_equip_if_better;

                if let Some(item) = try_drop_item(game_state) {
                    let item_name = item.display_name.clone();
                    let rarity = item.rarity;
                    let equipped = auto_equip_if_better(item, game_state);
                    let stars = "⭐".repeat(rarity as usize + 1);
                    let equipped_text = if equipped { " (equipped!)" } else { "" };
                    let message = format!(
                        "🎁 Found: {} [{}] {}{}",
                        item_name,
                        rarity.name(),
                        stars,
                        equipped_text
                    );
                    game_state.combat_state.add_log_entry(message, false, true);
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
