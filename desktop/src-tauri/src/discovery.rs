use std::path::{Path, PathBuf};

use crate::manifest;

/// Lists subfolders of `dir` that hold a manifest, sorted by folder name.
pub fn find_projects(dir: &Path) -> Vec<PathBuf> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = read
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && manifest::find(path).is_some())
        .collect();
    found.sort();
    found
}
