#![allow(dead_code)]
use crate::challenges::ActiveMinigame;

/// Facade: tick challenge AI with just the active minigame.
pub fn tick_challenge_ai_facade(_minigame: &mut Option<ActiveMinigame>) -> Option<()> {
    todo!("Wire to existing challenge AI tick functions")
}
