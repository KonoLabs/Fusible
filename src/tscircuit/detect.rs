//! Detection of available tscircuit execution backends.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::tscircuit::error::RunnerError;
use crate::tscircuit::{Runner, RunnerKind};

/// Auto-detects how to execute tscircuit based on project directory and environment PATH.
///
/// Order of precedence:
/// 1. Local `./node_modules/.bin/tsci`
/// 2. Global `tsci` on PATH
/// 3. `bun` on PATH (`bun x tscircuit ...`)
/// 4. `npx` on PATH (`npx tscircuit ...`)
pub fn detect_runner(current_dir: &Path, path_var: Option<&OsStr>) -> Result<Runner, RunnerError> {
    if let Some(local_path) = find_local_tsci(current_dir) {
        return Ok(Runner::new(RunnerKind::Local(local_path)));
    }

    if let Some(path_var) = path_var {
        if let Some(global_path) = find_binary_in_path("tsci", path_var) {
            return Ok(Runner::new(RunnerKind::Global(global_path)));
        }

        if let Some(bun_path) = find_binary_in_path("bun", path_var) {
            return Ok(Runner::new(RunnerKind::Bun(bun_path)));
        }

        if let Some(npx_path) = find_binary_in_path("npx", path_var) {
            return Ok(Runner::new(RunnerKind::Npx(npx_path)));
        }
    }

    Err(RunnerError::NotFound)
}

/// Checks for `./node_modules/.bin/tsci` relative to the given directory.
fn find_local_tsci(dir: &Path) -> Option<PathBuf> {
    let bin_dir = dir.join("node_modules").join(".bin");
    let candidates = [
        bin_dir.join("tsci"),
        bin_dir.join("tsci.cmd"),
        bin_dir.join("tsci.exe"),
    ];

    candidates.into_iter().find(|c| is_executable(c))
}

/// Finds an executable binary with the given name within PATH.
pub(crate) fn find_binary_in_path(binary: &str, path_var: &OsStr) -> Option<PathBuf> {
    std::env::split_paths(path_var).find_map(|dir| {
        let candidate = dir.join(binary);
        if is_executable(&candidate) {
            return Some(candidate);
        }
        find_windows_binary(&dir, binary)
    })
}

/// Looks for Windows-specific binary extensions (.cmd, .exe, .bat).
fn find_windows_binary(dir: &Path, binary: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        for ext in &[".cmd", ".exe", ".bat"] {
            let candidate_ext = dir.join(format!("{binary}{ext}"));
            if is_executable(&candidate_ext) {
                return Some(candidate_ext);
            }
        }
    }
    #[cfg(not(windows))]
    let _ = (dir, binary);
    None
}

/// Verifies whether a given path is an executable file.
#[cfg(unix)]
pub(crate) fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if !path.is_file() {
        return false;
    }
    path.metadata()
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// Verifies whether a given path is an executable file on non-Unix systems.
#[cfg(not(unix))]
pub(crate) fn is_executable(path: &Path) -> bool {
    path.is_file()
}
