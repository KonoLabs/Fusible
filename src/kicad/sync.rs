use std::path::Path;

use serde_json::{json, Value};

use super::diff::DiffReport;

pub fn apply_kicad_sync(
    report: &DiffReport,
    manual_edits_path: &Path,
    dry_run: bool,
) -> Result<String, String> {
    let mut root: Value = if manual_edits_path.is_file() {
        let content = std::fs::read_to_string(manual_edits_path)
            .map_err(|e| format!("cannot read {}: {e}", manual_edits_path.display()))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("invalid JSON in {}: {e}", manual_edits_path.display()))?
    } else {
        json!({
            "pcb_placements": [],
            "edit_events": []
        })
    };

    let placements = root
        .get_mut("pcb_placements")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| "missing 'pcb_placements' array in manual-edits.json".to_string())?;

    let mut updated_count = 0;
    for diff in &report.diffs {
        if let Some(pos) = diff.kicad_pos {
            let selector = &diff.designator;
            let mut found = false;

            for p in placements.iter_mut() {
                if p.get("selector").and_then(|s| s.as_str()) == Some(selector) {
                    if let Some(center) = p.get_mut("center") {
                        center["x"] = json!(pos[0]);
                        center["y"] = json!(pos[1]);
                    } else {
                        p["center"] = json!({ "x": pos[0], "y": pos[1] });
                    }
                    p["rotation"] = json!(diff.kicad_rot);
                    p["layer"] = json!(diff.kicad_layer);
                    found = true;
                    updated_count += 1;
                    break;
                }
            }

            if !found {
                placements.push(json!({
                    "selector": selector,
                    "center": { "x": pos[0], "y": pos[1] },
                    "rotation": diff.kicad_rot,
                    "layer": diff.kicad_layer
                }));
                updated_count += 1;
            }
        }
    }

    let output =
        serde_json::to_string_pretty(&root).map_err(|e| format!("failed to format JSON: {e}"))?;

    if !dry_run {
        if let Some(parent) = manual_edits_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(manual_edits_path, &output).map_err(|e| e.to_string())?;
    }

    Ok(format!(
        "Synced {updated_count} component placement override(s) into {}",
        manual_edits_path.display()
    ))
}
