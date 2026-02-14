// Tick and timing
pub const TICK_INTERVAL_MS: u64 = 100;
pub const ATTACK_INTERVAL_SECONDS: f64 = 1.5;
pub const HP_REGEN_DURATION_SECONDS: f64 = 2.5;
pub const _ENEMY_RESPAWN_SECONDS: f64 = 2.5;

// Enemy attack timing (by tier)
pub const ENEMY_ATTACK_INTERVAL_SECONDS: f64 = 2.0;
pub const ENEMY_BOSS_ATTACK_INTERVAL_SECONDS: f64 = 1.8;
pub const ENEMY_ZONE_BOSS_ATTACK_INTERVAL_SECONDS: f64 = 1.5;
pub const ENEMY_DUNGEON_ELITE_ATTACK_INTERVAL_SECONDS: f64 = 1.6;
pub const ENEMY_DUNGEON_BOSS_ATTACK_INTERVAL_SECONDS: f64 = 1.4;
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
pub const KILLS_FOR_BOSS_RETRY: u32 = 5;

// Zone enemy base stats: (base_hp, hp_step, base_dmg, dmg_step, base_def, def_step)
// Index 0 = Zone 1, Index 10 = Zone 11 (The Expanse)
// hp_step/dmg_step/def_step are per-subzone depth increments above depth 1
pub const ZONE_ENEMY_STATS: [(u32, u32, u32, u32, u32, u32); 11] = [
    (55, 9, 7, 2, 0, 0),           // Zone 1: Meadow
    (90, 14, 13, 3, 2, 1),         // Zone 2: Dark Forest
    (160, 22, 22, 4, 6, 2),        // Zone 3: Mountain Pass
    (215, 27, 31, 6, 10, 3),       // Zone 4: Ancient Ruins
    (305, 32, 42, 7, 16, 3),       // Zone 5: Volcanic Wastes
    (380, 40, 53, 8, 22, 4),       // Zone 6: Frozen Tundra
    (485, 45, 67, 10, 29, 4),      // Zone 7: Crystal Caverns
    (575, 54, 78, 11, 35, 6),      // Zone 8: Sunken Kingdom
    (685, 63, 92, 13, 43, 6),      // Zone 9: Floating Isles
    (810, 72, 109, 14, 52, 7),     // Zone 10: Storm Citadel
    (5000, 400, 500, 80, 250, 30), // Zone 11: The Expanse (endgame wall)
];

// Boss multipliers: (hp_mult, dmg_mult, def_mult)
pub const SUBZONE_BOSS_MULTIPLIERS: (f64, f64, f64) = (3.0, 1.5, 1.8);
pub const ZONE_BOSS_MULTIPLIERS: (f64, f64, f64) = (5.0, 1.8, 2.5);
pub const DUNGEON_ELITE_MULTIPLIERS: (f64, f64, f64) = (2.2, 1.5, 1.6);
pub const DUNGEON_BOSS_MULTIPLIERS: (f64, f64, f64) = (3.5, 1.8, 2.0);

// Prestige combat bonus formulas
pub const PRESTIGE_FLAT_DAMAGE_FACTOR: f64 = 5.0;
pub const PRESTIGE_FLAT_DAMAGE_EXPONENT: f64 = 0.7;
pub const PRESTIGE_FLAT_DEFENSE_FACTOR: f64 = 3.0;
pub const PRESTIGE_FLAT_DEFENSE_EXPONENT: f64 = 0.6;
pub const PRESTIGE_CRIT_PER_RANK: f64 = 0.5;
pub const PRESTIGE_CRIT_CAP: f64 = 15.0;
pub const PRESTIGE_FLAT_HP_FACTOR: f64 = 15.0;
pub const PRESTIGE_FLAT_HP_EXPONENT: f64 = 0.6;

// Derived stat formulas
pub const BASE_HP: i32 = 50;
pub const HP_PER_CON_MODIFIER: i32 = 10;
pub const BASE_PHYSICAL_DAMAGE: i32 = 5;
pub const BASE_MAGIC_DAMAGE: i32 = 5;
pub const DAMAGE_PER_STR_MODIFIER: i32 = 2;
pub const DAMAGE_PER_INT_MODIFIER: i32 = 2;
pub const BASE_CRIT_CHANCE_PERCENT: i32 = 5;
pub const XP_MULT_PER_WIS_MODIFIER: f64 = 0.05;
pub const BASE_CRIT_MULTIPLIER: f64 = 2.0;
pub const PRESTIGE_MULT_PER_CHA_MODIFIER: f64 = 0.1;
pub const AFFIX_PERCENT_DIVISOR: f64 = 100.0;

