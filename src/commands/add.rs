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
    if let Ok(Some((manifest_path, config))) = FusibleConfig::find_and_load(Path::new(".")) {
        if let Some(hw) = config.hardware {
            if let Some(entry) = hw.entry {
                let manifest_dir = manifest_path.parent().unwrap_or(Path::new("."));
                let resolved = manifest_dir.join(&entry);
                if let Some(parent) = resolved.parent() {
                    if parent.exists() {
                        return parent.to_path_buf();
                    }
                }
            }
        }
    }

    if Path::new("hardware/package.json").exists() || Path::new("hardware").is_dir() {
        return PathBuf::from("hardware");
    }

    PathBuf::from(".")
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
