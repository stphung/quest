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
    TriggerHavenDiscovery,
    TriggerSoulforgeDiscovery,
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
    TravelToZone(u32),
    SetPrestige(u32),
    SetLevel(u32),
    MaxAttributes,
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
    DebugAction::TriggerHavenDiscovery,
    DebugAction::TriggerSoulforgeDiscovery,
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
    // Character actions (prestige, levels)
    DebugAction::SetPrestige(1),
    DebugAction::SetPrestige(5),
    DebugAction::SetPrestige(10),
    DebugAction::SetPrestige(100),
    DebugAction::SetLevel(10),
    DebugAction::SetLevel(50),
    DebugAction::MaxAttributes,
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
];
const CHARACTER_ACTIONS: &[DebugAction] = &[
    DebugAction::SetPrestige(1),
    DebugAction::SetPrestige(5),
    DebugAction::SetPrestige(10),
    DebugAction::SetPrestige(100),
    DebugAction::SetLevel(10),
    DebugAction::SetLevel(50),
    DebugAction::MaxAttributes,
];
const BORDER_OPTION_START_INDEX: usize = DEBUG_ACTIONS.len();

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
            Self::TriggerHavenDiscovery => 12,
            Self::TriggerSoulforgeDiscovery => 13,
            Self::TriggerForgeAsprika => 14,
            Self::TriggerForgeSleipnir => 15,
            Self::TriggerForgeMegingjord => 16,
            Self::TriggerGrantStormglass => 17,
            Self::TriggerDiscoverStormglass => 18,
            Self::TriggerGrant100kStormglass => 19,
            Self::TriggerEtchRandomSigils => 20,
            Self::TriggerEtchSPlusSigil => 21,
            Self::TriggerForceOvercharge => 22,
            Self::TriggerDeepDiscovery => 23,
            Self::TriggerDeepGrantMarks => 24,
            Self::TriggerDeepRefreshMissionPool => 25,
            Self::TriggerDeepRefreshRecruits => 26,
            Self::TriggerDeepClearFrontierLayer => 27,
            Self::TriggerDeepCompleteActiveMissions => 28,
            Self::TravelToZone(zone_id) => 29 + zone_id as usize - 1, // 29-39
            Self::SetPrestige(amount) => match amount {
                1 => 40,
                5 => 41,
                10 => 42,
                _ => 43, // 100
            },
            Self::SetLevel(amount) => {
                if amount == 10 {
                    44
                } else {
                    45
                }
            }
            Self::MaxAttributes => 46,
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
            Self::TriggerHavenDiscovery => "Trigger Haven Discovery",
            Self::TriggerSoulforgeDiscovery => "Trigger Soulforge Discovery",
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
                _ => "Travel to Unknown Zone",
            },
            Self::SetPrestige(amount) => match amount {
                1 => "Set Prestige to P1",
                5 => "Set Prestige to P5",
                10 => "Set Prestige to P10",
                _ => "Set Prestige to P100",
            },
            Self::SetLevel(amount) => {
                if amount == 10 {
                    "Set Level to 10"
                } else {
                    "Set Level to 50"
                }
            }
            Self::MaxAttributes => "Max All Attributes",
        }
    }

    fn run(
        self,
        state: &mut GameState,
        haven: &mut Haven,
        enhancement: &mut EnhancementProgress,
        deep: &mut DeepState,
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
            Self::TriggerHavenDiscovery => trigger_haven_discovery(haven),
            Self::TriggerSoulforgeDiscovery => trigger_soulforge_discovery(enhancement),
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
            Self::TravelToZone(zone_id) => trigger_travel_to_zone(state, enhancement, zone_id),
            Self::SetPrestige(amount) => trigger_set_prestige(state, enhancement, amount),
            Self::SetLevel(amount) => trigger_set_level(state, enhancement, amount),
            Self::MaxAttributes => trigger_max_attributes(state, enhancement),
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
            DebugCategory::Borders => BORDER_OPTION_START_INDEX + visible_index,
        }
    }

    fn selected_option_global_index(&self) -> usize {
        self.global_option_index_for_visible(self.selected_index)
    }

    /// Trigger the selected debug action. Returns a message describing what happened.
    pub fn trigger_selected(
        &mut self,
        state: &mut GameState,
        haven: &mut Haven,
        enhancement: &mut EnhancementProgress,
        deep: &mut DeepState,
        achievements: &mut crate::achievements::Achievements,
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
            .map(|action| action.run(state, haven, enhancement, deep))
            .unwrap_or("Unknown option");
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

    // Unlock the target zone (and all zones at or below its prestige tier)
    for z in zones {
        if z.prestige_requirement <= state.prestige_rank {
            state.zone_progression.unlock_zone(z.id);
        }
    }

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

    // Unlock zones accessible at new prestige rank
    let zones = get_all_zones();
    for z in zones {
        if z.prestige_requirement <= state.prestige_rank {
            state.zone_progression.unlock_zone(z.id);
        }
    }

    // Recalculate stats
    state.recalculate_derived_stats(&enhancement.levels);

    match rank {
        1 => "Set Prestige to P1!",
        5 => "Set Prestige to P5!",
        10 => "Set Prestige to P10!",
        _ => "Set Prestige to P100!",
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

    if target_level == 10 {
        "Set Level to 10!"
    } else {
        "Set Level to 50!"
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
        assert_eq!(menu.selected_option_global_index(), 11);

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
        for _ in 0..7 {
            menu.navigate_next_category();
        }
        assert_eq!(menu.current_category(), DebugCategory::Borders);

        let mut state = GameState::new("Test".to_string(), 0);
        let mut haven = Haven::new();
        let mut enhancement = EnhancementProgress::new();
        let mut deep = DeepState::new();
        let mut achievements = crate::achievements::Achievements::default();
        let msg = menu.trigger_selected(
            &mut state,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
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
        assert_eq!(state.zone_progression.current_zone_id, 1);

        let msg = trigger_travel_to_zone(&mut state, &enhancement, 5);
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
        state.active_dungeon = Some(generate_dungeon(1, 0, 1));

        trigger_travel_to_zone(&mut state, &enhancement, 1);
        assert!(state.active_dungeon.is_none());
    }

    #[test]
    fn test_trigger_travel_no_prestige_downgrade() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        state.prestige_rank = 20;

        trigger_travel_to_zone(&mut state, &enhancement, 1);
        // Traveling to P0 zone should not lower prestige
        assert_eq!(state.prestige_rank, 20);
    }

    #[test]
    fn test_trigger_travel_unlocks_intermediate_zones() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();

        trigger_travel_to_zone(&mut state, &enhancement, 7);
        // Should unlock zones 1-8 (all P0, P5, P10, P15 zones)
        for zone_id in 1..=8 {
            assert!(
                state.zone_progression.is_zone_unlocked(zone_id),
                "Zone {zone_id} should be unlocked after traveling to Zone 7 (P15)"
            );
        }
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

        let msg = trigger_set_level(&mut state, &enhancement, 10);
        assert_eq!(msg, "Set Level to 10!");
        assert_eq!(state.character_level, 10);
    }

    #[test]
    fn test_trigger_set_level_distributes_attributes() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        let initial_sum: u32 = crate::character::attributes::AttributeType::all()
            .iter()
            .map(|a| state.attributes.get(*a))
            .sum();

        trigger_set_level(&mut state, &enhancement, 10);

        let final_sum: u32 = crate::character::attributes::AttributeType::all()
            .iter()
            .map(|a| state.attributes.get(*a))
            .sum();
        // 9 levels (1->10) * 3 points = 27 attribute points gained
        assert_eq!(final_sum, initial_sum + 27);
    }

    #[test]
    fn test_trigger_set_level_noop_if_already_at_target() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        state.character_level = 50;

        trigger_set_level(&mut state, &enhancement, 10);
        // Should not lower level — noop when already above target
        assert_eq!(state.character_level, 50);
    }

    #[test]
    fn test_character_category_has_7_options() {
        assert_eq!(option_count_for_category(DebugCategory::Character), 7);
    }

    #[test]
    fn test_zones_category_has_11_options() {
        assert_eq!(option_count_for_category(DebugCategory::Zones), 11);
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
}
