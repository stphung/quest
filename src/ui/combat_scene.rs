use super::responsive::{LayoutContext, SizeTier};
use crate::combat::logic::effective_enemy_attack_interval;
use crate::core::constants::{ATTACK_INTERVAL_SECONDS, BOSS_ENRAGE_SECONDS};
use crate::core::game_state::GameState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

use crate::combat::types::DAMAGE_FLASH_DURATION;

use super::combat_3d::render_combat_3d;
use super::enemy_sprites::zone_palette;

fn highest_slayer_badge(achievements: &crate::achievements::Achievements) -> Option<&'static str> {
    use crate::achievements::AchievementId;
    let ids = [
        AchievementId::SlayerXV,
        AchievementId::SlayerXIV,
        AchievementId::SlayerXIII,
        AchievementId::SlayerXII,
        AchievementId::SlayerXI,
        AchievementId::SlayerX,
        AchievementId::SlayerIX,
        AchievementId::SlayerVIII,
        AchievementId::SlayerVII,
        AchievementId::SlayerVI,
        AchievementId::SlayerV,
        AchievementId::SlayerIV,
        AchievementId::SlayerIII,
        AchievementId::SlayerII,
        AchievementId::SlayerI,
    ];
    for id in ids {
        if achievements.is_unlocked(id) {
            return crate::achievements::data::get_achievement_def(id).map(|def| def.icon);
        }
    }
    None
}

fn highest_boss_hunter_badge(
    achievements: &crate::achievements::Achievements,
) -> Option<&'static str> {
    use crate::achievements::AchievementId;
    let ids = [
        AchievementId::BossHunterXV,
        AchievementId::BossHunterXIV,
        AchievementId::BossHunterXIII,
        AchievementId::BossHunterXII,
        AchievementId::BossHunterXI,
        AchievementId::BossHunterX,
        AchievementId::BossHunterIX,
        AchievementId::BossHunterVIII,
        AchievementId::BossHunterVII,
        AchievementId::BossHunterVI,
        AchievementId::BossHunterV,
        AchievementId::BossHunterIV,
        AchievementId::BossHunterIII,
        AchievementId::BossHunterII,
        AchievementId::BossHunterI,
    ];
    for id in ids {
        if achievements.is_unlocked(id) {
            return crate::achievements::data::get_achievement_def(id).map(|def| def.icon);
        }
    }
    None
}

fn combat_title(achievements: &crate::achievements::Achievements) -> String {
    let slayer = highest_slayer_badge(achievements).unwrap_or("");
    let boss = highest_boss_hunter_badge(achievements).unwrap_or("");
    match (slayer.is_empty(), boss.is_empty()) {
        (true, true) => " \u{2694} Combat \u{2694} ".to_string(),
        (false, true) => format!(" \u{2694} Combat {} ", slayer),
        (true, false) => format!(" \u{2694} Combat {} ", boss),
        (false, false) => format!(" \u{2694} Combat {}{} ", slayer, boss),
    }
}

/// Draws the combat scene with 3D first-person view
pub fn draw_combat_scene(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    achievements: &crate::achievements::Achievements,
    ctx: &LayoutContext,
) {
    match ctx.tier {
        SizeTier::M => {
            // Compact: no 3D sprite, just HP bars + status, with border
            draw_combat_compact(frame, area, game_state, achievements);
        }
        SizeTier::S => {
            // Minimal: no border, no sprite — HP bars handled by S layout directly
            // When called from S layout, just show combat status
            draw_combat_status(frame, area, game_state);
        }
        _ => {
            // Full layout with 3D sprite (XL/L)
            draw_combat_full(frame, area, game_state, achievements);
        }
    }
}

