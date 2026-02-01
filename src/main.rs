mod constants;
mod game_logic;
mod game_state;
mod prestige;
mod save_manager;

use chrono::Utc;
use constants::*;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use game_logic::*;
use game_state::*;
use prestige::*;
use save_manager::SaveManager;
use std::io;
use std::time::{Duration, Instant};

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

    // Enable raw terminal mode
    enable_raw_mode()?;

    // Run game loop
    let result = run_game_loop(&mut game_state, &save_manager);

    // Cleanup and save on exit
    disable_raw_mode()?;

    // Save before exiting
    println!("\nSaving game...");
    save_manager.save(&game_state)?;
    println!("Game saved. Goodbye!");

    result
}

/// Main game loop that handles input, ticking, and autosaving
fn run_game_loop(game_state: &mut GameState, save_manager: &SaveManager) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut last_autosave = Instant::now();
    let mut tick_counter: u32 = 0;

    println!("Idle RPG - Coming Soon");
    println!("Press 'q' to quit, 'p' to prestige");

    loop {
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
                            println!("\nPrestige successful! Now at rank {}", game_state.prestige_rank);
                        } else {
                            let next_tier = get_next_prestige_tier(game_state.prestige_rank);
                            println!("\nCannot prestige yet. All stats must reach level {}", next_tier.required_level);
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

/// Processes a single game tick, updating XP and stats
fn game_tick(game_state: &mut GameState, tick_counter: &mut u32) {
    // Calculate XP per tick
    let xp_per_tick = xp_gain_per_tick(game_state.prestige_rank);

    // Apply XP to all stats
    for stat in &mut game_state.stats {
        apply_tick_xp(stat, xp_per_tick);
    }

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
