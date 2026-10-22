//! Step 3 — tscircuit runner and CLI orchestration.
//!
//! Provides execution support for compiling, exporting, and live-previewing
//! circuits via tscircuit. Supports local node_modules installations,
//! global binaries, and on-demand runners via `bun` and `npx`.

pub mod command;
pub mod detect;
pub mod error;
pub mod kicad;
pub mod output;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};

pub use command::{
    add_command, build_command, dev_command, export_command, install_command, mechanical_command,
    search_command,
};
pub use detect::{detect_runner, detect_ts_runner};
pub use error::RunnerError;
pub use kicad::{convert_kicad, is_kicad_target};

/// Represents the detected or configured tscircuit execution backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerKind {
    /// Local node_modules executable (`./node_modules/.bin/tsci` or `tscircuit`).
    Local(PathBuf),
    /// Globally installed executable on PATH (`tsci` or `tscircuit`).
    Global(PathBuf),
}

impl RunnerKind {
    /// Human-friendly description of this runner backend.
    pub fn description(&self) -> String {
        match self {
            Self::Local(p) => format!("local executable ({})", p.display()),
            Self::Global(p) => format!("global executable ({})", p.display()),
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
        let (working_dir, target_file) = resolve_hardware_context(file);
        let mut cmd = dev_command(&self.kind, &target_file, port);
        if let Some(ref dir) = working_dir {
            cmd.current_dir(dir);
            if let RunnerKind::Local(ref p) = self.kind {
                if p.is_relative() {
                    let abs = std::env::current_dir()
                        .unwrap_or_else(|_| PathBuf::from("."))
                        .join(p);
                    cmd = dev_command(&RunnerKind::Local(abs), &target_file, port);
                    cmd.current_dir(dir);
                }
            }
        }
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
        let (working_dir, target_file) = resolve_hardware_context(file);
        let mut cmd = build_command(&self.kind, &target_file);
        if let Some(ref dir) = working_dir {
            cmd.current_dir(dir);
            if let RunnerKind::Local(ref p) = self.kind {
                if p.is_relative() {
                    let abs = std::env::current_dir()
                        .unwrap_or_else(|_| PathBuf::from("."))
                        .join(p);
                    cmd = std::process::Command::new(abs);
                    cmd.arg("build").arg(&target_file).arg("--ignore-errors");
                    cmd.current_dir(dir);
                }
            }
        }
        cmd.stdin(Stdio::null());
        cmd.env("CI", "1");
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let cmd_str = command::format_command(&cmd);
        let mut child = cmd.spawn().map_err(|source| RunnerError::Io {
            command: cmd_str.clone(),
            source,
        })?;

        let mut stdout_pipe = child.stdout.take();
        let mut stderr_pipe = child.stderr.take();

        let stdout_handle = std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut pipe) = stdout_pipe.take() {
                use std::io::Read;
                let _ = pipe.read_to_end(&mut buf);
            }
            buf
        });

        let stderr_handle = std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut pipe) = stderr_pipe.take() {
                use std::io::Read;
                let _ = pipe.read_to_end(&mut buf);
            }
            buf
        });

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);
        let status = loop {
            match child.try_wait() {
                Ok(Some(s)) => break s,
                Ok(None) => {
                    if start.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(RunnerError::CommandFailed {
                            command: cmd_str,
                            code: None,
                            stderr: "compilation timed out after 30 seconds".to_string(),
                            help: "compilation took too long; falling back to cached circuit.json if available".to_string(),
                        });
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(source) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(RunnerError::Io {
                        command: cmd_str,
                        source,
                    });
                }
            }
        };

        let stdout = stdout_handle.join().unwrap_or_default();
        let stderr = stderr_handle.join().unwrap_or_default();
        let out = std::process::Output {
            status,
            stdout,
            stderr,
        };

        let stdout_str = String::from_utf8_lossy(&out.stdout);
        let produced = output::resolve_circuit_json(file, &stdout_str);

        if !out.status.success() && produced.is_err() {
            let details = command::extract_output_details(&out);
            return Err(RunnerError::CommandFailed {
                command: command::format_command(&cmd),
                code: out.status.code(),
                stderr: details.clone(),
                help: details,
            });
        }

        let produced_path = produced?;
        if let Some(dest) = out_file {
            output::copy_output(&produced_path, dest)?;
            Ok(dest.to_path_buf())
        } else {
            Ok(produced_path)
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

    /// Runs the mechanical/enclosure generation script via a TypeScript runner.
    ///
    /// Uses `detect_ts_runner()` to find a suitable TS executor (bun > npx > node).
    /// The script at `script_path` is executed with `args`.
    pub fn mechanical(
        &self,
        script_path: &Path,
        args: &[String],
    ) -> Result<(), RunnerError> {
        let path_var = std::env::var_os("PATH");
        let ts_runner = detect_ts_runner(path_var.as_deref()).ok_or(RunnerError::NotFound)?;

        let mut cmd = command::mechanical_command(&ts_runner, script_path, args);
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
                stderr: "mechanical script failed".into(),
                help: "check script arguments and dependencies".into(),
            });
        }
        Ok(())
    }
}

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
