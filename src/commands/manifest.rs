//! Values read from the project manifest with CLI overrides.

use std::path::{Path, PathBuf};

use crate::config::FusibleConfig;

/// Returns the explicit MCU, or the manifest `mcu.chip` when none is given.
pub fn resolve_mcu(explicit: Option<String>) -> Option<String> {
    explicit.or_else(|| {
        FusibleConfig::find_and_load(Path::new("."))
            .ok()
            .flatten()
            .and_then(|(_, cfg)| cfg.mcu.and_then(|m| m.chip))
    })
}

/// Returns the explicit pins output path, or the manifest `firmware.pins_rs`.
pub fn resolve_pins_rs(explicit: Option<PathBuf>) -> Option<PathBuf> {
    explicit.or_else(|| {
        FusibleConfig::find_and_load(Path::new("."))
            .ok()
            .flatten()
            .and_then(|(_, cfg)| cfg.firmware.and_then(|f| f.pins_rs))
    })
}

/// Returns the manifest `name`, or a generic board title when there is none.
pub fn resolve_board_name() -> String {
    FusibleConfig::find_and_load(Path::new("."))
        .ok()
        .flatten()
        .and_then(|(_, cfg)| cfg.name)
        .unwrap_or_else(|| "Hardware Board".to_string())
}
