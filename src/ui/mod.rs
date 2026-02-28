pub mod achievement_browser_scene;
mod achievement_details;
mod achievement_list;
mod achievement_tabs;
pub(crate) mod ascension_scene;
pub mod bug_report_scene;
pub mod challenge_menu_scene;
pub mod character_creation;
pub mod character_delete;
pub mod character_rename;
pub mod character_select;
pub mod chess_scene;
mod combat_3d;
pub mod combat_effects;
pub(crate) mod combat_scene;
pub mod debug_menu_scene;
mod deep_events;
mod deep_layers;
mod deep_missions;
mod deep_results;
mod deep_roster;
pub mod deep_scene;
mod deep_shared;
pub mod dungeon_map;
mod enemy_sprite_data;
mod enemy_sprites;
pub mod fishing_scene;
pub mod flappy_scene;
pub mod game_common;
pub mod go_scene;
pub mod gomoku_scene;
mod haven_details;
pub mod haven_scene;
mod haven_tree;
pub mod jezzball_scene;
pub mod minesweeper_scene;
pub mod morris_scene;
pub mod overlay_layout;
pub mod prestige_confirm;
pub mod responsive;
pub mod rune_scene;
pub mod runic_shift_scene;
mod scene_fx;
pub mod snake_scene;
mod soulforge_effects;
pub mod soulforge_scene;
mod soulforge_slots;
mod stats_attributes;
mod stats_equipment;
mod stats_panel;
pub(crate) mod stats_prestige;
mod stats_sigils;
pub mod stormglass_scene;
pub(crate) mod throbber;
pub mod ticker;
pub mod time_vault_scene;
pub mod title_browser_scene;
mod zone_bg;

use crate::challenges::ActiveMinigame;
use crate::core::game_state::GameState;
use crate::utils::updater::UpdateInfo;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
    Frame,
};
use std::sync::atomic::{AtomicU8, Ordering};

use crate::items::types::Rarity;
use responsive::{render_too_small, LayoutContext, SizeTier};

static GLOBAL_UI_BORDER_STYLE: AtomicU8 = AtomicU8::new(0);

fn set_global_ui_border_style(style: crate::achievements::UiBorderStyle) {
    GLOBAL_UI_BORDER_STYLE.store(style.storage_id(), Ordering::Relaxed);
}

pub(super) fn current_ui_border_style() -> crate::achievements::UiBorderStyle {
    crate::achievements::UiBorderStyle::from_storage_id(
        GLOBAL_UI_BORDER_STYLE.load(Ordering::Relaxed),
    )
}

pub(super) fn border_type_for_style(style: crate::achievements::UiBorderStyle) -> BorderType {
    match style {
        crate::achievements::UiBorderStyle::Classic => BorderType::Plain,
        crate::achievements::UiBorderStyle::Rounded => BorderType::Rounded,
        crate::achievements::UiBorderStyle::Double => BorderType::Double,
        crate::achievements::UiBorderStyle::Thick => BorderType::Thick,
        crate::achievements::UiBorderStyle::Dashed => BorderType::LightDoubleDashed,
        crate::achievements::UiBorderStyle::HeavyDashed => BorderType::HeavyDoubleDashed,
        crate::achievements::UiBorderStyle::TripleDashed => BorderType::LightTripleDashed,
        crate::achievements::UiBorderStyle::HeavyTripleDashed => BorderType::HeavyTripleDashed,
        crate::achievements::UiBorderStyle::QuadDashed => BorderType::LightQuadrupleDashed,
        crate::achievements::UiBorderStyle::HeavyQuadDashed => BorderType::HeavyQuadrupleDashed,
        crate::achievements::UiBorderStyle::HeavyCorner => BorderType::Plain,
        crate::achievements::UiBorderStyle::MicroDash => BorderType::Plain,
        crate::achievements::UiBorderStyle::HeaderRail => BorderType::Plain,
    }
}

