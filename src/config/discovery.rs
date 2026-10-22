//! Manifest lookup in a file path or directory.

use std::path::{Path, PathBuf};

use super::{ConfigError, FusibleConfig, CONFIG_FILENAMES};

/// Checks if a direct file path points to a known Fusible manifest.
pub(super) fn check_file_target(
    path: &Path,
) -> Result<Option<(PathBuf, FusibleConfig)>, ConfigError> {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if CONFIG_FILENAMES.contains(&name) {
            let config = FusibleConfig::from_file(path)?;
            return Ok(Some((path.to_path_buf(), config)));
        }
    }
    Ok(None)
}

/// Inspects a directory for `fusible.yml` or `fusible.yaml`.
pub(super) fn find_in_dir(dir: &Path) -> Result<Option<(PathBuf, FusibleConfig)>, ConfigError> {
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
