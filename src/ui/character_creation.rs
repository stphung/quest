use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[allow(dead_code)]
pub struct CharacterCreationScreen {
    pub name_input: String,
    pub cursor_position: usize,
    pub validation_error: Option<String>,
}

#[allow(dead_code)]
impl CharacterCreationScreen {
    pub fn new() -> Self {
        Self {
            name_input: String::new(),
            cursor_position: 0,
            validation_error: None,
        }
    }

    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Input label + field
                Constraint::Length(1), // Spacer
                Constraint::Length(4), // Rules
                Constraint::Length(2), // Validation
                Constraint::Min(0),    // Filler
                Constraint::Length(3), // Controls
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Create Your Hero")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Input label
        let label = Paragraph::new("Character Name:");
        f.render_widget(label, chunks[2]);

        // Input field with cursor
        let input_area = Rect {
            x: chunks[2].x,
            y: chunks[2].y + 1,
            width: chunks[2].width,
            height: 1,
        };

        let input_text = {
            let char_count = self.name_input.chars().count();
            if self.cursor_position < char_count {
                let chars: Vec<char> = self.name_input.chars().collect();
                let before: String = chars[..self.cursor_position].iter().collect();
                let after: String = chars[self.cursor_position..].iter().collect();
                format!("{}{}{}", before, "_", after)
            } else {
                format!("{}_", self.name_input)
            }
        };

        let input_widget = Paragraph::new(input_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        f.render_widget(input_widget, input_area);

        // Rules
        let rules = vec![
            Line::from("• 1-16 characters"),
            Line::from("• Letters, numbers, spaces, hyphens, underscores"),
            Line::from("• Must be unique"),
        ];
        let rules_widget = Paragraph::new(rules).style(Style::default().fg(Color::Gray));
        f.render_widget(rules_widget, chunks[4]);

        // Validation feedback
        let validation_text = if let Some(error) = &self.validation_error {
            Line::from(Span::styled(
                format!("✗ {}", error),
                Style::default().fg(Color::Red),
            ))
        } else if !self.name_input.trim().is_empty() {
            Line::from(Span::styled(
                "✓ Name is valid",
                Style::default().fg(Color::Green),
            ))
        } else {
            Line::from("")
        };
        let validation_widget = Paragraph::new(validation_text);
        f.render_widget(validation_widget, chunks[5]);

        // Controls
        let controls = Paragraph::new("[Enter] Create Character    [Esc] Cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(controls, chunks[7]);
    }

    pub fn handle_char_input(&mut self, c: char) {
        self.name_input.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.validate();
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            self.name_input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
            self.validate();
        }
    }

    pub fn validate(&mut self) {
        self.validation_error = crate::character_manager::validate_name(&self.name_input).err();
    }

    pub fn is_valid(&self) -> bool {
        self.validation_error.is_none() && !self.name_input.trim().is_empty()
    }

    pub fn get_name(&self) -> String {
        self.name_input.trim().to_string()
    }
}
