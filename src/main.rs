//! Fusible — electronic linter and tscircuit project manager.
//!
//! Exit codes are part of the contract (they let CI distinguish what failed):
//! * `0` — fuse intact, command succeeded;
//! * `1` — electrical violations: the fuse blew, generation is blocked;
//! * `2` — tool error (unreadable/malformed input, runner failure, usage error).

pub mod cli;
pub mod codegen;
pub mod commands;
pub mod config;
pub mod dev;
mod dispatch;
pub mod gerber;
pub mod kicad;
pub mod linter;
pub mod netlist;
pub mod parser;
pub mod router;
pub mod silicon;
pub mod svg;
pub mod tscircuit;

use std::process::ExitCode;

use clap::Parser;
use miette::Diagnostic;

use crate::cli::{Cli, Commands};

/// Exit code: electrical violations (the fuse blew).
pub const EXIT_VIOLATIONS: u8 = 1;
/// Exit code: the tool itself could not do its job (aligns with clap's 2).
pub const EXIT_TOOL_ERROR: u8 = 2;

fn main() -> ExitCode {
    let cli = Cli::parse();

    if !matches!(
        cli.command,
        Some(Commands::Init(_) | Commands::Doctor { .. })
    ) {
        if let Err(err) = config::FusibleConfig::find_and_load(std::path::Path::new(".")) {
            return fail(err, EXIT_TOOL_ERROR);
        }
    }

    match cli.command {
        Some(command) => run_command(command),
        None => dispatch::check(cli.input, cli.mcu, None),
    }
}

/// Dispatches one subcommand and returns its exit code.
fn run_command(command: Commands) -> ExitCode {
    match command {
        Commands::Dev { file, port, legacy } => dispatch::dev(file, port, legacy),
        Commands::Check { file, mcu, pins_rs } => dispatch::check(file, mcu, pins_rs),
        Commands::Build { file } => dispatch::build(file),
        Commands::Export(args) => dispatch::export(args),
        Commands::Mechanical(args) => dispatch::mechanical(args),
        Commands::Route(args) => dispatch::route(args),
        Commands::Diff { kicad_pcb, file } => dispatch::diff(kicad_pcb, file),
        Commands::Sync(args) => dispatch::sync(args),
        Commands::Doctor { install } => tool_exit(commands::doctor::run(install)),
        Commands::Init(a) => tool_exit(commands::init::run(a.path, a.name, a.mcu, a.no_install)),
        Commands::Add { packages } => tool_exit(commands::add::run(&packages)),
        Commands::Install { dir } => tool_exit(commands::install::run(dir)),
        Commands::Search { query } => tool_exit(commands::search::run(&query)),
    }
}

/// Maps a command result where every error is a tool error.
fn tool_exit<E: Diagnostic + Send + Sync + 'static>(result: Result<(), E>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => fail(err, EXIT_TOOL_ERROR),
    }
}

/// Renders an error through miette's fancy handler on stderr, then returns
/// the mapped exit code.
fn fail(error: impl Diagnostic + Send + Sync + 'static, code: u8) -> ExitCode {
    eprintln!("Error: {:?}", miette::Report::new(error));
    ExitCode::from(code)
}
