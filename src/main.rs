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
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use input::{GameOverlay, HavenUiState, InputResult};
use rand::Rng;
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
    let mut rng = rand::thread_rng();
    let jitter = rng.gen_range(0..=2 * UPDATE_CHECK_JITTER_SECONDS);
    let interval = UPDATE_CHECK_INTERVAL_SECONDS - UPDATE_CHECK_JITTER_SECONDS + jitter;
    Duration::from_secs(interval)
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
                            &state,
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
                            ui::prestige_confirm::draw_prestige_confirm(frame, &state);
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
                                &state,
                                haven.get_bonus(haven::HavenBonusType::VaultSlots) as u8,
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
                                state.prestige_rank,
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
                                        state.prestige_rank,
                                    );
                                }
                                input::HavenConfirmation::Forge => {
                                    ui::haven_scene::render_forge_confirmation(
                                        frame,
                                        frame.size(),
                                        &global_achievements,
                                        state.prestige_rank,
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

                            // Track achievements for state changes
                            let mut achievements_changed = false;

                            // Check if prestige occurred and track achievement
                            if state.prestige_rank > prestige_before {
                                global_achievements
                                    .on_prestige(state.prestige_rank, Some(&state.character_name));
                                achievements_changed = true;
                            }

                            // Check if a minigame win occurred
                            if let Some(ref win_info) = state.last_minigame_win {
                                global_achievements.on_minigame_won(
                                    win_info.game_type,
                                    win_info.difficulty,
                                    Some(&state.character_name),
                                );
                                achievements_changed = true;
                                // Clear the win info after processing
                                state.last_minigame_win = None;
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
                                        character_manager.save_character(&state)?;
                                        // Save achievements when quitting to character select
                                        achievements::save_achievements(&global_achievements)?;
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
                                InputResult::ToggleUpdateDetails => {
                                    update_expanded = !update_expanded;
                                }
                            }
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
                            let mut rng = rand::thread_rng();
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

/// Maps tick events to combat log entries and visual effects.
/// Returns true if the HavenDiscovered event was present.
fn apply_tick_events(game_state: &mut GameState, events: &[core::tick::TickEvent]) -> bool {
    use core::tick::TickEvent;
    let mut haven_discovered = false;
    for event in events {
        match event {
            TickEvent::PlayerAttack {
                damage,
                was_crit,
                message,
            } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), *was_crit, true);

                // Spawn damage number effect
                let damage_effect = ui::combat_effects::VisualEffect::new(
                    ui::combat_effects::EffectType::DamageNumber {
                        value: *damage,
                        is_crit: *was_crit,
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
            TickEvent::PlayerAttackBlocked { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::EnemyAttack { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, false);
            }
            TickEvent::EnemyDefeated { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::PlayerDied { message } | TickEvent::PlayerDiedInDungeon { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, false);
            }
            TickEvent::ItemDropped { .. } => {
                // Item drops and recent_drops tracking are handled inside game_tick
            }
            TickEvent::SubzoneBossDefeated { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::DungeonRoomEntered { message, .. }
            | TickEvent::DungeonTreasureFound { message, .. }
            | TickEvent::DungeonKeyFound { message }
            | TickEvent::DungeonBossUnlocked { message }
            | TickEvent::DungeonBossDefeated { message, .. }
            | TickEvent::DungeonEliteDefeated { message, .. }
            | TickEvent::DungeonCompleted { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::DungeonFailed { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, false);
            }
            TickEvent::FishingMessage { message }
            | TickEvent::FishCaught { message, .. }
            | TickEvent::FishingItemFound { message, .. }
            | TickEvent::FishingRankUp { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::StormLeviathanCaught => {
                // Achievement persistence handled by achievements_changed flag at call site
            }
            TickEvent::ChallengeDiscovered {
                message, follow_up, ..
            } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state
                    .combat_state
                    .add_log_entry(follow_up.clone(), false, true);
            }
            TickEvent::DungeonDiscovered { message }
            | TickEvent::FishingSpotDiscovered { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::AchievementUnlocked { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::HavenDiscovered => {
                haven_discovered = true;
            }
            TickEvent::LeveledUp { .. } => {
                // Level-up state changes are handled inside game_tick
            }
        }
    }
    haven_discovered
}
