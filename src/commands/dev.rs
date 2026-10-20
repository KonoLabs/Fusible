//! Subcommand: `fusible dev` / `view` / `webview` — live previewer.

use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::target::resolve_entry;
use crate::config::FusibleConfig;
use crate::tscircuit::{Runner, RunnerError};

#[derive(Debug, Error, Diagnostic)]
pub enum DevError {
    #[error("failed to run dev server: {0}")]
    #[diagnostic(code(fusible::dev::server_error))]
    Server(String),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Runner(#[from] RunnerError),
}

/// Starts the Fusible development server and interactive dark-theme web viewer.
pub fn run(file: Option<PathBuf>, port: Option<u16>, legacy: bool) -> Result<i32, DevError> {
    let entry = resolve_entry(file).unwrap_or_else(|_| PathBuf::from("index.tsx"));

    let resolved_port = port.or_else(|| {
        FusibleConfig::find_and_load(Path::new("."))
            .ok()
            .flatten()
            .and_then(|(_, cfg)| cfg.hardware.and_then(|hw| hw.port))
    });

    let port_num = resolved_port.unwrap_or(3020);

    if legacy {
        let runner = Runner::detect()?;
        println!(
            "⚡ Starting legacy tsci dev viewer for \"{}\"...",
            entry.display()
        );
        println!("🌐 Dev server active at http://localhost:{port_num} (press Ctrl+C to quit)");
        let status = runner.dev(&entry, resolved_port)?;
        return Ok(status.code().unwrap_or(0));
    }

    println!(
        "⚡ Starting Fusible Studio dev server for \"{}\"...",
        entry.display()
    );
    crate::dev::run_dev_server(entry, port_num).map_err(|e| DevError::Server(e.to_string()))?;

    Ok(0)
}
