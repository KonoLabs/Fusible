//! Timestamped copies of a file about to be overwritten, rotated to the newest few.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// How many backups of one board `fusible` keeps.
pub const BACKUP_KEEP: usize = 10;

/// Copies `path` to `dir/<name>-<unix ms>.<ext>` and deletes the oldest copies beyond
/// [`BACKUP_KEEP`]. Returns `None` when `path` does not exist yet.
pub fn backup_file(path: &Path, dir: &Path, name: &str) -> Result<Option<PathBuf>, String> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    backup_file_at(path, dir, name, millis, BACKUP_KEEP)
}

/// [`backup_file`] with an explicit timestamp and retention.
pub(super) fn backup_file_at(
    path: &Path,
    dir: &Path,
    name: &str,
    stamp: u128,
    keep: usize,
) -> Result<Option<PathBuf>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("bak");
    let copy = dir.join(format!("{name}-{stamp:020}.{extension}"));
    std::fs::copy(path, &copy).map_err(|e| format!("cannot back up {}: {e}", path.display()))?;
    prune(dir, name, keep);
    Ok(Some(copy))
}

/// Removes the oldest backups of `name` until `keep` remain.
fn prune(dir: &Path, name: &str, keep: usize) {
    let prefix = format!("{name}-");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut copies: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&prefix))
        })
        .collect();
    copies.sort();
    let surplus = copies.len().saturating_sub(keep);
    for old in &copies[..surplus] {
        let _ = std::fs::remove_file(old);
    }
}
