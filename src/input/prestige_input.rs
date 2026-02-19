//! Prestige confirmation and vault selection input handling.

use super::types::{GameOverlay, InputResult};
use crate::character::prestige::{get_prestige_tier, perform_prestige};
use crate::core::game_state::GameState;
use crate::haven::Haven;
use crate::items;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(super) fn handle_vault_selection(
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

pub(super) fn handle_prestige_confirm(
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
