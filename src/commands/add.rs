//! Subcommand: `fusible add` — install tscircuit packages or modules into the project.

use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::config::FusibleConfig;
use crate::tscircuit::error::RunnerError;
use crate::tscircuit::Runner;

/// Error raised when adding packages fails.
#[derive(Debug, Error, Diagnostic)]
pub enum AddError {
    #[error("failed to detect tscircuit runner: {0}")]
    #[diagnostic(help("run 'fusible doctor --install' to set up a runner"))]
    Runner(#[from] RunnerError),

    #[error("failed to locate hardware directory or package.json: {0}")]
    Io(#[from] std::io::Error),
}

/// Discovers the hardware workspace directory.
pub fn find_hardware_dir() -> PathBuf {
    if let Some(dir) = manifest_hardware_dir() {
        return dir;
    }

    if Path::new("hardware/package.json").exists() || Path::new("hardware").is_dir() {
        return PathBuf::from("hardware");
    }

    PathBuf::from(".")
}

/// Returns the existing parent directory of the manifest's hardware entry, if any.
fn manifest_hardware_dir() -> Option<PathBuf> {
    let (manifest_path, config) = FusibleConfig::find_and_load(Path::new(".")).ok()??;
    let entry = config.hardware?.entry?;
    let manifest_dir = manifest_path.parent().unwrap_or(Path::new("."));
    let parent = manifest_dir.join(&entry).parent()?.to_path_buf();
    parent.exists().then_some(parent)
}

/// Installs the given packages into the project's hardware workspace.
pub fn run(packages: &[String]) -> Result<(), AddError> {
    let hw_dir = find_hardware_dir();
    let runner = Runner::detect()?;

    println!(
        "📦 Adding {} package(s) via {} in \"{}\"...",
        packages.len(),
        runner.kind().description(),
        hw_dir.display()
    );

    runner.add(packages, Some(&hw_dir))?;

    println!("✅ Successfully added package(s): {}", packages.join(", "));
    Ok(())
}
