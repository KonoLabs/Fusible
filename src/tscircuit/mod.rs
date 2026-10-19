//! Step 3 — tscircuit runner and CLI orchestration.
//!
//! Provides execution support for compiling, exporting, and live-previewing
//! circuits via tscircuit. Supports local node_modules installations,
//! global binaries, and on-demand runners via `bun` and `npx`.

pub mod command;
pub mod detect;
pub mod error;
pub mod output;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};

pub use command::{build_command, dev_command, export_command};
pub use detect::detect_runner;
pub use error::RunnerError;

/// Represents the detected or configured tscircuit execution backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerKind {
    /// Local node_modules executable (`./node_modules/.bin/tsci`).
    Local(PathBuf),
    /// Globally installed executable on PATH (`tsci`).
    Global(PathBuf),
    /// Bun package executor (`bun x tscircuit ...`).
    Bun(PathBuf),
    /// NPX package executor (`npx tscircuit ...`).
    Npx(PathBuf),
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

    /// Executes `dev [file] -p [port]`, inheriting standard I/O for interactive hot-reloading.
    pub fn dev(&self, file: &Path, port: Option<u16>) -> Result<ExitStatus, RunnerError> {
        let mut cmd = dev_command(&self.kind, file, port);
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        cmd.status().map_err(|source| RunnerError::Io {
            command: command::format_command(&cmd),
            source,
        })
    }

    /// Runs the tscircuit export command.
    pub fn export(
        &self,
        file: &Path,
        format: &str,
        output: Option<&Path>,
    ) -> Result<(), RunnerError> {
        let mut cmd = export_command(&self.kind, file, format, output);
        let out = cmd.output().map_err(|source| RunnerError::Io {
            command: command::format_command(&cmd),
            source,
        })?;

        if !out.status.success() {
            let details = command::extract_output_details(&out);
            return Err(RunnerError::CommandFailed {
                command: command::format_command(&cmd),
                code: out.status.code(),
                stderr: details.clone(),
                help: details,
            });
        }
        Ok(())
    }

    /// Runs the tscircuit build command to produce circuit json.
    pub fn build(&self, file: &Path, out_file: Option<&Path>) -> Result<PathBuf, RunnerError> {
        let mut cmd = build_command(&self.kind, file);
        let out = cmd.output().map_err(|source| RunnerError::Io {
            command: command::format_command(&cmd),
            source,
        })?;

        if !out.status.success() {
            let details = command::extract_output_details(&out);
            return Err(RunnerError::CommandFailed {
                command: command::format_command(&cmd),
                code: out.status.code(),
                stderr: details.clone(),
                help: details,
            });
        }

        let stdout = String::from_utf8_lossy(&out.stdout);
        let produced = output::resolve_circuit_json(file, &stdout)?;

        if let Some(dest) = out_file {
            output::copy_output(&produced, dest)?;
            Ok(dest.to_path_buf())
        } else {
            Ok(produced)
        }
    }
}
