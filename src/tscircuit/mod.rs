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

pub use command::{
    add_command, build_command, dev_command, export_command, install_command, search_command,
};
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
    /// NPM package executor (`npm exec -- tscircuit ...`).
    Npm(PathBuf),
}

impl RunnerKind {
    /// Human-friendly description of this runner backend.
    pub fn description(&self) -> String {
        match self {
            Self::Local(p) => format!("local node_modules ({})", p.display()),
            Self::Global(p) => format!("global tsci ({})", p.display()),
            Self::Bun(p) => format!("bun executor ({})", p.display()),
            Self::Npx(p) => format!("npx executor ({})", p.display()),
            Self::Npm(p) => format!("npm executor ({})", p.display()),
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
        let abs_output = output.map(|p| {
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                std::env::current_dir()
                    .map(|cd| cd.join(p))
                    .unwrap_or_else(|_| p.to_path_buf())
            }
        });
        if let Some(ref p) = abs_output {
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent).ok();
            }
        }

        let mut cmd = export_command(&self.kind, file, format, abs_output.as_deref());
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

    /// Adds packages to the tscircuit project.
    pub fn add(&self, packages: &[String], cwd: Option<&Path>) -> Result<(), RunnerError> {
        let mut cmd = command::add_command(&self.kind, packages);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd.status().map_err(|source| RunnerError::Io {
            command: command::format_command(&cmd),
            source,
        })?;

        if !status.success() {
            return Err(RunnerError::CommandFailed {
                command: command::format_command(&cmd),
                code: status.code(),
                stderr: "failed to add package(s)".into(),
                help: "check package names or registry connectivity".into(),
            });
        }
        Ok(())
    }

    /// Installs dependencies in the project.
    pub fn install(&self, cwd: Option<&Path>) -> Result<(), RunnerError> {
        let mut cmd = command::install_command(&self.kind);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd.status().map_err(|source| RunnerError::Io {
            command: command::format_command(&cmd),
            source,
        })?;

        if !status.success() {
            return Err(RunnerError::CommandFailed {
                command: command::format_command(&cmd),
                code: status.code(),
                stderr: "failed to install dependencies".into(),
                help: "check network connection or package.json validity".into(),
            });
        }
        Ok(())
    }

    /// Searches the tscircuit registry for components, footprints, and CAD models.
    pub fn search(&self, query: &[String]) -> Result<(), RunnerError> {
        let mut cmd = command::search_command(&self.kind, query);
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd.status().map_err(|source| RunnerError::Io {
            command: command::format_command(&cmd),
            source,
        })?;

        if !status.success() {
            return Err(RunnerError::CommandFailed {
                command: command::format_command(&cmd),
                code: status.code(),
                stderr: "search command failed".into(),
                help: "check network connection".into(),
            });
        }
        Ok(())
    }
}