pub(super) fn themed_block<'a>(block: Block<'a>) -> Block<'a> {
    block.border_type(border_type_for_style(current_ui_border_style()))
}

pub(super) fn render_themed_block<'a>(
    frame: &mut Frame,
    area: Rect,
    block: Block<'a>,
    border_color: Color,
    ctx: BorderFxContext,
) -> Rect {
    let block = themed_block(block);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    apply_border_fx_for_style(frame, area, border_color, current_ui_border_style(), ctx);
    inner
}

pub(super) fn themed_border_color(base: Color) -> Color {
    base
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct BorderFxContext;

pub(super) fn apply_themed_border_fx(
    frame: &mut Frame,
    area: Rect,
    border_color: Color,
    ctx: BorderFxContext,
) {
    apply_border_fx_for_style(frame, area, border_color, current_ui_border_style(), ctx);
}

pub(super) fn apply_border_fx_for_style(
    frame: &mut Frame,
    area: Rect,
    border_color: Color,
    style: crate::achievements::UiBorderStyle,
    _ctx: BorderFxContext,
) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    match style {
        crate::achievements::UiBorderStyle::HeavyCorner => {
            apply_heavy_corner_overlay(frame, area, border_color)
        }
        crate::achievements::UiBorderStyle::MicroDash => {
            apply_micro_dash_overlay(frame, area, border_color)
        }
        crate::achievements::UiBorderStyle::HeaderRail => {
            apply_header_rail_overlay(frame, area, border_color)
        }
        _ => {}
    }
}

fn apply_heavy_corner_overlay(frame: &mut Frame, area: Rect, border_color: Color) {
    let left = area.x;
    let top = area.y;
    let right = area.x + area.width - 1;
    let bottom = area.y + area.height - 1;

    set_border_cell_symbol(frame, left, top, "┏", border_color);
    set_border_cell_symbol(frame, right, top, "┓", border_color);
    set_border_cell_symbol(frame, left, bottom, "┗", border_color);
    set_border_cell_symbol(frame, right, bottom, "┛", border_color);
}

fn apply_micro_dash_overlay(frame: &mut Frame, area: Rect, border_color: Color) {
    let left = area.x;
    let top = area.y;
    let right = area.x + area.width - 1;
    let bottom = area.y + area.height - 1;

    for x in (left + 1)..right {
        if let Some(cell) = frame.buffer_mut().cell_mut((x, top)) {
            if is_horizontal_border_symbol(cell.symbol()) {
                cell.set_symbol("┄")
                    .set_style(Style::default().fg(border_color));
            }
        }
        if let Some(cell) = frame.buffer_mut().cell_mut((x, bottom)) {
            if is_horizontal_border_symbol(cell.symbol()) {
                cell.set_symbol("┄")
                    .set_style(Style::default().fg(border_color));
            }
        }
    }

    for y in (top + 1)..bottom {
        if let Some(cell) = frame.buffer_mut().cell_mut((left, y)) {
            if is_vertical_border_symbol(cell.symbol()) {
                cell.set_symbol("┆")
                    .set_style(Style::default().fg(border_color));
            }
        }
        if let Some(cell) = frame.buffer_mut().cell_mut((right, y)) {
            if is_vertical_border_symbol(cell.symbol()) {
                cell.set_symbol("┆")
                    .set_style(Style::default().fg(border_color));
            }
        }
    }
}

fn apply_header_rail_overlay(frame: &mut Frame, area: Rect, border_color: Color) {
    let left = area.x;
    let top = area.y;
    let right = area.x + area.width - 1;

    set_border_cell_symbol(frame, left, top, "┏", border_color);
    set_border_cell_symbol(frame, right, top, "┓", border_color);

    for x in (left + 1)..right {
        if let Some(cell) = frame.buffer_mut().cell_mut((x, top)) {
            if is_horizontal_border_symbol(cell.symbol()) {
                cell.set_symbol("━")
                    .set_style(Style::default().fg(border_color));
            }
        }
    }
}

