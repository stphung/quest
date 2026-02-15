//! Input handling for the Game screen.
//!
//! Extracts the input dispatch logic from main.rs into a clean priority chain.

use crate::enhancement;
use crate::items::EquipmentSlot;

use crate::challenges::chess::logic::{
    apply_game_result as apply_chess_result, process_input as process_chess_input, ChessInput,
};
use crate::challenges::flappy::logic::{
    apply_game_result as apply_flappy_result, process_input as process_flappy_input,
    FlappyBirdInput,
};
use crate::challenges::go::{apply_go_result, process_input as process_go_input, GoInput};
use crate::challenges::gomoku::logic::{
    apply_game_result as apply_gomoku_result, process_input as process_gomoku_input, GomokuInput,
};
use crate::challenges::jezzball::logic::{
    apply_game_result as apply_jezzball_result, process_input as process_jezzball_input,
    JezzballInput,
};
use crate::challenges::menu::{process_input as process_menu_input, MenuInput};
use crate::challenges::minesweeper::logic::{
    apply_game_result as apply_minesweeper_result, process_input as process_minesweeper_input,
    MinesweeperInput,
};
use crate::challenges::morris::logic::{
    apply_game_result as apply_morris_result, process_input as process_morris_input, MorrisInput,
};
use crate::challenges::rune::logic::{
    apply_game_result as apply_rune_result, process_input as process_rune_input, RuneInput,
};
use crate::challenges::snake::logic::{
    apply_game_result as apply_snake_result, process_input as process_snake_input, SnakeInput,
};
use crate::challenges::ActiveMinigame;
use crate::character::prestige::{can_prestige, get_prestige_tier, perform_prestige};
use crate::core::game_logic::OfflineReport;
use crate::core::game_state::GameState;
use crate::haven;
use crate::haven::Haven;
use crate::items;
use crate::utils::debug_menu::DebugMenu;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Haven confirmation dialog state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HavenConfirmation {
    None,
    Build,
    Forge,
}

/// Haven overlay state, shared between CharacterSelect and Game screens.
pub struct HavenUiState {
    pub showing: bool,
    pub selected_room: usize,
    pub confirmation: HavenConfirmation,
}

impl HavenUiState {
    pub fn new() -> Self {
        Self {
            showing: false,
            selected_room: 0,
            confirmation: HavenConfirmation::None,
        }
    }

    pub fn open(&mut self) {
        self.showing = true;
        self.selected_room = 0;
        self.confirmation = HavenConfirmation::None;
    }

    pub fn close(&mut self) {
        self.showing = false;
        self.confirmation = HavenConfirmation::None;
    }
}

// Re-export blacksmith UI types from enhancement module
pub use crate::enhancement::{BlacksmithPhase, BlacksmithUiState, EnhancementResult};

/// Game-screen overlay state. At most one is active at a time.
pub enum GameOverlay {
    None,
    HavenDiscovery,
    BlacksmithDiscovery,
    PrestigeConfirm,
    VaultSelection {
        selected_index: usize,
        selected_slots: Vec<items::EquipmentSlot>,
    },
    OfflineWelcome {
        report: OfflineReport,
    },
    Achievements {
        browser: crate::ui::achievement_browser_scene::AchievementBrowserState,
    },
    /// Achievement unlock celebration modal
    AchievementUnlocked {
        achievements: Vec<crate::achievements::AchievementId>,
    },
    /// Storm Leviathan encounter modal (fishing)
    LeviathanEncounter {
        encounter_number: u8,
    },
}

/// Result of handling a game input event.
pub enum InputResult {
    /// Continue the game loop normally.
    Continue,
    /// Player quit to character select. State should be saved first.
    QuitToSelect,
    /// State was modified (prestige, haven build) and should be saved.
    NeedsSave,
    /// Haven was modified along with state — save both.
    NeedsSaveAll,
    /// Toggle the update details expanded state.
    ToggleUpdateDetails,
}

