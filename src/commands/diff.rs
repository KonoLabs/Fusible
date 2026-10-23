use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::check::{load_circuit_source, CheckError};
use crate::commands::target::{resolve_entry, TargetError};
use crate::kicad::diff::parse_circuit_elements;
use crate::kicad::{
    compare_kicad_with_circuit, extract_footprints, format_diff_report, parse_sexpr,
};

#[derive(Debug, Error, Diagnostic)]
pub enum DiffCommandError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Target(#[from] TargetError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Check(#[from] CheckError),

    #[error("cannot read KiCad PCB file \"{path}\": {source}")]
    #[diagnostic(code(fusible::diff::read_error))]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid Circuit JSON in \"{path}\": {source}")]
    #[diagnostic(code(fusible::diff::invalid_circuit_json))]
    CircuitJson {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to parse KiCad S-expression: {0}")]
    #[diagnostic(code(fusible::diff::sexpr_error))]
    Sexpr(String),

    #[error("layout drift detected between KiCad and tscircuit")]
    #[diagnostic(
        code(fusible::diff::drift_detected),
        help("run `fusible sync <kicad_pcb> --write` to reconcile changes into manual-edits.json")
    )]
    DriftDetected,
}

pub fn run(kicad_pcb: PathBuf, file: Option<PathBuf>) -> Result<(), DiffCommandError> {
    let content = std::fs::read_to_string(&kicad_pcb).map_err(|source| DiffCommandError::Read {
        path: kicad_pcb.display().to_string(),
        source,
    })?;

    let sexpr = parse_sexpr(&content).map_err(DiffCommandError::Sexpr)?;
    let footprints = extract_footprints(&sexpr);

    let entry = resolve_entry(file)?;
    let (json_path, source) = load_circuit_source(&entry)?;
    let elements =
        parse_circuit_elements(&source).map_err(|source| DiffCommandError::CircuitJson {
            path: json_path.display().to_string(),
            source,
        })?;

    let report = compare_kicad_with_circuit(&footprints, &elements);
    print!("{}", format_diff_report(&report));

    if report.has_drift {
        Err(DiffCommandError::DriftDetected)
    } else {
        println!("✅ Layout synchronized: zero component drift detected.");
        Ok(())
    }
}
