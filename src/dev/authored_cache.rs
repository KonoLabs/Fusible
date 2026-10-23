//! The GLB cache of converted authored bodies, keyed by path, modification time, and size.

use std::collections::hash_map::DefaultHasher;
use std::fs::Metadata;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::UNIX_EPOCH;

use super::authored::{Failure, TIMEOUT_MARK};
use super::limiter::{conversions, key_lock, lock};

/// Hashes the path alone, then the path with its modification time and size.
pub(super) fn cache_key(path: &Path, meta: &Metadata) -> String {
    let nanos = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos());
    let mut by_path = DefaultHasher::new();
    path.hash(&mut by_path);
    let mut by_state = DefaultHasher::new();
    (path, nanos, meta.len()).hash(&mut by_state);
    format!("{:016x}-{:016x}", by_path.finish(), by_state.finish())
}

/// Removes the cached GLBs of older states of the same file, never in-flight temporary files.
fn prune_stale(cache_dir: &Path, key: &str) {
    let prefix = key.split('-').next().unwrap_or(key);
    let entries = std::fs::read_dir(cache_dir).into_iter().flatten().flatten();
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".glb") && name.starts_with(prefix) && !name.starts_with(key) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

fn partial_name(key: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{key}.{}.{id}.part", std::process::id())
}

/// Returns the cached GLB for `key`, converting `source` once even when requests overlap.
///
/// Requests for the same key wait for the first conversion and reuse its result. At most
/// [`conversions`] different conversions run at once.
pub(super) fn cached_conversion(
    source: &Path,
    format: &str,
    key: &str,
    cache_dir: &Path,
    convert: &impl Fn(&Path, &str, &Path) -> Result<(), String>,
) -> Result<Vec<u8>, Failure> {
    let cached = cache_dir.join(format!("{key}.glb"));
    if let Ok(bytes) = std::fs::read(&cached) {
        return Ok(bytes);
    }
    let same_key = key_lock(key);
    let _turn = lock(&same_key);
    if let Ok(bytes) = std::fs::read(&cached) {
        return Ok(bytes);
    }
    let io = |e: std::io::Error| Failure::new(500, format!("cannot use the body cache: {e}"));
    std::fs::create_dir_all(cache_dir).map_err(io)?;
    let partial = cache_dir.join(partial_name(key));
    let _slot = conversions().acquire();
    if let Err(message) = convert(source, format, &partial) {
        let _ = std::fs::remove_file(&partial);
        let status = if message.contains(TIMEOUT_MARK) {
            504
        } else {
            422
        };
        return Err(Failure::new(
            status,
            format!("conversion failed: {message}"),
        ));
    }
    std::fs::rename(&partial, &cached).map_err(io)?;
    prune_stale(cache_dir, key);
    std::fs::read(&cached).map_err(io)
}
