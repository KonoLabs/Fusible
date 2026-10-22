//! Subcommand: `fusible mechanical` — generate mechanical CAD models from circuit design.

use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::check::{self, CheckError};
use crate::commands::target::resolve_entry;
use crate::config::MechanicalConfig;
use crate::tscircuit::{Runner, RunnerError};

/// Unified error enum for the mechanical pipeline.
#[derive(Debug, Error, Diagnostic)]
pub enum MechanicalError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Check(#[from] CheckError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Runner(#[from] RunnerError),

    #[error("failed to export mechanical files: {0}")]
    #[diagnostic(code(fusible::mechanical::io_error))]
    Io(#[from] std::io::Error),

    #[error("unknown mechanical format \"{format}\"")]
    #[diagnostic(
        code(fusible::mechanical::unknown_format),
        help("valid formats are: `step`, `iges`, `stl`, `glb`, `brep`, or `all`")
    )]
    UnknownFormat { format: String },

    #[error("no board outline found in circuit")]
    #[diagnostic(
        code(fusible::mechanical::no_outline),
        help("ensure the circuit contains a board outline or provide one via --outline")
    )]
    NoOutline,
}

/// Generates mechanical CAD models from a circuit design.
///
/// By default, all exports are **fuse-gated**: electrical rules are checked first.
/// If any violation is found, the process halts and no export occurs.
#[allow(clippy::too_many_arguments)]
pub fn run(
    file: Option<PathBuf>,
    config: MechanicalConfig,
    skip_lint: bool,
) -> Result<(), MechanicalError> {
    let format = config.format.as_deref().unwrap_or("step");
    let output = config.out_dir.clone();
    let outline = config.outline_source.clone().unwrap_or_else(|| "auto".to_string());

    validate_format(format)?;

    // Resolve the entry point for the circuit.
    let entry = resolve_entry(file.clone()).unwrap_or_else(|_| PathBuf::from("index.tsx"));

    if !skip_lint {
        println!("🔍 Verifying electrical rules before mechanical export (fuse check)...");
        check::run(file, None, None)?;
        println!("⚡ Fuse intact — proceeding with mechanical export.\n");
    } else {
        println!("⚠️  Skipping electrical rule verification (--skip-lint requested).\n");
    }

    let runner = Runner::detect().ok();
    let outline_path = resolve_outline(&entry, runner.as_ref(), &outline)?;

    // Use the provided config for export.
    // No need to recreate MechanicalConfig here.

    if format == "all" {
        for fmt in ["step", "iges", "stl", "glb", "brep"] {
            export_mechanical(
                &entry,
                fmt,
                output.as_deref(),
                &outline_path,
                &config,
                runner.as_ref(),
            )?;
        }
    } else {
        export_mechanical(
            &entry,
            format,
            output.as_deref(),
            &outline_path,
            &config,
            runner.as_ref(),
        )?;
    }

    Ok(())
}

fn validate_format(format: &str) -> Result<(), MechanicalError> {
    match format {
        "step" | "iges" | "stl" | "glb" | "brep" | "all" => Ok(()),
        other => Err(MechanicalError::UnknownFormat {
            format: other.to_string(),
        }),
    }
}

/// Derives the board outline from the circuit, KiCad, or SVG source.
fn resolve_outline(
    entry: &Path,
    runner: Option<&Runner>,
    source: &str,
) -> Result<PathBuf, MechanicalError> {
    if source != "auto" {
        let path = PathBuf::from(source);
        if path.exists() {
            return Ok(path);
        }
        return Err(MechanicalError::NoOutline);
    }

    // Try to locate an outline from circuit.json first.
    let candidates = [
        PathBuf::from("dist/circuit.json"),
        PathBuf::from("circuit.json"),
        PathBuf::from("hardware/circuit.json"),
        PathBuf::from("outline.svg"),
        PathBuf::from("hardware/outline.svg"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    // Fall back to building circuit JSON from the entry via the runner.
    if let Some(runner) = runner {
        if entry.extension().and_then(|e| e.to_str()) != Some("json") {
            if let Ok(json_path) = runner.build(entry, None) {
                if json_path.exists() {
                    return Ok(json_path);
                }
            }
        }
    }

    Err(MechanicalError::NoOutline)
}

/// Spawns the TypeScript mechanical generator script.
fn spawn_mechanical_script(
    _runner: &Runner,
    _entry: &Path,
    format: &str,
    outline: &Path,
    config: &MechanicalConfig,
) -> Result<(), MechanicalError> {
    let mut cmd = std::process::Command::new("bun");
    cmd.arg("run")
        .arg("--")
        .arg("scripts/generate-mechanical.ts")
        .arg("--format")
        .arg(format)
        .arg("--outline")
        .arg(outline)
        .arg("--wall-thickness")
        .arg(config.wall_thickness.unwrap_or(2.0).to_string())
        .arg("--ventilation")
        .arg(
            config
                .ventilation
                .clone()
                .unwrap_or_else(|| "none".to_string()),
        )
        .arg("--mounting")
        .arg(
            config
                .mounting
                .clone()
                .unwrap_or_else(|| "none".to_string()),
        );

    if let Some(h) = config.height {
        cmd.arg("--height").arg(h.to_string());
    }
    if config.component_cutouts.unwrap_or(false) {
        cmd.arg("--component-cutouts");
    }

    let out = cmd.output().map_err(|source| RunnerError::Io {
        command: format!("bun run scripts/generate-mechanical.ts --format {format}"),
        source,
    })?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        return Err(RunnerError::CommandFailed {
            command: "bun run scripts/generate-mechanical.ts".to_string(),
            code: out.status.code(),
            stderr: stderr.clone(),
            help: stderr,
        }
        .into());
    }

    Ok(())
}

/// High-level export for a single mechanical format.
fn export_mechanical(
    entry: &Path,
    format: &str,
    output: Option<&Path>,
    outline: &Path,
    config: &MechanicalConfig,
    runner: Option<&Runner>,
) -> Result<(), MechanicalError> {
    let runner = runner.ok_or(RunnerError::NotFound)?;
    let file_stem = entry.file_stem().unwrap_or_default().to_string_lossy();
    let dest = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(format!("dist/mechanical/{}.{}", file_stem, format)));

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    println!(
        "📦 Generating mechanical {} file to {}...",
        format,
        dest.display()
    );

    spawn_mechanical_script(runner, entry, format, outline, config)?;

    println!(
        "✅ Successfully exported mechanical {} to {}!",
        format,
        dest.display()
    );

    Ok(())
}
