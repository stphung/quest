//! Debug menu for testing chance-based discoveries.
//!
//! Activated with `--debug` flag. Press backtick to toggle menu.

use crate::achievements::{UiBorderStyle, SELECTABLE_UI_BORDER_STYLES};
use crate::challenges::menu::{create_challenge, ChallengeType};
use crate::core::game_state::GameState;
use crate::deep::DeepState;
use crate::dungeon::generation::generate_dungeon;
use crate::enhancement::EnhancementProgress;
use crate::fishing::generation::generate_fishing_session;
use crate::god_items;
use crate::haven::Haven;
use crate::items;
use crate::loom::{LoomArchetype, LoomState, LoomUiState};
use crate::zones::get_all_zones;
use chrono::{Duration, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DebugAction {
    TriggerDungeon,
    TriggerFishing,
    TriggerChessChallenge,
    TriggerMorrisChallenge,
    TriggerGomokuChallenge,
    TriggerMinesweeperChallenge,
    TriggerRuneChallenge,
    TriggerGoChallenge,
    TriggerFlappyChallenge,
    TriggerJezzballChallenge,
    TriggerSnakeChallenge,
    TriggerRunicShiftChallenge,
    TriggerShardFusionChallenge,
    TriggerHavenDiscovery,
    TriggerSoulforgeDiscovery,
    TriggerForgeStormbreaker,
    TriggerForgeAsprika,
    TriggerForgeSleipnir,
    TriggerForgeMegingjord,
    TriggerGrantStormglass,
    TriggerDiscoverStormglass,
    TriggerGrant100kStormglass,
    TriggerEtchRandomSigils,
    TriggerEtchSPlusSigil,
    TriggerForceOvercharge,
    TriggerDeepDiscovery,
    TriggerDeepGrantMarks,
    TriggerDeepRefreshMissionPool,
    TriggerDeepRefreshRecruits,
    TriggerDeepClearFrontierLayer,
    TriggerDeepCompleteActiveMissions,
    UnlockDeepLayer(u32),
    TravelToZone(u32),
    SetPrestige(u32),
    SetLevel(u32),
    MaxAttributes,
    SetEnhancement(u8),
    // Loom of Worlds debug actions
    TriggerLoomDiscovery,
    LoomSelectArchetype(LoomArchetype),
    LoomUnlockAllNodes,
    LoomGrantResources,
    LoomCompletePattern,
    LoomAdvanceToPattern(usize),
    LoomBuildTestShuttleT1,
    LoomBuildTestShuttleT2,
    LoomClearShuttles,
    TriggerSudokuChallenge,
}

const DEBUG_ACTIONS: &[DebugAction] = &[
    DebugAction::TriggerDungeon,
    DebugAction::TriggerFishing,
    DebugAction::TriggerChessChallenge,
    DebugAction::TriggerMorrisChallenge,
    DebugAction::TriggerGomokuChallenge,
    DebugAction::TriggerMinesweeperChallenge,
    DebugAction::TriggerRuneChallenge,
    DebugAction::TriggerGoChallenge,
    DebugAction::TriggerFlappyChallenge,
    DebugAction::TriggerJezzballChallenge,
    DebugAction::TriggerSnakeChallenge,
    DebugAction::TriggerRunicShiftChallenge,
    DebugAction::TriggerShardFusionChallenge,
    DebugAction::TriggerHavenDiscovery,
    DebugAction::TriggerSoulforgeDiscovery,
    DebugAction::TriggerForgeStormbreaker,
    DebugAction::TriggerForgeAsprika,
    DebugAction::TriggerForgeSleipnir,
    DebugAction::TriggerForgeMegingjord,
    DebugAction::TriggerGrantStormglass,
    DebugAction::TriggerDiscoverStormglass,
    DebugAction::TriggerGrant100kStormglass,
    DebugAction::TriggerEtchRandomSigils,
    DebugAction::TriggerEtchSPlusSigil,
    DebugAction::TriggerForceOvercharge,
    DebugAction::TriggerDeepDiscovery,
    DebugAction::TriggerDeepGrantMarks,
    DebugAction::TriggerDeepRefreshMissionPool,
    DebugAction::TriggerDeepRefreshRecruits,
    DebugAction::TriggerDeepClearFrontierLayer,
    DebugAction::TriggerDeepCompleteActiveMissions,
    DebugAction::UnlockDeepLayer(3),
    DebugAction::UnlockDeepLayer(7),
    DebugAction::UnlockDeepLayer(12),
    DebugAction::UnlockDeepLayer(18),
    DebugAction::UnlockDeepLayer(25),
    DebugAction::UnlockDeepLayer(30),
    // Zone travel actions
    DebugAction::TravelToZone(1),
    DebugAction::TravelToZone(2),
    DebugAction::TravelToZone(3),
    DebugAction::TravelToZone(4),
    DebugAction::TravelToZone(5),
    DebugAction::TravelToZone(6),
    DebugAction::TravelToZone(7),
    DebugAction::TravelToZone(8),
    DebugAction::TravelToZone(9),
    DebugAction::TravelToZone(10),
    DebugAction::TravelToZone(11),
    DebugAction::TravelToZone(12),
    DebugAction::TravelToZone(13),
    DebugAction::TravelToZone(14),
    DebugAction::TravelToZone(15),
    DebugAction::TravelToZone(16),
    DebugAction::TravelToZone(17),
    DebugAction::TravelToZone(18),
    DebugAction::TravelToZone(19),
    DebugAction::TravelToZone(20),
    DebugAction::TravelToZone(21),
    DebugAction::TravelToZone(22),
    DebugAction::TravelToZone(23),
    DebugAction::TravelToZone(24),
    DebugAction::TravelToZone(25),
    DebugAction::TravelToZone(26),
    DebugAction::TravelToZone(27),
    DebugAction::TravelToZone(28),
    DebugAction::TravelToZone(29),
    DebugAction::TravelToZone(30),
    // Character actions (prestige, levels)
    DebugAction::SetPrestige(1),
    DebugAction::SetPrestige(5),
    DebugAction::SetPrestige(25),
    DebugAction::SetPrestige(50),
    DebugAction::SetPrestige(100),
    DebugAction::SetPrestige(250),
    DebugAction::SetPrestige(500),
    DebugAction::SetPrestige(1000),
    DebugAction::SetPrestige(2500),
    DebugAction::SetPrestige(5000),
    DebugAction::SetLevel(1),
    DebugAction::SetLevel(5),
    DebugAction::SetLevel(25),
    DebugAction::SetLevel(50),
    DebugAction::SetLevel(100),
    DebugAction::SetLevel(250),
    DebugAction::SetLevel(500),
    DebugAction::SetLevel(1000),
    DebugAction::SetLevel(2500),
    DebugAction::SetLevel(5000),
    DebugAction::MaxAttributes,
    // Soulforge enhancement actions
    DebugAction::SetEnhancement(0),
    DebugAction::SetEnhancement(1),
    DebugAction::SetEnhancement(2),
    DebugAction::SetEnhancement(3),
    DebugAction::SetEnhancement(4),
    DebugAction::SetEnhancement(5),
    DebugAction::SetEnhancement(6),
    DebugAction::SetEnhancement(7),
    DebugAction::SetEnhancement(8),
    DebugAction::SetEnhancement(9),
    DebugAction::SetEnhancement(10),
    // Loom of Worlds debug actions (indices 98–111)
    DebugAction::TriggerLoomDiscovery,
    DebugAction::LoomSelectArchetype(LoomArchetype::BurnBright),
    DebugAction::LoomSelectArchetype(LoomArchetype::ReachWide),
    DebugAction::LoomSelectArchetype(LoomArchetype::RunDeep),
    DebugAction::LoomUnlockAllNodes,
    DebugAction::LoomGrantResources,
    DebugAction::LoomCompletePattern,
    DebugAction::LoomAdvanceToPattern(0),
    DebugAction::LoomAdvanceToPattern(5),
    DebugAction::LoomAdvanceToPattern(10),
    DebugAction::LoomAdvanceToPattern(17),
    DebugAction::LoomBuildTestShuttleT1,
    DebugAction::LoomBuildTestShuttleT2,
    DebugAction::LoomClearShuttles,
    DebugAction::TriggerSudokuChallenge,
];

const CHALLENGE_ACTIONS: &[DebugAction] = &[
    DebugAction::TriggerChessChallenge,
    DebugAction::TriggerMorrisChallenge,
    DebugAction::TriggerGomokuChallenge,
    DebugAction::TriggerMinesweeperChallenge,
    DebugAction::TriggerRuneChallenge,
    DebugAction::TriggerGoChallenge,
    DebugAction::TriggerFlappyChallenge,
    DebugAction::TriggerJezzballChallenge,
    DebugAction::TriggerSnakeChallenge,
    DebugAction::TriggerRunicShiftChallenge,
    DebugAction::TriggerSudokuChallenge,
    DebugAction::TriggerShardFusionChallenge,
];
const WORLD_ACTIONS: &[DebugAction] = &[
    DebugAction::TriggerDungeon,
    DebugAction::TriggerFishing,
    DebugAction::TriggerHavenDiscovery,
    DebugAction::TriggerSoulforgeDiscovery,
];
const RESOURCE_ACTIONS: &[DebugAction] = &[
    DebugAction::TriggerGrantStormglass,
    DebugAction::TriggerDiscoverStormglass,
    DebugAction::TriggerGrant100kStormglass,
    DebugAction::TriggerEtchRandomSigils,
    DebugAction::TriggerEtchSPlusSigil,
    DebugAction::TriggerForceOvercharge,
];
const ITEM_ACTIONS: &[DebugAction] = &[
    DebugAction::TriggerForgeStormbreaker,
    DebugAction::TriggerForgeAsprika,
    DebugAction::TriggerForgeSleipnir,
    DebugAction::TriggerForgeMegingjord,
];
const DEEP_ACTIONS: &[DebugAction] = &[
    DebugAction::TriggerDeepDiscovery,
    DebugAction::TriggerDeepGrantMarks,
    DebugAction::TriggerDeepRefreshMissionPool,
    DebugAction::TriggerDeepRefreshRecruits,
    DebugAction::TriggerDeepClearFrontierLayer,
    DebugAction::TriggerDeepCompleteActiveMissions,
    DebugAction::UnlockDeepLayer(3),
    DebugAction::UnlockDeepLayer(7),
    DebugAction::UnlockDeepLayer(12),
    DebugAction::UnlockDeepLayer(18),
    DebugAction::UnlockDeepLayer(25),
    DebugAction::UnlockDeepLayer(30),
];
const ZONE_ACTIONS: &[DebugAction] = &[
    DebugAction::TravelToZone(1),
    DebugAction::TravelToZone(2),
    DebugAction::TravelToZone(3),
    DebugAction::TravelToZone(4),
    DebugAction::TravelToZone(5),
    DebugAction::TravelToZone(6),
    DebugAction::TravelToZone(7),
    DebugAction::TravelToZone(8),
    DebugAction::TravelToZone(9),
    DebugAction::TravelToZone(10),
    DebugAction::TravelToZone(11),
    DebugAction::TravelToZone(12),
    DebugAction::TravelToZone(13),
    DebugAction::TravelToZone(14),
    DebugAction::TravelToZone(15),
    DebugAction::TravelToZone(16),
    DebugAction::TravelToZone(17),
    DebugAction::TravelToZone(18),
    DebugAction::TravelToZone(19),
    DebugAction::TravelToZone(20),
    DebugAction::TravelToZone(21),
    DebugAction::TravelToZone(22),
    DebugAction::TravelToZone(23),
    DebugAction::TravelToZone(24),
    DebugAction::TravelToZone(25),
    DebugAction::TravelToZone(26),
    DebugAction::TravelToZone(27),
    DebugAction::TravelToZone(28),
    DebugAction::TravelToZone(29),
    DebugAction::TravelToZone(30),
];
const CHARACTER_ACTIONS: &[DebugAction] = &[
    DebugAction::SetPrestige(1),
    DebugAction::SetPrestige(5),
    DebugAction::SetPrestige(25),
    DebugAction::SetPrestige(50),
    DebugAction::SetPrestige(100),
    DebugAction::SetPrestige(250),
    DebugAction::SetPrestige(500),
    DebugAction::SetPrestige(1000),
    DebugAction::SetPrestige(2500),
    DebugAction::SetPrestige(5000),
    DebugAction::SetLevel(1),
    DebugAction::SetLevel(5),
    DebugAction::SetLevel(25),
    DebugAction::SetLevel(50),
    DebugAction::SetLevel(100),
    DebugAction::SetLevel(250),
    DebugAction::SetLevel(500),
    DebugAction::SetLevel(1000),
    DebugAction::SetLevel(2500),
    DebugAction::SetLevel(5000),
    DebugAction::MaxAttributes,
];
const SOULFORGE_ACTIONS: &[DebugAction] = &[
    DebugAction::SetEnhancement(0),
    DebugAction::SetEnhancement(1),
    DebugAction::SetEnhancement(2),
    DebugAction::SetEnhancement(3),
    DebugAction::SetEnhancement(4),
    DebugAction::SetEnhancement(5),
    DebugAction::SetEnhancement(6),
    DebugAction::SetEnhancement(7),
    DebugAction::SetEnhancement(8),
    DebugAction::SetEnhancement(9),
    DebugAction::SetEnhancement(10),
];
const LOOM_ACTIONS: &[DebugAction] = &[
    DebugAction::TriggerLoomDiscovery,
    DebugAction::LoomSelectArchetype(LoomArchetype::BurnBright),
    DebugAction::LoomSelectArchetype(LoomArchetype::ReachWide),
    DebugAction::LoomSelectArchetype(LoomArchetype::RunDeep),
    DebugAction::LoomUnlockAllNodes,
    DebugAction::LoomGrantResources,
    DebugAction::LoomCompletePattern,
    DebugAction::LoomAdvanceToPattern(0),
    DebugAction::LoomAdvanceToPattern(5),
    DebugAction::LoomAdvanceToPattern(10),
    DebugAction::LoomAdvanceToPattern(17),
    DebugAction::LoomBuildTestShuttleT1,
    DebugAction::LoomBuildTestShuttleT2,
    DebugAction::LoomClearShuttles,
];
const BORDER_OPTION_START_INDEX: usize = DEBUG_ACTIONS.len();

const SET_VALUES: &[u32] = &[1, 5, 25, 50, 100, 250, 500, 1000, 2500, 5000];

const fn set_value_index(value: u32) -> usize {
    let mut i = 0;
    while i < SET_VALUES.len() {
        if SET_VALUES[i] == value {
            return i;
        }
        i += 1;
    }
    0 // fallback
}

impl DebugAction {
    const fn option_index(self) -> usize {
        match self {
            Self::TriggerDungeon => 0,
            Self::TriggerFishing => 1,
            Self::TriggerChessChallenge => 2,
            Self::TriggerMorrisChallenge => 3,
            Self::TriggerGomokuChallenge => 4,
            Self::TriggerMinesweeperChallenge => 5,
            Self::TriggerRuneChallenge => 6,
            Self::TriggerGoChallenge => 7,
            Self::TriggerFlappyChallenge => 8,
            Self::TriggerJezzballChallenge => 9,
            Self::TriggerSnakeChallenge => 10,
            Self::TriggerRunicShiftChallenge => 11,
            Self::TriggerShardFusionChallenge => 12,
            Self::TriggerHavenDiscovery => 13,
            Self::TriggerSoulforgeDiscovery => 14,
            Self::TriggerForgeStormbreaker => 15,
            Self::TriggerForgeAsprika => 16,
            Self::TriggerForgeSleipnir => 17,
            Self::TriggerForgeMegingjord => 18,
            Self::TriggerGrantStormglass => 19,
            Self::TriggerDiscoverStormglass => 20,
            Self::TriggerGrant100kStormglass => 21,
            Self::TriggerEtchRandomSigils => 22,
            Self::TriggerEtchSPlusSigil => 23,
            Self::TriggerForceOvercharge => 24,
            Self::TriggerDeepDiscovery => 25,
            Self::TriggerDeepGrantMarks => 26,
            Self::TriggerDeepRefreshMissionPool => 27,
            Self::TriggerDeepRefreshRecruits => 28,
            Self::TriggerDeepClearFrontierLayer => 29,
            Self::TriggerDeepCompleteActiveMissions => 30,
            Self::UnlockDeepLayer(layer) => match layer {
                3 => 31,
                7 => 32,
                12 => 33,
                18 => 34,
                25 => 35,
                _ => 36, // 30
            },
            Self::TravelToZone(zone_id) => 37 + zone_id as usize - 1, // 37-66
            Self::SetPrestige(amount) => 67 + set_value_index(amount),
            Self::SetLevel(amount) => 77 + set_value_index(amount),
            Self::MaxAttributes => 87,
            Self::SetEnhancement(level) => 88 + level as usize,
            // Loom actions: 99–109
            Self::TriggerLoomDiscovery => 99,
            Self::LoomSelectArchetype(a) => match a {
                LoomArchetype::BurnBright => 100,
                LoomArchetype::ReachWide => 101,
                LoomArchetype::RunDeep => 102,
            },
            Self::LoomUnlockAllNodes => 103,
            Self::LoomGrantResources => 104,
            Self::LoomCompletePattern => 105,
            Self::LoomAdvanceToPattern(n) => match n {
                0 => 106,
                5 => 107,
                10 => 108,
                _ => 109, // 17
            },
            Self::LoomBuildTestShuttleT1 => 110,
            Self::LoomBuildTestShuttleT2 => 111,
            Self::LoomClearShuttles => 112,
            Self::TriggerSudokuChallenge => 113,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::TriggerDungeon => "Trigger Dungeon",
            Self::TriggerFishing => "Trigger Fishing",
            Self::TriggerChessChallenge => "Trigger Chess Challenge",
            Self::TriggerMorrisChallenge => "Trigger Morris Challenge",
            Self::TriggerGomokuChallenge => "Trigger Gomoku Challenge",
            Self::TriggerMinesweeperChallenge => "Trigger Minesweeper Challenge",
            Self::TriggerRuneChallenge => "Trigger Rune Challenge",
            Self::TriggerGoChallenge => "Trigger Go Challenge",
            Self::TriggerFlappyChallenge => "Trigger Flappy Bird Challenge",
            Self::TriggerJezzballChallenge => "Trigger JezzBall Challenge",
            Self::TriggerSnakeChallenge => "Trigger Snake Challenge",
            Self::TriggerRunicShiftChallenge => "Trigger Sigil Surge Challenge",
            Self::TriggerShardFusionChallenge => "Trigger Shard Fusion Challenge",
            Self::TriggerHavenDiscovery => "Trigger Haven Discovery",
            Self::TriggerSoulforgeDiscovery => "Trigger Soulforge Discovery",
            Self::TriggerForgeStormbreaker => "Forge Stormbreaker",
            Self::TriggerForgeAsprika => "Forge Asprika (God Item)",
            Self::TriggerForgeSleipnir => "Forge Sleipnir (God Item)",
            Self::TriggerForgeMegingjord => "Forge Megingjord (God Item)",
            Self::TriggerGrantStormglass => "Grant 1000 Stormglass",
            Self::TriggerDiscoverStormglass => "Discover Stormglass",
            Self::TriggerGrant100kStormglass => "Grant 100k Stormglass",
            Self::TriggerEtchRandomSigils => "Etch Random Sigils (All Slots)",
            Self::TriggerEtchSPlusSigil => "Etch S+ Sigil (Slot 1)",
            Self::TriggerForceOvercharge => "Force Next Surge Overcharged",
            Self::TriggerDeepDiscovery => "Discover The Deep",
            Self::TriggerDeepGrantMarks => "Grant 10,000 Warband Marks",
            Self::TriggerDeepRefreshMissionPool => "Refresh Mission Pool",
            Self::TriggerDeepRefreshRecruits => "Refresh Recruit Pool",
            Self::TriggerDeepClearFrontierLayer => "Clear Current Frontier Layer",
            Self::TriggerDeepCompleteActiveMissions => "Complete Active Missions",
            Self::UnlockDeepLayer(layer) => match layer {
                3 => "Unlock Deep L3 (Red Fault)",
                7 => "Unlock Deep L7 (Mirror Scar)",
                12 => "Unlock Deep L12 (Black Mouth)",
                18 => "Unlock Deep L18 (Hollow Throne)",
                25 => "Unlock Deep L25 (Wailing Reach)",
                _ => "Unlock Deep L30 (Origin Wound)",
            },
            Self::TravelToZone(zone_id) => match zone_id {
                1 => "Travel to Meadow (Zone 1)",
                2 => "Travel to Dark Forest (Zone 2)",
                3 => "Travel to Mountain Pass (Zone 3)",
                4 => "Travel to Ancient Ruins (Zone 4)",
                5 => "Travel to Volcanic Wastes (Zone 5)",
                6 => "Travel to Frozen Tundra (Zone 6)",
                7 => "Travel to Crystal Caverns (Zone 7)",
                8 => "Travel to Sunken Kingdom (Zone 8)",
                9 => "Travel to Floating Isles (Zone 9)",
                10 => "Travel to Storm Citadel (Zone 10)",
                11 => "Travel to The Expanse (Zone 11)",
                12 => "Travel to Splintered Rim (Zone 12)",
                13 => "Travel to Ember Ravine (Zone 13)",
                14 => "Travel to Heart of the Fault (Zone 14)",
                15 => "Travel to Shard Fields (Zone 15)",
                16 => "Travel to Refraction Steps (Zone 16)",
                17 => "Travel to Hall of Second Suns (Zone 17)",
                18 => "Travel to Ashen Verge (Zone 18)",
                19 => "Travel to Throat of the World (Zone 19)",
                20 => "Travel to The Black Mouth (Zone 20)",
                21 => "Travel to Sunken Processional (Zone 21)",
                22 => "Travel to The Pale Archive (Zone 22)",
                23 => "Travel to The Hollow Throne (Zone 23)",
                24 => "Travel to The Stillborn Sea (Zone 24)",
                25 => "Travel to Resonance Fault (Zone 25)",
                26 => "Travel to The Wailing Reach (Zone 26)",
                27 => "Travel to The Scar Root (Zone 27)",
                28 => "Travel to Echoing Abyss (Zone 28)",
                29 => "Travel to Threshold of Silence (Zone 29)",
                30 => "Travel to The Origin Wound (Zone 30)",
                _ => "Travel to Unknown Zone",
            },
            Self::SetPrestige(amount) => match amount {
                1 => "Set Prestige to P1",
                5 => "Set Prestige to P5",
                25 => "Set Prestige to P25",
                50 => "Set Prestige to P50",
                100 => "Set Prestige to P100",
                250 => "Set Prestige to P250",
                500 => "Set Prestige to P500",
                1000 => "Set Prestige to P1000",
                2500 => "Set Prestige to P2500",
                _ => "Set Prestige to P5000",
            },
            Self::SetLevel(amount) => match amount {
                1 => "Set Level to 1",
                5 => "Set Level to 5",
                25 => "Set Level to 25",
                50 => "Set Level to 50",
                100 => "Set Level to 100",
                250 => "Set Level to 250",
                500 => "Set Level to 500",
                1000 => "Set Level to 1000",
                2500 => "Set Level to 2500",
                _ => "Set Level to 5000",
            },
            Self::MaxAttributes => "Max All Attributes",
            Self::SetEnhancement(level) => match level {
                0 => "Set All Enhancement +0",
                1 => "Set All Enhancement +1",
                2 => "Set All Enhancement +2",
                3 => "Set All Enhancement +3",
                4 => "Set All Enhancement +4",
                5 => "Set All Enhancement +5",
                6 => "Set All Enhancement +6",
                7 => "Set All Enhancement +7",
                8 => "Set All Enhancement +8",
                9 => "Set All Enhancement +9",
                _ => "Set All Enhancement +10",
            },
            Self::TriggerLoomDiscovery => "Trigger Loom Discovery",
            Self::LoomSelectArchetype(a) => match a {
                LoomArchetype::BurnBright => "Loom: Select Burn Bright",
                LoomArchetype::ReachWide => "Loom: Select Reach Wide",
                LoomArchetype::RunDeep => "Loom: Select Run Deep",
            },
            Self::LoomUnlockAllNodes => "Loom: Unlock All Nodes",
            Self::LoomGrantResources => "Loom: Grant 100 to All Buffers",
            Self::LoomCompletePattern => "Loom: Complete Current Pattern",
            Self::LoomAdvanceToPattern(0) => "Loom: Reset to Pattern 1",
            Self::LoomAdvanceToPattern(5) => "Loom: Jump to Pattern 6",
            Self::LoomAdvanceToPattern(10) => "Loom: Jump to Pattern 11",
            Self::LoomAdvanceToPattern(_) => "Loom: Jump to Final Pattern",
            Self::LoomBuildTestShuttleT1 => "Build T1 Shuttle (Ember+Void\u{2192}ForgedLight)",
            Self::LoomBuildTestShuttleT2 => "Build T2 Shuttle (FrgLt+Refl\u{2192}EchoGlass)",
            Self::LoomClearShuttles => "Clear All Shuttles",
            Self::TriggerSudokuChallenge => "Trigger Sigil Matrix Challenge",
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn run(
        self,
        state: &mut GameState,
        haven: &mut Haven,
        enhancement: &mut EnhancementProgress,
        deep: &mut DeepState,
        achievements: &mut crate::achievements::Achievements,
        loom: &mut LoomState,
        loom_ui: &mut LoomUiState,
    ) -> &'static str {
        match self {
            Self::TriggerDungeon => trigger_dungeon(state),
            Self::TriggerFishing => trigger_fishing(state),
            Self::TriggerChessChallenge => trigger_chess_challenge(state),
            Self::TriggerMorrisChallenge => trigger_morris_challenge(state),
            Self::TriggerGomokuChallenge => trigger_gomoku_challenge(state),
            Self::TriggerMinesweeperChallenge => trigger_minesweeper_challenge(state),
            Self::TriggerRuneChallenge => trigger_rune_challenge(state),
            Self::TriggerGoChallenge => trigger_go_challenge(state),
            Self::TriggerFlappyChallenge => trigger_flappy_challenge(state),
            Self::TriggerJezzballChallenge => trigger_jezzball_challenge(state),
            Self::TriggerSnakeChallenge => trigger_snake_challenge(state),
            Self::TriggerRunicShiftChallenge => trigger_runic_shift_challenge(state),
            Self::TriggerSudokuChallenge => trigger_sudoku_challenge(state),
            Self::TriggerShardFusionChallenge => trigger_shard_fusion_challenge(state),
            Self::TriggerHavenDiscovery => trigger_haven_discovery(haven),
            Self::TriggerSoulforgeDiscovery => trigger_soulforge_discovery(enhancement),
            Self::TriggerForgeStormbreaker => trigger_forge_stormbreaker(achievements),
            Self::TriggerForgeAsprika => trigger_forge_asprika(state, enhancement),
            Self::TriggerForgeSleipnir => trigger_forge_sleipnir(state, enhancement),
            Self::TriggerForgeMegingjord => trigger_forge_megingjord(state, enhancement),
            Self::TriggerGrantStormglass => trigger_grant_stormglass(state),
            Self::TriggerDiscoverStormglass => trigger_discover_stormglass(state),
            Self::TriggerGrant100kStormglass => trigger_grant_100k_stormglass(state),
            Self::TriggerEtchRandomSigils => trigger_etch_random_sigils(state),
            Self::TriggerEtchSPlusSigil => trigger_etch_s_plus_sigil(state),
            Self::TriggerForceOvercharge => trigger_force_overcharge(state),
            Self::TriggerDeepDiscovery => trigger_deep_discovery(deep, state.prestige_rank),
            Self::TriggerDeepGrantMarks => trigger_deep_grant_marks(deep),
            Self::TriggerDeepRefreshMissionPool => trigger_deep_refresh_mission_pool(deep),
            Self::TriggerDeepRefreshRecruits => trigger_deep_refresh_recruit_pool(deep),
            Self::TriggerDeepClearFrontierLayer => trigger_deep_clear_frontier_layer(deep),
            Self::TriggerDeepCompleteActiveMissions => trigger_deep_complete_active_missions(deep),
            Self::UnlockDeepLayer(layer) => {
                trigger_unlock_deep_layer(deep, state, achievements, layer)
            }
            Self::TravelToZone(zone_id) => {
                trigger_travel_to_zone(state, enhancement, deep, zone_id)
            }
            Self::SetPrestige(amount) => trigger_set_prestige(state, enhancement, amount),
            Self::SetLevel(amount) => trigger_set_level(state, enhancement, amount),
            Self::MaxAttributes => trigger_max_attributes(state, enhancement),
            Self::SetEnhancement(level) => trigger_set_enhancement(state, enhancement, level),
            Self::TriggerLoomDiscovery => {
                crate::loom::complete_discovery(loom);
                "Loom discovered."
            }
            Self::LoomSelectArchetype(archetype) => {
                if loom.persistent.archetype.is_none() {
                    crate::loom::select_archetype(loom, archetype);
                    loom_ui.open = true;
                    "Archetype selected."
                } else {
                    "Archetype already set."
                }
            }
            Self::LoomUnlockAllNodes => {
                for node in &mut loom.persistent.nodes {
                    node.unlocked = true;
                }
                "All nodes unlocked."
            }
            Self::LoomGrantResources => {
                for node in &mut loom.persistent.nodes {
                    node.buffer = (node.buffer + 100.0).min(node.buffer_capacity);
                }
                "Granted 100 to all node buffers."
            }
            Self::LoomCompletePattern => {
                let idx = loom.persistent.active_pattern;
                if let Some(p) = loom.persistent.patterns.get_mut(idx) {
                    for req in &mut p.requirements {
                        req.accumulated = req.amount;
                    }
                    p.completed = true;
                }
                if loom.persistent.active_pattern + 1 < loom.persistent.patterns.len() {
                    loom.persistent.active_pattern += 1;
                }
                let count = loom.persistent.completed_pattern_count();
                state.cached_loom_zone_cap = crate::loom::loom_zone_cap_for_patterns(count);
                match crate::loom::PatternMilestone::from_count(count) {
                    Some(crate::loom::PatternMilestone::ThreadWilds) => {
                        "Pattern milestone: Thread Wilds"
                    }
                    Some(crate::loom::PatternMilestone::WovenFrontier) => {
                        "Pattern milestone: Woven Frontier"
                    }
                    Some(crate::loom::PatternMilestone::TheUnraveling) => {
                        "Pattern milestone: Unraveling"
                    }
                    Some(crate::loom::PatternMilestone::GrandDesign) => {
                        "Pattern milestone: Grand Design"
                    }
                    Some(crate::loom::PatternMilestone::FinalWeave) => {
                        "Pattern milestone: Final Weave"
                    }
                    None => "Current pattern completed.",
                }
            }
            Self::LoomAdvanceToPattern(n) => {
                let target = n.min(loom.persistent.patterns.len().saturating_sub(1));
                for i in 0..target {
                    if let Some(p) = loom.persistent.patterns.get_mut(i) {
                        for req in &mut p.requirements {
                            req.accumulated = req.amount;
                        }
                        p.completed = true;
                    }
                }
                loom.persistent.active_pattern = target;
                "Advanced to pattern."
            }
            Self::LoomBuildTestShuttleT1 => {
                let (a, b, nature) = (
                    crate::loom::types::Resource::Ember,
                    crate::loom::types::Resource::VoidEssence,
                    crate::loom::types::NodeNature::Heat,
                );
                if let Some(recipe) = crate::loom::recipes::find_recipe(a, b, nature) {
                    let r = crate::loom::types::Shuttle::new(
                        recipe.input_a,
                        recipe.input_b,
                        recipe.node_nature,
                        recipe.output,
                        recipe.amount,
                        recipe.tier,
                        vec![crate::loom::types::LoomNodeRef::Extractor(
                            crate::loom::types::NodeId::EmberSpindle,
                        )],
                        vec![crate::loom::types::LoomNodeRef::Extractor(
                            crate::loom::types::NodeId::VoidCondenser,
                        )],
                    );
                    loom.persistent.shuttles.push(r);
                    "T1 Shuttle built (debug)."
                } else {
                    "No T1 recipe found."
                }
            }
            Self::LoomBuildTestShuttleT2 => {
                let (a, b, nature) = (
                    crate::loom::types::Resource::ForgedLight,
                    crate::loom::types::Resource::Reflection,
                    crate::loom::types::NodeNature::Form,
                );
                if let Some(recipe) = crate::loom::recipes::find_recipe(a, b, nature) {
                    let r = crate::loom::types::Shuttle::new(
                        recipe.input_a,
                        recipe.input_b,
                        recipe.node_nature,
                        recipe.output,
                        recipe.amount,
                        recipe.tier,
                        vec![crate::loom::types::LoomNodeRef::Shuttle(0)],
                        vec![crate::loom::types::LoomNodeRef::Extractor(
                            crate::loom::types::NodeId::ReflectionLens,
                        )],
                    );
                    loom.persistent.shuttles.push(r);
                    "T2 Shuttle built (debug)."
                } else {
                    "No T2 recipe found."
                }
            }
            Self::LoomClearShuttles => {
                loom.persistent.shuttles.clear();
                "All shuttles cleared."
            }
        }
    }
}

fn action_for_option_index(option_index: usize) -> Option<DebugAction> {
    DEBUG_ACTIONS.get(option_index).copied()
}

fn is_border_preview_option(option_index: usize) -> bool {
    border_style_for_option_index(option_index).is_some()
}

pub fn border_style_for_option_index(option_index: usize) -> Option<UiBorderStyle> {
    if option_index < BORDER_OPTION_START_INDEX {
        return None;
    }
    let idx = option_index - BORDER_OPTION_START_INDEX;
    SELECTABLE_UI_BORDER_STYLES.get(idx).copied()
}

pub fn option_label_for_index(option_index: usize) -> &'static str {
    if let Some(action) = action_for_option_index(option_index) {
        action.label()
    } else if let Some(style) = border_style_for_option_index(option_index) {
        style.debug_option_label()
    } else {
        "Unknown option"
    }
}

pub fn option_count_for_category(category: DebugCategory) -> usize {
    match category {
        DebugCategory::Challenges => CHALLENGE_ACTIONS.len(),
        DebugCategory::World => WORLD_ACTIONS.len(),
        DebugCategory::Resources => RESOURCE_ACTIONS.len(),
        DebugCategory::Items => ITEM_ACTIONS.len(),
        DebugCategory::Deep => DEEP_ACTIONS.len(),
        DebugCategory::Zones => ZONE_ACTIONS.len(),
        DebugCategory::Character => CHARACTER_ACTIONS.len(),
        DebugCategory::Soulforge => SOULFORGE_ACTIONS.len(),
        DebugCategory::Loom => LOOM_ACTIONS.len(),
        DebugCategory::Borders => SELECTABLE_UI_BORDER_STYLES.len(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugCategory {
    Challenges,
    World,
    Resources,
    Items,
    Deep,
    Zones,
    Character,
    Soulforge,
    Loom,
    Borders,
}

impl DebugCategory {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Challenges => "Challenges",
            Self::World => "World",
            Self::Resources => "Resources",
            Self::Items => "Items",
            Self::Deep => "The Deep",
            Self::Zones => "Zones",
            Self::Character => "Character",
            Self::Soulforge => "Soulforge",
            Self::Loom => "Loom",
            Self::Borders => "Borders",
        }
    }
}

pub const DEBUG_CATEGORIES: &[DebugCategory] = &[
    DebugCategory::Challenges,
    DebugCategory::World,
    DebugCategory::Resources,
    DebugCategory::Items,
    DebugCategory::Deep,
    DebugCategory::Zones,
    DebugCategory::Character,
    DebugCategory::Soulforge,
    DebugCategory::Loom,
    DebugCategory::Borders,
];

/// Debug menu state
#[derive(Debug, Clone, Default)]
pub struct DebugMenu {
    pub is_open: bool,
    pub selected_category: usize,
    pub selected_index: usize,
}

impl DebugMenu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.selected_category = 0;
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn current_category(&self) -> DebugCategory {
        DEBUG_CATEGORIES[self.selected_category]
    }

    pub fn visible_option_indices(&self) -> Vec<usize> {
        let count = option_count_for_category(self.current_category());
        (0..count)
            .map(|i| self.global_option_index_for_visible(i))
            .collect()
    }

    pub fn selected_border_style(&self) -> Option<UiBorderStyle> {
        border_style_for_option_index(self.selected_option_global_index())
    }

    pub fn navigate_prev_category(&mut self) {
        if self.selected_category == 0 {
            self.selected_category = DEBUG_CATEGORIES.len() - 1;
        } else {
            self.selected_category -= 1;
        }
        self.selected_index = 0;
    }

    pub fn navigate_next_category(&mut self) {
        self.selected_category = (self.selected_category + 1) % DEBUG_CATEGORIES.len();
        self.selected_index = 0;
    }

    pub fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn navigate_down(&mut self) {
        if self.selected_index + 1 < option_count_for_category(self.current_category()) {
            self.selected_index += 1;
        }
    }

    fn global_option_index_for_visible(&self, visible_index: usize) -> usize {
        match self.current_category() {
            DebugCategory::Challenges => CHALLENGE_ACTIONS[visible_index].option_index(),
            DebugCategory::World => WORLD_ACTIONS[visible_index].option_index(),
            DebugCategory::Resources => RESOURCE_ACTIONS[visible_index].option_index(),
            DebugCategory::Items => ITEM_ACTIONS[visible_index].option_index(),
            DebugCategory::Deep => DEEP_ACTIONS[visible_index].option_index(),
            DebugCategory::Zones => ZONE_ACTIONS[visible_index].option_index(),
            DebugCategory::Character => CHARACTER_ACTIONS[visible_index].option_index(),
            DebugCategory::Soulforge => SOULFORGE_ACTIONS[visible_index].option_index(),
            DebugCategory::Loom => LOOM_ACTIONS[visible_index].option_index(),
            DebugCategory::Borders => BORDER_OPTION_START_INDEX + visible_index,
        }
    }

    fn selected_option_global_index(&self) -> usize {
        self.global_option_index_for_visible(self.selected_index)
    }

    /// Trigger the selected debug action. Returns a message describing what happened.
    #[allow(clippy::too_many_arguments)]
    pub fn trigger_selected(
        &mut self,
        state: &mut GameState,
        haven: &mut Haven,
        enhancement: &mut EnhancementProgress,
        deep: &mut DeepState,
        achievements: &mut crate::achievements::Achievements,
        loom: &mut LoomState,
        loom_ui: &mut LoomUiState,
    ) -> &'static str {
        let selected_option = self.selected_option_global_index();
        if is_border_preview_option(selected_option) {
            if let Some(style) = border_style_for_option_index(selected_option) {
                achievements.ui_border_style = style;
                return style.border_set_message();
            }
            return "Border preview updated.";
        }

        let msg = action_for_option_index(selected_option)
            .map(|action| action.run(state, haven, enhancement, deep, achievements, loom, loom_ui))
            .unwrap_or("Unknown option");

        // Re-sync zone unlocks after every action — any action may change
        // prestige, achievements, or Deep state that affects zone access.
        let loom_cap =
            crate::loom::loom_zone_cap_for_patterns(loom.persistent.completed_pattern_count());
        state.cached_loom_zone_cap = loom_cap;
        crate::zones::access::sync_account_zone_unlocks(
            &mut state.zone_progression,
            achievements.is_unlocked(crate::achievements::AchievementId::StormsEnd),
            deep.persistent.fracture_zone_cap,
            state.prestige_rank,
            loom_cap,
            state.ascension_level,
        );
        state.cached_fracture_zone_cap = deep.persistent.fracture_zone_cap;

        self.close();
        msg
    }
}

