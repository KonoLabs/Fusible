//! Keeping the design's placements and the studio's KiCad status current.

use std::path::PathBuf;

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

/// Writes the parts KiCad moved into the board's TSX files, backing each changed file up.
///
/// The slot's file watcher sees the TSX change and rebuilds; that rebuild refreshes the status.
/// When nothing is written the status is refreshed here.
pub(super) fn sync_slot(slot: &Slot, state: &State) {
    let Some(pcb) = kicad_path(slot) else {
        return;
    };
    let name = backup_name(slot);
    let outcome = slot
        .source()
        .ok_or_else(|| "the board has not built yet".to_string())
        .and_then(|source| compare_board(&pcb, &source))
        .and_then(|found| {
            sync_positions(
                &found,
                &watch_dir(&slot.entry),
                state.kicad.backup_dir(),
                &name,
            )
        });
    match outcome {
        Ok(written) if !written.is_empty() => {
            println!("⇄ KiCad: moved {} in the TSX", written.join(", "));
        }
        Ok(_) => evaluate(slot, state),
        Err(message) => {
            eprintln!("⚠ KiCad sync failed: {message}");
            evaluate(slot, state);
        }
    }
}

/// The file stem backups of `slot` are named after.
fn backup_name(slot: &Slot) -> String {
    slot.name.clone().unwrap_or_else(|| {
        slot.entry
            .file_stem()
            .map_or_else(|| "board".into(), |s| s.to_string_lossy().into_owned())
    })
}
