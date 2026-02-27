//! The Deep — Event response sub-view rendering.

use crate::deep::{DeepState, DeepUiState, MissionStatus};
use chrono::Utc;
use ratatui::style::Color;

use super::deep_missions::{archetype_color, mission_type_color};
use super::deep_scene::DEEP_BORDER_COLOR;
use super::deep_shared::{draw_deep_card, format_hours, truncate_text};
use super::responsive::{LayoutContext, SizeTier};
use super::scene_fx::{current_millis, put_text, put_text_centered, SceneCell};

/// Auto-resolve fires after 30 minutes of no response.
const AUTO_RESOLVE_SECS: u64 = 30 * 60;

fn urgency_status(remaining_secs: u64) -> (&'static str, Color) {
    if remaining_secs <= 5 * 60 {
        ("CRITICAL", Color::LightRed)
    } else if remaining_secs <= 15 * 60 {
        ("URGENT", Color::Yellow)
    } else {
        ("STABLE", Color::Green)
    }
}

pub(super) fn render_event_response(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    height: usize,
    deep: &DeepState,
    ui: &DeepUiState,
    ctx: &LayoutContext,
) {
    if height < 4 || width < 20 {
        return;
    }

    // Find the mission with a pending event
    let mission = ui
        .event_mission_id
        .and_then(|id| deep.prestige.active_missions.iter().find(|m| m.id == id))
        .or_else(|| {
            deep.prestige
                .active_missions
                .iter()
                .find(|m| matches!(m.status, MissionStatus::EventPending))
        });

    let Some(mission) = mission else {
        put_text_centered(
            buffer,
            height as i32 / 2,
            width,
            "No pending event.",
            Color::DarkGray,
        );
        put_text(buffer, height as i32 - 1, 1, "[Esc] Back", Color::DarkGray);
        return;
    };

    // Find the pending (unresolved) event
    let event = mission.events.iter().find(|e| !e.is_resolved());

    let Some(event) = event else {
        put_text_centered(
            buffer,
            height as i32 / 2,
            width,
            "Event already resolved.",
            Color::DarkGray,
        );
        put_text(buffer, height as i32 - 1, 1, "[Esc] Back", Color::DarkGray);
        return;
    };

    let now = Utc::now();
    let tc = mission_type_color(mission.mission_type);
    let progress = mission.progress(now);
    let seconds_since_fired = (now - event.fired_at).num_seconds().max(0) as u64;
    let remaining = AUTO_RESOLVE_SECS.saturating_sub(seconds_since_fired);
    let ratio = remaining as f64 / AUTO_RESOLVE_SECS as f64; // 1.0 = full time, 0.0 = expired
    let (urgency_label, base_urgency_color) = urgency_status(remaining);
    let pulse_on = remaining <= 5 * 60 && (current_millis() / 350).is_multiple_of(2);
    let urgency_color = if pulse_on {
        Color::Red
    } else {
        base_urgency_color
    };

    // ── Header ──
    put_text(buffer, 0, 1, "EVENT RESPONSE", DEEP_BORDER_COLOR);
    let urgency_badge = format!("[{}]", urgency_label);
    let urgency_col = (width as i32 - urgency_badge.len() as i32 - 2).max(1);
    let header_col = 16i32;
    let summary = format!(
        "{}  L{}  {}%",
        mission.mission_type.display_name(),
        mission.layer,
        (progress * 100.0) as u32,
    );
    let max_summary = (urgency_col - header_col - 1).max(0) as usize;
    let summary_display: String = if max_summary == 0 {
        String::new()
    } else if summary.chars().count() > max_summary {
        summary.chars().take(max_summary).collect()
    } else {
        summary
    };
    put_text(buffer, 0, header_col, &summary_display, tc);
    put_text(buffer, 0, urgency_col, &urgency_badge, urgency_color);

    // Squad names
    let squad_names: Vec<String> = mission
        .squad
        .iter()
        .filter_map(|id| deep.prestige.find_merc(*id))
        .map(|m| format!("{} ({})", m.name, m.archetype.display_name()))
        .collect();
    if !squad_names.is_empty() {
        let squad_line = format!("Squad: {}", squad_names.join(", "));
        put_text(
            buffer,
            1,
            1,
            &truncate_text(&squad_line, width.saturating_sub(2)),
            Color::DarkGray,
        );
    }

    // ── Footer ──
    let footer = match ctx.tier {
        SizeTier::S => "[\u{2191}/\u{2193}] Choose  [Enter] Confirm  [Esc] Back",
        _ => "[\u{2191}/\u{2193}] Choose  [Enter] Confirm choice  [Esc] Back (auto-resolves later)",
    };
    put_text(buffer, height as i32 - 1, 1, footer, Color::DarkGray);
    let help_hint = "[?] Help";
    let help_col = (width as i32 - help_hint.len() as i32 - 1).max(1);
    put_text(
        buffer,
        height as i32 - 1,
        help_col,
        help_hint,
        Color::Rgb(50, 70, 100),
    );

    let content_top = 2i32;
    let content_bottom = height as i32 - 1;

    // Separator
    let sep_row = content_top;
    let sep: String = "\u{2500}".repeat(width.saturating_sub(2));
    put_text(buffer, sep_row, 1, &sep, Color::Rgb(40, 60, 80));

    // ── Event narrative ──
    let narrative_top = sep_row + 1;
    let narrative_height = ((content_bottom - narrative_top) / 2).max(3) as usize;

    // Event title bar + title
    let title_rule = "─".repeat(width.saturating_sub(16));
    put_text(buffer, narrative_top, 2, "── INCIDENT ", DEEP_BORDER_COLOR);
    put_text(
        buffer,
        narrative_top,
        14,
        &title_rule,
        Color::Rgb(40, 60, 80),
    );

    put_text_centered(
        buffer,
        narrative_top + 1,
        width,
        &format!("⚡ {}", event.title),
        urgency_color,
    );

    // Event description (word-wrapped manually)
    let desc_words: Vec<&str> = event.description.split_whitespace().collect();
    let max_line_w = width.saturating_sub(6).max(10);
    let mut desc_row = narrative_top + 3;
    let mut line_buf = String::new();
    for word in &desc_words {
        if desc_row >= narrative_top + narrative_height as i32 {
            break;
        }
        if line_buf.len() + word.len() + 1 > max_line_w && !line_buf.is_empty() {
            put_text_centered(buffer, desc_row, width, &line_buf, Color::White);
            desc_row += 1;
            line_buf.clear();
        }
        if !line_buf.is_empty() {
            line_buf.push(' ');
        }
        line_buf.push_str(word);
    }
    if !line_buf.is_empty() && desc_row < narrative_top + narrative_height as i32 {
        put_text_centered(buffer, desc_row, width, &line_buf, Color::White);
    }

    // ── Auto-resolve countdown with visual urgency block ──
    let auto_resolve_row = narrative_top + narrative_height as i32;
    let mut choices_top;

    // Prefer a larger timer card when there is enough vertical room.
    let use_timer_card = auto_resolve_row + 4 < content_bottom;
    if use_timer_card {
        let timer_left = 1i32;
        let timer_right = width as i32 - 2;
        let timer_top = auto_resolve_row;
        let timer_bottom = auto_resolve_row + 3;
        let timer_border = if pulse_on { Color::Red } else { urgency_color };
        draw_deep_card(
            buffer,
            timer_left,
            timer_top,
            timer_right,
            timer_bottom,
            timer_border,
            Color::Rgb(8, 14, 28),
            Some("RESPONSE TIMER"),
        );

        // Row 1 inside card: urgency label + digital countdown.
        let window_label = format!("Window [{}]", urgency_label);
        put_text(
            buffer,
            timer_top + 1,
            timer_left + 2,
            &window_label,
            Color::DarkGray,
        );
        let mm = remaining / 60;
        let ss = remaining % 60;
        let clock = format!("{:02}:{:02}", mm, ss);
        let clock_col = (timer_right - clock.len() as i32 - 2).max(timer_left + 2);
        put_text(buffer, timer_top + 1, clock_col, &clock, urgency_color);

        // Row 2 inside card: broad progress bar.
        let bar_col = timer_left + 2;
        let bar_width = (timer_right - bar_col - 1).max(12) as usize;
        let filled = ((ratio * bar_width as f64).round() as usize).min(bar_width);
        for i in 0..bar_width {
            let ch = if i < filled {
                if pulse_on {
                    '\u{2593}'
                } else {
                    '\u{2588}'
                }
            } else {
                '\u{2592}'
            };
            let color = if i < filled {
                urgency_color
            } else {
                Color::Rgb(28, 38, 58)
            };
            super::scene_fx::put_cell(buffer, timer_top + 2, bar_col + i as i32, ch, color);
        }

        if timer_bottom + 1 < content_bottom {
            put_text(
                buffer,
                timer_bottom + 1,
                1,
                "No response \u{2192} safest option auto-resolves.",
                Color::DarkGray,
            );
        }
        choices_top = timer_bottom + 2;
    } else {
        // Compact fallback for constrained heights.
        let bar_label = format!("Decision Window [{}]: ", urgency_label);
        put_text(buffer, auto_resolve_row, 1, &bar_label, Color::DarkGray);
        let bar_col = 1 + bar_label.len() as i32;
        let bar_width = 18usize
            .min(width.saturating_sub(bar_label.len() + 18))
            .max(8);
        let filled = ((ratio * bar_width as f64).round() as usize).min(bar_width);
        for i in 0..bar_width {
            let ch = if i < filled {
                if pulse_on {
                    '\u{2593}'
                } else {
                    '\u{2588}'
                }
            } else {
                '\u{2592}'
            };
            let color = if i < filled {
                urgency_color
            } else {
                Color::Rgb(30, 40, 60)
            };
            super::scene_fx::put_cell(buffer, auto_resolve_row, bar_col + i as i32, ch, color);
        }
        let time_col = bar_col + bar_width as i32 + 1;
        let time_text = if remaining < 60 {
            format!("{}s left", remaining)
        } else {
            format!("{}m left", remaining / 60)
        };
        put_text(
            buffer,
            auto_resolve_row,
            time_col,
            &time_text,
            urgency_color,
        );
        choices_top = auto_resolve_row + 1;
    }

    // ── First-event hint ──
    if ui.event_visit_count <= 1 && choices_top < content_bottom {
        put_text(
            buffer,
            choices_top,
            1,
            "Your choice affects outcome and timing. Events auto-resolve safely if ignored.",
            Color::Rgb(50, 80, 110),
        );
        choices_top += 1;
    }

    // ── Choices ──
    let squad_archetypes: Vec<crate::deep::MercArchetype> = mission
        .squad
        .iter()
        .filter_map(|id| deep.prestige.find_merc(*id))
        .map(|m| m.archetype)
        .collect();
    let total_options = event.choices.len() + 1; // + safe option
    let available_rows = (content_bottom - choices_top).max(0) as usize;
    let card_height = 3usize;
    let use_choice_cards = width >= 68 && available_rows >= total_options * card_height;

    if use_choice_cards {
        for (ci, choice) in event.choices.iter().enumerate() {
            let top = choices_top + (ci * card_height) as i32;
            let bottom = top + card_height as i32 - 1;
            if bottom >= content_bottom {
                break;
            }

            let is_sel = ci == ui.event_choice_index;
            let cursor = if is_sel { "\u{25b6} " } else { "  " };
            let is_available = choice
                .required_archetype
                .map(|a| squad_archetypes.contains(&a))
                .unwrap_or(true);
            let arch_tag = match choice.required_archetype {
                Some(arch) => format!("[{}] ", arch.display_name().to_uppercase()),
                None => "[Any] ".to_string(),
            };
            let arch_color = match choice.required_archetype {
                Some(arch) if is_available => archetype_color(arch),
                Some(_) => Color::DarkGray,
                None => Color::DarkGray,
            };
            let consequence = if choice.is_risky {
                match choice.risk_percent {
                    Some(pct) => format!("~{}% injury risk", pct),
                    None => "risky".to_string(),
                }
            } else if choice.time_delta_secs > 0 {
                format!("+{}", format_hours(choice.time_delta_secs as u64))
            } else if choice.time_delta_secs < 0 {
                format!("-{}", format_hours(choice.time_delta_secs.unsigned_abs()))
            } else {
                "safe".to_string()
            };

            let border = if is_sel {
                Color::Rgb(95, 175, 235)
            } else {
                Color::Rgb(40, 60, 90)
            };
            let fill = if is_sel {
                Color::Rgb(11, 22, 38)
            } else {
                Color::Rgb(6, 12, 24)
            };
            draw_deep_card(
                buffer,
                1,
                top,
                width as i32 - 2,
                bottom,
                border,
                fill,
                Some("CHOICE"),
            );

            let inner_left = 3i32;
            let inner_w = width.saturating_sub(10);
            let line_text = if is_available {
                format!("{}{}{}  —  {}", cursor, arch_tag, choice.label, consequence)
            } else if let Some(arch) = choice.required_archetype {
                format!(
                    "{}{}{}  —  {} missing in squad",
                    cursor,
                    arch_tag,
                    choice.label,
                    arch.display_name()
                )
            } else {
                format!("{}{}{}  —  {}", cursor, arch_tag, choice.label, consequence)
            };
            let label_color = if is_available {
                Color::White
            } else {
                Color::DarkGray
            };
            put_text(
                buffer,
                top + 1,
                inner_left,
                &truncate_text(&line_text, inner_w),
                label_color,
            );
            put_text(
                buffer,
                top + 1,
                inner_left + cursor.len() as i32,
                &arch_tag,
                arch_color,
            );

            let tag = if !is_available {
                ("LOCKED", Color::DarkGray)
            } else if choice.is_risky {
                ("RISK", Color::Yellow)
            } else {
                ("SAFE", Color::Green)
            };
            let tag_col = (width as i32 - tag.0.len() as i32 - 4).max(inner_left + 1);
            put_text(buffer, top + 1, tag_col, tag.0, tag.1);
        }

        let safe_top = choices_top + (event.choices.len() * card_height) as i32;
        let safe_bottom = safe_top + card_height as i32 - 1;
        if safe_bottom < content_bottom {
            let is_sel = ui.event_choice_index == event.choices.len();
            let cursor = if is_sel { "\u{25b6} " } else { "  " };
            let border = if is_sel {
                Color::Rgb(95, 175, 235)
            } else {
                Color::Rgb(40, 60, 90)
            };
            draw_deep_card(
                buffer,
                1,
                safe_top,
                width as i32 - 2,
                safe_bottom,
                border,
                Color::Rgb(6, 12, 24),
                Some("SAFE FALLBACK"),
            );
            let safe_line = format!("{}[Safe] Play it safe (no risk, no delay)", cursor);
            put_text(
                buffer,
                safe_top + 1,
                3,
                &truncate_text(&safe_line, width.saturating_sub(10)),
                Color::DarkGray,
            );
            put_text(
                buffer,
                safe_top + 1,
                width as i32 - 10,
                "SAFE",
                Color::Green,
            );
        }
    } else {
        // Compact fallback list rendering.
        for (ci, choice) in event.choices.iter().enumerate() {
            let row = choices_top + ci as i32;
            if row >= content_bottom {
                break;
            }
            let is_sel = ci == ui.event_choice_index;
            let cursor = if is_sel { "\u{25b6} " } else { "  " };

            let is_available = choice
                .required_archetype
                .map(|a| squad_archetypes.contains(&a))
                .unwrap_or(true);
            let arch_tag = match choice.required_archetype {
                Some(arch) => format!("[{}] ", arch.display_name().to_uppercase()),
                None => "[Any] ".to_string(),
            };
            let arch_color = match choice.required_archetype {
                Some(arch) if is_available => archetype_color(arch),
                Some(_) => Color::DarkGray,
                None => Color::DarkGray,
            };
            let consequence = if choice.is_risky {
                match choice.risk_percent {
                    Some(pct) => format!("~{}% risk", pct),
                    None => "risky".to_string(),
                }
            } else if choice.time_delta_secs > 0 {
                format!("+{}", format_hours(choice.time_delta_secs as u64))
            } else if choice.time_delta_secs < 0 {
                format!("-{}", format_hours(choice.time_delta_secs.unsigned_abs()))
            } else {
                "safe".to_string()
            };

            let line = format!("{}{}{}  —  {}", cursor, arch_tag, choice.label, consequence);
            let label_color = if is_available {
                Color::White
            } else {
                Color::DarkGray
            };
            put_text(
                buffer,
                row,
                1,
                &truncate_text(&line, width.saturating_sub(3)),
                label_color,
            );
            put_text(
                buffer,
                row,
                1,
                cursor,
                if is_sel { Color::Cyan } else { Color::DarkGray },
            );
            put_text(buffer, row, 3, &arch_tag, arch_color);
        }

        let auto_row = choices_top + event.choices.len() as i32;
        if auto_row < content_bottom {
            let is_sel = ui.event_choice_index == event.choices.len();
            let cursor = if is_sel { "\u{25b6} " } else { "  " };
            let safe_line = format!("{}[Safe] Play it safe (no risk)", cursor);
            put_text(
                buffer,
                auto_row,
                1,
                &truncate_text(&safe_line, width.saturating_sub(3)),
                Color::DarkGray,
            );
            put_text(
                buffer,
                auto_row,
                1,
                cursor,
                if is_sel { Color::Cyan } else { Color::DarkGray },
            );
        }
    }
}
