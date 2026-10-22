//! Runner commands attached to the user's terminal.

use std::path::Path;
use std::process::ExitStatus;

use super::process::{format_command, inherit_stdio, run_inherited};
use super::{command, resolve_hardware_context, Runner, RunnerError};

impl Runner {
    /// Executes `dev [file] -p [port]`, inheriting standard I/O for interactive hot-reloading.
    pub fn dev(&self, file: &Path, port: Option<u16>) -> Result<ExitStatus, RunnerError> {
        let (working_dir, target_file) = resolve_hardware_context(file);
        let mut cmd = command::dev_command(&self.kind, &target_file, port);
        if let Some(ref dir) = working_dir {
            cmd.current_dir(dir);
        }
        inherit_stdio(&mut cmd);

        cmd.status().map_err(|source| RunnerError::Io {
            command: format_command(&cmd),
            source,
        })
    }

    /// Adds packages to the tscircuit project.
    pub fn add(&self, packages: &[String], cwd: Option<&Path>) -> Result<(), RunnerError> {
        let mut cmd = command::add_command(&self.kind, packages);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        run_inherited(
            cmd,
            "failed to add package(s)",
            "check package names or registry connectivity",
        )
    }

    /// Installs dependencies in the project.
    pub fn install(&self, cwd: Option<&Path>) -> Result<(), RunnerError> {
        let mut cmd = command::install_command(&self.kind);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        run_inherited(
            cmd,
            "failed to install dependencies",
            "check network connection or package.json validity",
        )
    }

    /// Searches the tscircuit registry for components, footprints, and CAD models.
    pub fn search(&self, query: &[String]) -> Result<(), RunnerError> {
        let cmd = command::search_command(&self.kind, query);
        run_inherited(cmd, "search command failed", "check network connection")
    }
}