/// Main dispatcher for Game screen input. Handles the priority chain.
#[allow(clippy::too_many_arguments)]
pub fn handle_game_input(
    key: KeyEvent,
    state: &mut GameState,
    haven: &mut Haven,
    haven_ui: &mut HavenUiState,
    blacksmith_ui: &mut BlacksmithUiState,
    enhancement: &mut enhancement::EnhancementProgress,
    overlay: &mut GameOverlay,
    debug_menu: &mut DebugMenu,
    debug_mode: bool,
    achievements: &mut crate::achievements::Achievements,
    update_available: bool,
    update_expanded: bool,
) -> InputResult {
    // 0. Offline welcome overlay (any key dismisses)
    if matches!(overlay, GameOverlay::OfflineWelcome { .. }) {
        *overlay = GameOverlay::None;
        return InputResult::Continue;
    }

    // 0.25. Storm Leviathan encounter modal (Enter dismisses)
    if matches!(overlay, GameOverlay::LeviathanEncounter { .. }) {
        if matches!(key.code, KeyCode::Enter) {
            *overlay = GameOverlay::None;
        }
        return InputResult::Continue;
    }

    // 0.5. Achievement browser overlay
    if let GameOverlay::Achievements { ref mut browser } = overlay {
        match key.code {
            KeyCode::Esc | KeyCode::Char('a') | KeyCode::Char('A') => {
                achievements.clear_recently_unlocked();
                *overlay = GameOverlay::None;
            }
            KeyCode::Left => browser.prev_category(),
            KeyCode::Right => browser.next_category(),
            KeyCode::Up => browser.move_up(),
            KeyCode::Down => browser.move_down(1000),
            _ => {}
        }
        return InputResult::Continue;
    }

    // 1. Haven discovery modal (blocks all other input)
    if matches!(overlay, GameOverlay::HavenDiscovery) {
        return handle_haven_discovery(key, overlay);
    }

    // 1a. Blacksmith discovery modal (blocks all other input)
    if matches!(overlay, GameOverlay::BlacksmithDiscovery) {
        return handle_blacksmith_discovery(key, overlay);
    }

    // 1b. Achievement unlocked modal (blocks all other input)
    if matches!(overlay, GameOverlay::AchievementUnlocked { .. }) {
        return handle_achievement_unlocked(key, overlay);
    }

    // 2. Haven screen (blocks other input when open)
    if haven_ui.showing {
        return handle_haven(key, state, haven, haven_ui, achievements);
    }

    // 2.5. Blacksmith overlay
    if blacksmith_ui.open {
        return handle_blacksmith(
            key,
            blacksmith_ui,
            enhancement,
            &mut state.prestige_rank,
            &state.equipment,
            achievements,
            &state.character_name,
        );
    }

    // 3. Vault item selection
    if matches!(overlay, GameOverlay::VaultSelection { .. }) {
        return handle_vault_selection(key, state, haven, overlay);
    }

    // 4. Prestige confirmation
    if matches!(overlay, GameOverlay::PrestigeConfirm) {
        return handle_prestige_confirm(key, state, haven, overlay);
    }

    // 5. Debug menu
    if debug_mode {
        if key.code == KeyCode::Char('`') {
            debug_menu.toggle();
            return InputResult::Continue;
        }
        if debug_menu.is_open {
            return handle_debug_menu(key, state, haven, enhancement, overlay, debug_menu);
        }
    }

    // 6. Active minigame
    if state.active_minigame.is_some() {
        return handle_minigame(key, state);
    }

    // 7. Challenge menu
    if state.challenge_menu.is_open {
        return handle_challenge_menu(key, state);
    }

    // 8. Tab to open challenge menu
    if key.code == KeyCode::Tab && !state.challenge_menu.challenges.is_empty() {
        state.challenge_menu.open();
        return InputResult::Continue;
    }

    // 9. Base game input
    handle_base_game(
        key,
        state,
        haven,
        haven_ui,
        blacksmith_ui,
        enhancement,
        overlay,
        achievements,
        update_available,
        update_expanded,
    )
}

