//! Subcommand: `fusible export` — export circuit artifacts.

use std::path::PathBuf;

use crate::commands::target::resolve_entry;
use crate::tscircuit::{Runner, RunnerError};

/// Exports tscircuit source files into various formats (Gerbers, SVG, 3D, etc.).
pub fn run(
    file: Option<PathBuf>,
    format: &str,
    output: Option<PathBuf>,
) -> Result<(), RunnerError> {
    let runner = Runner::detect()?;
    let entry = resolve_entry(file).unwrap_or_else(|_| PathBuf::from("index.tsx"));

    println!(
        "📦 Exporting \"{}\" to format: {}...",
        entry.display(),
        format
    );
    runner.export(&entry, format, output.as_deref())
}
