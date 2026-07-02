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
pub const BOSS_ENRAGE_SECONDS: f64 = 60.0;
pub const AUTOSAVE_INTERVAL_SECONDS: u64 = 30;
pub const UPDATE_CHECK_INTERVAL_SECONDS: u64 = 15 * 60; // 15 minutes
pub const UPDATE_CHECK_JITTER_SECONDS: u64 = 5 * 60; // ±5 minutes jitter

// XP and leveling
pub const BASE_XP_PER_TICK: f64 = 1.0;
pub const XP_CURVE_BASE: f64 = 100.0;
pub const XP_CURVE_EXPONENT: f64 = 1.5;
pub const COMBAT_XP_MIN_TICKS: u64 = 200;
pub const COMBAT_XP_MAX_TICKS: u64 = 400;
pub const OFFLINE_MULTIPLIER: f64 = 0.25;
pub const MAX_OFFLINE_SECONDS: i64 = 7 * 24 * 60 * 60;
pub const XP_RATE_WINDOW_SECONDS: usize = 900; // 15 min of combat time

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
pub const MOB_RARITY_PRESTIGE_BONUS_PER_RANK: f64 = 0.005;
pub const MOB_RARITY_PRESTIGE_BONUS_CAP: f64 = 0.10;
pub const ZONE_ILVL_MULTIPLIER: u32 = 10;
pub const ILVL_SCALING_BASE: f64 = 10.0;
pub const ILVL_SCALING_DIVISOR: f64 = 30.0;

// Item tier (quality) system — T0 (worst) to T9 (best)
// Cumulative drop rate thresholds for exponential curve
pub const TIER_THRESHOLDS: [f64; 9] = [
    0.380, // T0: 38.0%
    0.620, // T1: 24.0%
    0.770, // T2: 15.0%
    0.870, // T3: 10.0%
    0.930, // T4:  6.0%
    0.965, // T5:  3.5%
    0.985, // T6:  2.0%
    0.995, // T7:  1.0%
    0.999, // T8:  0.4%
           // T9:  0.1% (remainder)
];
pub const TIER_MULTIPLIERS: [f64; 10] = [
    0.40, // T0
    0.47, // T1
    0.54, // T2
    0.61, // T3
    0.68, // T4
    0.74, // T5
    0.80, // T6
    0.86, // T7
    0.93, // T8
    1.00, // T9
];

// Discovery chances
pub const DUNGEON_DISCOVERY_CHANCE: f64 = 0.01;
pub const FISHING_DISCOVERY_CHANCE: f64 = 0.05;
pub const CHALLENGE_DISCOVERY_CHANCE: f64 = 0.000014;
pub const HAVEN_DISCOVERY_BASE_CHANCE: f64 = 0.000014;
pub const HAVEN_DISCOVERY_RANK_BONUS: f64 = 0.000007;
pub const HAVEN_MIN_PRESTIGE_RANK: u32 = 10;
pub const STORMGLASS_MIN_PRESTIGE_RANK: u32 = 15;
pub const SOULFORGE_DISCOVERY_BASE_CHANCE: f64 = 0.000014;
pub const SOULFORGE_DISCOVERY_RANK_BONUS: f64 = 0.000007;
pub const SOULFORGE_MIN_PRESTIGE_RANK: u32 = 15;
pub const DEEP_MIN_PRESTIGE_RANK: u32 = 15;

// Fishing ranks
pub const BASE_MAX_FISHING_RANK: u32 = 30;
pub const MAX_FISHING_RANK: u32 = 40;

// Real-time minigame frame rate
pub const REALTIME_FRAME_MS: u64 = 16; // ~60 FPS for action games

// Zone progression
pub const KILLS_FOR_BOSS: u32 = 10;

// Combat fitness: death loop and stalemate prevention
pub const DEATH_LOOP_THRESHOLD: u32 = 3;
pub const MOB_FIGHT_TIMEOUT_SECONDS: f64 = 30.0;
// Frontier backoff: after a death-loop retreat, the safe zone must be cycled
// this many times (growing per repeated retreat, capped) before auto-advancing
// back into the zone that triggered the retreat.
pub const FRONTIER_BACKOFF_MAX_CYCLES: u32 = 8;

