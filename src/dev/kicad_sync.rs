//! Keeping the design's placements and the studio's KiCad status current.

use std::path::{Path, PathBuf};

use super::kicad_status::KicadStatus;
use super::refresh::watch_dir;
use super::state::{Slot, State};
use crate::kicad::{board_file, compare_board, sync_positions};

/// The KiCad board file of `slot`, from the manifest or found next to its entry.
pub(super) fn kicad_path(slot: &Slot) -> Option<PathBuf> {
    board_file(slot.kicad.as_deref(), &slot.entry)
}

/// Compares the KiCad file with the latest build and tells the studio when the result changed.
pub(super) fn evaluate(slot: &Slot, state: &State) {
    let Some(pcb) = kicad_path(slot) else {
        return;
    };
    let result = slot
        .source()
        .ok_or_else(|| "the board has not built yet".to_string())
        .and_then(|source| compare_board(&pcb, &source));
    let board = slot.name.clone().unwrap_or_default();
    let status = KicadStatus::new(&board, &pcb.display().to_string(), result);
    if state.kicad.set(status) {
        state.hub.broadcast(&state.kicad.event());
    }
}

/// Writes the parts KiCad moved into the board's `manual-edits.json`, backing the old file up.
///
/// The edits file sits beside the entry, where the file watcher sees the write and rebuilds;
/// that rebuild refreshes the status. Without a write the status is refreshed here.
pub(super) fn sync_slot(slot: &Slot, state: &State) {
    let Some(pcb) = kicad_path(slot) else {
        return;
    };
    let edits = edits_path(&slot.entry);
    let name = backup_name(slot);
    let outcome = slot
        .source()
        .ok_or_else(|| "the board has not built yet".to_string())
        .and_then(|source| compare_board(&pcb, &source))
        .and_then(|found| sync_positions(&found, &edits, state.kicad.backup_dir(), &name));
    match outcome {
        Ok(moved) if !moved.is_empty() => {
            println!(
                "⇄ KiCad: synced {} into {}",
                moved.join(", "),
                edits.display()
            );
        }
        Ok(_) => evaluate(slot, state),
        Err(message) => {
            eprintln!("⚠ KiCad sync failed: {message}");
            evaluate(slot, state);
        }
    }
}

/// The `manual-edits.json` beside `entry`, which the board's TSX imports.
fn edits_path(entry: &Path) -> PathBuf {
    watch_dir(entry).join("manual-edits.json")
}

/// The file stem backups of `slot` are named after.
fn backup_name(slot: &Slot) -> String {
    slot.name.clone().unwrap_or_else(|| {
        slot.entry
            .file_stem()
            .map_or_else(|| "board".into(), |s| s.to_string_lossy().into_owned())
    })
}
