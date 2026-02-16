pub mod achievement_browser_scene;
pub mod challenge_menu_scene;
pub mod character_creation;
pub mod character_delete;
pub mod character_rename;
pub mod character_select;
pub mod chess_scene;
mod combat_3d;
pub mod combat_effects;
mod combat_scene;
pub mod debug_menu_scene;
pub mod dungeon_map;
mod enemy_sprites;
pub mod fishing_scene;
pub mod flappy_scene;
pub mod game_common;
pub mod go_scene;
pub mod gomoku_scene;
pub mod haven_scene;
pub mod help_overlay;
pub mod jezzball_scene;
pub mod minesweeper_scene;
pub mod morris_scene;
pub mod prestige_confirm;
pub mod responsive;
pub mod rune_scene;
pub mod runic_shift_scene;
mod scene_fx;
pub mod snake_scene;
pub mod soulforge_scene;
mod stats_panel;
mod throbber;
pub mod ticker;

use crate::challenges::ActiveMinigame;
use crate::core::game_state::GameState;
use crate::utils::updater::UpdateInfo;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use responsive::{render_too_small, LayoutContext, SizeTier};

/// Main UI drawing function with optional update notification
#[allow(clippy::too_many_arguments)]
pub fn draw_ui_with_update(
    frame: &mut Frame,
    game_state: &GameState,
    update_info: Option<&UpdateInfo>,
    update_expanded: bool,
    update_check_completed: bool,
    haven_discovered: bool,
    soulforge_discovered: bool,
    achievements: &crate::achievements::Achievements,
    enhancement_levels: &[u8; 7],
) {
    let ctx = LayoutContext::from_frame(frame);

    if ctx.tier == SizeTier::TooSmall {
        render_too_small(frame, &ctx);
        return;
    }

    match ctx.tier {
        SizeTier::XL | SizeTier::L => {
            draw_xl_l_layout(
                frame,
                &ctx,
                game_state,
                update_info,
                update_expanded,
                update_check_completed,
                haven_discovered,
                soulforge_discovered,
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
    update_expanded: bool,
    update_check_completed: bool,
    haven_discovered: bool,
    soulforge_discovered: bool,
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

    // Determine if we need space for update drawer
    let show_update_drawer = update_expanded && update_info.is_some();

    // Split vertically: growing content area, ticker, optional update drawer, footer
    let v_chunks = if show_update_drawer {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(27),    // Main content (stats + right panel, grows)
                Constraint::Length(1),  // Ticker
                Constraint::Length(20), // Update drawer panel (tall enough for wrapped changelog lines)
                Constraint::Length(4),  // Full-width footer (2 rows)
            ])
            .split(main_area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(27),   // Main content (stats + right panel, grows)
                Constraint::Length(1), // Ticker
                Constraint::Length(4), // Full-width footer (2 rows)
            ])
            .split(main_area)
    };

    let content_area = v_chunks[0];
    let info_area = v_chunks[1];
    let (update_drawer_area, footer_area) = if show_update_drawer {
        (Some(v_chunks[2]), v_chunks[3])
    } else {
        (None, v_chunks[2])
    };

    // Split main content into two areas: stats panel (left) and combat/dungeon (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Stats panel
            Constraint::Percentage(50), // Combat scene or dungeon
        ])
        .split(content_area);

    // Draw stats panel on the left
    stats_panel::draw_stats_panel(frame, chunks[0], game_state, ctx, enhancement_levels);

    // Draw ticker
    ticker::draw_ticker(frame, info_area, &game_state.loot_ticker);

    // Draw update drawer if expanded
    if let (Some(drawer_area), Some(info)) = (update_drawer_area, update_info) {
        stats_panel::draw_update_drawer(frame, drawer_area, info);
    }

    // Draw full-width footer at the bottom
    stats_panel::draw_footer(
        frame,
        footer_area,
        game_state,
        update_info,
        update_expanded,
        update_check_completed,
        haven_discovered,
        soulforge_discovered,
        achievements.pending_count(),
        ctx,
    );

    // Draw right panel with stable layout: zone info + content + info panel
    draw_right_panel(frame, chunks[1], game_state, achievements, ctx);
}

