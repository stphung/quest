//! Challenge menu UI rendering.

use crate::challenges::chess::ChessDifficulty;
use crate::challenges::flappy::FlappyBirdDifficulty;
use crate::challenges::go::GoDifficulty;
use crate::challenges::gomoku::GomokuDifficulty;
use crate::challenges::jezzball::JezzballDifficulty;
use crate::challenges::menu::{ChallengeMenu, ChallengeType, DifficultyInfo};
use crate::challenges::minesweeper::MinesweeperDifficulty;
use crate::challenges::morris::MorrisDifficulty;
use crate::challenges::rune::RuneDifficulty;
use crate::challenges::snake::SnakeDifficulty;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Render the challenge menu (list view or detail view)
pub fn render_challenge_menu(
    frame: &mut Frame,
    area: Rect,
    menu: &ChallengeMenu,
    _ctx: &super::responsive::LayoutContext,
) {
    frame.render_widget(Clear, area);

    if menu.viewing_detail && !menu.challenges.is_empty() {
        render_detail_view(frame, area, menu);
    } else {
        render_list_view(frame, area, menu);
    }
}

fn render_list_view(frame: &mut Frame, area: Rect, menu: &ChallengeMenu) {
    let block = Block::default()
        .title(" Pending Challenges ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if menu.challenges.is_empty() {
        let text =
            Paragraph::new("No pending challenges.").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, inner);
        return;
    }

    let items: Vec<ListItem> = menu
        .challenges
        .iter()
        .enumerate()
        .map(|(i, challenge)| {
            let prefix = if i == menu.selected_index { "> " } else { "  " };
            let style = if i == menu.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!("{}{} {}", prefix, challenge.icon, challenge.title)).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);

    // Help text at bottom
    if inner.height > 3 {
        let help_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        let help = Paragraph::new("[↑/↓] Navigate  [Enter] View  [Tab/Esc] Close")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, help_area);
    }
}

fn render_detail_view(frame: &mut Frame, area: Rect, menu: &ChallengeMenu) {
    let challenge = &menu.challenges[menu.selected_index];

    let block = Block::default()
        .title(format!(" {} ", challenge.title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split into body + help so body can flex while help stays anchored.
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    const DIFFICULTY_HEIGHT: u16 = 12;
    const TAIL_HEIGHT_WITHOUT_DIFFICULTY: u16 = 3; // spacer + spacer + outcomes

    // Size description to actual wrapped text height so difficulty options sit
    // directly below it, instead of being visually pinned near the bottom.
    let wrapped_lines =
        estimate_wrapped_line_count(&challenge.description, outer_chunks[0].width.max(1));
    let max_desc_for_full_difficulty = outer_chunks[0]
        .height
        .saturating_sub(DIFFICULTY_HEIGHT + TAIL_HEIGHT_WITHOUT_DIFFICULTY)
        .max(1);
    let description_height = wrapped_lines.clamp(1, max_desc_for_full_difficulty);

    let difficulty_height = DIFFICULTY_HEIGHT.min(
        outer_chunks[0]
            .height
            .saturating_sub(description_height + TAIL_HEIGHT_WITHOUT_DIFFICULTY),
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(description_height), // Description wraps naturally
            Constraint::Length(1),                  // Spacer
            Constraint::Length(difficulty_height),  // Difficulty selector
            Constraint::Length(1),                  // Spacer
            Constraint::Length(1),                  // Outcomes
        ])
        .split(outer_chunks[0]);

    // Description (with text wrapping)
    let desc = Paragraph::new(challenge.description.clone())
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(desc, chunks[0]);

    // Difficulty selector
    match challenge.challenge_type {
        ChallengeType::Chess => {
            render_difficulty_selector(
                frame,
                chunks[2],
                &ChessDifficulty::ALL,
                menu.selected_difficulty,
            );
        }
        ChallengeType::Morris => {
            render_difficulty_selector(
                frame,
                chunks[2],
                &MorrisDifficulty::ALL,
                menu.selected_difficulty,
            );
        }
        ChallengeType::Gomoku => {
            render_difficulty_selector(
                frame,
                chunks[2],
                &GomokuDifficulty::ALL,
                menu.selected_difficulty,
            );
        }
        ChallengeType::Minesweeper => {
            render_difficulty_selector(
                frame,
                chunks[2],
                &MinesweeperDifficulty::ALL,
                menu.selected_difficulty,
            );
        }
        ChallengeType::Rune => {
            render_difficulty_selector(
                frame,
                chunks[2],
                &RuneDifficulty::ALL,
                menu.selected_difficulty,
            );
        }
        ChallengeType::Go => {
            render_difficulty_selector(
                frame,
                chunks[2],
                &GoDifficulty::ALL,
                menu.selected_difficulty,
            );
        }
        ChallengeType::FlappyBird => {
            render_difficulty_selector(
                frame,
                chunks[2],
                &FlappyBirdDifficulty::ALL,
                menu.selected_difficulty,
            );
        }
        ChallengeType::Jezzball => {
            render_difficulty_selector(
                frame,
                chunks[2],
                &JezzballDifficulty::ALL,
                menu.selected_difficulty,
            );
        }
        ChallengeType::Snake => {
            render_difficulty_selector(
                frame,
                chunks[2],
                &SnakeDifficulty::ALL,
                menu.selected_difficulty,
            );
        }
    }

    // Outcomes
    let outcomes = Paragraph::new(vec![Line::from(vec![
        Span::styled("✓ ", Style::default().fg(Color::Green)),
        Span::styled("No penalty for losing", Style::default().fg(Color::Gray)),
        Span::styled("    ✓ ", Style::default().fg(Color::Green)),
        Span::styled("Draw grants bonus XP", Style::default().fg(Color::Gray)),
    ])]);
    frame.render_widget(outcomes, chunks[4]);

    // Help text
    let help = Paragraph::new("[↑/↓] Difficulty  [Enter] Play  [D] Walk away  [Esc] Back")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, outer_chunks[1]);
}

