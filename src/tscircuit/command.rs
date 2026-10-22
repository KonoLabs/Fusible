//! Command construction and execution formatting for tscircuit runners.

use std::path::Path;
use std::process::Command;

use crate::tscircuit::RunnerKind;

/// Builds the `Command` for the interactive `dev` server.
pub fn dev_command(kind: &RunnerKind, file: &Path, port: Option<u16>) -> Command {
    let mut cmd = base_command(kind);
    cmd.arg("dev").arg(file);
    if let Some(port) = port {
        cmd.arg("-p").arg(port.to_string());
    }
    cmd
}

/// Builds the `Command` for exporting a circuit to a given format.
pub fn export_command(
    kind: &RunnerKind,
    file: &Path,
    format: &str,
    output: Option<&Path>,
) -> Command {
    let mut cmd = base_command(kind);
    cmd.arg("export").arg(file).arg("-f").arg(format);
    if let Some(output) = output {
        cmd.arg("-o").arg(output);
    }
    cmd
}

/// Builds the `Command` for building circuit json from a circuit file.
pub fn build_command(kind: &RunnerKind, file: &Path) -> Command {
    let mut cmd = base_command(kind);
    cmd.arg("build").arg(file).arg("--ignore-errors");
    cmd
}

/// Builds the `Command` for adding tscircuit packages or registry components.
pub fn add_command(kind: &RunnerKind, packages: &[String]) -> Command {
    let mut cmd = base_command(kind);
    cmd.arg("add");
    for pkg in packages {
        cmd.arg(pkg);
    }
    cmd
}

/// Builds the `Command` for installing dependencies in a tscircuit project.
pub fn install_command(kind: &RunnerKind) -> Command {
    let mut cmd = base_command(kind);
    cmd.arg("install");
    cmd
}

/// Builds the `Command` for searching packages/footprints in the tscircuit registry.
pub fn search_command(kind: &RunnerKind, query: &[String]) -> Command {
    let mut cmd = base_command(kind);
    cmd.arg("search");
    for q in query {
        cmd.arg(q);
    }
    cmd
}

/// Constructs a base `Command` with the binary configured.
pub fn base_command(kind: &RunnerKind) -> Command {
    let program: &Path = match kind {
        RunnerKind::Local(p) | RunnerKind::Global(p) => p,
    };
    Command::new(program)
}
