//! Board substrate and silkscreen legend: component outlines, paths and text.

use super::bounds::Bounds;
use super::{polyline, DEFAULT_PALETTE};
use crate::gerber::layers::{PcbBoardElm, PcbModel};
use crate::svg::shapes::TextStyle;
use crate::svg::writer::SvgWriter;

/// Draws each board outline, or a rounded rectangle over `bounds` when there is no board.
pub(super) fn render_substrate(writer: &mut SvgWriter, boards: &[PcbBoardElm], bounds: &Bounds) {
    let edge = Some((DEFAULT_PALETTE.board_edge, 0.4));
    if boards.is_empty() {
        let w = bounds.max_x - bounds.min_x;
        let h = bounds.max_y - bounds.min_y;
        writer.rounded_rect(
            [bounds.min_x, bounds.min_y, w, h],
            2.0,
            DEFAULT_PALETTE.substrate,
            edge,
        );
        return;
    }

    for b in boards {
        let outline = b.outline_points();
        if outline.len() < 3 {
            continue;
        }
        let mut d = polyline(&outline);
        d.push('Z');
        writer.path(&d, DEFAULT_PALETTE.substrate, edge);
    }
}

/// Draws component courtyards, silkscreen paths and silkscreen text.
pub(super) fn render_silkscreen(writer: &mut SvgWriter, model: &PcbModel) {
    render_component_outlines(writer, model);
    render_silkscreen_paths(writer, model);
    render_silkscreen_texts(writer, model);
}

fn render_component_outlines(writer: &mut SvgWriter, model: &PcbModel) {
    for c in &model.components {
        if let (Some(center), Some(w), Some(h)) = (&c.center, c.width, c.height) {
            writer.rounded_rect(
                [center.x - w / 2.0, -center.y - h / 2.0, w, h],
                0.2,
                "none",
                Some(("rgba(247, 250, 252, 0.4)", 0.1)),
            );
        }
    }
}

fn render_silkscreen_paths(writer: &mut SvgWriter, model: &PcbModel) {
    for s in model.silkscreen_paths.iter().filter(|s| s.route.len() >= 2) {
        let stroke_w = s.stroke_width.unwrap_or(0.15);
        let d = polyline(&s.route);
        writer.path(&d, "none", Some((DEFAULT_PALETTE.silkscreen, stroke_w)));
    }
}

fn render_silkscreen_texts(writer: &mut SvgWriter, model: &PcbModel) {
    for t in &model.silkscreen_texts {
        let pos = t.anchor_position.as_ref().or(t.center.as_ref());
        let (Some(text), Some(pos)) = (&t.text, pos) else {
            continue;
        };
        let style = TextStyle {
            font_size: t.font_size.unwrap_or(0.6),
            fill: DEFAULT_PALETTE.silkscreen,
            anchor: "middle",
        };
        writer.text([pos.x, -pos.y], text, style);
    }
}