/// Full combat scene with 3D sprite (XL/L tier).
fn draw_combat_full(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    achievements: &crate::achievements::Achievements,
) {
    // Single outer border wrapping everything
    let title = combat_title(achievements);
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Red)))
        .title(title)
        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    let outer_block = super::themed_block(outer_block);

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);
    super::apply_themed_border_fx(frame, area, Color::Red, super::BorderFxContext);

    let is_regen = game_state.combat_state.is_regenerating;

    let is_boss = game_state.zone_progression.fighting_boss;
    let status_lines = if is_boss { 2 } else { 1 };

    let mut constraints = vec![Constraint::Length(1)]; // Player HP
    if is_regen {
        constraints.push(Constraint::Length(1)); // Regen throbber
    }
    constraints.push(Constraint::Min(5)); // Sprite
    constraints.push(Constraint::Length(1)); // Enemy HP
    constraints.push(Constraint::Length(status_lines)); // Status (2 lines during boss)

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let mut idx = 0;
    draw_player_hp(frame, chunks[idx], game_state);
    idx += 1;

    if is_regen {
        draw_regen_throbber(frame, chunks[idx], game_state);
        idx += 1;
    }

    render_combat_3d(frame, chunks[idx], game_state);
    draw_floating_damage(frame, chunks[idx], game_state);
    idx += 1;

    draw_enemy_hp(frame, chunks[idx], game_state);
    idx += 1;

    draw_combat_status(frame, chunks[idx], game_state);
}

/// Compact combat scene for M tier: HP bars + sprite + status.
fn draw_combat_compact(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    achievements: &crate::achievements::Achievements,
) {
    let title = combat_title(achievements);
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Red)))
        .title(title);
    let outer_block = super::themed_block(outer_block);

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);
    super::apply_themed_border_fx(frame, area, Color::Red, super::BorderFxContext);

    let is_regen = game_state.combat_state.is_regenerating;
    let is_boss = game_state.zone_progression.fighting_boss;
    let status_lines = if is_boss { 2 } else { 1 };

    let mut constraints = vec![Constraint::Length(1)]; // Player HP
    if is_regen {
        constraints.push(Constraint::Length(1)); // Regen throbber
    }
    constraints.push(Constraint::Min(3)); // Sprite
    constraints.push(Constraint::Length(1)); // Enemy HP
    constraints.push(Constraint::Length(status_lines)); // Status (2 lines during boss)

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let mut idx = 0;
    draw_player_hp(frame, chunks[idx], game_state);
    idx += 1;

    if is_regen {
        draw_regen_throbber(frame, chunks[idx], game_state);
        idx += 1;
    }

    render_combat_3d(frame, chunks[idx], game_state);
    draw_floating_damage(frame, chunks[idx], game_state);
    idx += 1;

    draw_enemy_hp(frame, chunks[idx], game_state);
    idx += 1;

    draw_combat_status(frame, chunks[idx], game_state);
}

/// Draws the player HP bar (borderless, single line)
pub(super) fn draw_player_hp(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let hp_ratio = game_state.combat_state.player_current_hp as f64
        / game_state.combat_state.player_max_hp as f64;

    let label = format!(
        "Player HP: {}/{}",
        game_state.combat_state.player_current_hp, game_state.combat_state.player_max_hp
    );

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .label(label)
        .ratio(hp_ratio);

    frame.render_widget(gauge, area);
}

/// Draws a regen throbber line below the player HP bar (spinner + flavor text).
fn draw_regen_throbber(frame: &mut Frame, area: Rect, game_state: &GameState) {
    use super::throbber::{regen_message, spinner_char};

    let spinner = spinner_char();
    let message = regen_message(game_state.character_xp);
    let text = Line::from(Span::styled(
        format!("{} {}", spinner, message),
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::ITALIC),
    ));

    let paragraph = Paragraph::new(text).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

/// Draws the enemy HP bar (borderless, single line) with zone-aware coloring
pub(super) fn draw_enemy_hp(frame: &mut Frame, area: Rect, game_state: &GameState) {
    if let Some(enemy) = &game_state.combat_state.current_enemy {
        let hp_ratio = enemy.current_hp as f64 / enemy.max_hp as f64;

        let label = format!("{}: {}/{}", enemy.name, enemy.current_hp, enemy.max_hp);

        let is_boss = game_state.zone_progression.fighting_boss;
        let is_dungeon_boss = enemy.name.starts_with("Boss ");
        let hp_color = if is_boss || is_dungeon_boss {
            Color::LightRed
        } else {
            let zone_id = game_state
                .active_dungeon
                .as_ref()
                .map(|d| d.zone_id)
                .unwrap_or(game_state.zone_progression.current_zone_id);
            zone_palette(zone_id).primary
        };

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(hp_color).add_modifier(Modifier::BOLD))
            .label(label)
            .ratio(hp_ratio);

        frame.render_widget(gauge, area);
    }
}

