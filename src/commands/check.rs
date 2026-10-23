//! Subcommand: `fusible check` / `lint` — run the electrical linter.

use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::manifest::resolve_pins_rs;
pub use crate::commands::manifest::{resolve_board_name, resolve_mcu};
use crate::commands::target::{is_kicad_target, is_tsx_target, resolve_entry, MissingTargetError};
use crate::linter::{FuseBlown, LintContext, Linter, Severity};
use crate::netlist::Netlist;
use crate::parser;
use crate::tscircuit::{Runner, RunnerError};

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
    MissingTarget(#[from] MissingTargetError),

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
}

/// Finds pre-built Circuit JSON for `entry` that is at least as new as the source file.
pub fn find_fallback_circuit_json(entry: &Path) -> Option<PathBuf> {
    let stem = entry
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();
    let parent = entry.parent().unwrap_or_else(|| Path::new(""));
    let source_time = std::fs::metadata(entry).and_then(|m| m.modified()).ok();
    let candidates = [
        parent.join("dist").join("circuit.json"),
        parent.join("dist").join(stem.as_ref()).join("circuit.json"),
        PathBuf::from("dist").join("circuit.json"),
        PathBuf::from("dist")
            .join(stem.as_ref())
            .join("circuit.json"),
        entry.with_extension("circuit.json"),
        entry.with_file_name("circuit.json"),
    ];
    candidates.into_iter().find(|p| {
        let built = std::fs::metadata(p).and_then(|m| m.modified()).ok();
        p.is_file() && built >= source_time
    })
}

fn read_json(path: PathBuf) -> Result<(PathBuf, String), CheckError> {
    let content = std::fs::read_to_string(&path).map_err(|source| ReadError {
        path: path.display().to_string(),
        source,
    })?;
    Ok((path, content))
}

fn compile_tsx(entry: &Path) -> Result<(PathBuf, String), CheckError> {
    let runner = match Runner::detect() {
        Ok(runner) => runner,
        Err(runner_err) => {
            let fallback = find_fallback_circuit_json(entry).ok_or(runner_err)?;
            println!(
                "⚠️  tscircuit runner not found; using \"{}\", which is newer than the source.",
                fallback.display()
            );
            return read_json(fallback);
        }
    };
    println!("⚙️  Compiling \"{}\" via tscircuit...", entry.display());
    let start = std::time::Instant::now();
    let built_json = runner.build(entry, None)?;
    println!(
        "✓ Circuit compiled in {:.1}s — preview ready",
        start.elapsed().as_secs_f32()
    );
    read_json(built_json)
}

fn convert_kicad(entry: &Path) -> Result<(PathBuf, String), CheckError> {
    crate::tscircuit::convert_kicad(entry).map_err(|err_msg| {
        CheckError::Read(ReadError {
            path: entry.display().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, err_msg),
        })
    })
}

pub fn load_circuit_source(entry: &Path) -> Result<(PathBuf, String), CheckError> {
    if is_kicad_target(entry) {
        convert_kicad(entry)
    } else if is_tsx_target(entry) {
        compile_tsx(entry)
    } else {
        let content = std::fs::read_to_string(entry).map_err(|source| ReadError {
            path: entry.display().to_string(),
            source,
        })?;
        Ok((entry.to_path_buf(), content))
    }
}

/// Loads, parses and lints the circuit at `entry` without writing any file.
///
/// Returns the path and text of the Circuit JSON that passed the fuse check.
pub fn lint_source(entry: &Path, mcu: Option<String>) -> Result<(PathBuf, String), CheckError> {
    let target_mcu = resolve_mcu(mcu);
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

/// Runs the electrical linter on a target file or project manifest.
pub fn run(
    file: Option<PathBuf>,
    mcu: Option<String>,
    pins_rs: Option<PathBuf>,
) -> Result<(), CheckError> {
    let entry = resolve_entry(file)?;
    let target_mcu = resolve_mcu(mcu);
    let (json_path, source) = lint_source(&entry, target_mcu.clone())?;

    match resolve_pins_rs(pins_rs) {
        Some(pins_path) => {
            let circuit = parser::parse(&source, &json_path)?;
            let netlist = Netlist::from_circuit(&circuit);
            let ctx = LintContext::new(&circuit, &netlist).with_mcu(target_mcu.as_deref());
            write_pins_rs(&ctx, &pins_path)
        }
        None => Ok(()),
    }
}

/// Generates firmware pin bindings for the circuit's MCU into `pins_path`.
fn write_pins_rs(ctx: &LintContext, pins_path: &Path) -> Result<(), CheckError> {
    let Some(mcu_info) = crate::codegen::extract_mcu_pins(ctx.circuit, ctx.netlist, ctx.mcu) else {
        return Ok(());
    };
    let code = crate::codegen::generate_pins_rs(&mcu_info, &resolve_board_name());
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
