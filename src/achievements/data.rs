//! Static achievement definitions.

use std::collections::HashMap;
use std::sync::LazyLock;

use super::types::{AchievementCategory, AchievementDef, AchievementId};

/// O(1) lookup cache: AchievementId -> &AchievementDef
static ACHIEVEMENT_BY_ID: LazyLock<HashMap<AchievementId, &'static AchievementDef>> =
    LazyLock::new(|| ALL_ACHIEVEMENTS.iter().map(|a| (a.id, a)).collect());

/// O(1) lookup cache: AchievementCategory -> Vec<&AchievementDef>
static ACHIEVEMENTS_BY_CATEGORY: LazyLock<
    HashMap<AchievementCategory, Vec<&'static AchievementDef>>,
> = LazyLock::new(|| {
    let mut map: HashMap<AchievementCategory, Vec<&'static AchievementDef>> = HashMap::new();
    for a in ALL_ACHIEVEMENTS {
        map.entry(a.category).or_default().push(a);
    }
    map
});

/// All achievement definitions in display order.
pub const ALL_ACHIEVEMENTS: &[AchievementDef] = &[
    // ═══════════════════════════════════════════════════════════════
    // COMBAT ACHIEVEMENTS - ENEMY KILLS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::SlayerI,
        name: "Slayer I",
        description: "Defeat 100 enemies",
        category: AchievementCategory::Combat,
        icon: "\u{1f5e1}\u{fe0f}",
        points: 5,
    },
    AchievementDef {
        id: AchievementId::SlayerII,
        name: "Slayer II",
        description: "Defeat 500 enemies",
        category: AchievementCategory::Combat,
        icon: "\u{2694}\u{fe0f}",
        points: 5,
    },
    AchievementDef {
        id: AchievementId::SlayerIII,
        name: "Slayer III",
        description: "Defeat 1,000 enemies",
        category: AchievementCategory::Combat,
        icon: "\u{1f3f9}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::SlayerIV,
        name: "Slayer IV",
        description: "Defeat 5,000 enemies",
        category: AchievementCategory::Combat,
        icon: "\u{1f6e1}\u{fe0f}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::SlayerV,
        name: "Slayer V",
        description: "Defeat 10,000 enemies",
        category: AchievementCategory::Combat,
        icon: "\u{1f480}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::SlayerVI,
        name: "Slayer VI",
        description: "Defeat 50,000 enemies",
        category: AchievementCategory::Combat,
        icon: "\u{2620}\u{fe0f}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::SlayerVII,
        name: "Slayer VII",
        description: "Defeat 100,000 enemies",
        category: AchievementCategory::Combat,
        icon: "\u{1f525}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::SlayerVIII,
        name: "Slayer VIII",
        description: "Defeat 500,000 enemies",
        category: AchievementCategory::Combat,
        icon: "\u{26a1}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::SlayerIX,
        name: "Slayer IX",
        description: "Defeat 1,000,000 enemies",
        category: AchievementCategory::Combat,
        icon: "\u{1f48e}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::SlayerX,
        name: "Slayer X",
        description: "Defeat 2,500,000 enemies",
        category: AchievementCategory::Combat,
        icon: "\u{269c}\u{fe0f}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::SlayerXI,
        name: "Slayer XI",
        description: "Defeat 10,000,000 enemies",
        category: AchievementCategory::Combat,
        icon: "\u{1f531}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::SlayerXII,
        name: "Slayer XII",
        description: "Defeat 50,000,000 enemies",
        category: AchievementCategory::Combat,
        icon: "\u{1f409}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::SlayerXIII,
        name: "Harbinger",
        description: "Defeat 100,000,000 enemies - The battlefield trembles at your approach",
        category: AchievementCategory::Combat,
        icon: "\u{1f451}",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::SlayerXIV,
        name: "Reaper",
        description: "Defeat 500,000,000 enemies - Even death takes note of your work",
        category: AchievementCategory::Combat,
        icon: "\u{1f31f}",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::SlayerXV,
        name: "Death Incarnate",
        description: "Defeat 1,000,000,000 enemies - You are the silence between heartbeats",
        category: AchievementCategory::Combat,
        icon: "\u{267e}\u{fe0f}",
        points: 500,
    },
    // ═══════════════════════════════════════════════════════════════
    // COMBAT ACHIEVEMENTS - BOSS KILLS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::BossHunterI,
        name: "Boss Hunter I",
        description: "Defeat your first boss",
        category: AchievementCategory::Combat,
        icon: "\u{1fa93}",
        points: 5,
    },
    AchievementDef {
        id: AchievementId::BossHunterII,
        name: "Boss Hunter II",
        description: "Defeat 10 bosses",
        category: AchievementCategory::Combat,
        icon: "\u{1f5e1}\u{fe0f}",
        points: 5,
    },
    AchievementDef {
        id: AchievementId::BossHunterIII,
        name: "Boss Hunter III",
        description: "Defeat 50 bosses",
        category: AchievementCategory::Combat,
        icon: "\u{2694}\u{fe0f}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::BossHunterIV,
        name: "Boss Hunter IV",
        description: "Defeat 100 bosses",
        category: AchievementCategory::Combat,
        icon: "\u{1f6e1}\u{fe0f}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::BossHunterV,
        name: "Boss Hunter V",
        description: "Defeat 500 bosses",
        category: AchievementCategory::Combat,
        icon: "\u{1f480}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::BossHunterVI,
        name: "Boss Hunter VI",
        description: "Defeat 1,000 bosses",
        category: AchievementCategory::Combat,
        icon: "\u{1f479}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::BossHunterVII,
        name: "Boss Hunter VII",
        description: "Defeat 5,000 bosses",
        category: AchievementCategory::Combat,
        icon: "\u{1f47f}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::BossHunterVIII,
        name: "Boss Hunter VIII",
        description: "Defeat 10,000 bosses",
        category: AchievementCategory::Combat,
        icon: "\u{1f608}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::BossHunterIX,
        name: "Boss Hunter IX",
        description: "Defeat 25,000 bosses",
        category: AchievementCategory::Combat,
        icon: "\u{2620}\u{fe0f}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::BossHunterX,
        name: "Boss Hunter X",
        description: "Defeat 75,000 bosses",
        category: AchievementCategory::Combat,
        icon: "\u{1f525}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::BossHunterXI,
        name: "Boss Hunter XI",
        description: "Defeat 250,000 bosses",
        category: AchievementCategory::Combat,
        icon: "\u{1f409}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::BossHunterXII,
        name: "Boss Hunter XII",
        description: "Defeat 750,000 bosses",
        category: AchievementCategory::Combat,
        icon: "\u{269c}\u{fe0f}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::BossHunterXIII,
        name: "Titan Breaker",
        description: "Defeat 2,500,000 bosses - Even gods kneel before you",
        category: AchievementCategory::Combat,
        icon: "\u{1f531}",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::BossHunterXIV,
        name: "Worldender",
        description: "Defeat 5,000,000 bosses - Worlds are born and die by your hand",
        category: AchievementCategory::Combat,
        icon: "\u{1fa78}",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::BossHunterXV,
        name: "The Absolute",
        description: "Defeat 10,000,000 bosses - Beyond power. Beyond reckoning. Beyond all.",
        category: AchievementCategory::Combat,
        icon: "\u{1f451}",
        points: 500,
    },
    // ═══════════════════════════════════════════════════════════════
    // LEVEL ACHIEVEMENTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::Level10,
        name: "Getting Started",
        description: "Reach level 10",
        category: AchievementCategory::Level,
        icon: "\u{1f949}",
        points: 5,
    },
    AchievementDef {
        id: AchievementId::Level25,
        name: "Adventurer",
        description: "Reach level 25",
        category: AchievementCategory::Level,
        icon: "\u{1f948}",
        points: 5,
    },
    AchievementDef {
        id: AchievementId::Level50,
        name: "Veteran",
        description: "Reach level 50",
        category: AchievementCategory::Level,
        icon: "\u{1f947}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::Level100,
        name: "Centurion",
        description: "Reach level 100",
        category: AchievementCategory::Level,
        icon: "\u{1f48e}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::Level150,
        name: "Elite",
        description: "Reach level 150",
        category: AchievementCategory::Level,
        icon: "\u{1f4a0}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::Level200,
        name: "Champion",
        description: "Reach level 200",
        category: AchievementCategory::Level,
        icon: "\u{2764}\u{fe0f}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::Level250,
        name: "Legendary",
        description: "Reach level 250",
        category: AchievementCategory::Level,
        icon: "\u{1f499}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::Level500,
        name: "Mythic",
        description: "Reach level 500",
        category: AchievementCategory::Level,
        icon: "\u{1f49a}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::Level750,
        name: "Demigod",
        description: "Reach level 750",
        category: AchievementCategory::Level,
        icon: "\u{1f5a4}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::Level1000,
        name: "Immortal",
        description: "Reach level 1000",
        category: AchievementCategory::Level,
        icon: "\u{1f49c}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::Level1500,
        name: "Transcendent",
        description: "Reach level 1500 - The universe bends to your will",
        category: AchievementCategory::Level,
        icon: "\u{267e}\u{fe0f}",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::Level2000,
        name: "Ascendant",
        description: "Reach level 2,000",
        category: AchievementCategory::Level,
        icon: "🌠",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::Level3000,
        name: "Paragon",
        description: "Reach level 3,000",
        category: AchievementCategory::Level,
        icon: "✨",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::Level5000,
        name: "Worldshaper",
        description: "Reach level 5,000",
        category: AchievementCategory::Level,
        icon: "🪐",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::Level7500,
        name: "Starforged",
        description: "Reach level 7,500",
        category: AchievementCategory::Level,
        icon: "🌌",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::Level10000,
        name: "Godmarch",
        description: "Reach level 10,000",
        category: AchievementCategory::Level,
        icon: "☄️",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::Level20000,
        name: "Infinite",
        description: "Reach level 20,000 - Mortal scale has long since failed to describe you",
        category: AchievementCategory::Level,
        icon: "♾️",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::Level100000,
        name: "Unfathomable",
        description: "Reach level 100,000 - You have gone beyond all useful measurement",
        category: AchievementCategory::Level,
        icon: "🌌",
        points: 500,
    },
    // ═══════════════════════════════════════════════════════════════
    // PRESTIGE ACHIEVEMENTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::FirstPrestige,
        name: "Rebirth",
        description: "Prestige for the first time",
        category: AchievementCategory::Prestige,
        icon: "🔄",
        points: 5,
    },
    AchievementDef {
        id: AchievementId::PrestigeV,
        name: "Bronze Rank",
        description: "Reach Prestige Rank 5",
        category: AchievementCategory::Prestige,
        icon: "🥉",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::PrestigeX,
        name: "Silver Rank",
        description: "Reach Prestige Rank 10",
        category: AchievementCategory::Prestige,
        icon: "🥈",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::PrestigeXV,
        name: "Gold Rank",
        description: "Reach Prestige Rank 15",
        category: AchievementCategory::Prestige,
        icon: "🥇",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::PrestigeXX,
        name: "Platinum Rank",
        description: "Reach Prestige Rank 20",
        category: AchievementCategory::Prestige,
        icon: "💎",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::PrestigeXXV,
        name: "Diamond Rank",
        description: "Reach Prestige Rank 25",
        category: AchievementCategory::Prestige,
        icon: "💠",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::PrestigeXXX,
        name: "Ruby Rank",
        description: "Reach Prestige Rank 30",
        category: AchievementCategory::Prestige,
        icon: "❤️",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::PrestigeXL,
        name: "Sapphire Rank",
        description: "Reach Prestige Rank 40",
        category: AchievementCategory::Prestige,
        icon: "💙",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::PrestigeL,
        name: "Emerald Rank",
        description: "Reach Prestige Rank 50",
        category: AchievementCategory::Prestige,
        icon: "💚",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::PrestigeLXX,
        name: "Obsidian Rank",
        description: "Reach Prestige Rank 70",
        category: AchievementCategory::Prestige,
        icon: "🖤",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::PrestigeXC,
        name: "Celestial Rank",
        description: "Reach Prestige Rank 90",
        category: AchievementCategory::Prestige,
        icon: "💜",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::Eternal,
        name: "Eternal",
        description: "Reach Prestige Rank 100 - Your legend echoes through eternity",
        category: AchievementCategory::Prestige,
        icon: "♾️",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::Prestige150,
        name: "Empyrean Rank",
        description: "Reach Prestige Rank 150",
        category: AchievementCategory::Prestige,
        icon: "🧿",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::Prestige200,
        name: "Aeonic Rank",
        description: "Reach Prestige Rank 200",
        category: AchievementCategory::Prestige,
        icon: "⏳",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::Prestige300,
        name: "Paragon Rank",
        description: "Reach Prestige Rank 300",
        category: AchievementCategory::Prestige,
        icon: "🌑",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::Prestige500,
        name: "Void Rank",
        description: "Reach Prestige Rank 500",
        category: AchievementCategory::Prestige,
        icon: "🌌",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::Prestige700,
        name: "Infinite Rank",
        description: "Reach Prestige Rank 700",
        category: AchievementCategory::Prestige,
        icon: "♾️",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::Prestige1000,
        name: "Boundless Rank",
        description: "Reach Prestige Rank 1,000 - Rebirth itself answers to you now",
        category: AchievementCategory::Prestige,
        icon: "👑",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::Prestige10000,
        name: "Omega Rank",
        description: "Reach Prestige Rank 10,000 - The cycle no longer contains you",
        category: AchievementCategory::Prestige,
        icon: "🔱",
        points: 500,
    },
    // ═══════════════════════════════════════════════════════════════
    // ASCENSION MILESTONE ACHIEVEMENTS (one per level, I-VI)
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::AscensionI,
        name: "Ascension I",
        description: "Reach Ascension I \u{2014} double your power",
        category: AchievementCategory::Prestige,
        icon: "\u{2728}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::AscensionII,
        name: "Ascension II",
        description: "Reach Ascension II \u{2014} 4x power",
        category: AchievementCategory::Prestige,
        icon: "\u{2728}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::AscensionIII,
        name: "Ascension III",
        description: "Reach Ascension III \u{2014} 8x power",
        category: AchievementCategory::Prestige,
        icon: "\u{2728}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::AscensionIV,
        name: "Ascension IV",
        description: "Reach Ascension IV \u{2014} 16x power",
        category: AchievementCategory::Prestige,
        icon: "\u{2728}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::AscensionV,
        name: "Ascension V",
        description: "Reach Ascension V \u{2014} 32x power",
        category: AchievementCategory::Prestige,
        icon: "\u{2728}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::AscensionVI,
        name: "Ascension VI",
        description: "Reach Ascension VI \u{2014} 64x power, the apex of mortal strength",
        category: AchievementCategory::Prestige,
        icon: "\u{1f31f}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::AscensionVII,
        name: "Ascension VII",
        description: "Reach Ascension VII \u{2014} 96x power, beyond mortal limits",
        category: AchievementCategory::Prestige,
        icon: "\u{1f31f}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::AscensionVIII,
        name: "Ascension VIII",
        description: "Reach Ascension VIII \u{2014} 144x power, reality bends",
        category: AchievementCategory::Prestige,
        icon: "\u{1f31f}",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::AscensionIX,
        name: "Ascension IX",
        description: "Reach Ascension IX \u{2014} 216x power, threads of creation",
        category: AchievementCategory::Prestige,
        icon: "\u{1f31f}",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::AscensionX,
        name: "Ascension X",
        description: "Reach Ascension X \u{2014} 324x power, the Final Weave",
        category: AchievementCategory::Prestige,
        icon: "\u{1f31f}",
        points: 500,
    },
    // Zone completion achievements (one per zone)
    AchievementDef {
        id: AchievementId::Zone1Complete,
        name: "Meadow Wanderer",
        description: "Clear Zone 1: Meadow",
        category: AchievementCategory::Progression,
        icon: "🌻",
        points: 5,
    },
    AchievementDef {
        id: AchievementId::Zone2Complete,
        name: "Forest Walker",
        description: "Clear Zone 2: Dark Forest",
        category: AchievementCategory::Progression,
        icon: "🌲",
        points: 5,
    },
    AchievementDef {
        id: AchievementId::Zone3Complete,
        name: "Peak Climber",
        description: "Clear Zone 3: Mountain Pass",
        category: AchievementCategory::Progression,
        icon: "🏔️",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::Zone4Complete,
        name: "Ruin Explorer",
        description: "Clear Zone 4: Ancient Ruins",
        category: AchievementCategory::Progression,
        icon: "🏛️",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::Zone5Complete,
        name: "Volcano Conqueror",
        description: "Clear Zone 5: Volcanic Wastes",
        category: AchievementCategory::Progression,
        icon: "🌋",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::Zone6Complete,
        name: "Frost Survivor",
        description: "Clear Zone 6: Frozen Tundra",
        category: AchievementCategory::Progression,
        icon: "❄️",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::Zone7Complete,
        name: "Crystal Seeker",
        description: "Clear Zone 7: Crystal Caverns",
        category: AchievementCategory::Progression,
        icon: "💎",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::Zone8Complete,
        name: "Deep Diver",
        description: "Clear Zone 8: Sunken Kingdom",
        category: AchievementCategory::Progression,
        icon: "🌊",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::Zone9Complete,
        name: "Sky Lord",
        description: "Clear Zone 9: Floating Isles",
        category: AchievementCategory::Progression,
        icon: "☁️",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::Zone10Complete,
        name: "Citadel Conqueror",
        description: "Clear Zone 10: Storm Citadel",
        category: AchievementCategory::Progression,
        icon: "⛈️",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::TheStormbreaker,
        name: "The Stormbreaker",
        description: "Forge the legendary Stormbreaker at the Haven forge",
        category: AchievementCategory::Progression,
        icon: "⚡",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::StormsEnd,
        name: "Storm's End",
        description: "Defeat the final boss of Zone 10: Storm Citadel",
        category: AchievementCategory::Progression,
        icon: "🏆",
        points: 250,
    },
    // The Expanse achievement
    AchievementDef {
        id: AchievementId::BeyondInfinity,
        name: "Beyond Infinity",
        description: "Complete a cycle of The Expanse \u{2014} there is nothing left to conquer, only endlessness",
        category: AchievementCategory::Progression,
        icon: "\u{267e}\u{fe0f}",
        points: 500,
    },
    // ═══════════════════════════════════════════════════════════════
    // FRACTURE ZONE COMPLETION ACHIEVEMENTS (ZONES 12-30)
    // ═══════════════════════════════════════════════════════════════
    // Ch.1: The Red Fault (🔥 fire — volcanic/ember theme)
    AchievementDef {
        id: AchievementId::FractureZone12,
        name: "Rimbreaker",
        description: "Clear Zone 12: Splintered Rim",
        category: AchievementCategory::Progression,
        icon: "\u{1f525}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::FractureZone13,
        name: "Cinderfall",
        description: "Clear Zone 13: Ember Ravine",
        category: AchievementCategory::Progression,
        icon: "\u{1f525}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::FractureZone14,
        name: "Heart Piercer",
        description: "Clear Zone 14: Heart of the Fault",
        category: AchievementCategory::Progression,
        icon: "\u{1f525}",
        points: 50,
    },
    // Ch.2: The Mirror Scar (🪞 mirror — reflection/light theme)
    AchievementDef {
        id: AchievementId::FractureZone15,
        name: "Shard Breaker",
        description: "Clear Zone 15: Shard Fields",
        category: AchievementCategory::Progression,
        icon: "\u{1fa9e}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::FractureZone16,
        name: "Light Bender",
        description: "Clear Zone 16: Refraction Steps",
        category: AchievementCategory::Progression,
        icon: "\u{1fa9e}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::FractureZone17,
        name: "Sunslayer",
        description: "Clear Zone 17: Hall of Second Suns",
        category: AchievementCategory::Progression,
        icon: "\u{1fa9e}",
        points: 100,
    },
    // Ch.3: The Black Mouth (🕳️ hole — abyss/void theme)
    AchievementDef {
        id: AchievementId::FractureZone18,
        name: "Ashen Sentinel",
        description: "Clear Zone 18: Ashen Verge",
        category: AchievementCategory::Progression,
        icon: "\u{1f573}\u{fe0f}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::FractureZone19,
        name: "Throat Runner",
        description: "Clear Zone 19: Throat of the World",
        category: AchievementCategory::Progression,
        icon: "\u{1f573}\u{fe0f}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::FractureZone20,
        name: "Maw Closer",
        description: "Clear Zone 20: The Black Mouth",
        category: AchievementCategory::Progression,
        icon: "\u{1f573}\u{fe0f}",
        points: 250,
    },
    // Ch.4: The Hollow Throne (👑 crown — throne/sovereignty theme)
    AchievementDef {
        id: AchievementId::FractureZone21,
        name: "Amber March",
        description: "Clear Zone 21: Sunken Processional",
        category: AchievementCategory::Progression,
        icon: "\u{1f451}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::FractureZone22,
        name: "Pale Scholar",
        description: "Clear Zone 22: The Pale Archive",
        category: AchievementCategory::Progression,
        icon: "\u{1f451}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::FractureZone23,
        name: "Thronebreaker",
        description: "Clear Zone 23: The Hollow Throne",
        category: AchievementCategory::Progression,
        icon: "\u{1f451}",
        points: 50,
    },
    // Ch.5: The Wailing Reach (🌊 wave — sea/resonance theme)
    AchievementDef {
        id: AchievementId::FractureZone24,
        name: "Stillwater",
        description: "Clear Zone 24: The Stillborn Sea",
        category: AchievementCategory::Progression,
        icon: "\u{1f30a}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::FractureZone25,
        name: "Resonance Breaker",
        description: "Clear Zone 25: Resonance Fault",
        category: AchievementCategory::Progression,
        icon: "\u{1f30a}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::FractureZone26,
        name: "Edge Walker",
        description: "Clear Zone 26: The Wailing Reach",
        category: AchievementCategory::Progression,
        icon: "\u{1f30a}",
        points: 100,
    },
    // Ch.6: The Origin Wound (🩸 drop of blood — wound/origin theme)
    AchievementDef {
        id: AchievementId::FractureZone27,
        name: "Scar Render",
        description: "Clear Zone 27: The Scar Root",
        category: AchievementCategory::Progression,
        icon: "\u{1fa78}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::FractureZone28,
        name: "Echo Silencer",
        description: "Clear Zone 28: Echoing Abyss",
        category: AchievementCategory::Progression,
        icon: "\u{1fa78}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::FractureZone29,
        name: "Last Listener",
        description: "Clear Zone 29: Threshold of Silence",
        category: AchievementCategory::Progression,
        icon: "\u{1fa78}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::FractureZone30,
        name: "Scar Mender",
        description: "Clear Zone 30: The Origin Wound \u{2014} seal the fracture at its source",
        category: AchievementCategory::Progression,
        icon: "\u{1fa78}",
        points: 500,
    },
    // ═══════════════════════════════════════════════════════════════
    // CHALLENGE ACHIEVEMENTS - CHESS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::ChessNovice,
        name: "Chess Novice",
        description: "Win chess on Novice difficulty",
        category: AchievementCategory::Challenges,
        icon: "♟️",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::ChessApprentice,
        name: "Chess Apprentice",
        description: "Win chess on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "♟️",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::ChessJourneyman,
        name: "Chess Journeyman",
        description: "Win chess on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "♟️",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::ChessMaster,
        name: "Chess Master",
        description: "Win chess on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "♛",
        points: 250,
    },
    // ═══════════════════════════════════════════════════════════════
    // CHALLENGE ACHIEVEMENTS - MORRIS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::MorrisNovice,
        name: "Morris Novice",
        description: "Win Morris on Novice difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚪",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::MorrisApprentice,
        name: "Morris Apprentice",
        description: "Win Morris on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚪",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::MorrisJourneyman,
        name: "Morris Journeyman",
        description: "Win Morris on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚪",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::MorrisMaster,
        name: "Morris Master",
        description: "Win Morris on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚪",
        points: 100,
    },
    // ═══════════════════════════════════════════════════════════════
    // CHALLENGE ACHIEVEMENTS - GOMOKU
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::GomokuNovice,
        name: "Gomoku Novice",
        description: "Win Gomoku on Novice difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚫",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::GomokuApprentice,
        name: "Gomoku Apprentice",
        description: "Win Gomoku on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚫",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::GomokuJourneyman,
        name: "Gomoku Journeyman",
        description: "Win Gomoku on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚫",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::GomokuMaster,
        name: "Gomoku Master",
        description: "Win Gomoku on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚫",
        points: 100,
    },
    // ═══════════════════════════════════════════════════════════════
    // CHALLENGE ACHIEVEMENTS - MINESWEEPER
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::MinesweeperNovice,
        name: "Minesweeper Novice",
        description: "Win Minesweeper on Novice difficulty",
        category: AchievementCategory::Challenges,
        icon: "💣",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::MinesweeperApprentice,
        name: "Minesweeper Apprentice",
        description: "Win Minesweeper on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "💣",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::MinesweeperJourneyman,
        name: "Minesweeper Journeyman",
        description: "Win Minesweeper on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "💣",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::MinesweeperMaster,
        name: "Minesweeper Master",
        description: "Win Minesweeper on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "💣",
        points: 100,
    },
    // ═══════════════════════════════════════════════════════════════
    // CHALLENGE ACHIEVEMENTS - RUNE
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::RuneNovice,
        name: "Rune Novice",
        description: "Win Rune on Novice difficulty",
        category: AchievementCategory::Challenges,
        icon: "🔮",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::RuneApprentice,
        name: "Rune Apprentice",
        description: "Win Rune on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "🔮",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::RuneJourneyman,
        name: "Rune Journeyman",
        description: "Win Rune on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "🔮",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::RuneMaster,
        name: "Rune Master",
        description: "Win Rune on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "🔮",
        points: 100,
    },
    // ═══════════════════════════════════════════════════════════════
    // CHALLENGE ACHIEVEMENTS - GO
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::GoNovice,
        name: "Go Novice",
        description: "Win Go on Novice difficulty",
        category: AchievementCategory::Challenges,
        icon: "🟤",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::GoApprentice,
        name: "Go Apprentice",
        description: "Win Go on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "🟤",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::GoJourneyman,
        name: "Go Journeyman",
        description: "Win Go on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "🟤",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::GoMaster,
        name: "Go Master",
        description: "Win Go on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "🟤",
        points: 250,
    },
    // ═══════════════════════════════════════════════════════════════
    // CHALLENGE ACHIEVEMENTS - FLAPPY BIRD
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::FlappyNovice,
        name: "Skyward Novice",
        description: "Win Skyward Gauntlet on Novice difficulty",
        category: AchievementCategory::Challenges,
        icon: "◆",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::FlappyApprentice,
        name: "Skyward Apprentice",
        description: "Win Skyward Gauntlet on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "◆",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::FlappyJourneyman,
        name: "Skyward Journeyman",
        description: "Win Skyward Gauntlet on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "◆",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::FlappyMaster,
        name: "Skyward Master",
        description: "Win Skyward Gauntlet on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "›",
        points: 100,
    },
    // ═══════════════════════════════════════════════════════════════
    // CHALLENGE ACHIEVEMENTS - SNAKE
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::SnakeNovice,
        name: "Serpent Novice",
        description: "Win Serpent's Path on Novice difficulty",
        category: AchievementCategory::Challenges,
        icon: "~",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::SnakeApprentice,
        name: "Serpent Apprentice",
        description: "Win Serpent's Path on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "~",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::SnakeJourneyman,
        name: "Serpent Journeyman",
        description: "Win Serpent's Path on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "~",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::SnakeMaster,
        name: "Serpent Master",
        description: "Win Serpent's Path on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "~",
        points: 100,
    },
    // ═══════════════════════════════════════════════════════════════
    // CHALLENGE ACHIEVEMENTS - CONTAINMENT BREACH
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::ContainmentBreachNovice,
        name: "Containment Breach Novice",
        description: "Win Containment Breach on Novice difficulty",
        category: AchievementCategory::Challenges,
        icon: "▣",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::ContainmentBreachApprentice,
        name: "Containment Breach Apprentice",
        description: "Win Containment Breach on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "▣",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::ContainmentBreachJourneyman,
        name: "Containment Breach Journeyman",
        description: "Win Containment Breach on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "▣",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::ContainmentBreachMaster,
        name: "Containment Breach Master",
        description: "Win Containment Breach on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "▣",
        points: 100,
    },
    // ═══════════════════════════════════════════════════════════════
    // CHALLENGE ACHIEVEMENTS - SIGIL SURGE
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::SigilSurgeNovice,
        name: "Sigil Surge Novice",
        description: "Win Sigil Surge on Novice difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚡",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::SigilSurgeApprentice,
        name: "Sigil Surge Apprentice",
        description: "Win Sigil Surge on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚡",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::SigilSurgeJourneyman,
        name: "Sigil Surge Journeyman",
        description: "Win Sigil Surge on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚡",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::SigilSurgeMaster,
        name: "Sigil Surge Master",
        description: "Win Sigil Surge on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚡",
        points: 100,
    },
    // ═══════════════════════════════════════════════════════════════
    // CHALLENGE ACHIEVEMENTS - SHARD FUSION
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::ShardFusionNovice,
        name: "Shard Fusion Novice",
        description: "Merge the crystal shards to 512 on Novice difficulty",
        category: AchievementCategory::Challenges,
        icon: "◆",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::ShardFusionApprentice,
        name: "Shard Fusion Apprentice",
        description: "Merge the crystal shards to 1024 on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "◆",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::ShardFusionJourneyman,
        name: "Shard Fusion Journeyman",
        description: "Merge the crystal shards to 2048 on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "◆",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::ShardFusionMaster,
        name: "Shard Fusion Master",
        description: "Merge the crystal shards to 4096 on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "◆",
        points: 100,
    },
    // ═══════════════════════════════════════════════════════════════
    // CHALLENGE ACHIEVEMENTS - META
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::GrandChampion,
        name: "Grand Champion",
        description: "Win 100 minigames total",
        category: AchievementCategory::Challenges,
        icon: "🏅",
        points: 250,
    },
    // ═══════════════════════════════════════════════════════════════
    // FISHING ACHIEVEMENTS - RANK MILESTONES
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::GoneFishing,
        name: "Gone Fishing",
        description: "Catch your first fish",
        category: AchievementCategory::Exploration,
        icon: "🎣",
        points: 5,
    },
    AchievementDef {
        id: AchievementId::FishermanI,
        name: "Fisherman I",
        description: "Reach fishing rank 10",
        category: AchievementCategory::Exploration,
        icon: "🐟",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::FishermanII,
        name: "Fisherman II",
        description: "Reach fishing rank 20",
        category: AchievementCategory::Exploration,
        icon: "🐠",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::FishermanIII,
        name: "Fisherman III",
        description: "Reach fishing rank 30 (base max)",
        category: AchievementCategory::Exploration,
        icon: "🐡",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::FishermanIV,
        name: "Fisherman IV",
        description: "Reach fishing rank 40 (requires Fishing Dock T4)",
        category: AchievementCategory::Exploration,
        icon: "\u{1f433}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::StormLeviathan,
        name: "Storm Leviathan",
        description: "Catch the legendary Storm Leviathan",
        category: AchievementCategory::Exploration,
        icon: "\u{1f30a}",
        points: 250,
    },
    // ═══════════════════════════════════════════════════════════════
    // FISHING ACHIEVEMENTS - CATCH COUNTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::FishCatcherI,
        name: "Fish Catcher I",
        description: "Catch 100 fish",
        category: AchievementCategory::Exploration,
        icon: "🐟",
        points: 5,
    },
    AchievementDef {
        id: AchievementId::FishCatcherII,
        name: "Fish Catcher II",
        description: "Catch 1,000 fish",
        category: AchievementCategory::Exploration,
        icon: "🐠",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::FishCatcherIII,
        name: "Fish Catcher III",
        description: "Catch 10,000 fish",
        category: AchievementCategory::Exploration,
        icon: "🐡",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::FishCatcherIV,
        name: "Fish Catcher IV",
        description: "Catch 100,000 fish",
        category: AchievementCategory::Exploration,
        icon: "🐋",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::FishCatcherV,
        name: "Fish Catcher V",
        description: "Catch 500,000 fish",
        category: AchievementCategory::Exploration,
        icon: "\u{1f40b}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::FishCatcherVI,
        name: "Fish Catcher VI",
        description: "Catch 1,000,000 fish",
        category: AchievementCategory::Exploration,
        icon: "\u{1f40b}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::FishCatcherVII,
        name: "Fish Catcher VII",
        description: "Catch 5,000,000 fish",
        category: AchievementCategory::Exploration,
        icon: "\u{1f40b}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::FishCatcherVIII,
        name: "Leviathan's Rival",
        description: "Catch 10,000,000 fish - Even the Leviathan watches its depths",
        category: AchievementCategory::Exploration,
        icon: "\u{1f988}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::FishCatcherIX,
        name: "Poseidon's Hand",
        description: "Catch 50,000,000 fish - Every wave carries your legend",
        category: AchievementCategory::Exploration,
        icon: "\u{1f30a}",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::FishCatcherX,
        name: "Lord of the Deep",
        description: "Catch 100,000,000 fish - You are the tide. You are the deep.",
        category: AchievementCategory::Exploration,
        icon: "\u{1f531}",
        points: 500,
    },
    // ═══════════════════════════════════════════════════════════════
    // DUNGEON ACHIEVEMENTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::DungeonDiver,
        name: "Dungeon Diver",
        description: "Complete your first dungeon",
        category: AchievementCategory::Exploration,
        icon: "\u{1f6aa}",
        points: 5,
    },
    AchievementDef {
        id: AchievementId::DungeonMasterI,
        name: "Dungeon Master I",
        description: "Complete 10 dungeons",
        category: AchievementCategory::Exploration,
        icon: "\u{1f5dd}\u{fe0f}",
        points: 5,
    },
    AchievementDef {
        id: AchievementId::DungeonMasterII,
        name: "Dungeon Master II",
        description: "Complete 50 dungeons",
        category: AchievementCategory::Exploration,
        icon: "\u{1f56f}\u{fe0f}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::DungeonMasterIII,
        name: "Dungeon Master III",
        description: "Complete 100 dungeons",
        category: AchievementCategory::Exploration,
        icon: "\u{1f5e1}\u{fe0f}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::DungeonMasterIV,
        name: "Dungeon Master IV",
        description: "Complete 1,000 dungeons",
        category: AchievementCategory::Exploration,
        icon: "\u{2694}\u{fe0f}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::DungeonMasterV,
        name: "Dungeon Master V",
        description: "Complete 5,000 dungeons",
        category: AchievementCategory::Exploration,
        icon: "\u{1f6e1}\u{fe0f}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::DungeonMasterVI,
        name: "Dungeon Master VI",
        description: "Complete 10,000 dungeons",
        category: AchievementCategory::Exploration,
        icon: "\u{1f480}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::DungeonMasterVII,
        name: "Dungeon Master VII",
        description: "Complete 25,000 dungeons",
        category: AchievementCategory::Exploration,
        icon: "\u{1f3f0}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::DungeonMasterVIII,
        name: "Labyrinth Lord",
        description: "Complete 100,000 dungeons - Every maze unravels at your touch",
        category: AchievementCategory::Exploration,
        icon: "\u{1f409}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::DungeonMasterIX,
        name: "Abyss Walker",
        description: "Complete 500,000 dungeons - The forgotten halls remember only you",
        category: AchievementCategory::Exploration,
        icon: "\u{269c}\u{fe0f}",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::DungeonMasterX,
        name: "The Undying Delver",
        description:
            "Complete 1,000,000 dungeons - The dungeon is not a place \u{2014} it is your kingdom",
        category: AchievementCategory::Exploration,
        icon: "\u{1f451}",
        points: 500,
    },
    // ═══════════════════════════════════════════════════════════════
    // HAVEN ACHIEVEMENTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::HavenDiscovered,
        name: "Haven Found",
        description: "Discover the Haven",
        category: AchievementCategory::Exploration,
        icon: "\u{26fa}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::HavenBuilderI,
        name: "Haven Builder I",
        description: "Upgrade all Haven rooms to Tier 1",
        category: AchievementCategory::Exploration,
        icon: "\u{1f3e0}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::HavenBuilderII,
        name: "Haven Builder II",
        description: "Upgrade all Haven rooms to Tier 2",
        category: AchievementCategory::Exploration,
        icon: "\u{1f3f0}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::HavenArchitect,
        name: "Haven Architect",
        description: "Upgrade all Haven rooms to Tier 3",
        category: AchievementCategory::Exploration,
        icon: "\u{1f3db}\u{fe0f}",
        points: 250,
    },
    // ═══════════════════════════════════════════════════════════════
    // ENHANCEMENT ACHIEVEMENTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::SoulforgeDiscovered,
        name: "Soulforge Found",
        description: "Uncover an ancient Soulforge",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::ApprenticeSmith,
        name: "Apprentice Smith",
        description: "Enhance any equipment to +1",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::FullyTempered,
        name: "Fully Tempered",
        description: "Enhance all 7 equipment slots to +4",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::JourneymanSmith,
        name: "Journeyman Smith",
        description: "Enhance any equipment to +5",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::SoulforgeAdept,
        name: "Soulforge Adept",
        description: "Enhance any equipment to +6",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::SoulforgeSavant,
        name: "Soulforge Savant",
        description: "Enhance any equipment to +7",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::SoulforgeMaster,
        name: "Soulforge Master",
        description: "Enhance any equipment to +8",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::SoulforgeGrandmaster,
        name: "Soulforge Grandmaster",
        description: "Enhance any equipment to +9",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::SoulforgeAscendant,
        name: "Soulforge Ascendant",
        description: "Enhance any equipment to +10",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::SoulConvergence,
        name: "Soul Convergence",
        description: "Enhance all 7 equipment slots to +7",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::PersistentHammering,
        name: "Persistent Hammering",
        description: "Attempt 100 enhancements",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
        points: 25,
    },
    // ═══════════════════════════════════════════════════════════════
    // THE DEEP ACHIEVEMENTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::TheDeepDiscovered,
        name: "Into The Deep",
        description: "Discover The Deep mercenary guild",
        category: AchievementCategory::Deep,
        icon: "\u{2693}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::FirstMissionComplete,
        name: "First Contract",
        description: "Complete your first Deep mission",
        category: AchievementCategory::Deep,
        icon: "\u{1F4DC}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::DeepMissionsX,
        name: "Seasoned Contractor",
        description: "Complete 10 Deep missions",
        category: AchievementCategory::Deep,
        icon: "\u{1F4DC}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::DeepMissionsXXV,
        name: "Veteran Contractor",
        description: "Complete 25 Deep missions",
        category: AchievementCategory::Deep,
        icon: "\u{1F4DC}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::DeepMissionsL,
        name: "Elite Contractor",
        description: "Complete 50 Deep missions",
        category: AchievementCategory::Deep,
        icon: "\u{1F4DC}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::DeepMissionsC,
        name: "Legendary Contractor",
        description: "Complete 100 Deep missions",
        category: AchievementCategory::Deep,
        icon: "\u{1F4DC}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::FirstBreakthrough,
        name: "Breakthrough",
        description: "Complete your first breakthrough mission in The Deep",
        category: AchievementCategory::Deep,
        icon: "\u{2B50}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::Layer5Cleared,
        name: "Warrens Explorer",
        description: "Reach Layer 5 of The Deep",
        category: AchievementCategory::Deep,
        icon: "\u{1F5FA}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::Layer10Cleared,
        name: "Hollows Delver",
        description: "Reach Layer 10 of The Deep",
        category: AchievementCategory::Deep,
        icon: "\u{1F5FA}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::Layer15Cleared,
        name: "Sunken Pioneer",
        description: "Reach Layer 15 of The Deep",
        category: AchievementCategory::Deep,
        icon: "\u{1F5FA}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::Layer20Cleared,
        name: "Abyss Breacher",
        description: "Reach Layer 20 of The Deep",
        category: AchievementCategory::Deep,
        icon: "\u{1F5FA}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::Layer25Cleared,
        name: "Abyssal Pioneer",
        description: "Reach Layer 25 of The Deep \u{2014} the true abyss opens before you",
        category: AchievementCategory::Deep,
        icon: "\u{1F30C}",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::VoidExplorer,
        name: "Void Explorer",
        description: "Reach Layer 26 of The Deep \u{2014} descend into The Void",
        category: AchievementCategory::Deep,
        icon: "\u{1F573}\u{fe0f}",
        points: 500,
    },
    AchievementDef {
        id: AchievementId::GuildRank2,
        name: "Company Captain",
        description: "Reach Guild Rank 2 (Company)",
        category: AchievementCategory::Deep,
        icon: "\u{2694}\u{fe0f}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::GuildRank3,
        name: "Battalion Commander",
        description: "Reach Guild Rank 3 (Battalion)",
        category: AchievementCategory::Deep,
        icon: "\u{2694}\u{fe0f}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::GuildRank4,
        name: "Legion Marshal",
        description: "Reach Guild Rank 4 (Legion)",
        category: AchievementCategory::Deep,
        icon: "\u{2694}\u{fe0f}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::GuildRank5,
        name: "Vanguard Overlord",
        description: "Reach Guild Rank 5 (Vanguard) \u{2014} you command the finest in The Deep",
        category: AchievementCategory::Deep,
        icon: "\u{1F451}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::FirstMercLost,
        name: "The Cost of War",
        description: "Lose a mercenary in The Deep",
        category: AchievementCategory::Deep,
        icon: "\u{1FAA6}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::GatewayOpened,
        name: "The Gate Opens",
        description: "Opened the Gateway beneath the world",
        category: AchievementCategory::Deep,
        icon: "\u{1F6AA}",
        points: 500,
    },
    // Power Core achievements — unlocked at fracture zone unlock layers
    AchievementDef {
        id: AchievementId::PowerCoreI,
        name: "Power Core I",
        description: "Reach Deep Layer 3 \u{2014} activate the Red Fault power core (2 PR/day)",
        category: AchievementCategory::Deep,
        icon: "\u{2B21}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::PowerCoreII,
        name: "Power Core II",
        description: "Reach Deep Layer 7 \u{2014} activate the Mirror Scar power core (3 PR/day)",
        category: AchievementCategory::Deep,
        icon: "\u{2B21}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::PowerCoreIII,
        name: "Power Core III",
        description: "Reach Deep Layer 12 \u{2014} activate the Black Mouth power core (5 PR/day)",
        category: AchievementCategory::Deep,
        icon: "\u{2B21}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::PowerCoreIV,
        name: "Power Core IV",
        description: "Reach Deep Layer 18 \u{2014} activate the Hollow Throne power core (8 PR/day)",
        category: AchievementCategory::Deep,
        icon: "\u{2B21}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::PowerCoreV,
        name: "Power Core V",
        description: "Reach Deep Layer 25 \u{2014} activate the Wailing Reach power core (12 PR/day)",
        category: AchievementCategory::Deep,
        icon: "\u{2B21}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::PowerCoreVI,
        name: "Power Core VI",
        description: "Reach Deep Layer 30 \u{2014} activate the Origin Wound power core (18 PR/day)",
        category: AchievementCategory::Deep,
        icon: "\u{2B21}",
        points: 500,
    },
    // Loom achievements
    AchievementDef {
        id: AchievementId::LoomDiscovered,
        name: "Thread Awakening",
        description: "Discover the Loom of Worlds",
        category: AchievementCategory::Loom,
        icon: "\u{1F9F5}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::LoomPattern1,
        name: "First Thread",
        description: "Complete your first Woven Pattern",
        category: AchievementCategory::Loom,
        icon: "\u{1F9F5}",
        points: 10,
    },
    AchievementDef {
        id: AchievementId::LoomPattern4,
        name: "Threadweaver",
        description: "Complete 4 Woven Patterns",
        category: AchievementCategory::Loom,
        icon: "\u{1F9F5}",
        points: 25,
    },
    AchievementDef {
        id: AchievementId::LoomPattern8,
        name: "Pattern Adept",
        description: "Complete 8 Woven Patterns",
        category: AchievementCategory::Loom,
        icon: "\u{1F9F5}",
        points: 50,
    },
    AchievementDef {
        id: AchievementId::LoomPattern16,
        name: "Pattern Master",
        description: "Complete 16 Woven Patterns",
        category: AchievementCategory::Loom,
        icon: "\u{1F9F5}",
        points: 100,
    },
    AchievementDef {
        id: AchievementId::LoomPattern22,
        name: "Grand Weaver",
        description: "Complete 22 Woven Patterns",
        category: AchievementCategory::Loom,
        icon: "\u{1F9F5}",
        points: 250,
    },
    AchievementDef {
        id: AchievementId::LoomPattern28,
        name: "Loom Perfected",
        description: "Complete all 28 Woven Patterns",
        category: AchievementCategory::Loom,
        icon: "\u{1F9F5}",
        points: 500,
    },
];