/// Renders floating damage numbers over the sprite area.
/// Enemy damage floats UP from the bottom; player damage floats DOWN from the top.
/// Numbers fade from their original color to DarkGray as they travel.
fn draw_floating_damage(frame: &mut Frame, sprite_area: Rect, game_state: &GameState) {
    if sprite_area.height == 0 || sprite_area.width == 0 {
        return;
    }

    // Enemy damage floats UP from bottom of sprite area
    for float in &game_state.combat_state.enemy_damage_floats {
        let progress = 1.0 - (float.remaining / DAMAGE_FLASH_DURATION);
        let max_rise = (sprite_area.height as f64 * 0.15).max(1.0);
        let row_offset = (progress * max_rise) as u16;
        let y = sprite_area.bottom().saturating_sub(1 + row_offset);
        if y < sprite_area.y {
            continue;
        }
        render_float_text(frame, sprite_area, y, float, progress);
    }

    // Player damage floats DOWN from top of sprite area
    for float in &game_state.combat_state.player_damage_floats {
        let progress = 1.0 - (float.remaining / DAMAGE_FLASH_DURATION);
        let max_drop = (sprite_area.height as f64 * 0.15).max(1.0);
        let row_offset = (progress * max_drop) as u16;
        let y = sprite_area.y + row_offset;
        if y >= sprite_area.bottom() {
            continue;
        }
        render_float_text(frame, sprite_area, y, float, progress);
    }
}

/// Renders a single floating damage text at the given y position, right-aligned.
/// Fades the color based on progress (0.0 = fresh, 1.0 = expired).
fn render_float_text(
    frame: &mut Frame,
    sprite_area: Rect,
    y: u16,
    float: &crate::combat::types::DamageFlash,
    progress: f64,
) {
    let text_width = float.text.chars().count() as u16;
    if text_width == 0 || text_width >= sprite_area.width {
        return;
    }

    let style = if progress > 0.8 {
        // Final fade: dim gray
        Style::default().fg(Color::DarkGray)
    } else if progress > 0.6 {
        // Mid fade: original color, no bold
        Style::default().fg(float.color)
    } else {
        // Full intensity: original color + bold if applicable
        let mut s = Style::default().fg(float.color);
        if float.bold {
            s = s.add_modifier(Modifier::BOLD);
        }
        s
    };

    let x = sprite_area.right().saturating_sub(text_width + 1);
    let rect = Rect::new(x, y, text_width, 1);
    let para = Paragraph::new(Span::styled(&float.text, style)).alignment(Alignment::Right);
    frame.render_widget(para, rect);
}

/// Draws the combat status information with DPS
pub(super) fn draw_combat_status(frame: &mut Frame, area: Rect, game_state: &GameState) {
    use super::throbber::{spinner_char, waiting_message};
    let spinner = spinner_char();

    // Use cached derived stats (includes real enhancement levels)
    let derived = game_state.cached_derived_stats;
    let base_dps = derived.total_damage() as f64 / ATTACK_INTERVAL_SECONDS;
    let effective_dps = base_dps
        * (1.0 + (derived.crit_chance_percent as f64 / 100.0) * (derived.crit_multiplier - 1.0));
    let dps_span = Span::styled(
        format!(" | DPS: {:.0}", effective_dps),
        Style::default().fg(Color::DarkGray),
    );

    let status_text = if game_state.combat_state.current_enemy.is_some() {
        let player_interval = ATTACK_INTERVAL_SECONDS / derived.attack_speed_multiplier;
        let player_next = (player_interval - game_state.combat_state.player_attack_timer).max(0.0);
        let enemy_interval = effective_enemy_attack_interval(game_state);
        let enemy_next = (enemy_interval - game_state.combat_state.enemy_attack_timer).max(0.0);

        let player_style = if player_next < 0.3 {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };
        let enemy_style = if enemy_next < 0.3 {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };

        let mut lines = vec![Line::from(vec![
            Span::styled(
                format!("{} In Combat", spinner),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(format!("You: {:.1}s", player_next), player_style),
            Span::raw("  "),
            Span::styled(format!("Foe: {:.1}s", enemy_next), enemy_style),
            dps_span,
        ])];

        // Boss enrage countdown (second line)
        if game_state.zone_progression.fighting_boss {
            let remaining =
                (BOSS_ENRAGE_SECONDS - game_state.combat_state.boss_fight_timer).max(0.0);
            let enrage_style = if remaining < 5.0 {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else if remaining < 10.0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Cyan)
            };
            lines.push(Line::from(vec![Span::styled(
                format!("\u{26a1} ENRAGE: {:.0}s", remaining),
                enrage_style,
            )]));
        }

        lines
    } else {
        let message = waiting_message(game_state.character_xp);
        vec![Line::from(vec![
            Span::styled(
                format!("{} {}", spinner, message),
                Style::default().fg(Color::Yellow),
            ),
            dps_span,
        ])]
    };

    let status_paragraph = Paragraph::new(status_text).alignment(Alignment::Center);
    frame.render_widget(status_paragraph, area);
}