fn set_border_cell_symbol(frame: &mut Frame, x: u16, y: u16, symbol: &'static str, color: Color) {
    if let Some(cell) = frame.buffer_mut().cell_mut((x, y)) {
        cell.set_symbol(symbol)
            .set_style(Style::default().fg(color));
    }
}

fn is_horizontal_border_symbol(symbol: &str) -> bool {
    matches!(symbol, "─" | "━" | "═" | "┄" | "┅" | "┈" | "┉" | "▀")
}

fn is_vertical_border_symbol(symbol: &str) -> bool {
    matches!(symbol, "│" | "┃" | "║" | "┆" | "┇" | "┊" | "┋" | "▌")
}

#[derive(Clone, Copy)]
pub(super) struct PanelBorderChars {
    pub tl: char,
    pub tr: char,
    pub bl: char,
    pub br: char,
    pub h: char,
    pub v: char,
}

pub(super) fn panel_border_chars() -> PanelBorderChars {
    match current_ui_border_style() {
        crate::achievements::UiBorderStyle::Rounded => PanelBorderChars {
            tl: '╭',
            tr: '╮',
            bl: '╰',
            br: '╯',
            h: '─',
            v: '│',
        },
        crate::achievements::UiBorderStyle::Double => PanelBorderChars {
            tl: '╔',
            tr: '╗',
            bl: '╚',
            br: '╝',
            h: '═',
            v: '║',
        },
        crate::achievements::UiBorderStyle::Thick => PanelBorderChars {
            tl: '┏',
            tr: '┓',
            bl: '┗',
            br: '┛',
            h: '━',
            v: '┃',
        },
        crate::achievements::UiBorderStyle::Dashed
        | crate::achievements::UiBorderStyle::TripleDashed => PanelBorderChars {
            tl: '┌',
            tr: '┐',
            bl: '└',
            br: '┘',
            h: '┄',
            v: '┆',
        },
        crate::achievements::UiBorderStyle::HeavyDashed
        | crate::achievements::UiBorderStyle::HeavyTripleDashed => PanelBorderChars {
            tl: '┏',
            tr: '┓',
            bl: '┗',
            br: '┛',
            h: '┅',
            v: '┇',
        },
        crate::achievements::UiBorderStyle::QuadDashed => PanelBorderChars {
            tl: '┌',
            tr: '┐',
            bl: '└',
            br: '┘',
            h: '┈',
            v: '┊',
        },
        crate::achievements::UiBorderStyle::HeavyQuadDashed => PanelBorderChars {
            tl: '┏',
            tr: '┓',
            bl: '┗',
            br: '┛',
            h: '┉',
            v: '┋',
        },
        crate::achievements::UiBorderStyle::HeavyCorner => PanelBorderChars {
            tl: '┏',
            tr: '┓',
            bl: '┗',
            br: '┛',
            h: '─',
            v: '│',
        },
        crate::achievements::UiBorderStyle::MicroDash => PanelBorderChars {
            tl: '┌',
            tr: '┐',
            bl: '└',
            br: '┘',
            h: '┄',
            v: '┆',
        },
        crate::achievements::UiBorderStyle::HeaderRail
        | crate::achievements::UiBorderStyle::Classic => PanelBorderChars {
            tl: '┌',
            tr: '┐',
            bl: '└',
            br: '┘',
            h: '─',
            v: '│',
        },
    }
}

/// Maps item rarity to its display color. Single source of truth for all UI.
pub fn rarity_color(rarity: Rarity) -> Color {
    match rarity {
        Rarity::Common => Color::White,
        Rarity::Magic => Color::Blue,
        Rarity::Rare => Color::Yellow,
        Rarity::Epic => Color::Magenta,
        Rarity::Legendary => Color::LightRed,
        Rarity::Mythic => Color::Rgb(255, 215, 0),
    }
}

