//! Errors of the assembly pipeline.

use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::body::BodyError;
use crate::commands::check::CheckError;
use crate::commands::target::TargetError;

/// Unified error enum for `fusible assembly`.
#[derive(Debug, Error, Diagnostic)]
pub enum AssemblyError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Check(#[from] CheckError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Target(#[from] TargetError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Body(#[from] BodyError),

    #[error("failed to write assembly files: {0}")]
    #[diagnostic(code(fusible::assembly::io_error))]
    Io(#[from] std::io::Error),

    #[error("fusible.yml has no `assembly:` section")]
    #[diagnostic(
        code(fusible::assembly::missing),
        help("add `assembly:` with the `parts` to place, e.g. `parts: {{main: {{}}}}`")
    )]
    Missing,

    #[error("unknown assembly format \"{format}\"")]
    #[diagnostic(
        code(fusible::assembly::unknown_format),
        help("valid formats are: `glb`, `step`, or `all`")
    )]
    UnknownFormat { format: String },

    #[error("board `{name}` has no tscircuit source, so it has no 3D model")]
    #[diagnostic(
        code(fusible::assembly::no_board_model),
        help(
            "point the board's `entry:` at its .tsx file; a Circuit JSON alone cannot be modeled"
        )
    )]
    NoBoardModel { name: String },

    #[error("board `{name}` needs a tscircuit runner to export its 3D model: {reason}")]
    #[diagnostic(
        code(fusible::assembly::board_model_failed),
        help("run `fusible doctor` to check the tscircuit runner, then rerun `fusible assembly`")
    )]
    BoardModelFailed { name: String, reason: String },

    #[error("body `{name}`: \"{}\" is not a STEP or STL file", path.display())]
    #[diagnostic(
        code(fusible::assembly::unsupported_source),
        help("export the body as .step, .stp, or .stl to place it in the assembly")
    )]
    UnsupportedSource { name: String, path: PathBuf },
}
