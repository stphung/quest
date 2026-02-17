//! Debug menu for testing chance-based discoveries.
//!
//! Activated with `--debug` flag. Press backtick to toggle menu.

use crate::challenges::menu::{create_challenge, ChallengeType};
use crate::core::game_state::GameState;
use crate::dungeon::generation::generate_dungeon;
use crate::enhancement::EnhancementProgress;
use crate::fishing::generation::generate_fishing_session;
use crate::god_items::{self, GodItemProgress, GodItemState};
use crate::haven::Haven;
use crate::items;

/// Menu options available in debug mode
pub const DEBUG_OPTIONS: &[&str] = &[
    "Trigger Dungeon",
    "Trigger Fishing",
    "Trigger Chess Challenge",
    "Trigger Morris Challenge",
    "Trigger Gomoku Challenge",
    "Trigger Minesweeper Challenge",
    "Trigger Rune Challenge",
    "Trigger Go Challenge",
    "Trigger Flappy Bird Challenge",
    "Trigger JezzBall Challenge",
    "Trigger Snake Challenge",
    "Trigger Sigil Surge Challenge",
    "Trigger Haven Discovery",
    "Trigger Soulforge Discovery",
    "Discover Asprika (God Item quest)",
    "Complete Asprika Milestones",
    "Forge Asprika",
    "Discover Sleipnir (God Item quest)",
    "Complete Sleipnir Milestones",
    "Forge Sleipnir",
    "Discover Megingjord (God Item quest)",
    "Complete Megingjord Milestones",
    "Forge Megingjord",
];

/// Debug menu state
#[derive(Debug, Clone, Default)]
pub struct DebugMenu {
    pub is_open: bool,
    pub selected_index: usize,
}

impl DebugMenu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.is_open = true;
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

    pub fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn navigate_down(&mut self) {
        if self.selected_index + 1 < DEBUG_OPTIONS.len() {
            self.selected_index += 1;
        }
    }

    /// Trigger the selected debug action. Returns a message describing what happened.
    pub fn trigger_selected(
        &mut self,
        state: &mut GameState,
        haven: &mut Haven,
        enhancement: &mut EnhancementProgress,
        god_item_progress: &mut GodItemProgress,
    ) -> &'static str {
        let msg = match self.selected_index {
            0 => trigger_dungeon(state),
            1 => trigger_fishing(state),
            2 => trigger_chess_challenge(state),
            3 => trigger_morris_challenge(state),
            4 => trigger_gomoku_challenge(state),
            5 => trigger_minesweeper_challenge(state),
            6 => trigger_rune_challenge(state),
            7 => trigger_go_challenge(state),
            8 => trigger_flappy_challenge(state),
            9 => trigger_jezzball_challenge(state),
            10 => trigger_snake_challenge(state),
            11 => trigger_runic_shift_challenge(state),
            12 => trigger_haven_discovery(haven),
            13 => trigger_soulforge_discovery(enhancement),
            14 => trigger_discover_asprika(god_item_progress),
            15 => trigger_complete_asprika_milestones(god_item_progress),
            16 => trigger_forge_asprika(state, god_item_progress, enhancement),
            17 => trigger_discover_sleipnir(god_item_progress),
            18 => trigger_complete_sleipnir_milestones(god_item_progress),
            19 => trigger_forge_sleipnir(state, god_item_progress, enhancement),
            20 => trigger_discover_megingjord(god_item_progress),
            21 => trigger_complete_megingjord_milestones(god_item_progress),
            22 => trigger_forge_megingjord(state, god_item_progress, enhancement),
            _ => "Unknown option",
        };
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

fn trigger_discover_asprika(god_item_progress: &mut GodItemProgress) -> &'static str {
    if god_item_progress.asprika_state != GodItemState::Undiscovered {
        return "Asprika already discovered!";
    }
    god_item_progress.asprika_state = GodItemState::Discovered;
    "Asprika quest discovered!"
}

fn trigger_complete_asprika_milestones(god_item_progress: &mut GodItemProgress) -> &'static str {
    if god_item_progress.asprika_state == GodItemState::Forged {
        return "Asprika already forged!";
    }
    if god_item_progress.asprika_state == GodItemState::Undiscovered {
        god_item_progress.asprika_state = GodItemState::Discovered;
    }
    god_item_progress.asprika_milestones.expanse_cycle_complete = true;
    god_item_progress.asprika_state = GodItemState::ReadyToForge;
    "Asprika milestones completed!"
}

