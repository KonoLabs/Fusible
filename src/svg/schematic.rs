//! Native vector renderer for circuit schematic diagrams.

use std::collections::HashMap;

use super::writer::SvgWriter;
use crate::netlist::Netlist;
use crate::parser::Circuit;

pub struct SchematicPalette {
    pub background: &'static str,
    pub component_box: &'static str,
    pub component_border: &'static str,
    pub text_primary: &'static str,
    pub text_muted: &'static str,
    pub wire: &'static str,
    pub junction: &'static str,
}

pub const SCHEMATIC_PALETTE: SchematicPalette = SchematicPalette {
    background: "#0d1117",       // GitHub Dark background
    component_box: "#161b22",    // Dark card
    component_border: "#30363d", // Border
    text_primary: "#f0f6fc",     // Crisp text
    text_muted: "#8b949e",       // Secondary pin text
    wire: "#58a6ff",             // Bright signal blue
    junction: "#79c0ff",         // Junction dot
};

/// Renders a schematic diagram SVG from the Circuit AST and lowered Netlist graph.
pub fn render_schematic(circuit: &Circuit, netlist: &Netlist) -> String {
    let mut writer = SvgWriter::new();
    let components: Vec<_> = circuit.components().collect();

    if components.is_empty() {
        writer.start_svg(1.0, 1.0, 0.0, 0.0, 1.0, 1.0);
        writer.end_svg();
        return writer.finish();
    }

    let comp_w = 40.0;
    let pin_h = 6.0;
    let gap_x = 35.0;
    let cols = 3.max((components.len() as f64).sqrt().ceil() as usize);

    // Calculate layout coordinates for each component
    let mut comp_positions = HashMap::new();
    let mut port_positions = HashMap::new();

    let mut cur_x = 20.0;
    let mut cur_y = 20.0;
    let mut max_row_h = 0.0;
    let mut total_w = 0.0;

    for (idx, comp) in components.iter().enumerate() {
        let cid = comp.component_id.as_deref().unwrap_or("U");
        let ports: Vec<_> = circuit
            .ports()
            .filter(|p| {
                p.source_component_id.as_deref() == Some(cid)
                    || (p.source_component_id.is_none()
                        && p.port_id.as_deref().and_then(|id| id.split('.').next()) == Some(cid))
            })
            .collect();

        let comp_h = 16.0 + (ports.len() as f64 * pin_h).max(12.0);
        if comp_h > max_row_h {
            max_row_h = comp_h;
        }

        comp_positions.insert(cid, (cur_x, cur_y, comp_w, comp_h));

        for (p_idx, port) in ports.iter().enumerate() {
            let py = cur_y + 16.0 + (p_idx as f64 * pin_h);
            let px = cur_x + comp_w;
            if let Some(pid) = port.port_id.as_deref() {
                port_positions.insert(pid, (px, py));
            }
        }

        cur_x += comp_w + gap_x;
        if cur_x > total_w {
            total_w = cur_x;
        }

        if (idx + 1) % cols == 0 {
            cur_x = 20.0;
            cur_y += max_row_h + 25.0;
            max_row_h = 0.0;
        }
    }

    let total_h = cur_y + max_row_h + 30.0;

    writer.start_svg(total_w, total_h, 0.0, 0.0, total_w, total_h);

    // Background
    writer.rect(0.0, 0.0, total_w, total_h, SCHEMATIC_PALETTE.background);

    // 1. Draw Components
    writer.start_group("components", None);
    for comp in &components {
        let cid = comp.component_id.as_deref().unwrap_or("U");
        let name = comp.name.as_deref().unwrap_or(cid);
        if let Some(&(x, y, w, h)) = comp_positions.get(cid) {
            writer.rounded_rect(
                [x, y, w, h],
                2.0,
                SCHEMATIC_PALETTE.component_box,
                Some((SCHEMATIC_PALETTE.component_border, 0.5)),
            );
            writer.text(
                x + 4.0,
                y + 8.0,
                name,
                4.0,
                SCHEMATIC_PALETTE.text_primary,
                "start",
            );
            writer.text(
                x + 4.0,
                y + 13.0,
                comp.kind.as_deref().unwrap_or("chip"),
                2.5,
                SCHEMATIC_PALETTE.text_muted,
                "start",
            );

            // Render pin names
            let ports: Vec<_> = circuit
                .ports()
                .filter(|p| {
                    p.source_component_id.as_deref() == Some(cid)
                        || (p.source_component_id.is_none()
                            && p.port_id.as_deref().and_then(|id| id.split('.').next())
                                == Some(cid))
                })
                .collect();
            for (p_idx, port) in ports.iter().enumerate() {
                let py = y + 17.0 + (p_idx as f64 * pin_h);
                let pname = port.name.as_deref().unwrap_or("pin");
                writer.text(
                    x + w - 4.0,
                    py - 1.0,
                    pname,
                    2.5,
                    SCHEMATIC_PALETTE.text_muted,
                    "end",
                );
                writer.circle(x + w, py - 2.0, 0.8, SCHEMATIC_PALETTE.junction, None);
            }
        }
    }
    writer.end_group();

    // 2. Draw Net Wires
    writer.start_group("nets", None);
    for net in &netlist.nets {
        let endpoints: Vec<_> = net
            .endpoints
            .iter()
            .filter_map(|ep| ep.id.as_deref().and_then(|id| port_positions.get(id)))
            .copied()
            .collect();

        if endpoints.len() >= 2 {
            for i in 0..endpoints.len() - 1 {
                let (x1, y1) = endpoints[i];
                let (x2, y2) = endpoints[i + 1];
                let mid_x = (x1 + x2) / 2.0;

                let d = format!(
                    "M {x1:.2} {y1:.2} L {mid_x:.2} {y1:.2} L {mid_x:.2} {y2:.2} L {x2:.2} {y2:.2}"
                );
                writer.path(&d, "none", Some((SCHEMATIC_PALETTE.wire, 0.4)));
                writer.circle(x1, y1, 0.6, SCHEMATIC_PALETTE.junction, None);
                writer.circle(x2, y2, 0.6, SCHEMATIC_PALETTE.junction, None);
            }
        }
    }
    writer.end_group();

    writer.end_svg();
    writer.finish()
}
