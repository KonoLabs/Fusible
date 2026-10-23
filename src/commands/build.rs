//! Subcommand: `fusible build` — full hardware validation and firmware build.

use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::check::{self, CheckError};
use crate::commands::target::{resolve_board, resolve_boards, BoardTarget};
use crate::config::{FirmwareConfig, FusibleConfig};

/// Error raised during full build orchestration.
#[derive(Debug, Error, Diagnostic)]
pub enum BuildError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Check(#[from] CheckError),

    #[error("firmware build command failed: {0}")]
    #[diagnostic(help("check firmware build configuration and dependencies"))]
    FirmwareFailed(String),

    #[error("firmware build execution failed: {0}")]
    #[diagnostic(help("ensure your shell and build tools are properly installed"))]
    ExecutionFailed(String),
}

/// Checks the target boards, then builds the firmware bound to them.
///
/// `target` names a firmware, a board, or a circuit file; without it every board and
/// firmware of the manifest is built. No firmware builds unless every board keeps the
/// fuse intact.
pub fn run(target: Option<PathBuf>) -> Result<(), BuildError> {
    let cfg = FusibleConfig::nearest().map(|(_, c)| c).unwrap_or_default();
    let (boards, firmware) = select(&cfg, target)?;
    check::run_boards(&boards, None, None)?;
    for (name, fw) in firmware {
        build_firmware(name, fw)?;
    }
    println!("✅ Full build succeeded (fuse intact)");
    Ok(())
}

type Selection<'a> = (Vec<BoardTarget>, Vec<(&'a String, &'a FirmwareConfig)>);

fn select(cfg: &FusibleConfig, target: Option<PathBuf>) -> Result<Selection<'_>, CheckError> {
    let by_name = target
        .as_deref()
        .and_then(|t| t.to_str())
        .and_then(|n| cfg.firmware.get_key_value(n));
    if let Some((name, fw)) = by_name {
        let board = resolve_board(fw.board.clone().map(PathBuf::from))?;
        return Ok((vec![board], vec![(name, fw)]));
    }
    let boards = resolve_boards(target)?;
    let firmware = cfg
        .firmware
        .iter()
        .filter(|(name, _)| boards.iter().any(|b| b.firmware.contains(name)))
        .collect();
    Ok((boards, firmware))
}

fn build_firmware(name: &str, fw: &FirmwareConfig) -> Result<(), BuildError> {
    let Some(cmd) = &fw.build_cmd else {
        return Ok(());
    };
    println!("🦀 Building firmware {name}: \"{cmd}\"...");
    let status = execute_shell(cmd, fw.path.as_deref())
        .map_err(|e| BuildError::ExecutionFailed(e.to_string()))?;
    if !status.success() {
        return Err(BuildError::FirmwareFailed(cmd.clone()));
    }
    Ok(())
}

/// Runs `cmd` through the platform shell, inside `work_dir` when given.
pub(crate) fn execute_shell(
    cmd: &str,
    work_dir: Option<&Path>,
) -> Result<std::process::ExitStatus, std::io::Error> {
    let mut command = if cfg!(target_os = "windows") {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", cmd]);
        c
    } else {
        let mut c = std::process::Command::new("sh");
        c.args(["-c", cmd]);
        c
    };

    if let Some(dir) = work_dir {
        command.current_dir(dir);
    }

    command.status()
}
