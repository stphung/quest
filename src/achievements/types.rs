//! Achievement system types and data structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Achievement categories for organization in the browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementCategory {
    Combat,
    Level,
    Prestige,
    Progression,
    Challenges,
    Exploration,
    Deep,
    Loom,
    Stats,
}

impl AchievementCategory {
    /// All categories in display order.
    pub const ALL: [AchievementCategory; 9] = [
        AchievementCategory::Combat,
        AchievementCategory::Level,
        AchievementCategory::Prestige,
        AchievementCategory::Progression,
        AchievementCategory::Challenges,
        AchievementCategory::Exploration,
        AchievementCategory::Deep,
        AchievementCategory::Loom,
        AchievementCategory::Stats,
    ];

    /// Display name for the category.
    pub fn name(&self) -> &'static str {
        match self {
            AchievementCategory::Combat => "Combat",
            AchievementCategory::Level => "Level",
            AchievementCategory::Prestige => "Prestige",
            AchievementCategory::Progression => "Progression",
            AchievementCategory::Challenges => "Challenges",
            AchievementCategory::Exploration => "Exploration",
            AchievementCategory::Deep => "The Deep",
            AchievementCategory::Loom => "Loom",
            AchievementCategory::Stats => "Stats",
        }
    }
}

/// Unique identifier for each achievement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementId {
    // Combat achievements - enemy kills
    SlayerI,    // 100 kills
    SlayerII,   // 500 kills
    SlayerIII,  // 1,000 kills
    SlayerIV,   // 5,000 kills
    SlayerV,    // 10,000 kills
    SlayerVI,   // 50,000 kills
    SlayerVII,  // 100,000 kills
    SlayerVIII, // 500,000 kills
    SlayerIX,   // 1,000,000 kills
    SlayerX,    // 2,500,000 kills
    SlayerXI,   // 10,000,000 kills
    SlayerXII,  // 50,000,000 kills
    SlayerXIII, // 100,000,000 kills (Harbinger)
    SlayerXIV,  // 500,000,000 kills (Reaper)
    SlayerXV,   // 1,000,000,000 kills (Death Incarnate)
    // Combat achievements - boss kills
    BossHunterI,    // 1 boss
    BossHunterII,   // 10 bosses
    BossHunterIII,  // 50 bosses
    BossHunterIV,   // 100 bosses
    BossHunterV,    // 500 bosses
    BossHunterVI,   // 1,000 bosses
    BossHunterVII,  // 5,000 bosses
    BossHunterVIII, // 10,000 bosses
    BossHunterIX,   // 25,000 bosses
    BossHunterX,    // 75,000 bosses
    BossHunterXI,   // 250,000 bosses
    BossHunterXII,  // 750,000 bosses
    BossHunterXIII, // 2,500,000 bosses (Titan Breaker)
    BossHunterXIV,  // 5,000,000 bosses (Worldender)
    BossHunterXV,   // 10,000,000 bosses (The Absolute)

    // Level achievements
    Level10,
    Level25,
    Level50,
    Level100,
    Level150,
    Level200,
    Level250,
    Level500,
    Level750,
    Level1000,
    Level1500,
    Level2000,
    Level3000,
    Level5000,
    Level7500,
    Level10000,
    Level20000,
    Level100000,

    // Prestige achievements
    FirstPrestige,
    PrestigeV,
    PrestigeX,
    PrestigeXV,
    PrestigeXX,
    PrestigeXXV,
    PrestigeXXX,
    PrestigeXL,
    PrestigeL,
    PrestigeLXX,
    PrestigeXC,
    Eternal,
    Prestige150,
    Prestige200,
    Prestige300,
    Prestige500,
    Prestige700,
    Prestige1000,
    Prestige10000,
    // Zone completion achievements (one per zone)
    Zone1Complete,  // Meadow
    Zone2Complete,  // Dark Forest
    Zone3Complete,  // Mountain Pass
    Zone4Complete,  // Ancient Ruins
    Zone5Complete,  // Volcanic Wastes
    Zone6Complete,  // Frozen Tundra
    Zone7Complete,  // Crystal Caverns
    Zone8Complete,  // Sunken Kingdom
    Zone9Complete,  // Floating Isles
    Zone10Complete, // Storm Citadel
    TheStormbreaker,
    StormsEnd,
    // The Expanse achievement
    BeyondInfinity, // Complete a cycle of The Expanse

    // Fracture zone completion achievements (zones 12-20)
    FractureZone12, // Rimbreaker
    FractureZone13, // Cinderfall
    FractureZone14, // Heart Piercer
    FractureZone15, // Shard Breaker
    FractureZone16, // Light Bender
    FractureZone17, // Sunslayer
    FractureZone18, // Ashen Sentinel
    FractureZone19, // Throat Runner
    FractureZone20, // Maw Closer
    FractureZone21, // Amber March
    FractureZone22, // Pale Scholar
    FractureZone23, // Thronebreaker
    FractureZone24, // Stillwater
    FractureZone25, // Resonance Breaker
    FractureZone26, // Edge Walker
    FractureZone27, // Scar Render
    FractureZone28, // Echo Silencer
    FractureZone29, // Last Listener
    FractureZone30, // Scar Mender
    // Ascension milestone achievements (one per level, I-VI)
    AscensionI,    // First Ascension
    AscensionII,   // Twice Risen
    AscensionIII,  // Deepborn
    AscensionIV,   // Fourfold
    AscensionV,    // Quintessence
    AscensionVI,   // Transcendent
    AscensionVII,  // Loombound
    AscensionVIII, // Threadwarden
    AscensionIX,   // Worldweaver
    AscensionX,    // The Absolute

    // Challenge achievements - Chess
    ChessNovice,
    ChessApprentice,
    ChessJourneyman,
    ChessMaster,
    // Challenge achievements - Morris
    MorrisNovice,
    MorrisApprentice,
    MorrisJourneyman,
    MorrisMaster,
    // Challenge achievements - Gomoku
    GomokuNovice,
    GomokuApprentice,
    GomokuJourneyman,
    GomokuMaster,
    // Challenge achievements - Minesweeper
    MinesweeperNovice,
    MinesweeperApprentice,
    MinesweeperJourneyman,
    MinesweeperMaster,
    // Challenge achievements - Rune
    RuneNovice,
    RuneApprentice,
    RuneJourneyman,
    RuneMaster,
    // Challenge achievements - Go
    GoNovice,
    GoApprentice,
    GoJourneyman,
    GoMaster,
    // Challenge achievements - Flappy Bird
    FlappyNovice,
    FlappyApprentice,
    FlappyJourneyman,
    FlappyMaster,
    // Challenge achievements - Snake
    SnakeNovice,
    SnakeApprentice,
    SnakeJourneyman,
    SnakeMaster,
    // Challenge achievements - Containment Breach
    ContainmentBreachNovice,
    ContainmentBreachApprentice,
    ContainmentBreachJourneyman,
    ContainmentBreachMaster,
    // Challenge achievements - Sigil Surge
    SigilSurgeNovice,
    SigilSurgeApprentice,
    SigilSurgeJourneyman,
    SigilSurgeMaster,
    // Challenge achievements - Meta
    GrandChampion,

    // Fishing achievements - rank milestones
    GoneFishing,
    FishermanI,
    FishermanII,
    FishermanIII,
    FishermanIV,
    StormLeviathan,
    // Fishing achievements - catch counts
    FishCatcherI,    // 100 fish
    FishCatcherII,   // 1,000 fish
    FishCatcherIII,  // 10,000 fish
    FishCatcherIV,   // 100,000 fish
    FishCatcherV,    // 500,000 fish
    FishCatcherVI,   // 1,000,000 fish
    FishCatcherVII,  // 5,000,000 fish
    FishCatcherVIII, // 10,000,000 fish (Leviathan's Rival)
    FishCatcherIX,   // 50,000,000 fish (Poseidon's Hand)
    FishCatcherX,    // 100,000,000 fish (Lord of the Deep)

    // Dungeon achievements
    DungeonDiver,
    DungeonMasterI,
    DungeonMasterII,
    DungeonMasterIII,  // 100 dungeons
    DungeonMasterIV,   // 1,000 dungeons
    DungeonMasterV,    // 5,000 dungeons
    DungeonMasterVI,   // 10,000 dungeons
    DungeonMasterVII,  // 25,000 dungeons
    DungeonMasterVIII, // 100,000 dungeons (Labyrinth Lord)
    DungeonMasterIX,   // 500,000 dungeons (Abyss Walker)
    DungeonMasterX,    // 1,000,000 dungeons (The Undying Delver)

    // Haven achievements
    HavenDiscovered,
    HavenBuilderI,  // All rooms at T1
    HavenBuilderII, // All rooms at T2
    HavenArchitect, // All rooms at T3

    // Enhancement
    SoulforgeDiscovered,  // Discover the Soulforge
    ApprenticeSmith,      // Reach +1 on any slot
    FullyTempered,        // Reach +4 on all 7 slots
    JourneymanSmith,      // Reach +5 on any slot
    SoulforgeAdept,       // Reach +6 on any slot
    SoulforgeSavant,      // Reach +7 on any slot
    SoulforgeMaster,      // Reach +8 on any slot
    SoulforgeGrandmaster, // Reach +9 on any slot
    SoulforgeAscendant,   // Reach +10 on any slot
    SoulConvergence,      // Reach +7 on all 7 slots
    PersistentHammering,  // 100 total enhancement attempts

    // The Deep achievements
    TheDeepDiscovered,    // Discover The Deep
    FirstMissionComplete, // Complete first mission
    DeepMissionsX,        // Complete 10 missions
    DeepMissionsXXV,      // Complete 25 missions
    DeepMissionsL,        // Complete 50 missions
    DeepMissionsC,        // Complete 100 missions
    FirstBreakthrough,    // Complete first breakthrough mission
    Layer5Cleared,        // Reach Layer 5
    Layer10Cleared,       // Reach Layer 10
    Layer15Cleared,       // Reach Layer 15
    Layer20Cleared,       // Reach Layer 20
    Layer25Cleared,       // Reach Layer 25 (The Abyss)
    VoidExplorer,         // Reach Layer 26 (The Void)
    GuildRank2,           // Reach Guild Rank 2 (Company)
    GuildRank3,           // Reach Guild Rank 3 (Battalion)
    GuildRank4,           // Reach Guild Rank 4 (Legion)
    GuildRank5,           // Reach Guild Rank 5 (Vanguard)
    FirstMercLost,        // Lose a mercenary for the first time
    GatewayOpened,        // Opened the Gateway beneath the world
    // Loom of Worlds achievements
    LoomDiscovered, // Discover the Loom of Worlds
    LoomPattern1,   // Complete first Woven Pattern
    LoomPattern4,   // Complete 4 Woven Patterns (unlocks Z31-34)
    LoomPattern8,   // Complete 8 Woven Patterns (unlocks Z35-38)
    LoomPattern16,  // Complete 16 Woven Patterns (unlocks Z39-42)
    LoomPattern22,  // Complete 22 Woven Patterns (unlocks Z43-46)
    LoomPattern28,  // Complete all 28 Woven Patterns (unlocks Z47-50)
    // Power Cores — unlocked at fracture zone unlock layers
    PowerCoreI,   // Deep Layer 3 — Red Fault core
    PowerCoreII,  // Deep Layer 7 — Mirror Scar core
    PowerCoreIII, // Deep Layer 12 — Black Mouth core
    PowerCoreIV,  // Deep Layer 18 — Hollow Throne core
    PowerCoreV,   // Deep Layer 25 — Wailing Reach core
    PowerCoreVI,  // Deep Layer 30 — Origin Wound core
}

