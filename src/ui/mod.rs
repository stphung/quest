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
pub mod gomoku_scene;
pub mod morris_scene;
pub mod prestige_confirm;
mod stats_panel;
mod throbber;

use crate::game_state::GameState;
use crate::updater::UpdateInfo;
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
    update_check_completed: bool,
) {
    let size = frame.size();

    // Check if we should show the challenge notification banner
    let show_challenge_banner = !game_state.challenge_menu.challenges.is_empty()
        && !game_state.challenge_menu.is_open
        && game_state.active_chess.is_none()
        && game_state.active_morris.is_none();

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

    // Split into two main areas: stats panel (left) and combat/dungeon (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Stats panel
            Constraint::Percentage(50), // Combat scene or dungeon
        ])
        .split(main_area);

    // Draw stats panel on the left (with optional update info)
    stats_panel::draw_stats_panel_with_update(
        frame,
        chunks[0],
        game_state,
        update_info,
        update_check_completed,
    );

    // Draw right panel based on current activity
    // Priority: morris > chess > challenge menu > fishing > dungeon > combat
    if let Some(ref game) = game_state.active_morris {
        morris_scene::render_morris_scene(frame, chunks[1], game, game_state.character_level);
    } else if let Some(ref game) = game_state.active_chess {
        chess_scene::render_chess_scene(frame, chunks[1], game);
    } else if game_state.challenge_menu.is_open {
        challenge_menu_scene::render_challenge_menu(frame, chunks[1], &game_state.challenge_menu);
    } else if let Some(ref session) = game_state.active_fishing {
        fishing_scene::render_fishing_scene(frame, chunks[1], session, &game_state.fishing);
    } else if let Some(dungeon) = &game_state.active_dungeon {
        draw_dungeon_view(frame, chunks[1], game_state, dungeon);
    } else {
        combat_scene::draw_combat_scene(frame, chunks[1], game_state);
    }
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

/// Draws the dungeon view with map and combat
fn draw_dungeon_view(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    dungeon: &crate::dungeon::Dungeon,
) {
    // Split into dungeon map (top) and combat (bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // Dungeon map
            Constraint::Percentage(60), // Combat scene
        ])
        .split(area);

    // Draw dungeon map
    draw_dungeon_panel(frame, chunks[0], dungeon);

    // Draw combat scene
    combat_scene::draw_combat_scene(frame, chunks[1], game_state);
}

/// Draws the dungeon map panel
fn draw_dungeon_panel(frame: &mut Frame, area: Rect, dungeon: &crate::dungeon::Dungeon) {
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
