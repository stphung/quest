//! The Deep — Layer infrastructure sub-view rendering.

use crate::deep::{
    effective_duration_secs, layer_power_thresholds,
    DeepState, DeepUiState, FamiliarityLevel, Infrastructure, LayerTier, MissionType,
};
use ratatui::style::Color;

use super::deep_scene::DEEP_BORDER_COLOR;
use super::deep_shared::{
    draw_deep_card, render_progress_bar as render_card_progress_bar, truncate_text,
};
use super::responsive::{LayoutContext, SizeTier};
use super::scene_fx::{put_cell, put_text, put_text_centered, SceneCell};

pub(super) fn layer_tier_color(tier: LayerTier) -> Color {
    match tier {
        LayerTier::Shallows => Color::Green,
        LayerTier::Warrens => Color::Yellow,
        LayerTier::Hollows => Color::Magenta,
        LayerTier::SunkenReach => Color::Cyan,
        LayerTier::Abyss => Color::LightRed,
        LayerTier::Void => Color::Rgb(255, 215, 0),
    }
}

/// Render a familiarity bar as `████░░░░` within a given width.
fn render_familiarity_bar(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    col: i32,
    width: usize,
    familiarity: u8,
) {
    if width == 0 {
        return;
    }
    let ratio = familiarity as f64 / 100.0;
    let filled = ((ratio * width as f64).round() as usize).min(width);
    let empty = width - filled;
    for i in 0..filled {
        put_cell(buffer, row, col + i as i32, '\u{2588}', Color::Cyan);
    }
    for i in filled..(filled + empty) {
        put_cell(
            buffer,
            row,
            col + i as i32,
            '\u{2591}',
            Color::Rgb(30, 40, 60),
        );
    }
}

/// Familiarity color by tier name.
fn familiarity_color(familiarity: u8) -> Color {
    match FamiliarityLevel::from_familiarity(familiarity) {
        FamiliarityLevel::Unknown => Color::DarkGray,
        FamiliarityLevel::Mapped => Color::Cyan,
        FamiliarityLevel::Familiar => Color::Green,
        FamiliarityLevel::Mastered => Color::Rgb(255, 215, 0),
    }
}

/// Effect text for a familiarity level.
fn familiarity_effect_text(familiarity: u8) -> &'static str {
    match FamiliarityLevel::from_familiarity(familiarity) {
        FamiliarityLevel::Unknown => "No duration bonus yet",
        FamiliarityLevel::Mapped => "-10% mission duration",
        FamiliarityLevel::Familiar => "-20% mission duration",
        FamiliarityLevel::Mastered => "-30% duration, +25% Mark yield",
    }
}

/// Render a familiarity bar with threshold tick marks below it.
fn render_familiarity_bar_with_thresholds(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    col: i32,
    bar_width: usize,
    familiarity: u8,
) {
    render_familiarity_bar(buffer, row, col, bar_width, familiarity);
    // Threshold tick marks below
    for threshold in [25usize, 50, 75] {
        let tick_col = col + (threshold * bar_width / 100) as i32;
        put_cell(buffer, row + 1, tick_col, '\u{25b2}', Color::DarkGray);
    }
}

/// Render infrastructure slots as aligned icon columns directly into the buffer.
///
/// Each slot occupies 2 terminal columns (1-wide symbol + 1 space), giving a
/// fixed-width table that aligns across all layer rows.
fn render_infra_slots(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    start_col: i32,
    layer: &crate::deep::types::LayerRecord,
) {
    let mut col = start_col;
    for infra in Infrastructure::ALL {
        if layer.has_infrastructure(*infra) {
            put_cell(buffer, row, col, infra.icon(), Color::White);
        } else {
            put_cell(buffer, row, col, '\u{00b7}', Color::DarkGray); // middot
        }
        col += 2; // 1 for symbol/middot, 1 for spacing
    }
}

/// Total display width consumed by infra slots (4 slots x 2 cols each, minus trailing space).
const INFRA_SLOTS_WIDTH: i32 = 7;

/// Symbol + color for layer progression state in the descent shaft.
fn shaft_status(layer_index: u32, frontier: u32, cleared: bool) -> (&'static str, Color) {
    if cleared {
        ("\u{25c6}", Color::Green) // ◆
    } else if layer_index == frontier {
        ("\u{25b6}", Color::Yellow) // ▶
    } else {
        ("\u{2592}", Color::DarkGray) // ▒
    }
}

