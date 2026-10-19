//! Subcommand: `fusible check` / `lint` — run the electrical linter.

use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::target::{is_tsx_target, resolve_entry, MissingTargetError};
use crate::config::FusibleConfig;
use crate::linter::{FuseBlown, Linter};
use crate::parser;
use crate::tscircuit::{Runner, RunnerError};

/// The input file could not be read.
#[derive(Debug, Error, Diagnostic)]
#[error("cannot read the Circuit JSON at \"{path}\"")]
#[diagnostic(help("check that the file exists and is readable"))]
pub struct ReadError {
    pub path: String,
    #[source]
    pub source: std::io::Error,
}

/// Unified error enum for the check pipeline.
#[derive(Debug, Error, Diagnostic)]
pub enum CheckError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingTarget(#[from] MissingTargetError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Runner(#[from] RunnerError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Read(#[from] ReadError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Parse(#[from] parser::JsonDiagnostic),

    #[error(transparent)]
    #[diagnostic(transparent)]
    FuseBlown(#[from] FuseBlown),
}

/// Runs the electrical linter on a target file or project manifest.
pub fn run(file: Option<PathBuf>, mcu: Option<String>) -> Result<(), CheckError> {
    let entry = resolve_entry(file)?;
    let target_mcu = resolve_mcu(mcu);

    let (json_path, source) = if is_tsx_target(&entry) {
        println!("⚙️  Compiling \"{}\" via tscircuit...", entry.display());
        let runner = Runner::detect()?;
        let built_json = runner.build(&entry, None)?;
        let content = std::fs::read_to_string(&built_json).map_err(|source| ReadError {
            path: built_json.display().to_string(),
            source,
        })?;
        (built_json, content)
    } else {
        let content = std::fs::read_to_string(&entry).map_err(|source| ReadError {
            path: entry.display().to_string(),
            source,
        })?;
        (entry, content)
    };

    let circuit = parser::parse(&source, &json_path)?;
    let violations = Linter::default().run(&circuit);

    if violations.is_empty() {
        println!("✅ Circuit validated (fuse intact)");
        if let Some(mcu) = target_mcu {
            println!("   silicon target: {mcu} (pin-muxing: phase 2)");
        }
        return Ok(());
    }

    Err(CheckError::FuseBlown(FuseBlown::new(violations)))
}

fn resolve_mcu(explicit: Option<String>) -> Option<String> {
    explicit.or_else(|| {
        FusibleConfig::find_and_load(Path::new("."))
            .ok()
            .flatten()
            .and_then(|(_, cfg)| cfg.mcu.and_then(|m| m.chip))
    })
}
