//! Static achievement definitions.

use super::types::{AchievementCategory, AchievementDef, AchievementId};

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
        icon: "💀",
    },
    AchievementDef {
        id: AchievementId::SlayerII,
        name: "Slayer II",
        description: "Defeat 500 enemies",
        category: AchievementCategory::Combat,
        icon: "💀",
    },
    AchievementDef {
        id: AchievementId::SlayerIII,
        name: "Slayer III",
        description: "Defeat 1,000 enemies",
        category: AchievementCategory::Combat,
        icon: "💀",
    },
    AchievementDef {
        id: AchievementId::SlayerIV,
        name: "Slayer IV",
        description: "Defeat 5,000 enemies",
        category: AchievementCategory::Combat,
        icon: "💀",
    },
    AchievementDef {
        id: AchievementId::SlayerV,
        name: "Slayer V",
        description: "Defeat 10,000 enemies",
        category: AchievementCategory::Combat,
        icon: "☠️",
    },
    AchievementDef {
        id: AchievementId::SlayerVI,
        name: "Slayer VI",
        description: "Defeat 50,000 enemies",
        category: AchievementCategory::Combat,
        icon: "☠️",
    },
    AchievementDef {
        id: AchievementId::SlayerVII,
        name: "Slayer VII",
        description: "Defeat 100,000 enemies",
        category: AchievementCategory::Combat,
        icon: "☠️",
    },
    AchievementDef {
        id: AchievementId::SlayerVIII,
        name: "Slayer VIII",
        description: "Defeat 500,000 enemies",
        category: AchievementCategory::Combat,
        icon: "⚔️",
    },
    AchievementDef {
        id: AchievementId::SlayerIX,
        name: "Slayer IX",
        description: "Defeat 1,000,000 enemies",
        category: AchievementCategory::Combat,
        icon: "⚔️",
    },
    AchievementDef {
        id: AchievementId::SlayerX,
        name: "Slayer X",
        description: "Defeat 2,500,000 enemies",
        category: AchievementCategory::Combat,
        icon: "⚔️",
    },
    AchievementDef {
        id: AchievementId::SlayerXI,
        name: "Slayer XI",
        description: "Defeat 10,000,000 enemies",
        category: AchievementCategory::Combat,
        icon: "⚔️",
    },
    AchievementDef {
        id: AchievementId::SlayerXII,
        name: "Slayer XII",
        description: "Defeat 50,000,000 enemies",
        category: AchievementCategory::Combat,
        icon: "⚔️",
    },
    AchievementDef {
        id: AchievementId::SlayerXIII,
        name: "Harbinger",
        description: "Defeat 100,000,000 enemies - The battlefield trembles at your approach",
        category: AchievementCategory::Combat,
        icon: "⚜️",
    },
    AchievementDef {
        id: AchievementId::SlayerXIV,
        name: "Reaper",
        description: "Defeat 500,000,000 enemies - Even death takes note of your work",
        category: AchievementCategory::Combat,
        icon: "🔱",
    },
    AchievementDef {
        id: AchievementId::SlayerXV,
        name: "Death Incarnate",
        description: "Defeat 1,000,000,000 enemies - You are the silence between heartbeats",
        category: AchievementCategory::Combat,
        icon: "🐉",
    },
    // ═══════════════════════════════════════════════════════════════
    // COMBAT ACHIEVEMENTS - BOSS KILLS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::BossHunterI,
        name: "Boss Hunter I",
        description: "Defeat your first boss",
        category: AchievementCategory::Combat,
        icon: "👹",
    },
    AchievementDef {
        id: AchievementId::BossHunterII,
        name: "Boss Hunter II",
        description: "Defeat 10 bosses",
        category: AchievementCategory::Combat,
        icon: "👹",
    },
    AchievementDef {
        id: AchievementId::BossHunterIII,
        name: "Boss Hunter III",
        description: "Defeat 50 bosses",
        category: AchievementCategory::Combat,
        icon: "👹",
    },
    AchievementDef {
        id: AchievementId::BossHunterIV,
        name: "Boss Hunter IV",
        description: "Defeat 100 bosses",
        category: AchievementCategory::Combat,
        icon: "👿",
    },
    AchievementDef {
        id: AchievementId::BossHunterV,
        name: "Boss Hunter V",
        description: "Defeat 500 bosses",
        category: AchievementCategory::Combat,
        icon: "👿",
    },
    AchievementDef {
        id: AchievementId::BossHunterVI,
        name: "Boss Hunter VI",
        description: "Defeat 1,000 bosses",
        category: AchievementCategory::Combat,
        icon: "😈",
    },
    AchievementDef {
        id: AchievementId::BossHunterVII,
        name: "Boss Hunter VII",
        description: "Defeat 5,000 bosses",
        category: AchievementCategory::Combat,
        icon: "😈",
    },
    AchievementDef {
        id: AchievementId::BossHunterVIII,
        name: "Boss Hunter VIII",
        description: "Defeat 10,000 bosses",
        category: AchievementCategory::Combat,
        icon: "👑",
    },
    AchievementDef {
        id: AchievementId::BossHunterIX,
        name: "Boss Hunter IX",
        description: "Defeat 25,000 bosses",
        category: AchievementCategory::Combat,
        icon: "👑",
    },
    AchievementDef {
        id: AchievementId::BossHunterX,
        name: "Boss Hunter X",
        description: "Defeat 75,000 bosses",
        category: AchievementCategory::Combat,
        icon: "👑",
    },
    AchievementDef {
        id: AchievementId::BossHunterXI,
        name: "Boss Hunter XI",
        description: "Defeat 250,000 bosses",
        category: AchievementCategory::Combat,
        icon: "👑",
    },
    AchievementDef {
        id: AchievementId::BossHunterXII,
        name: "Boss Hunter XII",
        description: "Defeat 750,000 bosses",
        category: AchievementCategory::Combat,
        icon: "👑",
    },
    AchievementDef {
        id: AchievementId::BossHunterXIII,
        name: "Titan Breaker",
        description: "Defeat 2,500,000 bosses - Even gods kneel before you",
        category: AchievementCategory::Combat,
        icon: "💀",
    },
    AchievementDef {
        id: AchievementId::BossHunterXIV,
        name: "Worldender",
        description: "Defeat 5,000,000 bosses - Worlds are born and die by your hand",
        category: AchievementCategory::Combat,
        icon: "☠️",
    },
    AchievementDef {
        id: AchievementId::BossHunterXV,
        name: "The Absolute",
        description: "Defeat 10,000,000 bosses - Beyond power. Beyond reckoning. Beyond all.",
        category: AchievementCategory::Combat,
        icon: "🩸",
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
    },
    AchievementDef {
        id: AchievementId::Level25,
        name: "Adventurer",
        description: "Reach level 25",
        category: AchievementCategory::Level,
        icon: "\u{1f948}",
    },
    AchievementDef {
        id: AchievementId::Level50,
        name: "Veteran",
        description: "Reach level 50",
        category: AchievementCategory::Level,
        icon: "\u{1f947}",
    },
    AchievementDef {
        id: AchievementId::Level100,
        name: "Centurion",
        description: "Reach level 100",
        category: AchievementCategory::Level,
        icon: "\u{1f48e}",
    },
    AchievementDef {
        id: AchievementId::Level150,
        name: "Elite",
        description: "Reach level 150",
        category: AchievementCategory::Level,
        icon: "\u{1f4a0}",
    },
    AchievementDef {
        id: AchievementId::Level200,
        name: "Champion",
        description: "Reach level 200",
        category: AchievementCategory::Level,
        icon: "\u{2764}\u{fe0f}",
    },
    AchievementDef {
        id: AchievementId::Level250,
        name: "Legendary",
        description: "Reach level 250",
        category: AchievementCategory::Level,
        icon: "\u{1f499}",
    },
    AchievementDef {
        id: AchievementId::Level500,
        name: "Mythic",
        description: "Reach level 500",
        category: AchievementCategory::Level,
        icon: "\u{1f49a}",
    },
    AchievementDef {
        id: AchievementId::Level750,
        name: "Demigod",
        description: "Reach level 750",
        category: AchievementCategory::Level,
        icon: "\u{1f5a4}",
    },
    AchievementDef {
        id: AchievementId::Level1000,
        name: "Immortal",
        description: "Reach level 1000",
        category: AchievementCategory::Level,
        icon: "\u{1f49c}",
    },
    AchievementDef {
        id: AchievementId::Level1500,
        name: "Transcendent",
        description: "Reach level 1500 - The universe bends to your will",
        category: AchievementCategory::Level,
        icon: "\u{267e}\u{fe0f}",
    },
    // ═══════════════════════════════════════════════════════════════
    // PRESTIGE ACHIEVEMENTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::FirstPrestige,
        name: "Rebirth",
        description: "Prestige for the first time",
        category: AchievementCategory::Level,
        icon: "🔄",
    },
    AchievementDef {
        id: AchievementId::PrestigeV,
        name: "Bronze Rank",
        description: "Reach Prestige Rank 5",
        category: AchievementCategory::Level,
        icon: "🥉",
    },
    AchievementDef {
        id: AchievementId::PrestigeX,
        name: "Silver Rank",
        description: "Reach Prestige Rank 10",
        category: AchievementCategory::Level,
        icon: "🥈",
    },
    AchievementDef {
        id: AchievementId::PrestigeXV,
        name: "Gold Rank",
        description: "Reach Prestige Rank 15",
        category: AchievementCategory::Level,
        icon: "🥇",
    },
    AchievementDef {
        id: AchievementId::PrestigeXX,
        name: "Platinum Rank",
        description: "Reach Prestige Rank 20",
        category: AchievementCategory::Level,
        icon: "💎",
    },
    AchievementDef {
        id: AchievementId::PrestigeXXV,
        name: "Diamond Rank",
        description: "Reach Prestige Rank 25",
        category: AchievementCategory::Level,
        icon: "💠",
    },
    AchievementDef {
        id: AchievementId::PrestigeXXX,
        name: "Ruby Rank",
        description: "Reach Prestige Rank 30",
        category: AchievementCategory::Level,
        icon: "❤️",
    },
    AchievementDef {
        id: AchievementId::PrestigeXL,
        name: "Sapphire Rank",
        description: "Reach Prestige Rank 40",
        category: AchievementCategory::Level,
        icon: "💙",
    },
    AchievementDef {
        id: AchievementId::PrestigeL,
        name: "Emerald Rank",
        description: "Reach Prestige Rank 50",
        category: AchievementCategory::Level,
        icon: "💚",
    },
    AchievementDef {
        id: AchievementId::PrestigeLXX,
        name: "Obsidian Rank",
        description: "Reach Prestige Rank 70",
        category: AchievementCategory::Level,
        icon: "🖤",
    },
    AchievementDef {
        id: AchievementId::PrestigeXC,
        name: "Celestial Rank",
        description: "Reach Prestige Rank 90",
        category: AchievementCategory::Level,
        icon: "💜",
    },
    AchievementDef {
        id: AchievementId::Eternal,
        name: "Eternal",
        description: "Reach Prestige Rank 100 - Your legend echoes through eternity",
        category: AchievementCategory::Level,
        icon: "♾️",
    },
    // Zone completion achievements (one per zone)
    AchievementDef {
        id: AchievementId::Zone1Complete,
        name: "Meadow Wanderer",
        description: "Clear Zone 1: Meadow",
        category: AchievementCategory::Progression,
        icon: "🌻",
    },
    AchievementDef {
        id: AchievementId::Zone2Complete,
        name: "Forest Walker",
        description: "Clear Zone 2: Dark Forest",
        category: AchievementCategory::Progression,
        icon: "🌲",
    },
    AchievementDef {
        id: AchievementId::Zone3Complete,
        name: "Peak Climber",
        description: "Clear Zone 3: Mountain Pass",
        category: AchievementCategory::Progression,
        icon: "🏔️",
    },
    AchievementDef {
        id: AchievementId::Zone4Complete,
        name: "Ruin Explorer",
        description: "Clear Zone 4: Ancient Ruins",
        category: AchievementCategory::Progression,
        icon: "🏛️",
    },
    AchievementDef {
        id: AchievementId::Zone5Complete,
        name: "Volcano Conqueror",
        description: "Clear Zone 5: Volcanic Wastes",
        category: AchievementCategory::Progression,
        icon: "🌋",
    },
    AchievementDef {
        id: AchievementId::Zone6Complete,
        name: "Frost Survivor",
        description: "Clear Zone 6: Frozen Tundra",
        category: AchievementCategory::Progression,
        icon: "❄️",
    },
    AchievementDef {
        id: AchievementId::Zone7Complete,
        name: "Crystal Seeker",
        description: "Clear Zone 7: Crystal Caverns",
        category: AchievementCategory::Progression,
        icon: "💎",
    },
    AchievementDef {
        id: AchievementId::Zone8Complete,
        name: "Deep Diver",
        description: "Clear Zone 8: Sunken Kingdom",
        category: AchievementCategory::Progression,
        icon: "🌊",
    },
    AchievementDef {
        id: AchievementId::Zone9Complete,
        name: "Sky Lord",
        description: "Clear Zone 9: Floating Isles",
        category: AchievementCategory::Progression,
        icon: "☁️",
    },
    AchievementDef {
        id: AchievementId::Zone10Complete,
        name: "Citadel Conqueror",
        description: "Clear Zone 10: Storm Citadel",
        category: AchievementCategory::Progression,
        icon: "⛈️",
    },
    AchievementDef {
        id: AchievementId::TheStormbreaker,
        name: "The Stormbreaker",
        description: "Forge the legendary Stormbreaker at the Haven forge",
        category: AchievementCategory::Progression,
        icon: "⚡",
    },
    AchievementDef {
        id: AchievementId::StormsEnd,
        name: "Storm's End",
        description: "Defeat the final boss of Zone 10: Storm Citadel",
        category: AchievementCategory::Progression,
        icon: "🏆",
    },
    // The Expanse achievement
    AchievementDef {
        id: AchievementId::BeyondInfinity,
        name: "Beyond Infinity",
        description: "Complete a cycle of The Expanse \u{2014} there is nothing left to conquer, only endlessness",
        category: AchievementCategory::Progression,
        icon: "\u{267e}\u{fe0f}",
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
    },
    AchievementDef {
        id: AchievementId::ChessApprentice,
        name: "Chess Apprentice",
        description: "Win chess on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "♟️",
    },
    AchievementDef {
        id: AchievementId::ChessJourneyman,
        name: "Chess Journeyman",
        description: "Win chess on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "♟️",
    },
    AchievementDef {
        id: AchievementId::ChessMaster,
        name: "Chess Master",
        description: "Win chess on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "♛",
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
    },
    AchievementDef {
        id: AchievementId::MorrisApprentice,
        name: "Morris Apprentice",
        description: "Win Morris on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚪",
    },
    AchievementDef {
        id: AchievementId::MorrisJourneyman,
        name: "Morris Journeyman",
        description: "Win Morris on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚪",
    },
    AchievementDef {
        id: AchievementId::MorrisMaster,
        name: "Morris Master",
        description: "Win Morris on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚪",
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
    },
    AchievementDef {
        id: AchievementId::GomokuApprentice,
        name: "Gomoku Apprentice",
        description: "Win Gomoku on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚫",
    },
    AchievementDef {
        id: AchievementId::GomokuJourneyman,
        name: "Gomoku Journeyman",
        description: "Win Gomoku on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚫",
    },
    AchievementDef {
        id: AchievementId::GomokuMaster,
        name: "Gomoku Master",
        description: "Win Gomoku on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚫",
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
    },
    AchievementDef {
        id: AchievementId::MinesweeperApprentice,
        name: "Minesweeper Apprentice",
        description: "Win Minesweeper on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "💣",
    },
    AchievementDef {
        id: AchievementId::MinesweeperJourneyman,
        name: "Minesweeper Journeyman",
        description: "Win Minesweeper on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "💣",
    },
    AchievementDef {
        id: AchievementId::MinesweeperMaster,
        name: "Minesweeper Master",
        description: "Win Minesweeper on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "💣",
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
    },
    AchievementDef {
        id: AchievementId::RuneApprentice,
        name: "Rune Apprentice",
        description: "Win Rune on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "🔮",
    },
    AchievementDef {
        id: AchievementId::RuneJourneyman,
        name: "Rune Journeyman",
        description: "Win Rune on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "🔮",
    },
    AchievementDef {
        id: AchievementId::RuneMaster,
        name: "Rune Master",
        description: "Win Rune on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "🔮",
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
    },
    AchievementDef {
        id: AchievementId::GoApprentice,
        name: "Go Apprentice",
        description: "Win Go on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "🟤",
    },
    AchievementDef {
        id: AchievementId::GoJourneyman,
        name: "Go Journeyman",
        description: "Win Go on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "🟤",
    },
    AchievementDef {
        id: AchievementId::GoMaster,
        name: "Go Master",
        description: "Win Go on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "🟤",
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
    },
    AchievementDef {
        id: AchievementId::FlappyApprentice,
        name: "Skyward Apprentice",
        description: "Win Skyward Gauntlet on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "◆",
    },
    AchievementDef {
        id: AchievementId::FlappyJourneyman,
        name: "Skyward Journeyman",
        description: "Win Skyward Gauntlet on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "◆",
    },
    AchievementDef {
        id: AchievementId::FlappyMaster,
        name: "Skyward Master",
        description: "Win Skyward Gauntlet on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "›",
    },
    // ═══════════════════════════════════════════════════════════════
    // CHALLENGE ACHIEVEMENTS - SNAKE
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::SnakeNovice,
        name: "Snake Novice",
        description: "Win Snake on Novice difficulty",
        category: AchievementCategory::Challenges,
        icon: "~",
    },
    AchievementDef {
        id: AchievementId::SnakeApprentice,
        name: "Snake Apprentice",
        description: "Win Snake on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "~",
    },
    AchievementDef {
        id: AchievementId::SnakeJourneyman,
        name: "Snake Journeyman",
        description: "Win Snake on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "~",
    },
    AchievementDef {
        id: AchievementId::SnakeMaster,
        name: "Snake Master",
        description: "Win Snake on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "~",
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
    },
    AchievementDef {
        id: AchievementId::ContainmentBreachApprentice,
        name: "Containment Breach Apprentice",
        description: "Win Containment Breach on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "▣",
    },
    AchievementDef {
        id: AchievementId::ContainmentBreachJourneyman,
        name: "Containment Breach Journeyman",
        description: "Win Containment Breach on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "▣",
    },
    AchievementDef {
        id: AchievementId::ContainmentBreachMaster,
        name: "Containment Breach Master",
        description: "Win Containment Breach on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "▣",
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
    },
    AchievementDef {
        id: AchievementId::SigilSurgeApprentice,
        name: "Sigil Surge Apprentice",
        description: "Win Sigil Surge on Apprentice difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚡",
    },
    AchievementDef {
        id: AchievementId::SigilSurgeJourneyman,
        name: "Sigil Surge Journeyman",
        description: "Win Sigil Surge on Journeyman difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚡",
    },
    AchievementDef {
        id: AchievementId::SigilSurgeMaster,
        name: "Sigil Surge Master",
        description: "Win Sigil Surge on Master difficulty",
        category: AchievementCategory::Challenges,
        icon: "⚡",
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
    },
    AchievementDef {
        id: AchievementId::FishermanI,
        name: "Fisherman I",
        description: "Reach fishing rank 10",
        category: AchievementCategory::Exploration,
        icon: "🐟",
    },
    AchievementDef {
        id: AchievementId::FishermanII,
        name: "Fisherman II",
        description: "Reach fishing rank 20",
        category: AchievementCategory::Exploration,
        icon: "🐠",
    },
    AchievementDef {
        id: AchievementId::FishermanIII,
        name: "Fisherman III",
        description: "Reach fishing rank 30 (base max)",
        category: AchievementCategory::Exploration,
        icon: "🐡",
    },
    AchievementDef {
        id: AchievementId::FishermanIV,
        name: "Fisherman IV",
        description: "Reach fishing rank 40 (requires Fishing Dock T4)",
        category: AchievementCategory::Exploration,
        icon: "\u{1f433}",
    },
    AchievementDef {
        id: AchievementId::StormLeviathan,
        name: "Storm Leviathan",
        description: "Catch the legendary Storm Leviathan",
        category: AchievementCategory::Exploration,
        icon: "\u{1f30a}",
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
    },
    AchievementDef {
        id: AchievementId::FishCatcherII,
        name: "Fish Catcher II",
        description: "Catch 1,000 fish",
        category: AchievementCategory::Exploration,
        icon: "🐠",
    },
    AchievementDef {
        id: AchievementId::FishCatcherIII,
        name: "Fish Catcher III",
        description: "Catch 10,000 fish",
        category: AchievementCategory::Exploration,
        icon: "🐡",
    },
    AchievementDef {
        id: AchievementId::FishCatcherIV,
        name: "Fish Catcher IV",
        description: "Catch 100,000 fish",
        category: AchievementCategory::Exploration,
        icon: "🐋",
    },
    AchievementDef {
        id: AchievementId::FishCatcherV,
        name: "Fish Catcher V",
        description: "Catch 500,000 fish",
        category: AchievementCategory::Exploration,
        icon: "\u{1f40b}",
    },
    AchievementDef {
        id: AchievementId::FishCatcherVI,
        name: "Fish Catcher VI",
        description: "Catch 1,000,000 fish",
        category: AchievementCategory::Exploration,
        icon: "\u{1f40b}",
    },
    AchievementDef {
        id: AchievementId::FishCatcherVII,
        name: "Fish Catcher VII",
        description: "Catch 5,000,000 fish",
        category: AchievementCategory::Exploration,
        icon: "\u{1f40b}",
    },
    AchievementDef {
        id: AchievementId::FishCatcherVIII,
        name: "Leviathan's Rival",
        description: "Catch 10,000,000 fish - Even the Leviathan watches its depths",
        category: AchievementCategory::Exploration,
        icon: "\u{1f988}",
    },
    AchievementDef {
        id: AchievementId::FishCatcherIX,
        name: "Poseidon's Hand",
        description: "Catch 50,000,000 fish - Every wave carries your legend",
        category: AchievementCategory::Exploration,
        icon: "\u{1f30a}",
    },
    AchievementDef {
        id: AchievementId::FishCatcherX,
        name: "Lord of the Deep",
        description: "Catch 100,000,000 fish - You are the tide. You are the deep.",
        category: AchievementCategory::Exploration,
        icon: "\u{1f531}",
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
    },
    AchievementDef {
        id: AchievementId::DungeonMasterI,
        name: "Dungeon Master I",
        description: "Complete 10 dungeons",
        category: AchievementCategory::Exploration,
        icon: "\u{1f5dd}\u{fe0f}",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterII,
        name: "Dungeon Master II",
        description: "Complete 50 dungeons",
        category: AchievementCategory::Exploration,
        icon: "\u{1f56f}\u{fe0f}",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterIII,
        name: "Dungeon Master III",
        description: "Complete 100 dungeons",
        category: AchievementCategory::Exploration,
        icon: "\u{1f5e1}\u{fe0f}",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterIV,
        name: "Dungeon Master IV",
        description: "Complete 1,000 dungeons",
        category: AchievementCategory::Exploration,
        icon: "\u{2694}\u{fe0f}",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterV,
        name: "Dungeon Master V",
        description: "Complete 5,000 dungeons",
        category: AchievementCategory::Exploration,
        icon: "\u{1f6e1}\u{fe0f}",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterVI,
        name: "Dungeon Master VI",
        description: "Complete 10,000 dungeons",
        category: AchievementCategory::Exploration,
        icon: "\u{1f480}",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterVII,
        name: "Dungeon Master VII",
        description: "Complete 25,000 dungeons",
        category: AchievementCategory::Exploration,
        icon: "\u{1f3f0}",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterVIII,
        name: "Labyrinth Lord",
        description: "Complete 100,000 dungeons - Every maze unravels at your touch",
        category: AchievementCategory::Exploration,
        icon: "\u{1f409}",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterIX,
        name: "Abyss Walker",
        description: "Complete 500,000 dungeons - The forgotten halls remember only you",
        category: AchievementCategory::Exploration,
        icon: "\u{269c}\u{fe0f}",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterX,
        name: "The Undying Delver",
        description:
            "Complete 1,000,000 dungeons - The dungeon is not a place \u{2014} it is your kingdom",
        category: AchievementCategory::Exploration,
        icon: "\u{1f451}",
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
    },
    AchievementDef {
        id: AchievementId::HavenBuilderI,
        name: "Haven Builder I",
        description: "Upgrade all Haven rooms to Tier 1",
        category: AchievementCategory::Exploration,
        icon: "\u{1f3e0}",
    },
    AchievementDef {
        id: AchievementId::HavenBuilderII,
        name: "Haven Builder II",
        description: "Upgrade all Haven rooms to Tier 2",
        category: AchievementCategory::Exploration,
        icon: "\u{1f3f0}",
    },
    AchievementDef {
        id: AchievementId::HavenArchitect,
        name: "Haven Architect",
        description: "Upgrade all Haven rooms to Tier 3",
        category: AchievementCategory::Exploration,
        icon: "\u{1f3db}\u{fe0f}",
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
    },
    AchievementDef {
        id: AchievementId::ApprenticeSmith,
        name: "Apprentice Smith",
        description: "Enhance any equipment to +1",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
    },
    AchievementDef {
        id: AchievementId::FullyTempered,
        name: "Fully Tempered",
        description: "Enhance all 7 equipment slots to +4",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
    },
    AchievementDef {
        id: AchievementId::JourneymanSmith,
        name: "Journeyman Smith",
        description: "Enhance any equipment to +5",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
    },
    AchievementDef {
        id: AchievementId::SoulforgeAdept,
        name: "Soulforge Adept",
        description: "Enhance any equipment to +6",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
    },
    AchievementDef {
        id: AchievementId::SoulforgeSavant,
        name: "Soulforge Savant",
        description: "Enhance any equipment to +7",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
    },
    AchievementDef {
        id: AchievementId::SoulforgeMaster,
        name: "Soulforge Master",
        description: "Enhance any equipment to +8",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
    },
    AchievementDef {
        id: AchievementId::SoulforgeGrandmaster,
        name: "Soulforge Grandmaster",
        description: "Enhance any equipment to +9",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
    },
    AchievementDef {
        id: AchievementId::MasterSmith,
        name: "Master Smith",
        description: "Enhance any equipment to +10",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
    },
    AchievementDef {
        id: AchievementId::SoulConvergence,
        name: "Soul Convergence",
        description: "Enhance all 7 equipment slots to +7",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
    },
    AchievementDef {
        id: AchievementId::PersistentHammering,
        name: "Persistent Hammering",
        description: "Attempt 100 enhancements",
        category: AchievementCategory::Progression,
        icon: "\u{2692}",
    },
];

/// Get the definition for a specific achievement.
pub fn get_achievement_def(id: AchievementId) -> Option<&'static AchievementDef> {
    ALL_ACHIEVEMENTS.iter().find(|a| a.id == id)
}

/// Get achievements filtered by category.
pub fn get_achievements_by_category(category: AchievementCategory) -> Vec<&'static AchievementDef> {
    ALL_ACHIEVEMENTS
        .iter()
        .filter(|a| a.category == category)
        .collect()
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
    }

    #[test]
    fn test_every_achievement_id_variant_has_definition() {
        let all_ids: Vec<AchievementId> = vec![
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
            AchievementId::MasterSmith,
            AchievementId::SoulConvergence,
            AchievementId::PersistentHammering,
        ];

        for id in &all_ids {
            assert!(
                get_achievement_def(*id).is_some(),
                "AchievementId::{:?} has no definition in ALL_ACHIEVEMENTS",
                id
            );
        }

        assert_eq!(
            all_ids.len(),
            ALL_ACHIEVEMENTS.len(),
            "Mismatch between AchievementId variants ({}) and ALL_ACHIEVEMENTS entries ({})",
            all_ids.len(),
            ALL_ACHIEVEMENTS.len()
        );
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
