//! Watching a board's KiCad file while the studio runs.

use std::path::Path;
use std::time::Duration;

use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};

use super::kicad_sync::{kicad_path, sync_slot};
use super::stamps::Stamps;
use super::state::{Slot, State};

const SETTLE: Duration = Duration::from_millis(600);

/// Starts a KiCad watcher for every board that has a KiCad file.
pub(super) fn start_kicad_watchers(state: &State) -> Vec<Debouncer<RecommendedWatcher>> {
    let watch = |slot| start_kicad_watcher(slot, state);
    state.slots.iter().filter_map(watch).collect()
}

/// Syncs `slot` whenever its KiCad file is saved; `None` when the board has no KiCad file.
///
/// Only events on the board file itself count, so KiCad's lock and backup files and the
/// sync's own writes to the TSX never come back around.
pub(super) fn start_kicad_watcher(
    slot: &Slot,
    state: &State,
) -> Option<Debouncer<RecommendedWatcher>> {
    let pcb = kicad_path(slot)?;
    let file = pcb.file_name()?.to_owned();
    let dir = pcb
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .to_path_buf();
    let (slot, state) = (slot.clone(), state.clone());
    let mut stamps = Stamps::new();
    let mut debouncer = new_debouncer(SETTLE, move |res: DebounceEventResult| {
        let saved = res.is_ok_and(|events| {
            events
                .iter()
                .any(|e| e.path.file_name() == Some(file.as_os_str()) && stamps.is_written(&e.path))
        });
        if saved {
            sync_slot(&slot, &state);
        }
    })
    .ok()?;
    let watched = debouncer.watcher().watch(&dir, RecursiveMode::NonRecursive);
    watched.ok().map(|()| debouncer)
}
