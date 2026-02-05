//! Dungeon map visualization with fog of war.

#![allow(dead_code)]

use crate::dungeon::types::{Dungeon, RoomState, RoomType, DIR_DOWN, DIR_RIGHT};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

/// Symbols used for dungeon map rendering (emojis are 2 cells wide)
mod symbols {
    pub const PLAYER: &str = "🧙";
    pub const PLAYER_TRAVEL: &str = "🏃"; // Traveling through cleared rooms
    pub const ENTRANCE: &str = "🚪";
    pub const COMBAT: &str = "💀";
    pub const TREASURE: &str = "💎";
    pub const ELITE: &str = "🗝️";
    pub const BOSS: &str = "👹";
    pub const CLEARED: &str = "✓ ";
    pub const HIDDEN: &str = "  "; // Not visible at all
    pub const UNEXPLORED: &str = "❓"; // Revealed but not entered
    pub const H_CORRIDOR: &str = "──";
    pub const V_CORRIDOR: &str = "│ ";
}

/// Widget for rendering the dungeon map
pub struct DungeonMapWidget<'a> {
    dungeon: &'a Dungeon,
    /// Animation tick for blinking effects (0.0 to 1.0)
    blink_phase: f64,
}

impl<'a> DungeonMapWidget<'a> {
    pub fn new(dungeon: &'a Dungeon, blink_phase: f64) -> Self {
        Self {
            dungeon,
            blink_phase,
        }
    }

    /// Returns the symbol and style for a room
    fn room_display(
        &self,
        room_type: RoomType,
        state: RoomState,
        is_current: bool,
    ) -> (&'static str, Style) {
        // Blinking effect for current position
        let blink_visible = self.blink_phase < 0.5;

        if is_current && blink_visible {
            // Show running symbol when traveling, wizard when exploring/fighting
            let symbol = if self.dungeon.is_traveling {
                symbols::PLAYER_TRAVEL
            } else {
                symbols::PLAYER
            };
            let color = if self.dungeon.is_traveling {
                Color::Cyan // Different color for traveling
            } else {
                Color::Yellow
            };
            return (
                symbol,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            );
        }

        match state {
            // Not yet visited - show that a room exists but not what type
            // Exception: Boss room is always visible so players know the goal
            RoomState::Hidden | RoomState::Revealed => {
                if room_type == RoomType::Boss {
                    (symbols::BOSS, Style::default().fg(Color::DarkGray))
                } else {
                    (symbols::UNEXPLORED, Style::default().fg(Color::DarkGray))
                }
            }
            // Currently in this room - show what it is
            RoomState::Current => {
                let (sym, color) = match room_type {
                    RoomType::Entrance => (symbols::ENTRANCE, Color::Green),
                    RoomType::Combat => (symbols::COMBAT, Color::Red),
                    RoomType::Treasure => (symbols::TREASURE, Color::Yellow),
                    RoomType::Elite => (symbols::ELITE, Color::Magenta),
                    RoomType::Boss => (symbols::BOSS, Color::LightRed),
                };
                (sym, Style::default().fg(color).add_modifier(Modifier::BOLD))
            }
            // Already visited - show what it was (dimmed)
            RoomState::Cleared => {
                let (sym, color) = match room_type {
                    RoomType::Entrance => (symbols::ENTRANCE, Color::DarkGray),
                    RoomType::Combat => (symbols::CLEARED, Color::DarkGray),
                    RoomType::Treasure => (symbols::TREASURE, Color::DarkGray),
                    RoomType::Elite => (symbols::ELITE, Color::DarkGray),
                    RoomType::Boss => (symbols::BOSS, Color::DarkGray),
                };
                (sym, Style::default().fg(color))
            }
        }
    }
}