fn handle_haven_discovery(key: KeyEvent, overlay: &mut GameOverlay) -> InputResult {
    if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
        *overlay = GameOverlay::None;
    }
    InputResult::Continue
}

fn handle_blacksmith_discovery(key: KeyEvent, overlay: &mut GameOverlay) -> InputResult {
    if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
        *overlay = GameOverlay::None;
    }
    InputResult::Continue
}

const SLOT_ORDER: [EquipmentSlot; 7] = [
    EquipmentSlot::Weapon,
    EquipmentSlot::Armor,
    EquipmentSlot::Helmet,
    EquipmentSlot::Gloves,
    EquipmentSlot::Boots,
    EquipmentSlot::Amulet,
    EquipmentSlot::Ring,
];

fn handle_blacksmith(
    key: KeyEvent,
    blacksmith_ui: &mut BlacksmithUiState,
    enhancement: &mut enhancement::EnhancementProgress,
    prestige_rank: &mut u32,
    equipment: &items::Equipment,
    achievements: &mut crate::achievements::Achievements,
    character_name: &str,
) -> InputResult {
    match blacksmith_ui.phase {
        BlacksmithPhase::Menu => match key.code {
            KeyCode::Up => {
                blacksmith_ui.selected_slot = blacksmith_ui.selected_slot.saturating_sub(1);
                InputResult::Continue
            }
            KeyCode::Down => {
                if blacksmith_ui.selected_slot < 6 {
                    blacksmith_ui.selected_slot += 1;
                }
                InputResult::Continue
            }
            KeyCode::Enter => {
                let slot_index = blacksmith_ui.selected_slot;
                let slot = SLOT_ORDER[slot_index];
                let current_level = enhancement.level(slot_index);

                // Check: item equipped, level < max, can afford
                if equipment.get(slot).is_some()
                    && current_level < enhancement::MAX_ENHANCEMENT_LEVEL
                {
                    let target_level = current_level + 1;
                    let cost = enhancement::enhancement_cost(target_level);
                    if *prestige_rank >= cost {
                        blacksmith_ui.phase = BlacksmithPhase::Confirming;
                    }
                }
                InputResult::Continue
            }
            KeyCode::Esc => {
                blacksmith_ui.close();
                InputResult::Continue
            }
            _ => InputResult::Continue,
        },
        BlacksmithPhase::Confirming => match key.code {
            KeyCode::Enter => {
                let slot_index = blacksmith_ui.selected_slot;
                let current_level = enhancement.level(slot_index);
                let target_level = current_level + 1;
                let cost = enhancement::enhancement_cost(target_level);

                // Deduct prestige cost
                *prestige_rank -= cost;

                // Attempt enhancement
                let mut rng = rand::rng();
                let success = enhancement::attempt_enhancement(enhancement, slot_index, &mut rng);
                let new_level = enhancement.level(slot_index);

                // Track enhancement achievements
                achievements.on_enhancement_upgraded(
                    new_level,
                    &enhancement.levels,
                    enhancement.total_attempts,
                    Some(character_name),
                );

                blacksmith_ui.last_result = Some(EnhancementResult {
                    slot_index,
                    success,
                    old_level: current_level,
                    new_level,
                });
                blacksmith_ui.phase = BlacksmithPhase::Hammering;
                blacksmith_ui.animation_tick = 0;

                InputResult::NeedsSaveAll
            }
            KeyCode::Esc => {
                blacksmith_ui.phase = BlacksmithPhase::Menu;
                InputResult::Continue
            }
            _ => InputResult::Continue,
        },
        BlacksmithPhase::Hammering => {
            // No input accepted during hammering animation
            InputResult::Continue
        }
        BlacksmithPhase::ResultSuccess | BlacksmithPhase::ResultFailure => {
            // Any key returns to menu
            blacksmith_ui.phase = BlacksmithPhase::Menu;
            InputResult::Continue
        }
    }
}

