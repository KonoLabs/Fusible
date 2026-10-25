//! Subcommand: `fusible check` / `lint` — run the electrical linter.

use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::assembly::{self, AssemblyError};
use crate::commands::check_merge::{label, merge};
use crate::commands::links::check_links;
pub use crate::commands::source::{find_fallback_circuit_json, load_circuit_source};
use crate::commands::target::{resolve_boards, BoardTarget, TargetError};
use crate::config::FusibleConfig;
use crate::linter::{FuseBlown, LintContext, Linter, Severity};
use crate::netlist::Netlist;
use crate::parser;
use crate::tscircuit::RunnerError;

/// The input file could not be read.
#[derive(Debug, Error, Diagnostic)]
#[error("cannot read the Circuit JSON at \"{path}\"")]
#[diagnostic(help("check that the file exists and is readable"))]
pub struct ReadError {
    pub path: String,
    #[source]
    pub source: std::io::Error,
}

/// Unified error enum for the check pipeline.
#[derive(Debug, Error, Diagnostic)]
pub enum CheckError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Target(#[from] TargetError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Runner(#[from] RunnerError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Read(#[from] ReadError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Parse(#[from] parser::JsonDiagnostic),

    #[error(transparent)]
    #[diagnostic(transparent)]
    FuseBlown(#[from] FuseBlown),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Assembly(Box<AssemblyError>),
}

/// Loads, parses and lints the circuit at `entry` without writing any file.
///
/// Returns the path and text of the Circuit JSON that passed the fuse check.
pub fn lint_source(
    entry: &Path,
    target_mcu: Option<String>,
) -> Result<(PathBuf, String), CheckError> {
    let (json_path, source) = load_circuit_source(entry)?;
    let circuit = parser::parse(&source, &json_path)?;
    let netlist = Netlist::from_circuit(&circuit);
    let ctx = LintContext::new(&circuit, &netlist).with_mcu(target_mcu.as_deref());
    let (warnings, violations): (Vec<_>, Vec<_>) = Linter::default()
        .run_with_context(&ctx)
        .into_iter()
        .partition(|d| d.severity == Severity::Warning);

    for warning in warnings {
        eprintln!("{:?}", miette::Report::new(warning));
    }
    if !violations.is_empty() {
        return Err(CheckError::FuseBlown(FuseBlown::new(violations)));
    }

    println!("✅ Circuit validated (fuse intact)");
    if let Some(mcu) = target_mcu {
        println!("   silicon target: {mcu}");
    }
    Ok((json_path, source))
}

/// Runs the electrical linter on the target board, or on every board of the manifest.
pub fn run(
    file: Option<PathBuf>,
    mcu: Option<String>,
    pins_rs: Option<PathBuf>,
) -> Result<(), CheckError> {
    run_boards(&resolve_boards(file)?, mcu, pins_rs)
}

/// Checks every board, then returns the merged violations or the first tool error.
///
/// CLI `mcu` and `pins_rs` override the manifest values of each board.
pub fn run_boards(
    boards: &[BoardTarget],
    mcu: Option<String>,
    pins_rs: Option<PathBuf>,
) -> Result<(), CheckError> {
    let mut failure = None;
    let mut report = |blown| eprintln!("Error: {:?}", miette::Report::new(blown));
    for board in boards {
        let name = board.name.as_deref().filter(|_| boards.len() > 1);
        if let Some(name) = name {
            println!("\n▸ board {name}");
        }
        let next = check_board(board, mcu.clone(), pins_rs.clone())
            .err()
            .map(|error| label(error, name));
        failure = merge(failure, next, &mut report);
    }
    failure = merge(failure, check_manifest_links(boards).err(), &mut report);
    if failure.is_none() {
        failure = check_manifest_assembly(boards).err();
    }
    failure.map_or(Ok(()), Err)
}

fn checked_every_board(cfg: &FusibleConfig, boards: &[BoardTarget]) -> bool {
    !cfg.boards.is_empty() && cfg.boards.len() == boards.len()
}

fn check_manifest_links(boards: &[BoardTarget]) -> Result<(), CheckError> {
    let Some((_, cfg)) = FusibleConfig::nearest() else {
        return Ok(());
    };
    if cfg.links.is_empty() || !checked_every_board(&cfg, boards) {
        return Ok(());
    }
    check_links(&cfg)
}

fn check_manifest_assembly(boards: &[BoardTarget]) -> Result<(), CheckError> {
    let Some((manifest, cfg)) = FusibleConfig::nearest() else {
        return Ok(());
    };
    if cfg.assembly.is_none() || !checked_every_board(&cfg, boards) {
        return Ok(());
    }
    let root = manifest.parent().unwrap_or(Path::new("."));
    assembly::verify(&cfg, root).map_err(|error| match error {
        AssemblyError::Check(inner) => inner,
        other => CheckError::Assembly(Box::new(other)),
    })
}

fn check_board(
    board: &BoardTarget,
    mcu: Option<String>,
    pins_rs: Option<PathBuf>,
) -> Result<(), CheckError> {
    let target_mcu = mcu.or_else(|| board.mcu.clone());
    let (json_path, source) = lint_source(&board.entry, target_mcu.clone())?;
    let pins = pins_rs.map_or_else(|| board.pins_rs.clone(), |path| vec![path]);
    if pins.is_empty() {
        return Ok(());
    }
    let circuit = parser::parse(&source, &json_path)?;
    let netlist = Netlist::from_circuit(&circuit);
    let ctx = LintContext::new(&circuit, &netlist).with_mcu(target_mcu.as_deref());
    pins.iter()
        .try_for_each(|path| write_pins_rs(&ctx, path, &board.title))
}

/// Generates firmware pin bindings for the circuit's MCU into `pins_path`.
fn write_pins_rs(ctx: &LintContext, pins_path: &Path, title: &str) -> Result<(), CheckError> {
    let Some(mcu_info) = crate::codegen::extract_mcu_pins(ctx.circuit, ctx.netlist, ctx.mcu) else {
        return Ok(());
    };
    let code = crate::codegen::generate_pins_rs(&mcu_info, title);
    if let Some(parent) = pins_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(pins_path, code).map_err(|source| ReadError {
        path: pins_path.display().to_string(),
        source,
    })?;
    println!(
        "🦀 Generated firmware pin bindings -> {}",
        pins_path.display()
    );
    Ok(())
}
