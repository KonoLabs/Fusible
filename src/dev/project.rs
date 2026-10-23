//! The project tree served at `GET /api/project`: boards, firmware, and bodies.

use std::path::Path;

use serde_json::{json, Value};

use super::state::State;
use crate::config::{BodyConfig, FirmwareConfig, FusibleConfig};

/// Describes the manifest items that belong to the served boards, with each board's fuse state.
///
/// Paths are shown relative to `root`, the directory of the manifest.
pub(super) fn project_json(state: &State, cfg: Option<&FusibleConfig>, root: &Path) -> String {
    let served = |board: Option<&String>| {
        board.is_none_or(|b| state.slots.iter().any(|s| s.name.as_deref() == Some(b)))
    };
    let firmware: Vec<Value> = cfg.map_or_else(Vec::new, |c| {
        let items = c
            .firmware
            .iter()
            .filter(|(_, fw)| served(fw.board.as_ref()));
        items
            .map(|(name, fw)| firmware_item(name, fw, root))
            .collect()
    });
    let bodies: Vec<Value> = cfg.map_or_else(Vec::new, |c| {
        let items = c
            .bodies
            .iter()
            .filter(|(_, body)| served(body.board.as_ref()));
        items
            .map(|(name, body)| body_item(name, body, root))
            .collect()
    });
    json!({
        "name": cfg.and_then(|c| c.name.clone()),
        "boards": board_items(state, root),
        "firmware": firmware,
        "bodies": bodies,
    })
    .to_string()
}

fn board_items(state: &State, root: &Path) -> Vec<Value> {
    state
        .slots
        .iter()
        .map(|slot| {
            let health = slot.health();
            json!({
                "name": slot.name,
                "entry": slot.entry.strip_prefix(root).unwrap_or(&slot.entry),
                "fuse_blown": health.fuse_blown,
                "violations": health.violations,
            })
        })
        .collect()
}

fn firmware_item(name: &str, fw: &FirmwareConfig, root: &Path) -> Value {
    json!({
        "name": name,
        "board": fw.board,
        "path": fw.path.as_deref().map(|p| p.strip_prefix(root).unwrap_or(p)),
        "framework": fw.framework,
        "buildable": fw.build_cmd.is_some(),
    })
}

fn body_item(name: &str, body: &BodyConfig, root: &Path) -> Value {
    let kind = if body.source.is_some() {
        "authored"
    } else {
        "generated"
    };
    let source = body
        .source
        .as_deref()
        .map(|p| p.strip_prefix(root).unwrap_or(p));
    let exists = body.source.as_deref().map(Path::is_file);
    json!({
        "name": name,
        "kind": kind,
        "board": body.board,
        "source": source,
        "exists": exists,
    })
}
