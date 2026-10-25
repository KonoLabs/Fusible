//! Writes that a concurrent reader never sees half done.

use std::path::{Path, PathBuf};

/// The sibling of `path` that a file is written to before it is renamed over `path`.
pub fn staging_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

/// Writes `bytes` to `path` through a temp file in the same directory and a rename, so a
/// reader never sees a truncated file.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let staging = staging_path(path);
    std::fs::write(&staging, bytes)?;
    std::fs::rename(&staging, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&staging);
    })
}
