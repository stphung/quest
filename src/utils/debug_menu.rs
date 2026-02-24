//! Debug menu for testing chance-based discoveries.
//!
//! Activated with `--debug` flag. Press backtick to toggle menu.

use crate::achievements::{UiBorderStyle, SELECTABLE_UI_BORDER_STYLES};
use crate::challenges::menu::{create_challenge, ChallengeType};
use crate::core::game_state::GameState;
use crate::dungeon::generation::generate_dungeon;
use crate::enhancement::EnhancementProgress;
use crate::fishing::generation::generate_fishing_session;
use crate::god_items;
use crate::haven::Haven;
use crate::items;

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
        }
    }

    fn run(
        self,
        state: &mut GameState,
        haven: &mut Haven,
        enhancement: &mut EnhancementProgress,
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
        DebugCategory::Borders => SELECTABLE_UI_BORDER_STYLES.len(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugCategory {
    Challenges,
    World,
    Resources,
    Items,
    Borders,
}

impl DebugCategory {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Challenges => "Challenges",
            Self::World => "World",
            Self::Resources => "Resources",
            Self::Items => "Items",
            Self::Borders => "Borders",
        }
    }
}

pub const DEBUG_CATEGORIES: &[DebugCategory] = &[
    DebugCategory::Challenges,
    DebugCategory::World,
    DebugCategory::Resources,
    DebugCategory::Items,
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
            .map(|action| action.run(state, haven, enhancement))
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
        assert_eq!(menu.current_category(), DebugCategory::Items);
    }

    #[test]
    fn test_border_preview_does_not_close_menu() {
        let mut menu = DebugMenu::new();
        menu.open();
        for _ in 0..4 {
            menu.navigate_next_category();
        }
        assert_eq!(menu.current_category(), DebugCategory::Borders);

        let mut state = GameState::new("Test".to_string(), 0);
        let mut haven = Haven::new();
        let mut enhancement = EnhancementProgress::new();
        let mut achievements = crate::achievements::Achievements::default();
        let msg =
            menu.trigger_selected(&mut state, &mut haven, &mut enhancement, &mut achievements);

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
}
