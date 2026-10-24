//! Shared dev server state: one slot per served board, plus the hub, bundle, and runs.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::body::BodyCache;
use crate::commands::target::BoardTarget;

pub(super) use super::shared::State;

/// The latest fuse result of a board, as shown in the project tree.
///
/// Until the first build finishes the fuse counts as blown, so an unchecked board never
/// shows as intact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Health {
    pub fuse_blown: bool,
    pub violations: usize,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            fuse_blown: true,
            violations: 0,
        }
    }
}

/// What the latest rebuild of a board produced, kept together so readers see one build.
#[derive(Default)]
struct Snapshot {
    payload: String,
    health: Health,
    source: Option<Arc<str>>,
}

/// One served board: its manifest name, entry file, latest payload, and body preview cache.
#[derive(Clone)]
pub(super) struct Slot {
    pub name: Option<String>,
    pub entry: PathBuf,
    pub kicad: Option<PathBuf>,
    pub body: BodyCache,
    latest: Arc<Mutex<Snapshot>>,
}

impl Slot {
    pub(super) fn new(board: &BoardTarget) -> Self {
        Self {
            name: board.name.clone(),
            entry: board.entry.clone(),
            kicad: board.kicad.clone(),
            body: BodyCache::default(),
            latest: Arc::default(),
        }
    }

    /// Stores the payload, fuse result, and Circuit JSON source of one rebuild together.
    ///
    /// The source is `None` when the board failed to load.
    pub(super) fn publish(&self, payload: String, health: Health, source: Option<Arc<str>>) {
        if let Ok(mut latest) = self.latest.lock() {
            *latest = Snapshot {
                payload,
                health,
                source,
            };
        }
    }

    /// Records only the Circuit JSON source, for tests that skip the pipeline.
    #[cfg(test)]
    pub(super) fn set_source(&self, source: Option<&str>) {
        if let Ok(mut latest) = self.latest.lock() {
            latest.source = source.map(Arc::from);
        }
    }

    /// The Circuit JSON source of the latest rebuild.
    pub(super) fn source(&self) -> Option<Arc<str>> {
        self.latest.lock().ok().and_then(|l| l.source.clone())
    }

    pub(super) fn payload(&self) -> String {
        self.latest
            .lock()
            .map(|l| l.payload.clone())
            .unwrap_or_default()
    }

    pub(super) fn health(&self) -> Health {
        self.latest.lock().map(|l| l.health).unwrap_or_default()
    }

    /// Whether the first build of this board has finished.
    pub(super) fn is_ready(&self) -> bool {
        self.latest.lock().is_ok_and(|l| !l.payload.is_empty())
    }
}
