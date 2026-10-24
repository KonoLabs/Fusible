//! Writing KiCad placements into the literal `pcbX`, `pcbY` and `pcbRotation` props of a TSX tree.

use std::path::{Path, PathBuf};

use super::backup::backup_file;
use super::diff::{DiffKind, PlacementDiff};
use super::placements::rotation_delta;
use super::tsx_tag::{find_tag, with_number};

const SKIPPED_DIRS: [&str; 3] = ["node_modules", "dist", "target"];

/// Rewrites the props of the tags of the moved parts in every TSX file under `dir`.
///
/// Each file is copied to `backup_dir` first. A part is rewritten only when every prop it
/// needs is a plain number in a tag that is the only one with its name; the others are left
/// for the studio to report. Returns the designators written.
pub(super) fn rewrite_tree(
    dir: &Path,
    diffs: &[&PlacementDiff],
    backup_dir: &Path,
    name: &str,
) -> Result<Vec<String>, String> {
    let mut pending: Vec<&PlacementDiff> = diffs.to_vec();
    let mut done = Vec::new();
    for file in tsx_files(dir) {
        if pending.is_empty() {
            break;
        }
        let source = std::fs::read_to_string(&file)
            .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
        let (text, written) = rewrite_source(&source, &pending);
        if written.is_empty() {
            continue;
        }
        let stem = file
            .file_stem()
            .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
        backup_file(&file, backup_dir, &format!("{name}-{stem}"))?;
        std::fs::write(&file, text).map_err(|e| format!("cannot write {}: {e}", file.display()))?;
        pending.retain(|d| !written.contains(&d.designator));
        done.extend(written);
    }
    Ok(done)
}

/// Rewrites the tags of `diffs` in `source`; returns the new text and the designators changed.
pub(super) fn rewrite_source(source: &str, diffs: &[&PlacementDiff]) -> (String, Vec<String>) {
    let mut text = source.to_string();
    let mut written = Vec::new();
    for diff in diffs {
        let Some(range) = find_tag(&text, &diff.designator) else {
            continue;
        };
        if let Some(tag) = rewritten_tag(&text[range.clone()], diff) {
            text.replace_range(range, &tag);
            written.push(diff.designator.clone());
        }
    }
    (text, written)
}

/// Returns `tag` with the position and rotation of `diff` applied, or `None` when a prop it
/// needs is missing or not a plain number.
fn rewritten_tag(tag: &str, diff: &PlacementDiff) -> Option<String> {
    let [x, y] = diff.kicad_pos?;
    let mut tag = tag.to_string();
    match diff.kind {
        DiffKind::Moved { .. } => {
            tag = with_number(&tag, "pcbX", x)?;
            tag = with_number(&tag, "pcbY", y)?;
        }
        DiffKind::Rotated { .. } => {}
        _ => return None,
    }
    if rotation_delta(diff.kicad_rot, diff.current_rot).abs() > 0.5 {
        tag = with_number(&tag, "pcbRotation", diff.kicad_rot.rem_euclid(360.0))?;
    }
    Some(tag)
}

/// Lists the `.tsx` files under `dir`, skipping dependency, build and hidden folders.
fn tsx_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files = Vec::new();
    for path in entries.flatten().map(|entry| entry.path()) {
        let name = path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        if path.is_dir() && !name.starts_with('.') && !SKIPPED_DIRS.contains(&name.as_str()) {
            files.extend(tsx_files(&path));
        } else if path.extension().is_some_and(|e| e == "tsx") {
            files.push(path);
        }
    }
    files.sort();
    files
}
