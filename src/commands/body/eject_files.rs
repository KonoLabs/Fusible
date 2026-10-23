//! Crash-safe file steps of `fusible body eject`: stage, publish, and replace.

use std::io;
use std::path::{Path, PathBuf};

use super::BodyError;

/// The staging file next to `path`: same directory, so the final step is a rename.
pub(super) fn staging_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

/// Moves the staged model to `path` without ever replacing a file that is already there.
///
/// A hard link fails when `path` exists, even if it appeared after the first check. Filesystems
/// without hard links fall back to a second check and a rename.
pub(super) fn publish_new(staged: &Path, path: &Path) -> Result<(), BodyError> {
    let result = match std::fs::hard_link(staged, path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Err(exists(path)),
        Err(_) if path.exists() => Err(exists(path)),
        Err(_) => std::fs::rename(staged, path).map_err(BodyError::from),
    };
    let _ = std::fs::remove_file(staged);
    result
}

fn exists(path: &Path) -> BodyError {
    BodyError::Exists {
        path: path.to_path_buf(),
    }
}

/// Replaces `path` with `contents` in one step: a crash leaves the old file or the new one.
pub(super) fn write_atomic(path: &Path, contents: &str) -> io::Result<()> {
    let staged = staging_path(path);
    std::fs::write(&staged, contents)?;
    let finish = || {
        if let Ok(meta) = std::fs::metadata(path) {
            std::fs::set_permissions(&staged, meta.permissions())?;
        }
        std::fs::rename(&staged, path)
    };
    finish().inspect_err(|_| {
        let _ = std::fs::remove_file(&staged);
    })
}
