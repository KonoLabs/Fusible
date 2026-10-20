//! Subcommand: `fusible check` / `lint` — run the electrical linter.

use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use crate::commands::target::{is_kicad_target, is_tsx_target, resolve_entry, MissingTargetError};
use crate::config::FusibleConfig;
use crate::linter::{FuseBlown, Linter};
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

/// Probes for existing pre-built Circuit JSON when runner is unavailable.
fn find_fallback_circuit_json(entry: &Path) -> Option<PathBuf> {
    let stem = entry
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();
    let parent = entry.parent().unwrap_or_else(|| Path::new(""));
    let candidates = [
        parent.join("dist").join("circuit.json"),
        parent.join("dist").join(stem.as_ref()).join("circuit.json"),
        PathBuf::from("dist").join("circuit.json"),
        PathBuf::from("dist")
            .join(stem.as_ref())
            .join("circuit.json"),
        entry.with_extension("circuit.json"),
        entry.with_file_name("circuit.json"),
        PathBuf::from("circuit.json"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

fn compile_tsx(entry: &Path) -> Result<(PathBuf, String), CheckError> {
    match Runner::detect() {
        Ok(runner) => {
            println!("⚙️  Compiling \"{}\" via tscircuit...", entry.display());
            match runner.build(entry, None) {
                Ok(built_json) => {
                    let content =
                        std::fs::read_to_string(&built_json).map_err(|source| ReadError {
                            path: built_json.display().to_string(),
                            source,
                        })?;
                    Ok((built_json, content))
                }
                Err(runner_err) => {
                    if let Some(fallback) = find_fallback_circuit_json(entry) {
                        println!(
                            "⚠️  tscircuit compilation failed; falling back to existing \"{}\"...",
                            fallback.display()
                        );
                        let content =
                            std::fs::read_to_string(&fallback).map_err(|source| ReadError {
                                path: fallback.display().to_string(),
                                source,
                            })?;
                        Ok((fallback, content))
                    } else {
                        Err(CheckError::Runner(runner_err))
                    }
                }
            }
        }
        Err(runner_err) => {
            if let Some(fallback) = find_fallback_circuit_json(entry) {
                println!(
                    "⚠️  tscircuit runner not found; falling back to existing \"{}\"...",
                    fallback.display()
                );
                let content = std::fs::read_to_string(&fallback).map_err(|source| ReadError {
                    path: fallback.display().to_string(),
                    source,
                })?;
                Ok((fallback, content))
            } else {
                Err(CheckError::Runner(runner_err))
            }
        }
    }
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

/// Runs the electrical linter on a target file or project manifest.
pub fn run(
    file: Option<PathBuf>,
    mcu: Option<String>,
    pins_rs: Option<PathBuf>,
) -> Result<(), CheckError> {
    let entry = resolve_entry(file)?;
    let target_mcu = resolve_mcu(mcu);
    let (json_path, source) = load_circuit_source(&entry)?;

    let circuit = parser::parse(&source, &json_path)?;
    let netlist = Netlist::from_circuit(&circuit);
    let ctx = crate::linter::LintContext::new(&circuit, &netlist).with_mcu(target_mcu.as_deref());
    let violations = Linter::default().run_with_context(&ctx);

    if violations.is_empty() {
        println!("✅ Circuit validated (fuse intact)");
        if let Some(ref mcu) = target_mcu {
            println!("   silicon target: {mcu}");
        }

        let target_pins_rs = resolve_pins_rs(pins_rs);
        if let Some(pins_path) = target_pins_rs {
            let board_name = resolve_board_name();
            if let Some(mcu_info) =
                crate::codegen::extract_mcu_pins(&circuit, &netlist, target_mcu.as_deref())
            {
                let code = crate::codegen::generate_pins_rs(&mcu_info, &board_name);
                if let Some(parent) = pins_path.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                std::fs::write(&pins_path, code).map_err(|source| ReadError {
                    path: pins_path.display().to_string(),
                    source,
                })?;
                println!(
                    "🦀 Generated firmware pin bindings -> {}",
                    pins_path.display()
                );
            }
        }
        return Ok(());
    }

    Err(CheckError::FuseBlown(FuseBlown::new(violations)))
}

pub fn resolve_mcu(explicit: Option<String>) -> Option<String> {
    explicit.or_else(|| {
        FusibleConfig::find_and_load(Path::new("."))
            .ok()
            .flatten()
            .and_then(|(_, cfg)| cfg.mcu.and_then(|m| m.chip))
    })
}

fn resolve_pins_rs(explicit: Option<PathBuf>) -> Option<PathBuf> {
    explicit.or_else(|| {
        FusibleConfig::find_and_load(Path::new("."))
            .ok()
            .flatten()
            .and_then(|(_, cfg)| cfg.firmware.and_then(|f| f.pins_rs))
    })
}

fn resolve_board_name() -> String {
    FusibleConfig::find_and_load(Path::new("."))
        .ok()
        .flatten()
        .and_then(|(_, cfg)| cfg.name)
        .unwrap_or_else(|| "Hardware Board".to_string())
}
