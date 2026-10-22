//! Resolution of mechanical settings from CLI overrides, the manifest and defaults.

use std::path::PathBuf;

use super::{MechanicalError, DEFAULT_WALL, FORMATS};
use crate::config::MechanicalConfig;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Ventilation {
    None,
    Grid,
    Slots,
}

#[derive(Debug)]
pub(super) struct Settings {
    pub(super) formats: Vec<&'static str>,
    pub(super) out_dir: PathBuf,
    pub(super) wall: f64,
    pub(super) height: Option<f64>,
    pub(super) cutouts: bool,
    pub(super) ventilation: Ventilation,
    pub(super) mounting: bool,
}

/// Merges `overrides` over `manifest` over built-in defaults and validates the result.
pub(super) fn resolve_settings(
    overrides: MechanicalConfig,
    manifest: MechanicalConfig,
) -> Result<Settings, MechanicalError> {
    let format = overrides.format.or(manifest.format);
    let pattern = overrides.ventilation.or(manifest.ventilation);
    Ok(Settings {
        formats: formats(format.unwrap_or_else(|| "step".to_string()))?,
        ventilation: ventilation(pattern.unwrap_or_else(|| "none".to_string()))?,
        wall: wall(overrides.wall_thickness.or(manifest.wall_thickness))?,
        out_dir: overrides
            .out_dir
            .or(manifest.out_dir)
            .unwrap_or_else(|| PathBuf::from("dist/mechanical")),
        height: overrides.height.or(manifest.height),
        cutouts: overrides.cutouts.or(manifest.cutouts).unwrap_or(true),
        mounting: overrides.mounting.or(manifest.mounting).unwrap_or(true),
    })
}

/// Resolves preview settings: manifest values apply, but the format is always GLB.
pub(super) fn preview_settings(manifest: MechanicalConfig) -> Result<Settings, MechanicalError> {
    let forced = MechanicalConfig {
        format: Some("glb".to_string()),
        ..Default::default()
    };
    resolve_settings(
        forced,
        MechanicalConfig {
            format: None,
            out_dir: None,
            ..manifest
        },
    )
}

fn formats(format: String) -> Result<Vec<&'static str>, MechanicalError> {
    if format == "all" {
        return Ok(FORMATS.to_vec());
    }
    FORMATS
        .iter()
        .find(|&&f| f == format)
        .map(|&f| vec![f])
        .ok_or(MechanicalError::UnknownFormat { format })
}

fn ventilation(pattern: String) -> Result<Ventilation, MechanicalError> {
    match pattern.as_str() {
        "none" => Ok(Ventilation::None),
        "grid" => Ok(Ventilation::Grid),
        "slots" => Ok(Ventilation::Slots),
        _ => Err(MechanicalError::UnknownVentilation { pattern }),
    }
}

fn wall(value: Option<f64>) -> Result<f64, MechanicalError> {
    let wall = value.unwrap_or(DEFAULT_WALL);
    if wall.is_finite() && wall > 0.0 {
        return Ok(wall);
    }
    Err(MechanicalError::InvalidDimension {
        name: "wall thickness",
        value: wall,
        help: "use a positive thickness such as 2.0".into(),
    })
}
