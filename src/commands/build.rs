//! Subcommand: `fusible build` — full hardware validation and firmware build.

use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::check::{self, CheckError};
use crate::config::FusibleConfig;

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

/// Orchestrates full circuit validation and firmware build.
pub fn run(file: Option<PathBuf>) -> Result<(), BuildError> {
    let loaded = FusibleConfig::find_and_load(Path::new(".")).ok().flatten();
    let cfg = loaded.as_ref().map(|(_, c)| c);

    let pins_rs = cfg
        .and_then(|c| c.firmware.as_ref())
        .and_then(|f| f.pins_rs.clone());

    // 1. Hardware validation & pin bindings: the fuse blows here if electrical rules fail!
    check::run(file, None, pins_rs)?;

    // 2. Firmware compilation: only if fuse is intact.
    build_firmware_if_configured(cfg)?;

    println!("✅ Full build succeeded (fuse intact)");
    Ok(())
}

fn build_firmware_if_configured(cfg: Option<&FusibleConfig>) -> Result<(), BuildError> {
    let Some(cfg) = cfg else {
        return Ok(());
    };
    let Some(fw) = &cfg.firmware else {
        return Ok(());
    };
    let Some(cmd) = &fw.build_cmd else {
        return Ok(());
    };

    println!("🦀 Building firmware: \"{}\"...", cmd);
    let status = execute_shell(cmd, fw.path.as_deref())
        .map_err(|e| BuildError::ExecutionFailed(e.to_string()))?;

    if !status.success() {
        return Err(BuildError::FirmwareFailed(cmd.clone()));
    }
    Ok(())
}

fn execute_shell(
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