impl AchievementId {
    /// Total number of `AchievementId` variants.
    ///
    /// **Must be kept in sync with the enum.**  Update this whenever you add or
    /// remove a variant so that `test_every_achievement_id_variant_has_definition`
    /// in `data.rs` catches mismatches between the enum and `ALL_ACHIEVEMENTS`.
    ///
    /// Once `std::mem::variant_count` stabilises (tracking issue #73662) this
    /// constant can be replaced with a `const_assert!` that computes the count
    /// automatically.
    // Used by `achievements::data` tests to verify ALL_ACHIEVEMENTS coverage.
    #[allow(dead_code)]
    pub const VARIANT_COUNT: usize = 224;
}

/// Static definition of an achievement.
#[derive(Debug, Clone)]
pub struct AchievementDef {
    pub id: AchievementId,
    pub name: &'static str,
    pub description: &'static str,
    pub category: AchievementCategory,
    pub icon: &'static str,
    pub points: u32,
}

/// Progress on a single achievement (for multi-stage achievements).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AchievementProgress {
    pub current: u64,
    pub target: u64,
}

/// Record of an unlocked achievement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockedAchievement {
    pub unlocked_at: i64,
    pub character_name: Option<String>,
}

/// Global UI border style shared across all characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UiBorderStyle {
    #[default]
    #[serde(alias = "RuneCrawl")]
    #[serde(alias = "ImpactRipples")]
    #[serde(alias = "BossThreatMode")]
    #[serde(alias = "Quadrant")]
    #[serde(alias = "QuadrantInside")]
    #[serde(alias = "IronRivets")]
    #[serde(alias = "RoyalCrest")]
    #[serde(alias = "SentinelSpikes")]
    #[serde(alias = "ArcaneSeals")]
    #[serde(alias = "MeridianMarks")]
    #[serde(alias = "CathedralFrame")]
    #[serde(alias = "BastionBraces")]
    #[serde(alias = "CelestialPins")]
    #[serde(alias = "ForgeBolts")]
    #[serde(alias = "RelicWard")]
    Classic,
    #[serde(alias = "ElementalFrost")]
    Rounded,
    #[serde(alias = "ElementalStorm")]
    #[serde(alias = "DualLayerIllusion")]
    Double,
    #[serde(alias = "ElementalVoid")]
    Thick,
    Dashed,
    HeavyDashed,
    TripleDashed,
    HeavyTripleDashed,
    #[serde(alias = "NeonPulse")]
    QuadDashed,
    #[serde(alias = "ElementalEmber")]
    HeavyQuadDashed,
    HeavyCorner,
    MicroDash,
    HeaderRail,
}

