mod attributes;
mod combat;
mod combat_logic;
mod constants;
mod derived_stats;
mod game_logic;
mod game_state;
mod prestige;
mod save_manager;
mod ui;

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
use ui::draw_ui;

fn main() -> io::Result<()> {
    // Initialize SaveManager
    let save_manager = SaveManager::new()?;

    // Load existing save or create new GameState
    let mut game_state = if save_manager.save_exists() {
        println!("Welcome back! Loading your save...");
        let state = save_manager.load()?;
        println!("Save loaded successfully.");
        state
    } else {
        println!("Starting new game...");
        GameState::new(Utc::now().timestamp())
    };

    // Process offline progression if > 60 seconds elapsed
    let current_time = Utc::now().timestamp();
    let elapsed_seconds = current_time - game_state.last_save_time;

    if elapsed_seconds > 60 {
        println!("Processing offline progression...");
        let report = process_offline_progression(&mut game_state);

        if report.total_level_ups > 0 {
            println!(
                "While you were away for {} seconds, you gained {} XP and {} total levels!",
                report.elapsed_seconds, report.xp_gained, report.total_level_ups
            );
        }
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run game loop
    let result = run_game_loop(&mut game_state, &save_manager, &mut terminal);

    // Cleanup terminal
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;

    // Save before exiting
    println!("\nSaving game...");
    save_manager.save(&game_state)?;
    println!("Game saved. Goodbye!");

    result
}

/// Main game loop that handles input, ticking, and autosaving
fn run_game_loop(
    game_state: &mut GameState,
    save_manager: &SaveManager,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut last_autosave = Instant::now();
    let mut tick_counter: u32 = 0;

    loop {
        // Draw UI
        terminal.draw(|frame| {
            draw_ui(frame, game_state);
        })?;

        // Poll for input (50ms non-blocking)
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    // Handle 'q'/'Q' to quit
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        return Ok(());
                    }
                    // Handle 'p'/'P' to prestige
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        if can_prestige(game_state) {
                            perform_prestige(game_state);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Game tick every 100ms
        if last_tick.elapsed() >= Duration::from_millis(TICK_INTERVAL_MS) {
            game_tick(game_state, &mut tick_counter);
            last_tick = Instant::now();
        }

        // Auto-save every 30 seconds
        if last_autosave.elapsed() >= Duration::from_secs(AUTOSAVE_INTERVAL_SECONDS) {
            save_manager.save(game_state)?;
            last_autosave = Instant::now();
        }
    }
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
            CombatEvent::EnemyDied { xp_gained } => {
                apply_tick_xp(game_state, xp_gained as f64);
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
