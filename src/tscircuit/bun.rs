//! Locates the bun executable for every place Fusible runs it.

use std::ffi::OsStr;
use std::path::PathBuf;

use super::detect::{find_binary_in_path, is_executable};

const ENV_VAR: &str = "FUSIBLE_BUN";

/// Finds bun: the file named by `FUSIBLE_BUN` when it exists, otherwise `bun` on PATH.
pub fn bun_path() -> Option<PathBuf> {
    resolve_bun(
        std::env::var_os(ENV_VAR).as_deref(),
        std::env::var_os("PATH").as_deref(),
    )
}

/// Picks bun from an explicit `FUSIBLE_BUN` value, then from `path_var`.
pub(crate) fn resolve_bun(env_bun: Option<&OsStr>, path_var: Option<&OsStr>) -> Option<PathBuf> {
    env_bun
        .map(PathBuf::from)
        .filter(|path| is_executable(path))
        .or_else(|| find_binary_in_path("bun", path_var?))
}