/// Maps item tier (T0–T9) to a cool-to-hot color gradient.
pub fn tier_color(tier: u8) -> Color {
    match tier {
        0 => Color::Gray,
        1 => Color::Rgb(60, 80, 180),   // dark blue
        2 => Color::Rgb(0, 180, 170),   // teal
        3 => Color::Rgb(80, 200, 80),   // green
        4 => Color::Rgb(220, 220, 50),  // yellow
        5 => Color::Rgb(255, 165, 0),   // orange
        6 => Color::Rgb(220, 50, 50),   // red
        7 => Color::Magenta,            // magenta
        8 => Color::Rgb(255, 150, 255), // light magenta
        _ => Color::Rgb(255, 215, 0),   // gold (T9+)
    }
}

/// Main UI drawing function with optional update notification
#[allow(clippy::too_many_arguments)]
pub fn draw_ui_with_update(
    frame: &mut Frame,
    game_state: &GameState,
    update_info: Option<&UpdateInfo>,
    update_check_completed: bool,
    update_check_failed: bool,
    haven_discovered: bool,
    soulforge_discovered: bool,
    stormglass_discovered: bool,
    deep_state: &crate::deep::DeepState,
    achievements: &crate::achievements::Achievements,
    enhancement_levels: &[u8; 7],
) {
    set_global_ui_border_style(achievements.ui_border_style);

    let ctx = LayoutContext::from_frame(frame);

    if ctx.tier == SizeTier::TooSmall {
        render_too_small(frame, &ctx);
        return;
    }

    let deep_indicator =
        stats_panel::DeepIndicatorStatus::from_deep(deep_state.persistent.discovered, deep_state);

    match ctx.tier {
        SizeTier::XL | SizeTier::L => {
            draw_xl_l_layout(
                frame,
                &ctx,
                game_state,
                update_info,
                update_check_completed,
                update_check_failed,
                haven_discovered,
                soulforge_discovered,
                stormglass_discovered,
                deep_indicator,
                achievements,
                enhancement_levels,
            );
        }
        SizeTier::M => {
            draw_m_layout(
                frame,
                &ctx,
                game_state,
                haven_discovered,
                soulforge_discovered,
                stormglass_discovered,
                deep_indicator,
                achievements,
            );
        }
        SizeTier::S => {
            draw_s_layout(frame, &ctx, game_state, achievements);
        }
        SizeTier::TooSmall => {
            // Already handled above
        }
    }
}

/// XL/L two-column layout (existing behavior).
#[allow(clippy::too_many_arguments)]
fn draw_xl_l_layout(
    frame: &mut Frame,
    ctx: &LayoutContext,
    game_state: &GameState,
    update_info: Option<&UpdateInfo>,
    update_check_completed: bool,
    update_check_failed: bool,
    haven_discovered: bool,
    soulforge_discovered: bool,
    stormglass_discovered: bool,
    deep_indicator: stats_panel::DeepIndicatorStatus,
    achievements: &crate::achievements::Achievements,
    enhancement_levels: &[u8; 7],
) {
    let size = frame.area();

    // Check if we should show the challenge notification banner
    let show_challenge_banner = !game_state.challenge_menu.challenges.is_empty()
        && !game_state.challenge_menu.is_open
        && game_state.active_minigame.is_none();

    // Split vertically: optional banner at top, main content below
    let main_area = if show_challenge_banner {
        let v_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Challenge banner
                Constraint::Min(0),    // Main content
            ])
            .split(size);

        draw_challenge_banner(frame, v_chunks[0], game_state, ctx);
        v_chunks[1]
    } else {
        size
    };

    // Split vertically: growing content area, ticker, footer
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(27),   // Main content (stats + right panel, grows)
            Constraint::Length(1), // Ticker
            Constraint::Length(4), // Full-width footer (2 rows)
        ])
        .split(main_area);

    let content_area = v_chunks[0];
    let info_area = v_chunks[1];
    let footer_area = v_chunks[2];

    // Split main content into two areas: stats panel (left) and combat/dungeon (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Stats panel
            Constraint::Percentage(50), // Combat scene or dungeon
        ])
        .split(content_area);

    // Draw stats panel on the left
    stats_panel::draw_stats_panel(
        frame,
        chunks[0],
        game_state,
        ctx,
        enhancement_levels,
        achievements,
    );

    // Draw ticker
    let sg = if game_state.stormglass_discovered {
        Some(game_state.stormglass)
    } else {
        None
    };
    ticker::draw_ticker(frame, info_area, &game_state.ticker, sg);

    // Draw full-width footer at the bottom
    stats_panel::draw_footer(
        frame,
        footer_area,
        game_state,
        update_info,
        update_check_completed,
        update_check_failed,
        haven_discovered,
        soulforge_discovered,
        stormglass_discovered,
        deep_indicator,
        achievements.pending_count(),
        ctx,
    );

    // Draw right panel with stable layout: zone info + content + info panel
    draw_right_panel(frame, chunks[1], game_state, achievements, ctx);
}

