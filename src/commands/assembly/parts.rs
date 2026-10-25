//! The parts of the assembly: each board or body as a model file and its placement.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::gate::Circuits;
use super::place::{compose, placement, translation, Matrix, IDENTITY};
use super::AssemblyError;
use crate::commands::body::{build_authored, generate_step};
use crate::commands::target::BoardTarget;
use crate::config::{AssemblyConfig, BodyConfig, FusibleConfig, PlacementConfig};
use crate::tscircuit::Runner;

/// One model file of the assembly and the transform that places it.
#[derive(Debug, Serialize)]
pub(super) struct Part {
    pub name: String,
    kind: &'static str,
    input: PathBuf,
    format: &'static str,
    matrix: Matrix,
}

/// The placement of each board that is a part of the assembly, keyed by board name.
pub(super) type Frames = BTreeMap<String, Matrix>;

/// What the parts are built from: the manifest, the linted circuits, and where files go.
pub(super) struct Context<'a> {
    pub cfg: &'a FusibleConfig,
    pub root: &'a Path,
    pub parts_dir: &'a Path,
    pub circuits: &'a Circuits,
    pub frames: &'a Frames,
}

fn is_board(cfg: &FusibleConfig, name: &str) -> bool {
    cfg.boards.contains_key(name)
}

/// The placement of each board part, so a body can follow the frame of the board it is modeled on.
pub(super) fn board_frames(cfg: &FusibleConfig, assembly: &AssemblyConfig) -> Frames {
    assembly
        .parts
        .iter()
        .filter(|(name, _)| is_board(cfg, name))
        .map(|(name, place)| (name.clone(), placement(place)))
        .collect()
}

/// Writes the model of every generated body into `parts_dir` and places each body part.
pub(super) fn body_parts(
    ctx: &Context,
    assembly: &AssemblyConfig,
) -> Result<Vec<Part>, AssemblyError> {
    std::fs::create_dir_all(ctx.parts_dir)?;
    assembly
        .parts
        .iter()
        .filter(|(name, _)| !is_board(ctx.cfg, name))
        .map(|(name, place)| body_part(ctx, name, &ctx.cfg.bodies[name], place))
        .collect()
}

/// Exports the STEP model of every board part into `parts_dir` and places each board part.
pub(super) fn board_parts(
    ctx: &Context,
    assembly: &AssemblyConfig,
) -> Result<Vec<Part>, AssemblyError> {
    assembly
        .parts
        .keys()
        .filter(|name| is_board(ctx.cfg, name))
        .map(|name| board_part(&ctx.circuits[name].0, name, ctx.parts_dir, ctx.frames[name]))
        .collect()
}

fn board_part(
    board: &BoardTarget,
    name: &str,
    parts_dir: &Path,
    matrix: Matrix,
) -> Result<Part, AssemblyError> {
    let failed = |reason: String| AssemblyError::BoardModelFailed {
        name: name.to_string(),
        reason,
    };
    let runner = Runner::detect().map_err(|err| failed(err.to_string()))?;
    let input = parts_dir.join(format!("{name}.step"));
    println!("🧊 Exporting board `{name}` as STEP...");
    runner
        .export(&board.entry, "step", Some(&input))
        .map_err(|err| failed(err.to_string()))?;
    Ok(Part {
        name: name.to_string(),
        kind: "board",
        input,
        format: "step",
        matrix,
    })
}

fn body_part(
    ctx: &Context,
    name: &str,
    body: &BodyConfig,
    place: &PlacementConfig,
) -> Result<Part, AssemblyError> {
    let board = ctx.cfg.body_board(body);
    let frame = board
        .and_then(|b| ctx.frames.get(b))
        .copied()
        .unwrap_or(IDENTITY);
    let (input, format, offset) = match &body.source {
        Some(source) => {
            build_authored(name, body, ctx.root)?;
            (source.clone(), source_format(name, source)?, IDENTITY)
        }
        None => {
            let board = board.expect("a generated body always has a board");
            let input = ctx.parts_dir.join(format!("{name}.step"));
            println!("🧊 Building body `{name}` as STEP...");
            let circuit = &ctx.circuits[board].1;
            let mid = generate_step(circuit, body.enclosure.clone(), input.clone())?;
            (input, "step", translation([0.0, 0.0, -mid]))
        }
    };
    Ok(Part {
        name: name.to_string(),
        kind: "body",
        input,
        format,
        matrix: compose(&frame, &compose(&placement(place), &offset)),
    })
}

fn source_format(name: &str, path: &Path) -> Result<&'static str, AssemblyError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("step" | "stp") => Ok("step"),
        Some("stl") => Ok("stl"),
        _ => Err(AssemblyError::UnsupportedSource {
            name: name.to_string(),
            path: path.to_path_buf(),
        }),
    }
}
