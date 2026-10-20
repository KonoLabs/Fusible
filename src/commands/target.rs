//! Shared helper for resolving circuit target files from CLI or manifest.

use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::config::FusibleConfig;

/// Error when no valid input file or entry point could be resolved.
#[derive(Debug, Error, Diagnostic)]
#[error("no circuit file specified and no fusible.yml found")]
#[diagnostic(help("specify a file via arguments or run `fusible init` to create a manifest"))]
pub struct MissingTargetError;

/// Resolves the circuit entry file from CLI argument, `fusible.yml`, or convention.
pub fn resolve_entry(explicit: Option<PathBuf>) -> Result<PathBuf, MissingTargetError> {
    if let Some(path) = explicit {
        return Ok(path);
    }
    if let Some(path) = entry_from_config() {
        return Ok(path);
    }
    entry_from_conventions().ok_or(MissingTargetError)
}

fn entry_from_config() -> Option<PathBuf> {
    let (_, cfg) = FusibleConfig::find_and_load(Path::new(".")).ok()??;
    let hw = cfg.hardware?;
    hw.entry.or(hw.circuit_json)
}

fn entry_from_conventions() -> Option<PathBuf> {
    const CANDIDATES: &[&str] = &[
        "hardware/index.tsx",
        "hardware/index.circuit.tsx",
        "hardware/camera.tsx",
        "src/index.tsx",
        "index.tsx",
        "hardware/camera.kicad_sch",
        "hardware/camera.kicad_pro",
        "dist/circuit.json",
        "circuit.json",
    ];
    CANDIDATES.iter().map(PathBuf::from).find(|p| p.exists())
}

/// Checks if a path points to a TypeScript/TSX circuit source file.
pub fn is_tsx_target(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("tsx") | Some("ts") | Some("jsx") | Some("js")
    )
}

/// Checks if a path points to KiCad schematics, PCB, or project directory.
pub fn is_kicad_target(path: &Path) -> bool {
    if path.is_file() {
        matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("kicad_sch") | Some("kicad_pcb") | Some("kicad_pro")
        )
    } else if path.is_dir() {
        // Directory containing KiCad files
        std::fs::read_dir(path)
            .ok()
            .map(|entries| {
                entries.filter_map(Result::ok).any(|e| {
                    let p = e.path();
                    matches!(
                        p.extension().and_then(|ext| ext.to_str()),
                        Some("kicad_sch") | Some("kicad_pcb") | Some("kicad_pro")
                    )
                })
            })
            .unwrap_or(false)
    } else {
        false
    }
}
