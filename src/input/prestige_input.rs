//! Prestige confirmation and vault selection input handling.

use super::types::{GameOverlay, InputResult};
use crate::character::prestige::{get_prestige_tier, perform_prestige};
use crate::core::game_state::GameState;
use crate::deep::DeepState;
use crate::haven::Haven;
use crate::items;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(super) fn handle_vault_selection(
    key: KeyEvent,
    state: &mut GameState,
    haven: &mut Haven,
    deep: &mut DeepState,
    deep_ui: &mut crate::deep::DeepUiState,
    overlay: &mut GameOverlay,
    achievements: &crate::achievements::Achievements,
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
                    // Remember selections for next time
                    haven.last_vault_selections = selected_slots.clone();
                    // Capture farewell data before prestige reset
                    deep_ui.farewell_mercs = deep
                        .prestige
                        .roster
                        .iter()
                        .map(|m| (m.name.clone(), m.level, m.missions_completed))
                        .collect();
                    crate::character::prestige::perform_prestige_with_vault(state, selected_slots);
                    // Reset prestige-scoped Deep state while preserving guild rank,
                    // layer progression, and infrastructure.
                    deep.on_prestige();
                    // Re-sync fracture zone unlocks after prestige reset
                    crate::zones::sync_account_zone_unlocks(
                        &mut state.zone_progression,
                        achievements.is_unlocked(crate::achievements::AchievementId::StormsEnd),
                        deep.persistent.fracture_zone_cap,
                        state.prestige_rank,
                    );
                    *overlay = GameOverlay::None;
                    let new_rank = state.prestige_rank;
                    state.combat_state.add_log_entry(
                        format!(
                            "Prestiged to {}! (Vault preserved items)",
                            get_prestige_tier(new_rank).name
                        ),
                        false,
                        true,
                    );
                    return InputResult::NeedsSaveWithEvent(
                        crate::history::SaveEvent::PrestigeRank(new_rank),
                    );
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
    deep: &mut DeepState,
    deep_ui: &mut crate::deep::DeepUiState,
    overlay: &mut GameOverlay,
    achievements: &crate::achievements::Achievements,
) -> InputResult {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if haven.vault_tier() > 0 {
                // Pre-populate with last vault selections (filtered to slots with items,
                // truncated to current vault capacity)
                let vault_slots =
                    haven.get_bonus(crate::haven::HavenBonusType::VaultSlots) as usize;
                let pre_selected: Vec<items::EquipmentSlot> = haven
                    .last_vault_selections
                    .iter()
                    .filter(|slot| state.equipment.get(**slot).is_some())
                    .take(vault_slots)
                    .copied()
                    .collect();

                *overlay = GameOverlay::VaultSelection {
                    selected_index: 0,
                    selected_slots: pre_selected,
                    confirm_pending: false,
                };
            } else {
                // Capture farewell data before prestige reset
                deep_ui.farewell_mercs = deep
                    .prestige
                    .roster
                    .iter()
                    .map(|m| (m.name.clone(), m.level, m.missions_completed))
                    .collect();
                perform_prestige(state);
                // Reset prestige-scoped Deep state while preserving guild rank,
                // layer progression, and infrastructure.
                deep.on_prestige();
                // Re-sync fracture zone unlocks after prestige reset
                crate::zones::sync_account_zone_unlocks(
                    &mut state.zone_progression,
                    achievements.is_unlocked(crate::achievements::AchievementId::StormsEnd),
                    deep.persistent.fracture_zone_cap,
                    state.prestige_rank,
                );
                *overlay = GameOverlay::None;
                let new_rank = state.prestige_rank;
                state.combat_state.add_log_entry(
                    format!("Prestiged to {}!", get_prestige_tier(new_rank).name),
                    false,
                    true,
                );
                return InputResult::NeedsSaveWithEvent(crate::history::SaveEvent::PrestigeRank(
                    new_rank,
                ));
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            *overlay = GameOverlay::None;
        }
        _ => {}
    }
    InputResult::Continue
}
