mod attributes;
mod character_manager;
mod combat;
mod combat_logic;
mod constants;
mod derived_stats;
mod equipment;
mod game_logic;
mod game_state;
mod item_drops;
mod item_generation;
mod item_names;
mod item_scoring;
mod items;
mod prestige;
mod save_manager;
mod ui;

use character_manager::CharacterManager;
use chrono::Utc;
use constants::*;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use game_logic::*;
use game_state::*;
use prestige::*;
use ratatui::{backend::CrosstermBackend, Terminal};
use save_manager::SaveManager;
use std::io;
use std::time::{Duration, Instant};
use ui::character_creation::CharacterCreationScreen;
use ui::character_delete::CharacterDeleteScreen;
use ui::character_rename::CharacterRenameScreen;
use ui::character_select::CharacterSelectScreen;
use ui::draw_ui;

enum Screen {
    CharacterSelect,
    CharacterCreation,
    CharacterDelete,
    CharacterRename,
    Game,
}

fn main() -> io::Result<()> {
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
                                if creation_screen.name_input.len() < 16 {
                                    creation_screen.name_input.push(c);
                                    creation_screen.cursor_position += 1;
                                    creation_screen.validation_error = None;
                                }
                            }
                            KeyCode::Backspace => {
                                if !creation_screen.name_input.is_empty() {
                                    creation_screen.name_input.pop();
                                    creation_screen.cursor_position =
                                        creation_screen.cursor_position.saturating_sub(1);
                                    creation_screen.validation_error = None;
                                }
                            }
                            KeyCode::Enter => {
                                // Validate and create character
                                match character_manager::validate_name(&creation_screen.name_input)
                                {
                                    Ok(_) => {
                                        let new_state = GameState::new(
                                            creation_screen.name_input.clone(),
                                            Utc::now().timestamp(),
                                        );
                                        if let Err(e) = character_manager.save_character(&new_state)
                                        {
                                            creation_screen.validation_error =
                                                Some(format!("Save failed: {}", e));
                                        } else {
                                            // Reset creation screen and go to select
                                            creation_screen = CharacterCreationScreen::new();
                                            select_screen = CharacterSelectScreen::new();
                                            current_screen = Screen::CharacterSelect;
                                        }
                                    }
                                    Err(e) => {
                                        creation_screen.validation_error = Some(e);
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
                                            // Process offline progression
                                            let current_time = Utc::now().timestamp();
                                            let elapsed_seconds =
                                                current_time - state.last_save_time;

                                            if elapsed_seconds > 60 {
                                                let report =
                                                    process_offline_progression(&mut state);
                                                // Store report in combat log
                                                if report.total_level_ups > 0 {
                                                    let message = format!(
                                                        "Offline: +{} XP, +{} levels",
                                                        report.xp_gained, report.total_level_ups
                                                    );
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
                                if delete_screen.confirmation_input.len() < 16 {
                                    delete_screen.confirmation_input.push(c);
                                    delete_screen.cursor_position += 1;
                                }
                            }
                            KeyCode::Backspace => {
                                if !delete_screen.confirmation_input.is_empty() {
                                    delete_screen.confirmation_input.pop();
                                    delete_screen.cursor_position =
                                        delete_screen.cursor_position.saturating_sub(1);
                                }
                            }
                            KeyCode::Enter => {
                                // Check if confirmation matches
                                if delete_screen.confirmation_input
                                    == selected_character.character_name
                                {
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
                                if rename_screen.new_name_input.len() < 16 {
                                    rename_screen.new_name_input.push(c);
                                    rename_screen.cursor_position += 1;
                                    rename_screen.validation_error = None;
                                }
                            }
                            KeyCode::Backspace => {
                                if !rename_screen.new_name_input.is_empty() {
                                    rename_screen.new_name_input.pop();
                                    rename_screen.cursor_position =
                                        rename_screen.cursor_position.saturating_sub(1);
                                    rename_screen.validation_error = None;
                                }
                            }
                            KeyCode::Enter => {
                                // Validate and rename
                                match character_manager::validate_name(
                                    &rename_screen.new_name_input,
                                ) {
                                    Ok(_) => {
                                        if let Err(e) = character_manager.rename_character(
                                            &selected_character.filename,
                                            rename_screen.new_name_input.clone(),
                                        ) {
                                            rename_screen.validation_error =
                                                Some(format!("Rename failed: {}", e));
                                        } else {
                                            rename_screen = CharacterRenameScreen::new();
                                            current_screen = Screen::CharacterSelect;
                                        }
                                    }
                                    Err(e) => {
                                        rename_screen.validation_error = Some(e);
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
                let mut tick_counter: u32 = 0;

                loop {
                    // Draw UI
                    terminal.draw(|frame| {
                        draw_ui(frame, &state);
                    })?;

                    // Poll for input (50ms non-blocking)
                    if event::poll(Duration::from_millis(50))? {
                        if let Event::Key(key_event) = event::read()? {
                            match key_event.code {
                                // Handle 'q'/'Q' to quit
                                KeyCode::Char('q') | KeyCode::Char('Q') => {
                                    // Save character before returning to select
                                    character_manager.save_character(&state)?;
                                    game_state = None;
                                    current_screen = Screen::CharacterSelect;
                                    break;
                                }
                                // Handle 'p'/'P' to prestige
                                KeyCode::Char('p') | KeyCode::Char('P') => {
                                    if can_prestige(&state) {
                                        perform_prestige(&mut state);
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

    // Update combat state (XP only gained from kills, not passively)
    // Each tick is 100ms = 0.1 seconds
    let delta_time = TICK_INTERVAL_MS as f64 / 1000.0;
    let combat_events = update_combat(game_state, delta_time);

    // Process combat events
    for event in combat_events {
        use combat_logic::CombatEvent;
        match event {
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

                // Try to drop item
                use item_drops::try_drop_item;
                use item_scoring::auto_equip_if_better;

                if let Some(item) = try_drop_item(game_state) {
                    let item_name = item.display_name.clone();
                    let rarity = item.rarity;
                    let equipped = auto_equip_if_better(item, game_state);

                    let rarity_name = match rarity {
                        items::Rarity::Common => "Common",
                        items::Rarity::Magic => "Magic",
                        items::Rarity::Rare => "Rare",
                        items::Rarity::Epic => "Epic",
                        items::Rarity::Legendary => "Legendary",
                    };

                    let stars = "⭐".repeat(rarity as usize + 1);
                    let equipped_text = if equipped { " (equipped!)" } else { "" };

                    let message = format!(
                        "🎁 Found: {} [{}] {}{}",
                        item_name, rarity_name, stars, equipped_text
                    );
                    game_state.combat_state.add_log_entry(message, false, true);
                }
            }
            CombatEvent::PlayerDied => {
                // Add to combat log
                game_state.combat_state.add_log_entry(
                    "💀 You died! Prestige ranks lost...".to_string(),
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
