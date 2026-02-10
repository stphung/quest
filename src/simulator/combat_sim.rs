//! Combat simulation - DEPRECATED
//!
//! This module previously contained SimPlayer, SimEnemy, and simulate_combat()
//! which duplicated game logic. That code has been removed in favor of using
//! CoreGame directly (see core/core_game.rs).
//!
//! The simulator now uses CoreGame.tick() which provides the same combat
//! mechanics as the real game, eliminating duplicated logic.
//!
//! This file is kept empty for backward compatibility but may be removed
//! in a future cleanup.

// No exports - all combat simulation now uses CoreGame
