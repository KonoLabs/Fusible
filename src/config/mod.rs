//! Configuration management for Fusible (`fusible.yml` / `fusible.yaml`).
//!
//! Provides types for project configuration, discovery via ancestor directory
//! traversal, parsing via `serde_yaml`, and starter templates for `fusible init`.

mod discovery;
pub mod error;
mod query;
mod raw;
#[cfg(test)]
mod sections_tests;
mod template;
#[cfg(test)]
mod tests;
mod types;
mod validate;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use discovery::{check_file_target, find_in_dir};
pub use error::ConfigError;
use raw::RawConfig;
pub use raw::LEGACY_BOARD;
pub use types::{BoardConfig, BodyConfig, EnclosureConfig, FirmwareConfig};
pub use validate::GENERATORS;

/// Recognized filenames for the Fusible manifest, in order of precedence.
pub const CONFIG_FILENAMES: &[&str] = &["fusible.yml", "fusible.yaml"];

/// Root configuration: named boards, firmware, and bodies of one project.
///
/// Names are unique across the three sections, so any command can take one as its target.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FusibleConfig {
    pub name: Option<String>,
    pub boards: BTreeMap<String, BoardConfig>,
    pub firmware: BTreeMap<String, FirmwareConfig>,
    pub bodies: BTreeMap<String, BodyConfig>,
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
        for board in self.boards.values_mut() {
            rebase(&mut board.entry);
            rebase(&mut board.out_dir);
            rebase(&mut board.circuit_json);
        }
        for fw in self.firmware.values_mut() {
            rebase(&mut fw.path);
            rebase(&mut fw.pins_rs);
        }
        for body in self.bodies.values_mut() {
            rebase(&mut body.source);
            rebase(&mut body.enclosure.out_dir);
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
        let raw: Option<RawConfig> = serde_yaml::from_str(content)
            .map_err(|err| ConfigError::from_yaml(err, content, file_name))?;
        let config = raw
            .unwrap_or_default()
            .normalize()
            .map_err(ConfigError::Invalid)?;
        validate::validate(&config).map_err(ConfigError::Invalid)?;
        Ok(config)
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
