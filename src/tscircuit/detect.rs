//! Detection of available tscircuit execution backends.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::tscircuit::bun::resolve_bun;
use crate::tscircuit::error::RunnerError;
use crate::tscircuit::{Runner, RunnerKind};

/// Auto-detects how to execute tscircuit based on project directory and environment PATH.
///
/// Order of precedence:
/// 1. Local `./node_modules/.bin/tsci` or `./node_modules/.bin/tscircuit`
/// 2. Local `./hardware/node_modules/.bin/tsci` or `./hardware/node_modules/.bin/tscircuit`
/// 3. Global `tsci` on PATH
/// 4. Global `tscircuit` on PATH
/// 5. `bun x --bun --package tscircuit tsci`, with bun from `FUSIBLE_BUN` or PATH
pub fn detect_runner(current_dir: &Path, path_var: Option<&OsStr>) -> Result<Runner, RunnerError> {
    if let Some(local_path) = find_local_tsci(current_dir) {
        return Ok(Runner::new(RunnerKind::Local(local_path)));
    }

    if let Some(path_var) = path_var {
        if let Some(global_path) = find_binary_in_path("tsci", path_var)
            .or_else(|| find_binary_in_path("tscircuit", path_var))
        {
            return Ok(Runner::new(RunnerKind::Global(global_path)));
        }
    }

    let env_bun = std::env::var_os("FUSIBLE_BUN");
    match resolve_bun(env_bun.as_deref(), path_var) {
        Some(bun) => Ok(Runner::new(RunnerKind::Bun(bun))),
        None => Err(RunnerError::NotFound),
    }
}

/// Checks for local tsci/tscircuit relative to the given directory or hardware subdirectory.
fn find_local_tsci(dir: &Path) -> Option<PathBuf> {
    let dirs = [
        dir.join("node_modules").join(".bin"),
        dir.join("hardware").join("node_modules").join(".bin"),
    ];
    for bin_dir in dirs {
        let candidates = [
            bin_dir.join("tsci"),
            bin_dir.join("tsci.cmd"),
            bin_dir.join("tsci.exe"),
            bin_dir.join("tscircuit"),
            bin_dir.join("tscircuit.cmd"),
            bin_dir.join("tscircuit.exe"),
        ];

        if let Some(found) = candidates.into_iter().find(|c| is_executable(c)) {
            return Some(found);
        }
    }
    None
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

/// Verifies whether a given path is a launchable file on non-Unix systems.
///
/// Extensionless files such as the shell shims in `node_modules/.bin` cannot
/// be started by `CreateProcess`, so only launchable extensions count.
#[cfg(not(unix))]
pub(crate) fn is_executable(path: &Path) -> bool {
    path.is_file()
        && path.extension().is_some_and(|ext| {
            ["exe", "cmd", "bat", "com"]
                .iter()
                .any(|known| ext.eq_ignore_ascii_case(known))
        })
}

/// Resolves `name` against PATH so Windows finds `.cmd` shims such as `npm`.
///
/// Falls back to the bare name, which lets the spawn report the failure.
pub(crate) fn resolve_program(name: &str) -> PathBuf {
    std::env::var_os("PATH")
        .and_then(|path| find_binary_in_path(name, &path))
        .unwrap_or_else(|| PathBuf::from(name))
}
