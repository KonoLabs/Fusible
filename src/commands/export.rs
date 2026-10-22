//! Subcommand: `fusible export` — export circuit artifacts.

use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::check::{self, CheckError};
use crate::commands::target::resolve_entry;
use crate::tscircuit::{Runner, RunnerError};

/// Unified error enum for the export pipeline.
#[derive(Debug, Error, Diagnostic)]
pub enum ExportError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Check(#[from] CheckError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Runner(#[from] RunnerError),

    #[error("failed to export manufacturing files: {0}")]
    #[diagnostic(code(fusible::export::io_error))]
    Io(#[from] std::io::Error),

    #[error("unknown export preset \"{preset}\"")]
    #[diagnostic(
        code(fusible::export::unknown_preset),
        help("valid presets are: `fab` (gerbers + json), `cad` (kicad + step), `docs` (svg + pdf), or `all`")
    )]
    UnknownPreset { preset: String },
}

/// Exports tscircuit source files into various formats (Gerbers, SVG, 3D, KiCad, etc.).
///
/// By default, all exports are **fuse-gated**: electrical rules are checked first.
/// If any violation is found, the process halts and no export occurs.
pub fn run(
    file: Option<PathBuf>,
    preset: Option<&str>,
    format: &str,
    output: Option<PathBuf>,
    skip_lint: bool,
) -> Result<(), ExportError> {
    if let Some(preset_name) = preset {
        validate_preset(preset_name)?;
    }

    let entry = resolve_entry(file.clone()).unwrap_or_else(|_| PathBuf::from("index.tsx"));

    if !skip_lint {
        println!("🔍 Verifying electrical rules before export (fuse check)...");
        check::run(file, None, None)?;
        println!("⚡ Fuse intact — proceeding with artifact export.\n");
    } else {
        println!("⚠️  Skipping electrical rule verification (--skip-lint requested).\n");
    }

    let runner = Runner::detect().ok();
    let board_title = check::resolve_board_name();

    if let Some(preset_name) = preset {
        let circuit_json = if preset_name == "fab" || preset_name == "all" || preset_name == "docs"
        {
            load_circuit_json(&entry)?
        } else {
            String::new()
        };
        run_preset(
            runner.as_ref(),
            &entry,
            &circuit_json,
            &board_title,
            preset_name,
            output.as_deref(),
        )
    } else if format == "gerbers" || format == "gerber" {
        let circuit_json = load_circuit_json(&entry)?;
        let dest = output.unwrap_or_else(|| PathBuf::from("dist/gerbers.zip"));
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        println!(
            "📦 Generating native RS-274X gerbers to {}...",
            dest.display()
        );
        let files = crate::gerber::generate_gerber_files(&circuit_json, &board_title)
            .map_err(std::io::Error::from)?;
        let zip_bytes = crate::gerber::create_gerber_zip(&files)?;
        std::fs::write(&dest, zip_bytes)?;
        println!("✅ Successfully exported gerbers to {}!", dest.display());
        Ok(())
    } else if format == "pcb-svg" || format == "pcb_svg" {
        let circuit_json = load_circuit_json(&entry)?;
        let dest = output.unwrap_or_else(|| PathBuf::from("dist/pcb.svg"));
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        println!(
            "📦 Generating native PCB layout SVG to {}...",
            dest.display()
        );
        let svg = crate::svg::render_pcb_svg(&circuit_json);
        std::fs::write(&dest, svg)?;
        println!(
            "✅ Successfully exported PCB layout SVG to {}!",
            dest.display()
        );
        Ok(())
    } else if format == "kicad_project" || format == "kicad" {
        let runner = runner.ok_or(RunnerError::NotFound)?;
        let dest = output.unwrap_or_else(|| PathBuf::from("dist/cad/kicad"));
        std::fs::create_dir_all(&dest)?;
        let tmp_zip = dest.join(".kicad_export.zip");
        println!("📦 Exporting KiCad project to {}...", dest.display());
        runner.export(&entry, "kicad_zip", Some(&tmp_zip))?;
        extract_zip_archive(&tmp_zip, &dest)?;
        std::fs::remove_file(&tmp_zip)?;
        println!(
            "✅ Successfully exported KiCad project to {}!",
            dest.display()
        );
        Ok(())
    } else {
        let runner = runner.ok_or(RunnerError::NotFound)?;
        println!(
            "📦 Exporting \"{}\" to format: {}...",
            entry.display(),
            format
        );
        runner.export(&entry, format, output.as_deref())?;
        println!("✅ Successfully exported {format}!");
        Ok(())
    }
}

