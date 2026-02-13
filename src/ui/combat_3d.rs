use crate::core::game_state::GameState;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::enemy_sprites::{
    detect_enemy_tier, get_sprite_for_enemy, zone_palette, EnemyTier, BOSS_CROWN, ZONE_BOSS_CROWN,
};

/// Returns the effective zone_id for the current combat context.
fn effective_zone_id(game_state: &GameState) -> u32 {
    game_state
        .active_dungeon
        .as_ref()
        .map(|d| d.zone_id)
        .unwrap_or(game_state.zone_progression.current_zone_id)
}

/// Eye characters that should be rendered in the secondary zone color.
const EYE_CHARS: &[char] = &['●', '◆'];

/// Renders the enemy sprite (borderless, no combat log)
pub fn render_combat_3d(frame: &mut Frame, area: Rect, game_state: &GameState) {
    if area.height < 3 || area.width < 20 {
        let msg = Paragraph::new("Area too small").alignment(Alignment::Center);
        frame.render_widget(msg, area);
        return;
    }

    render_simple_sprite(frame, area, game_state);
}

/// Renders a simple, centered enemy sprite with zone-based coloring,
/// two-tone eye rendering, tier decorations (crown), and tier-based name styling.
fn render_simple_sprite(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let mut sprite_lines: Vec<Line> = Vec::new();

    if let Some(enemy) = &game_state.combat_state.current_enemy {
        let zone_id = effective_zone_id(game_state);
        let sprite_template = get_sprite_for_enemy(&enemy.name, zone_id);
        let sprite_art = sprite_template.base_art;
        let tier = detect_enemy_tier(game_state);
        let palette = zone_palette(zone_id);

        // Determine sprite body color based on tier
        let (body_color, use_bold) = match tier {
            EnemyTier::Normal => (palette.primary, false),
            EnemyTier::DungeonElite => (palette.primary, false),
            EnemyTier::SubzoneBoss => (palette.primary, true),
            EnemyTier::DungeonBoss => (palette.primary, true),
            EnemyTier::ZoneBoss => (Color::LightRed, true),
        };

        // Add padding at top
        let available_height = area.height as usize;
        let sprite_height = sprite_art.lines().count();
        let has_crown = matches!(
            tier,
            EnemyTier::SubzoneBoss | EnemyTier::DungeonBoss | EnemyTier::ZoneBoss
        );
        // crown (1) + sprite + blank (1) + name (1)
        let extra_lines = if has_crown { 3 } else { 2 };
        let total_content = sprite_height + extra_lines;
        let top_padding = (available_height.saturating_sub(total_content)) / 2;

        for _ in 0..top_padding {
            sprite_lines.push(Line::from(""));
        }

        // Crown indicator for boss tiers
        if has_crown {
            let crown_text = if tier == EnemyTier::ZoneBoss {
                ZONE_BOSS_CROWN
            } else {
                BOSS_CROWN
            };
            // Render crown with star in Yellow, dashes/equals in DarkGray
            let mut crown_spans = Vec::new();
            for ch in crown_text.chars() {
                if ch == '\u{2605}' {
                    // Star character
                    crown_spans.push(Span::styled(
                        ch.to_string(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    crown_spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
            }
            sprite_lines.push(Line::from(crown_spans).alignment(Alignment::Center));
        }

        // Render sprite with two-tone coloring (body = primary, eyes = secondary)
        let body_style = if use_bold {
            Style::default().fg(body_color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(body_color)
        };
        let eye_style = if use_bold {
            Style::default()
                .fg(palette.secondary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(palette.secondary)
        };

        for line in sprite_art.lines() {
            let spans = render_two_tone_line(line, body_style, eye_style);
            sprite_lines.push(Line::from(spans).alignment(Alignment::Center));
        }

        // Add enemy name below sprite
        sprite_lines.push(Line::from(""));
        let name_style = match tier {
            EnemyTier::Normal => Style::default().fg(Color::Yellow),
            EnemyTier::DungeonElite => Style::default().fg(Color::LightRed),
            EnemyTier::SubzoneBoss | EnemyTier::DungeonBoss => Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
            EnemyTier::ZoneBoss => Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        };
        sprite_lines.push(
            Line::from(vec![Span::styled(enemy.name.clone(), name_style)])
                .alignment(Alignment::Center),
        );
    } else {
        // No enemy - show waiting message with spinner and rotating messages
        use super::throbber::{spinner_char, waiting_message};

        let spinner = spinner_char();
        let message = waiting_message(game_state.character_xp);

        let msg_line = (area.height / 2) as usize;
        for i in 0..area.height as usize {
            if i == msg_line {
                sprite_lines.push(
                    Line::from(vec![Span::styled(
                        format!("{} {}", spinner, message),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::ITALIC),
                    )])
                    .alignment(Alignment::Center),
                );
            } else {
                sprite_lines.push(Line::from(""));
            }
        }
    }

    let sprite_paragraph = Paragraph::new(sprite_lines).alignment(Alignment::Center);
    frame.render_widget(sprite_paragraph, area);
}

/// Renders a sprite line with two-tone coloring: eye characters in `eye_style`,
/// everything else in `body_style`.
fn render_two_tone_line<'a>(line: &str, body_style: Style, eye_style: Style) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let mut current_run = String::new();
    let mut current_is_eye = false;

    for ch in line.chars() {
        let is_eye = EYE_CHARS.contains(&ch);
        if is_eye != current_is_eye && !current_run.is_empty() {
            let style = if current_is_eye {
                eye_style
            } else {
                body_style
            };
            spans.push(Span::styled(current_run.clone(), style));
            current_run.clear();
        }
        current_is_eye = is_eye;
        current_run.push(ch);
    }

    if !current_run.is_empty() {
        let style = if current_is_eye {
            eye_style
        } else {
            body_style
        };
        spans.push(Span::styled(current_run, style));
    }

    spans
}
