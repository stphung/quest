use crate::constants::ENEMY_RESPAWN_SECONDS;
use crate::game_state::GameState;
use crate::ui::zones::{get_current_zone, get_random_enemy, get_random_environment};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Draws the combat scene with hero, enemy, and environment
pub fn draw_combat_scene(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let combat_block = Block::default()
        .borders(Borders::ALL)
        .title("Combat Arena");

    let inner = combat_block.inner(area);
    frame.render_widget(combat_block, area);

    // Split into three sections: environment, combat area, info
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Environment
            Constraint::Min(5),     // Combat area
            Constraint::Length(2), // Info
        ])
        .split(inner);

    // Draw environment
    draw_environment(frame, chunks[0], game_state);

    // Draw combat area with hero and enemy
    draw_combat_area(frame, chunks[1], game_state);

    // Draw combat info
    draw_combat_info(frame, chunks[2], game_state);
}

/// Draws the environment decoration
fn draw_environment(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let zone = get_current_zone(game_state);

    // Generate a line of environment emojis
    let mut env_line = String::new();
    for _ in 0..10 {
        env_line.push_str(get_random_environment(&zone));
        env_line.push(' ');
    }

    let env_text = vec![Line::from(Span::raw(env_line))];
    let env_paragraph = Paragraph::new(env_text).alignment(Alignment::Center);

    frame.render_widget(env_paragraph, area);
}

/// Draws the combat area with hero and enemy
fn draw_combat_area(frame: &mut Frame, area: Rect, game_state: &GameState) {
    // Calculate average level for hero display
    let total_level: u32 = game_state.stats.iter().map(|s| s.level).sum();
    let avg_level = total_level / game_state.stats.len() as u32;

    let hero_emoji = if avg_level < 25 {
        "🧙"
    } else if avg_level < 50 {
        "⚔️"
    } else if avg_level < 75 {
        "🛡️"
    } else {
        "👑"
    };

    // Determine enemy emoji based on enemy name
    let enemy_display = if let Some(ref enemy_name) = game_state.combat_state.current_enemy {
        let enemy_emoji = match enemy_name.as_str() {
            "Slime" => "🟢",
            "Rabbit" => "🐰",
            "Ladybug" => "🐞",
            "Butterfly" => "🦋",
            "Wolf" => "🐺",
            "Spider" => "🕷️",
            "Dark Elf" => "🧝",
            "Bat" => "🦇",
            "Golem" => "🗿",
            "Yeti" => "❄️",
            "Mountain Lion" => "🦁",
            "Eagle" => "🦅",
            "Skeleton" => "💀",
            "Ghost" => "👻",
            "Ancient Guardian" => "🗿",
            "Wraith" => "👤",
            "Fire Elemental" => "🔥",
            "Lava Beast" => "🌋",
            "Phoenix" => "🐦",
            "Dragon" => "🐉",
            _ => "👹",
        };
        format!("{} {}", enemy_emoji, enemy_name)
    } else {
        "... waiting ...".to_string()
    };

    // Show attack animation if timer is active
    let attack_indicator = if game_state.combat_state.attack_animation_timer > 0.0 {
        "💥"
    } else {
        " "
    };

    let combat_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("     "),
            Span::styled(
                format!("{} Hero", hero_emoji),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled(attack_indicator, Style::default().fg(Color::Yellow)),
            Span::raw("   "),
            Span::styled(
                enemy_display,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::styled(
            format!("     Lv.{}", avg_level),
            Style::default().fg(Color::Green),
        )]),
    ];

    let combat_paragraph = Paragraph::new(combat_text).alignment(Alignment::Center);

    frame.render_widget(combat_paragraph, area);
}

/// Draws combat info
fn draw_combat_info(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let zone = get_current_zone(game_state);

    let info_text = vec![Line::from(vec![
        Span::styled("Zone: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(zone.name, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled("Next spawn: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!(
            "{:.1}s",
            (ENEMY_RESPAWN_SECONDS - game_state.combat_state.enemy_spawn_timer).max(0.0)
        )),
    ])];

    let info_paragraph = Paragraph::new(info_text).alignment(Alignment::Center);

    frame.render_widget(info_paragraph, area);
}

/// Updates the combat state timers
///
/// # Arguments
/// * `state` - The game state to update
/// * `delta_time` - Time elapsed since last update in seconds
pub fn update_combat_state(state: &mut GameState, delta_time: f64) {
    // Update enemy spawn timer
    state.combat_state.enemy_spawn_timer += delta_time;

    // Update attack animation timer (decreases over time)
    if state.combat_state.attack_animation_timer > 0.0 {
        state.combat_state.attack_animation_timer -= delta_time;
        if state.combat_state.attack_animation_timer < 0.0 {
            state.combat_state.attack_animation_timer = 0.0;
        }
    }
}

/// Spawns a new enemy from the current zone
///
/// # Arguments
/// * `state` - The game state to update
pub fn spawn_enemy(state: &mut GameState) {
    // Check if it's time to spawn a new enemy
    if state.combat_state.enemy_spawn_timer >= ENEMY_RESPAWN_SECONDS {
        let zone = get_current_zone(state);
        let enemy = get_random_enemy(&zone);

        state.combat_state.current_enemy = Some(enemy.to_string());
        state.combat_state.enemy_spawn_timer = 0.0;
        state.combat_state.attack_animation_timer = 0.3; // Show attack animation for 0.3s
    }
}
