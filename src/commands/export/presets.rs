//! Export presets: `fab`, `cad`, `docs` and `all`.

use std::path::Path;

use super::{extract_zip_archive, ExportError, ExportJob};
use crate::commands::check::CheckError;

/// Rejects any preset name other than `fab`, `cad`, `docs` or `all`.
pub(super) fn validate_preset(preset: &str) -> Result<(), ExportError> {
    match preset {
        "fab" | "cad" | "docs" | "all" => Ok(()),
        other => Err(ExportError::UnknownPreset {
            preset: other.to_string(),
        }),
    }
}

/// Runs the named preset for `job`.
pub(super) fn run_preset(job: &ExportJob, preset: &str) -> Result<(), ExportError> {
    validate_preset(preset)?;
    let circuit_json = match preset {
        "cad" => String::new(),
        _ => job.circuit_json()?,
    };
    match preset {
        "fab" => export_fab(job, &circuit_json),
        "cad" => export_cad(job),
        "docs" => export_docs(job, &circuit_json),
        _ => export_all(job, &circuit_json),
    }
}

/// Runs the fab, cad and docs presets in sequence.
fn export_all(job: &ExportJob, circuit_json: &str) -> Result<(), ExportError> {
    export_fab(job, circuit_json)?;
    println!();
    export_cad(job)?;
    println!();
    export_docs(job, circuit_json)
}

/// Writes gerbers and Circuit JSON into the fab output directory.
fn export_fab(job: &ExportJob, circuit_json: &str) -> Result<(), ExportError> {
    let base = job.output_or("dist/fab");
    std::fs::create_dir_all(&base)?;
    println!("📦 Running fabrication export preset (fab)...");
    println!("  • Exporting RS-274X gerbers & drill files via native Rust engine...");
    crate::gerber::export_fab_bundle(circuit_json, &job.board_title, &base)?;
    println!(
        "✅ Fabrication artifacts generated in \"{}\" (gerbers.zip, circuit.json)",
        base.display()
    );
    Ok(())
}

/// Exports the KiCad project and STEP model through the tscircuit runner.
fn export_cad(job: &ExportJob) -> Result<(), ExportError> {
    let runner = job.runner()?;
    let base = job.output_or("dist/cad");
    std::fs::create_dir_all(&base)?;
    println!("📦 Running CAD export preset (cad)...");
    let kicad = base.join("kicad.zip");
    println!("  • Exporting KiCad project -> {}", kicad.display());
    runner.export(&job.entry, "kicad_zip", Some(&kicad))?;

    let kicad_dir = base.join("kicad");
    std::fs::create_dir_all(&kicad_dir)?;
    extract_zip_archive(&kicad, &kicad_dir)?;
    println!(
        "  • KiCad project files extracted -> {}",
        kicad_dir.display()
    );

    let step = base.join("board.step");
    println!("  • Exporting 3D STEP -> {}", step.display());
    runner.export(&job.entry, "step", Some(&step))?;
    println!("✅ CAD artifacts generated in \"{}\"", base.display());
    Ok(())
}

/// Renders schematic and PCB SVGs into the docs output directory.
fn export_docs(job: &ExportJob, circuit_json: &str) -> Result<(), ExportError> {
    let base = job.output_or("dist/docs");
    std::fs::create_dir_all(&base)?;
    println!("📦 Running documentation export preset (docs)...");
    println!("  • Exporting schematic SVG & PCB layout SVG via native Rust engine...");

    let circuit =
        crate::parser::parse(circuit_json, Path::new("circuit.json")).map_err(CheckError::from)?;
    let netlist = crate::netlist::Netlist::from_circuit(&circuit);

    crate::svg::export_docs_bundle(circuit_json, &circuit, &netlist, &base)?;
    let pdf =
        super::pdf::print_svg_to_pdf(&base.join("schematic.svg"), &base.join("schematic.pdf"));
    let files = if pdf {
        "schematic.svg, pcb.svg, schematic.pdf"
    } else {
        "schematic.svg, pcb.svg"
    };
    println!(
        "✅ Documentation artifacts generated in \"{}\" ({files})",
        base.display()
    );
    Ok(())
}
