//! The parts of the assembly: each board or body as a model file and its placement.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::gate::Circuits;
use super::place::{compose, placement, translation, Matrix, IDENTITY};
use super::AssemblyError;
use crate::commands::body::{build_authored, generate_step};
use crate::config::{AssemblyConfig, BodyConfig, FusibleConfig, PlacementConfig};
use crate::tscircuit::Runner;

/// One model file of the assembly and the transform that places it.
#[derive(Debug, Serialize)]
pub(super) struct Part {
    name: String,
    kind: &'static str,
    input: PathBuf,
    format: &'static str,
    matrix: Matrix,
}

struct Context<'a> {
    cfg: &'a FusibleConfig,
    root: &'a Path,
    parts_dir: &'a Path,
    circuits: &'a Circuits,
    frames: BTreeMap<String, Matrix>,
}

/// Writes the model of every board and generated body into `parts_dir` and places each part.
///
/// Boards come first, so a body can follow the frame of the board it is modeled on.
pub(super) fn collect_parts(
    cfg: &FusibleConfig,
    assembly: &AssemblyConfig,
    circuits: &Circuits,
    root: &Path,
    parts_dir: &Path,
) -> Result<Vec<Part>, AssemblyError> {
    std::fs::create_dir_all(parts_dir)?;
    let mut ctx = Context {
        cfg,
        root,
        parts_dir,
        circuits,
        frames: BTreeMap::new(),
    };
    let (boards, bodies): (Vec<_>, Vec<_>) = assembly
        .parts
        .iter()
        .partition(|(name, _)| cfg.boards.contains_key(*name));
    let mut parts = Vec::new();
    for (name, place) in boards {
        parts.push(board_part(&mut ctx, name, place)?);
    }
    for (name, place) in bodies {
        parts.push(body_part(&ctx, name, &cfg.bodies[name], place)?);
    }
    Ok(parts)
}

fn board_part(
    ctx: &mut Context,
    name: &str,
    place: &PlacementConfig,
) -> Result<Part, AssemblyError> {
    let (board, _) = &ctx.circuits[name];
    let failed = |reason: String| AssemblyError::BoardModelFailed {
        name: name.to_string(),
        reason,
    };
    let runner = Runner::detect().map_err(|err| failed(err.to_string()))?;
    let input = ctx.parts_dir.join(format!("{name}.step"));
    println!("🧊 Exporting board `{name}` as STEP...");
    runner
        .export(&board.entry, "step", Some(&input))
        .map_err(|err| failed(err.to_string()))?;
    let matrix = placement(place);
    ctx.frames.insert(name.to_string(), matrix);
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
