//! Configuration management for Fusible (`fusible.yml` / `fusible.yaml`).
//!
//! Provides types for project configuration, discovery via ancestor directory
//! traversal, parsing via `serde_yaml`, and starter templates for `fusible init`.

mod discovery;
pub mod error;
mod template;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use discovery::{check_file_target, find_in_dir};
pub use error::ConfigError;

/// Recognized filenames for the Fusible manifest, in order of precedence.
pub const CONFIG_FILENAMES: &[&str] = &["fusible.yml", "fusible.yaml"];

/// Root configuration schema for a Fusible project.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct FusibleConfig {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub hardware: Option<HardwareConfig>,
    #[serde(default)]
    pub mcu: Option<McuConfig>,
    #[serde(default)]
    pub firmware: Option<FirmwareConfig>,
    #[serde(default)]
    pub mechanical: Option<MechanicalConfig>,
}

/// Hardware-specific settings including entry points and artifact output directory.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct HardwareConfig {
    #[serde(default)]
    pub entry: Option<PathBuf>,
    #[serde(default, alias = "out-dir")]
    pub out_dir: Option<PathBuf>,
    #[serde(default, alias = "circuit-json")]
    pub circuit_json: Option<PathBuf>,
    #[serde(default)]
    pub port: Option<u16>,
}

/// Target microcontroller configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct McuConfig {
    #[serde(default)]
    pub chip: Option<String>,
}

/// Firmware settings, framework selection, and generated code destinations.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct FirmwareConfig {
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub framework: Option<String>,
    #[serde(default, alias = "pins-rs")]
    pub pins_rs: Option<PathBuf>,
    #[serde(default, alias = "build-cmd")]
    pub build_cmd: Option<String>,
}

/// Mechanical enclosure generation settings for `fusible mechanical`.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct MechanicalConfig {
    /// Export format: `step`, `stl`, `glb`, `brep`, or `all`.
    #[serde(default)]
    pub format: Option<String>,
    /// Directory receiving the generated models.
    #[serde(default, alias = "out-dir")]
    pub out_dir: Option<PathBuf>,
    /// Wall, floor, and lid thickness in millimetres.
    #[serde(default, alias = "wall-thickness")]
    pub wall_thickness: Option<f64>,
    /// Total external height of the base in millimetres.
    #[serde(default)]
    pub height: Option<f64>,
    /// Cut wall notches for components that reach the board edge.
    #[serde(default)]
    pub cutouts: Option<bool>,
    /// Lid ventilation pattern: `none`, `grid`, or `slots`.
    #[serde(default)]
    pub ventilation: Option<String>,
    /// Add screw bosses under every `pcb_hole` of the board.
    #[serde(default)]
    pub mounting: Option<bool>,
}

impl FusibleConfig {
    /// Searches for `fusible.yml` or `fusible.yaml` starting at `start_dir` and
    /// walking up ancestor directories until found or reaching the filesystem root.
    pub fn find_and_load(start_dir: &Path) -> Result<Option<(PathBuf, Self)>, ConfigError> {
        if start_dir.is_file() {
            if let Some(found) = check_file_target(start_dir)? {
                return Ok(Some(found));
            }
        }

        let start_dir =
            std::path::absolute(start_dir).map_err(|err| ConfigError::io(start_dir, err))?;
        let dir = if start_dir.is_file() {
            start_dir.parent().unwrap_or(&start_dir)
        } else {
            &start_dir
        };

        for ancestor in dir.ancestors() {
            if let Some(found) = find_in_dir(ancestor)? {
                return Ok(Some(found));
            }
        }

        Ok(None)
    }

    fn rebased(mut self, dir: &Path) -> Self {
        let rebase = |p: &mut Option<PathBuf>| {
            if let Some(path) = p.as_mut().filter(|path| path.is_relative()) {
                *path = dir.join(&*path);
            }
        };
        if let Some(hw) = self.hardware.as_mut() {
            rebase(&mut hw.entry);
            rebase(&mut hw.out_dir);
            rebase(&mut hw.circuit_json);
        }
        if let Some(fw) = self.firmware.as_mut() {
            rebase(&mut fw.path);
            rebase(&mut fw.pins_rs);
        }
        if let Some(mech) = self.mechanical.as_mut() {
            rebase(&mut mech.out_dir);
        }
        self
    }

    /// Reads and parses a configuration file from disk, resolving relative paths against its directory.
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|err| ConfigError::io(path, err))?;
        let config = Self::from_str_named(&content, &path.display().to_string())?;
        Ok(config.rebased(path.parent().unwrap_or(Path::new(""))))
    }

    /// Parses a YAML configuration string using the default filename label `fusible.yml`.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(content: &str) -> Result<Self, ConfigError> {
        Self::from_str_named(content, "fusible.yml")
    }

    /// Parses a YAML configuration string with an explicit label for error diagnostics.
    pub fn from_str_named(content: &str, file_name: &str) -> Result<Self, ConfigError> {
        if content.trim().is_empty() {
            return Ok(Self::default());
        }
        let config: Option<Self> = serde_yaml::from_str(content)
            .map_err(|err| ConfigError::from_yaml(err, content, file_name))?;
        Ok(config.unwrap_or_default())
    }
}

impl std::str::FromStr for FusibleConfig {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_named(s, "fusible.yml")
    }
}

impl FusibleConfig {
    /// Returns an annotated, clean template string for `fusible init`.
    pub fn default_manifest() -> &'static str {
        template::DEFAULT_MANIFEST
    }
}
