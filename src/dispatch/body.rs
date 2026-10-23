//! Dispatch of `fusible body`: build, eject, and kit.

use std::process::ExitCode;

use crate::cli::{BodyAction, BodyArgs};
use crate::commands;
use crate::commands::body::BodyError;
use crate::commands::check::CheckError;
use crate::config::EnclosureConfig;
use crate::{fail, EXIT_TOOL_ERROR, EXIT_VIOLATIONS};

/// Maps a body result onto the exit-code contract.
fn body_exit(result: Result<(), BodyError>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(BodyError::Check(CheckError::FuseBlown(blown))) => fail(blown, EXIT_VIOLATIONS),
        Err(err) => fail(err, EXIT_TOOL_ERROR),
    }
}

/// Builds the project bodies, or ejects one or writes a kit; CLI flags override enclosure settings.
pub fn body(args: BodyArgs) -> ExitCode {
    match args.action {
        Some(BodyAction::Eject(a)) => {
            return body_exit(commands::body::eject(a.name, a.output, a.skip_lint));
        }
        Some(BodyAction::Kit(a)) => {
            return body_exit(commands::body::kit(a.board, a.output, a.skip_lint));
        }
        None => {}
    }
    let overrides = EnclosureConfig {
        format: args.format,
        out_dir: args.output,
        wall_thickness: args.wall_thickness,
        height: args.height,
        cutouts: args.no_cutouts.then_some(false),
        ventilation: args.ventilation,
        mounting: args.no_mounting.then_some(false),
    };
    body_exit(commands::body::run(args.target, overrides, args.skip_lint))
}