impl Widget for DungeonMapWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let grid_size = self.dungeon.size.grid_size();

        // Calculate cell size (emojis are 2 chars wide, so 4 chars per cell, 2 rows tall)
        let cell_width: u16 = 4;
        let cell_height: u16 = 2;

        // Calculate total map size
        let map_width = grid_size as u16 * cell_width;
        let map_height = grid_size as u16 * cell_height;

        // Center the map in the area
        let start_x = area.x + area.width.saturating_sub(map_width) / 2;
        let start_y = area.y + area.height.saturating_sub(map_height) / 2;

        // Render each cell
        for gy in 0..grid_size {
            for gx in 0..grid_size {
                let screen_x = start_x + (gx as u16) * cell_width;
                let screen_y = start_y + (gy as u16) * cell_height;

                // Skip if outside render area
                if screen_x >= area.x + area.width || screen_y >= area.y + area.height {
                    continue;
                }

                if let Some(room) = self.dungeon.get_room(gx, gy) {
                    let is_current = self.dungeon.player_position == (gx, gy);
                    let (sym, style) = self.room_display(room.room_type, room.state, is_current);

                    // Render room symbol (emoji takes 2 cells)
                    let rx = screen_x;
                    let ry = screen_y;
                    if rx + 1 < area.x + area.width && ry < area.y + area.height {
                        buf.get_mut(rx, ry).set_symbol(sym).set_style(style);
                    }

                    // Render corridors (always show full dungeon layout)
                    {
                        let corridor_style = Style::default().fg(Color::DarkGray);

                        // Right corridor
                        if room.connections[DIR_RIGHT] {
                            let cx = screen_x + 2;
                            if cx + 1 < area.x + area.width && ry < area.y + area.height {
                                buf.get_mut(cx, ry)
                                    .set_symbol(symbols::H_CORRIDOR)
                                    .set_style(corridor_style);
                            }
                        }

                        // Down corridor
                        if room.connections[DIR_DOWN] {
                            let cy = screen_y + 1;
                            if rx + 1 < area.x + area.width && cy < area.y + area.height {
                                buf.get_mut(rx, cy)
                                    .set_symbol(symbols::V_CORRIDOR)
                                    .set_style(corridor_style);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Renders dungeon status information (key, rooms cleared, etc.)
pub struct DungeonStatusWidget<'a> {
    dungeon: &'a Dungeon,
}

impl<'a> DungeonStatusWidget<'a> {
    pub fn new(dungeon: &'a Dungeon) -> Self {
        Self { dungeon }
    }
}

impl Widget for DungeonStatusWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        let size_name = match self.dungeon.size {
            crate::dungeon::types::DungeonSize::Small => "Small",
            crate::dungeon::types::DungeonSize::Medium => "Medium",
            crate::dungeon::types::DungeonSize::Large => "Large",
            crate::dungeon::types::DungeonSize::Epic => "Epic",
            crate::dungeon::types::DungeonSize::Legendary => "Legendary",
        };

        let key_status = if self.dungeon.has_key {
            "[KEY]"
        } else {
            "[---]"
        };

        let status = format!(
            "{} Dungeon | Rooms: {}/{} | {}",
            size_name,
            self.dungeon.rooms_cleared,
            self.dungeon.room_count(),
            key_status
        );

        let key_style = if self.dungeon.has_key {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Render the status line
        let x = area.x;
        let y = area.y;

        // Find where [KEY] or [---] starts in the string
        let key_start = status.find('[').unwrap_or(status.len());

        for (i, ch) in status.chars().enumerate() {
            if x + i as u16 >= area.x + area.width {
                break;
            }

            let style = if i >= key_start {
                key_style
            } else {
                Style::default().fg(Color::White)
            };

            buf.get_mut(x + i as u16, y).set_char(ch).set_style(style);
        }
    }
}

/// Legend showing what each symbol means
pub struct DungeonLegendWidget;

impl Widget for DungeonLegendWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let legends = [
            (symbols::PLAYER, "You", Color::Yellow),
            (symbols::UNEXPLORED, "Unexplored", Color::DarkGray),
            (symbols::ENTRANCE, "Entrance", Color::Green),
            (symbols::COMBAT, "Combat", Color::Red),
            (symbols::TREASURE, "Treasure", Color::Yellow),
            (symbols::ELITE, "Key Guardian", Color::Magenta),
            (symbols::BOSS, "Boss", Color::LightRed),
            (symbols::CLEARED, "Cleared", Color::DarkGray),
        ];

        let mut y = area.y;
        for (sym, label, color) in legends {
            if y >= area.y + area.height {
                break;
            }

            // Render symbol (emoji, 2 cells wide)
            buf.get_mut(area.x, y)
                .set_symbol(sym)
                .set_style(Style::default().fg(color).add_modifier(Modifier::BOLD));

            // Render label (offset by 3 to account for emoji width + space)
            let label_start = area.x + 3;
            for (i, c) in label.chars().enumerate() {
                if label_start + i as u16 >= area.x + area.width {
                    break;
                }
                buf.get_mut(label_start + i as u16, y)
                    .set_char(c)
                    .set_style(Style::default().fg(Color::White));
            }

            y += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::generation::generate_dungeon;

    #[test]
    fn test_dungeon_map_widget_creation() {
        let dungeon = generate_dungeon(10, 0);
        let widget = DungeonMapWidget::new(&dungeon, 0.0);
        assert!((widget.blink_phase - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_room_display_unvisited() {
        let dungeon = generate_dungeon(10, 0);
        let widget = DungeonMapWidget::new(&dungeon, 0.0);
        // Unvisited rooms (Hidden or Revealed) show as unexplored
        let (sym, _style) = widget.room_display(RoomType::Combat, RoomState::Hidden, false);
        assert_eq!(sym, symbols::UNEXPLORED);
        let (sym, _style) = widget.room_display(RoomType::Combat, RoomState::Revealed, false);
        assert_eq!(sym, symbols::UNEXPLORED);
    }

    #[test]
    fn test_room_display_current_blink() {
        let dungeon = generate_dungeon(10, 0);

        // Blink visible (phase < 0.5)
        let widget = DungeonMapWidget::new(&dungeon, 0.25);
        let (sym, _) = widget.room_display(RoomType::Combat, RoomState::Current, true);
        assert_eq!(sym, symbols::PLAYER);

        // Blink hidden (phase >= 0.5)
        let widget = DungeonMapWidget::new(&dungeon, 0.75);
        let (sym, _) = widget.room_display(RoomType::Combat, RoomState::Current, true);
        assert_eq!(sym, symbols::COMBAT);
    }

    #[test]
    fn test_dungeon_status_widget() {
        let dungeon = generate_dungeon(10, 0);
        let _widget = DungeonStatusWidget::new(&dungeon);
        // Widget creation should not panic
    }
}
