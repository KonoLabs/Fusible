//! Subcommand: `fusible dev` / `view` / `webview` — live previewer.

use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::target::{resolve_boards, BoardTarget, TargetError};
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
///
/// Without a target it serves every board of the manifest in one studio.
pub fn run(file: Option<PathBuf>, port: Option<u16>, legacy: bool) -> Result<i32, DevError> {
    let boards = match resolve_boards(file) {
        Err(TargetError::Missing) => vec![fallback_board()],
        other => other?,
    };
    let resolved_port = port.or(boards[0].port);
    let port_num = resolved_port.unwrap_or(3020);

    if legacy {
        return run_legacy(&boards, resolved_port, port_num);
    }

    let names: Vec<_> = boards
        .iter()
        .map(|b| b.entry.display().to_string())
        .collect();
    println!(
        "⚡ Starting Fusible Studio dev server for {}...",
        names.join(", ")
    );
    crate::dev::run_dev_server(boards, port_num).map_err(|e| DevError::Server(e.to_string()))?;

    Ok(0)
}

fn fallback_board() -> BoardTarget {
    BoardTarget {
        entry: PathBuf::from("index.tsx"),
        ..Default::default()
    }
}

fn run_legacy(
    boards: &[BoardTarget],
    resolved_port: Option<u16>,
    port_num: u16,
) -> Result<i32, DevError> {
    if boards.len() > 1 {
        let names = boards.iter().filter_map(|b| b.name.clone()).collect();
        return Err(TargetError::Ambiguous { names }.into());
    }
    let runner = Runner::detect()?;
    let entry = &boards[0].entry;
    println!(
        "⚡ Starting legacy tsci dev viewer for \"{}\"...",
        entry.display()
    );
    println!("🌐 Dev server active at http://localhost:{port_num} (press Ctrl+C to quit)");
    let status = runner.dev(entry, resolved_port)?;
    Ok(status.code().unwrap_or(2))
}