/// Atmospheric flavor for selected layer.
fn layer_atmosphere_text(tier: LayerTier, cleared: bool, is_frontier: bool) -> &'static str {
    if !cleared && !is_frontier {
        return "Uncharted corridors. No reliable maps exist yet.";
    }
    if is_frontier {
        return "Your newest foothold. The stone still feels unsettled.";
    }
    match tier {
        LayerTier::Shallows => "Cold draft channels through narrow limestone halls.",
        LayerTier::Warrens => "Old tool marks and collapsed camps line the tunnels.",
        LayerTier::Hollows => "Bioluminescent spores drift above blackwater seams.",
        LayerTier::SunkenReach => "Pressure hums behind sealed stone and rusted gates.",
        LayerTier::Abyss => "Sound thins and distance no longer behaves normally.",
        LayerTier::Void => "Light is swallowed before it reaches the floor.",
    }
}

/// Format seconds as compact hours string "X.Xh".
fn format_hours(secs: u64) -> String {
    format!("{:.1}h", secs as f64 / 3600.0)
}

/// Render a vertical depth gauge at column 0.
///
/// Filled blocks for explored depth, empty blocks for unexplored.
/// Color follows tier color of each depth position.
fn render_depth_gauge(
    buffer: &mut [Vec<SceneCell>],
    content_top: i32,
    content_bottom: i32,
    deepest_layer: u32,
) {
    const VOID_START: u32 = 26;
    let gauge_height = (content_bottom - content_top - 2).max(3) as usize;
    let total_layers = VOID_START; // gauge maps 1..=25 to the bar

    for i in 0..gauge_height {
        let row = content_top + i as i32;
        if row >= content_bottom {
            break;
        }
        // Map gauge position to approximate layer depth
        let layer_at = ((i as u32 + 1) * total_layers / gauge_height as u32).max(1);
        let tier = LayerTier::from_layer(layer_at);
        let tc = layer_tier_color(tier);

        if layer_at <= deepest_layer {
            put_cell(buffer, row, 0, '\u{2588}', tc);
        } else {
            put_cell(buffer, row, 0, '\u{2591}', Color::Rgb(20, 30, 50));
        }
    }

    // Depth summary at bottom of gauge
    let pct = ((deepest_layer as f64 / VOID_START as f64) * 100.0)
        .round()
        .min(100.0) as u32;
    let depth_label = format!("{}% to Void", pct);
    let summary_row = content_top + gauge_height as i32;
    if summary_row < content_bottom {
        put_text(buffer, summary_row, 0, &depth_label, Color::DarkGray);
    }
}

/// Layer range for a tier (start, end inclusive).
fn tier_layer_range(tier: LayerTier) -> (u32, u32) {
    match tier {
        LayerTier::Shallows => (1, 3),
        LayerTier::Warrens => (4, 7),
        LayerTier::Hollows => (8, 12),
        LayerTier::SunkenReach => (13, 18),
        LayerTier::Abyss => (19, 25),
        LayerTier::Void => (26, 99),
    }
}

