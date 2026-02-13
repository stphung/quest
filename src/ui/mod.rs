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
mod info_panel;
pub mod minesweeper_scene;
pub mod morris_scene;
pub mod prestige_confirm;
pub mod rune_scene;
pub mod snake_scene;
mod stats_panel;
mod throbber;

use crate::challenges::ActiveMinigame;
use crate::core::game_state::GameState;
use crate::utils::updater::UpdateInfo;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Main UI drawing function with optional update notification
pub fn draw_ui_with_update(
    frame: &mut Frame,
    game_state: &GameState,
    update_info: Option<&UpdateInfo>,
    update_expanded: bool,
    update_check_completed: bool,
    haven_discovered: bool,
    achievements: &crate::achievements::Achievements,
) {
    let size = frame.size();

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

        draw_challenge_banner(frame, v_chunks[0], game_state);
        v_chunks[1]
    } else {
        size
    };

    // Determine if we need space for update drawer
    let show_update_drawer = update_expanded && update_info.is_some();

    // Split vertically: main content, full-width info panels, optional update drawer, footer
    let v_chunks = if show_update_drawer {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),     // Main content (stats + right panel)
                Constraint::Length(8),  // Full-width Loot + Combat
                Constraint::Length(12), // Update drawer panel (taller for changelog)
                Constraint::Length(3),  // Full-width footer
            ])
            .split(main_area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Main content (stats + right panel)
                Constraint::Length(8), // Full-width Loot + Combat
                Constraint::Length(3), // Full-width footer
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
    stats_panel::draw_stats_panel(frame, chunks[0], game_state);

    // Draw full-width Loot + Combat panels
    info_panel::draw_info_panel(frame, info_area, game_state);

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
        achievements.pending_count(),
    );

    // Draw right panel with stable layout: zone info + content + info panel
    draw_right_panel(frame, chunks[1], game_state, achievements);
}

/// Draws the challenge notification banner at the top of the screen
fn draw_challenge_banner(frame: &mut Frame, area: Rect, game_state: &GameState) {
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
) {
    let zone_completion = stats_panel::compute_zone_completion(game_state);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Zone info (always visible)
            Constraint::Min(10),   // Content (changes by activity)
        ])
        .split(area);

    // Zone info at top (always)
    stats_panel::draw_zone_info(frame, chunks[0], game_state, &zone_completion, achievements);

    // Content area — dispatched by current activity
    draw_right_content(frame, chunks[1], game_state);
}

/// Draws the main content area of the right panel based on current activity.
/// Priority: minigame > challenge menu > fishing > dungeon > combat
fn draw_right_content(frame: &mut Frame, area: Rect, game_state: &GameState) {
    match &game_state.active_minigame {
        Some(ActiveMinigame::Rune(game)) => {
            rune_scene::render_rune(frame, area, game);
        }
        Some(ActiveMinigame::Minesweeper(game)) => {
            minesweeper_scene::render_minesweeper(frame, area, game);
        }
        Some(ActiveMinigame::Gomoku(game)) => {
            gomoku_scene::render_gomoku_scene(frame, area, game);
        }
        Some(ActiveMinigame::Morris(game)) => {
            morris_scene::render_morris_scene(frame, area, game, game_state.character_level);
        }
        Some(ActiveMinigame::Chess(game)) => {
            chess_scene::render_chess_scene(frame, area, game);
        }
        Some(ActiveMinigame::Go(game)) => {
            go_scene::render_go_scene(frame, area, game);
        }
        Some(ActiveMinigame::FlappyBird(game)) => {
            flappy_scene::render_flappy_scene(frame, area, game);
        }
        Some(ActiveMinigame::Snake(game)) => {
            snake_scene::render_snake_scene(frame, area, game);
        }
        None => {
            if game_state.challenge_menu.is_open {
                challenge_menu_scene::render_challenge_menu(
                    frame,
                    area,
                    &game_state.challenge_menu,
                );
            } else if let Some(ref session) = game_state.active_fishing {
                fishing_scene::render_fishing_scene(frame, area, session, &game_state.fishing);
            } else if let Some(dungeon) = &game_state.active_dungeon {
                draw_dungeon_view(frame, area, game_state, dungeon);
            } else {
                combat_scene::draw_combat_scene(frame, area, game_state);
            }
        }
    }
}

/// Draws the dungeon view with map and combat
fn draw_dungeon_view(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    dungeon: &crate::dungeon::types::Dungeon,
) {
    // Dungeon map needs 2 rows per grid cell + 3 for border + status line
    let grid_size = dungeon.size.grid_size() as u16;
    let map_height = grid_size * 2 + 3;

    // Split: dungeon map gets what it needs, combat gets the rest
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(map_height), // Dungeon map (sized to fit)
            Constraint::Min(5),          // Combat scene (whatever remains)
        ])
        .split(area);

    // Draw dungeon map
    draw_dungeon_panel(frame, chunks[0], dungeon);

    // Draw combat scene
    combat_scene::draw_combat_scene(frame, chunks[1], game_state);
}

/// Draws the dungeon map panel
fn draw_dungeon_panel(frame: &mut Frame, area: Rect, dungeon: &crate::dungeon::types::Dungeon) {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Create border
    let block = Block::default()
        .title(" Dungeon ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split inner area for status and map
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Status line
            Constraint::Min(0),    // Map
        ])
        .split(inner);

    // Draw status
    let status_widget = dungeon_map::DungeonStatusWidget::new(dungeon);
    frame.render_widget(status_widget, inner_chunks[0]);

    // Calculate blink phase (0.5 second cycle)
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let blink_phase = (millis % 500) as f64 / 500.0;

    // Draw map
    let map_widget = dungeon_map::DungeonMapWidget::new(dungeon, blink_phase);
    frame.render_widget(map_widget, inner_chunks[1]);
}
