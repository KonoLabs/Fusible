//! Subcommand: `fusible assembly` — place boards and bodies in one frame and export one model.
//!
//! Each board is exported as STEP by the tscircuit runner, each generated body is rebuilt
//! as STEP, and each authored body is read from its `source:`. The parts are moved by their
//! placements in Rust, then an embedded occt-wasm script merges them into
//! `assembly.glb` and/or `assembly.step`. Every board involved is fuse-gated first.

mod error;
mod gate;
mod parts;
pub mod place;

#[cfg(test)]
mod place_tests;

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::commands::body::run_script;
use crate::config::FusibleConfig;

pub use error::AssemblyError;
use gate::fuse_gate;
use parts::{collect_parts, Part};

const FORMATS: &[&str] = &["glb", "step"];

#[derive(Debug, Serialize)]
struct Output {
    format: &'static str,
    path: PathBuf,
}

#[derive(Debug, Serialize)]
struct Job {
    parts: Vec<Part>,
    outputs: Vec<Output>,
}

/// Builds the `assembly:` of the manifest into `<out_dir>/assembly.<format>`.
///
/// `format` and `output` override the manifest's `format` and `out_dir`. Every board in the
/// assembly, and every board a generated body is fitted to, is linted before any file is
/// written unless `skip_lint` is set.
pub fn run(
    format: Option<String>,
    output: Option<PathBuf>,
    skip_lint: bool,
) -> Result<(), AssemblyError> {
    let (manifest, cfg) = FusibleConfig::nearest().ok_or(AssemblyError::Missing)?;
    let assembly = cfg.assembly.clone().ok_or(AssemblyError::Missing)?;
    let root = manifest.parent().unwrap_or(Path::new("."));
    let formats = formats(format.or(assembly.format.clone()))?;
    let out_dir = output
        .or(assembly.out_dir.clone())
        .unwrap_or_else(|| root.join("dist"));
    let out_dir = std::path::absolute(out_dir)?;
    let circuits = fuse_gate(&cfg, &assembly, skip_lint)?;
    let parts_dir = out_dir.join("assembly");
    let parts = collect_parts(&cfg, &assembly, &circuits, root, &parts_dir)?;
    let outputs = formats
        .iter()
        .map(|&format| Output {
            format,
            path: out_dir.join(format!("assembly.{format}")),
        })
        .collect();
    merge(Job { parts, outputs })
}

fn formats(format: Option<String>) -> Result<Vec<&'static str>, AssemblyError> {
    match format.as_deref().unwrap_or("glb") {
        "all" => Ok(FORMATS.to_vec()),
        name => FORMATS
            .iter()
            .find(|f| **f == name)
            .map(|f| vec![*f])
            .ok_or_else(|| AssemblyError::UnknownFormat {
                format: name.to_string(),
            }),
    }
}

fn merge(job: Job) -> Result<(), AssemblyError> {
    println!("🧩 Merging {} parts with OpenCascade...", job.parts.len());
    let payload = serde_json::to_vec(&job).map_err(std::io::Error::other)?;
    run_script("assembly.ts", &payload)?;
    for output in &job.outputs {
        println!(
            "✅ Assembly ({}) -> {}",
            output.format,
            output.path.display()
        );
    }
    Ok(())
}
