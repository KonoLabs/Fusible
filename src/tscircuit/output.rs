//! Output resolution and circuit json artifact handling for tscircuit builds.

use std::path::{Path, PathBuf};

use crate::tscircuit::error::RunnerError;

/// Resolves the path to the generated `circuit.json` on disk.
pub fn resolve_circuit_json(file: &Path, stdout: &str) -> Result<PathBuf, RunnerError> {
    let parent = file.parent().unwrap_or_else(|| Path::new(""));
    let stem = file
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();

    if let Some(path) = extract_circuit_json_path(stdout) {
        if path.exists() {
            return Ok(path);
        }
        let joined = parent.join(&path);
        if joined.exists() {
            return Ok(joined);
        }
    }

    let candidates = [
        parent.join("dist").join(stem.as_ref()).join("circuit.json"),
        parent.join("dist").join("circuit.json"),
        Path::new("dist")
            .join(file.with_extension(""))
            .join("circuit.json"),
        Path::new("dist").join(stem.as_ref()).join("circuit.json"),
        Path::new("dist").join("circuit.json"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    let fallback = candidates[0].clone();
    Err(RunnerError::OutputNotFound { path: fallback })
}

/// Copies the produced circuit json to the target destination path.
pub fn copy_output(src: &Path, dest: &Path) -> Result<(), RunnerError> {
    if let Some(parent) = dest.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|source| RunnerError::Io {
                command: format!("create_dir_all {}", parent.display()),
                source,
            })?;
        }
    }
    std::fs::copy(src, dest).map_err(|source| RunnerError::Io {
        command: format!("copy {} -> {}", src.display(), dest.display()),
        source,
    })?;
    Ok(())
}

/// Parses the output path from tscircuit's stdout log message.
pub fn extract_circuit_json_path(stdout: &str) -> Option<PathBuf> {
    const PREFIX: &str = "Circuit JSON written to ";
    for line in stdout.lines() {
        if let Some(pos) = line.find(PREFIX) {
            let path_part = line[pos + PREFIX.len()..].trim();
            if !path_part.is_empty() {
                return Some(PathBuf::from(path_part));
            }
        }
    }
    None
}

/// Returns `produced`, or a copy of it at `out_file` when one is requested.
pub fn deliver(produced: PathBuf, out_file: Option<&Path>) -> Result<PathBuf, RunnerError> {
    match out_file {
        Some(dest) => {
            copy_output(&produced, dest)?;
            Ok(dest.to_path_buf())
        }
        None => Ok(produced),
    }
}