/// M tier stacked single-column layout.
/// Compact stats bar + optional attrs + XP bar + full-width activity + compact info + footer
#[allow(clippy::too_many_arguments)]
fn draw_m_layout(
    frame: &mut Frame,
    ctx: &LayoutContext,
    game_state: &GameState,
    haven_discovered: bool,
    soulforge_discovered: bool,
    stormglass_discovered: bool,
    deep_indicator: stats_panel::DeepIndicatorStatus,
    achievements: &crate::achievements::Achievements,
) {
    let area = frame.area();
    let show_attrs = ctx.rows >= 26;

    let mut constraints = vec![
        Constraint::Length(1), // Compact stats bar
    ];
    if show_attrs {
        constraints.push(Constraint::Length(1)); // Attributes single line
    }
    constraints.push(Constraint::Length(1)); // XP bar
    constraints.push(Constraint::Min(5)); // Activity area (full width)
    constraints.push(Constraint::Length(1)); // Event ticker
    constraints.push(Constraint::Length(1)); // Footer

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut idx = 0;

    // Compact stats bar
    stats_panel::draw_compact_stats_bar(frame, chunks[idx], game_state, ctx, achievements);
    idx += 1;

    // Optional attributes line
    if show_attrs {
        stats_panel::draw_attributes_single_line(frame, chunks[idx], game_state);
        idx += 1;
    }

    // XP bar
    stats_panel::draw_xp_bar_compact(frame, chunks[idx], game_state);
    idx += 1;

    // Activity area - dispatched by current activity
    draw_right_content(frame, chunks[idx], game_state, achievements, ctx);
    idx += 1;

    // Event ticker
    {
        let sg = if game_state.stormglass_discovered {
            Some(game_state.stormglass)
        } else {
            None
        };
        ticker::draw_ticker(frame, chunks[idx], &game_state.ticker, sg);
    }
    idx += 1;

    // Compact footer
    stats_panel::draw_footer_compact(
        frame,
        chunks[idx],
        game_state,
        haven_discovered,
        soulforge_discovered,
        stormglass_discovered,
        deep_indicator,
        achievements.pending_count(),
    );
}

