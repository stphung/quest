// Game timing constants
pub const TICK_INTERVAL_MS: u64 = 100;

// Experience and progression constants
pub const BASE_XP_PER_TICK: f64 = 1.0;
pub const XP_CURVE_BASE: f64 = 100.0;
pub const XP_CURVE_EXPONENT: f64 = 1.5;

// Offline progression constants
pub const OFFLINE_MULTIPLIER: f64 = 0.5;
pub const MAX_OFFLINE_SECONDS: i64 = 7 * 24 * 60 * 60; // 7 days in seconds

// Save system constants
pub const AUTOSAVE_INTERVAL_SECONDS: u64 = 30;
pub const SAVE_VERSION_MAGIC: u64 = 0x49444C4552504700; // "IDLERPG\0" in hex

// Combat constants
pub const ENEMY_RESPAWN_SECONDS: f64 = 2.5;
