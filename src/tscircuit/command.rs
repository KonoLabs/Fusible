//! Command construction and execution formatting for tscircuit runners.

use std::path::Path;
use std::process::{Command, Output};

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
    cmd.arg("build").arg(file);
    cmd
}

/// Constructs a base `Command` with the binary and prefix arguments configured.
pub fn base_command(kind: &RunnerKind) -> Command {
    let program: &Path = match kind {
        RunnerKind::Local(p) | RunnerKind::Global(p) | RunnerKind::Bun(p) | RunnerKind::Npx(p) => {
            p
        }
    };
    let prefix_args: &[&str] = match kind {
        RunnerKind::Local(_) | RunnerKind::Global(_) => &[],
        RunnerKind::Bun(_) => &["x", "tscircuit"],
        RunnerKind::Npx(_) => &["tscircuit"],
    };
    let mut cmd = Command::new(program);
    cmd.args(prefix_args);
    cmd
}

/// Formats a command for diagnostic display.
pub fn format_command(cmd: &Command) -> String {
    let program = cmd.get_program().to_string_lossy();
    let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy()).collect();
    if args.is_empty() {
        program.into_owned()
    } else {
        format!("{program} {}", args.join(" "))
    }
}

/// Extracts error details from a command's stderr and stdout.
pub fn extract_output_details(out: &Output) -> String {
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr;
    }
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !stdout.is_empty() {
        return stdout;
    }
    format!("command failed with exit code {:?}", out.status.code())
}