/// S tier minimal text-only layout.
/// Status line + XP bar + player HP + enemy HP + combat status + merged feed + footer
fn draw_s_layout(
    frame: &mut Frame,
    ctx: &LayoutContext,
    game_state: &GameState,
    achievements: &crate::achievements::Achievements,
) {
    let area = frame.area();

    // Check if a minigame or special view is active — if so, give it all the space
    let has_special_activity = game_state.active_minigame.is_some()
        || game_state.challenge_menu.is_open
        || game_state.active_fishing.is_some()
        || game_state.active_dungeon.is_some();

    if has_special_activity {
        // Minimal: status line + activity area + footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Status line
                Constraint::Min(4),    // Activity
                Constraint::Length(1), // Footer
            ])
            .split(area);

        stats_panel::draw_compact_stats_bar(frame, chunks[0], game_state, ctx, achievements);
        draw_right_content(frame, chunks[1], game_state, achievements, ctx);
        stats_panel::draw_footer_minimal(frame, chunks[2], game_state);
        return;
    }

    // Standard S layout: combat-focused
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Status line
            Constraint::Length(1), // XP bar
            Constraint::Length(1), // Player HP
            Constraint::Length(1), // Enemy HP + name
            Constraint::Min(1),    // Combat status (grows)
            Constraint::Length(1), // Event ticker
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Status line
    stats_panel::draw_compact_stats_bar(frame, chunks[0], game_state, ctx, achievements);

    // XP bar
    stats_panel::draw_xp_bar_compact(frame, chunks[1], game_state);

    // Player HP bar
    draw_s_player_hp(frame, chunks[2], game_state);

    // Enemy HP + name
    draw_s_enemy_hp(frame, chunks[3], game_state);

    // Combat status
    combat_scene::draw_combat_scene(frame, chunks[4], game_state, achievements, ctx);

    // Event ticker
    {
        let sg = if game_state.stormglass_discovered {
            Some(game_state.stormglass)
        } else {
            None
        };
        ticker::draw_ticker(frame, chunks[5], &game_state.ticker, sg);
    }

    // Minimal footer
    stats_panel::draw_footer_minimal(frame, chunks[6], game_state);
}

/// Draws player HP bar for S tier (borderless, single line) with optional damage flash.
fn draw_s_player_hp(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let hp_ratio = game_state.combat_state.player_current_hp as f64
        / game_state.combat_state.player_max_hp as f64;

    let label = format!(
        "HP: {}/{}",
        game_state.combat_state.player_current_hp, game_state.combat_state.player_max_hp
    );

    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .label(label)
        .ratio(hp_ratio);

    if let Some(flash) = game_state.combat_state.player_damage_floats.last() {
        let flash_width = (flash.text.chars().count() as u16) + 1;
        if area.width > flash_width + 15 {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(15), Constraint::Length(flash_width)])
                .split(area);

            frame.render_widget(gauge, chunks[0]);

            let mut style = Style::default().fg(flash.color);
            if flash.bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            let flash_para =
                Paragraph::new(Span::styled(&flash.text, style)).alignment(Alignment::Right);
            frame.render_widget(flash_para, chunks[1]);
        } else {
            frame.render_widget(gauge, area);
        }
    } else {
        frame.render_widget(gauge, area);
    }
}

/// Draws enemy HP bar for S tier (borderless, single line) with optional damage flash.
fn draw_s_enemy_hp(frame: &mut Frame, area: Rect, game_state: &GameState) {
    if let Some(enemy) = &game_state.combat_state.current_enemy {
        let hp_ratio = enemy.current_hp as f64 / enemy.max_hp as f64;
        let label = format!("{}: {}/{}", enemy.name, enemy.current_hp, enemy.max_hp);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .label(label)
            .ratio(hp_ratio);

        if let Some(flash) = game_state.combat_state.enemy_damage_floats.last() {
            let flash_width = (flash.text.chars().count() as u16) + 1;
            if area.width > flash_width + 15 {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(15), Constraint::Length(flash_width)])
                    .split(area);

                frame.render_widget(gauge, chunks[0]);

                let mut style = Style::default().fg(flash.color);
                if flash.bold {
                    style = style.add_modifier(Modifier::BOLD);
                }
                let flash_para =
                    Paragraph::new(Span::styled(&flash.text, style)).alignment(Alignment::Right);
                frame.render_widget(flash_para, chunks[1]);
            } else {
                frame.render_widget(gauge, area);
            }
        } else {
            frame.render_widget(gauge, area);
        }
    }
}

