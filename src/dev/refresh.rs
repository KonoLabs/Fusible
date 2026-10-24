//! Rebuilding a board's payload when its files change.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use super::assets::load_studio_bundle;
use super::pipeline::execute_pipeline;
use super::state::{Health, Slot, State};

/// Loads the studio bundle: every time in debug builds, so a `bun run build` is picked up,
/// and once in release builds, where it is embedded and never changes.
pub(super) fn load_bundle(state: &State) {
    if let Ok(mut lock) = state.bundle.lock() {
        if cfg!(debug_assertions) || lock.is_none() {
            *lock = load_studio_bundle();
        }
    }
}

/// Tells the user on stderr when this build has no studio bundle to serve.
pub(super) fn warn_if_bundle_missing(state: &State) {
    if state.bundle.lock().is_ok_and(|bundle| bundle.is_none()) {
        eprintln!(
            "⚠ This fusible build has no studio bundle: schematic, PCB and 3D fall back to native previews. \
Install a GitHub release binary, or run `bun install && bun run build` in studio/ and rebuild."
        );
    }
}

/// Rebuilds `slot`, stores the result as one snapshot, and broadcasts it.
///
/// The snapshot is stored before the broadcast, so a client that subscribes in between
/// reads the new payload and at worst receives it twice.
pub(super) fn refresh(slot: &Slot, state: &State) {
    let start = std::time::Instant::now();
    let (payload, source) = execute_pipeline(&slot.entry);
    let shared: Option<Arc<str>> = source.as_deref().map(Arc::from);
    let health = Health {
        fuse_blown: payload.fuse_blown,
        violations: payload.diagnostics.len(),
    };
    slot.body
        .update(source, payload.revision.clone(), slot.name.clone());
    if let Ok(json_str) = serde_json::to_string(&payload) {
        slot.publish(json_str.clone(), health, shared);
        state.hub.broadcast(&json_str);
        println!("✓ Circuit updated in {:.1}s", start.elapsed().as_secs_f32());
    }
    load_bundle(state);
}

/// The directory to watch for changes to `entry`.
pub(super) fn watch_dir(entry: &Path) -> PathBuf {
    if entry.is_dir() {
        return entry.to_path_buf();
    }
    entry
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

/// Coalesces file events so at most one rebuild of `slot` runs at a time and none is lost.
pub(super) fn rebuild_trigger(slot: Slot, state: State) -> impl FnMut() + Send + 'static {
    let rebuilding = Arc::new(AtomicBool::new(false));
    let pending = Arc::new(AtomicBool::new(false));
    move || {
        pending.store(true, Ordering::SeqCst);
        if rebuilding.swap(true, Ordering::SeqCst) {
            return;
        }
        let (slot, state) = (slot.clone(), state.clone());
        let (rebuilding, pending) = (rebuilding.clone(), pending.clone());
        thread::spawn(move || loop {
            while pending.swap(false, Ordering::SeqCst) {
                println!("🔄 File change detected — rebuilding circuit preview...");
                refresh(&slot, &state);
            }
            rebuilding.store(false, Ordering::SeqCst);
            if !pending.load(Ordering::SeqCst) || rebuilding.swap(true, Ordering::SeqCst) {
                break;
            }
        });
    }
}
