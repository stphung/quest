//! Blacksmith UI rendering: equipment enhancement overlay with animations.

use crate::enhancement::{
    enhancement_cost, enhancement_multiplier, fail_penalty, success_rate, BlacksmithPhase,
    BlacksmithUiState, EnhancementProgress, MAX_ENHANCEMENT_LEVEL,
};
use crate::items::{Equipment, EquipmentSlot};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

const SLOT_ORDER: [EquipmentSlot; 7] = [
    EquipmentSlot::Weapon,
    EquipmentSlot::Armor,
    EquipmentSlot::Helmet,
    EquipmentSlot::Gloves,
    EquipmentSlot::Boots,
    EquipmentSlot::Amulet,
    EquipmentSlot::Ring,
];

const SLOT_ICONS: [&str; 7] = [
    "\u{2694}",  // Weapon: crossed swords
    "\u{1f6e1}", // Armor: shield
    "\u{26d1}",  // Helmet
    "\u{1f9e4}", // Gloves
    "\u{1f462}", // Boots
    "\u{1f4bf}", // Amulet (disc)
    "\u{1f48d}", // Ring
];

/// Enhancement level color based on tier
fn level_color(level: u8) -> Color {
    match level {
        0 => Color::DarkGray,
        1..=4 => Color::White,
        5..=7 => Color::Yellow,
        8..=9 => Color::Magenta,
        10 => Color::Rgb(255, 215, 0),
        _ => Color::DarkGray,
    }
}

/// Render the blacksmith overlay
pub fn render_blacksmith(
    frame: &mut Frame,
    area: Rect,
    blacksmith_ui: &BlacksmithUiState,
    enhancement: &EnhancementProgress,
    equipment: &Equipment,
    prestige_rank: u32,
    _ctx: &super::responsive::LayoutContext,
) {
    // Center overlay: 62 wide, 24 tall (or fit to terminal)
    let overlay_width = 62u16.min(area.width.saturating_sub(4));
    let overlay_height = 24u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let title = format!(
        " \u{2692} The Blacksmith  [Prestige Ranks: {}] ",
        prestige_rank
    );
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    match blacksmith_ui.phase {
        BlacksmithPhase::Menu => {
            render_menu(
                frame,
                inner,
                blacksmith_ui,
                enhancement,
                equipment,
                prestige_rank,
            );
        }
        BlacksmithPhase::Confirming => {
            render_confirming(
                frame,
                inner,
                blacksmith_ui,
                enhancement,
                equipment,
                prestige_rank,
            );
        }
        BlacksmithPhase::Hammering => {
            render_hammering(frame, inner, blacksmith_ui, enhancement, equipment);
        }
        BlacksmithPhase::ResultSuccess => {
            render_success(frame, inner, blacksmith_ui, enhancement, equipment);
        }
        BlacksmithPhase::ResultFailure => {
            render_failure(frame, inner, blacksmith_ui);
        }
    }
}