/// M tier stacked single-column layout.
/// Compact stats bar + optional attrs + XP bar + full-width activity + compact info + footer
fn draw_m_layout(
    frame: &mut Frame,
    ctx: &LayoutContext,
    game_state: &GameState,
    haven_discovered: bool,
    soulforge_discovered: bool,
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
    constraints.push(Constraint::Length(1)); // Loot ticker
    constraints.push(Constraint::Length(1)); // Footer

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut idx = 0;

    // Compact stats bar
    stats_panel::draw_compact_stats_bar(frame, chunks[idx], game_state, ctx);
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
    draw_right_content(frame, chunks[idx], game_state, ctx);
    idx += 1;

    // Loot ticker
    ticker::draw_ticker(frame, chunks[idx], &game_state.loot_ticker);
    idx += 1;

    // Compact footer
    stats_panel::draw_footer_compact(
        frame,
        chunks[idx],
        game_state,
        haven_discovered,
        soulforge_discovered,
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

        stats_panel::draw_compact_stats_bar(frame, chunks[0], game_state, ctx);
        draw_right_content(frame, chunks[1], game_state, ctx);
        stats_panel::draw_footer_minimal(frame, chunks[2], game_state);
        return;
    }

    // Standard S layout: combat-focused
    let _ = achievements; // Not used in S layout currently
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Status line
            Constraint::Length(1), // XP bar
            Constraint::Length(1), // Player HP
            Constraint::Length(1), // Enemy HP + name
            Constraint::Min(1),    // Combat status (grows)
            Constraint::Length(1), // Loot ticker
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Status line
    stats_panel::draw_compact_stats_bar(frame, chunks[0], game_state, ctx);

    // XP bar
    stats_panel::draw_xp_bar_compact(frame, chunks[1], game_state);

    // Player HP bar
    draw_s_player_hp(frame, chunks[2], game_state);

    // Enemy HP + name
    draw_s_enemy_hp(frame, chunks[3], game_state);

    // Combat status
    combat_scene::draw_combat_scene(frame, chunks[4], game_state, ctx);

    // Loot ticker
    ticker::draw_ticker(frame, chunks[5], &game_state.loot_ticker);

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
    let zone_completion = stats_panel::compute_zone_completion(game_state);

    // At L tier, the right panel is narrower so zone lines wrap — give more height
    let zone_height = if ctx.tier >= SizeTier::XL { 7 } else { 8 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(zone_height), // Zone info (includes flavor text + wrap)
            Constraint::Min(10),             // Content (changes by activity)
        ])
        .split(area);

    // Zone info at top (always)
    stats_panel::draw_zone_info(
        frame,
        chunks[0],
        game_state,
        &zone_completion,
        achievements,
        ctx,
    );

    // Content area — dispatched by current activity
    draw_right_content(frame, chunks[1], game_state, ctx);
}

/// Draws the main content area of the right panel based on current activity.
/// Priority: minigame > challenge menu > fishing > dungeon > combat
fn draw_right_content(frame: &mut Frame, area: Rect, game_state: &GameState, ctx: &LayoutContext) {
    match &game_state.active_minigame {
        Some(ActiveMinigame::Rune(game)) => {
            rune_scene::render_rune(frame, area, game, ctx);
        }
        Some(ActiveMinigame::Minesweeper(game)) => {
            minesweeper_scene::render_minesweeper(frame, area, game, ctx);
        }
        Some(ActiveMinigame::Gomoku(game)) => {
            gomoku_scene::render_gomoku_scene(frame, area, game, ctx);
        }
        Some(ActiveMinigame::Morris(game)) => {
            morris_scene::render_morris_scene(frame, area, game, game_state.character_level, ctx);
        }
        Some(ActiveMinigame::Chess(game)) => {
            chess_scene::render_chess_scene(frame, area, game, ctx);
        }
        Some(ActiveMinigame::Go(game)) => {
            go_scene::render_go_scene(frame, area, game, ctx);
        }
        Some(ActiveMinigame::FlappyBird(game)) => {
            flappy_scene::render_flappy_scene(frame, area, game, ctx);
        }
        Some(ActiveMinigame::Jezzball(game)) => {
            jezzball_scene::render_jezzball_scene(frame, area, game, ctx);
        }
        Some(ActiveMinigame::Snake(game)) => {
            snake_scene::render_snake_scene(frame, area, game, ctx);
        }
        Some(ActiveMinigame::RunicShift(game)) => {
            runic_shift_scene::render_runic_shift_scene(frame, area, game, ctx);
        }
        None => {
            if game_state.challenge_menu.is_open {
                challenge_menu_scene::render_challenge_menu(
                    frame,
                    area,
                    &game_state.challenge_menu,
                    ctx,
                );
            } else if let Some(ref session) = game_state.active_fishing {
                fishing_scene::render_fishing_scene(frame, area, session, &game_state.fishing, ctx);
            } else if let Some(dungeon) = &game_state.active_dungeon {
                draw_dungeon_view(frame, area, game_state, dungeon, ctx);
            } else {
                combat_scene::draw_combat_scene(frame, area, game_state, ctx);
            }
        }
    }
}

