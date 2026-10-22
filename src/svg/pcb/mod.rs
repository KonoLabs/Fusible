//! Native vector renderer for multi-layer PCB layout SVGs.

mod bounds;
mod copper;
mod legend;

use super::writer::SvgWriter;
use crate::gerber::layers::PcbModel;

/// Modern dark-theme PCB color palette.
pub struct PcbPalette {
    /// Solder mask fill: dark matte green by default.
    pub substrate: &'static str,
    /// Board contour stroke.
    pub board_edge: &'static str,
    /// Top copper traces and vias: shiny copper by default.
    pub top_copper: &'static str,
    /// Bottom copper traces: translucent blue by default.
    pub bottom_copper: &'static str,
    /// SMT pads and plated rings: ENIG gold surface finish by default.
    pub pad_gold: &'static str,
    /// Through-hole cutouts.
    pub drill_hole: &'static str,
    /// Silkscreen legend: crisp white by default.
    pub silkscreen: &'static str,
}

pub const DEFAULT_PALETTE: PcbPalette = PcbPalette {
    substrate: "#182a1d",
    board_edge: "#2d5a37",
    top_copper: "#cf8a3c",
    bottom_copper: "#3b5284",
    pad_gold: "#e6c35c",
    drill_hole: "#0a0a0f",
    silkscreen: "#f7fafc",
};

/// Canvas padding around the board, in mm.
const CANVAS_MARGIN_MM: f64 = 5.0;

/// Renders a complete vector SVG of the PCB layout from Circuit JSON.
pub fn render_pcb(model: &PcbModel) -> String {
    let mut writer = SvgWriter::new();
    let bounds = bounds::calculate_bounds(model);
    let margin = CANVAS_MARGIN_MM;
    let view_w = (bounds.max_x - bounds.min_x) + (margin * 2.0);
    let view_h = (bounds.max_y - bounds.min_y) + (margin * 2.0);
    writer.start_svg(
        [view_w, view_h],
        [bounds.min_x - margin, bounds.min_y - margin, view_w, view_h],
    );

    group(&mut writer, "substrate", None, |w| {
        legend::render_substrate(w, &model.boards, &bounds)
    });
    group(&mut writer, "bottom_copper", Some(0.6), |w| {
        copper::render_copper_traces(w, model, "bottom", DEFAULT_PALETTE.bottom_copper)
    });
    group(&mut writer, "top_copper", None, |w| {
        copper::render_copper_traces(w, model, "top", DEFAULT_PALETTE.top_copper)
    });
    group(&mut writer, "pads", None, |w| copper::render_pads(w, model));
    group(&mut writer, "holes_and_vias", None, |w| {
        copper::render_holes_and_vias(w, model)
    });
    group(&mut writer, "silkscreen", None, |w| {
        legend::render_silkscreen(w, model)
    });

    writer.end_svg();
    writer.finish()
}

/// Wraps whatever `draw` emits in a `<g>` with the given id and opacity.
fn group(
    writer: &mut SvgWriter,
    id: &str,
    opacity: Option<f64>,
    draw: impl FnOnce(&mut SvgWriter),
) {
    writer.start_group(id, opacity);
    draw(writer);
    writer.end_group();
}

/// Builds an SVG path through `points`, flipping Y from board space to screen space.
fn polyline<'a>(points: impl IntoIterator<Item = &'a crate::gerber::layers::PcbPoint>) -> String {
    let mut d = String::new();
    for (i, pt) in points.into_iter().enumerate() {
        let cmd = if i == 0 { "M" } else { "L" };
        d.push_str(&format!("{cmd} {:.3} {:.3} ", pt.x, -pt.y));
    }
    d
}
