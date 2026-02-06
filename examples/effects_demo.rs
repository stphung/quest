//! Demo of tachyonfx combat effects
//!
//! Run with: cargo run --example effects_demo
//!
//! Press keys to trigger different effects:
//! - 1: Enemy hit flash
//! - 2: Critical hit flash
//! - 3: Enemy death dissolve
//! - 4: Boss entrance
//! - 5: Idle pulse
//! - Q: Quit

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use quest::{CombatEffectManager, CombatEffectType};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{io, time::Duration};

// Sample enemy sprite (a goblin)
const GOBLIN_SPRITE: &str = r#"
      ╭───╮
    ╭─│ ◉ ◉│─╮
    │ ╰─▽─╯ │
    │  ╭─╮  │
   ╭┴──┤ ├──┴╮
   │ ┌─┴─┴─┐ │
   ╰─┤     ├─╯
     │ ├─┤ │
     ╰─╯ ╰─╯
"#;

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create effect manager
    let mut effect_manager = CombatEffectManager::new();
    let mut current_effect_name = String::from("None");

    loop {
        // Draw UI
        terminal.draw(|f| {
            draw_demo(f, &mut effect_manager, &current_effect_name);
        })?;

        // Handle input with short timeout for animation
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let area = terminal.get_frame().area();
                    match key.code {
                        KeyCode::Char('1') => {
                            effect_manager.trigger(CombatEffectType::EnemyHit, area);
                            current_effect_name = "Enemy Hit Flash".to_string();
                        }
                        KeyCode::Char('2') => {
                            effect_manager.trigger(CombatEffectType::CriticalHit, area);
                            current_effect_name = "Critical Hit Flash".to_string();
                        }
                        KeyCode::Char('3') => {
                            effect_manager.trigger(CombatEffectType::EnemyDeath, area);
                            current_effect_name = "Enemy Death Dissolve".to_string();
                        }
                        KeyCode::Char('4') => {
                            effect_manager.trigger(CombatEffectType::BossEntrance, area);
                            current_effect_name = "Boss Entrance".to_string();
                        }
                        KeyCode::Char('5') => {
                            effect_manager.trigger(CombatEffectType::IdlePulse, area);
                            current_effect_name = "Idle Pulse (continuous)".to_string();
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn draw_demo(f: &mut Frame, effect_manager: &mut CombatEffectManager, effect_name: &str) {
    let area = f.area();

    // Split into title, sprite area, and controls
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(15),   // Sprite area
            Constraint::Length(5), // Controls
        ])
        .split(area);

    // Title
    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(" TachyonFX Combat Effects Demo ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    let title_text = Paragraph::new(format!("Current Effect: {}", effect_name))
        .block(title_block)
        .alignment(Alignment::Center);
    f.render_widget(title_text, chunks[0]);

    // Sprite area with border
    let sprite_block = Block::default()
        .borders(Borders::ALL)
        .title(" Enemy Sprite ")
        .title_style(Style::default().fg(Color::Yellow));

    let sprite_inner = sprite_block.inner(chunks[1]);
    f.render_widget(sprite_block, chunks[1]);

    // Render sprite
    render_sprite(f, sprite_inner);

    // Apply effects to the sprite area
    let buf = f.buffer_mut();
    effect_manager.process(buf, sprite_inner);

    // Controls
    let controls = vec![
        Line::from(vec![
            Span::styled("[1]", Style::default().fg(Color::Green)),
            Span::raw(" Hit  "),
            Span::styled("[2]", Style::default().fg(Color::Yellow)),
            Span::raw(" Crit  "),
            Span::styled("[3]", Style::default().fg(Color::Gray)),
            Span::raw(" Death  "),
            Span::styled("[4]", Style::default().fg(Color::Red)),
            Span::raw(" Boss  "),
            Span::styled("[5]", Style::default().fg(Color::Magenta)),
            Span::raw(" Pulse  "),
            Span::styled("[Q]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Quit"),
        ]),
    ];

    let controls_block = Block::default()
        .borders(Borders::ALL)
        .title(" Controls ")
        .title_style(Style::default().fg(Color::White));

    let controls_para = Paragraph::new(controls)
        .block(controls_block)
        .alignment(Alignment::Center);
    f.render_widget(controls_para, chunks[2]);
}

fn render_sprite(f: &mut Frame, area: Rect) {
    let sprite_lines: Vec<Line> = GOBLIN_SPRITE
        .lines()
        .map(|line| {
            let padding = (area.width as usize).saturating_sub(line.chars().count()) / 2;
            Line::from(vec![
                Span::raw(" ".repeat(padding)),
                Span::styled(
                    line,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ])
        })
        .collect();

    // Center vertically
    let sprite_height = sprite_lines.len();
    let top_padding = (area.height as usize).saturating_sub(sprite_height) / 2;

    let mut padded_lines: Vec<Line> = Vec::new();
    for _ in 0..top_padding {
        padded_lines.push(Line::from(""));
    }
    padded_lines.extend(sprite_lines);

    let sprite_para = Paragraph::new(padded_lines);
    f.render_widget(sprite_para, area);
}