pub const SELECTABLE_UI_BORDER_STYLES: &[UiBorderStyle] = &[
    UiBorderStyle::Classic,
    UiBorderStyle::Rounded,
    UiBorderStyle::Double,
    UiBorderStyle::Thick,
    UiBorderStyle::Dashed,
    UiBorderStyle::HeavyDashed,
    UiBorderStyle::TripleDashed,
    UiBorderStyle::HeavyTripleDashed,
    UiBorderStyle::QuadDashed,
    UiBorderStyle::HeavyQuadDashed,
    UiBorderStyle::HeavyCorner,
    UiBorderStyle::MicroDash,
    UiBorderStyle::HeaderRail,
];

impl UiBorderStyle {
    pub const fn storage_id(self) -> u8 {
        match self {
            Self::Classic => 0,
            Self::Rounded => 1,
            Self::Double => 2,
            Self::Thick => 3,
            Self::Dashed => 4,
            Self::HeavyDashed => 5,
            Self::TripleDashed => 6,
            Self::HeavyTripleDashed => 7,
            Self::QuadDashed => 8,
            Self::HeavyQuadDashed => 9,
            Self::HeavyCorner => 32,
            Self::MicroDash => 33,
            Self::HeaderRail => 35,
        }
    }

    pub const fn from_storage_id(id: u8) -> Self {
        match id {
            1 => Self::Rounded,
            2 => Self::Double,
            3 => Self::Thick,
            4 => Self::Dashed,
            5 => Self::HeavyDashed,
            6 => Self::TripleDashed,
            7 => Self::HeavyTripleDashed,
            8 => Self::QuadDashed,
            9 => Self::HeavyQuadDashed,
            32 => Self::HeavyCorner,
            33 => Self::MicroDash,
            // Removed style fallback.
            34 => Self::Classic,
            35 => Self::HeaderRail,
            // Backward-compat fallback for removed styles.
            10 => Self::Classic,
            11 => Self::Classic,
            12 => Self::Classic,
            13 => Self::Classic,
            14 => Self::Classic,
            15 => Self::Double,
            22 => Self::Classic,
            23 => Self::Classic,
            24 => Self::Classic,
            25 => Self::Classic,
            26 => Self::Classic,
            27 => Self::Classic,
            28 => Self::Classic,
            29 => Self::Classic,
            30 => Self::Classic,
            31 => Self::Classic,
            _ => Self::Classic,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Rounded => "Rounded",
            Self::Double => "Double",
            Self::Thick => "Thick",
            Self::Dashed => "Dashed",
            Self::HeavyDashed => "Heavy Dashed",
            Self::TripleDashed => "Triple Dashed",
            Self::HeavyTripleDashed => "Heavy Triple Dash",
            Self::QuadDashed => "Quad Dashed",
            Self::HeavyQuadDashed => "Heavy Quad Dash",
            Self::HeavyCorner => "Heavy Corner",
            Self::MicroDash => "Micro Dash",
            Self::HeaderRail => "Header Rail",
        }
    }

    pub const fn debug_option_label(self) -> &'static str {
        match self {
            Self::Classic => "Set Border: Classic",
            Self::Rounded => "Set Border: Rounded",
            Self::Double => "Set Border: Double",
            Self::Thick => "Set Border: Thick",
            Self::Dashed => "Set Border: Dashed",
            Self::HeavyDashed => "Set Border: Heavy Dashed",
            Self::TripleDashed => "Set Border: Triple Dashed",
            Self::HeavyTripleDashed => "Set Border: Heavy Triple Dash",
            Self::QuadDashed => "Set Border: Quad Dashed",
            Self::HeavyQuadDashed => "Set Border: Heavy Quad Dash",
            Self::HeavyCorner => "Set Border: Heavy Corner",
            Self::MicroDash => "Set Border: Micro Dash",
            Self::HeaderRail => "Set Border: Header Rail",
        }
    }

    pub const fn border_set_message(self) -> &'static str {
        match self {
            Self::Classic => "Border style set: Classic",
            Self::Rounded => "Border style set: Rounded",
            Self::Double => "Border style set: Double",
            Self::Thick => "Border style set: Thick",
            Self::Dashed => "Border style set: Dashed",
            Self::HeavyDashed => "Border style set: Heavy Dashed",
            Self::TripleDashed => "Border style set: Triple Dashed",
            Self::HeavyTripleDashed => "Border style set: Heavy Triple Dash",
            Self::QuadDashed => "Border style set: Quad Dashed",
            Self::HeavyQuadDashed => "Border style set: Heavy Quad Dash",
            Self::HeavyCorner => "Border style set: Heavy Corner",
            Self::MicroDash => "Border style set: Micro Dash",
            Self::HeaderRail => "Border style set: Header Rail",
        }
    }
}

