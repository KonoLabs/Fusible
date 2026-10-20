use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::check::{load_circuit_source, CheckError};
use crate::commands::target::{resolve_entry, MissingTargetError};
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
    let (_json_path, source) = load_circuit_source(&entry)?;

    let report = compare_kicad_with_circuit(&footprints, &source);

    let manual_edits_path = target.unwrap_or_else(|| {
        if Path::new("hardware/manual-edits.json").is_file()
            || Path::new("hardware/package.json").is_file()
        {
            PathBuf::from("hardware/manual-edits.json")
        } else if let Some(parent) = entry.parent() {
            if !parent.as_os_str().is_empty() {
                parent.join("manual-edits.json")
            } else {
                PathBuf::from("manual-edits.json")
            }
        } else {
            PathBuf::from("manual-edits.json")
        }
    });

    let is_dry = dry_run || !write;
    let result_msg =
        apply_kicad_sync(&report, &manual_edits_path, is_dry).map_err(SyncCommandError::Sync)?;

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

    Ok(())
}
