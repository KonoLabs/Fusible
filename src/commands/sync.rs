use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::check::{load_circuit_source, CheckError};
use crate::commands::target::{resolve_entry, MissingTargetError};
use crate::kicad::diff::parse_circuit_elements;
use crate::kicad::{apply_kicad_sync, compare_kicad_with_circuit, extract_footprints, parse_sexpr};

#[derive(Debug, Error, Diagnostic)]
pub enum SyncCommandError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    MissingTarget(#[from] MissingTargetError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Check(#[from] CheckError),

    #[error("cannot read KiCad PCB file \"{path}\": {source}")]
    #[diagnostic(code(fusible::sync::read_error))]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid Circuit JSON in \"{path}\": {source}")]
    #[diagnostic(code(fusible::sync::invalid_circuit_json))]
    CircuitJson {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to parse KiCad S-expression: {0}")]
    #[diagnostic(code(fusible::sync::sexpr_error))]
    Sexpr(String),

    #[error("failed to sync manual edits: {0}")]
    #[diagnostic(code(fusible::sync::sync_error))]
    Sync(String),
}

pub fn run(
    kicad_pcb: PathBuf,
    file: Option<PathBuf>,
    target: Option<PathBuf>,
    write: bool,
    dry_run: bool,
) -> Result<(), SyncCommandError> {
    let content = std::fs::read_to_string(&kicad_pcb).map_err(|source| SyncCommandError::Read {
        path: kicad_pcb.display().to_string(),
        source,
    })?;

    let sexpr = parse_sexpr(&content).map_err(SyncCommandError::Sexpr)?;
    let footprints = extract_footprints(&sexpr);

    let entry = resolve_entry(file)?;
    let (json_path, source) = load_circuit_source(&entry)?;
    let elements =
        parse_circuit_elements(&source).map_err(|source| SyncCommandError::CircuitJson {
            path: json_path.display().to_string(),
            source,
        })?;

    let report = compare_kicad_with_circuit(&footprints, &elements);

    let manual_edits_path = target.unwrap_or_else(|| default_manual_edits_path(&entry));

    let is_dry = dry_run || !write;
    let result_msg =
        apply_kicad_sync(&report, &manual_edits_path, is_dry).map_err(SyncCommandError::Sync)?;

    report_sync(&result_msg, &manual_edits_path, is_dry);
    Ok(())
}

/// Picks `hardware/manual-edits.json` in a hardware workspace, else one beside `entry`.
fn default_manual_edits_path(entry: &Path) -> PathBuf {
    if Path::new("hardware/manual-edits.json").is_file()
        || Path::new("hardware/package.json").is_file()
    {
        return PathBuf::from("hardware/manual-edits.json");
    }
    match entry.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join("manual-edits.json"),
        _ => PathBuf::from("manual-edits.json"),
    }
}

/// Prints the sync outcome and the suggested next step.
fn report_sync(result_msg: &str, manual_edits_path: &Path, is_dry: bool) {
    if is_dry {
        println!("🔍 Dry-run: {result_msg}");
        println!(
            "💡 Pass `--write` to commit layout overrides into {}",
            manual_edits_path.display()
        );
    } else {
        println!("✅ {result_msg}");
        println!("💡 Run `fusible check` to verify design rules with the updated placement.");
    }
}
