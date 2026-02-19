//! Input handling for the Game screen.
//!
//! Extracts the input dispatch logic from main.rs into a clean priority chain.

mod minigame_input;
pub mod types;

// Re-export all types for backward compatibility
pub use types::*;

// Re-export soulforge UI types from enhancement module
pub use crate::enhancement::{EnhancementResult, SoulforgePhase, SoulforgeUiState};

use crate::achievements::get_achievements_by_category;
use crate::challenges::menu::{process_input as process_menu_input, MenuInput};
use crate::character::prestige::{can_prestige, get_prestige_tier, perform_prestige};
use crate::core::game_state::GameState;
use crate::enhancement;
use crate::haven;
use crate::haven::Haven;
use crate::items;
use crate::utils::debug_menu::DebugMenu;
use minigame_input::handle_minigame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Main dispatcher for Game screen input. Handles the priority chain.
#[allow(clippy::too_many_arguments)]
pub fn handle_game_input(
    key: KeyEvent,
    state: &mut GameState,
    haven: &mut Haven,
    haven_ui: &mut HavenUiState,
    soulforge_ui: &mut SoulforgeUiState,
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
            KeyCode::Down => {
                let count = get_achievements_by_category(browser.selected_category).len();
                browser.move_down(count);
            }
            _ => {}
        }
        return InputResult::Continue;
    }

    // 0.75. Help overlay
    if matches!(overlay, GameOverlay::Help) {
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('?')) {
            *overlay = GameOverlay::None;
        }
        return InputResult::Continue;
    }

    // 1. Haven discovery modal (blocks all other input)
    if matches!(overlay, GameOverlay::HavenDiscovery) {
        return handle_haven_discovery(key, overlay);
    }

    // 1a. Soulforge discovery modal (blocks all other input)
    if matches!(overlay, GameOverlay::SoulforgeDiscovery) {
        return handle_soulforge_discovery(key, overlay);
    }

    // 1b. Achievement unlocked modal (blocks all other input)
    if matches!(overlay, GameOverlay::AchievementUnlocked { .. }) {
        return handle_achievement_unlocked(key, overlay);
    }

    // 2. Haven screen (blocks other input when open)
    if haven_ui.showing {
        return handle_haven(key, state, haven, haven_ui, achievements);
    }

    // 2.5. Soulforge overlay
    if soulforge_ui.open {
        return handle_soulforge(key, soulforge_ui, enhancement, state.prestige_rank);
    }

    // 3. Vault item selection
    if matches!(overlay, GameOverlay::VaultSelection { .. }) {
        return handle_vault_selection(key, state, haven, overlay);
    }

    // 4. Prestige confirmation
    if matches!(overlay, GameOverlay::PrestigeConfirm) {
        return handle_prestige_confirm(key, state, haven, overlay);
    }

    // 4.5. Quit confirmation (pending challenges warning)
    if matches!(overlay, GameOverlay::QuitConfirm) {
        match key.code {
            KeyCode::Enter => return InputResult::QuitToSelect,
            _ => {
                *overlay = GameOverlay::None;
                return InputResult::Continue;
            }
        }
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
        soulforge_ui,
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

fn handle_soulforge_discovery(key: KeyEvent, overlay: &mut GameOverlay) -> InputResult {
    if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
        *overlay = GameOverlay::None;
    }
    InputResult::Continue
}

