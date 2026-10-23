//! Shared dev server state: one slot per served board, plus the hub, bundle, and runs.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::body::BodyCache;
use super::hub::ClientHub;
use super::runs::Runs;
use crate::commands::target::BoardTarget;

/// The latest fuse result of a board, as shown in the project tree.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Health {
    pub fuse_blown: bool,
    pub violations: usize,
}

/// One served board: its manifest name, entry file, latest payload, and body preview cache.
#[derive(Clone)]
pub(super) struct Slot {
    pub name: Option<String>,
    pub entry: PathBuf,
    pub payload: Arc<Mutex<String>>,
    pub health: Arc<Mutex<Health>>,
    pub body: BodyCache,
}

impl Slot {
    pub(super) fn new(board: &BoardTarget) -> Self {
        Self {
            name: board.name.clone(),
            entry: board.entry.clone(),
            payload: Arc::new(Mutex::new(String::new())),
            health: Arc::default(),
            body: BodyCache::default(),
        }
    }

    pub(super) fn payload(&self) -> String {
        self.payload.lock().map(|p| p.clone()).unwrap_or_default()
    }

    pub(super) fn health(&self) -> Health {
        self.health.lock().map(|h| *h).unwrap_or_default()
    }
}

#[derive(Clone)]
pub(super) struct State {
    pub hub: ClientHub,
    pub bundle: Arc<Mutex<Option<String>>>,
    pub slots: Arc<Vec<Slot>>,
    pub runs: Runs,
}

impl State {
    pub(super) fn new(boards: &[BoardTarget]) -> Self {
        Self {
            hub: ClientHub::default(),
            bundle: Arc::default(),
            slots: Arc::new(boards.iter().map(Slot::new).collect()),
            runs: Runs::default(),
        }
    }

    /// Finds the slot for `board`; without a name, the first one.
    pub(super) fn slot(&self, board: Option<&str>) -> Option<&Slot> {
        match board {
            Some(name) => self.slots.iter().find(|s| s.name.as_deref() == Some(name)),
            None => self.slots.first(),
        }
    }
}
