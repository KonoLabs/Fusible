//! Fusible — electronic linter and tscircuit project manager.
//!
//! Exit codes are part of the contract (they let CI distinguish what failed):
//! * `0` — fuse intact, command succeeded;
//! * `1` — electrical violations: the fuse blew, generation is blocked;
//! * `2` — tool error (unreadable/malformed input, runner failure, usage error).

pub mod cli;
pub mod commands;
pub mod config;
pub mod linter;
pub mod netlist;
pub mod parser;
pub mod tscircuit;

use std::path::Path;
use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use miette::Diagnostic;

use crate::cli::{Cli, Commands};
use crate::commands::check::CheckError;

/// Exit code: electrical violations (the fuse blew).
pub const EXIT_VIOLATIONS: u8 = 1;
/// Exit code: the tool itself could not do its job (aligns with clap's 2).
pub const EXIT_TOOL_ERROR: u8 = 2;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Dev { file, port }) => match commands::dev::run(file, port) {
            Ok(code) => ExitCode::from(code as u8),
            Err(err) => fail(err, EXIT_TOOL_ERROR),
        },
        Some(Commands::Check { file, mcu }) => dispatch_check(file, mcu),
        Some(Commands::Build { file }) => match commands::build::run(file) {
            Ok(()) => ExitCode::SUCCESS,
            Err(commands::build::BuildError::Check(CheckError::FuseBlown(blown))) => {
                fail(blown, EXIT_VIOLATIONS)
            }
            Err(err) => fail(err, EXIT_TOOL_ERROR),
        },
        Some(Commands::Export {
            file,
            format,
            output,
        }) => match commands::export::run(file, &format, output) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => fail(err, EXIT_TOOL_ERROR),
        },
        Some(Commands::Init { name }) => match commands::init::run(name) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => fail(err, EXIT_TOOL_ERROR),
        },
        None => handle_default_invocation(cli),
    }
}

fn handle_default_invocation(cli: Cli) -> ExitCode {
    let has_manifest = config::FusibleConfig::find_and_load(Path::new("."))
        .ok()
        .flatten()
        .is_some();

    if cli.input.is_some() || has_manifest {
        dispatch_check(cli.input, cli.mcu)
    } else {
        let _ = Cli::command().print_help();
        println!();
        ExitCode::from(EXIT_TOOL_ERROR)
    }
}

fn dispatch_check(file: Option<std::path::PathBuf>, mcu: Option<String>) -> ExitCode {
    match commands::check::run(file, mcu) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CheckError::FuseBlown(blown)) => fail(blown, EXIT_VIOLATIONS),
        Err(err) => fail(err, EXIT_TOOL_ERROR),
    }
}

/// Renders an error through miette's fancy handler on stderr, then returns
/// the mapped exit code.
fn fail(error: impl Diagnostic + Send + Sync + 'static, code: u8) -> ExitCode {
    eprintln!("Error: {:?}", miette::Report::new(error));
    ExitCode::from(code)
}