/// Render the equipment slot menu
fn render_menu(
    frame: &mut Frame,
    area: Rect,
    blacksmith_ui: &BlacksmithUiState,
    enhancement: &EnhancementProgress,
    equipment: &Equipment,
    prestige_rank: u32,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Flavor text
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Column header
            Constraint::Length(7), // Slot list (7 slots)
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Detail panel for selected slot
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Stats
            Constraint::Length(1), // Help
            Constraint::Min(0),    // Padding
        ])
        .split(area);

    // Flavor text from the blacksmith
    let flavor = Paragraph::new(
        "\u{201c}What I do here is forge the bond between warrior and \
         armament. The gear may change, but my work never fades. \
         Push further and the craft grows perilous \u{2014} \
         but the power grows faster.\u{201d}",
    )
    .style(Style::default().fg(Color::DarkGray))
    .wrap(Wrap { trim: true });
    frame.render_widget(flavor, chunks[0]);

    // Column header
    //       "▶ " + icon + " " + name(20) + " " + level info
    let header = Paragraph::new(Line::from(vec![Span::styled(
        "     Equipment            Level      Rate",
        Style::default().fg(Color::DarkGray),
    )]));
    frame.render_widget(header, chunks[2]);

    // Equipment slot rows
    let slot_area = chunks[3];
    for (i, (slot, icon)) in SLOT_ORDER.iter().zip(SLOT_ICONS.iter()).enumerate() {
        if i as u16 >= slot_area.height {
            break;
        }
        let row_area = Rect::new(slot_area.x, slot_area.y + i as u16, slot_area.width, 1);
        let is_selected = i == blacksmith_ui.selected_slot;
        let item = equipment.get(*slot);
        let current_level = enhancement.level(i);

        let mut spans = Vec::new();

        // Selection indicator
        if is_selected {
            spans.push(Span::styled(
                "\u{25b6} ",
                Style::default().fg(Color::Yellow),
            ));
        } else {
            spans.push(Span::raw("  "));
        }

        // Slot icon
        spans.push(Span::raw(format!("{} ", icon)));

        // Display name: item name if equipped, slot name if empty
        let max_name_len = 18;
        let (name, name_color) = if let Some(item_ref) = item.as_ref() {
            let n = if item_ref.display_name.chars().count() > max_name_len {
                let truncated: String = item_ref
                    .display_name
                    .chars()
                    .take(max_name_len - 3)
                    .collect();
                format!("{}...", truncated)
            } else {
                format!("{:width$}", item_ref.display_name, width = max_name_len)
            };
            (n, Color::White)
        } else {
            (
                format!("{:width$}", slot.name(), width = max_name_len),
                Color::DarkGray,
            )
        };
        spans.push(Span::styled(name, Style::default().fg(name_color)));
        spans.push(Span::raw(" "));

        // Enhancement level and target (fixed-width for column alignment)
        if current_level >= MAX_ENHANCEMENT_LEVEL {
            spans.push(Span::styled(
                "+10 MAX    ",
                Style::default()
                    .fg(Color::Rgb(255, 215, 0))
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            let lvl_color = level_color(current_level);
            spans.push(Span::styled(
                format!("+{:<2}", current_level),
                Style::default().fg(lvl_color),
            ));

            let target = current_level + 1;
            spans.push(Span::styled(
                format!(" \u{2192} +{:<2}", target),
                Style::default().fg(level_color(target)),
            ));

            let rate = success_rate(target);
            let rate_color = if rate >= 1.0 {
                Color::Green
            } else if rate >= 0.5 {
                Color::Yellow
            } else {
                Color::Red
            };
            spans.push(Span::styled(
                format!(" {:>3.0}%", rate * 100.0),
                Style::default().fg(rate_color),
            ));
        }

        let row_style = if is_selected {
            Style::default().bg(Color::Rgb(40, 40, 20))
        } else {
            Style::default()
        };

        let row = Paragraph::new(Line::from(spans)).style(row_style);
        frame.render_widget(row, row_area);
    }

    // Detail panel for selected slot
    let selected_level = enhancement.level(blacksmith_ui.selected_slot);
    let detail_area = chunks[5];

    if selected_level >= MAX_ENHANCEMENT_LEVEL {
        let detail = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Bonus: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "+50.0% stats",
                    Style::default()
                        .fg(Color::Rgb(255, 215, 0))
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                "Maximum enhancement reached.",
                Style::default().fg(Color::DarkGray),
            )),
        ]);
        frame.render_widget(detail, detail_area);
    } else {
        let target = selected_level + 1;
        let bonus = enhancement_multiplier(target);
        let bonus_pct = (bonus - 1.0) * 100.0;
        let rate = success_rate(target);
        let cost = enhancement_cost(target);
        let can_afford = prestige_rank >= cost;
        let penalty = fail_penalty(target);

        let rate_color = if rate >= 1.0 {
            Color::Green
        } else if rate >= 0.5 {
            Color::Yellow
        } else {
            Color::Red
        };
        let cost_color = if can_afford { Color::Cyan } else { Color::Red };

        let failure_text = if penalty == 0 {
            Span::styled("safe (no level loss)", Style::default().fg(Color::Green))
        } else {
            let result_level = selected_level.saturating_sub(penalty);
            Span::styled(
                format!(
                    "-{} level{} (+{} \u{2192} +{})",
                    penalty,
                    if penalty > 1 { "s" } else { "" },
                    selected_level,
                    result_level
                ),
                Style::default().fg(Color::Red),
            )
        };

        let detail = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Bonus: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("+{:.1}% stats", bonus_pct),
                    Style::default().fg(Color::Green),
                ),
                Span::styled("  Rate: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.0}%", rate * 100.0),
                    Style::default().fg(rate_color),
                ),
                Span::styled("  Cost: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} Prestige Ranks", cost),
                    Style::default().fg(cost_color),
                ),
            ]),
            Line::from(vec![
                Span::styled("On failure: ", Style::default().fg(Color::DarkGray)),
                failure_text,
            ]),
        ]);
        frame.render_widget(detail, detail_area);
    }

    // Lifetime stats
    let stats_line = Line::from(vec![
        Span::styled("Attempts: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", enhancement.total_attempts),
            Style::default().fg(Color::White),
        ),
        Span::styled(" | Successes: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", enhancement.total_successes),
            Style::default().fg(Color::Green),
        ),
        Span::styled(" | Failures: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", enhancement.total_failures),
            Style::default().fg(Color::Red),
        ),
    ]);
    let stats = Paragraph::new(stats_line);
    frame.render_widget(stats, chunks[7]);

    // Help
    let help = Paragraph::new(Line::from(Span::styled(
        "\u{2191}\u{2193} Select  Enter Enhance  Esc Close",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(help, chunks[8]);
}

/// Render the confirmation phase
fn render_confirming(
    frame: &mut Frame,
    area: Rect,
    blacksmith_ui: &BlacksmithUiState,
    enhancement: &EnhancementProgress,
    equipment: &Equipment,
    prestige_rank: u32,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Top padding
            Constraint::Length(8), // Confirmation content
            Constraint::Min(0),    // Bottom padding
        ])
        .split(area);

    let slot_index = blacksmith_ui.selected_slot;
    let slot = SLOT_ORDER[slot_index];
    let current_level = enhancement.level(slot_index);
    let target_level = current_level + 1;
    let cost = enhancement_cost(target_level);
    let rate = success_rate(target_level);

    let item_name = equipment
        .get(slot)
        .as_ref()
        .map(|i| i.display_name.as_str())
        .unwrap_or_else(|| slot.name());

    let bonus = enhancement_multiplier(target_level);
    let bonus_pct = (bonus - 1.0) * 100.0;

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("Enhance {} to +{}?", item_name, target_level),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Success rate: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:.0}%", rate * 100.0),
                Style::default().fg(if rate >= 0.5 {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
            Span::styled("  Cost: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{} Prestige Ranks", cost),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                format!(" ({} \u{2192} {})", prestige_rank, prestige_rank - cost),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("Bonus at +", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", target_level),
                Style::default().fg(level_color(target_level)),
            ),
            Span::styled(
                format!(": +{:.1}% stats", bonus_pct),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Enter Confirm  Esc Cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(text, chunks[1]);
}

/// Render the hammering animation
fn render_hammering(
    frame: &mut Frame,
    area: Rect,
    blacksmith_ui: &BlacksmithUiState,
    enhancement: &EnhancementProgress,
    equipment: &Equipment,
) {
    let tick = blacksmith_ui.animation_tick;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Top padding
            Constraint::Length(12), // Anvil area
            Constraint::Length(1),  // Progress
            Constraint::Min(0),     // Bottom padding
        ])
        .split(area);

    // Determine if this is a strike tick
    let is_strike = matches!(tick, 7 | 8 | 15 | 16 | 23 | 24);

    // Hammer position: raised vs striking
    let hammer_raised = [
        "       ___  ",
        "      |   | ",
        "      |___|/",
        "        |   ",
        "        |   ",
    ];
    let hammer_strike = [
        "            ",
        "            ",
        "   ___      ",
        "  |   |___  ",
        "  |___|/    ",
    ];

    let anvil = [
        "    _________    ",
        "   /         \\   ",
        "  /___________\\  ",
        "      |   |      ",
        "   ___|   |___   ",
        "  |___________|  ",
    ];

    let hammer = if is_strike {
        &hammer_strike
    } else {
        &hammer_raised
    };

    // Get item name for display on anvil
    let slot_index = blacksmith_ui.selected_slot;
    let slot = SLOT_ORDER[slot_index];
    let current_level = enhancement.level(slot_index);
    let item_name = equipment
        .get(slot)
        .as_ref()
        .map(|i| i.display_name.clone())
        .unwrap_or_else(|| slot.name().to_string());
    let item_display = if item_name.chars().count() > 15 {
        let truncated: String = item_name.chars().take(12).collect();
        format!("{}..+{}", truncated, current_level)
    } else {
        format!("{} +{}", item_name, current_level)
    };

    // Build the visual
    let mut lines = Vec::new();

    // Hammer lines
    for line in hammer {
        let style = if is_strike {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(Span::styled(*line, style)));
    }

    // Spark line (only on strike)
    if is_strike {
        lines.push(Line::from(vec![
            Span::styled("  \u{2726} ", Style::default().fg(Color::Yellow)),
            Span::styled("\u{2727} ", Style::default().fg(Color::Rgb(255, 215, 0))),
            Span::styled("* ", Style::default().fg(Color::Yellow)),
            Span::styled("\u{00b7} ", Style::default().fg(Color::White)),
            Span::styled("\u{2726}", Style::default().fg(Color::Rgb(255, 200, 0))),
        ]));
    } else {
        lines.push(Line::from(""));
    }

    // Anvil lines
    for line in &anvil {
        lines.push(Line::from(Span::styled(
            *line,
            Style::default().fg(Color::DarkGray),
        )));
    }

    let visual = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(visual, chunks[1]);

    // Progress bar using characters
    let progress = tick as f64 / 25.0;
    let bar_width = area.width.saturating_sub(8) as usize;
    let filled = (progress * bar_width as f64) as usize;
    let empty = bar_width.saturating_sub(filled);

    let progress_line = Line::from(vec![
        Span::styled("  [", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "\u{2588}".repeat(filled),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(
            "\u{2591}".repeat(empty),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled("] ", Style::default().fg(Color::DarkGray)),
        Span::styled(item_display, Style::default().fg(Color::White)),
    ]);
    let progress_widget = Paragraph::new(progress_line);
    frame.render_widget(progress_widget, chunks[2]);
}

/// Render the success animation
fn render_success(
    frame: &mut Frame,
    area: Rect,
    blacksmith_ui: &BlacksmithUiState,
    _enhancement: &EnhancementProgress,
    equipment: &Equipment,
) {
    let tick = blacksmith_ui.animation_tick;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Top padding
            Constraint::Length(9), // Content
            Constraint::Min(0),    // Bottom padding
        ])
        .split(area);

    let result = blacksmith_ui.last_result.as_ref().unwrap();
    let slot = SLOT_ORDER[result.slot_index];
    let item_name = equipment
        .get(slot)
        .as_ref()
        .map(|i| i.display_name.as_str())
        .unwrap_or_else(|| slot.name());

    let bonus = enhancement_multiplier(result.new_level);
    let bonus_pct = (bonus - 1.0) * 100.0;

    // Pulse between yellow and gold
    let title_color = if tick % 4 < 2 {
        Color::Yellow
    } else {
        Color::Rgb(255, 215, 0)
    };

    // Sparkle border characters
    let sparkle = if tick.is_multiple_of(3) {
        "\u{2726}"
    } else if tick % 3 == 1 {
        "\u{2727}"
    } else {
        "*"
    };

    let sparkle_line = format!(
        " {} {} {} {} {} {} {} ",
        sparkle, sparkle, sparkle, sparkle, sparkle, sparkle, sparkle
    );

    let text = Paragraph::new(vec![
        Line::from(Span::styled(
            &sparkle_line,
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "SUCCESS!",
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("{} is now +{}!", item_name, result.new_level),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("+{:.1}% stats", bonus_pct),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(Span::styled(
            &sparkle_line,
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            "Press any key to continue",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(text, chunks[1]);
}

/// Render the failure animation
fn render_failure(frame: &mut Frame, area: Rect, blacksmith_ui: &BlacksmithUiState) {
    let tick = blacksmith_ui.animation_tick;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Top padding
            Constraint::Length(9), // Content
            Constraint::Min(0),    // Bottom padding
        ])
        .split(area);

    let result = blacksmith_ui.last_result.as_ref().unwrap();

    // Shake offset for first 5 ticks
    let shake_offset = if tick < 5 {
        if tick.is_multiple_of(2) {
            " "
        } else {
            ""
        }
    } else {
        ""
    };

    let crack_line = " \u{2573}  \u{2573}  \u{2573}  \u{2573}  \u{2573} ";

    let level_drop = if result.old_level == result.new_level {
        // No level change (stayed same)
        format!("+{} (no change)", result.old_level)
    } else {
        format!("+{} \u{2192} +{}", result.old_level, result.new_level)
    };

    let text = Paragraph::new(vec![
        Line::from(Span::styled(crack_line, Style::default().fg(Color::Red))),
        Line::from(""),
        Line::from(Span::styled(
            format!("{}FAILED!", shake_offset),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Enhancement failed! {}", level_drop),
            Style::default().fg(Color::Red),
        )),
        Line::from(""),
        Line::from(Span::styled(crack_line, Style::default().fg(Color::Red))),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to continue",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(text, chunks[1]);
}

/// Render the Blacksmith discovery modal
pub fn render_blacksmith_discovery_modal(
    frame: &mut Frame,
    area: Rect,
    _ctx: &super::responsive::LayoutContext,
) {
    // Center the modal
    let modal_width = 50u16.min(area.width.saturating_sub(4));
    let modal_height = 7u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" \u{2692} Discovery! ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "A wandering Blacksmith has set up shop!",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::White)),
            Span::styled("[B]", Style::default().fg(Color::Yellow)),
            Span::styled(" to visit. ", Style::default().fg(Color::White)),
            Span::styled("[Enter]", Style::default().fg(Color::DarkGray)),
            Span::styled(" to dismiss.", Style::default().fg(Color::DarkGray)),
        ]),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}