/// Get the definition for a specific achievement (O(1) HashMap lookup).
pub fn get_achievement_def(id: AchievementId) -> Option<&'static AchievementDef> {
    ACHIEVEMENT_BY_ID.get(&id).copied()
}

/// Get achievements filtered by category (O(1) HashMap lookup).
pub fn get_achievements_by_category(category: AchievementCategory) -> Vec<&'static AchievementDef> {
    ACHIEVEMENTS_BY_CATEGORY
        .get(&category)
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_achievements_have_unique_ids() {
        use std::collections::HashSet;
        let mut ids = HashSet::new();
        for achievement in ALL_ACHIEVEMENTS {
            assert!(
                ids.insert(achievement.id),
                "Duplicate achievement ID: {:?}",
                achievement.id
            );
        }
    }

    #[test]
    fn test_get_achievement_def() {
        let def = get_achievement_def(AchievementId::SlayerI).unwrap();
        assert_eq!(def.name, "Slayer I");
        assert_eq!(def.category, AchievementCategory::Combat);
    }

    #[test]
    fn test_get_achievements_by_category() {
        let combat = get_achievements_by_category(AchievementCategory::Combat);
        assert!(!combat.is_empty());
        for a in combat {
            assert_eq!(a.category, AchievementCategory::Combat);
        }

        let level = get_achievements_by_category(AchievementCategory::Level);
        assert!(level.iter().any(|a| a.id == AchievementId::Level10));
        assert!(level.iter().any(|a| a.id == AchievementId::Level100000));
        assert!(!level.iter().any(|a| a.id == AchievementId::FirstPrestige));

        let prestige = get_achievements_by_category(AchievementCategory::Prestige);
        assert!(prestige
            .iter()
            .any(|a| a.id == AchievementId::FirstPrestige));
        assert!(prestige.iter().any(|a| a.id == AchievementId::Eternal));
        assert!(prestige
            .iter()
            .any(|a| a.id == AchievementId::Prestige10000));
        assert!(!prestige.iter().any(|a| a.id == AchievementId::Level10));
    }

    /// Verify every `AchievementId` variant has exactly one entry in
    /// `ALL_ACHIEVEMENTS`, and that `ALL_ACHIEVEMENTS` has no extras.
    ///
    /// ## How this test enforces coverage
    ///
    /// We compare `ALL_ACHIEVEMENTS.len()` against the manually maintained
    /// `AchievementId::VARIANT_COUNT` constant.  When a new variant is added,
    /// updating `VARIANT_COUNT` in `types.rs` will fail this test unless a
    /// matching definition is also added to `ALL_ACHIEVEMENTS`.
    ///
    /// Note: `std::mem::variant_count` (issue #73662) would replace the manual
    /// constant but is not yet stable on the Rust toolchain used by this project.
    /// Switch to it once it stabilises.
    ///
    /// The per-ID loop below catches the complementary error: a definition whose
    /// `id` field doesn't correspond to the expected variant (e.g. a copy-paste
    /// typo).  Together the two checks give complete bidirectional coverage.
    #[test]
    fn test_every_achievement_id_variant_has_definition() {
        // ── 1. Count check ────────────────────────────────────────────────────
        assert_eq!(
            AchievementId::VARIANT_COUNT,
            ALL_ACHIEVEMENTS.len(),
            "AchievementId::VARIANT_COUNT ({}) does not match ALL_ACHIEVEMENTS.len() ({}). \
             Add a definition for every new variant and update VARIANT_COUNT.",
            AchievementId::VARIANT_COUNT,
            ALL_ACHIEVEMENTS.len(),
        );

        // ── 2. Per-entry check ────────────────────────────────────────────────
        // Ensures no definition uses the wrong AchievementId (copy-paste typos).
        // `test_all_achievements_have_unique_ids` guarantees there are no duplicates.
        let all_ids: &[AchievementId] = &[
            AchievementId::SlayerI,
            AchievementId::SlayerII,
            AchievementId::SlayerIII,
            AchievementId::SlayerIV,
            AchievementId::SlayerV,
            AchievementId::SlayerVI,
            AchievementId::SlayerVII,
            AchievementId::SlayerVIII,
            AchievementId::SlayerIX,
            AchievementId::SlayerX,
            AchievementId::SlayerXI,
            AchievementId::SlayerXII,
            AchievementId::SlayerXIII,
            AchievementId::SlayerXIV,
            AchievementId::SlayerXV,
            AchievementId::BossHunterI,
            AchievementId::BossHunterII,
            AchievementId::BossHunterIII,
            AchievementId::BossHunterIV,
            AchievementId::BossHunterV,
            AchievementId::BossHunterVI,
            AchievementId::BossHunterVII,
            AchievementId::BossHunterVIII,
            AchievementId::BossHunterIX,
            AchievementId::BossHunterX,
            AchievementId::BossHunterXI,
            AchievementId::BossHunterXII,
            AchievementId::BossHunterXIII,
            AchievementId::BossHunterXIV,
            AchievementId::BossHunterXV,
            AchievementId::Level10,
            AchievementId::Level25,
            AchievementId::Level50,
            AchievementId::Level100,
            AchievementId::Level150,
            AchievementId::Level200,
            AchievementId::Level250,
            AchievementId::Level500,
            AchievementId::Level750,
            AchievementId::Level1000,
            AchievementId::Level1500,
            AchievementId::Level2000,
            AchievementId::Level3000,
            AchievementId::Level5000,
            AchievementId::Level7500,
            AchievementId::Level10000,
            AchievementId::Level20000,
            AchievementId::Level100000,
            AchievementId::FirstPrestige,
            AchievementId::PrestigeV,
            AchievementId::PrestigeX,
            AchievementId::PrestigeXV,
            AchievementId::PrestigeXX,
            AchievementId::PrestigeXXV,
            AchievementId::PrestigeXXX,
            AchievementId::PrestigeXL,
            AchievementId::PrestigeL,
            AchievementId::PrestigeLXX,
            AchievementId::PrestigeXC,
            AchievementId::Eternal,
            AchievementId::Prestige150,
            AchievementId::Prestige200,
            AchievementId::Prestige300,
            AchievementId::Prestige500,
            AchievementId::Prestige700,
            AchievementId::Prestige1000,
            AchievementId::Prestige10000,
            AchievementId::Zone1Complete,
            AchievementId::Zone2Complete,
            AchievementId::Zone3Complete,
            AchievementId::Zone4Complete,
            AchievementId::Zone5Complete,
            AchievementId::Zone6Complete,
            AchievementId::Zone7Complete,
            AchievementId::Zone8Complete,
            AchievementId::Zone9Complete,
            AchievementId::Zone10Complete,
            AchievementId::TheStormbreaker,
            AchievementId::StormsEnd,
            AchievementId::BeyondInfinity,
            AchievementId::FractureZone12,
            AchievementId::FractureZone13,
            AchievementId::FractureZone14,
            AchievementId::FractureZone15,
            AchievementId::FractureZone16,
            AchievementId::FractureZone17,
            AchievementId::FractureZone18,
            AchievementId::FractureZone19,
            AchievementId::FractureZone20,
            AchievementId::FractureZone21,
            AchievementId::FractureZone22,
            AchievementId::FractureZone23,
            AchievementId::FractureZone24,
            AchievementId::FractureZone25,
            AchievementId::FractureZone26,
            AchievementId::FractureZone27,
            AchievementId::FractureZone28,
            AchievementId::FractureZone29,
            AchievementId::FractureZone30,
            AchievementId::AscensionI,
            AchievementId::AscensionII,
            AchievementId::AscensionIII,
            AchievementId::AscensionIV,
            AchievementId::AscensionV,
            AchievementId::AscensionVI,
            AchievementId::AscensionVII,
            AchievementId::AscensionVIII,
            AchievementId::AscensionIX,
            AchievementId::AscensionX,
            AchievementId::ChessNovice,
            AchievementId::ChessApprentice,
            AchievementId::ChessJourneyman,
            AchievementId::ChessMaster,
            AchievementId::MorrisNovice,
            AchievementId::MorrisApprentice,
            AchievementId::MorrisJourneyman,
            AchievementId::MorrisMaster,
            AchievementId::GomokuNovice,
            AchievementId::GomokuApprentice,
            AchievementId::GomokuJourneyman,
            AchievementId::GomokuMaster,
            AchievementId::MinesweeperNovice,
            AchievementId::MinesweeperApprentice,
            AchievementId::MinesweeperJourneyman,
            AchievementId::MinesweeperMaster,
            AchievementId::RuneNovice,
            AchievementId::RuneApprentice,
            AchievementId::RuneJourneyman,
            AchievementId::RuneMaster,
            AchievementId::GoNovice,
            AchievementId::GoApprentice,
            AchievementId::GoJourneyman,
            AchievementId::GoMaster,
            AchievementId::FlappyNovice,
            AchievementId::FlappyApprentice,
            AchievementId::FlappyJourneyman,
            AchievementId::FlappyMaster,
            AchievementId::SnakeNovice,
            AchievementId::SnakeApprentice,
            AchievementId::SnakeJourneyman,
            AchievementId::SnakeMaster,
            AchievementId::ContainmentBreachNovice,
            AchievementId::ContainmentBreachApprentice,
            AchievementId::ContainmentBreachJourneyman,
            AchievementId::ContainmentBreachMaster,
            AchievementId::SigilSurgeNovice,
            AchievementId::SigilSurgeApprentice,
            AchievementId::SigilSurgeJourneyman,
            AchievementId::SigilSurgeMaster,
            AchievementId::ShardFusionNovice,
            AchievementId::ShardFusionApprentice,
            AchievementId::ShardFusionJourneyman,
            AchievementId::ShardFusionMaster,
            AchievementId::GrandChampion,
            AchievementId::GoneFishing,
            AchievementId::FishermanI,
            AchievementId::FishermanII,
            AchievementId::FishermanIII,
            AchievementId::FishermanIV,
            AchievementId::StormLeviathan,
            AchievementId::FishCatcherI,
            AchievementId::FishCatcherII,
            AchievementId::FishCatcherIII,
            AchievementId::FishCatcherIV,
            AchievementId::FishCatcherV,
            AchievementId::FishCatcherVI,
            AchievementId::FishCatcherVII,
            AchievementId::FishCatcherVIII,
            AchievementId::FishCatcherIX,
            AchievementId::FishCatcherX,
            AchievementId::DungeonDiver,
            AchievementId::DungeonMasterI,
            AchievementId::DungeonMasterII,
            AchievementId::DungeonMasterIII,
            AchievementId::DungeonMasterIV,
            AchievementId::DungeonMasterV,
            AchievementId::DungeonMasterVI,
            AchievementId::DungeonMasterVII,
            AchievementId::DungeonMasterVIII,
            AchievementId::DungeonMasterIX,
            AchievementId::DungeonMasterX,
            AchievementId::HavenDiscovered,
            AchievementId::HavenBuilderI,
            AchievementId::HavenBuilderII,
            AchievementId::HavenArchitect,
            AchievementId::SoulforgeDiscovered,
            AchievementId::ApprenticeSmith,
            AchievementId::FullyTempered,
            AchievementId::JourneymanSmith,
            AchievementId::SoulforgeAdept,
            AchievementId::SoulforgeSavant,
            AchievementId::SoulforgeMaster,
            AchievementId::SoulforgeGrandmaster,
            AchievementId::SoulforgeAscendant,
            AchievementId::SoulConvergence,
            AchievementId::PersistentHammering,
            AchievementId::TheDeepDiscovered,
            AchievementId::FirstMissionComplete,
            AchievementId::DeepMissionsX,
            AchievementId::DeepMissionsXXV,
            AchievementId::DeepMissionsL,
            AchievementId::DeepMissionsC,
            AchievementId::FirstBreakthrough,
            AchievementId::Layer5Cleared,
            AchievementId::Layer10Cleared,
            AchievementId::Layer15Cleared,
            AchievementId::Layer20Cleared,
            AchievementId::Layer25Cleared,
            AchievementId::VoidExplorer,
            AchievementId::GuildRank2,
            AchievementId::GuildRank3,
            AchievementId::GuildRank4,
            AchievementId::GuildRank5,
            AchievementId::FirstMercLost,
            AchievementId::GatewayOpened,
            AchievementId::PowerCoreI,
            AchievementId::PowerCoreII,
            AchievementId::PowerCoreIII,
            AchievementId::PowerCoreIV,
            AchievementId::PowerCoreV,
            AchievementId::PowerCoreVI,
            AchievementId::LoomDiscovered,
            AchievementId::LoomPattern1,
            AchievementId::LoomPattern4,
            AchievementId::LoomPattern8,
            AchievementId::LoomPattern16,
            AchievementId::LoomPattern22,
            AchievementId::LoomPattern28,
        ];

        // Sanity-check: the slice above must itself be up-to-date.
        assert_eq!(
            all_ids.len(),
            AchievementId::VARIANT_COUNT,
            "The all_ids slice in this test has {} entries but AchievementId::VARIANT_COUNT is \
             {} — update the slice when adding/removing variants.",
            all_ids.len(),
            AchievementId::VARIANT_COUNT,
        );

        for id in all_ids {
            assert!(
                get_achievement_def(*id).is_some(),
                "AchievementId::{:?} has no definition in ALL_ACHIEVEMENTS",
                id
            );
        }
    }

    #[test]
    fn test_every_category_has_achievements() {
        for category in AchievementCategory::ALL {
            // Stats is a special tab that shows game statistics, not achievements
            if category == AchievementCategory::Stats {
                continue;
            }
            let achievements = get_achievements_by_category(category);
            assert!(
                !achievements.is_empty(),
                "Category {:?} has no achievements",
                category
            );
        }
    }

    #[test]
    fn test_no_duplicate_names() {
        use std::collections::HashSet;
        let mut names = HashSet::new();
        for achievement in ALL_ACHIEVEMENTS {
            assert!(
                names.insert(achievement.name),
                "Duplicate achievement name: {:?}",
                achievement.name
            );
        }
    }

    #[test]
    fn test_no_empty_descriptions_or_names() {
        for achievement in ALL_ACHIEVEMENTS {
            assert!(
                !achievement.name.is_empty(),
                "Achievement {:?} has empty name",
                achievement.id
            );
            assert!(
                !achievement.description.is_empty(),
                "Achievement {:?} has empty description",
                achievement.id
            );
            assert!(
                !achievement.icon.is_empty(),
                "Achievement {:?} has empty icon",
                achievement.id
            );
        }
    }

    #[test]
    fn test_description_lengths_reasonable() {
        for achievement in ALL_ACHIEVEMENTS {
            assert!(
                achievement.description.len() <= 200,
                "Achievement {:?} description is too long ({} chars): {}",
                achievement.id,
                achievement.description.len(),
                achievement.description
            );
            assert!(
                achievement.name.len() <= 50,
                "Achievement {:?} name is too long ({} chars): {}",
                achievement.id,
                achievement.name.len(),
                achievement.name
            );
        }
    }

    #[test]
    fn test_challenge_achievements_cover_all_game_types_and_difficulties() {
        let game_types = [
            (
                "Chess",
                vec![
                    AchievementId::ChessNovice,
                    AchievementId::ChessApprentice,
                    AchievementId::ChessJourneyman,
                    AchievementId::ChessMaster,
                ],
            ),
            (
                "Morris",
                vec![
                    AchievementId::MorrisNovice,
                    AchievementId::MorrisApprentice,
                    AchievementId::MorrisJourneyman,
                    AchievementId::MorrisMaster,
                ],
            ),
            (
                "Gomoku",
                vec![
                    AchievementId::GomokuNovice,
                    AchievementId::GomokuApprentice,
                    AchievementId::GomokuJourneyman,
                    AchievementId::GomokuMaster,
                ],
            ),
            (
                "Minesweeper",
                vec![
                    AchievementId::MinesweeperNovice,
                    AchievementId::MinesweeperApprentice,
                    AchievementId::MinesweeperJourneyman,
                    AchievementId::MinesweeperMaster,
                ],
            ),
            (
                "Rune",
                vec![
                    AchievementId::RuneNovice,
                    AchievementId::RuneApprentice,
                    AchievementId::RuneJourneyman,
                    AchievementId::RuneMaster,
                ],
            ),
            (
                "Go",
                vec![
                    AchievementId::GoNovice,
                    AchievementId::GoApprentice,
                    AchievementId::GoJourneyman,
                    AchievementId::GoMaster,
                ],
            ),
            (
                "Flappy",
                vec![
                    AchievementId::FlappyNovice,
                    AchievementId::FlappyApprentice,
                    AchievementId::FlappyJourneyman,
                    AchievementId::FlappyMaster,
                ],
            ),
            (
                "Snake",
                vec![
                    AchievementId::SnakeNovice,
                    AchievementId::SnakeApprentice,
                    AchievementId::SnakeJourneyman,
                    AchievementId::SnakeMaster,
                ],
            ),
            (
                "Containment Breach",
                vec![
                    AchievementId::ContainmentBreachNovice,
                    AchievementId::ContainmentBreachApprentice,
                    AchievementId::ContainmentBreachJourneyman,
                    AchievementId::ContainmentBreachMaster,
                ],
            ),
            (
                "Sigil Surge",
                vec![
                    AchievementId::SigilSurgeNovice,
                    AchievementId::SigilSurgeApprentice,
                    AchievementId::SigilSurgeJourneyman,
                    AchievementId::SigilSurgeMaster,
                ],
            ),
            (
                "Shard Fusion",
                vec![
                    AchievementId::ShardFusionNovice,
                    AchievementId::ShardFusionApprentice,
                    AchievementId::ShardFusionJourneyman,
                    AchievementId::ShardFusionMaster,
                ],
            ),
        ];

        for (game_name, ids) in &game_types {
            assert_eq!(
                ids.len(),
                4,
                "{} should have exactly 4 difficulty achievements",
                game_name
            );
            for id in ids {
                let def = get_achievement_def(*id);
                assert!(
                    def.is_some(),
                    "{} achievement {:?} missing from definitions",
                    game_name,
                    id
                );
                let def = def.unwrap();
                assert_eq!(
                    def.category,
                    AchievementCategory::Challenges,
                    "{} achievement {:?} should be in Challenges category",
                    game_name,
                    id
                );
            }
        }
    }

    #[test]
    fn test_zone_achievements_cover_all_ten_zones() {
        let zone_ids = [
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

        for (zone_id, achievement_id) in zone_ids {
            let def = get_achievement_def(achievement_id).unwrap_or_else(|| {
                panic!("Zone {} achievement {:?} missing", zone_id, achievement_id)
            });
            assert_eq!(def.category, AchievementCategory::Progression);
            assert!(
                def.description.contains(&format!("Zone {}", zone_id)),
                "Zone {} achievement description should mention Zone {}: got {:?}",
                zone_id,
                zone_id,
                def.description
            );
        }
    }

    #[test]
    fn test_slayer_milestone_descriptions_monotonically_increasing() {
        let slayer_ids = [
            AchievementId::SlayerI,
            AchievementId::SlayerII,
            AchievementId::SlayerIII,
            AchievementId::SlayerIV,
            AchievementId::SlayerV,
            AchievementId::SlayerVI,
            AchievementId::SlayerVII,
            AchievementId::SlayerVIII,
            AchievementId::SlayerIX,
        ];

        let mut prev_number = 0u64;
        for id in slayer_ids {
            let def = get_achievement_def(id).unwrap();
            let number = extract_number_from_description(def.description);
            assert!(
                number > prev_number,
                "Slayer milestones should be strictly increasing: {:?} ({}) <= {}",
                id,
                number,
                prev_number
            );
            prev_number = number;
        }
    }

    #[test]
    fn test_boss_hunter_milestone_descriptions_monotonically_increasing() {
        // BossHunterI is "Defeat your first boss" (no numeric milestone)
        // BossHunterII onwards have increasing numeric milestones
        let boss_ids = [
            AchievementId::BossHunterII,
            AchievementId::BossHunterIII,
            AchievementId::BossHunterIV,
            AchievementId::BossHunterV,
            AchievementId::BossHunterVI,
            AchievementId::BossHunterVII,
            AchievementId::BossHunterVIII,
        ];

        let mut prev_number = 0u64;
        for id in boss_ids {
            let def = get_achievement_def(id).unwrap();
            let number = extract_number_from_description(def.description);
            assert!(
                number > prev_number,
                "BossHunter milestones should be strictly increasing: {:?} ({}) <= {}",
                id,
                number,
                prev_number
            );
            prev_number = number;
        }
    }

    #[test]
    fn test_level_milestone_descriptions_monotonically_increasing() {
        let level_ids = [
            AchievementId::Level10,
            AchievementId::Level25,
            AchievementId::Level50,
            AchievementId::Level100,
            AchievementId::Level150,
            AchievementId::Level200,
            AchievementId::Level250,
            AchievementId::Level500,
            AchievementId::Level750,
            AchievementId::Level1000,
            AchievementId::Level1500,
        ];

        let mut prev_number = 0u64;
        for id in level_ids {
            let def = get_achievement_def(id).unwrap();
            let number = extract_number_from_description(def.description);
            assert!(
                number > prev_number,
                "Level milestones should be strictly increasing: {:?} ({}) <= {}",
                id,
                number,
                prev_number
            );
            prev_number = number;
        }
    }

    fn extract_number_from_description(desc: &str) -> u64 {
        let cleaned: String = desc
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == ',')
            .collect();
        let without_commas: String = cleaned.chars().filter(|c| *c != ',').collect();
        without_commas.parse().unwrap_or(0)
    }
}
