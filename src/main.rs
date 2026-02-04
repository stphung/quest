mod attributes;
mod build_info;
mod challenge_menu;
mod character_manager;
mod chess;
mod chess_logic;
mod combat;
mod combat_logic;
mod constants;
mod debug_menu;
mod derived_stats;
mod dungeon;
mod dungeon_generation;
mod dungeon_logic;
mod equipment;
mod fishing;
mod fishing_generation;
mod fishing_logic;
mod game_logic;
mod game_state;
mod gomoku;
mod gomoku_logic;
mod item_drops;
mod item_generation;
mod item_names;
mod item_scoring;
mod items;
mod morris;
mod morris_logic;
mod prestige;
mod save_manager;
mod ui;
mod updater;
mod zones;

use challenge_menu::{try_discover_challenge, ChallengeType};
use character_manager::CharacterManager;
use chess::ChessDifficulty;
use chess_logic::{apply_game_result, process_ai_thinking, start_chess_game};
use chrono::Utc;
use constants::*;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use game_logic::*;
use game_state::*;
use gomoku::GomokuDifficulty;
use gomoku_logic::start_gomoku_game;
use morris::{
    CursorDirection as MorrisCursorDirection, MorrisDifficulty, MorrisMove, MorrisResult,
};
use morris_logic::{
    apply_game_result as apply_morris_result, get_legal_moves as get_morris_legal_moves,
    process_ai_thinking as process_morris_ai, start_morris_game,
};
use prestige::*;
use ratatui::{backend::CrosstermBackend, Terminal};
use save_manager::SaveManager;
use std::io;
use std::time::{Duration, Instant};
use ui::character_creation::CharacterCreationScreen;
use ui::character_delete::CharacterDeleteScreen;
use ui::character_rename::CharacterRenameScreen;
use ui::character_select::CharacterSelectScreen;
use ui::draw_ui_with_update;
use updater::UpdateInfo;

enum Screen {
    CharacterSelect,
    CharacterCreation,
    CharacterDelete,
    CharacterRename,
    Game,
}

/// Handle Enter key press during Morris game
fn handle_morris_enter(state: &mut GameState) {
    let morris_game = match state.active_morris.as_mut() {
        Some(game) => game,
        None => return,
    };

    let cursor = morris_game.cursor;

    // If must capture, try to capture at cursor
    if morris_game.must_capture {
        let capture_moves = get_morris_legal_moves(morris_game);
        if capture_moves
            .iter()
            .any(|m| matches!(m, MorrisMove::Capture(pos) if *pos == cursor))
        {
            morris_logic::apply_move(morris_game, MorrisMove::Capture(cursor));
        }
        return;
    }

    // During placing phase, place at cursor if empty
    if morris_game.phase == morris::MorrisPhase::Placing {
        if morris_game.board[cursor].is_none() {
            morris_logic::apply_move(morris_game, MorrisMove::Place(cursor));
        }
        return;
    }

    // During moving/flying phase
    if let Some(selected) = morris_game.selected_position {
        // Already selected a piece - try to move to cursor
        let legal_moves = get_morris_legal_moves(morris_game);
        if legal_moves.iter().any(
            |m| matches!(m, MorrisMove::Move { from, to } if *from == selected && *to == cursor),
        ) {
            morris_logic::apply_move(
                morris_game,
                MorrisMove::Move {
                    from: selected,
                    to: cursor,
                },
            );
        } else if morris_game.board[cursor] == Some(morris::Player::Human) {
            // Clicked on another human piece - select it instead
            morris_game.selected_position = Some(cursor);
        } else {
            // Invalid move - clear selection
            morris_game.clear_selection();
        }
    } else {
        // No piece selected - try to select piece at cursor
        if morris_game.board[cursor] == Some(morris::Player::Human) {
            morris_game.selected_position = Some(cursor);
        }
    }
}

