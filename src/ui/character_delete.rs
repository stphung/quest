use crate::character::manager::CharacterInfo;
use crate::character::prestige::get_prestige_tier;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[allow(dead_code)]
pub struct CharacterDeleteScreen {
    pub confirmation_input: String,
    pub cursor_position: usize,
}

#[allow(dead_code)]
impl CharacterDeleteScreen {
    pub fn new() -> Self {
        Self {
            confirmation_input: String::new(),
            cursor_position: 0,
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
                Constraint::Length(5), // Warning box
                Constraint::Length(1), // Spacer
                Constraint::Length(4), // Input label + field
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Controls
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Delete Character")
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Character details
        self.draw_character_details(f, chunks[2], character);

        // Warning box
        let warning_lines = vec![
            Line::from(Span::styled(
                "⚠ WARNING ⚠",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("This action is PERMANENT and IRREVERSIBLE."),
            Line::from("All progress will be lost forever."),
        ];
        let warning_widget = Paragraph::new(warning_lines)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            );
        f.render_widget(warning_widget, chunks[4]);

        // Input label
        let label = Paragraph::new(format!(
            "Type the character name '{}' to confirm deletion:",
            character.character_name
        ))
        .alignment(Alignment::Center);
        f.render_widget(label, chunks[6]);

        // Input field with cursor
        let input_area = Rect {
            x: chunks[6].x + (chunks[6].width.saturating_sub(50)) / 2,
            y: chunks[6].y + 1,
            width: 50.min(chunks[6].width),
            height: 3,
        };

        let input_text = {
            let char_count = self.confirmation_input.chars().count();
            if self.cursor_position < char_count {
                let chars: Vec<char> = self.confirmation_input.chars().collect();
                let before: String = chars[..self.cursor_position].iter().collect();
                let after: String = chars[self.cursor_position..].iter().collect();
                format!("{}{}{}", before, "_", after)
            } else {
                format!("{}_", self.confirmation_input)
            }
        };

        let input_widget = Paragraph::new(input_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        f.render_widget(input_widget, input_area);

        // Controls
        let controls = Paragraph::new("[Enter] Confirm Delete    [Esc] Cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(controls, chunks[8]);
    }

    fn draw_character_details(&self, f: &mut Frame, area: Rect, character: &CharacterInfo) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Character to Delete");

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
        let chars: Vec<char> = self.confirmation_input.chars().collect();
        let before: String = chars[..self.cursor_position].iter().collect();
        let after: String = chars[self.cursor_position..].iter().collect();
        self.confirmation_input = format!("{}{}{}", before, c, after);
        self.cursor_position += 1;
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            let chars: Vec<char> = self.confirmation_input.chars().collect();
            let before: String = chars[..self.cursor_position - 1].iter().collect();
            let after: String = chars[self.cursor_position..].iter().collect();
            self.confirmation_input = format!("{}{}", before, after);
            self.cursor_position -= 1;
        }
    }

    pub fn is_confirmed(&self, character_name: &str) -> bool {
        self.confirmation_input == character_name
    }

    pub fn reset(&mut self) {
        self.confirmation_input.clear();
        self.cursor_position = 0;
    }
}
