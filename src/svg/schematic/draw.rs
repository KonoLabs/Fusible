//! Drawing of component cards, pin labels and net wires.

use std::collections::HashMap;

use super::layout::PIN_H;
use super::{component_id, component_ports, SCHEMATIC_PALETTE};
use crate::netlist::Netlist;
use crate::parser::{Circuit, Component};
use crate::svg::shapes::TextStyle;
use crate::svg::writer::SvgWriter;

/// Draws a component card at `[x, y, w, h]` with its name, kind and pin labels.
pub(super) fn draw_component(
    writer: &mut SvgWriter,
    circuit: &Circuit,
    comp: &Component,
    rect: [f64; 4],
) {
    let cid = component_id(comp);
    let [x, y, _, _] = rect;
    writer.rounded_rect(
        rect,
        2.0,
        SCHEMATIC_PALETTE.component_box,
        Some((SCHEMATIC_PALETTE.component_border, 0.5)),
    );
    let name = comp.name.as_deref().unwrap_or(cid);
    writer.text(
        [x + 4.0, y + 8.0],
        name,
        start(4.0, SCHEMATIC_PALETTE.text_primary),
    );
    let kind = comp.kind.as_deref().unwrap_or("chip");
    writer.text(
        [x + 4.0, y + 13.0],
        kind,
        start(2.5, SCHEMATIC_PALETTE.text_muted),
    );

    draw_pins(writer, circuit, cid, rect);
}

/// Draws the pin labels and junction dots along the right edge of a card.
fn draw_pins(writer: &mut SvgWriter, circuit: &Circuit, cid: &str, rect: [f64; 4]) {
    let [x, y, w, _] = rect;
    for (p_idx, port) in component_ports(circuit, cid).iter().enumerate() {
        let py = y + 17.0 + (p_idx as f64 * PIN_H);
        let pname = port.name.as_deref().unwrap_or("pin");
        let style = TextStyle {
            font_size: 2.5,
            fill: SCHEMATIC_PALETTE.text_muted,
            anchor: "end",
        };
        writer.text([x + w - 4.0, py - 1.0], pname, style);
        writer.circle([x + w, py - 2.0], 0.8, SCHEMATIC_PALETTE.junction, None);
    }
}

/// Returns a start-anchored text style.
fn start(font_size: f64, fill: &str) -> TextStyle<'_> {
    TextStyle {
        font_size,
        fill,
        anchor: "start",
    }
}

/// Chains every net's placed endpoints with orthogonal wires.
pub(super) fn draw_nets(
    writer: &mut SvgWriter,
    netlist: &Netlist,
    ports: &HashMap<&str, (f64, f64)>,
) {
    for net in &netlist.nets {
        let endpoints: Vec<_> = net
            .endpoints
            .iter()
            .filter_map(|ep| ep.id.as_deref().and_then(|id| ports.get(id)))
            .copied()
            .collect();
        for pair in endpoints.windows(2) {
            draw_wire(writer, pair[0], pair[1]);
        }
    }
}

/// Draws an orthogonal wire with a vertical jog at the midpoint, plus end junctions.
fn draw_wire(writer: &mut SvgWriter, (x1, y1): (f64, f64), (x2, y2): (f64, f64)) {
    let mid_x = (x1 + x2) / 2.0;
    let d =
        format!("M {x1:.2} {y1:.2} L {mid_x:.2} {y1:.2} L {mid_x:.2} {y2:.2} L {x2:.2} {y2:.2}");
    writer.path(&d, "none", Some((SCHEMATIC_PALETTE.wire, 0.4)));
    writer.circle([x1, y1], 0.6, SCHEMATIC_PALETTE.junction, None);
    writer.circle([x2, y2], 0.6, SCHEMATIC_PALETTE.junction, None);
}