fn handle_achievement_unlocked(key: KeyEvent, overlay: &mut GameOverlay) -> InputResult {
    // Any key dismisses the achievement modal
    if matches!(key.code, KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ')) {
        *overlay = GameOverlay::None;
    }
    InputResult::Continue
}

fn handle_haven(
    key: KeyEvent,
    state: &mut GameState,
    haven: &mut Haven,
    haven_ui: &mut HavenUiState,
    achievements: &mut crate::achievements::Achievements,
) -> InputResult {
    match haven_ui.confirmation {
        HavenConfirmation::Forge => {
            match key.code {
                KeyCode::Enter => {
                    // Check requirements: Storm Leviathan caught and 25 prestige available
                    let (_has_leviathan, _has_prestige, can_forge) =
                        haven::can_forge_stormbreaker(achievements, state.prestige_rank);

                    if can_forge {
                        // Deduct prestige cost
                        state.prestige_rank -= 25;

                        // Unlock TheStormbreaker achievement
                        achievements.unlock(
                            crate::achievements::AchievementId::TheStormbreaker,
                            Some(state.character_name.clone()),
                        );

                        state.combat_state.add_log_entry(
                            "⚡ You forged the legendary Stormbreaker!".to_string(),
                            false,
                            true,
                        );
                        haven_ui.confirmation = HavenConfirmation::None;
                        return InputResult::NeedsSaveAll;
                    }
                    haven_ui.confirmation = HavenConfirmation::None;
                }
                KeyCode::Esc => {
                    haven_ui.confirmation = HavenConfirmation::None;
                }
                _ => {}
            }
            InputResult::Continue
        }
        HavenConfirmation::Build => {
            match key.code {
                KeyCode::Enter => {
                    let room = haven::HavenRoomId::ALL[haven_ui.selected_room];
                    if let Some((_tier, p_spent)) =
                        haven::try_build_room(room, haven, &mut state.prestige_rank)
                    {
                        // Haven saved via NeedsSaveAll (skipped in debug mode)
                        state.combat_state.add_log_entry(
                            format!(
                                "🏠 Built {} (spent {} Prestige Ranks)",
                                room.name(),
                                p_spent
                            ),
                            false,
                            true,
                        );
                        haven_ui.confirmation = HavenConfirmation::None;
                        return InputResult::NeedsSaveAll;
                    }
                    haven_ui.confirmation = HavenConfirmation::None;
                }
                KeyCode::Esc => {
                    haven_ui.confirmation = HavenConfirmation::None;
                }
                _ => {}
            }
            InputResult::Continue
        }
        HavenConfirmation::None => {
            match key.code {
                KeyCode::Up => {
                    haven_ui.selected_room = haven_ui.selected_room.saturating_sub(1);
                }
                KeyCode::Down => {
                    if haven_ui.selected_room + 1 < haven::HavenRoomId::ALL.len() {
                        haven_ui.selected_room += 1;
                    }
                }
                KeyCode::Enter => {
                    let room = haven::HavenRoomId::ALL[haven_ui.selected_room];

                    // Special handling for Storm Forge - show forge menu if already built
                    if room == haven::HavenRoomId::StormForge && haven.has_storm_forge() {
                        // Only show forge if not already forged
                        if !achievements
                            .is_unlocked(crate::achievements::AchievementId::TheStormbreaker)
                        {
                            haven_ui.confirmation = HavenConfirmation::Forge;
                        }
                    } else if haven.can_build(room)
                        && haven::can_afford(room, haven, state.prestige_rank)
                    {
                        haven_ui.confirmation = HavenConfirmation::Build;
                    }
                }
                KeyCode::Esc => {
                    haven_ui.close();
                }
                _ => {}
            }
            InputResult::Continue
        }
    }
}