fn trigger_dungeon(state: &mut GameState) -> &'static str {
    if state.active_dungeon.is_some() {
        return "Already in a dungeon!";
    }
    let zone_id = state.zone_progression.current_zone_id;
    state.active_dungeon = Some(generate_dungeon(
        state.character_level,
        state.prestige_rank,
        zone_id,
    ));
    "Dungeon discovered!"
}

fn trigger_fishing(state: &mut GameState) -> &'static str {
    if state.active_fishing.is_some() {
        return "Already fishing!";
    }
    if state.active_dungeon.is_some() {
        return "Cannot fish while in dungeon!";
    }
    let mut rng = rand::rng();
    state.active_fishing = Some(generate_fishing_session(&mut rng));
    "Fishing spot found!"
}

fn trigger_chess_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Chess) {
        return "Chess challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Chess));
    "Chess challenge added!"
}

fn trigger_morris_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Morris) {
        return "Morris challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Morris));
    "Morris challenge added!"
}

fn trigger_gomoku_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Gomoku) {
        return "Gomoku challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Gomoku));
    "Gomoku challenge added!"
}

fn trigger_rune_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Rune) {
        return "Rune challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Rune));
    "Rune challenge added!"
}

fn trigger_minesweeper_challenge(state: &mut GameState) -> &'static str {
    if state
        .challenge_menu
        .has_challenge(&ChallengeType::Minesweeper)
    {
        return "Minesweeper challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Minesweeper));
    "Minesweeper challenge added!"
}

