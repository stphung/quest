// Tick and timing
pub const TICK_INTERVAL_MS: u64 = 100;
pub const ATTACK_INTERVAL_SECONDS: f64 = 1.5;
pub const HP_REGEN_DURATION_SECONDS: f64 = 2.5;
pub const _ENEMY_RESPAWN_SECONDS: f64 = 2.5;
pub const AUTOSAVE_INTERVAL_SECONDS: u64 = 30;
pub const UPDATE_CHECK_INTERVAL_SECONDS: u64 = 30 * 60; // 30 minutes
pub const UPDATE_CHECK_JITTER_SECONDS: u64 = 5 * 60; // ±5 minutes jitter

// XP and leveling
pub const BASE_XP_PER_TICK: f64 = 1.0;
pub const XP_CURVE_BASE: f64 = 100.0;
pub const XP_CURVE_EXPONENT: f64 = 1.5;
pub const COMBAT_XP_MIN_TICKS: u64 = 200;
pub const COMBAT_XP_MAX_TICKS: u64 = 400;
pub const OFFLINE_MULTIPLIER: f64 = 0.25;
pub const MAX_OFFLINE_SECONDS: i64 = 7 * 24 * 60 * 60;

// Character attributes
pub const BASE_ATTRIBUTE_VALUE: u32 = 10;
pub const NUM_ATTRIBUTES: usize = 6;
pub const BASE_ATTRIBUTE_CAP: u32 = 20;
pub const ATTRIBUTE_CAP_PER_PRESTIGE: u32 = 5;
pub const LEVEL_UP_ATTRIBUTE_POINTS: u32 = 3;

// Prestige multiplier formula: 1.0 + BASE_FACTOR * rank^EXPONENT
pub const PRESTIGE_MULT_BASE_FACTOR: f64 = 0.5;
pub const PRESTIGE_MULT_EXPONENT: f64 = 0.7;

// Item drops
pub const ITEM_DROP_BASE_CHANCE: f64 = 0.15;
pub const ITEM_DROP_PRESTIGE_BONUS: f64 = 0.01;
pub const ITEM_DROP_MAX_CHANCE: f64 = 0.25;
pub const MOB_RARITY_PRESTIGE_BONUS_PER_RANK: f64 = 0.01;
pub const MOB_RARITY_PRESTIGE_BONUS_CAP: f64 = 0.10;
pub const ZONE_ILVL_MULTIPLIER: u32 = 10;
pub const ILVL_SCALING_BASE: f64 = 10.0;
pub const ILVL_SCALING_DIVISOR: f64 = 30.0;

// Discovery chances
pub const DUNGEON_DISCOVERY_CHANCE: f64 = 0.02;
pub const FISHING_DISCOVERY_CHANCE: f64 = 0.05;
pub const CHALLENGE_DISCOVERY_CHANCE: f64 = 0.000014;
pub const HAVEN_DISCOVERY_BASE_CHANCE: f64 = 0.000014;
pub const HAVEN_DISCOVERY_RANK_BONUS: f64 = 0.000007;
pub const HAVEN_MIN_PRESTIGE_RANK: u32 = 10;

// Fishing ranks
pub const BASE_MAX_FISHING_RANK: u32 = 30;
pub const MAX_FISHING_RANK: u32 = 40;

// Real-time minigame frame rate
pub const REALTIME_FRAME_MS: u64 = 16; // ~60 FPS for action games

// Zone progression
pub const KILLS_FOR_BOSS: u32 = 10;