fn main() -> io::Result<()> {
    // Handle CLI arguments
    let args: Vec<String> = std::env::args().collect();
    let mut debug_mode = false;

    if args.len() > 1 {
        match args[1].as_str() {
            "update" => match updater::run_update_command() {
                Ok(_) => std::process::exit(0),
                Err(_) => std::process::exit(1),
            },
            "--version" | "-v" => {
                println!(
                    "quest {} ({})",
                    build_info::BUILD_DATE,
                    build_info::BUILD_COMMIT
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
    let update_available = std::thread::spawn(updater::check_update_info);

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
                        match key_event.code {
                            KeyCode::Char(c) => {
                                creation_screen.handle_char_input(c);
                            }
                            KeyCode::Backspace => {
                                creation_screen.handle_backspace();
                            }
                            KeyCode::Enter => {
                                // Validate and create character
                                if creation_screen.is_valid() {
                                    let new_name = creation_screen.get_name();
                                    let new_state =
                                        GameState::new(new_name, Utc::now().timestamp());
                                    if let Err(e) = character_manager.save_character(&new_state) {
                                        creation_screen.validation_error =
                                            Some(format!("Save failed: {}", e));
                                    } else {
                                        // Reset creation screen and go to select
                                        creation_screen = CharacterCreationScreen::new();
                                        select_screen = CharacterSelectScreen::new();
                                        current_screen = Screen::CharacterSelect;
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                // Cancel - go to select if characters exist, else stay
                                let chars = character_manager.list_characters()?;
                                if !chars.is_empty() {
                                    creation_screen = CharacterCreationScreen::new();
                                    current_screen = Screen::CharacterSelect;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            Screen::CharacterSelect => {
                // Refresh character list
                let characters = character_manager.list_characters()?;

                // If no characters, go to creation
                if characters.is_empty() {
                    current_screen = Screen::CharacterCreation;
                    continue;
                }

                // Clamp selected index
                if select_screen.selected_index >= characters.len() {
                    select_screen.selected_index = characters.len().saturating_sub(1);
                }

                // Draw character select screen
                terminal.draw(|f| {
                    let area = f.size();
                    select_screen.draw(f, area, &characters);
                })?;

                // Handle input
                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key_event) = event::read()? {
                        match key_event.code {
                            KeyCode::Up => {
                                select_screen.selected_index =
                                    select_screen.selected_index.saturating_sub(1);
                            }
                            KeyCode::Down => {
                                if select_screen.selected_index + 1 < characters.len() {
                                    select_screen.selected_index += 1;
                                }
                            }
                            KeyCode::Enter => {
                                // Load selected character and start game
                                let selected = &characters[select_screen.selected_index];
                                if !selected.is_corrupted {
                                    match character_manager.load_character(&selected.filename) {
                                        Ok(mut state) => {
                                            // Sanity check: clear stale enemy if HP is impossibly high
                                            // (can happen if save was from before prestige reset)
                                            let derived = derived_stats::DerivedStats::calculate_derived_stats(
                                                &state.attributes,
                                                &state.equipment,
                                            );
                                            if let Some(enemy) = &state.combat_state.current_enemy {
                                                // Max possible enemy HP is 2.4x player HP (boss with max variance)
                                                // If enemy HP is > 2.5x, it's stale from before a stat reset
                                                if enemy.max_hp
                                                    > (derived.max_hp as f64 * 2.5) as u32
                                                {
                                                    state.combat_state.current_enemy = None;
                                                }
                                            }

                                            // Process offline progression
                                            let current_time = Utc::now().timestamp();
                                            let elapsed_seconds =
                                                current_time - state.last_save_time;

                                            if elapsed_seconds > 60 {
                                                let report =
                                                    process_offline_progression(&mut state);
                                                // Always show offline progress in combat log
                                                if report.xp_gained > 0 {
                                                    let message = if report.total_level_ups > 0 {
                                                        format!(
                                                            "Offline: +{} XP, +{} levels",
                                                            report.xp_gained,
                                                            report.total_level_ups
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
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                // New character
                                creation_screen = CharacterCreationScreen::new();
                                current_screen = Screen::CharacterCreation;
                            }
                            KeyCode::Char('d') | KeyCode::Char('D') => {
                                // Delete character
                                let selected = &characters[select_screen.selected_index];
                                if !selected.is_corrupted {
                                    delete_screen = CharacterDeleteScreen::new();
                                    current_screen = Screen::CharacterDelete;
                                }
                            }
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                // Rename character
                                let selected = &characters[select_screen.selected_index];
                                if !selected.is_corrupted {
                                    rename_screen = CharacterRenameScreen::new();
                                    current_screen = Screen::CharacterRename;
                                }
                            }
                            KeyCode::Char('q') | KeyCode::Char('Q') => {
                                // Quit
                                break;
                            }
                            _ => {}
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
                        match key_event.code {
                            KeyCode::Char(c) => {
                                delete_screen.handle_char_input(c);
                            }
                            KeyCode::Backspace => {
                                delete_screen.handle_backspace();
                            }
                            KeyCode::Enter => {
                                // Check if confirmation matches
                                if delete_screen.is_confirmed(&selected_character.character_name) {
                                    if let Err(e) = character_manager
                                        .delete_character(&selected_character.filename)
                                    {
                                        eprintln!("Failed to delete character: {}", e);
                                    }
                                    delete_screen = CharacterDeleteScreen::new();
                                    select_screen.selected_index = 0;
                                    current_screen = Screen::CharacterSelect;
                                }
                            }
                            KeyCode::Esc => {
                                // Cancel
                                delete_screen = CharacterDeleteScreen::new();
                                current_screen = Screen::CharacterSelect;
                            }
                            _ => {}
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
                        match key_event.code {
                            KeyCode::Char(c) => {
                                rename_screen.handle_char_input(c);
                            }
                            KeyCode::Backspace => {
                                rename_screen.handle_backspace();
                            }
                            KeyCode::Enter => {
                                // Validate and rename
                                if rename_screen.is_valid() {
                                    let new_name = rename_screen.get_name();
                                    if let Err(e) = character_manager
                                        .rename_character(&selected_character.filename, new_name)
                                    {
                                        rename_screen.validation_error =
                                            Some(format!("Rename failed: {}", e));
                                    } else {
                                        rename_screen = CharacterRenameScreen::new();
                                        current_screen = Screen::CharacterSelect;
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                // Cancel
                                rename_screen = CharacterRenameScreen::new();
                                current_screen = Screen::CharacterSelect;
                            }
                            _ => {}
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
                let mut debug_menu = debug_menu::DebugMenu::new();

                // Update check state - start initial background check immediately
                let mut update_info: Option<UpdateInfo> = None;
                let mut update_check_completed = false;
                let mut update_check_handle: Option<std::thread::JoinHandle<Option<UpdateInfo>>> =
                    Some(std::thread::spawn(updater::check_update_info));

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
                        // Draw debug indicator and menu if in debug mode
                        if debug_mode {
                            ui::debug_menu_scene::render_debug_indicator(frame, frame.size());
                            if debug_menu.is_open {
                                ui::debug_menu_scene::render_debug_menu(
                                    frame,
                                    frame.size(),
                                    &debug_menu,
                                );
                            }
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
                                        let _ = character_manager.save_character(&state);
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

                            // Handle active chess game input (highest priority)
                            if let Some(ref mut chess_game) = state.active_chess {
                                if chess_game.game_result.is_some() {
                                    // Any key dismisses result and adds combat log message
                                    let old_prestige = state.prestige_rank;
                                    if let Some((result, prestige_gained)) =
                                        apply_game_result(&mut state)
                                    {
                                        use chess::ChessResult;
                                        match result {
                                            ChessResult::Win => {
                                                let new_prestige = old_prestige + prestige_gained;
                                                state.combat_state.add_log_entry(
                                                    "♟ Checkmate! You defeated the mysterious figure.".to_string(),
                                                    false,
                                                    true,
                                                );
                                                state.combat_state.add_log_entry(
                                                    format!(
                                                        "♟ +{} Prestige Ranks (P{} → P{})",
                                                        prestige_gained, old_prestige, new_prestige
                                                    ),
                                                    false,
                                                    true,
                                                );
                                            }
                                            ChessResult::Loss => {
                                                state.combat_state.add_log_entry(
                                                    "♟ The mysterious figure nods respectfully and vanishes.".to_string(),
                                                    false,
                                                    true,
                                                );
                                            }
                                            ChessResult::Draw => {
                                                state.combat_state.add_log_entry(
                                                    "♟ The figure smiles knowingly and fades away."
                                                        .to_string(),
                                                    false,
                                                    true,
                                                );
                                            }
                                            ChessResult::Forfeit => {
                                                state.combat_state.add_log_entry(
                                                    "♟ You concede the game. The figure disappears without a word.".to_string(),
                                                    false,
                                                    true,
                                                );
                                            }
                                        }
                                    }
                                    continue;
                                }
                                if !chess_game.ai_thinking {
                                    match key_event.code {
                                        KeyCode::Up => chess_game.move_cursor(0, 1),
                                        KeyCode::Down => chess_game.move_cursor(0, -1),
                                        KeyCode::Left => chess_game.move_cursor(-1, 0),
                                        KeyCode::Right => chess_game.move_cursor(1, 0),
                                        KeyCode::Enter => {
                                            // If a piece is selected, try to move to cursor
                                            if chess_game.selected_square.is_some() {
                                                // Check if cursor is on a legal destination
                                                if chess_game
                                                    .legal_move_destinations
                                                    .contains(&chess_game.cursor)
                                                {
                                                    chess_game.try_move_to_cursor();
                                                } else if chess_game.cursor_on_player_piece() {
                                                    // Cursor on another player piece - select it instead
                                                    chess_game.select_piece_at_cursor();
                                                } else {
                                                    // Invalid destination - clear selection
                                                    chess_game.clear_selection();
                                                }
                                            } else {
                                                // No piece selected - try to select piece at cursor
                                                chess_game.select_piece_at_cursor();
                                            }
                                        }
                                        KeyCode::Esc => {
                                            if chess_game.forfeit_pending {
                                                chess_game.game_result =
                                                    Some(chess::ChessResult::Forfeit);
                                            } else if chess_game.selected_square.is_some() {
                                                chess_game.clear_selection();
                                                chess_game.forfeit_pending = false;
                                            } else {
                                                chess_game.forfeit_pending = true;
                                            }
                                        }
                                        _ => {
                                            chess_game.forfeit_pending = false;
                                        }
                                    }
                                }
                                continue;
                            }

                            // Handle active Morris game input
                            if let Some(ref mut morris_game) = state.active_morris {
                                if morris_game.game_result.is_some() {
                                    // Any key dismisses result and adds combat log message
                                    if let Some((result, xp_gained, fishing_rank_up)) =
                                        apply_morris_result(&mut state)
                                    {
                                        match result {
                                            MorrisResult::Win => {
                                                state.combat_state.add_log_entry(
                                                    "\u{25CB} Victory! The sage bows with respect."
                                                        .to_string(),
                                                    false,
                                                    true,
                                                );
                                                state.combat_state.add_log_entry(
                                                    format!("\u{25CB} +{} XP", xp_gained),
                                                    false,
                                                    true,
                                                );
                                                if fishing_rank_up {
                                                    state.combat_state.add_log_entry(
                                                        format!(
                                                            "\u{25CB} Fishing rank up! Now rank {}: {}",
                                                            state.fishing.rank,
                                                            state.fishing.rank_name()
                                                        ),
                                                        false,
                                                        true,
                                                    );
                                                }
                                            }
                                            MorrisResult::Loss => {
                                                state.combat_state.add_log_entry(
                                                    "\u{25CB} The sage nods knowingly and departs."
                                                        .to_string(),
                                                    false,
                                                    true,
                                                );
                                            }
                                            MorrisResult::Forfeit => {
                                                state.combat_state.add_log_entry(
                                                    "\u{25CB} You concede. The sage gathers their stones quietly.".to_string(),
                                                    false,
                                                    true,
                                                );
                                            }
                                        }
                                    }
                                    continue;
                                }
                                if !morris_game.ai_thinking {
                                    match key_event.code {
                                        KeyCode::Up => {
                                            morris_game.move_cursor(MorrisCursorDirection::Up)
                                        }
                                        KeyCode::Down => {
                                            morris_game.move_cursor(MorrisCursorDirection::Down)
                                        }
                                        KeyCode::Left => {
                                            morris_game.move_cursor(MorrisCursorDirection::Left)
                                        }
                                        KeyCode::Right => {
                                            morris_game.move_cursor(MorrisCursorDirection::Right)
                                        }
                                        KeyCode::Enter => {
                                            handle_morris_enter(&mut state);
                                        }
                                        KeyCode::Esc => {
                                            if morris_game.forfeit_pending {
                                                morris_game.game_result =
                                                    Some(MorrisResult::Forfeit);
                                            } else if morris_game.selected_position.is_some() {
                                                morris_game.clear_selection();
                                                morris_game.forfeit_pending = false;
                                            } else {
                                                morris_game.forfeit_pending = true;
                                            }
                                        }
                                        _ => {
                                            morris_game.forfeit_pending = false;
                                        }
                                    }
                                }
                                continue;
                            }

                            // Handle challenge menu input
                            if state.challenge_menu.is_open {
                                let menu = &mut state.challenge_menu;
                                if menu.viewing_detail {
                                    match key_event.code {
                                        KeyCode::Up => menu.navigate_up(),
                                        KeyCode::Down => menu.navigate_down(4),
                                        KeyCode::Enter => {
                                            if let Some(challenge) = menu.take_selected() {
                                                match challenge.challenge_type {
                                                    ChallengeType::Chess => {
                                                        let difficulty =
                                                            ChessDifficulty::from_index(
                                                                menu.selected_difficulty,
                                                            );
                                                        start_chess_game(&mut state, difficulty);
                                                    }
                                                    ChallengeType::Morris => {
                                                        let difficulty =
                                                            MorrisDifficulty::from_index(
                                                                menu.selected_difficulty,
                                                            );
                                                        start_morris_game(&mut state, difficulty);
                                                    }
                                                    ChallengeType::Gomoku => {
                                                        let difficulty =
                                                            GomokuDifficulty::from_index(
                                                                menu.selected_difficulty,
                                                            );
                                                        start_gomoku_game(&mut state, difficulty);
                                                    }
                                                }
                                            }
                                        }
                                        KeyCode::Char('d') | KeyCode::Char('D') => {
                                            menu.take_selected();
                                            menu.close_detail();
                                            if menu.challenges.is_empty() {
                                                menu.close();
                                            }
                                        }
                                        KeyCode::Esc => menu.close_detail(),
                                        _ => {}
                                    }
                                } else {
                                    match key_event.code {
                                        KeyCode::Up => menu.navigate_up(),
                                        KeyCode::Down => menu.navigate_down(4),
                                        KeyCode::Enter => menu.open_detail(),
                                        KeyCode::Tab | KeyCode::Esc => menu.close(),
                                        _ => {}
                                    }
                                }
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
                                    // Save character before returning to select
                                    character_manager.save_character(&state)?;
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

                    // Auto-save every 30 seconds
                    if last_autosave.elapsed() >= Duration::from_secs(AUTOSAVE_INTERVAL_SECONDS) {
                        character_manager.save_character(&state)?;
                        last_autosave = Instant::now();
                    }

                    // Periodic update check (every 30 minutes)
                    // Only start a new check if we don't have one running and haven't found an update
                    if update_info.is_none()
                        && update_check_handle.is_none()
                        && last_update_check.elapsed()
                            >= Duration::from_secs(UPDATE_CHECK_INTERVAL_SECONDS)
                    {
                        update_check_handle = Some(std::thread::spawn(updater::check_update_info));
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
    use combat_logic::update_combat;
    use dungeon_logic::{
        on_boss_defeated, on_elite_defeated, on_treasure_room_entered, update_dungeon,
    };
    use fishing_logic::tick_fishing;

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

    // Try challenge discovery (single roll, weighted table picks which type)
    {
        let mut rng = rand::thread_rng();
        if let Some(challenge_type) = try_discover_challenge(game_state, &mut rng) {
            let (icon, flavor) = match challenge_type {
                ChallengeType::Chess => ("♟", "A mysterious figure steps from the shadows..."),
                ChallengeType::Morris => (
                    "\u{25CB}",
                    "A cloaked stranger approaches with a weathered board...",
                ),
                ChallengeType::Gomoku => (
                    "◎",
                    "A wandering strategist places a worn board before you...",
                ),
            };
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
    let derived = derived_stats::DerivedStats::calculate_derived_stats(
        &game_state.attributes,
        &game_state.equipment,
    );
    game_state.combat_state.update_max_hp(derived.max_hp);

    // Update dungeon exploration if in a dungeon
    if game_state.active_dungeon.is_some() {
        let dungeon_events = update_dungeon(game_state, delta_time);
        for event in dungeon_events {
            use dungeon_logic::DungeonEvent;
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
        if let Some(rank_msg) = fishing_logic::check_rank_up(&mut game_state.fishing) {
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
        use combat_logic::CombatEvent;
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
                dungeon_logic::add_dungeon_xp(game_state, xp_gained);
                if let Some(dungeon) = &mut game_state.active_dungeon {
                    dungeon_logic::on_room_enemy_defeated(dungeon);
                }

                // Try to drop item
                use item_drops::try_drop_item;
                use item_scoring::auto_equip_if_better;

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
                    if let Some(message) = fishing_logic::try_discover_fishing(game_state, &mut rng)
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
                dungeon_logic::add_dungeon_xp(game_state, xp_gained);

                // Give key
                if let Some(dungeon) = &mut game_state.active_dungeon {
                    let events = on_elite_defeated(dungeon);
                    for event in events {
                        if matches!(event, dungeon_logic::DungeonEvent::FoundKey) {
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
                    let bonus = dungeon_logic::calculate_boss_xp_reward(dungeon.size);
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