fn trigger_go_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Go) {
        return "Go challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Go));
    "Go challenge added!"
}

fn trigger_flappy_challenge(state: &mut GameState) -> &'static str {
    if state
        .challenge_menu
        .has_challenge(&ChallengeType::FlappyBird)
    {
        return "Flappy Bird challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::FlappyBird));
    "Flappy Bird challenge added!"
}

fn trigger_snake_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Snake) {
        return "Snake challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Snake));
    "Snake challenge added!"
}

fn trigger_jezzball_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Jezzball) {
        return "JezzBall challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Jezzball));
    "JezzBall challenge added!"
}

fn trigger_runic_shift_challenge(state: &mut GameState) -> &'static str {
    if state
        .challenge_menu
        .has_challenge(&ChallengeType::RunicShift)
    {
        return "Sigil Surge challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::RunicShift));
    "Sigil Surge challenge added!"
}

fn trigger_sudoku_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Sudoku) {
        return "Sigil Matrix challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Sudoku));
    "Sigil Matrix challenge added!"
}

fn trigger_shard_fusion_challenge(state: &mut GameState) -> &'static str {
    if state
        .challenge_menu
        .has_challenge(&ChallengeType::ShardFusion)
    {
        return "Shard Fusion challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::ShardFusion));
    "Shard Fusion challenge added!"
}

