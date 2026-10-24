//! Telling a file that was written from one that was only opened.
//!
//! On Linux, merely reading a watched file raises a watch event. A rebuild reads its sources,
//! so reacting to every event would rebuild forever.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const CLOCK_SLACK: Duration = Duration::from_secs(2);

type Stamp = (SystemTime, u64);

/// The modification time and size last seen for each watched file.
pub(super) struct Stamps {
    started: SystemTime,
    seen: HashMap<PathBuf, Stamp>,
}

impl Stamps {
    pub(super) fn new() -> Self {
        Self {
            started: SystemTime::now() - CLOCK_SLACK,
            seen: HashMap::new(),
        }
    }

    /// Whether `path` was written since this was last asked.
    ///
    /// A file never seen before counts as written only if it changed shortly before watching
    /// began or later; file clocks lag the system clock by a few milliseconds.
    /// A file that cannot be read counts as written, so a deletion is not missed.
    pub(super) fn is_written(&mut self, path: &Path) -> bool {
        let Some(stamp) = stamp_of(path) else {
            self.seen.remove(path);
            return true;
        };
        match self.seen.insert(path.to_path_buf(), stamp) {
            Some(previous) => previous != stamp,
            None => stamp.0 >= self.started,
        }
    }
}

fn stamp_of(path: &Path) -> Option<Stamp> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}
