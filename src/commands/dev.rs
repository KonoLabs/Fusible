//! Subcommand: `fusible dev` / `view` / `webview` — live previewer.

use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::target::{resolve_board, BoardTarget, TargetError};
use crate::tscircuit::{Runner, RunnerError};

#[derive(Debug, Error, Diagnostic)]
pub enum DevError {
    #[error("failed to run dev server: {0}")]
    #[diagnostic(code(fusible::dev::server_error))]
    Server(String),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Runner(#[from] RunnerError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Target(#[from] TargetError),
}

/// Starts the Fusible development server and interactive dark-theme web viewer.
pub fn run(file: Option<PathBuf>, port: Option<u16>, legacy: bool) -> Result<i32, DevError> {
    let board = match resolve_board(file) {
        Err(TargetError::Missing) => BoardTarget {
            entry: PathBuf::from("index.tsx"),
            ..Default::default()
        },
        other => other?,
    };
    let entry = board.entry;
    let resolved_port = port.or(board.port);

    let port_num = resolved_port.unwrap_or(3020);

    if legacy {
        let runner = Runner::detect()?;
        println!(
            "⚡ Starting legacy tsci dev viewer for \"{}\"...",
            entry.display()
        );
        println!("🌐 Dev server active at http://localhost:{port_num} (press Ctrl+C to quit)");
        let status = runner.dev(&entry, resolved_port)?;
        return Ok(status.code().unwrap_or(2));
    }

    println!(
        "⚡ Starting Fusible Studio dev server for \"{}\"...",
        entry.display()
    );
    crate::dev::run_dev_server(entry, port_num).map_err(|e| DevError::Server(e.to_string()))?;

    Ok(0)
}