/// Render a danger profile card with mission threshold bars.
fn render_danger_profile_card(
    buffer: &mut [Vec<SceneCell>],
    left: i32,
    inner_w: usize,
    top: i32,
    bottom_limit: i32,
    layer_index: u32,
    tier: LayerTier,
) -> i32 {
    if inner_w < 20 || top + 7 >= bottom_limit {
        return top;
    }

    let card_left = left;
    let card_right = left + inner_w as i32 - 1;
    let card_bottom = (top + 7).min(bottom_limit - 1);
    draw_deep_card(
        buffer,
        card_left,
        top,
        card_right,
        card_bottom,
        Color::Rgb(70, 130, 190),
        Color::Rgb(7, 14, 28),
        Some("DANGER PROFILE"),
    );

    let threat_label = match tier {
        LayerTier::Shallows => "Threat Level: Controlled",
        LayerTier::Warrens => "Threat Level: Elevated",
        LayerTier::Hollows => "Threat Level: Hostile",
        LayerTier::SunkenReach => "Threat Level: Severe",
        LayerTier::Abyss => "Threat Level: Extreme",
        LayerTier::Void => "Threat Level: Catastrophic",
    };
    let threat_w = (card_right - card_left - 3).max(1) as usize;
    let threat_line = truncate_text(threat_label, threat_w);
    put_text(
        buffer,
        top + 1,
        card_left + 2,
        &threat_line,
        layer_tier_color(tier),
    );

    let hint_w = (card_right - card_left - 3).max(1) as usize;
    let hint_line = truncate_text("Numbers = recommended squad power", hint_w);
    put_text(buffer, top + 2, card_left + 2, &hint_line, Color::DarkGray);

    let thresholds = layer_power_thresholds(layer_index);
    let max_power = thresholds.breakthrough.max(1) as f64;
    let use_short_labels = inner_w < 40;
    let label_width = if use_short_labels { 6 } else { 12 };
    let rows = [
        (
            if use_short_labels {
                "Supply"
            } else {
                "Supply Run"
            },
            thresholds.supply_run,
            Color::Green,
        ),
        ("Recon", thresholds.recon, Color::Cyan),
        (
            if use_short_labels {
                "Exped."
            } else {
                "Expedition"
            },
            thresholds.expedition,
            Color::Yellow,
        ),
        (
            if use_short_labels {
                "Break."
            } else {
                "Breakthrough"
            },
            thresholds.breakthrough,
            Color::LightRed,
        ),
    ];

    for (idx, (label, value, bar_color)) in rows.iter().enumerate() {
        let row = top + 3 + idx as i32;
        if row >= card_bottom {
            break;
        }
        let info = format!(
            "{:<label_width$} {:>4}",
            label,
            value,
            label_width = label_width
        );
        put_text(buffer, row, card_left + 2, &info, Color::White);
        let bar_col = card_left + 3 + info.chars().count() as i32;
        let bar_w = (card_right - bar_col - 1).max(6) as usize;
        render_card_progress_bar(
            buffer,
            row,
            bar_col,
            bar_w,
            *value as f64 / max_power,
            *bar_color,
        );
    }

    card_bottom + 1
}