/// Draws the dungeon view with combat HUD overlay on the map.
/// Instead of splitting into separate dungeon map + combat panels,
/// combat info (HP bars, status) is rendered inside the dungeon panel.
fn draw_dungeon_view(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    dungeon: &crate::dungeon::types::Dungeon,
    _ctx: &LayoutContext,
) {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Single border wrapping everything
    let block = Block::default()
        .title(" Dungeon ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    frame.render_widget(block, area);

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

/// Draws a dark, mysterious animated backdrop for dungeon map areas.
fn draw_dungeon_backdrop(frame: &mut Frame, area: Rect, zone_id: u32) {
    use scene_fx::{current_millis, hash2d, lerp_rgb, put_cell, render_buffer, SceneCell};

    if area.width < 8 || area.height < 4 {
        return;
    }

    let width = area.width as usize;
    let height = area.height as usize;
    let mut buffer = vec![vec![SceneCell::default(); width]; height];

    let millis = current_millis() as f64;
    let phase = (millis / 620.0) as usize;

    let tint = match zone_id {
        1 | 2 => (8, 18, 8),
        3 | 6 => (10, 14, 26),
        4 | 7 => (20, 8, 28),
        5 | 10 => (24, 12, 8),
        8 | 9 => (6, 14, 30),
        11 => (26, 8, 18),
        _ => (12, 10, 14),
    };

    let top = (
        (14 + tint.0 / 3) as u8,
        (16 + tint.1 / 3) as u8,
        (28 + tint.2 / 3) as u8,
    );
    let bottom = (
        (7 + tint.0 / 5) as u8,
        (8 + tint.1 / 5) as u8,
        (14 + tint.2 / 5) as u8,
    );
    let mist_dim = (
        (36 + tint.0 / 2) as u8,
        (40 + tint.1 / 2) as u8,
        (58 + tint.2 / 2) as u8,
    );
    let mist_bright = (
        (54 + tint.0 / 2) as u8,
        (60 + tint.1 / 2) as u8,
        (84 + tint.2 / 2) as u8,
    );

    // Base gradient with very slow shimmer and a low "breathing" pulse.
    let gloom = ((millis / 12_000.0).sin() * 0.5 + 0.5) * 0.16;
    let pulse = (gloom * 12.0) as i16;
    for (row, row_cells) in buffer.iter_mut().enumerate() {
        let row_t = if height <= 1 {
            0.0
        } else {
            row as f64 / (height - 1) as f64
        };
        let base_rgb = lerp_rgb(top, bottom, row_t.powf(0.84));

        for (col, cell) in row_cells.iter_mut().enumerate() {
            let wave = (col as f64 * 0.08 + millis * 0.00011 + zone_id as f64 * 0.7).sin()
                + (row as f64 * 0.14 - millis * 0.00008 + zone_id as f64 * 0.41).cos();
            let jitter = ((wave * 1.6).round() as i16).clamp(-3, 4);
            let r = clamp_i16_to_u8(base_rgb.0 as i16 + jitter - pulse);
            let g = clamp_i16_to_u8(base_rgb.1 as i16 + jitter - pulse);
            let b = clamp_i16_to_u8(base_rgb.2 as i16 + jitter - pulse);
            cell.bg = Color::Rgb(r, g, b);
        }
    }

    // Faint, slow-moving shadow columns to make the scene feel deeper.
    let drift = (millis * 0.00004) as usize;
    for col in 0..width {
        if hash2d(zone_id as usize * 41 + col, drift).is_multiple_of(9) {
            for row_cells in buffer.iter_mut().take(height) {
                if let Some(cell) = row_cells.get_mut(col) {
                    if let Color::Rgb(r, g, b) = cell.bg {
                        cell.bg = Color::Rgb(
                            clamp_i16_to_u8(r as i16 - 6),
                            clamp_i16_to_u8(g as i16 - 6),
                            clamp_i16_to_u8(b as i16 - 6),
                        );
                    }
                }
            }
        }
    }

    // Drifting mist ribbons.
    let flow = millis * 0.00005;
    for band in 0..3usize {
        let anchor = (height as f64 * (0.24 + band as f64 * 0.27)).round();
        for col in 0..width {
            let wave =
                (col as f64 * 0.08 + flow * (1.0 + band as f64 * 0.12) + band as f64 * 0.8).sin();
            let row = (anchor + wave * (height as f64 * 0.03)) as i32;
            if row < 0 || row >= height as i32 {
                continue;
            }

            if hash2d(
                row as usize + phase + band * 11,
                col + zone_id as usize * 17,
            )
            .is_multiple_of(7)
            {
                let glyph = if band == 1 { '\'' } else { ':' };
                put_cell(
                    &mut buffer,
                    row,
                    col as i32,
                    glyph,
                    Color::Rgb(mist_dim.0, mist_dim.1, mist_dim.2),
                );
            }
        }
    }

    // Sparse arcane specks (low flicker rate).
    for row in 0..height {
        for col in 0..width {
            let seed = hash2d(row + zone_id as usize * 23, col + zone_id as usize * 37);
            if !seed.is_multiple_of(257) {
                continue;
            }
            let bright = hash2d(row + phase / 2, col + phase / 2).is_multiple_of(4);
            let fg = if bright {
                Color::Rgb(mist_bright.0, mist_bright.1, mist_bright.2)
            } else {
                Color::Rgb(mist_dim.0, mist_dim.1, mist_dim.2)
            };
            let ch = if bright { '*' } else { ':' };
            put_cell(&mut buffer, row as i32, col as i32, ch, fg);
        }
    }

    render_buffer(frame, area, &buffer);
}

fn clamp_i16_to_u8(value: i16) -> u8 {
    value.clamp(0, 255) as u8
}