fn trigger_haven_discovery(haven: &mut Haven) -> &'static str {
    if haven.discovered {
        return "Haven already discovered!";
    }
    haven.discovered = true;
    "Haven discovered!"
}

fn trigger_soulforge_discovery(enhancement: &mut EnhancementProgress) -> &'static str {
    if enhancement.discovered {
        return "Soulforge already discovered!";
    }
    enhancement.discovered = true;
    "Soulforge discovered!"
}

fn trigger_forge_stormbreaker(
    achievements: &mut crate::achievements::Achievements,
) -> &'static str {
    if achievements.is_unlocked(crate::achievements::AchievementId::TheStormbreaker) {
        return "Stormbreaker already forged!";
    }
    achievements.unlock(
        crate::achievements::AchievementId::TheStormbreaker,
        Some("Debug".to_string()),
    );
    // Also unlock StormsEnd so Zone 11 becomes accessible
    if !achievements.is_unlocked(crate::achievements::AchievementId::StormsEnd) {
        achievements.unlock(
            crate::achievements::AchievementId::StormsEnd,
            Some("Debug".to_string()),
        );
    }
    "Forged Stormbreaker!"
}

fn trigger_forge_asprika(state: &mut GameState, enhancement: &EnhancementProgress) -> &'static str {
    if state
        .equipment
        .get(items::EquipmentSlot::Armor)
        .as_ref()
        .is_some_and(|i| i.god_item_id == Some(god_items::GodItemId::Asprika))
    {
        return "Asprika already equipped!";
    }
    let asprika = god_items::asprika_definition().to_item();
    state
        .equipment
        .set(items::EquipmentSlot::Armor, Some(asprika));
    state.recalculate_prestige_bonuses();
    state.recalculate_derived_stats(&enhancement.levels);
    "Asprika forged and equipped!"
}

fn trigger_forge_sleipnir(
    state: &mut GameState,
    enhancement: &EnhancementProgress,
) -> &'static str {
    if state
        .equipment
        .get(items::EquipmentSlot::Boots)
        .as_ref()
        .is_some_and(|i| i.god_item_id == Some(god_items::GodItemId::Sleipnir))
    {
        return "Sleipnir already equipped!";
    }
    let sleipnir = god_items::sleipnir_definition().to_item();
    state
        .equipment
        .set(items::EquipmentSlot::Boots, Some(sleipnir));
    state.recalculate_prestige_bonuses();
    state.recalculate_derived_stats(&enhancement.levels);
    "Sleipnir forged and equipped!"
}

fn trigger_forge_megingjord(
    state: &mut GameState,
    enhancement: &EnhancementProgress,
) -> &'static str {
    if state
        .equipment
        .get(items::EquipmentSlot::Ring)
        .as_ref()
        .is_some_and(|i| i.god_item_id == Some(god_items::GodItemId::Megingjord))
    {
        return "Megingjord already equipped!";
    }
    let megingjord = god_items::megingjord_definition().to_item();
    state
        .equipment
        .set(items::EquipmentSlot::Ring, Some(megingjord));
    state.recalculate_prestige_bonuses();
    state.recalculate_derived_stats(&enhancement.levels);
    "Megingjord forged and equipped!"
}

fn trigger_grant_stormglass(state: &mut GameState) -> &'static str {
    state.stormglass += 1000;
    state.stormglass_discovered = true;
    "Granted 1000 Stormglass!"
}

fn trigger_discover_stormglass(state: &mut GameState) -> &'static str {
    if state.stormglass_discovered {
        return "Stormglass already discovered!";
    }
    state.stormglass_discovered = true;
    "Stormglass discovered!"
}

fn trigger_grant_100k_stormglass(state: &mut GameState) -> &'static str {
    state.stormglass += 100_000;
    state.stormglass_discovered = true;
    "Granted 100,000 Stormglass!"
}

fn trigger_etch_random_sigils(state: &mut GameState) -> &'static str {
    use crate::stormglass::sigils::{generate_sigil_choices, SigilEffectType, MAX_SIGIL_SLOTS};

    // Unlock all slots first
    state.storm_sigils.slots_unlocked = MAX_SIGIL_SLOTS as u8;
    // Fill each slot with a random sigil (pick first of 3 choices)
    // Debug uses ALL types, bypassing daily rotation
    let mut rng = rand::rng();
    for slot in 0..MAX_SIGIL_SLOTS {
        let choices = generate_sigil_choices(&mut rng, &SigilEffectType::ALL);
        state.storm_sigils.sigils[slot] = Some(choices[0].clone());
    }
    "All 5 sigil slots unlocked and etched!"
}

