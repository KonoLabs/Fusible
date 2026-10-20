use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::check::{load_circuit_source, CheckError};
use crate::commands::target::{resolve_entry, MissingTargetError};
use crate::router::{route_circuit, RouteOptions};

#[derive(Debug, Error, Diagnostic)]
pub enum RouteCommandError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingTarget(#[from] MissingTargetError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Check(#[from] CheckError),

    #[error("failed to execute autorouter: {0}")]
    #[diagnostic(code(fusible::route::engine_error))]
    Engine(String),

    #[error("failed to write routed output: {0}")]
    #[diagnostic(code(fusible::route::io_error))]
    Io(#[from] std::io::Error),

    #[error("the fuse blew: {count} unrouted net(s) could not be connected: {nets:?}")]
    #[diagnostic(
        code(fusible::route::unrouted_nets),
        help("adjust component placement or relax clearance rules to resolve routing congestion")
    )]
    UnroutedNets { count: usize, nets: Vec<String> },
}

pub fn run(
    file: Option<PathBuf>,
    output: Option<PathBuf>,
    clearance: f64,
    trace_width: f64,
    layers: u8,
    dry_run: bool,
) -> Result<(), RouteCommandError> {
    let entry = resolve_entry(file)?;
    println!(
        "⚡ Initializing parallel autorouter for \"{}\"...",
        entry.display()
    );

    let (_json_path, source) = load_circuit_source(&entry)?;

    let options = RouteOptions {
        clearance,
        trace_width,
        layers,
        dry_run,
        ..Default::default()
    };

    let result = route_circuit(&source, &options).map_err(RouteCommandError::Engine)?;

    println!(
        "📊 Autorouting complete: {}/{} net(s) routed ({} trace(s), {} via(s))",
        result.routed_nets, result.total_nets, result.generated_traces, result.generated_vias
    );

    if !result.unrouted_nets.is_empty() {
        return Err(RouteCommandError::UnroutedNets {
            count: result.unrouted_nets.len(),
            nets: result.unrouted_nets,
        });
    }

    if !dry_run {
        let dest = output.unwrap_or_else(|| {
            if entry.extension().and_then(|e| e.to_str()) == Some("json") {
                entry.clone()
            } else {
                PathBuf::from("dist").join("circuit.routed.json")
            }
        });

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&dest, result.output_json)?;
        println!(
            "✅ Successfully exported routed circuit to \"{}\"!",
            dest.display()
        );
    } else {
        println!("🔍 Dry run complete — verified all nets routable (fuse intact).");
    }

    Ok(())
}
