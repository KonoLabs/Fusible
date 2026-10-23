//! Circuit source kinds and the conventional entry file locations.

use std::path::{Path, PathBuf};

pub(super) fn entry_from_conventions() -> Option<PathBuf> {
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
