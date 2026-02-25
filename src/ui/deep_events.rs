//! The Deep — Event response sub-view rendering.

use crate::deep::{DeepState, DeepUiState, MissionStatus};
use chrono::Utc;
use ratatui::style::Color;

use super::deep_missions::{archetype_color, mission_type_color};
use super::deep_scene::DEEP_BORDER_COLOR;
use super::responsive::{LayoutContext, SizeTier};
use super::scene_fx::{put_text, put_text_centered, SceneCell};

/// Format seconds as a compact duration string (e.g., "2h", "30m", "1h 30m").
fn format_duration(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h == 0 {
        format!("{}m", m.max(1))
    } else if m == 0 {
        format!("{}h", h)
    } else {
        format!("{}h {}m", h, m)
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

    // ── Header ──
    put_text(buffer, 0, 1, "EVENT", DEEP_BORDER_COLOR);
    put_text(
        buffer,
        0,
        8,
        &format!(
            "Mission: {} Layer {}    Progress: {}%",
            mission.mission_type.display_name(),
            mission.layer,
            (progress * 100.0) as u32,
        ),
        tc,
    );

    // Squad names
    let squad_names: Vec<String> = mission
        .squad
        .iter()
        .filter_map(|id| deep.prestige.find_merc(*id))
        .map(|m| format!("{} ({})", m.name, m.archetype.display_name()))
        .collect();
    if !squad_names.is_empty() {
        put_text(
            buffer,
            1,
            1,
            &format!("Squad: {}", squad_names.join(", ")),
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

    // Event title (centered, bold via full-caps)
    put_text_centered(
        buffer,
        narrative_top + 1,
        width,
        &event.title.to_uppercase(),
        Color::White,
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

    // ── Auto-resolve countdown ──
    let auto_resolve_row = narrative_top + narrative_height as i32;
    let seconds_since_fired = (now - event.fired_at).num_seconds().max(0) as u64;
    // Auto-resolve fires after 30 minutes of no response
    const AUTO_RESOLVE_SECS: u64 = 30 * 60;
    let remaining = AUTO_RESOLVE_SECS.saturating_sub(seconds_since_fired);
    let (countdown_color, countdown_text) = if remaining < 5 * 60 {
        (Color::LightRed, format!("Auto-resolve in: {}s", remaining))
    } else {
        let m = remaining / 60;
        (Color::Yellow, format!("Auto-resolve in: {}m", m))
    };
    put_text(
        buffer,
        auto_resolve_row,
        1,
        &countdown_text,
        countdown_color,
    );
    put_text(
        buffer,
        auto_resolve_row,
        countdown_text.len() as i32 + 3,
        "(safe choice will be selected)",
        Color::DarkGray,
    );

    // ── First-event hint ──
    let mut hint_offset = 0i32;
    if ui.event_visit_count <= 1 && auto_resolve_row + 1 < content_bottom {
        put_text(
            buffer,
            auto_resolve_row + 1,
            1,
            "Your choice affects outcome and timing. Events auto-resolve safely if ignored.",
            Color::Rgb(50, 80, 110),
        );
        hint_offset = 1;
    }

    // ── Choices ──
    let choices_top = auto_resolve_row + 2 + hint_offset;
    let squad_archetypes: Vec<crate::deep::MercArchetype> = mission
        .squad
        .iter()
        .filter_map(|id| deep.prestige.find_merc(*id))
        .map(|m| m.archetype)
        .collect();

    for (ci, choice) in event.choices.iter().enumerate() {
        let row = choices_top + ci as i32;
        if row >= content_bottom {
            break;
        }
        let is_sel = ci == ui.event_choice_index;
        let cursor = if is_sel { "\u{25b6} " } else { "  " };

        // Check if choice is available (archetype present in squad)
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

        // Consequence preview with explicit time delta and risk percentage
        let consequence = if choice.is_risky {
            match choice.risk_percent {
                Some(pct) => format!("\u{2014} ~{}% injury risk", pct),
                None => "\u{2014} risky".to_string(),
            }
        } else if choice.time_delta_secs > 0 {
            format!(
                "\u{2014} +{}",
                format_duration(choice.time_delta_secs as u64)
            )
        } else if choice.time_delta_secs < 0 {
            format!(
                "\u{2014} -{}",
                format_duration(choice.time_delta_secs.unsigned_abs())
            )
        } else {
            "\u{2014} safe".to_string()
        };

        // Unavailable choice explanation
        let unavail_suffix = if !is_available {
            if let Some(arch) = choice.required_archetype {
                format!("  ({} not in squad)", arch.display_name())
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let line = format!("{}{}{}{}", cursor, arch_tag, choice.label, unavail_suffix);
        let label_color = if is_available {
            Color::White
        } else {
            Color::DarkGray
        };
        put_text(buffer, row, 1, &line, label_color);
        put_text(
            buffer,
            row,
            1,
            cursor,
            if is_sel { Color::Cyan } else { Color::DarkGray },
        );
        put_text(buffer, row, 3, &arch_tag, arch_color);
        if !unavail_suffix.is_empty() {
            let suffix_col = 1 + format!("{}{}{}", cursor, arch_tag, choice.label).len() as i32;
            put_text(
                buffer,
                row,
                suffix_col,
                &unavail_suffix,
                Color::Rgb(80, 80, 80),
            );
        }

        // Consequence at right side
        let consequence_col =
            (width as i32 - consequence.len() as i32 - 2).max(line.len() as i32 + 2);
        put_text(buffer, row, consequence_col, &consequence, Color::DarkGray);
    }

    // Auto-resolve choice row (always last)
    let auto_row = choices_top + event.choices.len() as i32;
    if auto_row < content_bottom {
        let is_sel = ui.event_choice_index == event.choices.len();
        let cursor = if is_sel { "\u{25b6} " } else { "  " };
        put_text(
            buffer,
            auto_row,
            1,
            &format!("{}[Auto]  Let them decide", cursor),
            Color::DarkGray,
        );
        put_text(
            buffer,
            auto_row,
            1,
            cursor,
            if is_sel { Color::Cyan } else { Color::DarkGray },
        );
        put_text(
            buffer,
            auto_row,
            (width as i32 - 20).max(42),
            "— always safe",
            Color::DarkGray,
        );
    }
}
