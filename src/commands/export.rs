//! Subcommand: `fusible export` — export circuit artifacts.

mod formats;
mod presets;

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

/// Everything an export step needs once the fuse check has passed.
struct ExportJob {
    runner: Option<Runner>,
    entry: PathBuf,
    board_title: String,
    output: Option<PathBuf>,
}

impl ExportJob {
    /// Returns the detected runner, or a not-found error for runner-backed formats.
    fn runner(&self) -> Result<&Runner, RunnerError> {
        self.runner.as_ref().ok_or(RunnerError::NotFound)
    }

    /// Returns the custom output path, or `default` when none was given.
    fn output_or(&self, default: &str) -> PathBuf {
        self.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(default))
    }

    /// Loads the Circuit JSON for the job's entry file.
    fn circuit_json(&self) -> Result<String, ExportError> {
        Ok(check::load_circuit_source(&self.entry)?.1)
    }
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
        presets::validate_preset(preset_name)?;
    }

    let entry = resolve_entry(file.clone()).unwrap_or_else(|_| PathBuf::from("index.tsx"));
    fuse_gate(file, skip_lint)?;

    let job = ExportJob {
        runner: Runner::detect().ok(),
        entry,
        board_title: check::resolve_board_name(),
        output,
    };

    match preset {
        Some(preset_name) => presets::run_preset(&job, preset_name),
        None => formats::run_format(&job, format),
    }
}

/// Runs the electrical check unless the user asked to skip it.
fn fuse_gate(file: Option<PathBuf>, skip_lint: bool) -> Result<(), ExportError> {
    if skip_lint {
        println!("⚠️  Skipping electrical rule verification (--skip-lint requested).\n");
        return Ok(());
    }
    println!("🔍 Verifying electrical rules before export (fuse check)...");
    check::run(file, None, None)?;
    println!("⚡ Fuse intact — proceeding with artifact export.\n");
    Ok(())
}

/// Extracts every entry of `zip_path` into `target_dir`.
fn extract_zip_archive(zip_path: &Path, target_dir: &Path) -> Result<(), ExportError> {
    let file = std::fs::File::open(zip_path)?;
    zip::ZipArchive::new(file)
        .and_then(|mut archive| archive.extract(target_dir))
        .map_err(|err| ExportError::Io(std::io::Error::other(err)))
}
