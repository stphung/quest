//! Soulforge overlay input handling.

use super::types::InputResult;
use crate::enhancement::{
    self, enhancement_cost, roll_enhancement, EnhancementResult, SoulforgePhase, SoulforgeUiState,
    MAX_ENHANCEMENT_LEVEL,
};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(super) fn handle_soulforge(
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
                if current_level < MAX_ENHANCEMENT_LEVEL {
                    let target_level = current_level + 1;
                    let cost = enhancement_cost(target_level);
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
                let cost = enhancement_cost(target_level);

                // Roll the outcome (applied after animation completes in main loop)
                let mut rng = rand::rng();
                let (success, new_level) = roll_enhancement(current_level, &mut rng);

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
