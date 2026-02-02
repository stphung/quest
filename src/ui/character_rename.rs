use crate::character_manager::CharacterInfo;
use crate::prestige::get_prestige_tier;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[allow(dead_code)]
pub struct CharacterRenameScreen {
    pub new_name_input: String,
    pub cursor_position: usize,
    pub validation_error: Option<String>,
}

#[allow(dead_code)]
impl CharacterRenameScreen {
    pub fn new() -> Self {
        Self {
            new_name_input: String::new(),
            cursor_position: 0,
            validation_error: None,
        }
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, character: &CharacterInfo) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(1), // Spacer
                Constraint::Min(0),    // Character details
                Constraint::Length(1), // Spacer
                Constraint::Length(4), // Input label + field
                Constraint::Length(1), // Spacer
                Constraint::Length(4), // Rules
                Constraint::Length(2), // Validation
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Controls
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Rename Character")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Character details
        self.draw_character_details(f, chunks[2], character);

        // Input label
        let label = Paragraph::new(format!(
            "New name (currently: {}):",
            character.character_name
        ))
        .alignment(Alignment::Center);
        f.render_widget(label, chunks[4]);

        // Input field with cursor
        let input_area = Rect {
            x: chunks[4].x + (chunks[4].width.saturating_sub(50)) / 2,
            y: chunks[4].y + 1,
            width: 50.min(chunks[4].width),
            height: 3,
        };

        let input_text = {
            let char_count = self.new_name_input.chars().count();
            if self.cursor_position < char_count {
                let chars: Vec<char> = self.new_name_input.chars().collect();
                let before: String = chars[..self.cursor_position].iter().collect();
                let after: String = chars[self.cursor_position..].iter().collect();
                format!("{}{}{}", before, "_", after)
            } else {
                format!("{}_", self.new_name_input)
            }
        };

        let input_widget = Paragraph::new(input_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        f.render_widget(input_widget, input_area);

        // Rules
        let rules = vec![
            Line::from("• 1-16 characters"),
            Line::from("• Letters, numbers, spaces, hyphens, underscores"),
            Line::from("• Must be unique"),
        ];
        let rules_widget = Paragraph::new(rules)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(rules_widget, chunks[6]);

        // Validation feedback
        let validation_text = if let Some(error) = &self.validation_error {
            Line::from(Span::styled(
                format!("✗ {}", error),
                Style::default().fg(Color::Red),
            ))
        } else if !self.new_name_input.trim().is_empty() {
            Line::from(Span::styled(
                "✓ Name is valid",
                Style::default().fg(Color::Green),
            ))
        } else {
            Line::from("")
        };
        let validation_widget = Paragraph::new(validation_text).alignment(Alignment::Center);
        f.render_widget(validation_widget, chunks[7]);

        // Controls
        let controls = Paragraph::new("[Enter] Rename Character    [Esc] Cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(controls, chunks[9]);
    }

    fn draw_character_details(&self, f: &mut Frame, area: Rect, character: &CharacterInfo) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Character to Rename");

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let prestige_name = get_prestige_tier(character.prestige_rank).name;

        // Format playtime
        let hours = character.play_time_seconds / 3600;
        let minutes = (character.play_time_seconds % 3600) / 60;
        let playtime_str = if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        };

        let lines = vec![
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
        ];

        let details_widget = Paragraph::new(lines).alignment(Alignment::Center);
        f.render_widget(details_widget, inner_area);
    }

    pub fn handle_char_input(&mut self, c: char) {
        let chars: Vec<char> = self.new_name_input.chars().collect();
        let before: String = chars[..self.cursor_position].iter().collect();
        let after: String = chars[self.cursor_position..].iter().collect();
        self.new_name_input = format!("{}{}{}", before, c, after);
        self.cursor_position += 1;
        self.validate();
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            let chars: Vec<char> = self.new_name_input.chars().collect();
            let before: String = chars[..self.cursor_position - 1].iter().collect();
            let after: String = chars[self.cursor_position..].iter().collect();
            self.new_name_input = format!("{}{}", before, after);
            self.cursor_position -= 1;
            self.validate();
        }
    }

    pub fn validate(&mut self) {
        self.validation_error = crate::character_manager::validate_name(&self.new_name_input).err();
    }

    pub fn is_valid(&self) -> bool {
        self.validation_error.is_none() && !self.new_name_input.trim().is_empty()
    }

    pub fn get_name(&self) -> String {
        self.new_name_input.trim().to_string()
    }

    pub fn reset(&mut self) {
        self.new_name_input.clear();
        self.cursor_position = 0;
        self.validation_error = None;
    }
}
