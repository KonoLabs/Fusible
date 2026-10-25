//! Subcommand: `fusible assembly` — place boards and bodies in one frame and export one model.
//!
//! Each generated body is rebuilt as STEP and each authored body is read from its `source:`.
//! The cross-world rules (`body_interference`, `hole_misaligned`, `connector_blocked`) then
//! measure the bodies against probes built from the boards' Circuit JSON, and write
//! `check.json` and `preview.glb` next to the parts. When the fuse holds, each board is
//! exported as STEP by the tscircuit runner and an embedded occt-wasm script merges every
//! part, placed in Rust, into `assembly.glb` and/or `assembly.step`. Every board involved is
//! fuse-gated first.

mod atomic;
mod error;
mod gate;
mod measure;
mod parts;
pub mod place;
mod probe;
mod probe_holes;
mod probe_warnings;
mod rules;
mod rules_hole;
mod rules_validate;
mod shrink;

#[cfg(test)]
mod atomic_tests;
#[cfg(test)]
mod place_tests;
#[cfg(test)]
mod probe_hole_tests;
#[cfg(test)]
pub(crate) mod probe_tests;
#[cfg(test)]
mod rules_hole_tests;
#[cfg(test)]
mod rules_support;
#[cfg(test)]
mod rules_tests;
#[cfg(test)]
mod rules_validate_tests;

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::commands::body::{generator_dir, run_script};
use crate::commands::check::CheckError;
use crate::config::FusibleConfig;
use crate::linter::FuseBlown;

pub use error::AssemblyError;
use gate::{fuse_gate, load_circuits};
use measure::{clearance, remove_stale, report_clearance};
use parts::{board_frames, board_parts, body_parts, Context, Part};
use rules::Violation;

const FORMATS: &[&str] = &["glb", "step"];

#[derive(Debug, Serialize)]
struct Output {
    format: &'static str,
    path: PathBuf,
}

#[derive(Debug, Serialize)]
struct Job {
    mode: &'static str,
    parts: Vec<Part>,
    outputs: Vec<Output>,
}

/// Builds the `assembly:` of the manifest into `<out_dir>/assembly.<format>`.
///
/// `format` and `output` override the manifest's `format` and `out_dir`. Every board in the
/// assembly, and every board a generated body is fitted to, is linted before any file is
/// written unless `skip_lint` is set. Unless `skip_lint` is set, the bodies are then measured
/// against the boards: `check.json` and `preview.glb` land in `<out_dir>/assembly/`, and a
/// blown fuse stops the run before any `assembly.*` file is written.
pub fn run(
    format: Option<String>,
    output: Option<PathBuf>,
    skip_lint: bool,
) -> Result<(), AssemblyError> {
    let (manifest, cfg) = FusibleConfig::nearest().ok_or(AssemblyError::Missing)?;
    let assembly = cfg.assembly.clone().ok_or(AssemblyError::Missing)?;
    let root = manifest.parent().unwrap_or(Path::new("."));
    let formats = formats(format.or(assembly.format.clone()))?;
    let out_dir = out_dir(output.or(assembly.out_dir.clone()), root)?;
    let parts_dir = out_dir.join("assembly");
    remove_stale(&parts_dir, &out_dir);
    let circuits = fuse_gate(&cfg, &assembly, skip_lint)?;
    let frames = board_frames(&cfg, &assembly);
    let ctx = Context {
        cfg: &cfg,
        root,
        parts_dir: &parts_dir,
        circuits: &circuits,
        frames: &frames,
    };
    let bodies = body_parts(&ctx, &assembly)?;
    if !skip_lint {
        report_clearance(&ctx, &bodies, &parts_dir)?;
    }
    let mut parts = board_parts(&ctx, &assembly)?;
    parts.extend(bodies);
    let outputs = outputs(&formats, &out_dir);
    merge(Job {
        mode: "merge",
        parts,
        outputs,
    })
}

fn out_dir(chosen: Option<PathBuf>, root: &Path) -> std::io::Result<PathBuf> {
    std::path::absolute(chosen.unwrap_or_else(|| root.join("dist")))
}

fn outputs(formats: &[&'static str], out_dir: &Path) -> Vec<Output> {
    formats
        .iter()
        .map(|&format| Output {
            format,
            path: out_dir.join(format!("assembly.{format}")),
        })
        .collect()
}

/// Checks the cross-world rules of the manifest's `assembly:` without writing any output.
///
/// Used by `fusible check` once every board passed. A manifest without `assembly:` passes.
pub fn verify(cfg: &FusibleConfig, root: &Path) -> Result<(), AssemblyError> {
    let Some(assembly) = &cfg.assembly else {
        return Ok(());
    };
    let circuits = load_circuits(cfg, assembly)?;
    let scratch = generator_dir().join(format!("check-{}", std::process::id()));
    let frames = board_frames(cfg, assembly);
    let ctx = Context {
        cfg,
        root,
        parts_dir: &scratch,
        circuits: &circuits,
        frames: &frames,
    };
    let found = body_parts(&ctx, assembly).and_then(|bodies| clearance(&ctx, &bodies, None));
    let _ = std::fs::remove_dir_all(&scratch);
    blow_if_any(&found?)
}

fn blow_if_any(found: &[Violation]) -> Result<(), AssemblyError> {
    if found.is_empty() {
        return Ok(());
    }
    let diagnostics = found.iter().map(Violation::diagnostic).collect();
    Err(CheckError::FuseBlown(FuseBlown::new(diagnostics)).into())
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
