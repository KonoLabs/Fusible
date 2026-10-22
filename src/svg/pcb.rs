//! Native vector renderer for multi-layer PCB layout SVGs.

use super::writer::SvgWriter;
use crate::gerber::layers::{PcbBoardElm, PcbModel};

/// Modern dark-theme PCB color palette.
pub struct PcbPalette {
    pub substrate: &'static str,
    pub board_edge: &'static str,
    pub top_copper: &'static str,
    pub bottom_copper: &'static str,
    pub pad_gold: &'static str,
    pub drill_hole: &'static str,
    pub silkscreen: &'static str,
}

pub const DEFAULT_PALETTE: PcbPalette = PcbPalette {
    substrate: "#182a1d",     // Dark matte solder mask green
    board_edge: "#2d5a37",    // Board contour
    top_copper: "#cf8a3c",    // Shiny copper
    bottom_copper: "#3b5284", // Translucent bottom copper (blueish)
    pad_gold: "#e6c35c",      // ENIG Gold surface finish
    drill_hole: "#0a0a0f",    // Dark through-hole cutout
    silkscreen: "#f7fafc",    // Crisp white legend
};

/// Renders a complete vector SVG of the PCB layout from Circuit JSON.
pub fn render_pcb(model: &PcbModel) -> String {
    let mut writer = SvgWriter::new();
    let (min_x, min_y, max_x, max_y) = calculate_bounds(model);

    let margin = 5.0; // 5mm canvas padding
    let view_x = min_x - margin;
    let view_y = min_y - margin;
    let view_w = (max_x - min_x) + (margin * 2.0);
    let view_h = (max_y - min_y) + (margin * 2.0);

    writer.start_svg(view_w, view_h, view_x, view_y, view_w, view_h);

    // 1. Board Substrate Outline
    writer.start_group("substrate", None);
    render_substrate(&mut writer, &model.boards, min_x, min_y, max_x, max_y);
    writer.end_group();

    // 2. Bottom Copper (dimmed)
    writer.start_group("bottom_copper", Some(0.6));
    render_copper_traces(&mut writer, model, "bottom", DEFAULT_PALETTE.bottom_copper);
    writer.end_group();

    // 3. Top Copper Traces
    writer.start_group("top_copper", None);
    render_copper_traces(&mut writer, model, "top", DEFAULT_PALETTE.top_copper);
    writer.end_group();

    // 4. SMT Pads (ENIG Gold)
    writer.start_group("pads", None);
    render_pads(&mut writer, model);
    writer.end_group();

    // 5. Plated Through-Holes & Vias
    writer.start_group("holes_and_vias", None);
    render_holes_and_vias(&mut writer, model);
    writer.end_group();

    // 6. Silkscreen Legend
    writer.start_group("silkscreen", None);
    render_silkscreen(&mut writer, model);
    writer.end_group();

    writer.end_svg();
    writer.finish()
}

fn calculate_bounds(model: &PcbModel) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    let mut update = |x: f64, y: f64| {
        if x < min_x {
            min_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if x > max_x {
            max_x = x;
        }
        if y > max_y {
            max_y = y;
        }
    };

    for pt in model.boards.iter().flat_map(PcbBoardElm::outline_points) {
        update(pt.x, -pt.y);
    }

    for p in &model.smt_pads {
        update(p.x, -p.y);
    }
    for h in &model.plated_holes {
        update(h.x, -h.y);
    }
    for v in &model.vias {
        update(v.x, -v.y);
    }

    if min_x > max_x {
        // Fallback default board bounds
        (-25.0, -25.0, 25.0, 25.0)
    } else {
        (min_x, min_y, max_x, max_y)
    }
}

fn render_substrate(
    writer: &mut SvgWriter,
    boards: &[PcbBoardElm],
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) {
    if boards.is_empty() {
        let w = max_x - min_x;
        let h = max_y - min_y;
        writer.rounded_rect(
            [min_x, min_y, w, h],
            2.0,
            DEFAULT_PALETTE.substrate,
            Some((DEFAULT_PALETTE.board_edge, 0.4)),
        );
        return;
    }

    for b in boards {
        let outline = b.outline_points();
        if outline.len() < 3 {
            continue;
        }
        let mut d = String::new();
        for (i, pt) in outline.iter().enumerate() {
            let cmd = if i == 0 { "M" } else { "L" };
            d.push_str(&format!("{cmd} {:.3} {:.3} ", pt.x, -pt.y));
        }
        d.push('Z');
        writer.path(
            &d,
            DEFAULT_PALETTE.substrate,
            Some((DEFAULT_PALETTE.board_edge, 0.4)),
        );
    }
}

