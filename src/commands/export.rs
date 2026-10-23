//! Subcommand: `fusible export` — export circuit artifacts.

mod formats;
mod pdf;
mod presets;

use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::check::{self, CheckError};
use crate::commands::target::{resolve_board, BoardTarget, TargetError};
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
    out_dir: PathBuf,
    output: Option<PathBuf>,
    source: Option<String>,
}

impl ExportJob {
    /// Returns the detected runner, or a not-found error for runner-backed formats.
    fn runner(&self) -> Result<&Runner, RunnerError> {
        self.runner.as_ref().ok_or(RunnerError::NotFound)
    }

    /// Returns the custom output path, or `default` inside the board's output directory.
    fn output_or(&self, default: &str) -> PathBuf {
        self.output
            .clone()
            .unwrap_or_else(|| self.out_dir.join(default))
    }

    /// Returns the Circuit JSON loaded by the fuse gate, loading it when the gate was skipped.
    fn circuit_json(&self) -> Result<String, ExportError> {
        match &self.source {
            Some(source) => Ok(source.clone()),
            None => Ok(check::load_circuit_source(&self.entry)?.1),
        }
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

    let board = match resolve_board(file) {
        Err(TargetError::Missing) if skip_lint => BoardTarget {
            entry: PathBuf::from("index.tsx"),
            out_dir: PathBuf::from("dist"),
            ..Default::default()
        },
        other => other.map_err(CheckError::from)?,
    };
    let source = fuse_gate(&board, skip_lint)?;

    let job = ExportJob {
        runner: Runner::detect().ok(),
        entry: board.entry,
        board_title: board.title,
        out_dir: board.out_dir,
        output,
        source,
    };

    match preset {
        Some(preset_name) => presets::run_preset(&job, preset_name),
        None => formats::run_format(&job, format),
    }
}

/// Runs the electrical check unless the user asked to skip it.
///
/// Returns the Circuit JSON the check validated, or `None` when it was skipped.
fn fuse_gate(board: &BoardTarget, skip_lint: bool) -> Result<Option<String>, ExportError> {
    if skip_lint {
        println!("⚠️  Skipping electrical rule verification (--skip-lint requested).\n");
        return Ok(None);
    }
    println!("🔍 Verifying electrical rules before export (fuse check)...");
    let (_, source) = check::lint_source(&board.entry, board.mcu.clone())?;
    println!("⚡ Fuse intact — proceeding with artifact export.\n");
    Ok(Some(source))
}

/// Extracts every entry of `zip_path` into `target_dir`.
fn extract_zip_archive(zip_path: &Path, target_dir: &Path) -> Result<(), ExportError> {
    let file = std::fs::File::open(zip_path)?;
    zip::ZipArchive::new(file)
        .and_then(|mut archive| archive.extract(target_dir))
        .map_err(|err| ExportError::Io(std::io::Error::other(err)))
}
