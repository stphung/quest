use crate::character::manager::CharacterInfo;
use crate::character::prestige::get_prestige_tier;
use crate::items::types::EquipmentSlot;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[allow(dead_code)]
pub struct CharacterSelectScreen {
    pub selected_index: usize,
}

#[allow(dead_code)]
impl CharacterSelectScreen {
    pub fn new() -> Self {
        Self { selected_index: 0 }
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, characters: &[CharacterInfo]) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Controls
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Select Your Hero")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Main content - split horizontally
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Character list
                Constraint::Percentage(60), // Details panel
            ])
            .split(chunks[1]);

        // Draw character list
        self.draw_character_list(f, main_chunks[0], characters);

        // Draw character details
        self.draw_character_details(f, main_chunks[1], characters);

        // Controls
        let new_button = if characters.len() >= 3 {
            "[N] New (Max 3)"
        } else {
            "[N] New"
        };
        let controls = Paragraph::new(format!(
            "[Enter] Play    [R] Rename    [D] Delete    {}    [Q] Quit",
            new_button
        ))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
        f.render_widget(controls, chunks[2]);
    }

    fn draw_character_list(&self, f: &mut Frame, area: Rect, characters: &[CharacterInfo]) {
        let block = Block::default().borders(Borders::ALL).title("Characters");

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        if characters.is_empty() {
            let empty_message = Paragraph::new("No characters yet.\nPress [N] to create one.")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            f.render_widget(empty_message, inner_area);
            return;
        }

        let mut lines = Vec::new();

        for (i, character) in characters.iter().enumerate() {
            let is_selected = i == self.selected_index;

            let prestige_name = get_prestige_tier(character.prestige_rank).name;

            let text = if character.is_corrupted {
                format!("{} (CORRUPTED)", character.filename)
            } else {
                format!(
                    "{} (Lv {} {})",
                    character.character_name, character.character_level, prestige_name
                )
            };

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(Span::styled(text, style)));
            lines.push(Line::from("")); // Empty line for spacing
        }

        let list_widget = Paragraph::new(lines);
        f.render_widget(list_widget, inner_area);
    }

    fn draw_character_details(&self, f: &mut Frame, area: Rect, characters: &[CharacterInfo]) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Character Details");

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        if characters.is_empty() {
            return;
        }

        let character = match characters.get(self.selected_index) {
            Some(c) => c,
            None => return,
        };

        if character.is_corrupted {
            let corrupted_message = Paragraph::new(vec![
                Line::from(Span::styled(
                    "CORRUPTED SAVE FILE",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("This save file is corrupted or tampered."),
                Line::from("You can delete it with [D]."),
            ])
            .alignment(Alignment::Center);
            f.render_widget(corrupted_message, inner_area);
            return;
        }

        let prestige_name = get_prestige_tier(character.prestige_rank).name;

        // Format playtime
        let hours = character.play_time_seconds / 3600;
        let minutes = (character.play_time_seconds % 3600) / 60;
        let playtime_str = if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        };

        let mut lines = vec![
            Line::from(Span::styled(
                &character.character_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("Level: {}", character.character_level)),
            Line::from(format!("Prestige: {}", prestige_name)),
            Line::from(format!("Playtime: {}", playtime_str)),
            Line::from(""),
            Line::from(Span::styled(
                "Attributes:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ];

        // Display attributes in 2 rows of 3
        let attrs = &character.attributes;
        lines.push(Line::from(format!(
            "STR: {}  DEX: {}  CON: {}",
            attrs.get(crate::character::attributes::AttributeType::Strength),
            attrs.get(crate::character::attributes::AttributeType::Dexterity),
            attrs.get(crate::character::attributes::AttributeType::Constitution)
        )));
        lines.push(Line::from(format!(
            "INT: {}  WIS: {}  CHA: {}",
            attrs.get(crate::character::attributes::AttributeType::Intelligence),
            attrs.get(crate::character::attributes::AttributeType::Wisdom),
            attrs.get(crate::character::attributes::AttributeType::Charisma)
        )));

        // Equipment section
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Equipment:",
            Style::default().add_modifier(Modifier::BOLD),
        )));

        let equipment_count = character.equipment.iter_equipped().count();
        lines.push(Line::from(format!("Equipped: {} / 7", equipment_count)));

        // List equipped items with emojis
        let slots_with_emojis = [
            (EquipmentSlot::Weapon, "⚔️"),
            (EquipmentSlot::Armor, "🛡"),
            (EquipmentSlot::Helmet, "🪖"),
            (EquipmentSlot::Gloves, "🧤"),
            (EquipmentSlot::Boots, "👢"),
            (EquipmentSlot::Amulet, "📿"),
            (EquipmentSlot::Ring, "💍"),
        ];

        for (slot, emoji) in slots_with_emojis {
            if let Some(item) = character.equipment.get(slot) {
                lines.push(Line::from(format!("{} {}", emoji, item.display_name)));
            }
        }

        let details_widget = Paragraph::new(lines);
        f.render_widget(details_widget, inner_area);
    }

    pub fn move_up(&mut self, characters: &[CharacterInfo]) {
        if !characters.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self, characters: &[CharacterInfo]) {
        if !characters.is_empty() && self.selected_index < characters.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn get_selected_character<'a>(
        &self,
        characters: &'a [CharacterInfo],
    ) -> Option<&'a CharacterInfo> {
        characters.get(self.selected_index)
    }
}
