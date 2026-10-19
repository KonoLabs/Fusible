//! Configuration management for Fusible (`fusible.yml` / `fusible.yaml`).
//!
//! Provides types for project configuration, discovery via ancestor directory
//! traversal, parsing via `serde_yaml`, and starter templates for `fusible init`.

pub mod error;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use error::ConfigError;

/// Recognized filenames for the Fusible manifest, in order of precedence.
pub const CONFIG_FILENAMES: &[&str] = &["fusible.yml", "fusible.yaml"];

/// Root configuration schema for a Fusible project.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct FusibleConfig {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub hardware: Option<HardwareConfig>,
    #[serde(default)]
    pub mcu: Option<McuConfig>,
    #[serde(default)]
    pub firmware: Option<FirmwareConfig>,
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

impl FusibleConfig {
    /// Searches for `fusible.yml` or `fusible.yaml` starting at `start_dir` and
    /// walking up ancestor directories until found or reaching the filesystem root.
    pub fn find_and_load(start_dir: &Path) -> Result<Option<(PathBuf, Self)>, ConfigError> {
        if start_dir.is_file() {
            if let Some(found) = check_file_target(start_dir)? {
                return Ok(Some(found));
            }
        }

        let dir = if start_dir.is_file() {
            start_dir.parent().unwrap_or(start_dir)
        } else {
            start_dir
        };

        for ancestor in dir.ancestors() {
            if let Some(found) = find_in_dir(ancestor)? {
                return Ok(Some(found));
            }
        }

        Ok(None)
    }

    /// Reads and parses a configuration file from disk.
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|err| ConfigError::io(path, err))?;
        Self::from_str_named(&content, &path.display().to_string())
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
        r#"# Fusible project manifest
# Docs: https://github.com/KonoLabs/Fusible

name: sensor-node

hardware:
  # Top-level board entry point (.tsx or circuit.json)
  entry: hardware/index.tsx
  # Output directory for circuit artifacts
  out_dir: dist
  # Dev server port for interactive previewer
  port: 3020

mcu:
  # Target silicon identifier (e.g., stm32g431, esp32s3, rp2040)
  chip: stm32g431

firmware:
  # Directory containing firmware crate or code
  path: firmware
  # Firmware framework: embassy | cortex-m | esp-idf
  framework: embassy
  # Destination for auto-generated pin bindings
  pins_rs: firmware/src/pins.rs
  # Optional command to build firmware
  build_cmd: cargo build
"#
    }
}

/// Checks if a direct file path points to a known Fusible manifest.
fn check_file_target(path: &Path) -> Result<Option<(PathBuf, FusibleConfig)>, ConfigError> {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if CONFIG_FILENAMES.contains(&name) {
            let config = FusibleConfig::from_file(path)?;
            return Ok(Some((path.to_path_buf(), config)));
        }
    }
    Ok(None)
}

/// Inspects a directory for `fusible.yml` or `fusible.yaml`.
fn find_in_dir(dir: &Path) -> Result<Option<(PathBuf, FusibleConfig)>, ConfigError> {
    for filename in CONFIG_FILENAMES {
        let candidate = if dir.as_os_str().is_empty() {
            PathBuf::from(filename)
        } else {
            dir.join(filename)
        };
        if candidate.is_file() {
            let config = FusibleConfig::from_file(&candidate)?;
            return Ok(Some((candidate, config)));
        }
    }
    Ok(None)
}