fn trigger_forge_asprika(
    state: &mut GameState,
    god_item_progress: &mut GodItemProgress,
    enhancement: &EnhancementProgress,
) -> &'static str {
    if god_item_progress.asprika_state == GodItemState::Forged {
        return "Asprika already forged!";
    }
    let asprika = god_items::asprika_definition().to_item();
    state
        .equipment
        .set(items::EquipmentSlot::Armor, Some(asprika));
    god_item_progress.asprika_state = GodItemState::Forged;
    // Recalculate derived stats with new equipment
    state.recalculate_prestige_bonuses();
    state.recalculate_derived_stats(&enhancement.levels);
    "Asprika forged and equipped!"
}

fn trigger_discover_sleipnir(god_item_progress: &mut GodItemProgress) -> &'static str {
    if god_item_progress.sleipnir_state != GodItemState::Undiscovered {
        return "Sleipnir already discovered!";
    }
    god_item_progress.sleipnir_state = GodItemState::Discovered;
    "Sleipnir quest discovered!"
}

fn trigger_complete_sleipnir_milestones(god_item_progress: &mut GodItemProgress) -> &'static str {
    if god_item_progress.sleipnir_state == GodItemState::Forged {
        return "Sleipnir already forged!";
    }
    if god_item_progress.sleipnir_state == GodItemState::Undiscovered {
        god_item_progress.sleipnir_state = GodItemState::Discovered;
    }
    god_item_progress
        .sleipnir_milestones
        .master_challenge_types_won =
        vec!["chess".to_string(), "go".to_string(), "snake".to_string()];
    god_item_progress
        .sleipnir_milestones
        .highest_enhancement_level = 7;
    god_item_progress.sleipnir_state = GodItemState::ReadyToForge;
    "Sleipnir milestones completed!"
}

fn trigger_forge_sleipnir(
    state: &mut GameState,
    god_item_progress: &mut GodItemProgress,
    enhancement: &EnhancementProgress,
) -> &'static str {
    if god_item_progress.sleipnir_state == GodItemState::Forged {
        return "Sleipnir already forged!";
    }
    let sleipnir = god_items::sleipnir_definition().to_item();
    state
        .equipment
        .set(items::EquipmentSlot::Boots, Some(sleipnir));
    god_item_progress.sleipnir_state = GodItemState::Forged;
    state.recalculate_prestige_bonuses();
    state.recalculate_derived_stats(&enhancement.levels);
    "Sleipnir forged and equipped!"
}

fn trigger_discover_megingjord(god_item_progress: &mut GodItemProgress) -> &'static str {
    if god_item_progress.megingjord_state != GodItemState::Undiscovered {
        return "Megingjord already discovered!";
    }
    god_item_progress.megingjord_state = GodItemState::Discovered;
    "Megingjord quest discovered!"
}

fn trigger_complete_megingjord_milestones(god_item_progress: &mut GodItemProgress) -> &'static str {
    if god_item_progress.megingjord_state == GodItemState::Forged {
        return "Megingjord already forged!";
    }
    if god_item_progress.megingjord_state == GodItemState::Undiscovered {
        god_item_progress.megingjord_state = GodItemState::Discovered;
    }
    god_item_progress
        .megingjord_milestones
        .highest_enhancement_level = 9;
    god_item_progress.megingjord_state = GodItemState::ReadyToForge;
    "Megingjord milestones completed!"
}

