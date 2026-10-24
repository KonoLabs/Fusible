//! Runs fuse-gated subcommands and maps their results onto the exit-code contract.

mod body;

pub use body::body;

use std::path::PathBuf;
use std::process::ExitCode;

use crate::cli::{ExportArgs, RouteArgs, SyncArgs};
use crate::commands;
use crate::commands::check::CheckError;
use crate::router::RouteOptions;
use crate::{fail, tool_exit, EXIT_TOOL_ERROR, EXIT_VIOLATIONS};

/// Runs the dev server and forwards its exit code.
pub fn dev(file: Option<PathBuf>, port: Option<u16>, legacy: bool, desktop: bool) -> ExitCode {
    match commands::dev::run(file, port, legacy, desktop) {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(EXIT_TOOL_ERROR)),
        Err(err) => fail(err, EXIT_TOOL_ERROR),
    }
}

/// Runs the electrical linter.
pub fn check(file: Option<PathBuf>, mcu: Option<String>, pins_rs: Option<PathBuf>) -> ExitCode {
    match commands::check::run(file, mcu, pins_rs) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CheckError::FuseBlown(blown)) => fail(blown, EXIT_VIOLATIONS),
        Err(err) => fail(err, EXIT_TOOL_ERROR),
    }
}

/// Runs the full build pipeline.
pub fn build(file: Option<PathBuf>) -> ExitCode {
    match commands::build::run(file) {
        Ok(()) => ExitCode::SUCCESS,
        Err(commands::build::BuildError::Check(CheckError::FuseBlown(blown))) => {
            fail(blown, EXIT_VIOLATIONS)
        }
        Err(err) => fail(err, EXIT_TOOL_ERROR),
    }
}

/// Runs a fuse-gated export.
pub fn export(args: ExportArgs) -> ExitCode {
    let ExportArgs {
        file,
        preset,
        format,
        output,
        skip_lint,
    } = args;
    match commands::export::run(file, preset.as_deref(), &format, output, skip_lint) {
        Ok(()) => ExitCode::SUCCESS,
        Err(commands::export::ExportError::Check(CheckError::FuseBlown(blown))) => {
            fail(blown, EXIT_VIOLATIONS)
        }
        Err(err) => fail(err, EXIT_TOOL_ERROR),
    }
}

/// Runs the autorouter; unrouted nets blow the fuse.
pub fn route(args: RouteArgs) -> ExitCode {
    let options = RouteOptions {
        clearance: args.clearance,
        trace_width: args.trace_width,
        layers: args.layers,
        dry_run: args.dry_run,
        ..Default::default()
    };
    match commands::route::run(args.file, args.output, options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err @ commands::route::RouteCommandError::UnroutedNets { .. }) => {
            fail(err, EXIT_VIOLATIONS)
        }
        Err(err) => fail(err, EXIT_TOOL_ERROR),
    }
}

/// Compares KiCad placement with tscircuit; drift blows the fuse.
pub fn diff(kicad_pcb: PathBuf, file: Option<PathBuf>) -> ExitCode {
    match commands::diff::run(kicad_pcb, file) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err @ commands::diff::DiffCommandError::DriftDetected) => fail(err, EXIT_VIOLATIONS),
        Err(err) => fail(err, EXIT_TOOL_ERROR),
    }
}

/// Syncs KiCad layout edits back into manual-edits.json.
pub fn sync(args: SyncArgs) -> ExitCode {
    tool_exit(commands::sync::run(
        args.kicad_pcb,
        args.file,
        args.target,
        args.write,
        args.dry_run,
    ))
}
