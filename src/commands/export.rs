//! Subcommand: `fusible export` — export circuit artifacts.

use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::check::{self, CheckError};
use crate::commands::target::resolve_entry;
use crate::config::FusibleConfig;
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

    let runner = Runner::detect()?;
    let board_title = resolve_board_title();

    if let Some(preset_name) = preset {
        let circuit_json = if preset_name == "fab" || preset_name == "all" {
            load_or_build_circuit_json(&entry, &runner)?
        } else {
            String::new()
        };
        run_preset(
            &runner,
            &entry,
            &circuit_json,
            &board_title,
            preset_name,
            output.as_deref(),
        )
    } else if format == "gerbers" || format == "gerber" {
        let circuit_json = load_or_build_circuit_json(&entry, &runner)?;
        let dest = output.unwrap_or_else(|| PathBuf::from("dist/gerbers.zip"));
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        println!(
            "📦 Generating native RS-274X gerbers to {}...",
            dest.display()
        );
        let files = crate::gerber::generate_gerber_files(&circuit_json, &board_title);
        let zip_bytes = crate::gerber::create_gerber_zip(&files)?;
        std::fs::write(&dest, zip_bytes)?;
        println!("✅ Successfully exported gerbers to {}!", dest.display());
        Ok(())
    } else if format == "pcb-svg" || format == "pcb_svg" {
        let circuit_json = load_or_build_circuit_json(&entry, &runner)?;
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
        let dest = output.unwrap_or_else(|| PathBuf::from("dist/cad/kicad"));
        std::fs::create_dir_all(&dest)?;
        let tmp_zip = dest.join(".kicad_export.zip");
        println!("📦 Exporting KiCad project to {}...", dest.display());
        runner.export(&entry, "kicad_zip", Some(&tmp_zip))?;
        extract_zip_archive(&tmp_zip, &dest);
        let _ = std::fs::remove_file(&tmp_zip);
        println!(
            "✅ Successfully exported KiCad project to {}!",
            dest.display()
        );
        Ok(())
    } else {
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

fn resolve_board_title() -> String {
    FusibleConfig::find_and_load(Path::new("."))
        .ok()
        .flatten()
        .and_then(|(_, cfg)| cfg.name)
        .unwrap_or_else(|| "Hardware Board".to_string())
}

fn load_or_build_circuit_json(entry: &Path, runner: &Runner) -> Result<String, ExportError> {
    if entry.extension().and_then(|e| e.to_str()) == Some("json") {
        std::fs::read_to_string(entry).map_err(|e| {
            CheckError::Read(check::ReadError {
                path: entry.display().to_string(),
                source: e,
            })
            .into()
        })
    } else {
        println!(
            "⚙️  Evaluating \"{}\" for native export...",
            entry.display()
        );
        let json_path = runner.build(entry, None)?;
        std::fs::read_to_string(&json_path).map_err(|e| {
            CheckError::Read(check::ReadError {
                path: json_path.display().to_string(),
                source: e,
            })
            .into()
        })
    }
}

fn extract_zip_archive(zip_path: &Path, target_dir: &Path) {
    let file = match std::fs::File::open(zip_path) {
        Ok(f) => f,
        Err(_) => return,
    };
    if let Ok(mut archive) = zip::ZipArchive::new(file) {
        let _ = archive.extract(target_dir);
    }
}

fn export_fab(
    _runner: &Runner,
    _entry: &Path,
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
    std::fs::create_dir_all(&base).ok();
    println!("📦 Running CAD export preset (cad)...");
    let kicad = base.join("kicad.zip");
    println!("  • Exporting KiCad project -> {}", kicad.display());
    runner.export(entry, "kicad_zip", Some(&kicad))?;

    let kicad_dir = base.join("kicad");
    std::fs::create_dir_all(&kicad_dir).ok();
    extract_zip_archive(&kicad, &kicad_dir);
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

fn export_docs(
    _runner: &Runner,
    _entry: &Path,
    circuit_json: &str,
    out: Option<&Path>,
) -> Result<(), ExportError> {
    let base = out
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dist/docs"));
    std::fs::create_dir_all(&base).ok();
    println!("📦 Running documentation export preset (docs)...");
    println!("  • Exporting schematic SVG & PCB layout SVG via native Rust engine...");

    let circuit = crate::parser::parse(circuit_json, Path::new("circuit.json")).ok();
    let dummy_circuit = crate::parser::Circuit {
        elements: Vec::new(),
    };
    let c = circuit.as_ref().unwrap_or(&dummy_circuit);
    let netlist = crate::netlist::Netlist::from_circuit(c);

    crate::svg::export_docs_bundle(circuit_json, c, &netlist, &base)?;
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
    runner: &Runner,
    entry: &Path,
    circuit_json: &str,
    board_title: &str,
    preset: &str,
    custom_output: Option<&Path>,
) -> Result<(), ExportError> {
    validate_preset(preset)?;
    match preset {
        "fab" => export_fab(runner, entry, circuit_json, board_title, custom_output),
        "cad" => export_cad(runner, entry, custom_output),
        "docs" => export_docs(runner, entry, circuit_json, custom_output),
        "all" => {
            export_fab(runner, entry, circuit_json, board_title, custom_output)?;
            println!();
            export_cad(runner, entry, custom_output)?;
            println!();
            export_docs(runner, entry, circuit_json, custom_output)?;
            Ok(())
        }
        _ => unreachable!(),
    }
}
