//! Subcommand: `fusible install` — install all dependencies in the hardware workspace.

use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::add::find_hardware_dir;
use crate::tscircuit::error::RunnerError;
use crate::tscircuit::Runner;

/// Error raised when dependency installation fails.
#[derive(Debug, Error, Diagnostic)]
pub enum InstallError {
    #[error("failed to detect tscircuit runner: {0}")]
    #[diagnostic(help("run 'fusible doctor --install' to set up a runner"))]
    Runner(#[from] RunnerError),

    #[error("failed to install dependencies: {0}")]
    Io(#[from] std::io::Error),
}

/// Installs dependencies in the project's hardware workspace.
pub fn run(dir: Option<PathBuf>) -> Result<(), InstallError> {
    let hw_dir = dir.unwrap_or_else(find_hardware_dir);
    let runner = Runner::detect()?;

    println!(
        "📦 Installing dependencies via {} in \"{}\"...",
        runner.kind().description(),
        hw_dir.display()
    );

    runner.install(Some(&hw_dir))?;

    println!("✅ All hardware dependencies are up to date.");
    Ok(())
}
