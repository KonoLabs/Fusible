//! Pure-Rust vector SVG rendering for PCB layouts and schematic diagrams.

pub mod pcb;
pub mod schematic;
pub mod shapes;
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
mod tests;
