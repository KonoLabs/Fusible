//! `fusible body kit`: a reference kit to model your own parts around a board.

use std::path::{Path, PathBuf};

use super::kit_data::build_kit;
use super::{fuse_gate, BodyError};
use crate::commands::target::{is_tsx_target, resolve_board};
use crate::config::FusibleConfig;
use crate::tscircuit::Runner;

/// Writes `kit.json` and, when a tscircuit runner is available, `board.step` for one board.
///
/// The board is named like any other target; the kit goes to `bodies/kit/<board>` next to
/// the manifest unless `output` is given. Fuse-gated unless `skip_lint` is set.
pub fn kit(
    board: Option<PathBuf>,
    output: Option<PathBuf>,
    skip_lint: bool,
) -> Result<(), BodyError> {
    let target = resolve_board(board)?;
    let manifest = FusibleConfig::nearest().map(|(path, _)| path);
    let root = manifest.as_deref().and_then(Path::parent);
    let name = target.name.clone().unwrap_or_else(|| "board".into());
    let out_dir = output.unwrap_or_else(|| {
        let base = root.unwrap_or(Path::new(""));
        base.join("bodies").join("kit").join(&name)
    });
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
    export_step(&target.entry, &out_dir.join("board.step"));
    Ok(())
}

fn export_step(entry: &Path, step: &Path) {
    if !is_tsx_target(entry) {
        println!("ℹ️  The board entry is not a tscircuit source: skipped board.step");
        return;
    }
    let Ok(runner) = Runner::detect() else {
        println!("ℹ️  No tscircuit runner found: skipped board.step (kit.json is complete)");
        return;
    };
    match runner.export(entry, "step", Some(step)) {
        Ok(()) => println!("✅ Kit board model -> {}", step.display()),
        Err(err) => println!("⚠️  Could not export board.step: {err}"),
    }
}