/// Global achievement state (saved to disk).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Achievements {
    /// Map of unlocked achievements.
    pub unlocked: HashMap<AchievementId, UnlockedAchievement>,
    /// Progress tracking for multi-stage achievements.
    pub progress: HashMap<AchievementId, AchievementProgress>,

    // Aggregate tracking across all characters
    pub total_kills: u64,
    pub total_bosses_defeated: u64,
    pub total_fish_caught: u64,
    pub total_dungeons_completed: u64,
    pub total_minigame_wins: u64,
    pub highest_prestige_rank: u32,
    pub highest_level: u32,
    pub highest_fishing_rank: u32,
    pub zones_fully_cleared: u32,
    pub expanse_cycles_completed: u64,
    #[serde(default)]
    pub total_deep_missions_completed: u64,
    #[serde(default)]
    pub highest_deep_layer: u32,
    #[serde(default)]
    pub highest_guild_rank: u32,
    /// Global border style applied to panel UI.
    #[serde(default)]
    pub ui_border_style: UiBorderStyle,

    /// Currently selected title (account-wide).
    #[serde(default)]
    pub selected_title: Option<AchievementId>,

    /// Achievements unlocked but not yet viewed (not persisted) - for UI indicator
    #[serde(skip)]
    pub pending_notifications: Vec<AchievementId>,

    /// Achievements unlocked this tick that need to be logged (not persisted)
    #[serde(skip)]
    pub newly_unlocked: Vec<AchievementId>,

    /// Achievements waiting to be shown in modal (accumulation window)
    #[serde(skip)]
    pub modal_queue: Vec<AchievementId>,

    /// Achievements recently unlocked — visible as "NEW" badges in browser (not persisted)
    #[serde(skip)]
    pub recently_unlocked: Vec<AchievementId>,

    /// When the accumulation window started (first achievement unlocked)
    #[serde(skip)]
    pub accumulation_start: Option<std::time::Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_unlock() {
        let mut achievements = Achievements::default();

        assert!(!achievements.is_unlocked(AchievementId::SlayerI));
        assert!(achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string())));
        assert!(achievements.is_unlocked(AchievementId::SlayerI));

        // Second unlock should return false
        assert!(!achievements.unlock(AchievementId::SlayerI, None));
    }

    #[test]
    fn test_achievement_progress() {
        let mut achievements = Achievements::default();

        achievements.update_progress(AchievementId::SlayerI, 50, 100);

        let progress = achievements.get_progress(AchievementId::SlayerI).unwrap();
        assert_eq!(progress.current, 50);
        assert_eq!(progress.target, 100);
    }

    #[test]
    fn test_category_names() {
        assert_eq!(AchievementCategory::Combat.name(), "Combat");
        assert_eq!(AchievementCategory::Level.name(), "Level");
        assert_eq!(AchievementCategory::Prestige.name(), "Prestige");
        assert_eq!(AchievementCategory::Progression.name(), "Progression");
        assert_eq!(AchievementCategory::Challenges.name(), "Challenges");
        assert_eq!(AchievementCategory::Exploration.name(), "Exploration");
        assert_eq!(AchievementCategory::Deep.name(), "The Deep");
        assert_eq!(AchievementCategory::Loom.name(), "Loom");
        assert_eq!(AchievementCategory::Stats.name(), "Stats");
    }

    #[test]
    fn test_category_order_includes_prestige_after_level() {
        assert_eq!(
            AchievementCategory::ALL,
            [
                AchievementCategory::Combat,
                AchievementCategory::Level,
                AchievementCategory::Prestige,
                AchievementCategory::Progression,
                AchievementCategory::Challenges,
                AchievementCategory::Exploration,
                AchievementCategory::Deep,
                AchievementCategory::Loom,
                AchievementCategory::Stats,
            ]
        );
    }

    #[test]
    fn test_ui_border_style_defaults_to_classic() {
        let achievements = Achievements::default();
        assert_eq!(achievements.ui_border_style, UiBorderStyle::Classic);
    }

    #[test]
    fn test_ui_border_style_storage_roundtrip_for_selectable_styles() {
        for style in SELECTABLE_UI_BORDER_STYLES {
            let id = style.storage_id();
            assert_eq!(UiBorderStyle::from_storage_id(id), *style);
        }
    }

    #[test]
    fn test_ui_border_style_legacy_aliases_deserialize() {
        let legacy_neon: UiBorderStyle = serde_json::from_str("\"NeonPulse\"").unwrap();
        assert_eq!(legacy_neon, UiBorderStyle::QuadDashed);

        let legacy_ember: UiBorderStyle = serde_json::from_str("\"ElementalEmber\"").unwrap();
        assert_eq!(legacy_ember, UiBorderStyle::HeavyQuadDashed);

        let legacy_frost: UiBorderStyle = serde_json::from_str("\"ElementalFrost\"").unwrap();
        assert_eq!(legacy_frost, UiBorderStyle::Rounded);

        let legacy_rune_crawl: UiBorderStyle = serde_json::from_str("\"RuneCrawl\"").unwrap();
        assert_eq!(legacy_rune_crawl, UiBorderStyle::Classic);

        let legacy_impact_ripples: UiBorderStyle =
            serde_json::from_str("\"ImpactRipples\"").unwrap();
        assert_eq!(legacy_impact_ripples, UiBorderStyle::Classic);

        let legacy_boss_threat: UiBorderStyle = serde_json::from_str("\"BossThreatMode\"").unwrap();
        assert_eq!(legacy_boss_threat, UiBorderStyle::Classic);

        let legacy_quadrant: UiBorderStyle = serde_json::from_str("\"Quadrant\"").unwrap();
        assert_eq!(legacy_quadrant, UiBorderStyle::Classic);

        let legacy_relic_ward: UiBorderStyle = serde_json::from_str("\"RelicWard\"").unwrap();
        assert_eq!(legacy_relic_ward, UiBorderStyle::Classic);

        let legacy_dual_layer: UiBorderStyle =
            serde_json::from_str("\"DualLayerIllusion\"").unwrap();
        assert_eq!(legacy_dual_layer, UiBorderStyle::Double);
    }

    // =========================================================================
    // Slayer Achievement Tests
    // =========================================================================

    #[test]
    fn test_slayer_achievements_milestones() {
        let mut achievements = Achievements::default();

        // Kill 99 enemies - no slayer achievement yet
        for _ in 0..99 {
            achievements.on_enemy_killed(false, Some("Hero"));
        }
        assert!(!achievements.is_unlocked(AchievementId::SlayerI));

        // 100th kill unlocks SlayerI
        achievements.on_enemy_killed(false, Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::SlayerI));
        assert!(!achievements.is_unlocked(AchievementId::SlayerII));

        // Reach 500 kills for SlayerII
        for _ in 0..400 {
            achievements.on_enemy_killed(false, Some("Hero"));
        }
        assert!(achievements.is_unlocked(AchievementId::SlayerII));
        assert!(!achievements.is_unlocked(AchievementId::SlayerIII));

        // Reach 1000 kills for SlayerIII
        for _ in 0..500 {
            achievements.on_enemy_killed(false, Some("Hero"));
        }
        assert!(achievements.is_unlocked(AchievementId::SlayerIII));
    }

    #[test]
    fn test_slayer_all_milestones() {
        let mut achievements = Achievements::default();

        // Set total_kills directly to test all milestones
        let milestones = [
            (100, AchievementId::SlayerI),
            (500, AchievementId::SlayerII),
            (1000, AchievementId::SlayerIII),
            (5000, AchievementId::SlayerIV),
            (10000, AchievementId::SlayerV),
            (50000, AchievementId::SlayerVI),
            (100000, AchievementId::SlayerVII),
            (500000, AchievementId::SlayerVIII),
            (1000000, AchievementId::SlayerIX),
        ];

        for (kills, achievement_id) in milestones {
            achievements.total_kills = kills - 1;
            achievements.on_enemy_killed(false, Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Expected {:?} to be unlocked at {} kills",
                achievement_id,
                kills
            );
        }
    }

    // =========================================================================
    // Boss Hunter Achievement Tests
    // =========================================================================

    #[test]
    fn test_boss_hunter_achievements_milestones() {
        let mut achievements = Achievements::default();

        // First boss unlocks BossHunterI
        assert!(!achievements.is_unlocked(AchievementId::BossHunterI));
        achievements.on_enemy_killed(true, Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::BossHunterI));
        assert!(!achievements.is_unlocked(AchievementId::BossHunterII));

        // 9 more bosses (10 total) unlocks BossHunterII
        for _ in 0..9 {
            achievements.on_enemy_killed(true, Some("Hero"));
        }
        assert!(achievements.is_unlocked(AchievementId::BossHunterII));
        assert!(!achievements.is_unlocked(AchievementId::BossHunterIII));

        // 40 more bosses (50 total) unlocks BossHunterIII
        for _ in 0..40 {
            achievements.on_enemy_killed(true, Some("Hero"));
        }
        assert!(achievements.is_unlocked(AchievementId::BossHunterIII));
    }

    #[test]
    fn test_boss_hunter_all_milestones() {
        let mut achievements = Achievements::default();

        let milestones = [
            (1, AchievementId::BossHunterI),
            (10, AchievementId::BossHunterII),
            (50, AchievementId::BossHunterIII),
            (100, AchievementId::BossHunterIV),
            (500, AchievementId::BossHunterV),
            (1000, AchievementId::BossHunterVI),
            (5000, AchievementId::BossHunterVII),
            (10000, AchievementId::BossHunterVIII),
        ];

        for (bosses, achievement_id) in milestones {
            achievements.total_bosses_defeated = bosses - 1;
            achievements.on_enemy_killed(true, Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Expected {:?} to be unlocked at {} bosses",
                achievement_id,
                bosses
            );
        }
    }

    // =========================================================================
    // Expanse Cycle Achievement Tests
    // =========================================================================

    #[test]
    fn test_expanse_achievement_on_first_completion() {
        let mut achievements = Achievements::default();

        assert!(!achievements.is_unlocked(AchievementId::BeyondInfinity));
        assert_eq!(achievements.expanse_cycles_completed, 0);

        // Complete first cycle of The Expanse (zone 11)
        achievements.on_zone_fully_cleared(11, Some("Hero"));

        assert!(achievements.is_unlocked(AchievementId::BeyondInfinity));
        assert_eq!(achievements.expanse_cycles_completed, 1);
    }

    #[test]
    fn test_expanse_does_not_affect_other_zones() {
        let mut achievements = Achievements::default();

        // Completing zone 11 should not unlock zone completion achievements for zones 1-10
        achievements.on_zone_fully_cleared(11, Some("Hero"));

        assert!(!achievements.is_unlocked(AchievementId::Zone1Complete));
        assert!(!achievements.is_unlocked(AchievementId::Zone10Complete));
        assert!(achievements.is_unlocked(AchievementId::BeyondInfinity));
    }

    // =========================================================================
    // Zone Completion Achievement Tests
    // =========================================================================

    #[test]
    fn test_zone_completion_achievements() {
        let mut achievements = Achievements::default();

        let zones = [
            (1, AchievementId::Zone1Complete),
            (2, AchievementId::Zone2Complete),
            (3, AchievementId::Zone3Complete),
            (4, AchievementId::Zone4Complete),
            (5, AchievementId::Zone5Complete),
            (6, AchievementId::Zone6Complete),
            (7, AchievementId::Zone7Complete),
            (8, AchievementId::Zone8Complete),
            (9, AchievementId::Zone9Complete),
            (10, AchievementId::Zone10Complete),
        ];

        for (zone_id, achievement_id) in zones {
            assert!(
                !achievements.is_unlocked(achievement_id),
                "Zone {} should not be unlocked initially",
                zone_id
            );
            achievements.on_zone_fully_cleared(zone_id, Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Zone {} should be unlocked after clearing",
                zone_id
            );
        }
    }

    // =========================================================================
    // Fish Catcher Achievement Tests
    // =========================================================================

    #[test]
    fn test_fish_catcher_achievements() {
        let mut achievements = Achievements::default();

        // First fish unlocks GoneFishing
        achievements.on_fish_caught(Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::GoneFishing));
        assert!(!achievements.is_unlocked(AchievementId::FishCatcherI));

        // 99 more fish (100 total) unlocks FishCatcherI
        for _ in 0..99 {
            achievements.on_fish_caught(Some("Hero"));
        }
        assert!(achievements.is_unlocked(AchievementId::FishCatcherI));
        assert!(!achievements.is_unlocked(AchievementId::FishCatcherII));
    }

    #[test]
    fn test_fish_catcher_all_milestones() {
        let mut achievements = Achievements::default();

        let milestones = [
            (1, AchievementId::GoneFishing),
            (100, AchievementId::FishCatcherI),
            (1000, AchievementId::FishCatcherII),
            (10000, AchievementId::FishCatcherIII),
            (100000, AchievementId::FishCatcherIV),
            (500000, AchievementId::FishCatcherV),
            (1000000, AchievementId::FishCatcherVI),
            (5000000, AchievementId::FishCatcherVII),
            (10000000, AchievementId::FishCatcherVIII),
            (50000000, AchievementId::FishCatcherIX),
            (100000000, AchievementId::FishCatcherX),
        ];

        for (fish, achievement_id) in milestones {
            achievements.total_fish_caught = fish - 1;
            achievements.on_fish_caught(Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Expected {:?} to be unlocked at {} fish",
                achievement_id,
                fish
            );
        }
    }

    // =========================================================================
    // Dungeon Master Achievement Tests
    // =========================================================================

    #[test]
    fn test_dungeon_master_achievements() {
        let mut achievements = Achievements::default();

        // First dungeon unlocks DungeonDiver
        achievements.on_dungeon_completed(Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::DungeonDiver));
        assert!(!achievements.is_unlocked(AchievementId::DungeonMasterI));

        // 9 more dungeons (10 total) unlocks DungeonMasterI
        for _ in 0..9 {
            achievements.on_dungeon_completed(Some("Hero"));
        }
        assert!(achievements.is_unlocked(AchievementId::DungeonMasterI));
        assert!(!achievements.is_unlocked(AchievementId::DungeonMasterII));
    }

    #[test]
    fn test_dungeon_master_all_milestones() {
        let mut achievements = Achievements::default();

        let milestones = [
            (1, AchievementId::DungeonDiver),
            (10, AchievementId::DungeonMasterI),
            (50, AchievementId::DungeonMasterII),
            (100, AchievementId::DungeonMasterIII),
            (1000, AchievementId::DungeonMasterIV),
            (5000, AchievementId::DungeonMasterV),
            (10000, AchievementId::DungeonMasterVI),
            (25000, AchievementId::DungeonMasterVII),
            (100000, AchievementId::DungeonMasterVIII),
            (500000, AchievementId::DungeonMasterIX),
            (1000000, AchievementId::DungeonMasterX),
        ];

        for (dungeons, achievement_id) in milestones {
            achievements.total_dungeons_completed = dungeons - 1;
            achievements.on_dungeon_completed(Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Expected {:?} to be unlocked at {} dungeons",
                achievement_id,
                dungeons
            );
        }
    }

    // =========================================================================
    // Haven Achievement Tests
    // =========================================================================

    #[test]
    fn test_haven_achievements() {
        let mut achievements = Achievements::default();

        // Haven discovered
        assert!(!achievements.is_unlocked(AchievementId::HavenDiscovered));
        achievements.on_haven_discovered(Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::HavenDiscovered));

        // Haven builder tiers
        assert!(!achievements.is_unlocked(AchievementId::HavenBuilderI));
        achievements.on_haven_all_t1(Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::HavenBuilderI));

        assert!(!achievements.is_unlocked(AchievementId::HavenBuilderII));
        achievements.on_haven_all_t2(Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::HavenBuilderII));

        assert!(!achievements.is_unlocked(AchievementId::HavenArchitect));
        achievements.on_haven_architect(Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::HavenArchitect));
    }

    // =========================================================================
    // Level Achievement Tests
    // =========================================================================

    #[test]
    fn test_level_achievements() {
        let mut achievements = Achievements::default();

        let milestones = [
            (10, AchievementId::Level10),
            (25, AchievementId::Level25),
            (50, AchievementId::Level50),
            (100, AchievementId::Level100),
            (150, AchievementId::Level150),
            (200, AchievementId::Level200),
            (250, AchievementId::Level250),
            (500, AchievementId::Level500),
            (750, AchievementId::Level750),
            (1000, AchievementId::Level1000),
            (1500, AchievementId::Level1500),
            (2000, AchievementId::Level2000),
            (3000, AchievementId::Level3000),
            (5000, AchievementId::Level5000),
            (7500, AchievementId::Level7500),
            (10000, AchievementId::Level10000),
            (20000, AchievementId::Level20000),
            (100000, AchievementId::Level100000),
        ];

        for (level, achievement_id) in milestones {
            achievements.on_level_up(level, Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Expected {:?} to be unlocked at level {}",
                achievement_id,
                level
            );
        }
    }

    // =========================================================================
    // Prestige Achievement Tests
    // =========================================================================

    #[test]
    fn test_prestige_achievements() {
        let mut achievements = Achievements::default();

        let milestones = [
            (1, AchievementId::FirstPrestige),
            (5, AchievementId::PrestigeV),
            (10, AchievementId::PrestigeX),
            (15, AchievementId::PrestigeXV),
            (20, AchievementId::PrestigeXX),
            (25, AchievementId::PrestigeXXV),
            (30, AchievementId::PrestigeXXX),
            (40, AchievementId::PrestigeXL),
            (50, AchievementId::PrestigeL),
            (70, AchievementId::PrestigeLXX),
            (90, AchievementId::PrestigeXC),
            (100, AchievementId::Eternal),
            (150, AchievementId::Prestige150),
            (200, AchievementId::Prestige200),
            (300, AchievementId::Prestige300),
            (500, AchievementId::Prestige500),
            (700, AchievementId::Prestige700),
            (1000, AchievementId::Prestige1000),
            (10000, AchievementId::Prestige10000),
        ];

        for (rank, achievement_id) in milestones {
            achievements.on_prestige(rank, Some("Hero"));
            assert!(
                achievements.is_unlocked(achievement_id),
                "Expected {:?} to be unlocked at prestige rank {}",
                achievement_id,
                rank
            );
        }
    }

    // =========================================================================
    // Storms End Achievement Test
    // =========================================================================

    #[test]
    fn test_storms_end_achievement() {
        let mut achievements = Achievements::default();

        assert!(!achievements.is_unlocked(AchievementId::StormsEnd));
        achievements.on_storms_end(Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::StormsEnd));
    }

    // =========================================================================
    // State Synchronization Tests
    // =========================================================================

    #[test]
    fn test_sync_from_game_state_level_achievements() {
        let mut achievements = Achievements::default();

        // Sync with level 120 character
        achievements.sync_from_game_state(120, 0, 1, 0, &[], Some("Hero"));

        // Should have all level achievements up to 100
        assert!(achievements.is_unlocked(AchievementId::Level10));
        assert!(achievements.is_unlocked(AchievementId::Level25));
        assert!(achievements.is_unlocked(AchievementId::Level50));
        assert!(achievements.is_unlocked(AchievementId::Level100));
        // But not 150+
        assert!(!achievements.is_unlocked(AchievementId::Level150));
    }

    #[test]
    fn test_sync_from_game_state_prestige_achievements() {
        let mut achievements = Achievements::default();

        // Sync with prestige 17 character
        achievements.sync_from_game_state(1, 17, 1, 0, &[], Some("Hero"));

        // Should have prestige achievements up to P15
        assert!(achievements.is_unlocked(AchievementId::FirstPrestige));
        assert!(achievements.is_unlocked(AchievementId::PrestigeV));
        assert!(achievements.is_unlocked(AchievementId::PrestigeX));
        assert!(achievements.is_unlocked(AchievementId::PrestigeXV));
        // But not P20+
        assert!(!achievements.is_unlocked(AchievementId::PrestigeXX));
    }

    #[test]
    fn test_sync_from_game_state_new_endgame_achievements() {
        let mut achievements = Achievements::default();

        achievements.sync_from_game_state(100000, 10000, 1, 0, &[], Some("Hero"));

        assert!(achievements.is_unlocked(AchievementId::Level100000));
        assert!(achievements.is_unlocked(AchievementId::Prestige10000));
    }

    #[test]
    fn test_sync_from_game_state_fishing_achievements() {
        let mut achievements = Achievements::default();

        // Sync with fishing rank 15
        achievements.sync_from_game_state(1, 0, 15, 500, &[], Some("Hero"));

        // Should have FishermanI (rank 10)
        assert!(achievements.is_unlocked(AchievementId::FishermanI));
        // But not FishermanII (rank 20)
        assert!(!achievements.is_unlocked(AchievementId::FishermanII));

        // Should have fish catch achievements
        assert!(achievements.is_unlocked(AchievementId::GoneFishing));
        assert!(achievements.is_unlocked(AchievementId::FishCatcherI)); // 100 fish
        assert!(!achievements.is_unlocked(AchievementId::FishCatcherII)); // 1000 fish
    }

    #[test]
    fn test_sync_from_game_state_zone_completions() {
        let mut achievements = Achievements::default();

        // Zone 1 has 3 subzones, Zone 2 has 3 subzones
        let defeated_bosses = vec![
            (1, 1),
            (1, 2),
            (1, 3), // Zone 1 complete
            (2, 1),
            (2, 2), // Zone 2 incomplete (missing subzone 3)
        ];

        achievements.sync_from_game_state(1, 0, 1, 0, &defeated_bosses, Some("Hero"));

        assert!(achievements.is_unlocked(AchievementId::Zone1Complete));
        assert!(!achievements.is_unlocked(AchievementId::Zone2Complete));
    }

    #[test]
    fn test_sync_from_game_state_full_progression() {
        let mut achievements = Achievements::default();

        // Simulate a well-progressed character
        let defeated_bosses = vec![
            // Zone 1-4 complete
            (1, 1),
            (1, 2),
            (1, 3),
            (2, 1),
            (2, 2),
            (2, 3),
            (3, 1),
            (3, 2),
            (3, 3),
            (4, 1),
            (4, 2),
            (4, 3),
        ];

        achievements.sync_from_game_state(
            150,  // level
            25,   // prestige
            20,   // fishing rank
            5000, // fish caught
            &defeated_bosses,
            Some("Veteran"),
        );

        // Level achievements
        assert!(achievements.is_unlocked(AchievementId::Level100));
        assert!(achievements.is_unlocked(AchievementId::Level150));
        assert!(!achievements.is_unlocked(AchievementId::Level200));

        // Prestige achievements
        assert!(achievements.is_unlocked(AchievementId::PrestigeXX));
        assert!(achievements.is_unlocked(AchievementId::PrestigeXXV));
        assert!(!achievements.is_unlocked(AchievementId::PrestigeXXX));

        // Fishing achievements
        assert!(achievements.is_unlocked(AchievementId::FishermanI));
        assert!(achievements.is_unlocked(AchievementId::FishermanII));
        assert!(!achievements.is_unlocked(AchievementId::FishermanIII)); // needs rank 30
        assert!(!achievements.is_unlocked(AchievementId::FishermanIV)); // needs rank 40

        // Fish catch achievements (5000 fish)
        assert!(achievements.is_unlocked(AchievementId::FishCatcherI)); // 100
        assert!(achievements.is_unlocked(AchievementId::FishCatcherII)); // 1000
        assert!(!achievements.is_unlocked(AchievementId::FishCatcherIII)); // 10000 - not reached

        // Zone completions
        assert!(achievements.is_unlocked(AchievementId::Zone1Complete));
        assert!(achievements.is_unlocked(AchievementId::Zone2Complete));
        assert!(achievements.is_unlocked(AchievementId::Zone3Complete));
        assert!(achievements.is_unlocked(AchievementId::Zone4Complete));
        assert!(!achievements.is_unlocked(AchievementId::Zone5Complete));
    }

    #[test]
    fn test_sync_does_not_overwrite_higher_counters() {
        // Pre-set a higher fish count in achievements
        let mut achievements = Achievements {
            total_fish_caught: 50000,
            ..Default::default()
        };

        // Sync with a lower fish count from save
        achievements.sync_from_game_state(1, 0, 1, 1000, &[], Some("Hero"));

        // Should NOT have decreased the counter
        assert_eq!(achievements.total_fish_caught, 50000);
        // Should still have the high-count achievements
        assert!(achievements.is_unlocked(AchievementId::FishCatcherIII)); // 10000
    }

    // =========================================================================
    // Storm Leviathan Achievement Tests
    // =========================================================================

    #[test]
    fn test_storm_leviathan_unlocking() {
        let mut achievements = Achievements::default();

        // Storm Leviathan should not be unlocked initially
        assert!(!achievements.is_unlocked(AchievementId::StormLeviathan));

        // Call the event handler for catching Storm Leviathan
        achievements.on_storm_leviathan_caught(Some("Hero"));

        // Should now be unlocked
        assert!(achievements.is_unlocked(AchievementId::StormLeviathan));
    }

    #[test]
    fn test_storm_leviathan_only_unlocks_once() {
        let mut achievements = Achievements::default();

        // First catch unlocks the achievement
        assert!(achievements.unlock(AchievementId::StormLeviathan, Some("Hero".to_string())));

        // Second catch should not unlock again
        assert!(!achievements.unlock(AchievementId::StormLeviathan, None));
    }

    // =========================================================================
    // TheStormbreaker Achievement Tests
    // =========================================================================

    #[test]
    fn test_stormbreaker_can_be_unlocked() {
        let mut achievements = Achievements::default();

        // TheStormbreaker should not be unlocked initially
        assert!(!achievements.is_unlocked(AchievementId::TheStormbreaker));

        // Unlock TheStormbreaker (simulating forge)
        achievements.unlock(AchievementId::TheStormbreaker, Some("Hero".to_string()));

        // Should now be unlocked
        assert!(achievements.is_unlocked(AchievementId::TheStormbreaker));
    }

    #[test]
    fn test_stormbreaker_unlocks_independently_of_leviathan() {
        let mut achievements = Achievements::default();

        // Can unlock TheStormbreaker without Storm Leviathan (test only - game logic prevents this)
        achievements.unlock(AchievementId::TheStormbreaker, Some("Hero".to_string()));

        assert!(achievements.is_unlocked(AchievementId::TheStormbreaker));
        assert!(!achievements.is_unlocked(AchievementId::StormLeviathan));
    }

    // =========================================================================
    // Haven Sync Achievement Tests
    // =========================================================================

    /// Build a HashMap of Haven room tiers for testing.
    /// Sets all buildable rooms (excluding StormForge) to the given tier.
    fn build_haven_tiers(tier: u8) -> HashMap<crate::haven::types::HavenRoomId, u8> {
        use crate::haven::types::HavenRoomId;
        HavenRoomId::ALL
            .iter()
            .filter(|r| **r != HavenRoomId::StormForge)
            .map(|r| (*r, tier))
            .collect()
    }

    #[test]
    fn test_haven_sync_discovered() {
        let mut achievements = Achievements::default();
        let room_tiers = HashMap::new();

        achievements.sync_from_haven(true, &room_tiers, Some("Hero"));

        assert!(achievements.is_unlocked(AchievementId::HavenDiscovered));
    }

    #[test]
    fn test_haven_sync_builder_i() {
        let mut achievements = Achievements::default();
        let room_tiers = build_haven_tiers(1);

        achievements.sync_from_haven(true, &room_tiers, Some("Hero"));

        assert!(achievements.is_unlocked(AchievementId::HavenDiscovered));
        assert!(achievements.is_unlocked(AchievementId::HavenBuilderI));
        assert!(!achievements.is_unlocked(AchievementId::HavenBuilderII));
    }

    #[test]
    fn test_haven_sync_builder_ii() {
        let mut achievements = Achievements::default();
        let room_tiers = build_haven_tiers(2);

        achievements.sync_from_haven(true, &room_tiers, Some("Hero"));

        assert!(achievements.is_unlocked(AchievementId::HavenBuilderI));
        assert!(achievements.is_unlocked(AchievementId::HavenBuilderII));
        assert!(!achievements.is_unlocked(AchievementId::HavenArchitect));
    }

    #[test]
    fn test_haven_sync_architect() {
        use crate::haven::types::HavenRoomId;
        let mut achievements = Achievements::default();
        let room_tiers: HashMap<HavenRoomId, u8> = HavenRoomId::ALL
            .iter()
            .map(|r| (*r, r.max_tier()))
            .collect();

        achievements.sync_from_haven(true, &room_tiers, Some("Hero"));

        assert!(achievements.is_unlocked(AchievementId::HavenBuilderI));
        assert!(achievements.is_unlocked(AchievementId::HavenBuilderII));
        assert!(achievements.is_unlocked(AchievementId::HavenArchitect));
    }

    // =========================================================================
    // Fishing Rank Achievement Tests
    // =========================================================================

    #[test]
    fn test_fishing_rank_milestones() {
        let mut achievements = Achievements::default();

        // Rank 9 — no achievements yet
        achievements.on_fishing_rank_up(9, Some("Hero"));
        assert!(!achievements.is_unlocked(AchievementId::FishermanI));

        // Rank 10 unlocks FishermanI
        achievements.on_fishing_rank_up(10, Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::FishermanI));
        assert!(!achievements.is_unlocked(AchievementId::FishermanII));

        // Rank 20 unlocks FishermanII
        achievements.on_fishing_rank_up(20, Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::FishermanII));
        assert!(!achievements.is_unlocked(AchievementId::FishermanIII));

        // Rank 30 unlocks FishermanIII
        achievements.on_fishing_rank_up(30, Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::FishermanIII));
        assert!(!achievements.is_unlocked(AchievementId::FishermanIV));

        // Rank 40 unlocks FishermanIV
        achievements.on_fishing_rank_up(40, Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::FishermanIV));
    }

    #[test]
    fn test_fishing_rank_tracks_highest() {
        let mut achievements = Achievements::default();

        achievements.on_fishing_rank_up(15, Some("Hero"));
        assert_eq!(achievements.highest_fishing_rank, 15);

        // Lower rank should not decrease the highest
        achievements.on_fishing_rank_up(10, Some("Hero"));
        assert_eq!(achievements.highest_fishing_rank, 15);

        // Higher rank should update
        achievements.on_fishing_rank_up(25, Some("Hero"));
        assert_eq!(achievements.highest_fishing_rank, 25);
    }

    // =========================================================================
    // Count by Category Tests
    // =========================================================================

    #[test]
    fn test_count_by_category_empty() {
        let achievements = Achievements::default();
        let (unlocked, total) = achievements.count_by_category(AchievementCategory::Combat);
        assert_eq!(unlocked, 0);
        assert!(total > 0);
    }

    #[test]
    fn test_count_by_category_partial_unlock() {
        let mut achievements = Achievements {
            total_kills: 99,
            ..Default::default()
        };

        // Unlock some combat achievements
        achievements.on_enemy_killed(false, Some("Hero")); // 100 kills → SlayerI
        achievements.on_enemy_killed(true, Some("Hero")); // 1 boss → BossHunterI

        let (unlocked, total) = achievements.count_by_category(AchievementCategory::Combat);
        assert_eq!(unlocked, 2); // SlayerI + BossHunterI
        assert!(total > 2);

        // Other categories unaffected
        let (level_unlocked, _) = achievements.count_by_category(AchievementCategory::Level);
        assert_eq!(level_unlocked, 0);
        let (prestige_unlocked, _) = achievements.count_by_category(AchievementCategory::Prestige);
        assert_eq!(prestige_unlocked, 0);
    }

    // =========================================================================
    // Recently Unlocked Tests
    // =========================================================================

    #[test]
    fn test_clear_pending_moves_to_recently_unlocked() {
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
        achievements.unlock(AchievementId::BossHunterI, Some("Hero".to_string()));

        assert_eq!(achievements.pending_count(), 2);
        assert!(achievements.recently_unlocked.is_empty());

        achievements.clear_pending_notifications();

        assert_eq!(achievements.pending_count(), 0);
        assert_eq!(achievements.recently_unlocked.len(), 2);
        assert!(achievements.is_recently_unlocked(AchievementId::SlayerI));
        assert!(achievements.is_recently_unlocked(AchievementId::BossHunterI));
    }

    #[test]
    fn test_clear_recently_unlocked() {
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
        achievements.clear_pending_notifications();

        assert!(!achievements.recently_unlocked.is_empty());

        achievements.clear_recently_unlocked();
        assert!(achievements.recently_unlocked.is_empty());
        assert!(!achievements.is_recently_unlocked(AchievementId::SlayerI));
    }

    #[test]
    fn test_count_recently_unlocked_by_category() {
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
        achievements.unlock(AchievementId::BossHunterI, Some("Hero".to_string()));
        achievements.unlock(AchievementId::Level10, Some("Hero".to_string()));
        achievements.unlock(AchievementId::FirstPrestige, Some("Hero".to_string()));

        achievements.clear_pending_notifications();

        assert_eq!(
            achievements.count_recently_unlocked_by_category(AchievementCategory::Combat),
            2
        );
        assert_eq!(
            achievements.count_recently_unlocked_by_category(AchievementCategory::Level),
            1
        );
        assert_eq!(
            achievements.count_recently_unlocked_by_category(AchievementCategory::Prestige),
            1
        );
        assert_eq!(
            achievements.count_recently_unlocked_by_category(AchievementCategory::Progression),
            0
        );
    }

    // =========================================================================
    // Ascension Achievement Tests
    // =========================================================================

    #[test]
    fn test_on_ascended_unlocks_correct_achievement() {
        let mut achievements = Achievements::default();

        achievements.on_ascended(1, Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::AscensionI));
        assert!(!achievements.is_unlocked(AchievementId::AscensionII));

        achievements.on_ascended(2, Some("Hero"));
        assert!(achievements.is_unlocked(AchievementId::AscensionII));
        assert!(!achievements.is_unlocked(AchievementId::AscensionIII));
    }

    #[test]
    fn test_on_ascended_all_levels() {
        let mut achievements = Achievements::default();

        for level in 1..=6 {
            achievements.on_ascended(level, Some("Hero"));
        }

        assert!(achievements.is_unlocked(AchievementId::AscensionI));
        assert!(achievements.is_unlocked(AchievementId::AscensionII));
        assert!(achievements.is_unlocked(AchievementId::AscensionIII));
        assert!(achievements.is_unlocked(AchievementId::AscensionIV));
        assert!(achievements.is_unlocked(AchievementId::AscensionV));
        assert!(achievements.is_unlocked(AchievementId::AscensionVI));
    }

    #[test]
    fn test_on_ascended_beyond_vi_does_not_panic() {
        let mut achievements = Achievements::default();
        // Level 7+ should be a no-op, not panic
        achievements.on_ascended(7, Some("Hero"));
        achievements.on_ascended(100, Some("Hero"));
        assert!(!achievements.is_unlocked(AchievementId::AscensionVI));
    }

    #[test]
    fn test_sync_from_ascension_retroactive() {
        let mut achievements = Achievements::default();

        // Simulate loading a character at ascension level 3
        achievements.sync_from_ascension(3, Some("Hero"));

        assert!(achievements.is_unlocked(AchievementId::AscensionI));
        assert!(achievements.is_unlocked(AchievementId::AscensionII));
        assert!(achievements.is_unlocked(AchievementId::AscensionIII));
        assert!(!achievements.is_unlocked(AchievementId::AscensionIV));
    }

    #[test]
    fn test_sync_from_ascension_zero_unlocks_nothing() {
        let mut achievements = Achievements::default();

        achievements.sync_from_ascension(0, Some("Hero"));

        assert!(!achievements.is_unlocked(AchievementId::AscensionI));
    }

    #[test]
    fn test_recently_unlocked_not_serialized() {
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
        achievements.clear_pending_notifications();

        let json = serde_json::to_string(&achievements).unwrap();
        let loaded: Achievements = serde_json::from_str(&json).unwrap();

        assert!(loaded.recently_unlocked.is_empty());
    }
}
