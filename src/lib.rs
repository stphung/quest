//! Quest - Terminal-Based Idle RPG Library
//!
//! This module exposes the game logic for testing and external use.

// Allow dead code in library - some functions are only used by the binary
#![allow(dead_code)]

pub mod attributes;
pub mod build_info;
pub mod challenge_menu;
pub mod character_manager;
pub mod chess;
pub mod chess_logic;
pub mod combat;
pub mod combat_logic;
pub mod constants;
pub mod derived_stats;
pub mod dungeon;
pub mod dungeon_generation;
pub mod dungeon_logic;
pub mod equipment;
pub mod fishing;
pub mod fishing_generation;
pub mod fishing_logic;
pub mod game_logic;
pub mod game_state;
pub mod item_drops;
pub mod item_generation;
pub mod item_names;
pub mod item_scoring;
pub mod items;
pub mod prestige;
pub mod save_manager;
pub mod updater;
pub mod zones;

// UI module is not exposed as it's tightly coupled to the terminal
mod ui;
