//! Shared helpers for fishing integration tests.

use quest::GameState;

/// Build a fresh character state for fishing tests.
pub fn fresh_state() -> GameState {
    GameState::new("TestHero".to_string(), 0)
}
