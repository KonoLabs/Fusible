use std::path::{Path, PathBuf};

use crate::manifest;
use crate::model::{Project, Recent};

/// Canonical form of a path as a string, or the path as given when it does not exist.
pub fn canonical(path: &Path) -> String {
    let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let text = resolved.to_string_lossy().into_owned();
    text.strip_prefix(r"\\?\")
        .map(str::to_owned)
        .unwrap_or(text)
}

fn folder_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

/// Builds the card for one folder. An unreadable manifest gives zero counts.
pub fn describe(path: &Path, last_opened: Option<String>) -> Project {
    let summary = manifest::read(path).unwrap_or_default();
    Project {
        path: canonical(path),
        name: summary.name.unwrap_or_else(|| folder_name(path)),
        boards: summary.boards,
        firmware: summary.firmware,
        bodies: summary.bodies,
        last_opened,
        missing: !path.is_dir(),
    }
}

/// Recent projects first, then discovered folders, deduplicated by canonical path.
pub fn merge(recent: &[Recent], discovered: &[PathBuf]) -> Vec<Project> {
    let mut seen = std::collections::HashSet::new();
    let known = recent
        .iter()
        .map(|entry| (PathBuf::from(&entry.path), Some(entry.last_opened.clone())));
    let found = discovered.iter().map(|path| (path.clone(), None));
    known
        .chain(found)
        .map(|(path, opened)| describe(&path, opened))
        .filter(|project| seen.insert(project.path.clone()))
        .collect()
}