fn trigger_deep_discovery(deep: &mut DeepState, _prestige_rank: u32) -> &'static str {
    if deep.persistent.discovered {
        return "The Deep already discovered!";
    }
    let mut rng = rand::rng();
    crate::deep::complete_discovery(deep, &mut rng);
    "The Deep discovered!"
}

fn trigger_deep_grant_marks(deep: &mut DeepState) -> &'static str {
    if !deep.persistent.discovered {
        return "Discover The Deep first!";
    }
    deep.prestige.warband_marks = deep.prestige.warband_marks.saturating_add(10_000);
    "Granted 10,000 Warband Marks!"
}

fn trigger_deep_refresh_mission_pool(deep: &mut DeepState) -> &'static str {
    if !deep.persistent.discovered {
        return "Discover The Deep first!";
    }
    let mut rng = rand::rng();
    deep.prestige.available_missions =
        crate::deep::generate_mission_pool(&deep.persistent, &mut rng);
    deep.prestige.pool_refreshed_at = Some(Utc::now());
    "Deep mission pool refreshed!"
}

fn trigger_deep_refresh_recruit_pool(deep: &mut DeepState) -> &'static str {
    if !deep.persistent.discovered {
        return "Discover The Deep first!";
    }
    let mut rng = rand::rng();
    let guild_rank = deep.persistent.guild_rank;
    deep.prestige.recruit_pool =
        crate::deep::generate_recruit_pool(guild_rank, || deep.persistent.next_merc_id(), &mut rng);
    "Deep recruit pool refreshed!"
}

fn trigger_deep_clear_frontier_layer(deep: &mut DeepState) -> &'static str {
    if !deep.persistent.discovered {
        return "Discover The Deep first!";
    }
    let frontier = deep.persistent.frontier_layer();
    crate::deep::mark_layer_cleared(&mut deep.persistent, frontier);
    let mut rng = rand::rng();
    deep.prestige.available_missions =
        crate::deep::generate_mission_pool(&deep.persistent, &mut rng);
    deep.prestige.pool_refreshed_at = Some(Utc::now());
    "Cleared current Deep frontier layer!"
}

fn trigger_deep_complete_active_missions(deep: &mut DeepState) -> &'static str {
    if !deep.persistent.discovered {
        return "Discover The Deep first!";
    }
    if deep.prestige.active_missions.is_empty() {
        return "No active Deep missions.";
    }
    let mut rng = rand::rng();
    let now = Utc::now() + Duration::days(7);
    let summary =
        crate::deep::tick_all_missions(&mut deep.prestige, &mut deep.persistent, now, &mut rng);
    if summary.missions_completed > 0 {
        "Completed active Deep missions!"
    } else {
        "No Deep missions completed."
    }
}

fn trigger_unlock_deep_layer(
    deep: &mut DeepState,
    state: &mut GameState,
    achievements: &mut crate::achievements::Achievements,
    target_layer: u32,
) -> &'static str {
    // Auto-discover The Deep if needed
    if !deep.persistent.discovered {
        let mut rng = rand::rng();
        crate::deep::complete_discovery(deep, &mut rng);
    }

    // Clear layers up to and including the target
    while deep.persistent.frontier_layer() <= target_layer {
        let frontier = deep.persistent.frontier_layer();
        crate::deep::mark_layer_cleared(&mut deep.persistent, frontier);
    }

    // Unlock layer achievements (triggers Power Core activation)
    achievements.on_deep_breakthrough(target_layer, Some(&state.character_name));

    // Set fracture zone cap based on the region this layer unlocks
    if let Some(region) = crate::zones::FractureRegion::from_layer(target_layer) {
        let cap = region.end_zone_id();
        if deep.persistent.fracture_zone_cap < cap {
            deep.persistent.fracture_zone_cap = cap;
        }
        // Unlock the zones (loom_zone_cap=30: no loom context here;
        // trigger_selected re-syncs with full loom cap after action runs)
        crate::zones::access::sync_account_zone_unlocks(
            &mut state.zone_progression,
            true,
            deep.persistent.fracture_zone_cap,
            state.prestige_rank,
            30,
            state.ascension_level,
        );
    }

    // Refresh mission pool for new layers
    let mut rng = rand::rng();
    deep.prestige.available_missions =
        crate::deep::generate_mission_pool(&deep.persistent, &mut rng);
    deep.prestige.pool_refreshed_at = Some(Utc::now());

    match target_layer {
        3 => "Cleared to Deep L3 \u{2014} Red Fault unlocked (Z12-14)!",
        7 => "Cleared to Deep L7 \u{2014} Mirror Scar unlocked (Z15-17)!",
        12 => "Cleared to Deep L12 \u{2014} Black Mouth unlocked (Z18-20)!",
        18 => "Cleared to Deep L18 \u{2014} Hollow Throne unlocked (Z21-23)!",
        25 => "Cleared to Deep L25 \u{2014} Wailing Reach unlocked (Z24-26)!",
        30 => "Cleared to Deep L30 \u{2014} Origin Wound unlocked (Z27-30)!",
        _ => "Deep layers cleared!",
    }
}

fn trigger_etch_s_plus_sigil(state: &mut GameState) -> &'static str {
    use crate::stormglass::sigils::{Sigil, SigilEffectType, SigilGrade};

    // Ensure at least 1 slot is unlocked
    if state.storm_sigils.slots_unlocked == 0 {
        state.storm_sigils.slots_unlocked = 1;
    }
    // Etch S+ Sigil of Fury (max damage%) in slot 0
    let effect = SigilEffectType::DamagePercent;
    let (_, max) = effect.range();
    state.storm_sigils.sigils[0] = Some(Sigil {
        effect,
        value: max,
        grade: SigilGrade::SPlus,
    });
    "S+ Sigil of Fury etched in slot 1!"
}

fn trigger_force_overcharge(state: &mut GameState) -> &'static str {
    state.debug_force_overcharge = true;
    "Next Chrono Surge will be Overcharged!"
}

fn trigger_travel_to_zone(
    state: &mut GameState,
    enhancement: &EnhancementProgress,
    deep: &mut DeepState,
    zone_id: u32,
) -> &'static str {
    let zones = get_all_zones();
    let zone = match zones.iter().find(|z| z.id == zone_id) {
        Some(z) => z,
        None => return "Invalid zone ID!",
    };

    // Clear active content
    state.active_dungeon = None;
    state.active_fishing = None;
    state.combat_state.current_enemy = None;

    // Auto-bump prestige if needed
    if state.prestige_rank < zone.prestige_requirement {
        state.prestige_rank = zone.prestige_requirement;
        state.recalculate_prestige_bonuses();
    }

    // Zone 11 is achievement-gated (not prestige-gated), but needs high prestige to survive
    if zone_id == 11 && state.prestige_rank < 20 {
        state.prestige_rank = 20;
        state.recalculate_prestige_bonuses();
    }

    // Fracture zones 12+: auto-set Deep fracture_zone_cap and discover Deep
    if zone_id >= 12 {
        if !deep.persistent.discovered {
            let mut rng = rand::rng();
            crate::deep::complete_discovery(deep, &mut rng);
        }
        if deep.persistent.fracture_zone_cap < zone_id {
            deep.persistent.fracture_zone_cap = zone_id;
        }
        // Ensure high enough prestige to survive fracture zones
        if state.prestige_rank < 20 {
            state.prestige_rank = 20;
            state.recalculate_prestige_bonuses();
        }
    }

    // Unlock the target zone (and all zones at or below its prestige tier)
    for z in zones {
        if z.prestige_requirement <= state.prestige_rank {
            state.zone_progression.unlock_zone(z.id);
        }
    }

    // Unlock fracture zones up to the cap (loom_zone_cap=30: no loom context;
    // trigger_selected re-syncs with full loom cap after action runs)
    crate::zones::access::sync_account_zone_unlocks(
        &mut state.zone_progression,
        true,
        deep.persistent.fracture_zone_cap,
        state.prestige_rank,
        30,
        state.ascension_level,
    );

    // Travel to subzone 1
    state.zone_progression.current_zone_id = zone_id;
    state.zone_progression.current_subzone_id = 1;
    state.zone_progression.kills_in_subzone = 0;
    state.zone_progression.fighting_boss = false;

    // Recalculate stats
    state.recalculate_derived_stats(&enhancement.levels);

    match zone_id {
        1 => "Traveled to Meadow (Zone 1)",
        2 => "Traveled to Dark Forest (Zone 2)",
        3 => "Traveled to Mountain Pass (Zone 3, P5)",
        4 => "Traveled to Ancient Ruins (Zone 4, P5)",
        5 => "Traveled to Volcanic Wastes (Zone 5, P10)",
        6 => "Traveled to Frozen Tundra (Zone 6, P10)",
        7 => "Traveled to Crystal Caverns (Zone 7, P15)",
        8 => "Traveled to Sunken Kingdom (Zone 8, P15)",
        9 => "Traveled to Floating Isles (Zone 9, P20)",
        10 => "Traveled to Storm Citadel (Zone 10, P20)",
        11 => "Traveled to The Expanse (Zone 11)",
        12 => "Traveled to Splintered Rim (Zone 12)",
        13 => "Traveled to Ember Ravine (Zone 13)",
        14 => "Traveled to Heart of the Fault (Zone 14)",
        15 => "Traveled to Shard Fields (Zone 15)",
        16 => "Traveled to Refraction Steps (Zone 16)",
        17 => "Traveled to Hall of Second Suns (Zone 17)",
        18 => "Traveled to Ashen Verge (Zone 18)",
        19 => "Traveled to Throat of the World (Zone 19)",
        20 => "Traveled to The Black Mouth (Zone 20)",
        21 => "Traveled to Sunken Processional (Zone 21)",
        22 => "Traveled to The Pale Archive (Zone 22)",
        23 => "Traveled to The Hollow Throne (Zone 23)",
        24 => "Traveled to The Stillborn Sea (Zone 24)",
        25 => "Traveled to Resonance Fault (Zone 25)",
        26 => "Traveled to The Wailing Reach (Zone 26)",
        27 => "Traveled to The Scar Root (Zone 27)",
        28 => "Traveled to Echoing Abyss (Zone 28)",
        29 => "Traveled to Threshold of Silence (Zone 29)",
        30 => "Traveled to The Origin Wound (Zone 30)",
        _ => "Traveled to unknown zone",
    }
}

fn trigger_set_prestige(
    state: &mut GameState,
    enhancement: &EnhancementProgress,
    rank: u32,
) -> &'static str {
    state.prestige_rank = rank;
    state.recalculate_prestige_bonuses();

    // Unlock zones accessible at new prestige rank (skip fracture zones 12+,
    // those are gated by Deep layer breakthroughs via sync_account_zone_unlocks)
    let zones = get_all_zones();
    for z in zones {
        if z.id <= 11 && z.prestige_requirement <= state.prestige_rank {
            state.zone_progression.unlock_zone(z.id);
        }
    }

    // Recalculate stats
    state.recalculate_derived_stats(&enhancement.levels);

    match rank {
        1 => "Set Prestige to P1!",
        5 => "Set Prestige to P5!",
        25 => "Set Prestige to P25!",
        50 => "Set Prestige to P50!",
        100 => "Set Prestige to P100!",
        250 => "Set Prestige to P250!",
        500 => "Set Prestige to P500!",
        1000 => "Set Prestige to P1000!",
        2500 => "Set Prestige to P2500!",
        _ => "Set Prestige to P5000!",
    }
}

fn trigger_set_level(
    state: &mut GameState,
    enhancement: &EnhancementProgress,
    target_level: u32,
) -> &'static str {
    let mut rng = rand::rng();
    // Grant levels up to target, distributing attribute points for each
    while state.character_level < target_level {
        state.character_level += 1;
        crate::core::xp::distribute_level_up_points(&mut rng, state);
    }
    state.character_xp = 0;
    state.recalculate_derived_stats(&enhancement.levels);

    match target_level {
        1 => "Set Level to 1!",
        5 => "Set Level to 5!",
        25 => "Set Level to 25!",
        50 => "Set Level to 50!",
        100 => "Set Level to 100!",
        250 => "Set Level to 250!",
        500 => "Set Level to 500!",
        1000 => "Set Level to 1000!",
        2500 => "Set Level to 2500!",
        _ => "Set Level to 5000!",
    }
}

