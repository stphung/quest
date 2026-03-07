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
mod fracture_sprites_1;
mod fracture_sprites_2;
pub mod game_common;
pub mod go_scene;
pub mod gomoku_scene;
mod haven_details;
pub mod haven_scene;
mod haven_tree;
pub mod jezzball_scene;
pub mod loom_scene;
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
pub mod stormglass_scene;
pub(crate) mod throbber;
pub mod ticker;
pub mod time_vault_scene;
pub mod title_browser_scene;
mod zone_bg;

use crate::challenges::ActiveMinigame;
use crate::combat::logic::effective_enemy_attack_interval;
use crate::core::constants::{ATTACK_INTERVAL_SECONDS, BOSS_ENRAGE_SECONDS};
use crate::core::game_state::GameState;
use crate::utils::updater::UpdateInfo;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, LineGauge, Paragraph},
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
    loom_discovered: bool,
    achievements: &crate::achievements::Achievements,
    enhancement_levels: &[u8; 7],
    deep_for_cores: &crate::deep::DeepState,
) {
    set_global_ui_border_style(achievements.ui_border_style);

    let ctx = LayoutContext::from_frame(frame);

    if ctx.tier == SizeTier::TooSmall {
        render_too_small(frame, &ctx);
        return;
    }

    let deep_indicator =
        stats_panel::DeepIndicatorStatus::from_deep(deep_state.persistent.discovered, deep_state);

    // Only show [U] Ascend once the player meets the Deep layer gate for their next level
    // (or already has an ascension level). They don't need enough PR — just the layer unlock.
    let show_ascend = deep_state.persistent.discovered
        && (game_state.ascension_level > 0
            || crate::ascension::types::ascension_deep_gate(game_state.ascension_level + 1)
                .is_none_or(|gate| deep_state.persistent.deepest_layer_reached >= gate));

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
                loom_discovered,
                show_ascend,
                achievements,
                enhancement_levels,
                deep_for_cores,
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
                loom_discovered,
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
    loom_discovered: bool,
    show_ascend: bool,
    achievements: &crate::achievements::Achievements,
    enhancement_levels: &[u8; 7],
    deep_for_cores: &crate::deep::DeepState,
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
        deep_for_cores,
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
        loom_discovered,
        show_ascend,
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
    loom_discovered: bool,
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
        loom_discovered,
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
    let ratio = game_state.combat_state.player_current_hp as f64
        / game_state.combat_state.player_max_hp as f64;
    let label = format_hp_label(
        "HP",
        game_state.combat_state.player_current_hp,
        game_state.combat_state.player_max_hp,
    );
    draw_status_row(
        frame,
        area,
        &label,
        ratio,
        Color::Green,
        0,
        None,
        &[],
        game_state.combat_state.player_damage_floats.last(),
    );
}

/// Draws enemy HP bar for S tier (borderless, single line) with optional damage flash.
fn draw_s_enemy_hp(frame: &mut Frame, area: Rect, game_state: &GameState) {
    if let Some(enemy) = &game_state.combat_state.current_enemy {
        let ratio = enemy.current_hp as f64 / enemy.max_hp as f64;
        let label = format_hp_label(&enemy.name, enemy.current_hp, enemy.max_hp);
        draw_status_row(
            frame,
            area,
            &label,
            ratio,
            Color::Red,
            0,
            None,
            &[],
            game_state.combat_state.enemy_damage_floats.last(),
        );
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

/// Draws the right panel with a bordered outer frame (L/XL tiers).
/// Minigames skip the border and get zone info + full content instead.
/// Non-minigame activities get: outer border with zone title, zone info (top),
/// content (middle), and a context-sensitive status strip (bottom 2 rows).
fn draw_right_panel(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    achievements: &crate::achievements::Achievements,
    ctx: &LayoutContext,
) {
    // Skip border wrapping for minigames — they have their own frames
    if game_state.active_minigame.is_some() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(10)])
            .split(area);
        stats_panel::draw_zone_info(frame, chunks[0], game_state, achievements, ctx);
        draw_right_content(frame, chunks[1], game_state, achievements, ctx);
        return;
    }

    // Get zone name and color for the border
    use crate::zones::get_all_zones;
    let zones = get_all_zones();
    let prog = &game_state.zone_progression;
    let zone = zones.iter().find(|z| z.id == prog.current_zone_id);
    let zone_name = zone.map(|z| z.name).unwrap_or("Unknown");
    let zone_color = stats_panel::zone_color_for_id(prog.current_zone_id);

    // Outer bordered block with zone name as title
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(themed_border_color(zone_color)))
        .title(format!(" {} ", zone_name))
        .title_style(Style::default().fg(zone_color).add_modifier(Modifier::BOLD));
    let block = themed_block(block);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    apply_themed_border_fx(frame, area, zone_color, BorderFxContext);

    // Split inner into zone info, divider, content, divider, status strip
    let zone_height = if ctx.tier >= SizeTier::XL { 6 } else { 7 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(zone_height), // Zone info
            Constraint::Length(1),           // Divider
            Constraint::Min(8),              // Content
            Constraint::Length(1),           // Divider
            Constraint::Length(2),           // Status strip
        ])
        .split(inner);

    stats_panel::draw_zone_info(frame, chunks[0], game_state, achievements, ctx);

    // Horizontal dividers styled with zone border color
    let divider_style = Style::default().fg(themed_border_color(zone_color));
    let divider_char = panel_border_chars().h;
    let divider_str: String = std::iter::repeat_n(divider_char, chunks[1].width as usize).collect();
    let divider = Paragraph::new(divider_str.as_str()).style(divider_style);
    frame.render_widget(divider.clone(), chunks[1]);
    frame.render_widget(divider, chunks[3]);

    draw_right_content(frame, chunks[2], game_state, achievements, ctx);
    draw_status_strip(frame, chunks[4], game_state);
}