/// Draws the challenge notification banner at the top of the screen
fn draw_challenge_banner(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    _ctx: &LayoutContext,
) {
    let challenges = &game_state.challenge_menu.challenges;
    let count = challenges.len();

    let spans = if count == 1 {
        // Show specific challenge info
        let challenge = &challenges[0];
        vec![
            Span::styled(
                format!(" {} {} ", challenge.icon, challenge.title),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("[Tab] to view", Style::default().fg(Color::DarkGray)),
        ]
    } else {
        // Show count
        vec![
            Span::styled(
                format!(" 🎲 {} Challenges Available! ", count),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("[Tab] to view", Style::default().fg(Color::DarkGray)),
        ]
    };

    let banner = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .style(Style::default().bg(Color::Rgb(40, 40, 20)));

    frame.render_widget(banner, area);
}

/// Draws the right panel with a stable 2-part layout: zone info and content.
/// The content area changes based on activity but zone info stays fixed.
fn draw_right_panel(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    achievements: &crate::achievements::Achievements,
    ctx: &LayoutContext,
) {
    // Zone info + progress bar at top (compact during minigames for more grid space)
    let zone_height = if game_state.active_minigame.is_some() {
        3
    } else if ctx.tier >= SizeTier::XL {
        9
    } else {
        10
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(zone_height), // Zone info + segmented progress bar
            Constraint::Min(10),             // Content (changes by activity)
        ])
        .split(area);

    stats_panel::draw_zone_info(frame, chunks[0], game_state, achievements, ctx);

    // Content area — dispatched by current activity
    draw_right_content(frame, chunks[1], game_state, achievements, ctx);
}

/// Draws the main content area of the right panel based on current activity.
/// Priority: minigame > challenge menu > fishing > dungeon > combat
fn draw_right_content(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    achievements: &crate::achievements::Achievements,
    ctx: &LayoutContext,
) {
    // Show "[Press any key]" only after the game-over dismiss cooldown expires
    let show_dismiss_hint = game_state
        .game_over_shown_at
        .is_some_and(|t| t.elapsed() >= std::time::Duration::from_secs(2));

    let sg_discovered = game_state.stormglass_discovered;

    match &game_state.active_minigame {
        Some(ActiveMinigame::Rune(game)) => {
            rune_scene::render_rune(frame, area, game, ctx, show_dismiss_hint, sg_discovered);
        }
        Some(ActiveMinigame::Minesweeper(game)) => {
            minesweeper_scene::render_minesweeper(frame, area, game, ctx, show_dismiss_hint);
        }
        Some(ActiveMinigame::Gomoku(game)) => {
            gomoku_scene::render_gomoku_scene(
                frame,
                area,
                game,
                ctx,
                show_dismiss_hint,
                sg_discovered,
            );
        }
        Some(ActiveMinigame::Morris(game)) => {
            morris_scene::render_morris_scene(
                frame,
                area,
                game,
                ctx,
                show_dismiss_hint,
                sg_discovered,
            );
        }
        Some(ActiveMinigame::Chess(game)) => {
            chess_scene::render_chess_scene(frame, area, game, ctx, show_dismiss_hint);
        }
        Some(ActiveMinigame::Go(game)) => {
            go_scene::render_go_scene(frame, area, game, ctx, show_dismiss_hint, sg_discovered);
        }
        Some(ActiveMinigame::FlappyBird(game)) => {
            flappy_scene::render_flappy_scene(
                frame,
                area,
                game,
                ctx,
                show_dismiss_hint,
                sg_discovered,
            );
        }
        Some(ActiveMinigame::Jezzball(game)) => {
            jezzball_scene::render_jezzball_scene(
                frame,
                area,
                game,
                ctx,
                show_dismiss_hint,
                sg_discovered,
            );
        }
        Some(ActiveMinigame::Snake(game)) => {
            snake_scene::render_snake_scene(
                frame,
                area,
                game,
                ctx,
                show_dismiss_hint,
                sg_discovered,
            );
        }
        Some(ActiveMinigame::RunicShift(game)) => {
            runic_shift_scene::render_runic_shift_scene(
                frame,
                area,
                game,
                ctx,
                show_dismiss_hint,
                sg_discovered,
            );
        }
        None => {
            if game_state.challenge_menu.is_open {
                challenge_menu_scene::render_challenge_menu(
                    frame,
                    area,
                    &game_state.challenge_menu,
                    ctx,
                    game_state.stormglass_discovered,
                );
            } else if let Some(ref session) = game_state.active_fishing {
                fishing_scene::render_fishing_scene(frame, area, session, &game_state.fishing, ctx);
            } else if let Some(dungeon) = &game_state.active_dungeon {
                draw_dungeon_view(frame, area, game_state, dungeon, achievements, ctx);
            } else {
                combat_scene::draw_combat_scene(frame, area, game_state, achievements, ctx);
            }
        }
    }
}

/// Returns the icon of the highest unlocked dungeon achievement, if any.
fn highest_dungeon_badge(achievements: &crate::achievements::Achievements) -> Option<&'static str> {
    use crate::achievements::AchievementId;

    let dungeon_achievements = [
        AchievementId::DungeonMasterX,
        AchievementId::DungeonMasterIX,
        AchievementId::DungeonMasterVIII,
        AchievementId::DungeonMasterVII,
        AchievementId::DungeonMasterVI,
        AchievementId::DungeonMasterV,
        AchievementId::DungeonMasterIV,
        AchievementId::DungeonMasterIII,
        AchievementId::DungeonMasterII,
        AchievementId::DungeonMasterI,
        AchievementId::DungeonDiver,
    ];

    for id in dungeon_achievements {
        if achievements.is_unlocked(id) {
            return crate::achievements::data::get_achievement_def(id).map(|def| def.icon);
        }
    }
    None
}