fn handle_vault_selection(
    key: KeyEvent,
    state: &mut GameState,
    haven: &Haven,
    overlay: &mut GameOverlay,
) -> InputResult {
    if let GameOverlay::VaultSelection {
        ref mut selected_index,
        ref mut selected_slots,
    } = overlay
    {
        match key.code {
            KeyCode::Up => {
                *selected_index = selected_index.saturating_sub(1);
            }
            KeyCode::Down => {
                if *selected_index < 6 {
                    *selected_index += 1;
                }
            }
            KeyCode::Enter => {
                let slots = [
                    items::EquipmentSlot::Weapon,
                    items::EquipmentSlot::Armor,
                    items::EquipmentSlot::Helmet,
                    items::EquipmentSlot::Gloves,
                    items::EquipmentSlot::Boots,
                    items::EquipmentSlot::Amulet,
                    items::EquipmentSlot::Ring,
                ];
                let slot = slots[*selected_index];
                if state.equipment.get(slot).is_some() {
                    if let Some(pos) = selected_slots.iter().position(|s| *s == slot) {
                        selected_slots.remove(pos);
                    } else if selected_slots.len()
                        < haven.get_bonus(crate::haven::HavenBonusType::VaultSlots) as usize
                    {
                        selected_slots.push(slot);
                    }
                }
            }
            KeyCode::Char(' ') => {
                crate::character::prestige::perform_prestige_with_vault(state, selected_slots);
                *overlay = GameOverlay::None;
                state.combat_state.add_log_entry(
                    format!(
                        "Prestiged to {}! (Vault preserved items)",
                        get_prestige_tier(state.prestige_rank).name
                    ),
                    false,
                    true,
                );
                return InputResult::NeedsSave;
            }
            KeyCode::Esc => {
                *overlay = GameOverlay::None;
            }
            _ => {}
        }
    }
    InputResult::Continue
}

fn handle_prestige_confirm(
    key: KeyEvent,
    state: &mut GameState,
    haven: &Haven,
    overlay: &mut GameOverlay,
) -> InputResult {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if haven.vault_tier() > 0 {
                *overlay = GameOverlay::VaultSelection {
                    selected_index: 0,
                    selected_slots: Vec::new(),
                };
            } else {
                perform_prestige(state);
                *overlay = GameOverlay::None;
                state.combat_state.add_log_entry(
                    format!(
                        "Prestiged to {}!",
                        get_prestige_tier(state.prestige_rank).name
                    ),
                    false,
                    true,
                );
                return InputResult::NeedsSave;
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            *overlay = GameOverlay::None;
        }
        _ => {}
    }
    InputResult::Continue
}

fn handle_debug_menu(
    key: KeyEvent,
    state: &mut GameState,
    haven: &mut Haven,
    enhancement: &mut enhancement::EnhancementProgress,
    overlay: &mut GameOverlay,
    debug_menu: &mut DebugMenu,
) -> InputResult {
    match key.code {
        KeyCode::Up => debug_menu.navigate_up(),
        KeyCode::Down => debug_menu.navigate_down(),
        KeyCode::Enter => {
            let msg = debug_menu.trigger_selected(state, haven, enhancement);
            state
                .combat_state
                .add_log_entry(format!("[DEBUG] {}", msg), false, true);
            // Show discovery modals (no save in debug mode)
            if msg == "Haven discovered!" {
                *overlay = GameOverlay::HavenDiscovery;
            } else if msg == "Blacksmith discovered!" {
                *overlay = GameOverlay::BlacksmithDiscovery;
            }
        }
        KeyCode::Esc => debug_menu.close(),
        _ => {}
    }
    InputResult::Continue
}

