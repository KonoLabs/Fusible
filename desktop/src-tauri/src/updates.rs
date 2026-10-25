use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use serde::Serialize;
use tauri_plugin_updater::Update;

pub const RELEASES_URL: &str = "https://github.com/KonoLabs/Fusible/releases/latest";
pub const CHECK_INTERVAL_SECS: i64 = 24 * 3600;
pub const PROGRESS_EVENT: &str = "update-progress";

/// Answer of `check_update`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum UpdateCheck {
    Skipped,
    UpToDate,
    Available {
        version: String,
        notes: String,
        date: Option<String>,
    },
    Unsupported {
        url: String,
    },
}

/// Payload of the `update-progress` event.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Progress {
    pub downloaded: u64,
    pub total: Option<u64>,
}

/// An update that was downloaded and verified but not yet applied (Windows applies it on restart).
pub struct Staged {
    pub update: Update,
    pub bytes: Vec<u8>,
}

/// Update state shared by the commands: the update found, the one downloaded, and the launch attempt.
#[derive(Default)]
pub struct Updates {
    pub pending: Mutex<Option<Update>>,
    pub staged: Mutex<Option<Staged>>,
    pub attempted: AtomicBool,
}

/// Whether this install can update itself: a Linux build must run as an AppImage.
pub fn supported(linux: bool, appimage: bool) -> bool {
    !linux || appimage
}

/// Whether an automatic check is due: never checked, or the last one is a day old or more.
pub fn due(last: Option<i64>, now: i64) -> bool {
    last.is_none_or(|at| now.saturating_sub(at) >= CHECK_INTERVAL_SECS)
}
