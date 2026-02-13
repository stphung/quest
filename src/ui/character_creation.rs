use crate::ui::responsive::SizeTier;
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

    pub fn draw(&self, f: &mut Frame, area: Rect, ctx: &super::responsive::LayoutContext) {
        match ctx.tier {
            SizeTier::S | SizeTier::TooSmall => self.draw_small(f, area),
            SizeTier::M => self.draw_medium(f, area),
            _ => self.draw_large(f, area),
        }
    }

    fn draw_large(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(1), // Spacer
                Constraint::Length(4), // Input label + field
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
            height: 3,
        };

        self.render_input_field(f, input_area);

        // Rules
        let rules = vec![
            Line::from("• 1-16 characters"),
            Line::from("• Letters, numbers, spaces, hyphens, underscores"),
            Line::from("• Must be unique"),
        ];
        let rules_widget = Paragraph::new(rules).style(Style::default().fg(Color::Gray));
        f.render_widget(rules_widget, chunks[4]);

        // Validation feedback
        self.render_validation(f, chunks[5]);

        // Controls
        let controls = Paragraph::new("[Enter] Create Character    [Esc] Cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(controls, chunks[7]);
    }

    fn draw_medium(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Title
                Constraint::Length(4), // Input label + field
                Constraint::Length(3), // Rules
                Constraint::Length(1), // Validation
                Constraint::Min(0),    // Filler
                Constraint::Length(2), // Controls
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
        f.render_widget(label, chunks[1]);

        // Input field with cursor
        let input_area = Rect {
            x: chunks[1].x,
            y: chunks[1].y + 1,
            width: chunks[1].width,
            height: 3,
        };

        self.render_input_field(f, input_area);

        // Rules
        let rules = vec![
            Line::from("• 1-16 chars, letters/numbers/spaces/-/_"),
            Line::from("• Must be unique"),
        ];
        let rules_widget = Paragraph::new(rules).style(Style::default().fg(Color::Gray));
        f.render_widget(rules_widget, chunks[2]);

        // Validation
        self.render_validation(f, chunks[3]);

        // Controls
        let controls = Paragraph::new("[Enter] Create    [Esc] Cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(controls, chunks[5]);
    }

    fn draw_small(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(1)
            .vertical_margin(0)
            .constraints([
                Constraint::Length(1), // Title
                Constraint::Length(1), // Label
                Constraint::Length(3), // Input field
                Constraint::Length(1), // Validation
                Constraint::Length(1), // Rules hint
                Constraint::Min(0),    // Filler
                Constraint::Length(1), // Controls
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
        let label = Paragraph::new("Name:");
        f.render_widget(label, chunks[1]);

        // Input field
        self.render_input_field(f, chunks[2]);

        // Validation
        self.render_validation(f, chunks[3]);

        // Rules hint
        let rules =
            Paragraph::new("1-16 chars, unique").style(Style::default().fg(Color::DarkGray));
        f.render_widget(rules, chunks[4]);

        // Controls
        let controls = Paragraph::new("[Enter] Create  [Esc] Cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(controls, chunks[6]);
    }

    fn render_input_field(&self, f: &mut Frame, area: Rect) {
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
        f.render_widget(input_widget, area);
    }

    fn render_validation(&self, f: &mut Frame, area: Rect) {
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
        f.render_widget(validation_widget, area);
    }

    pub fn handle_char_input(&mut self, c: char) {
        let chars: Vec<char> = self.name_input.chars().collect();
        let before: String = chars[..self.cursor_position].iter().collect();
        let after: String = chars[self.cursor_position..].iter().collect();
        self.name_input = format!("{}{}{}", before, c, after);
        self.cursor_position += 1;
        self.validate();
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            let chars: Vec<char> = self.name_input.chars().collect();
            let before: String = chars[..self.cursor_position - 1].iter().collect();
            let after: String = chars[self.cursor_position..].iter().collect();
            self.name_input = format!("{}{}", before, after);
            self.cursor_position -= 1;
            self.validate();
        }
    }

    pub fn validate(&mut self) {
        self.validation_error = crate::character::manager::validate_name(&self.name_input).err();
    }

    pub fn is_valid(&self) -> bool {
        self.validation_error.is_none() && !self.name_input.trim().is_empty()
    }

    pub fn get_name(&self) -> String {
        self.name_input.trim().to_string()
    }
}
