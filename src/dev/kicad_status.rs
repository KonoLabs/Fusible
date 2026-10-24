//! What the studio shows about each board's KiCad file.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::kicad::Comparison;

/// The KiCad state of one board: in sync, differing from the design, or unreadable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct KicadStatus {
    pub board: String,
    pub file: String,
    pub state: &'static str,
    pub differences: Vec<String>,
    pub message: Option<String>,
}

impl KicadStatus {
    /// Builds the status of `board` from the comparison of its KiCad file with the design.
    ///
    /// A part KiCad moved counts as a difference until the design follows it.
    pub(super) fn new(board: &str, file: &str, result: Result<Comparison, String>) -> Self {
        let status = |state, differences, message| Self {
            board: board.to_string(),
            file: file.to_string(),
            state,
            differences,
            message,
        };
        match result {
            Err(message) => status("error", Vec::new(), Some(message)),
            Ok(comparison) => {
                let mut differences = comparison.differences.clone();
                differences.extend(
                    comparison
                        .moved()
                        .iter()
                        .map(|name| format!("{name} placement differs")),
                );
                match differences.is_empty() {
                    true => status("in_sync", differences, None),
                    false => status("differs", differences, None),
                }
            }
        }
    }
}

/// The latest status of every board that has a KiCad file, shared by the handlers, and the
/// folder that receives backups of the files a sync overwrites.
#[derive(Clone)]
pub(super) struct KicadStates {
    all: Arc<Mutex<BTreeMap<String, KicadStatus>>>,
    backup: PathBuf,
}

impl Default for KicadStates {
    fn default() -> Self {
        Self {
            all: Arc::default(),
            backup: PathBuf::from(".fusible/backup"),
        }
    }
}

impl KicadStates {
    /// The folder holding the backups of overwritten files.
    pub(super) fn backup_dir(&self) -> &Path {
        &self.backup
    }

    /// States that keep their backups in `dir`, for tests that must not touch the working directory.
    #[cfg(test)]
    pub(super) fn with_backup(dir: PathBuf) -> Self {
        Self {
            backup: dir,
            ..Self::default()
        }
    }

    /// Stores `status` and returns whether it differs from the one stored before.
    pub(super) fn set(&self, status: KicadStatus) -> bool {
        let Ok(mut all) = self.all.lock() else {
            return false;
        };
        all.insert(status.board.clone(), status.clone()) != Some(status)
    }

    /// The statuses as the JSON array the studio reads.
    pub(super) fn json(&self) -> String {
        let all: Vec<KicadStatus> = self
            .all
            .lock()
            .map(|all| all.values().cloned().collect())
            .unwrap_or_default();
        serde_json::to_string(&all).unwrap_or_else(|_| "[]".into())
    }

    /// The `SYNC` event pushed to every studio client.
    pub(super) fn event(&self) -> String {
        format!(r#"{{"event":"sync","boards":{}}}"#, self.json())
    }
}