// Mob rarity distribution thresholds
pub const MOB_RARITY_COMMON_BASE: f64 = 0.60;
pub const MOB_RARITY_MAGIC_BASE: f64 = 0.28;
pub const MOB_RARITY_RARE_BASE: f64 = 0.10;
pub const MOB_RARITY_COMMON_FLOOR: f64 = 0.20;
pub const MOB_RARITY_HAVEN_BONUS_CAP: f64 = 0.25;
pub const MOB_RARITY_RARE_BONUS_SHARE: f64 = 0.6;

// Boss rarity distribution (normal boss)
pub const BOSS_NORMAL_MAGIC_THRESHOLD: f64 = 0.40;
pub const BOSS_NORMAL_RARE_THRESHOLD: f64 = 0.75;
pub const BOSS_NORMAL_EPIC_THRESHOLD: f64 = 0.95;
// Boss rarity distribution (final zone boss)
pub const BOSS_FINAL_MAGIC_THRESHOLD: f64 = 0.20;
pub const BOSS_FINAL_RARE_THRESHOLD: f64 = 0.60;
pub const BOSS_FINAL_EPIC_THRESHOLD: f64 = 0.90;

// Fishing session
pub const FISHING_SESSION_MIN_FISH: u32 = 3;
pub const FISHING_SESSION_MAX_FISH: u32 = 8;
pub const FISH_RARITY_COMMON_FLOOR: f64 = 10.0;
pub const FISH_RARITY_BONUS_INTERVAL: u32 = 5;

// Fishing item drop chances by fish rarity
pub const FISHING_DROP_CHANCE_COMMON: f64 = 0.05;
pub const FISHING_DROP_CHANCE_UNCOMMON: f64 = 0.05;
pub const FISHING_DROP_CHANCE_RARE: f64 = 0.15;
pub const FISHING_DROP_CHANCE_EPIC: f64 = 0.35;
pub const FISHING_DROP_CHANCE_LEGENDARY: f64 = 0.75;

// Prestige level requirements
pub const PRESTIGE_HIGH_RANK_THRESHOLD: u32 = 19;
pub const PRESTIGE_HIGH_RANK_BASE_LEVEL: u32 = 220;
pub const PRESTIGE_HIGH_RANK_LEVEL_STEP: u32 = 15;

// Dungeon progression
pub const DUNGEON_LEVEL_TIER_MEDIUM: u32 = 25;
pub const DUNGEON_LEVEL_TIER_LARGE: u32 = 75;
pub const DUNGEON_PRESTIGE_PER_SIZE_TIER: u32 = 2;
pub const DUNGEON_SIZE_VARIATION_DOWN: f64 = 0.2;
pub const DUNGEON_SIZE_VARIATION_UP: f64 = 0.8;

// Level-up point distribution
pub const LEVEL_UP_MAX_DISTRIBUTION_ATTEMPTS: u32 = 100;

// Zone identifiers
pub const FINAL_ZONE_ID: u32 = 10;
pub const EXPANSE_ZONE_ID: u32 = 11;

// Ticks per second (reciprocal of TICK_INTERVAL_MS / 1000)
pub const TICKS_PER_SECOND: u32 = 10;

// Combat log and recent drops
pub const COMBAT_LOG_CAPACITY: usize = 10;

// Enemy stat variance
pub const ENEMY_STAT_VARIANCE_MIN: f64 = 0.9;
pub const ENEMY_STAT_VARIANCE_MAX: f64 = 1.1;

// Number of equipment slots
pub const NUM_EQUIPMENT_SLOTS: u32 = 7;

// Character management
pub const CHARACTER_NAME_MAX_LENGTH: usize = 16;
pub const SAVE_FILE_VERSION: u32 = 2;

// Dungeon generation
pub const DUNGEON_EXTRA_CONNECTION_CHANCE: f64 = 0.15;
pub const DUNGEON_MIN_BOSS_DISTANCE: usize = 4;
pub const DUNGEON_MIN_ELITE_DISTANCE: usize = 1;

// Haven - Stormbreaker
pub const STORMBREAKER_PRESTIGE_REQUIREMENT: u32 = 25;
