//! The Deep — Layer infrastructure sub-view rendering.

use crate::deep::{
    base_marks_earned, base_mission_duration_secs, infrastructure_build_cost,
    layer_power_thresholds, DeepState, DeepUiState, FamiliarityLevel, Infrastructure, LayerTier,
    MissionType,
};
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
        FamiliarityLevel::Mastered => "-30% duration, +15% Mark yield",
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

/// Infrastructure slot display: "[OCWB]" where each letter is present or space.
fn infra_slots_str(layer: &crate::deep::types::LayerRecord) -> String {
    let o = if layer.has_infrastructure(Infrastructure::Outpost) {
        'O'
    } else {
        ' '
    };
    let c = if layer.has_infrastructure(Infrastructure::SupplyCache) {
        'C'
    } else {
        ' '
    };
    let w = if layer.has_infrastructure(Infrastructure::Watchtower) {
        'W'
    } else {
        ' '
    };
    let b = if layer.has_infrastructure(Infrastructure::Bridge) {
        'B'
    } else {
        ' '
    };
    format!("[{}{}{}{}]", o, c, w, b)
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
        let status_glyph = if layer.cleared {
            "\u{2713}"
        } else if layer.index == frontier {
            "\u{25b6}"
        } else {
            "?"
        };
        let status_color = if layer.cleared {
            Color::Green
        } else if layer.index == frontier {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let infra_str = infra_slots_str(layer);
        let line = format!(
            "{}  L{:2}  {:12}  {}",
            status_glyph,
            layer.index,
            tier.display_name().chars().take(12).collect::<String>(),
            infra_str,
        );
        put_text(buffer, row, 1, &line, Color::White);
        put_text(buffer, row, 1, status_glyph, status_color);
        if is_sel {
            put_text(buffer, row, 1, status_glyph, Color::Cyan);
        }
        // Layer number in tier color
        put_text(buffer, row, 4, &format!("L{:2}", layer.index), tc);
        // Frontier label at end of line
        if layer.index == frontier && !layer.cleared {
            let front_col = 1 + line.len() as i32 + 2;
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
            &format!("?  L{:2}  ???", next_unknown),
            Color::DarkGray,
        );
        put_text(buffer, row, 4, &format!("L{:2}", next_unknown), tc);
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
        let status_glyph = if layer.cleared {
            "\u{2713}"
        } else if layer.index == frontier {
            "\u{25b6}"
        } else {
            "?"
        };
        let status_color = if layer.cleared {
            Color::Green
        } else if layer.index == frontier {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let infra_str = infra_slots_str(layer);
        let line = format!(
            "{}  L{:2}  {:14}  {}",
            status_glyph,
            layer.index,
            tier.display_name().chars().take(14).collect::<String>(),
            infra_str,
        );
        put_text(buffer, row, 1, &line, Color::White);
        put_text(buffer, row, 1, status_glyph, status_color);
        if is_sel {
            put_text(buffer, row, 1, status_glyph, Color::Cyan);
        }
        // Layer number in tier color
        put_text(buffer, row, 4, &format!("L{:2}", layer.index), tc);
        // Frontier tag
        if layer.index == frontier && !layer.cleared {
            let front_col = 1 + line.len() as i32 + 1;
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
                        .max(1 + line.len() as i32 + 1);
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
            &format!("?  L{:2}  ???", next),
            Color::DarkGray,
        );
        put_text(buffer, row, 4, &format!("L{:2}", next), tc);
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

    // Total duration reduction summary
    let total_reduction = layer.total_duration_reduction();
    if total_reduction > 0.001 && row < content_bottom {
        let total_pct = (total_reduction * 100.0).round() as u32;
        let has_outpost = layer.has_infrastructure(Infrastructure::Outpost);
        let fam_reduction = match FamiliarityLevel::from_familiarity(layer.familiarity) {
            FamiliarityLevel::Unknown => 0,
            FamiliarityLevel::Mapped => 10,
            FamiliarityLevel::Familiar => 20,
            FamiliarityLevel::Mastered => 30,
        };
        let mut breakdown = Vec::new();
        if has_outpost {
            breakdown.push("Outpost -25%");
        }
        if fam_reduction > 0 {
            breakdown.push(match fam_reduction {
                10 => "Mapped -10%",
                20 => "Familiar -20%",
                30 => "Mastered -30%",
                _ => "",
            });
        }
        let breakdown_str = if breakdown.is_empty() {
            String::new()
        } else {
            format!("  ({})", breakdown.join("  "))
        };
        put_text(
            buffer,
            row,
            detail_inner_left,
            &format!("  Duration reduction: -{}%{}", total_pct, breakdown_str),
            Color::Cyan,
        );
        let breakdown_col =
            detail_inner_left + format!("  Duration reduction: -{}%", total_pct).len() as i32;
        put_text(buffer, row, breakdown_col, &breakdown_str, Color::DarkGray);
        row += 1;
    }

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
            ("Supply Run:", thresholds.supply_run, "safe farming"),
            ("Recon:", thresholds.recon, "low risk, build intel"),
            (
                "Expedition:",
                thresholds.expedition,
                "medium risk, primary XP",
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

    // Infrastructure
    row += 1;
    if row >= content_bottom {
        return;
    }
    let built_count = layer.infrastructure.len();
    put_text(
        buffer,
        row,
        detail_inner_left,
        &format!("Infrastructure  [{}/4 built]", built_count),
        Color::Cyan,
    );
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
        let desc_text = format!("{:12}  {}", infra.display_name(), short_infra_desc(*infra));
        put_text(buffer, row, detail_inner_left + 4, &desc_text, desc_color);
        // Show cost for unbuilt infrastructure with affordability color
        if !built {
            let cost = infrastructure_build_cost(*infra, layer.index);
            let cost_str = format!("{}M", cost);
            let cost_col = detail_inner_left + 4 + desc_text.len() as i32 + 2;
            let cost_color = if deep.prestige.warband_marks >= cost {
                Color::Green
            } else {
                Color::LightRed
            };
            if cost_col < width as i32 - cost_str.len() as i32 - 1 {
                put_text(buffer, row, cost_col, &cost_str, cost_color);
            }
        }
        row += 1;
        // ROI context line for unbuilt infrastructure
        if !built && row < content_bottom {
            let roi_hint = match infra {
                Infrastructure::Outpost => "Saves ~25% time on every mission here".to_string(),
                Infrastructure::SupplyCache => {
                    let cost = infrastructure_build_cost(*infra, layer.index);
                    let base_supply = base_marks_earned(MissionType::SupplyRun, layer.index);
                    let extra_per_run = (base_supply as f64 * 0.75).round() as u32;
                    if extra_per_run > 0 {
                        let runs = (cost as f64 / extra_per_run as f64).ceil() as u32;
                        format!(
                            "~{} supply runs to recoup ({}M extra/run)",
                            runs, extra_per_run
                        )
                    } else {
                        "Boosts supply run yields".to_string()
                    }
                }
                Infrastructure::Watchtower => "Instant value: +25 intel on build".to_string(),
                Infrastructure::Bridge => {
                    let bridge_count = deep
                        .persistent
                        .layers
                        .iter()
                        .filter(|l| l.has_infrastructure(Infrastructure::Bridge))
                        .count();
                    if bridge_count == 0 {
                        "First bridge: -10% on deeper missions".to_string()
                    } else {
                        format!(
                            "Bridge #{}: compounds to -{:.0}% total",
                            bridge_count + 1,
                            (1.0 - (0.9_f64).powi((bridge_count + 1) as i32)) * 100.0
                        )
                    }
                }
            };
            put_text(
                buffer,
                row,
                detail_inner_left + 4,
                &roi_hint,
                Color::Rgb(60, 90, 130),
            );
            row += 1;
        }
    }

    // BUILD OPTIONS — list unbuilt infrastructure with ROI descriptions
    let unbuilt: Vec<&Infrastructure> = Infrastructure::ALL
        .iter()
        .filter(|i| !layer.has_infrastructure(**i))
        .collect();
    if !unbuilt.is_empty() && row + 2 < content_bottom {
        row += 1;
        put_text(
            buffer,
            row,
            detail_inner_left,
            "BUILD OPTIONS",
            DEEP_BORDER_COLOR,
        );
        row += 1;

        for infra in &unbuilt {
            if row >= content_bottom {
                break;
            }
            let cost = infrastructure_build_cost(**infra, layer.index);
            let roi = match infra {
                Infrastructure::Outpost => "-25% mission duration".to_string(),
                Infrastructure::SupplyCache => {
                    let runs = if cost > 0 { cost / 40 } else { 1 };
                    format!("~{} supply runs to break even", runs.max(1))
                }
                Infrastructure::Watchtower => "+25 familiarity immediately".to_string(),
                Infrastructure::Bridge => {
                    let bridge_count = deep
                        .persistent
                        .layers
                        .iter()
                        .filter(|l| l.has_infrastructure(Infrastructure::Bridge))
                        .count();
                    if bridge_count == 0 {
                        "Skip this layer on deeper missions (-10% duration)".to_string()
                    } else {
                        let new_reduction = 1.0 - (0.9_f64).powi((bridge_count + 1) as i32);
                        format!(
                            "Skip layer (-{:.0}% total with {} bridge{})",
                            new_reduction * 100.0,
                            bridge_count + 1,
                            if bridge_count == 0 { "" } else { "s" },
                        )
                    }
                }
            };
            let line = format!("  {}  {}  ({}M)", infra.display_name(), roi, cost);
            let cost_color = if deep.prestige.warband_marks >= cost {
                Color::White
            } else {
                Color::DarkGray
            };
            put_text(buffer, row, detail_inner_left, &line, cost_color);
            // Highlight cost portion with affordability color
            let cost_label = format!("({}M)", cost);
            if let Some(pos) = line.rfind(&cost_label) {
                let afford_color = if deep.prestige.warband_marks >= cost {
                    Color::Green
                } else {
                    Color::LightRed
                };
                put_text(
                    buffer,
                    row,
                    detail_inner_left + pos as i32,
                    &cost_label,
                    afford_color,
                );
            }
            row += 1;
        }
    }

    // First-visit infrastructure hint
    if ui.layer_visit_count < 3 && row < content_bottom {
        put_text(
            buffer,
            row,
            detail_inner_left,
            "  Build via Construction missions (safe, 4-8h). Permanent.",
            Color::Rgb(50, 80, 110),
        );
        row += 1;
    }

    // Duration section for cleared layers
    if layer.cleared && row < content_bottom - 3 {
        row += 1;
        put_text(
            buffer,
            row,
            detail_inner_left,
            "Mission Durations:",
            Color::Cyan,
        );
        row += 1;

        let has_outpost = layer.has_infrastructure(Infrastructure::Outpost);
        let fam_factor = fam_level.duration_factor();
        let outpost_factor = if has_outpost { 0.75 } else { 1.0 };

        let mission_types = [
            ("Supply Run:", MissionType::SupplyRun),
            ("Recon:", MissionType::Recon),
            ("Expedition:", MissionType::Expedition),
        ];
        for (label, mt) in mission_types {
            if row >= content_bottom {
                break;
            }
            let base_secs = base_mission_duration_secs(tier, mt);
            let effective_secs =
                ((base_secs as f64) * outpost_factor * fam_factor).max(1800.0) as u64;
            if effective_secs != base_secs {
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    &format!(
                        "  {:12} {} \u{2192} {}",
                        label,
                        format_hours(base_secs),
                        format_hours(effective_secs)
                    ),
                    Color::White,
                );
            } else {
                put_text(
                    buffer,
                    row,
                    detail_inner_left,
                    &format!("  {:12} {}", label, format_hours(base_secs)),
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
        Infrastructure::SupplyCache => "+50% Marks from supply runs",
        Infrastructure::Watchtower => "+25 intel instantly",
        Infrastructure::Bridge => "Skip this layer on deep push",
    }
}
