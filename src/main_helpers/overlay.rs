//! Game overlay rendering.

use crate::achievements;
use crate::core::game_state::GameState;
use crate::enhancement;
use crate::haven;
use crate::input::{GameOverlay, HavenUiState, SoulforgeUiState};
use crate::stormglass::types::{ChronoSurgeState, ChronoSurgeSummary, ExchangeUiState};
use crate::ui;
use crate::utils;
use std::time::{Duration, Instant};

/// Draws the quit confirmation dialog when pending challenges exist.
fn draw_quit_confirm(frame: &mut ratatui::Frame, pending_count: usize) {
    use ratatui::{
        layout::{Alignment, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Clear, Paragraph},
    };

    let size = frame.area();
    let w = 42.min(size.width.saturating_sub(4));
    let h = 8.min(size.height.saturating_sub(4));
    let x = (size.width.saturating_sub(w)) / 2;
    let y = (size.height.saturating_sub(h)) / 2;
    let area = Rect::new(x, y, w, h);

    frame.render_widget(Clear, area);

    let challenge_word = if pending_count == 1 {
        "challenge"
    } else {
        "challenges"
    };

    let lines = vec![
        Line::from(""),
        Line::from(format!(
            "  {} pending {} will be lost.",
            pending_count, challenge_word
        )),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "[Enter] Leave",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled(
                "[Esc] Stay",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(
                Line::from(Span::styled(
                    " Unsaved Challenges ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(paragraph, area);
}

/// Draw all game overlays on top of the main game UI.
#[allow(clippy::too_many_arguments)]
pub fn draw_game_overlays(
    frame: &mut ratatui::Frame,
    state: &GameState,
    overlay: &GameOverlay,
    haven: &haven::Haven,
    haven_ui: &HavenUiState,
    soulforge_ui: &SoulforgeUiState,
    exchange_ui: &ExchangeUiState,
    enhancement: &enhancement::EnhancementProgress,
    global_achievements: &achievements::Achievements,
    debug_mode: bool,
    debug_menu: &utils::debug_menu::DebugMenu,
    last_save_instant: Option<Instant>,
    last_save_time: Option<chrono::DateTime<chrono::Local>>,
    chrono_surge: Option<&ChronoSurgeState>,
    chrono_summary: Option<&ChronoSurgeSummary>,
    ctx: &ui::responsive::LayoutContext,
) {
    let area = frame.area();
    match overlay {
        GameOverlay::OfflineWelcome { report } => {
            ui::game_common::render_offline_welcome(frame, area, report, ctx);
        }
        GameOverlay::PrestigeConfirm => {
            ui::prestige_confirm::draw_prestige_confirm(frame, state, ctx);
        }
        GameOverlay::HavenDiscovery => {
            ui::haven_scene::render_haven_discovery_modal(frame, area, ctx);
        }
        GameOverlay::SoulforgeDiscovery => {
            ui::soulforge_scene::render_soulforge_discovery_modal(frame, area, ctx);
        }
        GameOverlay::StormglassDiscovery => {
            ui::stormglass_scene::render_stormglass_discovery_modal(frame, area, ctx);
        }
        GameOverlay::AchievementUnlocked { ref achievements } => {
            ui::achievement_browser_scene::render_achievement_unlocked_modal(
                frame,
                area,
                achievements,
                ctx,
            );
        }
        GameOverlay::VaultSelection {
            selected_index,
            ref selected_slots,
            confirm_pending,
        } => {
            ui::haven_scene::render_vault_selection(
                frame,
                area,
                state,
                haven.get_bonus(haven::HavenBonusType::VaultSlots) as u8,
                *selected_index,
                selected_slots,
                &enhancement.levels,
                *confirm_pending,
                ctx,
            );
        }
        GameOverlay::Achievements {
            browser,
            title_browser,
        } => {
            if title_browser.showing {
                ui::title_browser_scene::render_title_browser(
                    frame,
                    area,
                    global_achievements,
                    title_browser,
                    &state.character_name,
                );
            } else {
                ui::achievement_browser_scene::render_achievement_browser(
                    frame,
                    area,
                    global_achievements,
                    browser,
                    enhancement,
                    ctx,
                );
            }
        }
        GameOverlay::LeviathanEncounter { encounter_number } => {
            ui::fishing_scene::render_leviathan_encounter_modal(
                frame,
                area,
                *encounter_number,
                ctx,
            );
        }
        GameOverlay::QuitConfirm => {
            draw_quit_confirm(frame, state.challenge_menu.challenges.len());
        }
        GameOverlay::Help => {
            ui::help_overlay::draw_help_overlay(frame);
        }
        GameOverlay::BugReport {
            ref summary,
            clipboard_ready,
            ref error,
        } => {
            ui::bug_report_scene::draw_bug_report_overlay(frame, summary, *clipboard_ready, error);
        }
        GameOverlay::Timeline { ref browser } => {
            ui::timeline_scene::draw_timeline_browser(frame, area, browser);
        }
        GameOverlay::None => {}
    }

    // Haven screen overlay
    if haven_ui.showing {
        ui::haven_scene::render_haven_tree(
            frame,
            area,
            haven,
            haven_ui.selected_room,
            haven_ui.open_elapsed_ms(),
            state.prestige_rank,
            global_achievements,
            ctx,
        );
        match haven_ui.confirmation {
            crate::input::HavenConfirmation::Build => {
                let room = haven::HavenRoomId::ALL[haven_ui.selected_room];
                ui::haven_scene::render_build_confirmation(
                    frame,
                    area,
                    room,
                    haven,
                    state.prestige_rank,
                    ctx,
                );
            }
            crate::input::HavenConfirmation::Forge => {
                ui::haven_scene::render_forge_confirmation(
                    frame,
                    area,
                    global_achievements,
                    state.prestige_rank,
                    ctx,
                );
            }
            crate::input::HavenConfirmation::None => {}
        }
    }

    // Soulforge overlay
    if soulforge_ui.open {
        ui::soulforge_scene::render_soulforge(
            frame,
            area,
            soulforge_ui,
            enhancement,
            state.prestige_rank,
            ctx,
        );
    }

    // Stormglass Exchange overlay
    if exchange_ui.open {
        ui::stormglass_scene::render_stormglass_exchange(frame, area, exchange_ui, state, ctx);
    }

    // Chrono Surge active: status banner at bottom
    if let Some(surge) = chrono_surge {
        ui::stormglass_scene::render_chrono_surge_time_warp(frame, area, surge, ctx);
        ui::stormglass_scene::render_chrono_surge_banner(frame, area, surge, ctx);
    }

    // Chrono Surge summary modal
    if let Some(summary) = chrono_summary {
        ui::stormglass_scene::render_chrono_surge_summary(frame, area, summary, ctx);
    }

    // Debug indicator / save indicator
    if debug_mode {
        ui::debug_menu_scene::render_debug_indicator(frame, area, ctx);
        if debug_menu.is_open {
            ui::debug_menu_scene::render_debug_menu(
                frame,
                area,
                debug_menu,
                global_achievements.ui_border_style,
                ctx,
            );
        }
    } else {
        let is_saving = last_save_instant
            .map(|t| t.elapsed() < Duration::from_secs(1))
            .unwrap_or(false);
        ui::debug_menu_scene::render_save_indicator(frame, area, is_saving, last_save_time, ctx);
    }
}
