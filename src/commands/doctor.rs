//! Subcommand: `fusible doctor` / `setup` — inspect toolchain health and available runners.

use std::path::Path;
use std::process::Command;

use miette::Diagnostic;
use thiserror::Error;

use crate::config::FusibleConfig;
use crate::tscircuit::detect::find_binary_in_path;
use crate::tscircuit::{bun_path, Runner};

pub mod json;

/// Errors that can occur during toolchain inspection or automated installation.
#[derive(Debug, Error, Diagnostic)]
pub enum DoctorError {
    #[error("failed to run package manager: {command}")]
    #[diagnostic(
        code(fusible::doctor::install_failed),
        help(
            "verify your internet connection or install manually with `npm install -g tscircuit`"
        )
    )]
    InstallFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("package manager command `{command}` exited with failure ({code:?})")]
    #[diagnostic(
        code(fusible::doctor::install_non_zero),
        help("check stderr output for permission or network issues")
    )]
    InstallNonZero { command: String, code: Option<i32> },

    #[error("no supported package manager found to install tscircuit")]
    #[diagnostic(
        code(fusible::doctor::no_package_manager),
        help("install bun (https://bun.sh) or node/npm (https://nodejs.org) first")
    )]
    NoPackageManager,
}

fn report_prerequisites() -> (bool, bool) {
    let path_var = std::env::var_os("PATH");
    let has_bun = bun_path();
    let has_node = path_var
        .as_deref()
        .and_then(|p| find_binary_in_path("node", p));
    let has_npm = path_var
        .as_deref()
        .and_then(|p| find_binary_in_path("npm", p));
    let has_tsci = path_var.as_deref().and_then(|p| {
        find_binary_in_path("tsci", p).or_else(|| find_binary_in_path("tscircuit", p))
    });

    println!("Runtime prerequisites:");
    print_status("tsci (tscircuit)", has_tsci.as_deref());
    print_status("bun", has_bun.as_deref());
    print_status("node", has_node.as_deref());
    print_status("npm", has_npm.as_deref());
    (has_bun.is_some(), has_npm.is_some())
}

fn report_manifest() {
    println!("\nMonorepo configuration:");
    match FusibleConfig::find_and_load(Path::new(".")) {
        Ok(Some((path, cfg))) => {
            println!("  ✅ Found manifest at \"{}\"", path.display());
            report_sections(&cfg, path.parent().unwrap_or(Path::new("")));
        }
        Ok(None) => {
            println!("  ℹ️  No fusible.yml found in current directory (run `fusible init` to create one)");
        }
        Err(e) => {
            println!("  ⚠️  Manifest error: {e}");
        }
    }
}

fn report_sections(cfg: &FusibleConfig, root: &Path) {
    let shown = |path: &Path| {
        path.strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string()
    };
    for (name, board) in &cfg.boards {
        let entry = board
            .entry
            .as_deref()
            .map_or_else(|| "default".into(), shown);
        let mcu = board.mcu.as_deref().unwrap_or("none");
        println!("     • board {name}: {entry} (silicon target: {mcu})");
    }
    for (name, fw) in &cfg.firmware {
        let board = fw.board.as_deref().unwrap_or("the only board");
        println!("     • firmware {name} -> {board}");
    }
    for (name, body) in &cfg.bodies {
        match (&body.source, &body.generate) {
            (Some(source), _) => println!("     • body {name}: authored, {}", shown(source)),
            (None, Some(generator)) => println!("     • body {name}: generated {generator}"),
            (None, None) => println!("     • body {name}: no source"),
        }
    }
}

fn report_runner(install: bool, has_bun: bool, has_npm: bool) -> Result<(), DoctorError> {
    println!("\ntscircuit runner status:");
    match Runner::detect() {
        Ok(runner) => {
            println!(
                "  ✅ Ready — active backend: {}",
                runner.kind().description()
            );
        }
        Err(_) => {
            println!("  ⚠️  No active runner detected.");
            println!(
                "     Fusible can still lint pre-existing circuit.json files, but TSX compilation,"
            );
            println!("     dev webview, and exports require tscircuit.");

            if install {
                println!("\n🔧 Installing tscircuit CLI...");
                install_tscircuit(has_bun, has_npm)?;
                println!("✅ tscircuit installed successfully!");
            } else {
                println!(
                    "\nTip: Run `fusible doctor --install` to automatically install tscircuit,"
                );
                println!("     or run: `npm install -g tscircuit` / `bun add -g tscircuit`");
            }
        }
    }
    Ok(())
}

/// Runs the environment diagnostics and optionally installs missing tools.
///
/// With `as_json` it prints only the machine-readable toolchain report and never fails on missing tools.
pub fn run(install: bool, as_json: bool) -> Result<(), DoctorError> {
    if as_json {
        println!("{}", json::report());
        return Ok(());
    }
    println!("🩺 Fusible Toolchain Doctor\n");
    let (has_bun, has_npm) = report_prerequisites();
    report_manifest();
    report_runner(install, has_bun, has_npm)?;
    println!();
    Ok(())
}

fn print_status(name: &str, path: Option<&Path>) {
    match path {
        Some(p) => println!("  ✅ {name:<14} -> {}", p.display()),
        None => println!("  ⚪ {name:<14} -> not found"),
    }
}

fn install_tscircuit(has_bun: bool, has_npm: bool) -> Result<(), DoctorError> {
    let (prog, args): (String, &[&str]) = match (bun_path(), has_bun, has_npm) {
        (Some(bun), true, _) => (bun.display().to_string(), &["add", "-g", "tscircuit"]),
        (_, _, true) => ("npm".into(), &["install", "-g", "tscircuit"]),
        _ => return Err(DoctorError::NoPackageManager),
    };

    println!("   Executing: {prog} {}", args.join(" "));
    let status =
        Command::new(&prog)
            .args(args)
            .status()
            .map_err(|source| DoctorError::InstallFailed {
                command: format!("{prog} {}", args.join(" ")),
                source,
            })?;

    if !status.success() {
        return Err(DoctorError::InstallNonZero {
            command: format!("{prog} {}", args.join(" ")),
            code: status.code(),
        });
    }

    Ok(())
}