fn trigger_forge_megingjord(
    state: &mut GameState,
    god_item_progress: &mut GodItemProgress,
    enhancement: &EnhancementProgress,
) -> &'static str {
    if god_item_progress.megingjord_state == GodItemState::Forged {
        return "Megingjord already forged!";
    }
    let megingjord = god_items::megingjord_definition().to_item();
    state
        .equipment
        .set(items::EquipmentSlot::Ring, Some(megingjord));
    god_item_progress.megingjord_state = GodItemState::Forged;
    state.recalculate_prestige_bonuses();
    state.recalculate_derived_stats(&enhancement.levels);
    "Megingjord forged and equipped!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_navigation() {
        let mut menu = DebugMenu::new();
        menu.open();
        assert_eq!(menu.selected_index, 0);

        menu.navigate_down();
        assert_eq!(menu.selected_index, 1);

        for _ in 0..32 {
            menu.navigate_down();
        }
        assert_eq!(menu.selected_index, DEBUG_OPTIONS.len() - 1);

        // Can't go past end
        menu.navigate_down();
        assert_eq!(menu.selected_index, DEBUG_OPTIONS.len() - 1);

        menu.navigate_up();
        assert_eq!(menu.selected_index, DEBUG_OPTIONS.len() - 2);

        // Can't go before start
        for _ in 0..32 {
            menu.navigate_up();
        }
        assert_eq!(menu.selected_index, 0);
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
    fn test_trigger_discover_asprika() {
        let mut progress = GodItemProgress::default();
        assert_eq!(progress.asprika_state, GodItemState::Undiscovered);

        let msg = trigger_discover_asprika(&mut progress);
        assert_eq!(msg, "Asprika quest discovered!");
        assert_eq!(progress.asprika_state, GodItemState::Discovered);

        // Can't discover again
        let msg = trigger_discover_asprika(&mut progress);
        assert_eq!(msg, "Asprika already discovered!");
    }

    #[test]
    fn test_trigger_complete_asprika_milestones() {
        let mut progress = GodItemProgress::default();

        // Auto-discovers if undiscovered
        let msg = trigger_complete_asprika_milestones(&mut progress);
        assert_eq!(msg, "Asprika milestones completed!");
        assert_eq!(progress.asprika_state, GodItemState::ReadyToForge);
        assert!(progress.asprika_milestones.all_met());

        // Can't complete if already forged
        progress.asprika_state = GodItemState::Forged;
        let msg = trigger_complete_asprika_milestones(&mut progress);
        assert_eq!(msg, "Asprika already forged!");
    }

    #[test]
    fn test_trigger_forge_asprika() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut progress = GodItemProgress::default();
        let enhancement = EnhancementProgress::new();

        let msg = trigger_forge_asprika(&mut state, &mut progress, &enhancement);
        assert_eq!(msg, "Asprika forged and equipped!");
        assert_eq!(progress.asprika_state, GodItemState::Forged);
        assert!(state.equipment.get(items::EquipmentSlot::Armor).is_some());

        // Can't forge again
        let msg = trigger_forge_asprika(&mut state, &mut progress, &enhancement);
        assert_eq!(msg, "Asprika already forged!");
    }

    #[test]
    fn test_trigger_discover_sleipnir() {
        let mut progress = GodItemProgress::default();
        let msg = trigger_discover_sleipnir(&mut progress);
        assert_eq!(msg, "Sleipnir quest discovered!");
        assert_eq!(progress.sleipnir_state, GodItemState::Discovered);

        let msg = trigger_discover_sleipnir(&mut progress);
        assert_eq!(msg, "Sleipnir already discovered!");
    }

    #[test]
    fn test_trigger_complete_sleipnir_milestones() {
        let mut progress = GodItemProgress::default();
        let msg = trigger_complete_sleipnir_milestones(&mut progress);
        assert_eq!(msg, "Sleipnir milestones completed!");
        assert_eq!(progress.sleipnir_state, GodItemState::ReadyToForge);
        assert!(progress.sleipnir_milestones.all_met());

        progress.sleipnir_state = GodItemState::Forged;
        let msg = trigger_complete_sleipnir_milestones(&mut progress);
        assert_eq!(msg, "Sleipnir already forged!");
    }

    #[test]
    fn test_trigger_forge_sleipnir() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut progress = GodItemProgress::default();
        let enhancement = EnhancementProgress::new();

        let msg = trigger_forge_sleipnir(&mut state, &mut progress, &enhancement);
        assert_eq!(msg, "Sleipnir forged and equipped!");
        assert_eq!(progress.sleipnir_state, GodItemState::Forged);
        assert!(state.equipment.get(items::EquipmentSlot::Boots).is_some());

        let msg = trigger_forge_sleipnir(&mut state, &mut progress, &enhancement);
        assert_eq!(msg, "Sleipnir already forged!");
    }

    #[test]
    fn test_trigger_discover_megingjord() {
        let mut progress = GodItemProgress::default();
        let msg = trigger_discover_megingjord(&mut progress);
        assert_eq!(msg, "Megingjord quest discovered!");
        assert_eq!(progress.megingjord_state, GodItemState::Discovered);

        let msg = trigger_discover_megingjord(&mut progress);
        assert_eq!(msg, "Megingjord already discovered!");
    }

    #[test]
    fn test_trigger_complete_megingjord_milestones() {
        let mut progress = GodItemProgress::default();
        let msg = trigger_complete_megingjord_milestones(&mut progress);
        assert_eq!(msg, "Megingjord milestones completed!");
        assert_eq!(progress.megingjord_state, GodItemState::ReadyToForge);
        assert!(progress.megingjord_milestones.all_met());

        progress.megingjord_state = GodItemState::Forged;
        let msg = trigger_complete_megingjord_milestones(&mut progress);
        assert_eq!(msg, "Megingjord already forged!");
    }

    #[test]
    fn test_trigger_forge_megingjord() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut progress = GodItemProgress::default();
        let enhancement = EnhancementProgress::new();

        let msg = trigger_forge_megingjord(&mut state, &mut progress, &enhancement);
        assert_eq!(msg, "Megingjord forged and equipped!");
        assert_eq!(progress.megingjord_state, GodItemState::Forged);
        assert!(state.equipment.get(items::EquipmentSlot::Ring).is_some());

        let msg = trigger_forge_megingjord(&mut state, &mut progress, &enhancement);
        assert_eq!(msg, "Megingjord already forged!");
    }
}
