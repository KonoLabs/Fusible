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
    let entry = resolve_entry(file.clone()).unwrap_or_else(|_| PathBuf::from("index.tsx"));

    if !skip_lint {
        println!("🔍 Verifying electrical rules before export (fuse check)...");
        check::run(file, None, None)?;
        println!("⚡ Fuse intact — proceeding with artifact export.\n");
    } else {
        println!("⚠️  Skipping electrical rule verification (--skip-lint requested).\n");
    }

    let runner = Runner::detect()?;

    if let Some(preset_name) = preset {
        run_preset(&runner, &entry, preset_name, output.as_deref())
    } else if format == "kicad_project" || format == "kicad" {
        let dest = output.unwrap_or_else(|| PathBuf::from("dist/cad/kicad"));
        std::fs::create_dir_all(&dest).ok();
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

fn extract_zip_archive(zip_path: &Path, target_dir: &Path) {
    let mut cmd = std::process::Command::new("unzip");
    cmd.args(["-o", "-q"])
        .arg(zip_path)
        .arg("-d")
        .arg(target_dir);
    let _ = cmd.status();
}

fn export_fab(runner: &Runner, entry: &Path, out: Option<&Path>) -> Result<(), ExportError> {
    let base = out
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dist/fab"));
    std::fs::create_dir_all(&base).ok();
    println!("📦 Running fabrication export preset (fab)...");
    let gerbers = base.join("gerbers.zip");
    println!("  • Exporting gerbers -> {}", gerbers.display());
    runner.export(entry, "gerbers", Some(&gerbers))?;
    let json = base.join("circuit.json");
    println!("  • Exporting circuit-json -> {}", json.display());
    runner.export(entry, "circuit-json", Some(&json))?;
    println!(
        "✅ Fabrication artifacts generated in \"{}\"",
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

fn export_docs(runner: &Runner, entry: &Path, out: Option<&Path>) -> Result<(), ExportError> {
    let base = out
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dist/docs"));
    std::fs::create_dir_all(&base).ok();
    println!("📦 Running documentation export preset (docs)...");
    let sch = base.join("schematic.svg");
    println!("  • Exporting schematic SVG -> {}", sch.display());
    runner.export(entry, "schematic-svg", Some(&sch))?;
    let pcb = base.join("pcb.svg");
    println!("  • Exporting PCB layout SVG -> {}", pcb.display());
    runner.export(entry, "pcb-svg", Some(&pcb))?;
    let pdf = base.join("schematic.pdf");
    println!("  • Exporting schematic PDF -> {}", pdf.display());
    runner.export(entry, "schematic-pdf", Some(&pdf))?;
    println!(
        "✅ Documentation artifacts generated in \"{}\"",
        base.display()
    );
    Ok(())
}

fn run_preset(
    runner: &Runner,
    entry: &Path,
    preset: &str,
    custom_output: Option<&Path>,
) -> Result<(), ExportError> {
    match preset {
        "fab" => export_fab(runner, entry, custom_output),
        "cad" => export_cad(runner, entry, custom_output),
        "docs" => export_docs(runner, entry, custom_output),
        "all" => {
            export_fab(runner, entry, custom_output)?;
            println!();
            export_cad(runner, entry, custom_output)?;
            println!();
            export_docs(runner, entry, custom_output)?;
            Ok(())
        }
        other => Err(ExportError::UnknownPreset {
            preset: other.to_string(),
        }),
    }
}
