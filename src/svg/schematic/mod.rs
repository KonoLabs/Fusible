//! Native vector renderer for circuit schematic diagrams.

mod draw;
mod layout;

use super::writer::SvgWriter;
use crate::netlist::Netlist;
use crate::parser::{Circuit, Component, Port};

pub struct SchematicPalette {
    /// Canvas fill: GitHub dark background by default.
    pub background: &'static str,
    /// Component card fill.
    pub component_box: &'static str,
    /// Component card border.
    pub component_border: &'static str,
    /// Component names.
    pub text_primary: &'static str,
    /// Component kinds and pin names.
    pub text_muted: &'static str,
    /// Net wires: bright signal blue by default.
    pub wire: &'static str,
    /// Pin and wire junction dots.
    pub junction: &'static str,
}

pub const SCHEMATIC_PALETTE: SchematicPalette = SchematicPalette {
    background: "#0d1117",
    component_box: "#161b22",
    component_border: "#30363d",
    text_primary: "#f0f6fc",
    text_muted: "#8b949e",
    wire: "#58a6ff",
    junction: "#79c0ff",
};

/// Renders a schematic diagram SVG from the Circuit AST and lowered Netlist graph.
pub fn render_schematic(circuit: &Circuit, netlist: &Netlist) -> String {
    let mut writer = SvgWriter::new();
    let components: Vec<_> = circuit.components().collect();

    if components.is_empty() {
        writer.start_svg([1.0, 1.0], [0.0, 0.0, 1.0, 1.0]);
        writer.end_svg();
        return writer.finish();
    }

    let layout = layout::layout(circuit, &components);
    let (total_w, total_h) = (layout.width, layout.height);
    writer.start_svg([total_w, total_h], [0.0, 0.0, total_w, total_h]);
    writer.rect([0.0, 0.0, total_w, total_h], SCHEMATIC_PALETTE.background);

    writer.start_group("components", None);
    for comp in &components {
        if let Some(&rect) = layout.components.get(component_id(comp)) {
            draw::draw_component(&mut writer, circuit, comp, rect);
        }
    }
    writer.end_group();

    writer.start_group("nets", None);
    draw::draw_nets(&mut writer, netlist, &layout.ports);
    writer.end_group();

    writer.end_svg();
    writer.finish()
}

/// Returns the component's identifier, defaulting to `U`.
fn component_id(comp: &Component) -> &str {
    comp.component_id.as_deref().unwrap_or("U")
}

/// Returns the ports owned by `cid`, matching `source_component_id` or, failing that,
/// the `cid.` prefix of the port id.
fn component_ports<'a>(circuit: &'a Circuit, cid: &str) -> Vec<&'a Port> {
    circuit
        .ports()
        .filter(|p| {
            p.source_component_id.as_deref() == Some(cid)
                || (p.source_component_id.is_none()
                    && p.port_id.as_deref().and_then(|id| id.split('.').next()) == Some(cid))
        })
        .collect()
}
