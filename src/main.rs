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
use chrono::{Local, Utc};
use core::constants::*;
use core::game_logic::*;
use core::game_state::*;
use core::{CombatBonuses, CombatEngine, GameLoop, TickResult};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use input::{GameOverlay, HavenUiState, InputResult};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};
use ui::achievement_browser_scene::AchievementBrowserState;
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
    let mut combat_engine: Option<CombatEngine> = None;
    let mut pending_offline_report: Option<core::game_logic::OfflineReport> = None;
    let mut pending_haven_offline_bonus: Option<f64> = None;

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
                            &global_achievements,
                        );
                    }
                    // Draw achievement browser overlay if open
                    if achievement_browser.showing {
                        ui::achievement_browser_scene::render_achievement_browser(
                            f,
                            area,
                            &global_achievements,
                            &achievement_browser,
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
                                KeyCode::Esc => achievement_browser.close(),
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

                                        // Log synced achievements (batch message if multiple)
                                        let synced_count = global_achievements.pending_count();
                                        if synced_count > 0 {
                                            if synced_count == 1 {
                                                // Single achievement - show the name
                                                if let Some(id) = global_achievements
                                                    .pending_notifications
                                                    .first()
                                                {
                                                    if let Some(def) =
                                                        achievements::get_achievement_def(*id)
                                                    {
                                                        state.combat_state.add_log_entry(
                                                            format!(
                                                                "🏆 Achievement Unlocked: {}",
                                                                def.name
                                                            ),
                                                            false,
                                                            true,
                                                        );
                                                    }
                                                }
                                            } else {
                                                // Multiple achievements - show count
                                                state.combat_state.add_log_entry(
                                                    format!(
                                                        "🏆 {} achievements synced from progress!",
                                                        synced_count
                                                    ),
                                                    false,
                                                    true,
                                                );
                                            }
                                            // Clear newly_unlocked since we handled logging here
                                            global_achievements.newly_unlocked.clear();
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
                                                    false,
                                                    true,
                                                );
                                                state.combat_state.add_log_entry(
                                                    format!(
                                                        "⚔️ +{} XP gained offline",
                                                        ui::game_common::format_number_short(
                                                            report.xp_gained
                                                        )
                                                    ),
                                                    false,
                                                    true,
                                                );
                                                if report.total_level_ups > 0 {
                                                    state.combat_state.add_log_entry(
                                                        format!(
                                                            "📈 Leveled up {} times! ({} → {})",
                                                            report.total_level_ups,
                                                            report.level_before,
                                                            report.level_after,
                                                        ),
                                                        false,
                                                        true,
                                                    );
                                                }

                                                // Store report and haven bonus for welcome overlay
                                                pending_haven_offline_bonus =
                                                    Some(haven_offline_bonus);
                                                pending_offline_report = Some(report);
                                            }
                                        }

                                        combat_engine = Some(CombatEngine::from_state(state));
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
                    let area = f.size();
                    rename_screen.draw(f, area, selected_character);
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
                // Take core game (it should always be Some when we're in Game screen)
                let mut game = combat_engine
                    .take()
                    .expect("CombatEngine should be initialized when entering Game screen");

                // Run the game loop
                let mut last_tick = Instant::now();
                let mut last_autosave = Instant::now();
                let mut last_update_check = Instant::now();
                let mut tick_counter: u32 = 0;
                let mut overlay = if let Some(report) = pending_offline_report.take() {
                    // Haven bonus already included in report from process_offline_progression
                    let _ = pending_haven_offline_bonus.take(); // Clear unused bonus
                    GameOverlay::OfflineWelcome { report }
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
                let mut update_expanded = false;
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
                            game.state(),
                            update_info.as_ref(),
                            update_expanded,
                            update_check_completed,
                            haven.discovered,
                            &global_achievements,
                        );
                        // Draw offline welcome overlay if active
                        if let GameOverlay::OfflineWelcome { report } = &overlay {
                            ui::game_common::render_offline_welcome(frame, frame.size(), report);
                        }
                        // Draw prestige confirmation overlay if active
                        if matches!(overlay, GameOverlay::PrestigeConfirm) {
                            ui::prestige_confirm::draw_prestige_confirm(frame, game.state());
                        }
                        // Draw Haven discovery modal if active
                        if matches!(overlay, GameOverlay::HavenDiscovery) {
                            ui::haven_scene::render_haven_discovery_modal(frame, frame.size());
                        }
                        // Draw Achievement unlocked modal if active
                        if let GameOverlay::AchievementUnlocked { ref achievements } = overlay {
                            ui::achievement_browser_scene::render_achievement_unlocked_modal(
                                frame,
                                frame.size(),
                                achievements,
                            );
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
                                game.state(),
                                haven.vault_tier(),
                                selected_index,
                                selected_slots,
                            );
                        }
                        // Draw Achievement browser if active
                        if let GameOverlay::Achievements { browser } = &overlay {
                            ui::achievement_browser_scene::render_achievement_browser(
                                frame,
                                frame.size(),
                                &global_achievements,
                                browser,
                            );
                        }
                        // Draw Leviathan encounter modal if active
                        if let GameOverlay::LeviathanEncounter { encounter_number } = overlay {
                            ui::fishing_scene::render_leviathan_encounter_modal(
                                frame,
                                frame.size(),
                                encounter_number,
                            );
                        }
                        // Draw Haven screen if active
                        if haven_ui.showing {
                            ui::haven_scene::render_haven_tree(
                                frame,
                                frame.size(),
                                &haven,
                                haven_ui.selected_room,
                                game.state().prestige_rank,
                                &global_achievements,
                            );
                            match haven_ui.confirmation {
                                input::HavenConfirmation::Build => {
                                    let room = haven::HavenRoomId::ALL[haven_ui.selected_room];
                                    ui::haven_scene::render_build_confirmation(
                                        frame,
                                        frame.size(),
                                        room,
                                        &haven,
                                        game.state().prestige_rank,
                                    );
                                }
                                input::HavenConfirmation::Forge => {
                                    ui::haven_scene::render_forge_confirmation(
                                        frame,
                                        frame.size(),
                                        &global_achievements,
                                        game.state().prestige_rank,
                                    );
                                }
                                input::HavenConfirmation::None => {}
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
                            // Only handle key press events (ignore release/repeat)
                            if key_event.kind != KeyEventKind::Press {
                                continue;
                            }
                            // Track prestige rank before input to detect prestige
                            let prestige_before = game.state().prestige_rank;

                            let result = input::handle_game_input(
                                key_event,
                                game.state_mut(),
                                &mut haven,
                                &mut haven_ui,
                                &mut overlay,
                                &mut debug_menu,
                                debug_mode,
                                &mut global_achievements,
                                update_info.is_some(),
                                update_expanded,
                            );

                            // Track achievements for state changes
                            let mut achievements_changed = false;

                            // Check if prestige occurred and track achievement
                            if game.state().prestige_rank > prestige_before {
                                global_achievements.on_prestige(
                                    game.state().prestige_rank,
                                    Some(&game.state().character_name),
                                );
                                achievements_changed = true;
                            }

                            // Check if a minigame win occurred
                            if let Some(ref win_info) = game.state().last_minigame_win {
                                global_achievements.on_minigame_won(
                                    win_info.game_type,
                                    win_info.difficulty,
                                    Some(&game.state().character_name),
                                );
                                achievements_changed = true;
                                // Clear the win info after processing
                                game.state_mut().last_minigame_win = None;
                            }

                            // Save achievements if any changed (skip in debug mode)
                            if achievements_changed && !debug_mode {
                                if let Err(e) =
                                    achievements::save_achievements(&global_achievements)
                                {
                                    eprintln!("Failed to save achievements: {}", e);
                                }
                            }

                            match result {
                                InputResult::Continue => {}
                                InputResult::QuitToSelect => {
                                    if !debug_mode {
                                        character_manager.save_character(game.state())?;
                                        // Save achievements when quitting to character select
                                        achievements::save_achievements(&global_achievements)?;
                                    }
                                    combat_engine = None;
                                    current_screen = Screen::CharacterSelect;
                                    break;
                                }
                                InputResult::NeedsSave => {
                                    if !debug_mode {
                                        let _ = character_manager.save_character(game.state());
                                        last_save_instant = Some(Instant::now());
                                        last_save_time = Some(Local::now());
                                    }
                                }
                                InputResult::NeedsSaveAll => {
                                    if !debug_mode {
                                        let _ = character_manager.save_character(game.state());
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
                    }

                    // Game tick every 100ms
                    if last_tick.elapsed() >= Duration::from_millis(TICK_INTERVAL_MS) {
                        // Skip game ticks while Leviathan modal is showing
                        if !matches!(overlay, GameOverlay::LeviathanEncounter { .. }) {
                            let leviathan_encounter = game_tick(
                                &mut game,
                                &mut tick_counter,
                                &haven,
                                &mut global_achievements,
                                debug_mode,
                            );

                            // Show Leviathan encounter modal if one occurred
                            if let Some(encounter_number) = leviathan_encounter {
                                overlay = GameOverlay::LeviathanEncounter { encounter_number };
                            }
                        }
                        last_tick = Instant::now();

                        // Haven discovery check (independent roll, once per tick)
                        if !haven.discovered
                            && game.state().prestige_rank >= 10
                            && game.state().active_dungeon.is_none()
                            && game.state().active_fishing.is_none()
                            && game.state().active_minigame.is_none()
                        {
                            let mut rng = rand::thread_rng();
                            if haven::try_discover_haven(
                                &mut haven,
                                game.state().prestige_rank,
                                &mut rng,
                            ) {
                                if !debug_mode {
                                    haven::save_haven(&haven).ok();
                                }
                                // Track Haven discovery achievement
                                global_achievements
                                    .on_haven_discovered(Some(&game.state().character_name));
                                if !debug_mode {
                                    if let Err(e) =
                                        achievements::save_achievements(&global_achievements)
                                    {
                                        eprintln!("Failed to save achievements: {}", e);
                                    }
                                }
                                overlay = GameOverlay::HavenDiscovery;
                            }
                        }

                        // Check if achievement modal is ready to show
                        // Only show if no other overlay is active
                        if matches!(overlay, GameOverlay::None)
                            && global_achievements.is_modal_ready()
                        {
                            let achievements = global_achievements.take_modal_queue();
                            if !achievements.is_empty() {
                                overlay = GameOverlay::AchievementUnlocked { achievements };
                            }
                        }

                        // Save achievements immediately when any are newly unlocked
                        if !debug_mode && !global_achievements.newly_unlocked.is_empty() {
                            if let Err(e) = achievements::save_achievements(&global_achievements) {
                                eprintln!("Failed to save achievements: {}", e);
                            }
                        }
                    }

                    // Auto-save every 30 seconds (skip in debug mode)
                    if !debug_mode
                        && last_autosave.elapsed() >= Duration::from_secs(AUTOSAVE_INTERVAL_SECONDS)
                    {
                        character_manager.save_character(game.state())?;
                        // Only save Haven if it has been discovered
                        if haven.discovered {
                            haven::save_haven(&haven)?;
                        }
                        // Save achievements (global, shared across characters)
                        achievements::save_achievements(&global_achievements)?;
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

/// Processes overworld combat using CombatEngine's combat_tick.
/// Handles attack timing, applies bonuses, and generates visual effects from TickResult.
fn process_overworld_combat(
    combat_engine: &mut CombatEngine,
    delta_time: f64,
    haven: &haven::Haven,
    global_achievements: &mut achievements::Achievements,
) {
    use crate::character::derived_stats::DerivedStats;

    let state = combat_engine.state_mut();

    // Handle HP regeneration first
    if state.combat_state.is_regenerating {
        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        let haven_regen_percent = haven.get_bonus(haven::HavenBonusType::HpRegenPercent);
        let haven_regen_delay = haven.get_bonus(haven::HavenBonusType::HpRegenDelayReduction);

        let total_regen_mult = derived.hp_regen_multiplier * (1.0 + haven_regen_percent / 100.0);
        let base_duration = HP_REGEN_DURATION_SECONDS * (1.0 - haven_regen_delay / 100.0);
        let effective_duration = base_duration / total_regen_mult;

        state.combat_state.regen_timer += delta_time;

        if state.combat_state.regen_timer >= effective_duration {
            state.combat_state.player_current_hp = state.combat_state.player_max_hp;
            state.combat_state.is_regenerating = false;
            state.combat_state.regen_timer = 0.0;
        } else {
            let progress = state.combat_state.regen_timer / effective_duration;
            let start_hp = state.combat_state.player_current_hp;
            let target_hp = state.combat_state.player_max_hp;
            state.combat_state.player_current_hp =
                start_hp + ((target_hp - start_hp) as f64 * progress) as u32;
        }
        return;
    }

    // Check attack timer
    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
    let effective_attack_interval = ATTACK_INTERVAL_SECONDS / derived.attack_speed_multiplier;

    state.combat_state.attack_timer += delta_time;
    if state.combat_state.attack_timer < effective_attack_interval {
        return; // Not time to attack yet
    }
    state.combat_state.attack_timer = 0.0;

    // Build combat bonuses from Haven and set on CombatEngine
    let bonuses = CombatBonuses {
        damage_percent: haven.get_bonus(haven::HavenBonusType::DamagePercent),
        crit_chance: haven.get_bonus(haven::HavenBonusType::CritChancePercent) as u32,
        drop_rate_percent: haven.get_bonus(haven::HavenBonusType::DropRatePercent),
        item_rarity_percent: haven.get_bonus(haven::HavenBonusType::ItemRarityPercent),
        xp_gain_percent: haven.get_bonus(haven::HavenBonusType::XpGainPercent),
        double_strike_chance: haven.get_bonus(haven::HavenBonusType::DoubleStrikeChance),
    };
    combat_engine.set_bonuses(bonuses);

    // Execute combat using CombatEngine's combat_tick
    let mut rng = rand::thread_rng();
    let result = combat_engine.combat_tick(global_achievements, &mut rng);

    // Convert TickResult to visual effects and combat log entries
    process_tick_result(combat_engine.state_mut(), &result, global_achievements);
}

/// Converts a TickResult into visual effects and combat log entries.
fn process_tick_result(
    game_state: &mut GameState,
    result: &TickResult,
    global_achievements: &mut achievements::Achievements,
) {
    if !result.had_combat {
        return;
    }

    // Attack blocked by weapon requirement
    if result.attack_blocked {
        if let Some(ref weapon) = result.weapon_needed {
            let message = format!("🚫 {} required to damage this foe!", weapon);
            game_state.combat_state.add_log_entry(message, false, true);
        }
    }

    // Player attack
    if result.damage_dealt > 0 {
        let message = if result.was_double_strike && result.was_crit {
            format!("⚔⚔ DOUBLE STRIKE! 💥 CRITICAL! {} damage!", result.damage_dealt)
        } else if result.was_double_strike {
            format!("⚔⚔ DOUBLE STRIKE for {} damage!", result.damage_dealt)
        } else if result.was_crit {
            format!("💥 CRITICAL HIT for {} damage!", result.damage_dealt)
        } else {
            format!("⚔ You hit for {} damage", result.damage_dealt)
        };
        game_state
            .combat_state
            .add_log_entry(message, result.was_crit, true);

        // Visual effects
        let damage_effect = ui::combat_effects::VisualEffect::new(
            ui::combat_effects::EffectType::DamageNumber {
                value: result.damage_dealt,
                is_crit: result.was_crit,
            },
            0.8,
        );
        game_state.combat_state.visual_effects.push(damage_effect);

        let flash_effect =
            ui::combat_effects::VisualEffect::new(ui::combat_effects::EffectType::AttackFlash, 0.2);
        game_state.combat_state.visual_effects.push(flash_effect);

        let impact_effect =
            ui::combat_effects::VisualEffect::new(ui::combat_effects::EffectType::HitImpact, 0.3);
        game_state.combat_state.visual_effects.push(impact_effect);
    }

    // Enemy attack
    if result.damage_taken > 0 {
        // Get enemy name from result (if killed) or from current enemy (if still alive)
        let enemy_name = result.enemy_name.as_ref().or_else(|| {
            game_state
                .combat_state
                .current_enemy
                .as_ref()
                .map(|e| &e.name)
        });
        if let Some(name) = enemy_name {
            let message = format!("🛡 {} hits you for {} damage", name, result.damage_taken);
            game_state.combat_state.add_log_entry(message, false, false);
        }
    }

    // Enemy defeated
    if result.player_won {
        if let Some(ref enemy_name) = result.enemy_name {
            let message = format!("✨ {} defeated! +{} XP", enemy_name, result.xp_gained);
            game_state.combat_state.add_log_entry(message, false, true);
        }

        game_state.session_kills += 1;

        // Track level up achievement
        if result.leveled_up {
            global_achievements.on_level_up(result.new_level, Some(&game_state.character_name));
        }

        // Handle loot display
        if let Some(ref item) = result.loot_dropped {
            let icon = if result.was_boss { "👑" } else { "🎁" };
            game_state.add_recent_drop(
                item.display_name.clone(),
                item.rarity,
                result.loot_equipped,
                icon,
                item.slot_name().to_string(),
                item.stat_summary(),
            );
        }

        // Try dungeon/fishing discovery
        if try_discover_dungeon(game_state) {
            game_state.combat_state.add_log_entry(
                "🌀 You notice a dark passage leading underground...".to_string(),
                false,
                true,
            );
        } else if game_state.active_fishing.is_none() {
            let mut rng = rand::thread_rng();
            if let Some(message) = fishing::logic::try_discover_fishing(game_state, &mut rng) {
                game_state
                    .combat_state
                    .add_log_entry(format!("🎣 {}", message), false, true);
            }
        }

        // Log zone advancement
        if result.zone_advanced {
            if let Some(zone) = zones::get_zone(result.new_zone) {
                game_state.combat_state.add_log_entry(
                    format!("👑 Advancing to {}!", zone.name),
                    false,
                    true,
                );
            }
        }
    }

    // Player died
    if result.player_died {
        if result.was_boss {
            game_state.combat_state.add_log_entry(
                "💀 You died! Boss encounter reset.".to_string(),
                false,
                false,
            );
        } else {
            game_state.combat_state.add_log_entry(
                "💀 You died! Regenerating...".to_string(),
                false,
                false,
            );
        }
    }
}

/// Processes a single game tick, updating combat and stats.
/// Returns Some(encounter_number) if a Storm Leviathan encounter occurred during fishing.
fn game_tick(
    combat_engine: &mut CombatEngine,
    tick_counter: &mut u32,
    haven: &haven::Haven,
    global_achievements: &mut achievements::Achievements,
    debug_mode: bool,
) -> Option<u8> {
    use combat::logic::update_combat;
    use dungeon::logic::{
        on_boss_defeated, on_elite_defeated, on_treasure_room_entered, update_dungeon,
    };

    // Each tick is 100ms = 0.1 seconds
    let delta_time = TICK_INTERVAL_MS as f64 / 1000.0;

    // Get mutable access to game state
    let game_state = combat_engine.state_mut();

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
            max_fishing_rank_bonus: haven.fishing_rank_bonus(),
        };
        let fishing_result =
            fishing::logic::tick_fishing_with_haven_result(game_state, &mut rng, &haven_fishing);

        // Check if Storm Leviathan was caught - unlock achievement
        if fishing_result.caught_storm_leviathan {
            global_achievements.on_storm_leviathan_caught(Some(&game_state.character_name));
            if !debug_mode {
                if let Err(e) = achievements::save_achievements(global_achievements) {
                    eprintln!("Failed to save achievements: {}", e);
                }
            }
        }

        let fishing_messages = fishing_result.messages;
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
                game_state.add_recent_drop(
                    fish_name,
                    rarity,
                    false,
                    "🐟",
                    String::new(),
                    String::new(),
                );
            } else if message.contains("Found item:") {
                let item_name = message
                    .split("Found item: ")
                    .nth(1)
                    .map(|s| s.trim_end_matches('!'))
                    .unwrap_or("Item")
                    .to_string();
                game_state.add_recent_drop(
                    item_name,
                    items::types::Rarity::Rare,
                    false,
                    "📦",
                    String::new(),
                    String::new(),
                );
            }
        }

        // Check for fishing rank up (capped by Haven Fishing Dock tier)
        let max_rank = fishing::logic::get_max_fishing_rank(haven_fishing.max_fishing_rank_bonus);
        if let Some(rank_msg) =
            fishing::logic::check_rank_up_with_max(&mut game_state.fishing, max_rank)
        {
            game_state
                .combat_state
                .add_log_entry(format!("🎣 {}", rank_msg), false, true);
        }

        // Update play_time_seconds while fishing
        *tick_counter += 1;
        if *tick_counter >= 10 {
            game_state.play_time_seconds += 1;
            *tick_counter = 0;
        }

        return fishing_result.leviathan_encounter; // Skip combat processing while fishing
    }

    // Combat processing - use different engines for overworld vs dungeon
    // Check dungeon status before releasing the borrow
    let in_dungeon = game_state.active_dungeon.is_some();

    if in_dungeon {
        // Re-borrow for dungeon combat (game_state is still valid here)
        // Dungeon combat uses the full update_combat with special handling for
        // elites, bosses, and dungeon-specific events
        let haven_combat = combat::logic::HavenCombatBonuses {
            hp_regen_percent: haven.get_bonus(haven::HavenBonusType::HpRegenPercent),
            hp_regen_delay_reduction: haven.get_bonus(haven::HavenBonusType::HpRegenDelayReduction),
            damage_percent: haven.get_bonus(haven::HavenBonusType::DamagePercent),
            crit_chance_percent: haven.get_bonus(haven::HavenBonusType::CritChancePercent),
            double_strike_chance: haven.get_bonus(haven::HavenBonusType::DoubleStrikeChance),
            xp_gain_percent: haven.get_bonus(haven::HavenBonusType::XpGainPercent),
        };
        let combat_events =
            update_combat(game_state, delta_time, &haven_combat, global_achievements);

        // Process dungeon combat events
        for event in combat_events {
            use combat::logic::CombatEvent;
            match event {
                CombatEvent::PlayerAttackBlocked { weapon_needed } => {
                    let message = format!("🚫 {} required to damage this foe!", weapon_needed);
                    game_state.combat_state.add_log_entry(message, false, true);
                }
                CombatEvent::PlayerAttack { damage, was_crit } => {
                    let message = if was_crit {
                        format!("💥 CRITICAL HIT for {} damage!", damage)
                    } else {
                        format!("⚔ You hit for {} damage", damage)
                    };
                    game_state
                        .combat_state
                        .add_log_entry(message, was_crit, true);

                    // Visual effects
                    let damage_effect = ui::combat_effects::VisualEffect::new(
                        ui::combat_effects::EffectType::DamageNumber {
                            value: damage,
                            is_crit: was_crit,
                        },
                        0.8,
                    );
                    game_state.combat_state.visual_effects.push(damage_effect);

                    let flash_effect = ui::combat_effects::VisualEffect::new(
                        ui::combat_effects::EffectType::AttackFlash,
                        0.2,
                    );
                    game_state.combat_state.visual_effects.push(flash_effect);

                    let impact_effect = ui::combat_effects::VisualEffect::new(
                        ui::combat_effects::EffectType::HitImpact,
                        0.3,
                    );
                    game_state.combat_state.visual_effects.push(impact_effect);
                }
                CombatEvent::EnemyAttack { damage } => {
                    if let Some(enemy) = &game_state.combat_state.current_enemy {
                        let message = format!("🛡 {} hits you for {} damage", enemy.name, damage);
                        game_state.combat_state.add_log_entry(message, false, false);
                    }
                }
                CombatEvent::EnemyDied { xp_gained } => {
                    if let Some(enemy) = &game_state.combat_state.current_enemy {
                        let message = format!("✨ {} defeated! +{} XP", enemy.name, xp_gained);
                        game_state.combat_state.add_log_entry(message, false, true);
                    }
                    let level_before = game_state.character_level;
                    apply_tick_xp(game_state, xp_gained as f64);
                    if game_state.character_level > level_before {
                        global_achievements.on_level_up(
                            game_state.character_level,
                            Some(&game_state.character_name),
                        );
                    }
                    game_state.session_kills += 1;

                    // Track XP in dungeon and mark room cleared
                    dungeon::logic::add_dungeon_xp(game_state, xp_gained);
                    if let Some(dungeon) = &mut game_state.active_dungeon {
                        dungeon::logic::on_room_enemy_defeated(dungeon);
                    }

                    // Handle loot drops
                    use items::drops::{try_drop_from_boss, try_drop_from_mob};
                    use items::scoring::auto_equip_if_better;

                    let zone_id = game_state.zone_progression.current_zone_id as usize;
                    let was_boss = game_state.zone_progression.fighting_boss;
                    let is_final_zone = zone_id == 10;

                    let dropped_item = if was_boss {
                        Some(try_drop_from_boss(zone_id, is_final_zone))
                    } else {
                        let haven_drop_rate =
                            haven.get_bonus(haven::HavenBonusType::DropRatePercent);
                        let haven_rarity =
                            haven.get_bonus(haven::HavenBonusType::ItemRarityPercent);
                        try_drop_from_mob(game_state, zone_id, haven_drop_rate, haven_rarity)
                    };

                    if let Some(item) = dropped_item {
                        let item_name = item.display_name.clone();
                        let rarity = item.rarity;
                        let slot = item.slot_name().to_string();
                        let stats = item.stat_summary();
                        let icon = if was_boss { "👑" } else { "🎁" };
                        let equipped = auto_equip_if_better(item, game_state);
                        game_state.add_recent_drop(item_name, rarity, equipped, icon, slot, stats);
                    }
                }
                CombatEvent::EliteDefeated { xp_gained } => {
                    // Elite defeated - give key
                    if let Some(enemy) = &game_state.combat_state.current_enemy {
                        let message = format!("⚔️ {} defeated! +{} XP", enemy.name, xp_gained);
                        game_state.combat_state.add_log_entry(message, false, true);
                    }
                    let level_before = game_state.character_level;
                    apply_tick_xp(game_state, xp_gained as f64);
                    if game_state.character_level > level_before {
                        global_achievements.on_level_up(
                            game_state.character_level,
                            Some(&game_state.character_name),
                        );
                    }
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
                    let level_before = game_state.character_level;
                    apply_tick_xp(game_state, xp_gained as f64);

                    // Calculate boss bonus XP (copy values before mutable borrow)
                    let (bonus_xp, total_xp, items) =
                        if let Some(dungeon) = &game_state.active_dungeon {
                            let bonus = dungeon::logic::calculate_boss_xp_reward(dungeon.size);
                            let total = dungeon.xp_earned + xp_gained + bonus;
                            let item_count = dungeon.collected_items.len();
                            (bonus, total, item_count)
                        } else {
                            (0, xp_gained, 0)
                        };

                    apply_tick_xp(game_state, bonus_xp as f64);
                    if game_state.character_level > level_before {
                        global_achievements.on_level_up(
                            game_state.character_level,
                            Some(&game_state.character_name),
                        );
                    }

                    // Track dungeon completion for achievements
                    global_achievements.on_dungeon_completed(Some(&game_state.character_name));

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
                    let level_before = game_state.character_level;
                    apply_tick_xp(game_state, xp_gained as f64);
                    if game_state.character_level > level_before {
                        global_achievements.on_level_up(
                            game_state.character_level,
                            Some(&game_state.character_name),
                        );
                    }
                    game_state.session_kills += 1;

                    // Track zone fully cleared for achievements
                    match &result {
                        BossDefeatResult::ZoneComplete { old_zone, .. }
                        | BossDefeatResult::ZoneCompleteButGated {
                            zone_name: old_zone,
                            ..
                        } => {
                            // Get zone ID from the old zone name
                            if let Some(zone) =
                                zones::get_all_zones().iter().find(|z| z.name == *old_zone)
                            {
                                global_achievements.on_zone_fully_cleared(
                                    zone.id,
                                    Some(&game_state.character_name),
                                );
                            }
                        }
                        BossDefeatResult::StormsEnd => {
                            // Zone 10 (Storm Citadel) completed
                            global_achievements
                                .on_zone_fully_cleared(10, Some(&game_state.character_name));
                            global_achievements.on_storms_end(Some(&game_state.character_name));
                        }
                        BossDefeatResult::ExpanseCycle => {
                            // Zone 11 (The Expanse) cycle completed
                            global_achievements
                                .on_zone_fully_cleared(11, Some(&game_state.character_name));
                        }
                        _ => {}
                    }

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
                        BossDefeatResult::StormsEnd => {
                            format!(
                                "👑 All zones conquered! +{} XP — You have completed the game!",
                                xp_gained
                            )
                        }
                        BossDefeatResult::WeaponRequired { .. } => {
                            // Already handled by PlayerAttackBlocked
                            continue;
                        }
                        BossDefeatResult::ExpanseCycle => {
                            format!(
                                "👑 The Endless defeated! +{} XP — The Expanse cycles anew...",
                                xp_gained
                            )
                        }
                    };
                    game_state.combat_state.add_log_entry(message, false, true);
                }
                _ => {}
            }
        }
    } else {
        // Overworld combat uses CombatEngine's combat_tick for game logic
        // This separates the game logic (in CombatEngine) from UI concerns (here)
        // Note: We're done with game_state in this branch - release the borrow
        let _ = game_state;
        process_overworld_combat(combat_engine, delta_time, haven, global_achievements);
    }

    // Update visual effects
    let delta_time = TICK_INTERVAL_MS as f64 / 1000.0;
    combat_engine
        .state_mut()
        .combat_state
        .visual_effects
        .retain_mut(|effect| effect.update(delta_time));

    // Spawn enemy if needed
    spawn_enemy_if_needed(combat_engine.state_mut());

    // Update play_time_seconds
    // Each tick is 100ms (TICK_INTERVAL_MS), so 10 ticks = 1 second
    *tick_counter += 1;
    if *tick_counter >= 10 {
        combat_engine.state_mut().play_time_seconds += 1;
        *tick_counter = 0;
    }

    // Log any newly unlocked achievements to combat log
    for id in global_achievements.take_newly_unlocked() {
        if let Some(def) = achievements::get_achievement_def(id) {
            combat_engine.state_mut().combat_state.add_log_entry(
                format!("🏆 Achievement Unlocked: {}", def.name),
                false,
                true,
            );
        }
    }

    None
}
