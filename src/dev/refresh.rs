//! Rebuilding a board's payload when its files change.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use super::assets::load_studio_bundle;
use super::pipeline::execute_pipeline;
use super::state::{Health, Slot, State};

/// Rebuilds the payload of `slot`, records its source, health, and broadcasts it.
pub(super) fn refresh(slot: &Slot, state: &State) {
    let start = std::time::Instant::now();
    let (payload, source) = execute_pipeline(&slot.entry);
    slot.set_source(source.as_deref());
    slot.body
        .update(source, payload.revision.clone(), slot.name.clone());
    if let Ok(mut health) = slot.health.lock() {
        *health = Health {
            fuse_blown: payload.fuse_blown,
            violations: payload.diagnostics.len(),
        };
    }
    if let Ok(json_str) = serde_json::to_string(&payload) {
        if let Ok(mut lock) = slot.payload.lock() {
            *lock = json_str.clone();
        }
        state.hub.broadcast(&json_str);
        println!("✓ Circuit updated in {:.1}s", start.elapsed().as_secs_f32());
    }
    if let Ok(mut lock) = state.bundle.lock() {
        *lock = load_studio_bundle();
    }
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
