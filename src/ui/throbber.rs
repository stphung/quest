//! Shared throbber/spinner utilities for UI animations.

use std::time::{SystemTime, UNIX_EPOCH};

/// Braille spinner characters for animated loading indicators.
const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Atmospheric messages shown while waiting for enemies.
const WAITING_MESSAGES: [&str; 20] = [
    "Scanning the horizon...",
    "Something stirs...",
    "Danger lurks nearby...",
    "The hunt continues...",
    "A foe approaches...",
    "Sensing hostiles...",
    "Shadows gather...",
    "The air grows cold...",
    "Footsteps echo...",
    "Steel at the ready...",
    "Eyes in the dark...",
    "Battle draws near...",
    "Vigilance rewarded...",
    "Prey becomes hunter...",
    "Fortune favors the bold...",
    "Destiny awaits...",
    "The wilds stir...",
    "Trouble brewing...",
    "On guard...",
    "Adventure beckons...",
];

/// Returns the current time in milliseconds since UNIX epoch.
fn current_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// Returns the current spinner character based on system time.
/// The spinner cycles every 100ms, completing a full rotation every second.
pub fn spinner_char() -> char {
    let millis = current_millis();
    SPINNER[((millis / 100) % 10) as usize]
}

/// Returns a waiting message based on a seed value.
/// The message stays stable for the same seed, changing only when the seed changes.
pub fn waiting_message(seed: u64) -> &'static str {
    WAITING_MESSAGES[(seed.wrapping_mul(7) as usize) % WAITING_MESSAGES.len()]
}