fn load_circuit_json(entry: &Path) -> Result<String, ExportError> {
    Ok(check::load_circuit_source(entry)?.1)
}

fn extract_zip_archive(zip_path: &Path, target_dir: &Path) -> Result<(), ExportError> {
    let file = std::fs::File::open(zip_path)?;
    zip::ZipArchive::new(file)
        .and_then(|mut archive| archive.extract(target_dir))
        .map_err(|err| ExportError::Io(std::io::Error::other(err)))
}

fn export_fab(
    circuit_json: &str,
    board_title: &str,
    out: Option<&Path>,
) -> Result<(), ExportError> {
    let base = out
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dist/fab"));
    std::fs::create_dir_all(&base)?;
    println!("📦 Running fabrication export preset (fab)...");
    println!("  • Exporting RS-274X gerbers & drill files via native Rust engine...");
    crate::gerber::export_fab_bundle(circuit_json, board_title, &base)?;
    println!(
        "✅ Fabrication artifacts generated in \"{}\" (gerbers.zip, circuit.json)",
        base.display()
    );
    Ok(())
}

fn export_cad(runner: &Runner, entry: &Path, out: Option<&Path>) -> Result<(), ExportError> {
    let base = out
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dist/cad"));
    std::fs::create_dir_all(&base)?;
    println!("📦 Running CAD export preset (cad)...");
    let kicad = base.join("kicad.zip");
    println!("  • Exporting KiCad project -> {}", kicad.display());
    runner.export(entry, "kicad_zip", Some(&kicad))?;

    let kicad_dir = base.join("kicad");
    std::fs::create_dir_all(&kicad_dir)?;
    extract_zip_archive(&kicad, &kicad_dir)?;
    println!(
        "  • KiCad project files extracted -> {}",
        kicad_dir.display()
    );

    let step = base.join("board.step");
    println!("  • Exporting 3D STEP -> {}", step.display());
    runner.export(entry, "step", Some(&step))?;
    println!("✅ CAD artifacts generated in \"{}\"", base.display());
    Ok(())
}

fn export_docs(circuit_json: &str, out: Option<&Path>) -> Result<(), ExportError> {
    let base = out
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dist/docs"));
    std::fs::create_dir_all(&base)?;
    println!("📦 Running documentation export preset (docs)...");
    println!("  • Exporting schematic SVG & PCB layout SVG via native Rust engine...");

    let circuit =
        crate::parser::parse(circuit_json, Path::new("circuit.json")).map_err(CheckError::from)?;
    let netlist = crate::netlist::Netlist::from_circuit(&circuit);

    crate::svg::export_docs_bundle(circuit_json, &circuit, &netlist, &base)?;
    println!(
        "✅ Documentation artifacts generated in \"{}\" (schematic.svg, pcb.svg)",
        base.display()
    );
    Ok(())
}

fn validate_preset(preset: &str) -> Result<(), ExportError> {
    match preset {
        "fab" | "cad" | "docs" | "all" => Ok(()),
        other => Err(ExportError::UnknownPreset {
            preset: other.to_string(),
        }),
    }
}

fn run_preset(
    runner: Option<&Runner>,
    entry: &Path,
    circuit_json: &str,
    board_title: &str,
    preset: &str,
    custom_output: Option<&Path>,
) -> Result<(), ExportError> {
    validate_preset(preset)?;
    match preset {
        "fab" => export_fab(circuit_json, board_title, custom_output),
        "cad" => {
            let runner = runner.ok_or(RunnerError::NotFound)?;
            export_cad(runner, entry, custom_output)
        }
        "docs" => export_docs(circuit_json, custom_output),
        "all" => {
            export_fab(circuit_json, board_title, custom_output)?;
            println!();
            let runner = runner.ok_or(RunnerError::NotFound)?;
            export_cad(runner, entry, custom_output)?;
            println!();
            export_docs(circuit_json, custom_output)?;
            Ok(())
        }
        _ => unreachable!(),
    }
}