fn handle_minigame(key: KeyEvent, state: &mut GameState) -> InputResult {
    if let Some(ref mut minigame) = state.active_minigame {
        match minigame {
            ActiveMinigame::Rune(rune_game) => {
                if rune_game.game_result.is_some() {
                    state.last_minigame_win = apply_rune_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Left => RuneInput::Left,
                    KeyCode::Right => RuneInput::Right,
                    KeyCode::Up => RuneInput::Up,
                    KeyCode::Down => RuneInput::Down,
                    KeyCode::Enter => RuneInput::Submit,
                    KeyCode::Char('f') | KeyCode::Char('F') => RuneInput::ClearGuess,
                    KeyCode::Esc => RuneInput::Forfeit,
                    _ => RuneInput::Other,
                };
                let mut rng = rand::rng();
                process_rune_input(rune_game, input, &mut rng);
            }
            ActiveMinigame::Minesweeper(minesweeper_game) => {
                if minesweeper_game.game_result.is_some() {
                    state.last_minigame_win = apply_minesweeper_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Up => MinesweeperInput::Up,
                    KeyCode::Down => MinesweeperInput::Down,
                    KeyCode::Left => MinesweeperInput::Left,
                    KeyCode::Right => MinesweeperInput::Right,
                    KeyCode::Enter => MinesweeperInput::Reveal,
                    KeyCode::Char('f') | KeyCode::Char('F') => MinesweeperInput::ToggleFlag,
                    KeyCode::Esc => MinesweeperInput::Forfeit,
                    _ => MinesweeperInput::Other,
                };
                let mut rng = rand::rng();
                process_minesweeper_input(minesweeper_game, input, &mut rng);
            }
            ActiveMinigame::Gomoku(gomoku_game) => {
                if gomoku_game.game_result.is_some() {
                    state.last_minigame_win = apply_gomoku_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Up => GomokuInput::Up,
                    KeyCode::Down => GomokuInput::Down,
                    KeyCode::Left => GomokuInput::Left,
                    KeyCode::Right => GomokuInput::Right,
                    KeyCode::Enter => GomokuInput::PlaceStone,
                    KeyCode::Esc => GomokuInput::Forfeit,
                    _ => GomokuInput::Other,
                };
                process_gomoku_input(gomoku_game, input);
            }
            ActiveMinigame::Chess(chess_game) => {
                if chess_game.game_result.is_some() {
                    state.last_minigame_win = apply_chess_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Up => ChessInput::Up,
                    KeyCode::Down => ChessInput::Down,
                    KeyCode::Left => ChessInput::Left,
                    KeyCode::Right => ChessInput::Right,
                    KeyCode::Enter => ChessInput::Select,
                    KeyCode::Esc => ChessInput::Forfeit,
                    _ => ChessInput::Other,
                };
                process_chess_input(chess_game, input);
            }
            ActiveMinigame::Morris(morris_game) => {
                if morris_game.game_result.is_some() {
                    state.last_minigame_win = apply_morris_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Up => MorrisInput::Up,
                    KeyCode::Down => MorrisInput::Down,
                    KeyCode::Left => MorrisInput::Left,
                    KeyCode::Right => MorrisInput::Right,
                    KeyCode::Enter => MorrisInput::Select,
                    KeyCode::Esc => MorrisInput::Forfeit,
                    _ => MorrisInput::Other,
                };
                process_morris_input(morris_game, input);
            }
            ActiveMinigame::Go(go_game) => {
                if go_game.game_result.is_some() {
                    state.last_minigame_win = apply_go_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Up => GoInput::Up,
                    KeyCode::Down => GoInput::Down,
                    KeyCode::Left => GoInput::Left,
                    KeyCode::Right => GoInput::Right,
                    KeyCode::Enter => GoInput::PlaceStone,
                    KeyCode::Char('p') | KeyCode::Char('P') => GoInput::Pass,
                    KeyCode::Esc => GoInput::Forfeit,
                    _ => GoInput::Other,
                };
                process_go_input(go_game, input);
            }
            ActiveMinigame::FlappyBird(flappy_game) => {
                if flappy_game.game_result.is_some() {
                    state.last_minigame_win = apply_flappy_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Char(' ') | KeyCode::Up => FlappyBirdInput::Flap,
                    KeyCode::Esc => FlappyBirdInput::Forfeit,
                    _ => FlappyBirdInput::Other,
                };
                process_flappy_input(flappy_game, input);
            }
            ActiveMinigame::Jezzball(jezzball_game) => {
                if jezzball_game.game_result.is_some() {
                    state.last_minigame_win = apply_jezzball_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Up => JezzballInput::Up,
                    KeyCode::Down => JezzballInput::Down,
                    KeyCode::Left => JezzballInput::Left,
                    KeyCode::Right => JezzballInput::Right,
                    KeyCode::Enter | KeyCode::Char(' ') => JezzballInput::Select,
                    KeyCode::Char('x') | KeyCode::Char('X') => JezzballInput::ToggleOrientation,
                    KeyCode::Esc => JezzballInput::Forfeit,
                    _ => JezzballInput::Other,
                };
                process_jezzball_input(jezzball_game, input);
            }
            ActiveMinigame::Snake(snake_game) => {
                if snake_game.game_result.is_some() {
                    state.last_minigame_win = apply_snake_result(state);
                    return InputResult::Continue;
                }
                let input = match key.code {
                    KeyCode::Up => SnakeInput::Up,
                    KeyCode::Down => SnakeInput::Down,
                    KeyCode::Left => SnakeInput::Left,
                    KeyCode::Right => SnakeInput::Right,
                    KeyCode::Char(' ') => SnakeInput::Select,
                    KeyCode::Esc => SnakeInput::Forfeit,
                    _ => SnakeInput::Other,
                };
                process_snake_input(snake_game, input);
            }
        }
    }
    InputResult::Continue
}