/// Draws the 2-row status strip at the bottom of the unified right panel.
/// Shows context-sensitive info: fishing status, dungeon HP, combat HP, or idle.
fn draw_status_strip(frame: &mut Frame, area: Rect, game_state: &GameState) {
    if area.height < 2 {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // Priority: dungeon > fishing > combat/idle
    if game_state.active_dungeon.is_some() {
        draw_status_strip_dungeon(frame, rows[0], rows[1], game_state);
    } else if let Some(ref session) = game_state.active_fishing {
        draw_status_strip_fishing(frame, rows[0], rows[1], session, game_state);
    } else {
        draw_status_strip_combat(frame, rows[0], rows[1], game_state);
    }
}

/// Fishing status strip: rank/caught on row 1, phase on row 2.
fn draw_status_strip_fishing(
    frame: &mut Frame,
    row0: Rect,
    row1: Rect,
    session: &crate::fishing::types::FishingSession,
    game_state: &GameState,
) {
    let rank_name = game_state.fishing.rank_name();
    let caught = session.fish_caught.len() as u32;
    let total = session.total_fish;
    let rank_text = Paragraph::new(Line::from(vec![
        Span::styled("Rank: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(rank_name, Style::default().fg(Color::Cyan)),
        Span::raw(format!("  |  Caught: {}/{}", caught, total)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(rank_text, row0);

    let spinner = throbber::spinner_char();
    let (phase_text, phase_color) = match session.phase {
        crate::fishing::types::FishingPhase::Casting => {
            (format!("{} Casting line...", spinner), Color::White)
        }
        crate::fishing::types::FishingPhase::Waiting => {
            (format!("{} Waiting for bite...", spinner), Color::Cyan)
        }
        crate::fishing::types::FishingPhase::Reeling => {
            ("\u{1f41f} FISH ON! Reeling in!".to_string(), Color::Yellow)
        }
    };
    let phase = Paragraph::new(Line::from(Span::styled(
        phase_text,
        Style::default()
            .fg(phase_color)
            .add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(phase, row1);
}

/// Dungeon status strip: delegates to shared table layout with room/key info.
fn draw_status_strip_dungeon(frame: &mut Frame, row0: Rect, row1: Rect, game_state: &GameState) {
    // Row 1 info: room/key progress
    let row1_info = if let Some(ref dungeon) = game_state.active_dungeon {
        let key_str = if dungeon.has_key { " \u{1f511}" } else { "" };
        Span::styled(
            format!(
                " Rm {}/{}{}",
                dungeon.rooms_cleared,
                dungeon.room_count(),
                key_str
            ),
            Style::default().fg(Color::Magenta),
        )
    } else {
        Span::default()
    };
    let idle_msg = format!("{} Exploring...", throbber::spinner_char());
    draw_status_strip_table(frame, row0, row1, game_state, &row1_info, &idle_msg);
}

/// Formats an HP label like "HP:340/500" or "Goblin:12.4K/25K" using short
/// number formatting for values >= 10,000.
fn format_hp_label(name: &str, current: u32, max: u32) -> String {
    let cur_s = game_common::format_number_short(current as u64);
    let max_s = game_common::format_number_short(max as u64);
    format!("{}:{}/{}", name, cur_s, max_s)
}

/// Attack timer gauge info for `draw_status_row`.
struct TimerGauge {
    label: String,
    /// 0.0 = just attacked, 1.0 = about to attack
    ratio: f64,
    color: Color,
}

/// Renders one status strip row: `[HP Gauge] [Timer Gauge] | seg | seg ... flash`
///
/// Uses ratatui Gauges for both the HP bar and the optional attack timer.
/// Layout: `[HP gauge (fixed)] [timer gauge (fixed, optional)] [text segments + flash (fill)]`
#[allow(clippy::too_many_arguments)]
fn draw_status_row(
    frame: &mut Frame,
    area: Rect,
    hp_label: &str,
    hp_ratio: f64,
    bar_color: Color,
    label_width: usize,
    timer: Option<&TimerGauge>,
    segments: &[Span],
    flash: Option<&crate::combat::types::DamageFlash>,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    // Truncate HP label to fit gauge column
    let label: &str = if label_width > 0 && hp_label.len() > label_width {
        &hp_label[..hp_label.floor_char_boundary(label_width)]
    } else {
        hp_label
    };

    // Layout: [HP gauge] [timer gauge (optional)] [info text]
    let hp_gauge_width = (label_width as u16 + 2).min(area.width);
    let timer_width: u16 = if timer.is_some() { 11 } else { 0 };
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(hp_gauge_width),
            Constraint::Length(timer_width),
            Constraint::Min(1),
        ])
        .split(area);

    // HP Gauge
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(bar_color).add_modifier(Modifier::BOLD))
        .label(label)
        .ratio(hp_ratio.clamp(0.0, 1.0));
    frame.render_widget(gauge, chunks[0]);

    // Timer Gauge (optional) — thin line gauge for attack cooldown
    if let Some(timer) = timer {
        let timer_gauge = LineGauge::default()
            .filled_style(Style::default().fg(timer.color))
            .unfilled_style(Style::default().fg(Color::DarkGray))
            .label(timer.label.as_str())
            .ratio(timer.ratio.clamp(0.0, 1.0));
        frame.render_widget(timer_gauge, chunks[1]);
    }

    // Text segments + flash
    let info_area = chunks[2];
    let mut spans: Vec<Span> = Vec::with_capacity(8);

    for seg in segments {
        spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
        spans.push(seg.clone());
    }

    // Flash: render right-aligned in remaining space
    if let Some(flash) = flash {
        let used: usize = spans.iter().map(|s| s.content.chars().count()).sum();
        let flash_len = flash.text.chars().count();
        let available = (info_area.width as usize).saturating_sub(used + 1);
        if flash_len <= available {
            let pad = available.saturating_sub(flash_len);
            if pad > 0 {
                spans.push(Span::raw(" ".repeat(pad)));
            }
            let progress = 1.0 - (flash.remaining / crate::combat::types::DAMAGE_FLASH_DURATION);
            let style = if progress > 0.8 {
                Style::default().fg(Color::DarkGray)
            } else if progress > 0.6 {
                Style::default().fg(flash.color)
            } else {
                let mut s = Style::default().fg(flash.color);
                if flash.bold {
                    s = s.add_modifier(Modifier::BOLD);
                }
                s
            };
            spans.push(Span::styled(&flash.text, style));
        }
    }

    if !spans.is_empty() {
        let info = Paragraph::new(Line::from(spans));
        frame.render_widget(info, info_area);
    }
}

/// Combat/idle status strip: delegates to shared table layout with player DPS.
fn draw_status_strip_combat(frame: &mut Frame, row0: Rect, row1: Rect, game_state: &GameState) {
    let derived = game_state.cached_derived_stats;
    let base_dps = derived.total_damage() as f64 / ATTACK_INTERVAL_SECONDS;
    let effective_dps = base_dps
        * (1.0 + (derived.crit_chance_percent as f64 / 100.0) * (derived.crit_multiplier - 1.0));

    let row1_info = Span::styled(
        format!(" DPS:{:.0}", effective_dps),
        Style::default().fg(Color::DarkGray),
    );
    let msg = throbber::waiting_message(game_state.character_xp);
    let idle_msg = format!("{} {}", throbber::spinner_char(), msg);
    draw_status_strip_table(frame, row0, row1, game_state, &row1_info, &idle_msg);
}

/// Renders a damage flash right-aligned in the given area.
fn render_flash(frame: &mut Frame, area: Rect, flash: Option<&crate::combat::types::DamageFlash>) {
    if let Some(flash) = flash {
        let progress = 1.0 - (flash.remaining / crate::combat::types::DAMAGE_FLASH_DURATION);
        let style = if progress > 0.8 {
            Style::default().fg(Color::DarkGray)
        } else if progress > 0.6 {
            Style::default().fg(flash.color)
        } else {
            let mut s = Style::default().fg(flash.color);
            if flash.bold {
                s = s.add_modifier(Modifier::BOLD);
            }
            s
        };
        let text = Paragraph::new(Span::styled(&flash.text, style))
            .alignment(ratatui::layout::Alignment::Right);
        frame.render_widget(text, area);
    }
}

/// Shared table-layout status strip for combat and dungeon contexts.
/// Columns: [HP Gauge] [Spacer] [Timer Label] [Timer LineGauge] [Info] [Flash]
///
/// `row1_info`: right-column content for the player row (DPS or room info).
/// `idle_msg`: label shown in the HP gauge area when no enemy is present.
fn draw_status_strip_table(
    frame: &mut Frame,
    row0: Rect,
    row1: Rect,
    game_state: &GameState,
    row1_info: &Span,
    idle_msg: &str,
) {
    let hp = &game_state.combat_state;
    let derived = game_state.cached_derived_stats;

    let hp_ratio = if hp.player_max_hp > 0 {
        (hp.player_current_hp as f64 / hp.player_max_hp as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let player_label = format_hp_label("HP", hp.player_current_hp, hp.player_max_hp);

    // Proportional 4-column layout: [HP Gauge] [Timer w/ label] [Info] [Flash]
    let col_constraints = [
        Constraint::Percentage(38), // HP Gauge
        Constraint::Fill(1),        // Timer LineGauge (with label)
        Constraint::Percentage(15), // Info (DPS / room)
        Constraint::Percentage(15), // Flash
    ];

    // --- Row 1: Player ---
    let cols0 = Layout::horizontal(col_constraints).split(row0);

    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .label(player_label.as_str())
        .ratio(hp_ratio.clamp(0.0, 1.0));
    frame.render_widget(gauge, cols0[0]);

    let player_interval = ATTACK_INTERVAL_SECONDS / derived.attack_speed_multiplier;
    let (player_timer_label, player_timer_ratio, player_timer_color) = if hp.current_enemy.is_some()
    {
        let next = (player_interval - hp.player_attack_timer).max(0.0);
        let ratio = 1.0 - (next / player_interval);
        let color = if next < 0.3 {
            Color::LightGreen
        } else {
            Color::Green
        };
        (format!(" You:{:.1}s", next), ratio, color)
    } else {
        (
            format!(" You:{:.1}s", player_interval),
            0.0,
            Color::DarkGray,
        )
    };

    let timer_gauge = LineGauge::default()
        .filled_style(Style::default().fg(player_timer_color))
        .unfilled_style(Style::default().fg(Color::DarkGray))
        .label(player_timer_label.as_str())
        .ratio(player_timer_ratio.clamp(0.0, 1.0));
    frame.render_widget(timer_gauge, cols0[1]);

    let info_text = Paragraph::new(Line::from(row1_info.clone()));
    frame.render_widget(info_text, cols0[2]);

    render_flash(frame, cols0[3], hp.player_damage_floats.last());

    // --- Row 2: Enemy or idle ---
    let cols1 = Layout::horizontal(col_constraints).split(row1);

    if let Some(enemy) = &hp.current_enemy {
        let enemy_ratio = if enemy.max_hp > 0 {
            (enemy.current_hp as f64 / enemy.max_hp as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let is_boss = game_state.zone_progression.fighting_boss || enemy.name.starts_with("Boss ");
        let hp_color = if is_boss {
            Color::LightRed
        } else {
            let zone_id = game_state
                .active_dungeon
                .as_ref()
                .map(|d| d.zone_id)
                .unwrap_or(game_state.zone_progression.current_zone_id);
            enemy_sprites::zone_palette(zone_id).primary
        };

        // Dynamic name truncation based on gauge width
        let gauge_width = cols1[0].width as usize;
        let name_budget = gauge_width.saturating_sub(12).max(4);
        let name: String = enemy.name.chars().take(name_budget).collect();
        let enemy_label = format_hp_label(&name, enemy.current_hp, enemy.max_hp);

        let enemy_gauge = Gauge::default()
            .gauge_style(Style::default().fg(hp_color).add_modifier(Modifier::BOLD))
            .label(enemy_label.as_str())
            .ratio(enemy_ratio.clamp(0.0, 1.0));
        frame.render_widget(enemy_gauge, cols1[0]);

        let enemy_interval = effective_enemy_attack_interval(game_state);

        // Boss enrage uses the timer column; normal enemies use LineGauge
        if game_state.zone_progression.fighting_boss {
            let remaining = (BOSS_ENRAGE_SECONDS - hp.boss_fight_timer).max(0.0);
            let enrage_style = if remaining < 5.0 {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else if remaining < 10.0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Cyan)
            };
            let enrage_text = Paragraph::new(Span::styled(
                format!(" \u{26a1}Enrage:{:.0}s", remaining),
                enrage_style,
            ));
            frame.render_widget(enrage_text, cols1[1]);
        } else {
            let enemy_next = (enemy_interval - hp.enemy_attack_timer).max(0.0);
            let ratio = 1.0 - (enemy_next / enemy_interval);
            let color = if enemy_next < 0.3 {
                Color::LightRed
            } else {
                Color::Red
            };

            let foe_gauge = LineGauge::default()
                .filled_style(Style::default().fg(color))
                .unfilled_style(Style::default().fg(Color::DarkGray))
                .label(format!(" Foe:{:.1}s", enemy_next))
                .ratio(ratio.clamp(0.0, 1.0));
            frame.render_widget(foe_gauge, cols1[1]);
        }

        // Enemy DPS
        let enemy_dps = enemy.damage as f64 / enemy_interval;
        let enemy_dps_text = Paragraph::new(Span::styled(
            format!(" DPS:{:.0}", enemy_dps),
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(enemy_dps_text, cols1[2]);

        render_flash(frame, cols1[3], hp.enemy_damage_floats.last());
    } else {
        // Idle row
        let idle_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::DarkGray))
            .label(idle_msg)
            .ratio(0.0);
        frame.render_widget(idle_gauge, cols1[0]);

        let foe_gauge = LineGauge::default()
            .filled_style(Style::default().fg(Color::DarkGray))
            .unfilled_style(Style::default().fg(Color::DarkGray))
            .label(" Foe:---")
            .ratio(0.0);
        frame.render_widget(foe_gauge, cols1[1]);
    }
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
                fishing_scene::render_fishing_scene(
                    frame,
                    area,
                    session,
                    game_state,
                    achievements,
                    ctx,
                );
            } else if let Some(dungeon) = &game_state.active_dungeon {
                draw_dungeon_view(frame, area, game_state, dungeon, achievements, ctx);
            } else {
                combat_scene::draw_combat_scene(frame, area, game_state, achievements, ctx);
            }
        }
    }
}

/// Draws the dungeon view: status line + map (no border or HP bars).
/// The unified right panel provides the outer border and status strip.
fn draw_dungeon_view(
    frame: &mut Frame,
    area: Rect,
    _game_state: &GameState,
    dungeon: &crate::dungeon::types::Dungeon,
    _achievements: &crate::achievements::Achievements,
    _ctx: &LayoutContext,
) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Dungeon status line (size, rooms, key)
            Constraint::Min(0),    // Map (fills remaining)
        ])
        .split(area);

    // Dungeon status (size, rooms cleared, key)
    let status_widget = dungeon_map::DungeonStatusWidget::new(dungeon);
    frame.render_widget(status_widget, chunks[0]);

    // Calculate blink phase (0.5 second cycle)
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let blink_phase = (millis % 500) as f64 / 500.0;

    // Dungeon map backdrop + map overlay
    draw_dungeon_backdrop(frame, chunks[1], dungeon.zone_id);
    let map_widget = dungeon_map::DungeonMapWidget::new(dungeon, blink_phase);
    frame.render_widget(map_widget, chunks[1]);
}

/// Draws a black backdrop behind the dungeon map.
fn draw_dungeon_backdrop(frame: &mut Frame, area: Rect, _zone_id: u32) {
    use ratatui::widgets::Clear;
    frame.render_widget(Clear, area);
}