/// Draws the dungeon view with combat HUD overlay on the map.
/// Instead of splitting into separate dungeon map + combat panels,
/// combat info (HP bars, status) is rendered inside the dungeon panel.
fn draw_dungeon_view(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    dungeon: &crate::dungeon::types::Dungeon,
    achievements: &crate::achievements::Achievements,
    _ctx: &LayoutContext,
) {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Single border wrapping everything
    let dungeon_title = match highest_dungeon_badge(achievements) {
        Some(icon) => format!(" Dungeon {} ", icon),
        None => " Dungeon ".to_string(),
    };
    let block = Block::default()
        .title(dungeon_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(themed_border_color(Color::Magenta)));
    let inner = render_themed_block(frame, area, block, Color::Magenta, BorderFxContext);

    // Layout: player HP, dungeon status, map, enemy HP, combat status
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Player HP
            Constraint::Length(1), // Dungeon status line
            Constraint::Min(0),    // Map (fills remaining space)
            Constraint::Length(1), // Enemy HP
            Constraint::Length(1), // Combat status
        ])
        .split(inner);

    // Player HP bar
    combat_scene::draw_player_hp(frame, inner_chunks[0], game_state);

    // Dungeon status (size, rooms cleared, key)
    let status_widget = dungeon_map::DungeonStatusWidget::new(dungeon);
    frame.render_widget(status_widget, inner_chunks[1]);

    // Calculate blink phase (0.5 second cycle)
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let blink_phase = (millis % 500) as f64 / 500.0;

    // Dungeon map backdrop + map overlay
    draw_dungeon_backdrop(frame, inner_chunks[2], dungeon.zone_id);
    let map_widget = dungeon_map::DungeonMapWidget::new(dungeon, blink_phase);
    frame.render_widget(map_widget, inner_chunks[2]);

    // Enemy HP bar
    combat_scene::draw_enemy_hp(frame, inner_chunks[3], game_state);

    // Combat status (timers, DPS)
    combat_scene::draw_combat_status(frame, inner_chunks[4], game_state);
}

/// Draws a black backdrop behind the dungeon map.
fn draw_dungeon_backdrop(frame: &mut Frame, area: Rect, _zone_id: u32) {
    use ratatui::widgets::Clear;
    frame.render_widget(Clear, area);
}
