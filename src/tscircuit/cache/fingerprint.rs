//! Content fingerprint of a hardware workspace: any input change changes the key.

use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use super::hasher::Hasher;

const SKIPPED_DIRS: [&str; 3] = ["node_modules", "dist", "target"];
const SOURCE_EXTENSIONS: [&str; 7] = ["ts", "tsx", "js", "jsx", "mjs", "cjs", "json"];
const TOOL_MANIFESTS: [&str; 2] = [
    "node_modules/tscircuit/package.json",
    "node_modules/@tscircuit/core/package.json",
];
const LEAVES_WORKSPACE: [&[u8]; 3] = [b"\"../", b"'../", b"`../"];

fn is_source(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| SOURCE_EXTENSIONS.contains(&ext))
}

fn escapes_workspace(path: &Path, bytes: &[u8]) -> bool {
    let code = path.extension().is_some_and(|ext| ext != "json");
    code && LEAVES_WORKSPACE
        .iter()
        .any(|needle| bytes.windows(needle.len()).any(|w| w == *needle))
}

fn hash_file(path: &Path, rel: &str, hasher: &mut Hasher) -> Option<()> {
    hasher.write(rel.as_bytes());
    if is_source(path) {
        let bytes = fs::read(path).ok()?;
        if escapes_workspace(path, &bytes) {
            return None;
        }
        hasher.write(&bytes);
    } else {
        let meta = fs::metadata(path).ok()?;
        let nanos = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
        hasher.write(&meta.len().to_le_bytes());
        hasher.write(&nanos.as_nanos().to_le_bytes());
    }
    Some(())
}

fn skipped(name: &str) -> bool {
    name.starts_with('.') || SKIPPED_DIRS.contains(&name)
}

fn walk(dir: &Path, root: &Path, hasher: &mut Hasher) -> Option<()> {
    let mut entries: Vec<_> = fs::read_dir(dir).ok()?.flatten().collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        if skipped(&name) {
            continue;
        }
        let path = entry.path();
        if entry.file_type().ok()?.is_dir() {
            walk(&path, root, hasher)?;
        } else {
            let rel = path.strip_prefix(root).unwrap_or(&path).to_string_lossy();
            hash_file(&path, &rel, hasher)?;
        }
    }
    Some(())
}

/// Hashes `salt`, every tracked file under `root`, and the tscircuit manifests.
///
/// Returns `None` when an input cannot be read or an import leaves the workspace, so the
/// caller must build instead of trusting a stale entry.
pub fn fingerprint(root: &Path, salt: &str) -> Option<String> {
    let mut hasher = Hasher::new();
    hasher.write(salt.as_bytes());
    walk(root, root, &mut hasher)?;
    for manifest in TOOL_MANIFESTS {
        if let Ok(bytes) = fs::read(root.join(manifest)) {
            hasher.write(&bytes);
        }
    }
    Some(hasher.finish())
}