fn handle_soulforge(
    key: KeyEvent,
    soulforge_ui: &mut SoulforgeUiState,
    enhancement: &enhancement::EnhancementProgress,
    prestige_rank: u32,
) -> InputResult {
    match soulforge_ui.phase {
        SoulforgePhase::Menu => match key.code {
            KeyCode::Up => {
                soulforge_ui.selected_slot = soulforge_ui.selected_slot.saturating_sub(1);
                InputResult::Continue
            }
            KeyCode::Down => {
                if soulforge_ui.selected_slot < 6 {
                    soulforge_ui.selected_slot += 1;
                }
                InputResult::Continue
            }
            KeyCode::Enter => {
                let slot_index = soulforge_ui.selected_slot;
                let current_level = enhancement.level(slot_index);

                // Check: level < max, can afford (slot enhancement is independent of equipped item)
                if current_level < enhancement::MAX_ENHANCEMENT_LEVEL {
                    let target_level = current_level + 1;
                    let cost = enhancement::enhancement_cost(target_level);
                    if prestige_rank >= cost {
                        soulforge_ui.phase = SoulforgePhase::Confirming;
                    }
                }
                InputResult::Continue
            }
            KeyCode::Esc => {
                soulforge_ui.close();
                InputResult::Continue
            }
            _ => InputResult::Continue,
        },
        SoulforgePhase::Confirming => match key.code {
            KeyCode::Enter => {
                let slot_index = soulforge_ui.selected_slot;
                let current_level = enhancement.level(slot_index);
                let target_level = current_level + 1;
                let cost = enhancement::enhancement_cost(target_level);

                // Roll the outcome (applied after animation completes in main loop)
                let mut rng = rand::rng();
                let (success, new_level) = enhancement::roll_enhancement(current_level, &mut rng);

                soulforge_ui.last_result = Some(EnhancementResult {
                    slot_index,
                    success,
                    old_level: current_level,
                    new_level,
                    cost,
                });
                soulforge_ui.phase = SoulforgePhase::Hammering;
                soulforge_ui.animation_tick = 0;

                InputResult::Continue
            }
            KeyCode::Esc => {
                soulforge_ui.phase = SoulforgePhase::Menu;
                InputResult::Continue
            }
            _ => InputResult::Continue,
        },
        SoulforgePhase::Hammering => {
            // No input accepted during hammering animation
            InputResult::Continue
        }
        SoulforgePhase::ResultSuccess | SoulforgePhase::ResultFailure => {
            // Any key returns to menu
            soulforge_ui.phase = SoulforgePhase::Menu;
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
                            "\u{26a1} You forged the legendary Stormbreaker!".to_string(),
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
                        // Check Haven tier achievements after upgrade
                        achievements.sync_from_haven(
                            haven.discovered,
                            &haven.rooms,
                            Some(&state.character_name),
                        );
                        // Haven saved via NeedsSaveAll (skipped in debug mode)
                        state.combat_state.add_log_entry(
                            format!(
                                "\u{1f3e0} Built {} (spent {} Prestige Ranks)",
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
        ref mut confirm_pending,
    } = overlay
    {
        let vault_slots = haven.get_bonus(crate::haven::HavenBonusType::VaultSlots) as usize;

        match key.code {
            KeyCode::Up => {
                *selected_index = selected_index.saturating_sub(1);
                *confirm_pending = false;
            }
            KeyCode::Down => {
                if *selected_index < 6 {
                    *selected_index += 1;
                }
                *confirm_pending = false;
            }
            KeyCode::Char(' ') => {
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
                    } else if selected_slots.len() < vault_slots {
                        selected_slots.push(slot);
                    }
                }
                *confirm_pending = false;
            }
            KeyCode::Enter => {
                // At max selections: prestige immediately.
                // Under max: require a second Enter to confirm.
                if selected_slots.len() >= vault_slots || *confirm_pending {
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
                } else {
                    *confirm_pending = true;
                }
            }
            KeyCode::Esc => {
                if *confirm_pending {
                    *confirm_pending = false;
                } else {
                    *overlay = GameOverlay::None;
                }
            }
            _ => {
                *confirm_pending = false;
            }
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
                    confirm_pending: false,
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
            } else if msg == "Soulforge discovered!" {
                *overlay = GameOverlay::SoulforgeDiscovery;
            }
        }
        KeyCode::Esc => debug_menu.close(),
        _ => {}
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
    soulforge_ui: &mut SoulforgeUiState,
    enhancement: &enhancement::EnhancementProgress,
    overlay: &mut GameOverlay,
    achievements: &mut crate::achievements::Achievements,
    update_available: bool,
    update_expanded: bool,
) -> InputResult {
    match key.code {
        KeyCode::Esc => {
            if state.challenge_menu.challenges.is_empty() {
                InputResult::QuitToSelect
            } else {
                *overlay = GameOverlay::QuitConfirm;
                InputResult::Continue
            }
        }
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
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if enhancement.discovered {
                soulforge_ui.open();
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
        KeyCode::Char('?') => {
            *overlay = GameOverlay::Help;
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}