fn render_copper_traces(writer: &mut SvgWriter, model: &PcbModel, target_layer: &str, color: &str) {
    for t in &model.traces {
        if t.route.len() < 2 {
            continue;
        }

        let mut current_path = String::new();
        let mut stroke_w = 0.25;

        for (i, pt) in t.route.iter().enumerate() {
            let layer = pt.layer.as_deref().unwrap_or("top");
            if !layer.eq_ignore_ascii_case(target_layer) {
                continue;
            }
            if let Some(w) = pt.width {
                stroke_w = w;
            }

            let cmd = if i == 0 || current_path.is_empty() {
                "M"
            } else {
                "L"
            };
            current_path.push_str(&format!("{cmd} {:.3} {:.3} ", pt.x, -pt.y));
        }

        if !current_path.is_empty() {
            writer.path(&current_path, "none", Some((color, stroke_w)));
        }
    }
}

fn render_pads(writer: &mut SvgWriter, model: &PcbModel) {
    for pad in &model.smt_pads {
        let y = -pad.y;
        let is_circle =
            pad.shape.as_deref() == Some("circle") || (pad.radius.is_some() && pad.width.is_none());

        if is_circle {
            let r = pad.radius.unwrap_or(pad.width.unwrap_or(1.0) / 2.0);
            writer.circle(pad.x, y, r, DEFAULT_PALETTE.pad_gold, None);
        } else {
            let w = pad.width.unwrap_or(1.0);
            let h = pad.height.unwrap_or(1.0);
            writer.rounded_rect(
                [pad.x - w / 2.0, y - h / 2.0, w, h],
                0.15,
                DEFAULT_PALETTE.pad_gold,
                None,
            );
        }
    }
}

fn render_holes_and_vias(writer: &mut SvgWriter, model: &PcbModel) {
    // Plated Holes
    for h in &model.plated_holes {
        let y = -h.y;
        let outer_r = h.outer_diameter.unwrap_or(1.6) / 2.0;
        let drill_r = h.hole_diameter.unwrap_or(0.8) / 2.0;

        writer.circle(h.x, y, outer_r, DEFAULT_PALETTE.pad_gold, None);
        writer.circle(h.x, y, drill_r, DEFAULT_PALETTE.drill_hole, None);
    }

    // Vias
    for v in &model.vias {
        let y = -v.y;
        let outer_r = v.outer_diameter.unwrap_or(0.6) / 2.0;
        let drill_r = v.hole_diameter.unwrap_or(0.3) / 2.0;

        writer.circle(v.x, y, outer_r, DEFAULT_PALETTE.top_copper, None);
        writer.circle(v.x, y, drill_r, DEFAULT_PALETTE.drill_hole, None);
    }
}

fn render_silkscreen(writer: &mut SvgWriter, model: &PcbModel) {
    for c in &model.components {
        if let (Some(ref center), Some(w), Some(h)) = (&c.center, c.width, c.height) {
            writer.rounded_rect(
                [center.x - w / 2.0, -center.y - h / 2.0, w, h],
                0.2,
                "none",
                Some(("rgba(247, 250, 252, 0.4)", 0.1)),
            );
        }
    }

    for s in &model.silkscreen_paths {
        if s.route.len() < 2 {
            continue;
        }
        let stroke_w = s.stroke_width.unwrap_or(0.15);
        let mut d = String::new();
        for (i, pt) in s.route.iter().enumerate() {
            let cmd = if i == 0 { "M" } else { "L" };
            d.push_str(&format!("{cmd} {:.3} {:.3} ", pt.x, -pt.y));
        }
        writer.path(&d, "none", Some((DEFAULT_PALETTE.silkscreen, stroke_w)));
    }

    for t in &model.silkscreen_texts {
        if let Some(ref text) = t.text {
            let pt = t.anchor_position.as_ref().or(t.center.as_ref());
            if let Some(pos) = pt {
                let size = t.font_size.unwrap_or(0.6);
                writer.text(
                    pos.x,
                    -pos.y,
                    text,
                    size,
                    DEFAULT_PALETTE.silkscreen,
                    "middle",
                );
            }
        }
    }
}