/// Estimate wrapped line count for word-wrapped text in a fixed-width area.
fn estimate_wrapped_line_count(text: &str, width: u16) -> u16 {
    let width = usize::from(width.max(1));
    let mut total_lines: usize = 0;

    for raw_line in text.lines() {
        if raw_line.trim().is_empty() {
            total_lines += 1;
            continue;
        }

        let mut current_len = 0usize;
        for word in raw_line.split_whitespace() {
            let word_len = word.len();

            if current_len == 0 {
                if word_len <= width {
                    current_len = word_len;
                } else {
                    total_lines += word_len.div_ceil(width).saturating_sub(1);
                    current_len = word_len % width;
                    if current_len == 0 {
                        current_len = width;
                    }
                }
            } else if current_len + 1 + word_len <= width {
                current_len += 1 + word_len;
            } else {
                total_lines += 1;
                if word_len <= width {
                    current_len = word_len;
                } else {
                    total_lines += word_len.div_ceil(width).saturating_sub(1);
                    current_len = word_len % width;
                    if current_len == 0 {
                        current_len = width;
                    }
                }
            }
        }

        total_lines += 1;
    }

    total_lines.max(1) as u16
}

/// Generic difficulty selector that works with any type implementing DifficultyInfo
fn render_difficulty_selector<D: DifficultyInfo>(
    frame: &mut Frame,
    area: Rect,
    options: &[D],
    selected: usize,
) {
    let title = Paragraph::new("Select difficulty:").style(
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(title, Rect { height: 1, ..area });

    // Each option uses 3 lines: name + reward + blank (last has no trailing blank)
    for (i, diff) in options.iter().enumerate() {
        let row_y = area.y + 1 + (i as u16) * 3;
        if row_y + 1 >= area.y + area.height {
            break;
        }

        let is_selected = i == selected;
        let prefix = if is_selected { "> " } else { "  " };

        let prefix_style = if is_selected {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        let name_style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        };
        let reward_style = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Line 1: difficulty name + extra info
        let mut name_spans = vec![
            Span::styled(prefix, prefix_style),
            Span::styled(diff.name(), name_style),
        ];
        if let Some(extra) = diff.extra_info() {
            name_spans.push(Span::styled(
                format!("  {}", extra),
                Style::default().fg(Color::DarkGray),
            ));
        }
        let name_line = Paragraph::new(Line::from(name_spans));
        let name_area = Rect {
            x: area.x,
            y: row_y,
            width: area.width,
            height: 1,
        };
        frame.render_widget(name_line, name_area);

        // Line 2: reward (indented)
        let reward_line = Paragraph::new(Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled(
                format!("Win: {}", diff.reward().description()),
                reward_style,
            ),
        ]));
        let reward_area = Rect {
            x: area.x,
            y: row_y + 1,
            width: area.width,
            height: 1,
        };
        frame.render_widget(reward_line, reward_area);
    }
}
