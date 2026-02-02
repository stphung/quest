pub mod character_creation;
pub mod character_delete;
pub mod character_rename;
pub mod character_select;
mod combat_3d;
pub mod combat_effects;
mod combat_scene;
mod enemy_sprites;
mod stats_panel;
pub mod zones;

use crate::game_state::GameState;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

/// Main UI drawing function that creates the layout and draws all components
pub fn draw_ui(frame: &mut Frame, game_state: &GameState) {
    let size = frame.size();

    // Split into two main areas: stats panel (left) and combat scene (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Stats panel
            Constraint::Percentage(50), // Combat scene
        ])
        .split(size);

    // Draw stats panel on the left
    stats_panel::draw_stats_panel(frame, chunks[0], game_state);

    // Draw combat scene on the right
    combat_scene::draw_combat_scene(frame, chunks[1], game_state);
}
