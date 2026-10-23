//! Errors of the body pipeline.

use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::check::CheckError;
use crate::commands::target::TargetError;

/// Unified error enum for the body pipeline.
#[derive(Debug, Error, Diagnostic)]
pub enum BodyError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Check(#[from] CheckError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Target(#[from] TargetError),

    #[error("body `{name}`: source file \"{}\" does not exist", path.display())]
    #[diagnostic(
        code(fusible::body::missing_source),
        help("export the model from your CAD tool to this path, or fix `source:` in fusible.yml")
    )]
    MissingSource { name: String, path: PathBuf },

    #[error("body `{name}`: build command `{command}` failed")]
    #[diagnostic(code(fusible::body::build_failed))]
    BuildFailed { name: String, command: String },

    #[error("failed to write body files: {0}")]
    #[diagnostic(code(fusible::body::io_error))]
    Io(#[from] std::io::Error),

    #[error("unknown body format \"{format}\"")]
    #[diagnostic(
        code(fusible::body::unknown_format),
        help("valid formats are: `step`, `stl`, `glb`, `brep`, or `all`")
    )]
    UnknownFormat { format: String },

    #[error("unknown ventilation pattern \"{pattern}\"")]
    #[diagnostic(
        code(fusible::body::unknown_ventilation),
        help("valid patterns are: `none`, `grid`, or `slots`")
    )]
    UnknownVentilation { pattern: String },

    #[error("invalid {name}: {value} mm")]
    #[diagnostic(code(fusible::body::invalid_dimension), help("{help}"))]
    InvalidDimension {
        name: &'static str,
        value: f64,
        help: String,
    },

    #[error("no usable pcb_board outline in the circuit")]
    #[diagnostic(
        code(fusible::body::no_board),
        help("add a <board width=\"..\" height=\"..\" /> or an outline with at least 3 points")
    )]
    NoBoard,

    #[error("no body named `{name}` in fusible.yml")]
    #[diagnostic(code(fusible::body::unknown_body), help("{help}"))]
    UnknownBody { name: String, help: String },

    #[error("body `{name}` is authored, not generated")]
    #[diagnostic(
        code(fusible::body::not_generated),
        help(
            "only a body with `generate:` can be ejected; an authored body already has a `source:`"
        )
    )]
    NotGenerated { name: String },

    #[error("\"{}\" already exists", path.display())]
    #[diagnostic(
        code(fusible::body::exists),
        help("fusible never overwrites a file you may have edited; move it or pass another `-o` path")
    )]
    Exists { path: PathBuf },

    #[error("{} was left unchanged: {reason}", manifest.display())]
    #[diagnostic(
        code(fusible::body::manifest_unchanged),
        help(
            "the model is written to {model}. In body `{name}`, remove `generate:` and its settings \
             (format, out_dir, wall_thickness, height, cutouts, ventilation, mounting) and add `source: {model}`"
        )
    )]
    ManifestUnchanged {
        manifest: PathBuf,
        name: String,
        model: String,
        reason: String,
    },

    #[error("cannot read the Circuit JSON: {0}")]
    #[diagnostic(
        code(fusible::body::invalid_circuit),
        help("rebuild the board so its Circuit JSON is valid, then rerun")
    )]
    InvalidCircuit(String),

    #[error("kit.json is written to {dir}, but the board model export failed: {reason}")]
    #[diagnostic(
        code(fusible::body::kit_step_failed),
        help("run `fusible doctor` to check the tscircuit runner, then rerun `fusible body kit`")
    )]
    KitStepFailed { dir: String, reason: String },

    #[error("bun is required to run the enclosure generator")]
    #[diagnostic(
        code(fusible::body::bun_missing),
        help("install bun from https://bun.sh, then rerun `fusible body`")
    )]
    BunMissing,

    #[error("enclosure generator failed (`{command}`, exit code {code:?})")]
    #[diagnostic(code(fusible::body::generator_failed), help("{stderr}"))]
    GeneratorFailed {
        command: String,
        code: Option<i32>,
        stderr: String,
    },
}
