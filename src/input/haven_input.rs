//! Haven overlay input handling.

use super::types::{HavenConfirmation, HavenUiState, InputResult};
use crate::core::game_state::GameState;
use crate::haven;
use crate::haven::Haven;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub(super) fn handle_haven(
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