fn handle_challenge_menu(key: KeyEvent, state: &mut GameState) -> InputResult {
    let input = match key.code {
        KeyCode::Up => MenuInput::Up,
        KeyCode::Down => MenuInput::Down,
        KeyCode::Enter => MenuInput::Select,
        KeyCode::Char('d') | KeyCode::Char('D') => MenuInput::Decline,
        KeyCode::Esc | KeyCode::Tab => MenuInput::Cancel,
        _ => MenuInput::Other,
    };
    process_menu_input(state, input);
    InputResult::Continue
}

#[allow(clippy::too_many_arguments)]
fn handle_base_game(
    key: KeyEvent,
    state: &mut GameState,
    haven: &Haven,
    haven_ui: &mut HavenUiState,
    blacksmith_ui: &mut BlacksmithUiState,
    enhancement: &enhancement::EnhancementProgress,
    overlay: &mut GameOverlay,
    achievements: &mut crate::achievements::Achievements,
    update_available: bool,
    update_expanded: bool,
) -> InputResult {
    match key.code {
        KeyCode::Esc => InputResult::QuitToSelect,
        KeyCode::Char('u') | KeyCode::Char('U') => {
            // Toggle update details if update available OR already expanded
            if update_available || update_expanded {
                InputResult::ToggleUpdateDetails
            } else {
                InputResult::Continue
            }
        }
        KeyCode::Char('p') | KeyCode::Char('P') => {
            if can_prestige(state) {
                *overlay = GameOverlay::PrestigeConfirm;
            }
            InputResult::Continue
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            if haven.discovered {
                haven_ui.open();
            }
            InputResult::Continue
        }
        KeyCode::Char('b') | KeyCode::Char('B') => {
            if enhancement.discovered {
                blacksmith_ui.open();
            }
            InputResult::Continue
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            // Clear pending notifications when opening achievements
            achievements.clear_pending_notifications();
            *overlay = GameOverlay::Achievements {
                browser: crate::ui::achievement_browser_scene::AchievementBrowserState::new(),
            };
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}