// Zone enemy base stats: (base_hp, hp_step, base_dmg, dmg_step, base_def, def_step)
// Index 0 = Zone 1, Index 10 = Zone 11 (The Expanse)
// hp_step/dmg_step/def_step are per-subzone depth increments above depth 1
pub const ZONE_ENEMY_STATS: [(u32, u32, u32, u32, u32, u32); 50] = [
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
    // Fracture zones — 1.6x exponential scaling from Zone 11
    (8000, 640, 800, 128, 400, 48),          // Zone 12: Splintered Rim
    (12800, 1024, 1280, 205, 640, 77),       // Zone 13: Ember Ravine
    (20480, 1638, 2048, 328, 1024, 123),     // Zone 14: Heart of the Fault
    (32768, 2621, 3277, 524, 1638, 197),     // Zone 15: Shard Fields
    (52429, 4194, 5243, 839, 2621, 315),     // Zone 16: Refraction Steps
    (83886, 6711, 8389, 1342, 4194, 503),    // Zone 17: Hall of Second Suns
    (134218, 10737, 13422, 2148, 6711, 805), // Zone 18: Ashen Verge
    (214748, 17180, 21475, 3436, 10737, 1289), // Zone 19: Throat of the World
    (343597, 27488, 34360, 5498, 17180, 2062), // Zone 20: The Black Mouth
    // Chapter 4: The Hollow Throne (Zones 21-23) — 1.6x continued
    (549755, 43981, 54976, 8797, 27488, 3299), // Zone 21: Sunken Processional
    (879608, 70370, 87962, 14075, 43981, 5278), // Zone 22: The Pale Archive
    (1407373, 112592, 140739, 22520, 70370, 8445), // Zone 23: The Hollow Throne
    // Chapter 5: The Wailing Reach (Zones 24-26)
    (2251797, 180147, 225182, 36032, 112592, 13512), // Zone 24: The Stillborn Sea
    (3602875, 288235, 360291, 57651, 180147, 21619), // Zone 25: Resonance Fault
    (5764600, 461176, 576466, 92242, 288235, 34590), // Zone 26: The Wailing Reach
    // Chapter 6: The Origin Wound (Zones 27-30)
    (9223360, 737882, 922346, 147587, 461176, 55344), // Zone 27: The Scar Root
    (14757376, 1180611, 1475754, 236139, 737882, 88550), // Zone 28: Echoing Abyss
    (23611802, 1888978, 2361206, 377822, 1180611, 141680), // Zone 29: Threshold of Silence
    (37778883, 3022365, 3777930, 604515, 1888978, 226688), // Zone 30: The Origin Wound
    // Loom Zones — 1.25x exponential scaling from Zone 30
    (47223604, 3777956, 4722412, 755644, 2361222, 283360), // Zone 31: Threadbare Wastes
    (59029505, 4722445, 5903016, 944555, 2951528, 354200), // Zone 32: Spindle Hollow
    (73786881, 5903057, 7378770, 1180693, 3689410, 442750), // Zone 33: The Weft Expanse
    (92233601, 7378821, 9223462, 1475867, 4611763, 553438), // Zone 34: Heart of the Thread Wilds
    (115292001, 9223526, 11529327, 1844833, 5764703, 691797), // Zone 35: Loom's Edge
    (144115002, 11529408, 14411659, 2306042, 7205879, 864746), // Zone 36: Shuttle Run
    (180143752, 14411759, 18014574, 2882552, 9007349, 1080933), // Zone 37: The Pattern Gate
    (225179690, 18014699, 22518218, 3603190, 11259186, 1351166), // Zone 38: Heart of the Woven Frontier
    (281474613, 22518374, 28147772, 4503988, 14073983, 1688957), // Zone 39: Frayed Reaches
    (351843266, 28147968, 35184715, 5629985, 17592479, 2111197), // Zone 40: The Loose Ends
    (439804082, 35184959, 43980894, 7037481, 21990598, 2638996), // Zone 41: Tangle of Fates
    (549755103, 43981199, 54976117, 8796851, 27488248, 3298745), // Zone 42: Heart of the Unraveling
    (687193879, 54976499, 68720146, 10996064, 34360310, 4123431), // Zone 43: The Blueprint Halls
    (858992348, 68720624, 85900183, 13745080, 42950387, 5154288), // Zone 44: Architect's Loom
    (1073740435, 85900780, 107375229, 17181350, 53687984, 6442860), // Zone 45: Tapestry of Stars
    (
        1342175544, 107375975, 134219036, 21476687, 67109980, 8053576,
    ), // Zone 46: Heart of the Grand Design
    (
        1677719430, 134219968, 167773795, 26845859, 83887475, 10066969,
    ), // Zone 47: The Last Shuttle
    (
        2097149288, 167774961, 209717244, 33557324, 104859343, 12583712,
    ), // Zone 48: Reality's Seam
    (
        2621436609, 209718701, 262146554, 41946654, 131074179, 15729640,
    ), // Zone 49: The World Loom
    (
        3276795762, 262148376, 327683193, 52433318, 163842724, 19662050,
    ), // Zone 50: The Origin Thread
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

// Boss rarity distribution (normal boss): 40% Magic, 35% Rare, 23% Epic, 2% Legendary
pub const BOSS_NORMAL_MAGIC_THRESHOLD: f64 = 0.40;
pub const BOSS_NORMAL_RARE_THRESHOLD: f64 = 0.75;
pub const BOSS_NORMAL_EPIC_THRESHOLD: f64 = 0.98;
// Boss rarity distribution (final zone boss): 20% Magic, 40% Rare, 35% Epic, 5% Legendary
pub const BOSS_FINAL_MAGIC_THRESHOLD: f64 = 0.20;
pub const BOSS_FINAL_RARE_THRESHOLD: f64 = 0.60;
pub const BOSS_FINAL_EPIC_THRESHOLD: f64 = 0.95;

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
#[allow(dead_code)]
pub const FIRST_FRACTURE_ZONE_ID: u32 = 12;
#[allow(dead_code)]
pub const LAST_FRACTURE_ZONE_ID: u32 = 30;
#[allow(dead_code)]
pub const FRACTURE_ZONE_STAT_MULTIPLIER: f64 = 1.6;
#[allow(dead_code)]
pub const FIRST_LOOM_ZONE_ID: u32 = 31;
#[allow(dead_code)]
pub const LAST_LOOM_ZONE_ID: u32 = 50;
#[allow(dead_code)]
pub const LOOM_ZONE_STAT_MULTIPLIER: f64 = 1.25;

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

// Wiki
pub const WIKI_URL: &str = "github.com/stphung/quest/wiki";

pub fn wiki_url_for_browser() -> String {
    if WIKI_URL.starts_with("http://") || WIKI_URL.starts_with("https://") {
        WIKI_URL.to_string()
    } else {
        format!("https://{WIKI_URL}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_enemy_stats_has_50_entries() {
        assert_eq!(ZONE_ENEMY_STATS.len(), 50);
    }

    #[test]
    fn test_fracture_zone_stats_zone_12() {
        let z12 = ZONE_ENEMY_STATS[11]; // Index 11 = Zone 12
        assert_eq!(z12, (8000, 640, 800, 128, 400, 48));
    }

    #[test]
    fn test_fracture_zone_stats_zone_20() {
        let z20 = ZONE_ENEMY_STATS[19]; // Index 19 = Zone 20
        assert_eq!(z20, (343597, 27488, 34360, 5498, 17180, 2062));
    }

    #[test]
    fn test_fracture_constants_exist() {
        assert_eq!(FIRST_FRACTURE_ZONE_ID, 12);
        assert_eq!(LAST_FRACTURE_ZONE_ID, 30);
        assert!((FRACTURE_ZONE_STAT_MULTIPLIER - 1.6).abs() < 1e-10);
    }

    #[test]
    fn test_loom_zone_constants() {
        assert_eq!(FIRST_LOOM_ZONE_ID, 31);
        assert_eq!(LAST_LOOM_ZONE_ID, 50);
        assert!((LOOM_ZONE_STAT_MULTIPLIER - 1.25).abs() < 1e-10);
    }

    #[test]
    fn test_fracture_zone_stats_zone_30() {
        let z30 = ZONE_ENEMY_STATS[29]; // Index 29 = Zone 30
        assert_eq!(z30, (37778883, 3022365, 3777930, 604515, 1888978, 226688));
    }

    #[test]
    fn test_fracture_zone_scaling_consistency() {
        // Verify each fracture zone Z12-Z30 is approximately 1.6x the previous zone's base_hp
        for zone_id in 12..=30u32 {
            let prev = ZONE_ENEMY_STATS[(zone_id - 2) as usize]; // previous zone
            let curr = ZONE_ENEMY_STATS[(zone_id - 1) as usize]; // current zone
            let ratio = curr.0 as f64 / prev.0 as f64;
            assert!(
                (ratio - 1.6).abs() < 0.01,
                "Zone {} base_hp ratio to Zone {} is {:.4}, expected ~1.6",
                zone_id,
                zone_id - 1,
                ratio
            );
        }
    }
}
