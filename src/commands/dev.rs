//! Subcommand: `fusible dev` / `view` / `webview` — live previewer.

use std::path::{Path, PathBuf};

use crate::commands::target::resolve_entry;
use crate::config::FusibleConfig;
use crate::tscircuit::{Runner, RunnerError};

/// Starts the tscircuit development server and interactive web viewer.
pub fn run(file: Option<PathBuf>, port: Option<u16>) -> Result<i32, RunnerError> {
    let runner = Runner::detect()?;
    let entry = resolve_entry(file).unwrap_or_else(|_| PathBuf::from("index.tsx"));

    let resolved_port = port.or_else(|| {
        FusibleConfig::find_and_load(Path::new("."))
            .ok()
            .flatten()
            .and_then(|(_, cfg)| cfg.hardware.and_then(|hw| hw.port))
    });

    let port_num = resolved_port.unwrap_or(3020);
    println!(
        "⚡ Starting tscircuit dev viewer for \"{}\"...",
        entry.display()
    );
    println!("🌐 Dev server active at http://localhost:{port_num} (press Ctrl+C to quit)");
    let status = runner.dev(&entry, resolved_port)?;
    Ok(status.code().unwrap_or(0))
}
