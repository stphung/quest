//! Quest - Terminal-Based Idle RPG Library
//!
//! This module exposes the game logic for testing and external use.

// Allow dead code in library - some functions are only used by the binary
#![allow(dead_code)]

pub mod achievements;
pub mod challenges;
pub mod character;
pub mod combat;
pub mod core;
pub mod dungeon;
pub mod fishing;
pub mod haven;
pub mod items;
pub mod utils;
pub mod zones;

// UI module is not exposed as it's tightly coupled to the terminal
mod ui;

// Re-export commonly used types at crate root for convenience
pub use achievements::{AchievementCategory, AchievementId, Achievements};
pub use challenges::{
    ActiveMinigame, ChessDifficulty, ChessGame, ChessResult, GoDifficulty, GoGame, GoResult,
    GomokuDifficulty, GomokuGame, GomokuResult, MinesweeperDifficulty, MinesweeperGame,
    MinesweeperResult, MorrisDifficulty, MorrisGame, MorrisPhase, MorrisResult, RuneDifficulty,
    RuneGame, RuneResult,
};
pub use character::{Attributes, DerivedStats, PrestigeTier};
pub use combat::{CombatState, Enemy};
pub use core::{GameState, TickEvent, TickResult, TICK_INTERVAL_MS};
pub use dungeon::{Dungeon, Room, RoomType};
pub use fishing::{FishRarity, FishingSession};
pub use haven::{Haven, HavenBonusType, HavenBonuses, HavenRoomId};
pub use items::{Equipment, EquipmentSlot, Item, Rarity};
