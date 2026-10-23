//! Copper traces, SMT pads, plated holes and vias.

use super::DEFAULT_PALETTE;
use crate::gerber::layers::{PcbModel, PcbSmtPadElm, PcbTracePoint};
use crate::svg::writer::SvgWriter;

/// Draws every trace's route points on `target_layer` as one stroked path per trace.
pub(super) fn render_copper_traces(
    writer: &mut SvgWriter,
    model: &PcbModel,
    target_layer: &str,
    color: &str,
) {
    for t in model.traces.iter().filter(|t| t.route.len() >= 2) {
        if let Some((d, stroke_w)) = layer_path(&t.route, target_layer) {
            writer.path(&d, "none", Some((color, stroke_w)));
        }
    }
}

/// Returns the path through the route points on `target_layer` and the last width seen,
/// or `None` when no point is on that layer.
fn layer_path(route: &[PcbTracePoint], target_layer: &str) -> Option<(String, f64)> {
    let mut d = String::new();
    let mut stroke_w = 0.25;
    for pt in route {
        let layer = pt.layer.as_deref().unwrap_or("top");
        if !layer.eq_ignore_ascii_case(target_layer) {
            continue;
        }
        if let Some(w) = pt.width {
            stroke_w = w;
        }
        let cmd = if d.is_empty() { "M" } else { "L" };
        d.push_str(&format!("{cmd} {:.3} {:.3} ", pt.x, -pt.y));
    }
    (!d.is_empty()).then_some((d, stroke_w))
}

/// Returns the SVG path of a polygon pad, or `None` for any other pad.
fn polygon_path(pad: &PcbSmtPadElm) -> Option<String> {
    let points = pad.points.as_deref()?;
    if pad.shape.as_deref() != Some("polygon") || points.is_empty() {
        return None;
    }
    let d: Vec<String> = points
        .iter()
        .enumerate()
        .map(|(i, pt)| {
            format!(
                "{} {:.3} {:.3}",
                if i == 0 { "M" } else { "L" },
                pt.x,
                -pt.y
            )
        })
        .collect();
    Some(format!("{} Z", d.join(" ")))
}

/// Draws SMT pads as gold polygons, circles or rounded rectangles.
pub(super) fn render_pads(writer: &mut SvgWriter, model: &PcbModel) {
    for pad in &model.smt_pads {
        if let Some(d) = polygon_path(pad) {
            writer.path(&d, DEFAULT_PALETTE.pad_gold, None);
            continue;
        }
        let y = -pad.y;
        let is_circle =
            pad.shape.as_deref() == Some("circle") || (pad.radius.is_some() && pad.width.is_none());

        if is_circle {
            let r = pad.radius.unwrap_or(pad.width.unwrap_or(1.0) / 2.0);
            writer.circle([pad.x, y], r, DEFAULT_PALETTE.pad_gold, None);
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

/// Draws plated holes as gold rings and vias as copper rings, each with a drill cutout.
pub(super) fn render_holes_and_vias(writer: &mut SvgWriter, model: &PcbModel) {
    for h in &model.plated_holes {
        let at = [h.x, -h.y];
        let outer_r = h.outer_diameter.unwrap_or(1.6) / 2.0;
        let drill_r = h.hole_diameter.unwrap_or(0.8) / 2.0;

        writer.circle(at, outer_r, DEFAULT_PALETTE.pad_gold, None);
        writer.circle(at, drill_r, DEFAULT_PALETTE.drill_hole, None);
    }

    for v in &model.vias {
        let at = [v.x, -v.y];
        let outer_r = v.outer_diameter.unwrap_or(0.6) / 2.0;
        let drill_r = v.hole_diameter.unwrap_or(0.3) / 2.0;

        writer.circle(at, outer_r, DEFAULT_PALETTE.top_copper, None);
        writer.circle(at, drill_r, DEFAULT_PALETTE.drill_hole, None);
    }
}
