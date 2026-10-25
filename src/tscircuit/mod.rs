//! Step 3 — tscircuit runner and CLI orchestration.
//!
//! Provides execution support for compiling, exporting, and live-previewing
//! circuits via tscircuit. Supports local node_modules installations,
//! global binaries, and on-demand runners via `bun` and `npx`.

mod build;
mod bun;
mod cache;
pub mod command;
pub mod detect;
pub mod error;
mod export;
mod interactive;
pub mod kicad;
pub mod output;
mod process;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

pub use bun::bun_path;
pub use command::{
    add_command, build_command, dev_command, export_command, install_command, search_command,
};
pub use detect::detect_runner;
pub use error::RunnerError;
pub use kicad::convert_kicad;

/// Represents the detected or configured tscircuit execution backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerKind {
    /// Local node_modules executable (`./node_modules/.bin/tsci` or `tscircuit`).
    Local(PathBuf),
    /// Globally installed executable on PATH (`tsci` or `tscircuit`).
    Global(PathBuf),
    /// `bun x --bun tsci`, used when no `tsci` is installed but bun is.
    Bun(PathBuf),
}

impl RunnerKind {
    /// Human-friendly description of this runner backend.
    pub fn description(&self) -> String {
        match self {
            Self::Local(p) => format!("local executable ({})", p.display()),
            Self::Global(p) => format!("global executable ({})", p.display()),
            Self::Bun(p) => format!("bun x tsci via {}", p.display()),
        }
    }
}

/// Orchestrates tscircuit CLI commands (`dev`, `export`, `build`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Runner {
    kind: RunnerKind,
}

/// Auto-detects an available tscircuit runner using current directory and PATH.
pub fn detect() -> Result<Runner, RunnerError> {
    Runner::detect()
}

impl Runner {
    /// Creates a runner with a specific execution backend.
    pub fn new(kind: RunnerKind) -> Self {
        Self { kind }
    }

    /// Auto-detects an available runner in the current directory and environment.
    pub fn detect() -> Result<Self, RunnerError> {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let path_var = std::env::var_os("PATH");
        detect_runner(&current_dir, path_var.as_deref())
    }

    /// Returns the runner backend kind.
    pub fn kind(&self) -> &RunnerKind {
        &self.kind
    }
}

/// Splits `file` into the hardware workspace to run in and the path relative to it.
fn resolve_hardware_context(file: &Path) -> (Option<PathBuf>, PathBuf) {
    if let Some(parent) = file.parent() {
        if !parent.as_os_str().is_empty() && parent.join("package.json").is_file() {
            let filename = file
                .file_name()
                .map(PathBuf::from)
                .unwrap_or_else(|| file.to_path_buf());
            return (Some(parent.to_path_buf()), filename);
        }
    }
    if Path::new("hardware").join("package.json").is_file() {
        if let Ok(rel) = file.strip_prefix("hardware") {
            return (Some(PathBuf::from("hardware")), rel.to_path_buf());
        }
    }
    (None, file.to_path_buf())
}
