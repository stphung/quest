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
    // ═══════════════════════════════════════════════════════════════
    // LEVEL ACHIEVEMENTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::Level10,
        name: "Getting Started",
        description: "Reach level 10",
        category: AchievementCategory::Level,
        icon: "📈",
    },
    AchievementDef {
        id: AchievementId::Level25,
        name: "Adventurer",
        description: "Reach level 25",
        category: AchievementCategory::Level,
        icon: "📈",
    },
    AchievementDef {
        id: AchievementId::Level50,
        name: "Veteran",
        description: "Reach level 50",
        category: AchievementCategory::Level,
        icon: "📈",
    },
    AchievementDef {
        id: AchievementId::Level100,
        name: "Centurion",
        description: "Reach level 100",
        category: AchievementCategory::Level,
        icon: "🌟",
    },
    AchievementDef {
        id: AchievementId::Level150,
        name: "Elite",
        description: "Reach level 150",
        category: AchievementCategory::Level,
        icon: "🌟",
    },
    AchievementDef {
        id: AchievementId::Level200,
        name: "Champion",
        description: "Reach level 200",
        category: AchievementCategory::Level,
        icon: "🌟",
    },
    AchievementDef {
        id: AchievementId::Level250,
        name: "Legendary",
        description: "Reach level 250",
        category: AchievementCategory::Level,
        icon: "⭐",
    },
    AchievementDef {
        id: AchievementId::Level500,
        name: "Mythic",
        description: "Reach level 500",
        category: AchievementCategory::Level,
        icon: "⭐",
    },
    AchievementDef {
        id: AchievementId::Level750,
        name: "Demigod",
        description: "Reach level 750",
        category: AchievementCategory::Level,
        icon: "✨",
    },
    AchievementDef {
        id: AchievementId::Level1000,
        name: "Immortal",
        description: "Reach level 1000",
        category: AchievementCategory::Level,
        icon: "✨",
    },
    AchievementDef {
        id: AchievementId::Level1500,
        name: "Transcendent",
        description: "Reach level 1500 - The universe bends to your will",
        category: AchievementCategory::Level,
        icon: "💫",
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
    // The Expanse cycle achievements
    AchievementDef {
        id: AchievementId::ExpanseCycleI,
        name: "Beyond Infinity I",
        description: "Complete 1 cycle of The Expanse",
        category: AchievementCategory::Progression,
        icon: "♾️",
    },
    AchievementDef {
        id: AchievementId::ExpanseCycleII,
        name: "Beyond Infinity II",
        description: "Complete 100 cycles of The Expanse",
        category: AchievementCategory::Progression,
        icon: "♾️",
    },
    AchievementDef {
        id: AchievementId::ExpanseCycleIII,
        name: "Beyond Infinity III",
        description: "Complete 1,000 cycles of The Expanse",
        category: AchievementCategory::Progression,
        icon: "♾️",
    },
    AchievementDef {
        id: AchievementId::ExpanseCycleIV,
        name: "Beyond Infinity IV",
        description: "Complete 10,000 cycles of The Expanse",
        category: AchievementCategory::Progression,
        icon: "♾️",
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
        icon: "🐋",
    },
    AchievementDef {
        id: AchievementId::StormLeviathan,
        name: "Storm Leviathan",
        description: "Catch the legendary Storm Leviathan",
        category: AchievementCategory::Exploration,
        icon: "🐋",
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
    // ═══════════════════════════════════════════════════════════════
    // DUNGEON ACHIEVEMENTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::DungeonDiver,
        name: "Dungeon Diver",
        description: "Complete your first dungeon",
        category: AchievementCategory::Exploration,
        icon: "🏰",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterI,
        name: "Dungeon Master I",
        description: "Complete 10 dungeons",
        category: AchievementCategory::Exploration,
        icon: "🗝️",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterII,
        name: "Dungeon Master II",
        description: "Complete 50 dungeons",
        category: AchievementCategory::Exploration,
        icon: "🗝️",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterIII,
        name: "Dungeon Master III",
        description: "Complete 100 dungeons",
        category: AchievementCategory::Exploration,
        icon: "🗝️",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterIV,
        name: "Dungeon Master IV",
        description: "Complete 1,000 dungeons",
        category: AchievementCategory::Exploration,
        icon: "⚔️",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterV,
        name: "Dungeon Master V",
        description: "Complete 5,000 dungeons",
        category: AchievementCategory::Exploration,
        icon: "⚔️",
    },
    AchievementDef {
        id: AchievementId::DungeonMasterVI,
        name: "Dungeon Master VI",
        description: "Complete 10,000 dungeons",
        category: AchievementCategory::Exploration,
        icon: "👑",
    },
    // ═══════════════════════════════════════════════════════════════
    // HAVEN ACHIEVEMENTS
    // ═══════════════════════════════════════════════════════════════
    AchievementDef {
        id: AchievementId::HavenDiscovered,
        name: "Haven Found",
        description: "Discover the Haven",
        category: AchievementCategory::Exploration,
        icon: "🏠",
    },
    AchievementDef {
        id: AchievementId::HavenBuilderI,
        name: "Haven Builder I",
        description: "Upgrade all Haven rooms to Tier 1",
        category: AchievementCategory::Exploration,
        icon: "🔨",
    },
    AchievementDef {
        id: AchievementId::HavenBuilderII,
        name: "Haven Builder II",
        description: "Upgrade all Haven rooms to Tier 2",
        category: AchievementCategory::Exploration,
        icon: "🔧",
    },
    AchievementDef {
        id: AchievementId::HavenArchitect,
        name: "Haven Architect",
        description: "Upgrade all Haven rooms to Tier 3",
        category: AchievementCategory::Exploration,
        icon: "🏛️",
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
}