/// Render the fracture region unlock celebration modal.
pub fn render_fracture_region_unlock_modal(
    frame: &mut Frame,
    area: Rect,
    region: crate::zones::FractureRegion,
    ascension_level: u32,
    _ctx: &LayoutContext,
) {
    use crate::ascension::types::{ascension_combat_multiplier, ascension_cost};
    use crate::power_cores::ALL_POWER_CORES;
    use ratatui::widgets::Clear;

    const GOLD: Color = Color::Rgb(255, 215, 0);

    // Determine whether to show ascension hint
    let asc_unlocked = region.ascension_level_unlocked();
    let show_ascension = ascension_level < asc_unlocked;
    let extra_zones = (region.end_zone_id() - region.start_zone_id() + 1).saturating_sub(3) as u16;
    let modal_height = if show_ascension {
        25u16 + extra_zones
    } else {
        21u16 + extra_zones
    };

    let modal_width = 58u16.min(area.width.saturating_sub(4));
    let modal_height = modal_height.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" \u{25b6} New Region Unlocked \u{25c0} ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Magenta)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        Color::Magenta,
        super::BorderFxContext,
    );

    let mut lines: Vec<Line> = vec![
        // Headline
        Line::from(""),
        Line::from(Span::styled(
            region.unlock_headline(),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        // Atmospheric text + power core narrative
        Line::from(""),
        Line::from(Span::styled(
            region.unlock_atmospheric(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            region.power_core_narrative(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    // Power Core mechanic line
    if let Some(core) = ALL_POWER_CORES
        .iter()
        .find(|c| c.required_layer == region.unlock_layer())
    {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!(
                "\u{2b21} Power Core: {}  \u{2014}  {} PR/day",
                core.name, core.pr_per_day
            ),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
    }

    // Zone list — one per line with ◆ prefix
    lines.push(Line::from(""));
    for zid in region.start_zone_id()..=region.end_zone_id() {
        let name = crate::zones::get_zone(zid).map(|z| z.name).unwrap_or("???");
        lines.push(Line::from(vec![
            Span::styled("  \u{25c6} ", Style::default().fg(Color::Magenta)),
            Span::styled(
                format!("Zone {}  {}", zid, name),
                Style::default().fg(Color::White),
            ),
        ]));
    }

    // Ascension narrative bridge (only if a new level just became available)
    if show_ascension {
        let roman = crate::ui::stats_prestige::to_roman(asc_unlocked);
        let mult = ascension_combat_multiplier(asc_unlocked);
        let cost = ascension_cost(asc_unlocked);

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            region.ascension_narrative(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!(
                "\u{2726} ASCENSION {}  \u{2014}  \u{00d7}{:.1} power",
                roman, mult
            ),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::styled(
            format!("  Cost: {} PR  \u{2022}  [U] to Ascend", cost),
            Style::default().fg(GOLD),
        )]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Enter] to dismiss",
        Style::default().fg(Color::DarkGray),
    )));

    let text = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(text, inner);
}
