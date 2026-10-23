//! `fusible body kit`: a reference kit to model your own parts around a board.

use std::path::{Path, PathBuf};

use super::kit_data::build_kit;
use super::{fuse_gate, BodyError};
use crate::commands::target::{is_tsx_target, resolve_board, BoardTarget};
use crate::config::FusibleConfig;
use crate::tscircuit::Runner;

/// Writes `kit.json` and, when a tscircuit runner is available, `board.step` for one board.
///
/// The board is named like any other target; the kit goes to `bodies/kit/<board>` next to
/// the manifest unless `output` is given. A board that is a bare file is named after the
/// file. Fuse-gated unless `skip_lint` is set. A runner that is found but fails the
/// `board.step` export is a tool error, raised after `kit.json` is written.
pub fn kit(
    board: Option<PathBuf>,
    output: Option<PathBuf>,
    skip_lint: bool,
) -> Result<(), BodyError> {
    let target = resolve_board(board)?;
    let name = kit_name(&target);
    let out_dir = output.unwrap_or_else(|| default_dir(&name));
    let json = fuse_gate(&target, skip_lint)?;
    let data = build_kit(&json, target.name.clone())?;
    std::fs::create_dir_all(&out_dir)?;
    let path = out_dir.join("kit.json");
    let text = serde_json::to_string_pretty(&data).map_err(std::io::Error::other)?;
    std::fs::write(&path, text + "\n")?;
    println!(
        "✅ Kit {name}: {} holes, {} openings -> {}",
        data.holes.len(),
        data.openings.len(),
        path.display()
    );
    for warning in &data.warnings {
        eprintln!("⚠️  {warning}");
    }
    export_step(&target.entry, &out_dir)
}

/// The manifest board name, or the file stem of a bare circuit file.
fn kit_name(target: &BoardTarget) -> String {
    let stem = target.entry.file_stem().and_then(|s| s.to_str());
    target
        .name
        .clone()
        .or_else(|| stem.map(str::to_string))
        .unwrap_or_else(|| "board".into())
}

fn default_dir(name: &str) -> PathBuf {
    let manifest = FusibleConfig::nearest().map(|(path, _)| path);
    let root = manifest.as_deref().and_then(Path::parent);
    root.unwrap_or(Path::new(""))
        .join("bodies")
        .join("kit")
        .join(name)
}

fn export_step(entry: &Path, out_dir: &Path) -> Result<(), BodyError> {
    if !is_tsx_target(entry) {
        println!("ℹ️  The board entry is not a tscircuit source: skipped board.step");
        return Ok(());
    }
    let Ok(runner) = Runner::detect() else {
        println!("ℹ️  No tscircuit runner found: skipped board.step (kit.json is complete)");
        return Ok(());
    };
    let step = out_dir.join("board.step");
    runner
        .export(entry, "step", Some(&step))
        .map_err(|err| BodyError::KitStepFailed {
            dir: out_dir.display().to_string(),
            reason: err.to_string(),
        })?;
    println!("✅ Kit board model -> {}", step.display());
    Ok(())
}
