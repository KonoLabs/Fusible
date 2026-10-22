//! Pure-Rust vector SVG rendering for PCB layouts and schematic diagrams.

pub mod pcb;
pub mod schematic;
pub mod writer;

use std::path::Path;

use crate::gerber::layers::PcbModel;
use crate::netlist::Netlist;
use crate::parser::Circuit;

pub use pcb::render_pcb;
pub use schematic::render_schematic;

/// Renders PCB layout SVG directly from Circuit JSON.
pub fn render_pcb_svg(circuit_json: &str) -> String {
    let model = PcbModel::parse(circuit_json);
    render_pcb(&model)
}

/// Renders schematic diagram SVG from Circuit AST and Netlist.
pub fn render_schematic_svg(circuit: &Circuit, netlist: &Netlist) -> String {
    render_schematic(circuit, netlist)
}

/// Generates documentation artifacts (`pcb.svg`, `schematic.svg`) directly
/// into the destination directory in pure Rust with zero external toolchain.
pub fn export_docs_bundle(
    circuit_json: &str,
    circuit: &Circuit,
    netlist: &Netlist,
    out_dir: &Path,
) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(out_dir)?;

    let pcb_svg = render_pcb_svg(circuit_json);
    let pcb_path = out_dir.join("pcb.svg");
    std::fs::write(&pcb_path, pcb_svg)?;

    let sch_svg = render_schematic_svg(circuit, netlist);
    let sch_path = out_dir.join("schematic.svg");
    std::fs::write(&sch_path, sch_svg)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_valid_pcb_svg() {
        let json = r#"[
            { "type": "pcb_board", "width": 50.0, "height": 30.0, "center": { "x": 0.0, "y": 0.0 } },
            { "type": "pcb_smtpad", "x": 10.0, "y": 5.0, "width": 1.5, "height": 2.0, "layer": "top" },
            { "type": "pcb_via", "x": -5.0, "y": -5.0, "outer_diameter": 0.8, "hole_diameter": 0.4 }
        ]"#;

        let svg = render_pcb_svg(json);
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>\n"));
        assert!(svg.contains("substrate"));
        assert!(svg.contains("pads"));
        assert!(svg.contains("holes_and_vias"));
    }

    #[test]
    fn pcb_svg_draws_board_from_outline_points() {
        let svg = render_pcb_svg(
            r#"[{ "type": "pcb_board", "width": 50.0, "height": 30.0, "center": { "x": 0.0, "y": 0.0 } }]"#,
        );
        assert!(svg.contains("M -25.000 15.000 L 25.000 15.000 L 25.000 -15.000"));
    }

    #[test]
    fn empty_schematic_is_still_a_valid_svg() {
        let circuit = crate::parser::parse("[]", Path::new("empty.json")).unwrap();
        let netlist = Netlist::from_circuit(&circuit);
        let svg = render_schematic_svg(&circuit, &netlist);
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>\n"));
    }

    #[test]
    fn renders_valid_schematic_svg() {
        let json = r#"[
            { "type": "source_component", "component_id": "U1", "name": "MCU", "kind": "chip" },
            { "type": "source_component", "component_id": "R1", "name": "R1", "kind": "resistor" },
            { "type": "source_net", "net_id": "n1", "name": "SIG" },
            { "type": "source_port", "port_id": "U1.1", "source_component_id": "U1", "name": "IO1", "net_id": "n1" },
            { "type": "source_port", "port_id": "R1.1", "source_component_id": "R1", "name": "1", "net_id": "n1" }
        ]"#;

        let circuit = crate::parser::parse(json, Path::new("test.json")).unwrap();
        let netlist = Netlist::from_circuit(&circuit);
        let svg = render_schematic_svg(&circuit, &netlist);

        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>\n"));
        assert!(svg.contains("components"));
        assert!(svg.contains("nets"));
        assert!(svg.contains("MCU"));
    }
}