fn trigger_set_enhancement(
    state: &mut GameState,
    enhancement: &mut EnhancementProgress,
    level: u8,
) -> &'static str {
    let clamped = level.min(10);
    for i in 0..7 {
        enhancement.levels[i] = clamped;
    }
    enhancement.discovered = true;
    state.recalculate_derived_stats(&enhancement.levels);
    match clamped {
        0 => "Set all enhancement to +0!",
        1 => "Set all enhancement to +1!",
        2 => "Set all enhancement to +2!",
        3 => "Set all enhancement to +3!",
        4 => "Set all enhancement to +4!",
        5 => "Set all enhancement to +5!",
        6 => "Set all enhancement to +6!",
        7 => "Set all enhancement to +7!",
        8 => "Set all enhancement to +8!",
        9 => "Set all enhancement to +9!",
        _ => "Set all enhancement to +10!",
    }
}

fn trigger_max_attributes(
    state: &mut GameState,
    enhancement: &EnhancementProgress,
) -> &'static str {
    let cap = state.get_attribute_cap();
    for attr in crate::character::attributes::AttributeType::all() {
        state.attributes.set(attr, cap);
    }
    state.recalculate_derived_stats(&enhancement.levels);
    "All attributes set to cap!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_navigation_within_category() {
        let mut menu = DebugMenu::new();
        menu.open();
        assert_eq!(menu.current_category(), DebugCategory::Challenges);
        assert_eq!(menu.selected_index, 0);
        assert_eq!(menu.selected_option_global_index(), 2);

        menu.navigate_down();
        assert_eq!(menu.selected_index, 1);
        assert_eq!(menu.selected_option_global_index(), 3);

        for _ in 0..32 {
            menu.navigate_down();
        }
        assert_eq!(menu.selected_index, CHALLENGE_ACTIONS.len() - 1);
        assert_eq!(menu.selected_option_global_index(), 12);

        // Can't go past end
        menu.navigate_down();
        assert_eq!(menu.selected_index, CHALLENGE_ACTIONS.len() - 1);

        menu.navigate_up();
        assert_eq!(menu.selected_index, CHALLENGE_ACTIONS.len() - 2);

        // Can't go before start
        for _ in 0..32 {
            menu.navigate_up();
        }
        assert_eq!(menu.selected_index, 0);
    }

    #[test]
    fn test_category_navigation_resets_selection() {
        let mut menu = DebugMenu::new();
        menu.open();
        menu.navigate_down();
        assert_eq!(menu.selected_index, 1);

        menu.navigate_next_category();
        assert_eq!(menu.current_category(), DebugCategory::World);
        assert_eq!(menu.selected_index, 0);
        assert_eq!(menu.selected_option_global_index(), 0);

        menu.navigate_prev_category();
        assert_eq!(menu.current_category(), DebugCategory::Challenges);
        assert_eq!(menu.selected_index, 0);
        assert_eq!(menu.selected_option_global_index(), 2);

        menu.navigate_prev_category();
        assert_eq!(menu.current_category(), DebugCategory::Borders);

        menu.navigate_prev_category();
        assert_eq!(menu.current_category(), DebugCategory::Loom);

        menu.navigate_prev_category();
        assert_eq!(menu.current_category(), DebugCategory::Soulforge);

        menu.navigate_prev_category();
        assert_eq!(menu.current_category(), DebugCategory::Character);

        menu.navigate_prev_category();
        assert_eq!(menu.current_category(), DebugCategory::Zones);

        menu.navigate_prev_category();
        assert_eq!(menu.current_category(), DebugCategory::Deep);
    }

    #[test]
    fn test_border_preview_does_not_close_menu() {
        let mut menu = DebugMenu::new();
        menu.open();
        for _ in 0..9 {
            menu.navigate_next_category();
        }
        assert_eq!(menu.current_category(), DebugCategory::Borders);

        let mut state = GameState::new("Test".to_string(), 0);
        let mut haven = Haven::new();
        let mut enhancement = EnhancementProgress::new();
        let mut deep = DeepState::new();
        let mut achievements = crate::achievements::Achievements::default();
        let mut loom = LoomState::new();
        let mut loom_ui = LoomUiState::new();
        let msg = menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        assert_eq!(msg, "Border style set: Classic");
        assert!(menu.is_open);
        assert_eq!(achievements.ui_border_style, UiBorderStyle::Classic);
    }

    #[test]
    fn test_border_style_mapping_complete() {
        assert_eq!(
            option_count_for_category(DebugCategory::Borders),
            SELECTABLE_UI_BORDER_STYLES.len()
        );
        for idx in BORDER_OPTION_START_INDEX
            ..(BORDER_OPTION_START_INDEX + SELECTABLE_UI_BORDER_STYLES.len())
        {
            assert!(border_style_for_option_index(idx).is_some());
        }
        assert_eq!(
            border_style_for_option_index(
                BORDER_OPTION_START_INDEX + SELECTABLE_UI_BORDER_STYLES.len() - 1
            ),
            Some(UiBorderStyle::HeaderRail),
        );
    }

    #[test]
    fn test_toggle() {
        let mut menu = DebugMenu::new();
        assert!(!menu.is_open);

        menu.toggle();
        assert!(menu.is_open);

        menu.toggle();
        assert!(!menu.is_open);
    }

    #[test]
    fn test_trigger_dungeon() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_dungeon(&mut state);
        assert_eq!(msg, "Dungeon discovered!");
        assert!(state.active_dungeon.is_some());

        // Can't trigger again
        let msg = trigger_dungeon(&mut state);
        assert_eq!(msg, "Already in a dungeon!");
    }

    #[test]
    fn test_trigger_fishing() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_fishing(&mut state);
        assert_eq!(msg, "Fishing spot found!");
        assert!(state.active_fishing.is_some());

        // Can't trigger again
        let msg = trigger_fishing(&mut state);
        assert_eq!(msg, "Already fishing!");
    }

    #[test]
    fn test_trigger_chess_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_chess_challenge(&mut state);
        assert_eq!(msg, "Chess challenge added!");
        assert!(state.challenge_menu.has_challenge(&ChallengeType::Chess));

        // Can't add duplicate
        let msg = trigger_chess_challenge(&mut state);
        assert_eq!(msg, "Chess challenge already pending!");
    }

    #[test]
    fn test_trigger_morris_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_morris_challenge(&mut state);
        assert_eq!(msg, "Morris challenge added!");
        assert!(state.challenge_menu.has_challenge(&ChallengeType::Morris));

        // Can't add duplicate
        let msg = trigger_morris_challenge(&mut state);
        assert_eq!(msg, "Morris challenge already pending!");
    }

    #[test]
    fn test_trigger_gomoku_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_gomoku_challenge(&mut state);
        assert_eq!(msg, "Gomoku challenge added!");
        assert!(state.challenge_menu.has_challenge(&ChallengeType::Gomoku));

        // Can't add duplicate
        let msg = trigger_gomoku_challenge(&mut state);
        assert_eq!(msg, "Gomoku challenge already pending!");
    }

    #[test]
    fn test_trigger_rune_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_rune_challenge(&mut state);
        assert_eq!(msg, "Rune challenge added!");
        assert!(state.challenge_menu.has_challenge(&ChallengeType::Rune));

        let msg = trigger_rune_challenge(&mut state);
        assert_eq!(msg, "Rune challenge already pending!");
    }

    #[test]
    fn test_trigger_minesweeper_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_minesweeper_challenge(&mut state);
        assert_eq!(msg, "Minesweeper challenge added!");
        assert!(state
            .challenge_menu
            .has_challenge(&ChallengeType::Minesweeper));

        // Can't add duplicate
        let msg = trigger_minesweeper_challenge(&mut state);
        assert_eq!(msg, "Minesweeper challenge already pending!");
    }

    #[test]
    fn test_trigger_go_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_go_challenge(&mut state);
        assert_eq!(msg, "Go challenge added!");
        assert!(state.challenge_menu.has_challenge(&ChallengeType::Go));

        // Can't add duplicate
        let msg = trigger_go_challenge(&mut state);
        assert_eq!(msg, "Go challenge already pending!");
    }

    #[test]
    fn test_trigger_flappy_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_flappy_challenge(&mut state);
        assert_eq!(msg, "Flappy Bird challenge added!");
        assert!(state
            .challenge_menu
            .has_challenge(&ChallengeType::FlappyBird));

        // Can't add duplicate
        let msg = trigger_flappy_challenge(&mut state);
        assert_eq!(msg, "Flappy Bird challenge already pending!");
    }

    #[test]
    fn test_trigger_jezzball_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_jezzball_challenge(&mut state);
        assert_eq!(msg, "JezzBall challenge added!");
        assert!(state.challenge_menu.has_challenge(&ChallengeType::Jezzball));

        // Can't add duplicate
        let msg = trigger_jezzball_challenge(&mut state);
        assert_eq!(msg, "JezzBall challenge already pending!");
    }

    #[test]
    fn test_trigger_snake_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_snake_challenge(&mut state);
        assert_eq!(msg, "Snake challenge added!");
        assert!(state.challenge_menu.has_challenge(&ChallengeType::Snake));

        // Can't add duplicate
        let msg = trigger_snake_challenge(&mut state);
        assert_eq!(msg, "Snake challenge already pending!");
    }

    #[test]
    fn test_trigger_runic_shift_challenge() {
        let mut state = GameState::new("Test".to_string(), 0);
        let msg = trigger_runic_shift_challenge(&mut state);
        assert_eq!(msg, "Sigil Surge challenge added!");
        assert!(state
            .challenge_menu
            .has_challenge(&ChallengeType::RunicShift));

        // Can't add duplicate
        let msg = trigger_runic_shift_challenge(&mut state);
        assert_eq!(msg, "Sigil Surge challenge already pending!");
    }

    #[test]
    fn test_trigger_haven_discovery() {
        let mut haven = Haven::new();
        assert!(!haven.discovered);

        let msg = trigger_haven_discovery(&mut haven);
        assert_eq!(msg, "Haven discovered!");
        assert!(haven.discovered);

        // Can't discover again
        let msg = trigger_haven_discovery(&mut haven);
        assert_eq!(msg, "Haven already discovered!");
    }

    #[test]
    fn test_trigger_soulforge_discovery() {
        let mut enhancement = EnhancementProgress::new();
        assert!(!enhancement.discovered);

        let msg = trigger_soulforge_discovery(&mut enhancement);
        assert_eq!(msg, "Soulforge discovered!");
        assert!(enhancement.discovered);

        // Can't discover again
        let msg = trigger_soulforge_discovery(&mut enhancement);
        assert_eq!(msg, "Soulforge already discovered!");
    }

    #[test]
    fn test_trigger_forge_asprika() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();

        let msg = trigger_forge_asprika(&mut state, &enhancement);
        assert_eq!(msg, "Asprika forged and equipped!");
        assert!(state.equipment.get(items::EquipmentSlot::Armor).is_some());

        // Can't forge again
        let msg = trigger_forge_asprika(&mut state, &enhancement);
        assert_eq!(msg, "Asprika already equipped!");
    }

    #[test]
    fn test_trigger_forge_sleipnir() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();

        let msg = trigger_forge_sleipnir(&mut state, &enhancement);
        assert_eq!(msg, "Sleipnir forged and equipped!");
        assert!(state.equipment.get(items::EquipmentSlot::Boots).is_some());

        // Can't forge again
        let msg = trigger_forge_sleipnir(&mut state, &enhancement);
        assert_eq!(msg, "Sleipnir already equipped!");
    }

    #[test]
    fn test_trigger_forge_megingjord() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();

        let msg = trigger_forge_megingjord(&mut state, &enhancement);
        assert_eq!(msg, "Megingjord forged and equipped!");
        assert!(state.equipment.get(items::EquipmentSlot::Ring).is_some());

        // Can't forge again
        let msg = trigger_forge_megingjord(&mut state, &enhancement);
        assert_eq!(msg, "Megingjord already equipped!");
    }

    #[test]
    fn test_trigger_deep_refresh_mission_pool_requires_discovery() {
        let mut deep = DeepState::new();
        let msg = trigger_deep_refresh_mission_pool(&mut deep);
        assert_eq!(msg, "Discover The Deep first!");
    }

    #[test]
    fn test_trigger_deep_grant_marks() {
        let mut deep = DeepState::new();
        let mut rng = rand::rng();
        crate::deep::complete_discovery(&mut deep, &mut rng);
        let before = deep.prestige.warband_marks;
        let msg = trigger_deep_grant_marks(&mut deep);
        assert_eq!(msg, "Granted 10,000 Warband Marks!");
        assert_eq!(deep.prestige.warband_marks, before + 10_000);
    }

    #[test]
    fn test_trigger_deep_clear_frontier_layer_marks_layer_cleared() {
        let mut deep = DeepState::new();
        let mut rng = rand::rng();
        crate::deep::complete_discovery(&mut deep, &mut rng);
        let frontier = deep.persistent.frontier_layer();
        let msg = trigger_deep_clear_frontier_layer(&mut deep);
        assert_eq!(msg, "Cleared current Deep frontier layer!");
        assert!(deep
            .persistent
            .layer_record(frontier)
            .map(|r| r.cleared)
            .unwrap_or(false));
    }

    #[test]
    fn test_trigger_travel_to_zone_basic() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        let mut deep = DeepState::new();
        assert_eq!(state.zone_progression.current_zone_id, 1);

        let msg = trigger_travel_to_zone(&mut state, &enhancement, &mut deep, 5);
        assert_eq!(msg, "Traveled to Volcanic Wastes (Zone 5, P10)");
        assert_eq!(state.zone_progression.current_zone_id, 5);
        assert_eq!(state.zone_progression.current_subzone_id, 1);
        // Prestige auto-bumped to P10
        assert_eq!(state.prestige_rank, 10);
    }

    #[test]
    fn test_trigger_travel_clears_active_content() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        let mut deep = DeepState::new();
        state.active_dungeon = Some(generate_dungeon(1, 0, 1));

        trigger_travel_to_zone(&mut state, &enhancement, &mut deep, 1);
        assert!(state.active_dungeon.is_none());
    }

    #[test]
    fn test_trigger_travel_no_prestige_downgrade() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        let mut deep = DeepState::new();
        state.prestige_rank = 20;

        trigger_travel_to_zone(&mut state, &enhancement, &mut deep, 1);
        // Traveling to P0 zone should not lower prestige
        assert_eq!(state.prestige_rank, 20);
    }

    #[test]
    fn test_trigger_travel_unlocks_intermediate_zones() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        let mut deep = DeepState::new();

        trigger_travel_to_zone(&mut state, &enhancement, &mut deep, 7);
        // Should unlock zones 1-8 (all P0, P5, P10, P15 zones)
        for zone_id in 1..=8 {
            assert!(
                state.zone_progression.is_zone_unlocked(zone_id),
                "Zone {zone_id} should be unlocked after traveling to Zone 7 (P15)"
            );
        }
    }

    #[test]
    fn test_trigger_travel_to_fracture_zone() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        let mut deep = DeepState::new();

        let msg = trigger_travel_to_zone(&mut state, &enhancement, &mut deep, 15);
        assert_eq!(msg, "Traveled to Shard Fields (Zone 15)");
        assert_eq!(state.zone_progression.current_zone_id, 15);
        // Deep auto-discovered and fracture_zone_cap set
        assert!(deep.persistent.discovered);
        assert!(deep.persistent.fracture_zone_cap >= 15);
        // Zone 15 should be unlocked
        assert!(state.zone_progression.is_zone_unlocked(15));
    }

    #[test]
    fn test_trigger_unlock_deep_layer_3() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 50; // Meet P50 requirement for zones 12-14
        let mut deep = DeepState::new();
        let mut achievements = crate::achievements::Achievements::default();

        let msg = trigger_unlock_deep_layer(&mut deep, &mut state, &mut achievements, 3);
        assert_eq!(
            msg,
            "Cleared to Deep L3 \u{2014} Red Fault unlocked (Z12-14)!"
        );
        assert!(deep.persistent.discovered);
        assert!(deep.persistent.fracture_zone_cap >= 14);
        // Zones 12-14 should be unlocked
        for zone_id in 12..=14 {
            assert!(
                state.zone_progression.is_zone_unlocked(zone_id),
                "Zone {zone_id} should be unlocked after Deep L3"
            );
        }
        // Power Core achievement should be unlocked
        assert!(achievements.is_unlocked(crate::achievements::AchievementId::PowerCoreI));
    }

    #[test]
    fn test_trigger_unlock_deep_layer_12() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 100; // Meet P100 requirement for zones 18-20
        let mut deep = DeepState::new();
        let mut achievements = crate::achievements::Achievements::default();

        trigger_unlock_deep_layer(&mut deep, &mut state, &mut achievements, 12);
        assert!(deep.persistent.fracture_zone_cap >= 20);
        for zone_id in 12..=20 {
            assert!(
                state.zone_progression.is_zone_unlocked(zone_id),
                "Zone {zone_id} should be unlocked after Deep L12"
            );
        }
        // Layer achievements up to layer 10 should be unlocked
        assert!(achievements.is_unlocked(crate::achievements::AchievementId::Layer5Cleared));
        assert!(achievements.is_unlocked(crate::achievements::AchievementId::Layer10Cleared));
        // Power Core achievements up to layer 12 should be unlocked
        assert!(achievements.is_unlocked(crate::achievements::AchievementId::PowerCoreI));
        assert!(achievements.is_unlocked(crate::achievements::AchievementId::PowerCoreII));
        assert!(achievements.is_unlocked(crate::achievements::AchievementId::PowerCoreIII));
    }

    #[test]
    fn test_trigger_set_prestige() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();

        let msg = trigger_set_prestige(&mut state, &enhancement, 5);
        assert_eq!(msg, "Set Prestige to P5!");
        assert_eq!(state.prestige_rank, 5);

        // Zones 3-4 (P5) should now be unlocked
        assert!(state.zone_progression.is_zone_unlocked(3));
        assert!(state.zone_progression.is_zone_unlocked(4));
    }

    #[test]
    fn test_trigger_set_prestige_overwrites() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();

        trigger_set_prestige(&mut state, &enhancement, 10);
        trigger_set_prestige(&mut state, &enhancement, 5);
        assert_eq!(state.prestige_rank, 5);
    }

    #[test]
    fn test_trigger_set_level() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        assert_eq!(state.character_level, 1);

        let msg = trigger_set_level(&mut state, &enhancement, 25);
        assert_eq!(msg, "Set Level to 25!");
        assert_eq!(state.character_level, 25);
    }

    #[test]
    fn test_trigger_set_level_distributes_attributes() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        // Bump prestige so attribute cap doesn't interfere
        state.prestige_rank = 10; // cap = 20 + 50 = 70
        let initial_sum: u32 = crate::character::attributes::AttributeType::all()
            .iter()
            .map(|a| state.attributes.get(*a))
            .sum();

        trigger_set_level(&mut state, &enhancement, 25);

        let final_sum: u32 = crate::character::attributes::AttributeType::all()
            .iter()
            .map(|a| state.attributes.get(*a))
            .sum();
        // 24 levels (1->25) * 3 points = 72 attribute points gained
        assert_eq!(final_sum, initial_sum + 72);
    }

    #[test]
    fn test_trigger_set_level_noop_if_already_at_target() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        state.character_level = 50;

        trigger_set_level(&mut state, &enhancement, 25);
        // Should not lower level — noop when already above target
        assert_eq!(state.character_level, 50);
    }

    #[test]
    fn test_character_category_has_21_options() {
        assert_eq!(option_count_for_category(DebugCategory::Character), 21);
    }

    #[test]
    fn test_zones_category_has_30_options() {
        assert_eq!(option_count_for_category(DebugCategory::Zones), 30);
    }

    #[test]
    fn test_trigger_set_prestige_100() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();

        let msg = trigger_set_prestige(&mut state, &enhancement, 100);
        assert_eq!(msg, "Set Prestige to P100!");
        assert_eq!(state.prestige_rank, 100);
    }

    #[test]
    fn test_trigger_max_attributes() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        state.prestige_rank = 10;

        let msg = trigger_max_attributes(&mut state, &enhancement);
        assert_eq!(msg, "All attributes set to cap!");

        let cap = state.get_attribute_cap(); // 20 + 10*5 = 70
        for attr in crate::character::attributes::AttributeType::all() {
            assert_eq!(state.attributes.get(attr), cap);
        }
    }

    // Helper to build the canonical trigger_selected prerequisites
    fn make_test_fixtures() -> (
        GameState,
        Haven,
        EnhancementProgress,
        DeepState,
        crate::achievements::Achievements,
        LoomState,
        LoomUiState,
    ) {
        (
            GameState::new("Test".to_string(), 0),
            Haven::new(),
            EnhancementProgress::new(),
            DeepState::new(),
            crate::achievements::Achievements::default(),
            LoomState::new(),
            LoomUiState::new(),
        )
    }

    fn select_action(menu: &mut DebugMenu, action: DebugAction) {
        // Navigate to the correct category and position for the given action
        let global_idx = action.option_index();
        // Find which category slice contains this action
        let categories = [
            (DebugCategory::Challenges, CHALLENGE_ACTIONS),
            (DebugCategory::World, WORLD_ACTIONS),
            (DebugCategory::Resources, RESOURCE_ACTIONS),
            (DebugCategory::Items, ITEM_ACTIONS),
            (DebugCategory::Deep, DEEP_ACTIONS),
            (DebugCategory::Zones, ZONE_ACTIONS),
            (DebugCategory::Character, CHARACTER_ACTIONS),
        ];
        for (cat, slice) in categories.iter() {
            for (idx, a) in slice.iter().enumerate() {
                if a.option_index() == global_idx {
                    // Navigate to this category
                    menu.open();
                    while menu.current_category() != *cat {
                        menu.navigate_next_category();
                    }
                    menu.selected_index = idx;
                    return;
                }
            }
        }
        panic!("Action not found in any category slice");
    }

    // 1. Set P100 → Unlock L3 → verify zones 12-14 are unlocked
    #[test]
    fn test_set_p100_then_unlock_l3_unlocks_zones_12_to_14() {
        let (
            mut state,
            mut haven,
            mut enhancement,
            mut deep,
            mut achievements,
            mut loom,
            mut loom_ui,
        ) = make_test_fixtures();
        let mut menu = DebugMenu::new();

        // Set prestige to P100
        trigger_set_prestige(&mut state, &enhancement, 100);
        crate::zones::access::sync_account_zone_unlocks(
            &mut state.zone_progression,
            achievements.is_unlocked(crate::achievements::AchievementId::StormsEnd),
            deep.persistent.fracture_zone_cap,
            state.prestige_rank,
            30,
            state.ascension_level,
        );
        state.cached_fracture_zone_cap = deep.persistent.fracture_zone_cap;
        assert_eq!(state.prestige_rank, 100);

        // Unlock Deep Layer 3 via trigger_selected
        select_action(&mut menu, DebugAction::UnlockDeepLayer(3));
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        // Zones 12-14 should now be unlocked (prestige=100 >= 50 requirement)
        for zone_id in 12..=14 {
            assert!(
                state.zone_progression.is_zone_unlocked(zone_id),
                "Zone {zone_id} should be unlocked after SetP100 + UnlockL3"
            );
        }
        // Zone 15 should NOT be unlocked (cap is 14)
        assert!(
            !state.zone_progression.is_zone_unlocked(15),
            "Zone 15 should not be unlocked when cap is 14"
        );
    }

    // 2. Unlock L3 → Set P100 (reverse order) → verify zones 12-14 are unlocked
    #[test]
    fn test_unlock_l3_then_set_p100_unlocks_zones_12_to_14() {
        let (
            mut state,
            mut haven,
            mut enhancement,
            mut deep,
            mut achievements,
            mut loom,
            mut loom_ui,
        ) = make_test_fixtures();
        let mut menu = DebugMenu::new();

        // First unlock L3 (Deep auto-discovered, but prestige=0 won't unlock zones yet)
        select_action(&mut menu, DebugAction::UnlockDeepLayer(3));
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        // Zones 12-14 should NOT be unlocked yet (prestige=0 < 50 requirement)
        for zone_id in 12..=14 {
            assert!(
                !state.zone_progression.is_zone_unlocked(zone_id),
                "Zone {zone_id} should not be unlocked before setting prestige"
            );
        }

        // Now set prestige to P100 via trigger_selected
        select_action(&mut menu, DebugAction::SetPrestige(100));
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        // After sync (called inside trigger_selected), zones 12-14 should be unlocked
        for zone_id in 12..=14 {
            assert!(
                state.zone_progression.is_zone_unlocked(zone_id),
                "Zone {zone_id} should be unlocked after UnlockL3 + SetP100"
            );
        }
    }

    // 3. Forge Stormbreaker → verify both TheStormbreaker AND StormsEnd are unlocked
    #[test]
    fn test_forge_stormbreaker_unlocks_both_achievements() {
        let (
            mut state,
            mut haven,
            mut enhancement,
            mut deep,
            mut achievements,
            mut loom,
            mut loom_ui,
        ) = make_test_fixtures();
        let mut menu = DebugMenu::new();

        assert!(!achievements.is_unlocked(crate::achievements::AchievementId::TheStormbreaker));
        assert!(!achievements.is_unlocked(crate::achievements::AchievementId::StormsEnd));

        select_action(&mut menu, DebugAction::TriggerForgeStormbreaker);
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        assert!(
            achievements.is_unlocked(crate::achievements::AchievementId::TheStormbreaker),
            "TheStormbreaker achievement should be unlocked"
        );
        assert!(
            achievements.is_unlocked(crate::achievements::AchievementId::StormsEnd),
            "StormsEnd achievement should be unlocked"
        );
    }

    // 4. Forge Stormbreaker at P25 → verify zone 11 is unlocked
    #[test]
    fn test_forge_stormbreaker_at_p25_unlocks_zone_11() {
        let (
            mut state,
            mut haven,
            mut enhancement,
            mut deep,
            mut achievements,
            mut loom,
            mut loom_ui,
        ) = make_test_fixtures();
        let mut menu = DebugMenu::new();

        // Set prestige to P25
        trigger_set_prestige(&mut state, &enhancement, 25);

        // Forge Stormbreaker via trigger_selected (which calls sync after)
        select_action(&mut menu, DebugAction::TriggerForgeStormbreaker);
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        assert!(
            achievements.is_unlocked(crate::achievements::AchievementId::StormsEnd),
            "StormsEnd should be unlocked"
        );
        // Zone 11 requires StormsEnd AND prestige >= 25
        assert!(
            state.zone_progression.is_zone_unlocked(11),
            "Zone 11 should be unlocked after ForgeStormbreaker at P25"
        );
    }

    // 5. Forge Stormbreaker at P0 → verify zone 11 is NOT unlocked (needs P25)
    #[test]
    fn test_forge_stormbreaker_at_p0_does_not_unlock_zone_11() {
        let (
            mut state,
            mut haven,
            mut enhancement,
            mut deep,
            mut achievements,
            mut loom,
            mut loom_ui,
        ) = make_test_fixtures();
        let mut menu = DebugMenu::new();

        // prestige_rank starts at 0
        assert_eq!(state.prestige_rank, 0);

        select_action(&mut menu, DebugAction::TriggerForgeStormbreaker);
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        assert!(
            achievements.is_unlocked(crate::achievements::AchievementId::StormsEnd),
            "StormsEnd should still be unlocked by the forge action"
        );
        // Zone 11 needs prestige >= 25 (the Expanse's prestige_requirement)
        assert!(
            !state.zone_progression.is_zone_unlocked(11),
            "Zone 11 should NOT be unlocked at P0 even with StormsEnd"
        );
    }

    // 6. Set P100 → Unlock L3 → Forge Stormbreaker → verify full zone track accessible
    #[test]
    fn test_set_p100_unlock_l3_forge_stormbreaker_full_zone_track() {
        let (
            mut state,
            mut haven,
            mut enhancement,
            mut deep,
            mut achievements,
            mut loom,
            mut loom_ui,
        ) = make_test_fixtures();
        let mut menu = DebugMenu::new();

        // Step 1: Set prestige P100
        select_action(&mut menu, DebugAction::SetPrestige(100));
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        // Step 2: Unlock Deep L3
        select_action(&mut menu, DebugAction::UnlockDeepLayer(3));
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        // Step 3: Forge Stormbreaker
        select_action(&mut menu, DebugAction::TriggerForgeStormbreaker);
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        // Zone 11 unlocked (StormsEnd + P100 >= P25)
        assert!(
            state.zone_progression.is_zone_unlocked(11),
            "Zone 11 should be unlocked"
        );
        // Zones 12-14 unlocked (L3 + P100 >= P50)
        for zone_id in 12..=14 {
            assert!(
                state.zone_progression.is_zone_unlocked(zone_id),
                "Zone {zone_id} should be unlocked"
            );
        }
        // Zone 15 NOT unlocked (cap=14)
        assert!(
            !state.zone_progression.is_zone_unlocked(15),
            "Zone 15 should not be unlocked when cap is 14"
        );
    }

    // 7. Unlock L30 at P300 → verify all zones 12-30 are unlocked
    #[test]
    fn test_unlock_l30_at_p300_unlocks_all_fracture_zones() {
        let (
            mut state,
            mut haven,
            mut enhancement,
            mut deep,
            mut achievements,
            mut loom,
            mut loom_ui,
        ) = make_test_fixtures();
        let mut menu = DebugMenu::new();

        // Set prestige to P300 first
        trigger_set_prestige(&mut state, &enhancement, 300);

        // Unlock Deep L30 via trigger_selected
        select_action(&mut menu, DebugAction::UnlockDeepLayer(30));
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        // All zones 12-30 should be unlocked (P300 meets all prestige requirements)
        for zone_id in 12..=30 {
            assert!(
                state.zone_progression.is_zone_unlocked(zone_id),
                "Zone {zone_id} should be unlocked after UnlockL30 at P300"
            );
        }
        assert_eq!(deep.persistent.fracture_zone_cap, 30);
    }

    // 8. Unlock L30 at P100 → verify only zones 12-20 unlocked (P100 < P150 for Z21+)
    #[test]
    fn test_unlock_l30_at_p100_only_unlocks_zones_up_to_20() {
        let (
            mut state,
            mut haven,
            mut enhancement,
            mut deep,
            mut achievements,
            mut loom,
            mut loom_ui,
        ) = make_test_fixtures();
        let mut menu = DebugMenu::new();

        // Set prestige to P100
        trigger_set_prestige(&mut state, &enhancement, 100);

        // Unlock Deep L30 via trigger_selected
        select_action(&mut menu, DebugAction::UnlockDeepLayer(30));
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        // cap is 30 but prestige=100 only meets requirements up to Z20 (P100)
        for zone_id in 12..=20 {
            assert!(
                state.zone_progression.is_zone_unlocked(zone_id),
                "Zone {zone_id} should be unlocked at P100"
            );
        }
        // Zones 21-30 require P150+ which we don't have
        for zone_id in 21..=30 {
            assert!(
                !state.zone_progression.is_zone_unlocked(zone_id),
                "Zone {zone_id} should NOT be unlocked at P100 (requires P150+)"
            );
        }
    }

    // 9. Verify cached_fracture_zone_cap is updated after unlock deep layer action
    #[test]
    fn test_cached_fracture_zone_cap_updated_after_unlock_deep_layer() {
        let (
            mut state,
            mut haven,
            mut enhancement,
            mut deep,
            mut achievements,
            mut loom,
            mut loom_ui,
        ) = make_test_fixtures();
        let mut menu = DebugMenu::new();

        assert_eq!(state.cached_fracture_zone_cap, 0);

        // Unlock Deep L3
        select_action(&mut menu, DebugAction::UnlockDeepLayer(3));
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        assert_eq!(
            state.cached_fracture_zone_cap, 14,
            "cached_fracture_zone_cap should be 14 after UnlockL3"
        );

        // Unlock Deep L7 (extends cap to 17)
        select_action(&mut menu, DebugAction::UnlockDeepLayer(7));
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        assert_eq!(
            state.cached_fracture_zone_cap, 17,
            "cached_fracture_zone_cap should be 17 after UnlockL7"
        );

        // Unlock Deep L30 (extends cap to 30)
        select_action(&mut menu, DebugAction::UnlockDeepLayer(30));
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        assert_eq!(
            state.cached_fracture_zone_cap, 30,
            "cached_fracture_zone_cap should be 30 after UnlockL30"
        );
    }

    // 10. Forge Stormbreaker twice → second time returns "already forged" message
    #[test]
    fn test_forge_stormbreaker_twice_returns_already_forged() {
        let (_state, _haven, _enhancement, _deep, mut achievements, _loom, _loom_ui) =
            make_test_fixtures();

        // First forge
        let msg = trigger_forge_stormbreaker(&mut achievements);
        assert_eq!(msg, "Forged Stormbreaker!");

        // Second forge attempt
        let msg2 = trigger_forge_stormbreaker(&mut achievements);
        assert_eq!(msg2, "Stormbreaker already forged!");
    }

    // Bonus: verify trigger_selected version also returns "already forged" on second call
    #[test]
    fn test_trigger_selected_forge_stormbreaker_twice() {
        let (
            mut state,
            mut haven,
            mut enhancement,
            mut deep,
            mut achievements,
            mut loom,
            mut loom_ui,
        ) = make_test_fixtures();
        let mut menu = DebugMenu::new();

        // First forge via trigger_selected
        select_action(&mut menu, DebugAction::TriggerForgeStormbreaker);
        let msg = menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );
        assert_eq!(msg, "Forged Stormbreaker!");

        // Second forge via trigger_selected
        select_action(&mut menu, DebugAction::TriggerForgeStormbreaker);
        let msg2 = menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );
        assert_eq!(msg2, "Stormbreaker already forged!");
    }

    // Additional: Verify that trigger_selected correctly syncs zone unlocks after SetPrestige
    // when fracture cap was previously set
    #[test]
    fn test_zone_sync_happens_after_set_prestige_with_existing_fracture_cap() {
        let (
            mut state,
            mut haven,
            mut enhancement,
            mut deep,
            mut achievements,
            mut loom,
            mut loom_ui,
        ) = make_test_fixtures();
        let mut menu = DebugMenu::new();

        // First unlock L3 (cap=14, but prestige=0 won't unlock zones)
        select_action(&mut menu, DebugAction::UnlockDeepLayer(3));
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );
        assert!(!state.zone_progression.is_zone_unlocked(12));

        // Now bump prestige to P50 — sync inside trigger_selected should unlock 12-14
        select_action(&mut menu, DebugAction::SetPrestige(50));
        menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            &mut loom,
            &mut loom_ui,
        );

        for zone_id in 12..=14 {
            assert!(
                state.zone_progression.is_zone_unlocked(zone_id),
                "Zone {zone_id} should be unlocked after SetP50 with cap=14"
            );
        }
    }
}
