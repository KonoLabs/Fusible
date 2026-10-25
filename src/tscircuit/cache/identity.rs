//! Identity of the runner and the on/off switch of the cache.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::tscircuit::RunnerKind;

const DISABLE_ENV: &str = "FUSIBLE_NO_CACHE";

/// Names the runner so a different or updated `tsci` never reuses an entry.
pub(super) fn identity(kind: &RunnerKind) -> String {
    let (path, daily) = match kind {
        RunnerKind::Local(p) | RunnerKind::Global(p) => (p, 0),
        RunnerKind::Bun(p) => (p, now_secs() / 86_400),
    };
    let meta = std::fs::metadata(path).ok();
    let size = meta.as_ref().map_or(0, std::fs::Metadata::len);
    let stamp = meta
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos());
    format!("{}|{size}|{stamp}|{daily}", kind.description())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

pub(super) fn disabled() -> bool {
    std::env::var_os(DISABLE_ENV).is_some_and(|v| !v.is_empty() && v != "0")
}
