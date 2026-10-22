//! Writes KiCad placement drift back into tscircuit `manual-edits.json`.

use std::path::Path;

use serde_json::{json, Value};

use super::diff::{DiffReport, PlacementDiff};

pub fn apply_kicad_sync(
    report: &DiffReport,
    manual_edits_path: &Path,
    dry_run: bool,
) -> Result<String, String> {
    let mut root = load_manual_edits(manual_edits_path)?;
    let placements = root
        .get_mut("pcb_placements")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| "missing 'pcb_placements' array in manual-edits.json".to_string())?;

    let mut updated_count = 0;
    for diff in &report.diffs {
        if let Some(pos) = diff.kicad_pos {
            upsert_placement(placements, diff, pos);
            updated_count += 1;
        }
    }

    let output =
        serde_json::to_string_pretty(&root).map_err(|e| format!("failed to format JSON: {e}"))?;
    if !dry_run {
        write_manual_edits(manual_edits_path, &output)?;
    }

    Ok(format!(
        "Synced {updated_count} component placement override(s) into {}",
        manual_edits_path.display()
    ))
}

/// Reads `manual-edits.json`, or returns an empty document when the file does not exist.
fn load_manual_edits(path: &Path) -> Result<Value, String> {
    if !path.is_file() {
        return Ok(json!({
            "pcb_placements": [],
            "edit_events": []
        }));
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    serde_json::from_str(&content).map_err(|e| format!("invalid JSON in {}: {e}", path.display()))
}

/// Updates the placement whose selector matches the diff's designator, or appends a new one.
fn upsert_placement(placements: &mut Vec<Value>, diff: &PlacementDiff, pos: [f64; 2]) {
    let selector = &diff.designator;
    let existing = placements
        .iter_mut()
        .find(|p| p.get("selector").and_then(|s| s.as_str()) == Some(selector));
    let Some(p) = existing else {
        placements.push(json!({
            "selector": selector,
            "center": { "x": pos[0], "y": pos[1] },
            "rotation": diff.kicad_rot,
            "layer": diff.kicad_layer
        }));
        return;
    };
    if let Some(center) = p.get_mut("center").and_then(Value::as_object_mut) {
        center.insert("x".to_string(), json!(pos[0]));
        center.insert("y".to_string(), json!(pos[1]));
    } else {
        p["center"] = json!({ "x": pos[0], "y": pos[1] });
    }
    p["rotation"] = json!(diff.kicad_rot);
    p["layer"] = json!(diff.kicad_layer);
}

/// Writes the synced document, creating its parent directory first.
fn write_manual_edits(path: &Path, output: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, output).map_err(|e| e.to_string())
}
