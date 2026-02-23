//! The Deep — Layer infrastructure sub-view rendering.

use crate::deep::{DeepState, DeepUiState, Infrastructure, LayerTier};
use ratatui::style::Color;

use super::deep_scene::DEEP_BORDER_COLOR;
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
fn render_familiarity_bar(buffer: &mut [Vec<SceneCell>], row: i32, col: i32, width: usize, familiarity: u8) {
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
        put_cell(buffer, row, col + i as i32, '\u{2591}', Color::Rgb(30, 40, 60));
    }
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
    put_text(
        buffer,
        0,
        9,
        &format!(
            "Frontier: Layer {}    Deepest ever: Layer {}",
            frontier,
            deepest.max(1)
        ),
        Color::DarkGray,
    );

    // ── Footer ──
    put_text(
        buffer,
        height as i32 - 1,
        1,
        "[\u{2191}/\u{2193}] Navigate Layers  [Esc] Back",
        Color::DarkGray,
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
        render_layers_compact(buffer, width, height, deep, ui, content_top, content_bottom, frontier);
    } else {
        render_layers_split(buffer, width, height, deep, ui, content_top, content_bottom, frontier);
    }
}

fn render_layers_compact(
    buffer: &mut [Vec<SceneCell>],
    _width: usize,
    _height: usize,
    deep: &DeepState,
    ui: &DeepUiState,
    content_top: i32,
    content_bottom: i32,
    frontier: u32,
) {
    let mut row = content_top;

    for (i, layer) in deep.persistent.layers.iter().enumerate() {
        if row >= content_bottom {
            break;
        }
        let is_sel = i == ui.selected_index;
        let cursor = if is_sel { "\u{25b6} " } else { "  " };
        let tier = LayerTier::from_layer(layer.index);
        let tc = layer_tier_color(tier);

        let status_str = if layer.cleared {
            "CLEAR"
        } else if layer.index == frontier {
            "FRONT"
        } else {
            "???"
        };
        let status_color = if layer.cleared {
            Color::Green
        } else if layer.index == frontier {
            Color::Yellow
        } else {
            Color::DarkGray
        };

        let infra_count = layer.infrastructure.len();
        let line = format!(
            "{}L{:2}  {:12}  {}  [{}/4]",
            cursor,
            layer.index,
            tier.display_name().chars().take(12).collect::<String>(),
            status_str,
            infra_count,
        );
        put_text(buffer, row, 1, &line, Color::White);
        put_text(buffer, row, 1, cursor, if is_sel { Color::Cyan } else { Color::DarkGray });
        // Status color
        let stat_col = 1 + 2 + 4 + 14 + 2;
        put_text(buffer, row, stat_col, status_str, status_color);
        let _ = tc;
        row += 1;
    }

    // Hint for one layer beyond cleared (frontier preview)
    let next_unknown = deep.persistent.deepest_layer_reached + 1;
    if row < content_bottom {
        let tier = LayerTier::from_layer(next_unknown);
        let tc = layer_tier_color(tier);
        put_text(buffer, row, 1, &format!("  L{:2}  ???", next_unknown), tc);
    }
}

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

    // Left: layer list
    put_text(buffer, content_top, 1, " #   Tier              Status", Color::DarkGray);
    let list_inner_top = content_top + 1;

    for (i, layer) in deep.persistent.layers.iter().enumerate() {
        let row = list_inner_top + i as i32;
        if row >= content_bottom {
            break;
        }
        let is_sel = i == ui.selected_index;
        let cursor = if is_sel { "\u{25b6} " } else { "  " };
        let tier = LayerTier::from_layer(layer.index);
        let tc = layer_tier_color(tier);

        let (status_str, status_color) = if layer.cleared {
            ("[CLEAR]", Color::Green)
        } else if layer.index == frontier {
            ("[FRONT]", Color::Yellow)
        } else {
            ("[?????]", Color::DarkGray)
        };

        let tier_abbrev: String = tier.display_name().chars().take(14).collect();
        let line = format!(
            "{}L{:2}  {:14}  {}",
            cursor,
            layer.index,
            tier_abbrev,
            status_str,
        );
        put_text(buffer, row, 1, &line, Color::White);
        put_text(buffer, row, 1, cursor, if is_sel { Color::Cyan } else { Color::DarkGray });
        // Layer number in tier color
        put_text(buffer, row, 3, &format!("L{:2}", layer.index), tc);
        // Status in status color
        let stat_col = 1 + 2 + 4 + 16 + 2;
        put_text(buffer, row, stat_col, status_str, status_color);
    }

    // Frontier unknown row
    let unknown_row = list_inner_top + deep.persistent.layers.len() as i32;
    if unknown_row < content_bottom {
        let next = deep.persistent.deepest_layer_reached + 1;
        let tier = LayerTier::from_layer(next);
        let tc = layer_tier_color(tier);
        put_text(buffer, unknown_row, 3, &format!("L{:2}  {}", next, "[?????]"), tc);
    }

    // Right: layer detail for selected
    let Some(layer) = deep.persistent.layers.get(ui.selected_index) else {
        return;
    };

    let tier = LayerTier::from_layer(layer.index);
    let tc = layer_tier_color(tier);
    let mut row = content_top;

    put_text(buffer, row, detail_inner_left, &format!("Layer {} \u{2014} {}", layer.index, tier.display_name()), tc);
    row += 1;

    let status_str = if layer.cleared {
        "Cleared"
    } else if layer.index == frontier {
        "Frontier (active)"
    } else {
        "Unknown"
    };
    put_text(buffer, row, detail_inner_left, &format!("Status: {}", status_str), Color::DarkGray);
    row += 1;

    // Familiarity bar
    let bar_label = format!("Intel:  {:3}%  ", layer.familiarity);
    put_text(buffer, row, detail_inner_left, &bar_label, Color::DarkGray);
    let bar_start = detail_inner_left + bar_label.len() as i32;
    let bar_width = detail_inner_w.saturating_sub(bar_label.len() + 1).min(20);
    render_familiarity_bar(buffer, row, bar_start, bar_width, layer.familiarity);
    row += 1;

    row += 1;
    put_text(buffer, row, detail_inner_left, "Infrastructure:", Color::Cyan);
    row += 1;

    for infra in Infrastructure::ALL {
        if row >= content_bottom {
            break;
        }
        let built = layer.has_infrastructure(*infra);
        let check = if built { "[\u{2713}]" } else { "[ ]" };
        let check_color = if built { Color::Green } else { Color::DarkGray };
        let desc_color = if built { Color::White } else { Color::DarkGray };
        put_text(buffer, row, detail_inner_left, check, check_color);
        put_text(
            buffer,
            row,
            detail_inner_left + 4,
            &format!(
                "{:12}  {}",
                infra.display_name(),
                short_infra_desc(*infra)
            ),
            desc_color,
        );
        row += 1;
    }
}

fn short_infra_desc(infra: Infrastructure) -> &'static str {
    match infra {
        Infrastructure::Outpost => "-25% duration",
        Infrastructure::SupplyCache => "+yield",
        Infrastructure::Watchtower => "+intel",
        Infrastructure::Bridge => "shortcut",
    }
}
