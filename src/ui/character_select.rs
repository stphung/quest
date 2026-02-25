use crate::character::manager::CharacterInfo;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub struct CharacterSelectScreen {
    pub selected_index: usize,
    /// Whether the first-launch cloud restore prompt is showing.
    pub cloud_restore_showing: bool,
    /// PAT input for the cloud restore prompt.
    pub cloud_restore_input: String,
    /// Repo name input for the cloud restore prompt.
    pub cloud_restore_repo: String,
    /// Which field is focused: 0 = token, 1 = repo name.
    pub cloud_restore_field: u8,
    /// Error message from a failed cloud restore attempt.
    pub cloud_restore_error: Option<String>,
    /// True while a link-and-pull operation is in flight.
    pub cloud_restore_in_flight: bool,
    /// Set to true once the user has dismissed the prompt (so it doesn't reappear).
    pub cloud_restore_dismissed: bool,
}

impl CharacterSelectScreen {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            cloud_restore_showing: false,
            cloud_restore_input: String::new(),
            cloud_restore_repo: crate::history::cloud::DEFAULT_REPO_NAME.to_string(),
            cloud_restore_field: 0,
            cloud_restore_error: None,
            cloud_restore_in_flight: false,
            cloud_restore_dismissed: false,
        }
    }

    /// Draw the first-launch cloud restore prompt overlay.
    pub fn draw_cloud_restore_prompt(&self, f: &mut Frame, area: Rect) {
        let dialog_w = 56u16.min(area.width.saturating_sub(4));
        let dialog_h = if self.cloud_restore_error.is_some() {
            17u16
        } else {
            16u16
        }
        .min(area.height.saturating_sub(2));

        let x = area.x + (area.width.saturating_sub(dialog_w)) / 2;
        let y = area.y + (area.height.saturating_sub(dialog_h)) / 2;
        let dialog_area = Rect::new(x, y, dialog_w, dialog_h);

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(dialog_area);
        f.render_widget(block, dialog_area);

        let mut lines: Vec<Line<'_>> = Vec::new();

        lines.push(Line::from(Span::styled(
            "Restore saves from GitHub?",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        if self.cloud_restore_in_flight {
            lines.push(Line::from(Span::styled(
                "Linking...",
                Style::default().fg(Color::Cyan),
            )));
        } else {
            // PAT creation instructions.
            lines.push(Line::from(vec![
                Span::styled("1. ", Style::default().fg(Color::DarkGray)),
                Span::styled("Go to ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "github.com/settings/tokens",
                    Style::default().fg(Color::Cyan),
                ),
            ]));
            lines.push(Line::from(Span::styled(
                "   Tokens (classic) > Generate",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(vec![
                Span::styled("2. ", Style::default().fg(Color::DarkGray)),
                Span::styled("Select scope: ", Style::default().fg(Color::DarkGray)),
                Span::styled("repo", Style::default().fg(Color::Yellow)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("3. ", Style::default().fg(Color::DarkGray)),
                Span::styled("Fill in fields below", Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(""));

            // Token input.
            let token_color = if self.cloud_restore_field == 0 {
                Color::Yellow
            } else {
                Color::DarkGray
            };
            let raw = &self.cloud_restore_input;
            let token_val = if self.cloud_restore_field == 0 {
                if raw.len() <= 4 {
                    format!("{}_", raw)
                } else {
                    let dots: String = "\u{2022}".repeat(raw.len() - 4);
                    format!("{}{}_", dots, &raw[raw.len() - 4..])
                }
            } else if raw.len() <= 4 {
                raw.clone()
            } else {
                let dots: String = "\u{2022}".repeat(raw.len() - 4);
                format!("{}{}", dots, &raw[raw.len() - 4..])
            };
            lines.push(Line::from(vec![
                Span::styled("Token: ", Style::default().fg(token_color)),
                Span::styled(token_val, Style::default().fg(token_color)),
            ]));

            // Repo name input.
            let repo_color = if self.cloud_restore_field == 1 {
                Color::Yellow
            } else {
                Color::DarkGray
            };
            let repo_val = if self.cloud_restore_field == 1 {
                format!("{}_", self.cloud_restore_repo)
            } else {
                self.cloud_restore_repo.clone()
            };
            lines.push(Line::from(vec![
                Span::styled("Repo:  ", Style::default().fg(repo_color)),
                Span::styled(repo_val, Style::default().fg(repo_color)),
            ]));

            if let Some(ref err) = self.cloud_restore_error {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    err.as_str(),
                    Style::default().fg(Color::Red),
                )));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
                Span::styled(" Link  ", Style::default().fg(Color::DarkGray)),
                Span::styled("[Tab]", Style::default().fg(Color::Cyan)),
                Span::styled(" Switch  ", Style::default().fg(Color::DarkGray)),
                Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
                Span::styled(" Skip", Style::default().fg(Color::DarkGray)),
            ]));
        }

        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
        f.render_widget(paragraph, inner);
    }

    pub fn move_up(&mut self, _characters: &[CharacterInfo]) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self, characters: &[CharacterInfo]) {
        // Allow selecting up to characters.len() when < 3 (the "+ Create" slot)
        let max_index = if characters.len() < 3 {
            characters.len()
        } else {
            characters.len() - 1
        };
        if self.selected_index < max_index {
            self.selected_index += 1;
        }
    }
}
