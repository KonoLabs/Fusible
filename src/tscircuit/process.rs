//! Process execution helpers shared by every runner command.

use std::process::{Command, Output, Stdio};

use crate::tscircuit::RunnerError;

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

/// Connects the command's stdin, stdout and stderr to the terminal.
pub fn inherit_stdio(cmd: &mut Command) {
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
}

/// Runs `cmd` attached to the terminal; a non-zero exit becomes `CommandFailed`.
pub fn run_inherited(mut cmd: Command, stderr: &str, help: &str) -> Result<(), RunnerError> {
    inherit_stdio(&mut cmd);
    let status = cmd.status().map_err(|source| RunnerError::Io {
        command: format_command(&cmd),
        source,
    })?;

    if !status.success() {
        return Err(RunnerError::CommandFailed {
            command: format_command(&cmd),
            code: status.code(),
            stderr: stderr.into(),
            help: help.into(),
        });
    }
    Ok(())
}