pub(super) fn render_layers(
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

    let frontier = deep.persistent.frontier_layer();
    let deepest = deep.persistent.deepest_layer_reached;

    // ── Title ──
    put_text(buffer, 0, 1, "LAYERS", DEEP_BORDER_COLOR);
    let header_str = format!(
        "Frontier: Layer {}    Deepest Descent: Layer {}",
        frontier,
        deepest.max(1)
    );
    put_text(
        buffer,
        0,
        9,
        &truncate_text(&header_str, width.saturating_sub(10)),
        Color::DarkGray,
    );

    // ── Infrastructure legend ──
    let legend = "\u{25c6}=Cleared  \u{25b6}=Frontier  \u{2592}=Unknown    [OCWB]=Infrastructure";
    let legend_col = (width as i32 - legend.len() as i32 - 1).max(1);
    if height as i32 - 2 > 1 {
        put_text(
            buffer,
            height as i32 - 2,
            legend_col,
            &truncate_text(legend, width.saturating_sub(2)),
            Color::Rgb(50, 70, 100),
        );
    }

    // ── Footer ──
    put_text(
        buffer,
        height as i32 - 1,
        1,
        &truncate_text(
            "[\u{2191}/\u{2193}] Navigate Layers  [Esc] Close",
            width.saturating_sub(2),
        ),
        Color::DarkGray,
    );
    let help_hint = "[?] Help";
    let help_col = (width as i32 - help_hint.len() as i32 - 1).max(1);
    put_text(
        buffer,
        height as i32 - 1,
        help_col,
        help_hint,
        Color::Rgb(50, 70, 100),
    );

    let content_top = 1i32;
    let content_bottom = height as i32 - 1;

    if deep.persistent.layers.is_empty() {
        put_text_centered(
            buffer,
            content_top + (content_bottom - content_top) / 2,
            width,
            "No layers explored yet.",
            Color::DarkGray,
        );
        put_text_centered(
            buffer,
            content_top + (content_bottom - content_top) / 2 + 1,
            width,
            "Complete your first mission to reveal Layer 1.",
            Color::Rgb(50, 70, 100),
        );
        return;
    }

    let is_compact = ctx.tier <= SizeTier::S || width < 60;

    if is_compact {
        render_layers_compact(
            buffer,
            width,
            height,
            deep,
            ui,
            content_top,
            content_bottom,
            frontier,
        );
    } else {
        render_layers_split(
            buffer,
            width,
            height,
            deep,
            ui,
            content_top,
            content_bottom,
            frontier,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_layers_compact(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    _height: usize,
    deep: &DeepState,
    ui: &DeepUiState,
    content_top: i32,
    content_bottom: i32,
    frontier: u32,
) {
    let mut row = content_top;
    let mut last_tier: Option<LayerTier> = None;

    if row < content_bottom {
        put_text(buffer, row, 1, "SURFACE", Color::Rgb(70, 95, 125));
        row += 1;
    }

    for (i, layer) in deep.persistent.layers.iter().enumerate() {
        if row >= content_bottom {
            break;
        }
        let tier = LayerTier::from_layer(layer.index);
        let tc = layer_tier_color(tier);

        // Tier section header when tier changes
        if last_tier != Some(tier) {
            let sep_w = width.saturating_sub(2).min(40);
            let (lr_start, lr_end) = tier_layer_range(tier);
            let header = format!(" {} (L{}-{}) ", tier.display_name(), lr_start, lr_end);
            let sep_left = sep_w.saturating_sub(header.len()) / 2;
            let sep_right = sep_w.saturating_sub(header.len() + sep_left);
            let sep_line = format!(
                "{}{}{}",
                "\u{2550}".repeat(sep_left),
                header,
                "\u{2550}".repeat(sep_right)
            );
            put_text(buffer, row, 1, &sep_line, Color::Rgb(40, 60, 80));
            // Tier name in tier color
            put_text(buffer, row, 1 + sep_left as i32, &header, tc);
            row += 1;
            last_tier = Some(tier);
            if row >= content_bottom {
                break;
            }
        }

        let is_sel = i == ui.selected_index;
        let (status_glyph, status_color) = shaft_status(layer.index, frontier, layer.cleared);
        let shaft_color = if layer.index <= frontier {
            Color::Rgb(60, 90, 130)
        } else {
            Color::Rgb(30, 45, 70)
        };

        let tier_name: String = tier.display_name().chars().take(16).collect();
        let prefix = format!(
            "\u{2502} {} L{:02}  {:16}  ",
            status_glyph, layer.index, tier_name,
        );
        let infra_col = 1 + prefix.len() as i32;

        if is_sel {
            for c in 1..width.saturating_sub(1) {
                if row >= 0 && (row as usize) < buffer.len() && c < buffer[row as usize].len() {
                    buffer[row as usize][c].bg = Color::Rgb(10, 22, 38);
                }
            }
        }

        put_text(buffer, row, 1, &prefix, Color::White);
        put_text(buffer, row, 1, "\u{2502}", shaft_color);
        put_text(buffer, row, 3, status_glyph, status_color);
        if is_sel {
            put_text(buffer, row, 3, status_glyph, Color::Cyan);
        }
        // Layer number in tier color
        put_text(buffer, row, 5, &format!("L{:02}", layer.index), tc);
        // Infrastructure emoji columns
        render_infra_slots(buffer, row, infra_col, layer);
        // Frontier label at end of line
        if layer.index == frontier && !layer.cleared {
            let front_col = infra_col + INFRA_SLOTS_WIDTH + 2;
            put_text(buffer, row, front_col, "FRONTIER", Color::Yellow);
        }
        row += 1;
    }

    // Hint for one layer beyond cleared (frontier preview)
    let next_unknown = deep.persistent.deepest_layer_reached + 1;
    if row < content_bottom {
        let tier = LayerTier::from_layer(next_unknown);
        let tc = layer_tier_color(tier);
        put_text(
            buffer,
            row,
            1,
            &format!("\u{2502} \u{2592} L{:02}  UNCHARTED DEPTH", next_unknown),
            Color::DarkGray,
        );
        put_text(buffer, row, 5, &format!("L{:02}", next_unknown), tc);
        row += 1;
    }

    if row < content_bottom {
        put_text(
            buffer,
            row,
            1,
            "\u{2514}\u{2500} deeper darkness below...",
            Color::Rgb(45, 65, 95),
        );
    }

    // Depth gauge at column 0
    render_depth_gauge(
        buffer,
        content_top,
        content_bottom,
        deep.persistent.deepest_layer_reached,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_layers_split(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    _height: usize,
    deep: &DeepState,
    ui: &DeepUiState,
    content_top: i32,
    content_bottom: i32,
    frontier: u32,
) {
    let list_width = (width * 42 / 100).max(22).min(width.saturating_sub(20));
    let detail_left = list_width as i32;
    let detail_inner_left = detail_left + 1;
    let detail_inner_w = (width as i32 - detail_left - 2).max(0) as usize;

    // Inner divider
    let glyphs = super::panel_border_chars();
    for r in content_top..content_bottom {
        put_cell(buffer, r, detail_left, glyphs.v, Color::Rgb(40, 60, 80));
    }

    // Left: layer list with tier section headers
    let mut row = content_top;
    let mut last_tier: Option<LayerTier> = None;

    if row < content_bottom {
        put_text(buffer, row, 1, "SURFACE", Color::Rgb(70, 95, 125));
        row += 1;
    }

    for (i, layer) in deep.persistent.layers.iter().enumerate() {
        if row >= content_bottom {
            break;
        }
        let tier = LayerTier::from_layer(layer.index);
        let tc = layer_tier_color(tier);

        // Tier section header when tier changes
        if last_tier != Some(tier) {
            let max_sep = list_width.saturating_sub(2).min(42);
            let sep_line: String = "\u{2550}".repeat(max_sep);
            put_text(buffer, row, 1, &sep_line, Color::Rgb(40, 60, 80));
            let tier_label = format!(" {} ", tier.display_name());
            put_text(buffer, row, 2, &tier_label, tc);
            row += 1;
            last_tier = Some(tier);
            if row >= content_bottom {
                break;
            }
        }

        let is_sel = i == ui.selected_index;
        let (status_glyph, status_color) = shaft_status(layer.index, frontier, layer.cleared);
        let shaft_color = if layer.index <= frontier {
            Color::Rgb(60, 90, 130)
        } else {
            Color::Rgb(30, 45, 70)
        };

        let tier_name: String = tier.display_name().chars().take(16).collect();
        let prefix = format!(
            "\u{2502} {} L{:02}  {:16}  ",
            status_glyph, layer.index, tier_name,
        );
        let infra_col = 1 + prefix.len() as i32;

        if is_sel {
            for c in 1..(list_width.saturating_sub(1)) {
                if row >= 0 && (row as usize) < buffer.len() && c < buffer[row as usize].len() {
                    buffer[row as usize][c].bg = Color::Rgb(10, 22, 38);
                }
            }
        }

        put_text(buffer, row, 1, &prefix, Color::White);
        put_text(buffer, row, 1, "\u{2502}", shaft_color);
        put_text(buffer, row, 3, status_glyph, status_color);
        if is_sel {
            put_text(buffer, row, 3, status_glyph, Color::Cyan);
        }
        // Layer number in tier color
        put_text(buffer, row, 5, &format!("L{:02}", layer.index), tc);
        // Infrastructure emoji columns
        render_infra_slots(buffer, row, infra_col, layer);
        // Frontier tag
        if layer.index == frontier && !layer.cleared {
            let front_col = infra_col + INFRA_SLOTS_WIDTH + 1;
            if front_col < list_width as i32 - 2 {
                put_text(buffer, row, front_col, "FRONT", Color::Yellow);
            }
        }
        // Guild rank breakthrough target marker
        if let Some(next_rank) = deep.persistent.guild_rank.next() {
            if let Some(needed_layer) = next_rank.required_breakthrough_layer() {
                if layer.index == needed_layer && !layer.cleared {
                    let marker = format!("\u{2605} Rank {}", next_rank.0);
                    let marker_col = (list_width as i32 - marker.len() as i32 - 1)
                        .max(infra_col + INFRA_SLOTS_WIDTH + 1);
                    if marker_col < list_width as i32 {
                        put_text(buffer, row, marker_col, &marker, Color::Rgb(255, 215, 0));
                    }
                }
            }
        }
        row += 1;
    }

    // Frontier unknown row
    if row < content_bottom {
        let next = deep.persistent.deepest_layer_reached + 1;
        let tier = LayerTier::from_layer(next);
        let tc = layer_tier_color(tier);
        put_text(
            buffer,
            row,
            1,
            &format!("\u{2502} \u{2592} L{:02}  UNCHARTED DEPTH", next),
            Color::DarkGray,
        );
        put_text(buffer, row, 5, &format!("L{:02}", next), tc);
        row += 1;
    }

    if row < content_bottom {
        put_text(
            buffer,
            row,
            1,
            "\u{2514}\u{2500} deeper darkness below...",
            Color::Rgb(45, 65, 95),
        );
    }

    // Depth gauge at column 0
    render_depth_gauge(
        buffer,
        content_top,
        content_bottom,
        deep.persistent.deepest_layer_reached,
    );

    // Right: layer detail for selected
    let Some(layer) = deep.persistent.layers.get(ui.selected_index) else {
        return;
    };

    let tier = LayerTier::from_layer(layer.index);
    let tc = layer_tier_color(tier);
    let mut row = content_top;

    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("Layer {} \u{2014} {}", layer.index, tier.display_name()),
        tc,
    );
    row += 1;

    let (status_str, status_color) = if layer.cleared {
        ("Cleared", Color::Green)
    } else if layer.index == frontier {
        ("FRONTIER", Color::Yellow)
    } else {
        ("Unknown", Color::DarkGray)
    };
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("{}  \u{00b7}  {}", tier.display_name(), status_str),
        Color::DarkGray,
    );
    put_text(
        buffer,
        row,
        detail_inner_left + tier.display_name().len() as i32 + 5,
        status_str,
        status_color,
    );
    row += 1;

    if row < content_bottom {
        let ambience = layer_atmosphere_text(tier, layer.cleared, layer.index == frontier);
        put_text(
            buffer,
            row,
            detail_inner_left,
            &format!("\"{}\"", ambience),
            Color::Rgb(70, 90, 120),
        );
        row += 1;
    }

    // Danger profile card
    row = render_danger_profile_card(
        buffer,
        detail_inner_left,
        detail_inner_w,
        row,
        content_bottom,
        layer.index,
        tier,
    );

    // Familiarity with named tier and effect
    row += 1;
    let fam_level = FamiliarityLevel::from_familiarity(layer.familiarity);
    let fam_color = familiarity_color(layer.familiarity);
    let fam_label = format!(
        "Familiarity: {} ({}%)",
        fam_level.display_name(),
        layer.familiarity
    );
    put_text(buffer, row, detail_inner_left, &fam_label, fam_color);
    row += 1;

    // Familiarity bar with thresholds
    let bar_col = detail_inner_left + 2;
    let bar_width = detail_inner_w.saturating_sub(4).min(20);
    render_familiarity_bar_with_thresholds(buffer, row, bar_col, bar_width, layer.familiarity);
    row += 2; // bar + threshold markers

    // Effect text
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("  Effect: {}", familiarity_effect_text(layer.familiarity)),
        Color::DarkGray,
    );
    row += 1;

    // Power thresholds for frontier layers
    if layer.index == frontier && !layer.cleared && row < content_bottom - 5 {
        row += 1;
        put_text(
            buffer,
            row,
            detail_inner_left,
            "Power Required:",
            Color::Cyan,
        );
        row += 1;
        let thresholds = layer_power_thresholds(layer.index);
        let power_lines = [
            (
                "Supply Run:",
                thresholds.supply_run,
                "safest Warband Marks income",
            ),
            ("Recon:", thresholds.recon, "boosts familiarity/speed"),
            (
                "Expedition:",
                thresholds.expedition,
                "higher Warband Marks, medium risk",
            ),
            (
                "Breakthrough:",
                thresholds.breakthrough,
                "clears this layer",
            ),
        ];
        for (label, value, desc) in power_lines {
            if row >= content_bottom {
                break;
            }
            put_text(
                buffer,
                row,
                detail_inner_left,
                &format!("  {:14} {:4}  ({})", label, value, desc),
                Color::White,
            );
            row += 1;
        }
    }

    // Infrastructure — single merged list: built items in green, unbuilt in gray with cost
    row += 1;
    if row >= content_bottom {
        return;
    }
    let built_count = layer.infrastructure.len();
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("Infrastructure  [{}/4]", built_count),
        Color::Cyan,
    );
    row += 1;

    let bridge_count = deep
        .persistent
        .layers
        .iter()
        .filter(|l| l.has_infrastructure(Infrastructure::Bridge))
        .count();

    for infra in Infrastructure::ALL {
        if row >= content_bottom {
            break;
        }
        let built = layer.has_infrastructure(*infra);
        let icon_col = detail_inner_left + 2;
        let text_col = icon_col + 2;

        if built {
            // Built: green icon + active effect description
            put_cell(buffer, row, icon_col, infra.icon(), Color::Green);
            let effect = active_infra_effect(*infra, bridge_count);
            let line = format!("{:12}  {}", infra.display_name(), effect);
            put_text(buffer, row, text_col, &line, Color::Green);
        } else {
            // Unbuilt: gray icon + description
            put_cell(buffer, row, icon_col, infra.icon(), Color::DarkGray);
            let desc = short_infra_desc(*infra);
            let line = format!("{:12}  {}", infra.display_name(), desc);
            put_text(buffer, row, text_col, &line, Color::DarkGray);
        }
        row += 1;
    }

    // Duration section for cleared layers
    if layer.cleared && row < content_bottom - 3 {
        row += 1;

        // Build modifier summary.
        let fam_level = FamiliarityLevel::from_familiarity(layer.familiarity);
        let has_outpost = layer.has_infrastructure(Infrastructure::Outpost);
        let bridge_count = (1..layer.index)
            .filter(|l| {
                deep.persistent
                    .layer_record(*l)
                    .is_some_and(|r| r.has_infrastructure(Infrastructure::Bridge))
            })
            .count() as u32;

        let mut modifiers: Vec<String> = Vec::new();
        if has_outpost {
            modifiers.push(format!("-25% {} Outpost", Infrastructure::Outpost.icon()));
        }
        let fam_reduction = ((1.0 - fam_level.duration_factor()) * 100.0).round() as u32;
        if fam_reduction > 0 {
            modifiers.push(format!("-{}% Familiarity", fam_reduction));
        }
        let bridge_pct = bridge_count.min(15) * 2;
        if bridge_pct > 0 {
            modifiers.push(format!(
                "-{}% {} Bridge",
                bridge_pct,
                Infrastructure::Bridge.icon()
            ));
        }

        if modifiers.is_empty() {
            put_text(
                buffer,
                row,
                detail_inner_left,
                "Mission Durations:",
                Color::Cyan,
            );
        } else {
            let header = format!("Mission Durations:  ({})", modifiers.join(", "));
            // Render "Mission Durations:" in cyan, modifiers in green.
            put_text(
                buffer,
                row,
                detail_inner_left,
                "Mission Durations:",
                Color::Cyan,
            );
            let mod_text = format!("  ({})", modifiers.join(", "));
            put_text(
                buffer,
                row,
                detail_inner_left + "Mission Durations:".len() as i32,
                &mod_text,
                Color::Green,
            );
            let _ = header; // suppress unused
        }
        row += 1;

        let has_any_modifier = has_outpost || fam_reduction > 0 || bridge_pct > 0;

        let mission_types = [
            ("Supply Run:", MissionType::SupplyRun),
            ("Recon:", MissionType::Recon),
            ("Expedition:", MissionType::Expedition),
        ];
        for (label, mt) in mission_types {
            if row >= content_bottom {
                break;
            }
            let effective = effective_duration_secs(tier, mt, layer.index, &deep.persistent);
            if has_any_modifier {
                let table = crate::deep::mission_duration_secs(tier, mt);
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    &format!(
                        "  {:12} {} \u{2192} {}",
                        label,
                        format_hours(table),
                        format_hours(effective)
                    ),
                    Color::White,
                );
            } else {
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    &format!("  {:12} {}", label, format_hours(effective)),
                    Color::White,
                );
            }
            row += 1;
        }
    }
}

fn short_infra_desc(infra: Infrastructure) -> &'static str {
    match infra {
        Infrastructure::Outpost => "-25% duration on this layer",
        Infrastructure::SupplyCache => "+50% Warband Marks from supply runs",
        Infrastructure::Watchtower => "+40 familiarity instantly",
        Infrastructure::Bridge => "-2% duration per bridge (max -30%)",
    }
}

/// Active effect text for built infrastructure (contextual for Bridge).
fn active_infra_effect(infra: Infrastructure, total_bridges: usize) -> String {
    match infra {
        Infrastructure::Outpost => "-25% duration".to_string(),
        Infrastructure::SupplyCache => "+50% Marks on supply runs".to_string(),
        Infrastructure::Watchtower => "+40 familiarity granted".to_string(),
        Infrastructure::Bridge => {
            let pct = (total_bridges as u32).min(15) * 2;
            format!("-{}% total ({} bridge{})", pct, total_bridges, if total_bridges == 1 { "" } else { "s" })
        }
    }
}
