use std::collections::HashMap;

use serde_json::Value;

use super::sexpr::KicadFootprint;

#[derive(Debug, Clone, PartialEq)]
pub enum DiffKind {
    Unchanged,
    Moved { dx: f64, dy: f64 },
    Rotated { d_rot: f64 },
    Flipped { from: String, to: String },
    AddedInKicad,
    MissingInKicad,
}

#[derive(Debug, Clone)]
pub struct PlacementDiff {
    pub designator: String,
    pub current_pos: Option<[f64; 2]>,
    pub current_rot: f64,
    pub current_layer: String,
    pub kicad_pos: Option<[f64; 2]>,
    pub kicad_rot: f64,
    pub kicad_layer: String,
    pub kind: DiffKind,
}

#[derive(Debug, Default)]
pub struct DiffReport {
    pub diffs: Vec<PlacementDiff>,
    pub modified_count: usize,
    pub unmatched_count: usize,
    pub unchanged_count: usize,
    pub has_drift: bool,
}

pub fn compare_kicad_with_circuit(kicad_fps: &[KicadFootprint], circuit_json: &str) -> DiffReport {
    let elements: Vec<Value> = serde_json::from_str(circuit_json).unwrap_or_default();
    let mut current_map: HashMap<String, ([f64; 2], f64, String)> = HashMap::new();

    for el in &elements {
        let typ = el.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if typ == "pcb_smtpad" || typ == "source_component" {
            let name = el
                .get("name")
                .or_else(|| el.get("component_id"))
                .and_then(|v| v.as_str());

            if let Some(des) = name {
                let x = el.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let y = el.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let rot = el.get("rotation").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let layer = el
                    .get("layer")
                    .and_then(|v| v.as_str())
                    .unwrap_or("top")
                    .to_string();

                current_map
                    .entry(des.to_string())
                    .or_insert(([x, y], rot, layer));
            }
        }
    }

    let mut common_count = 0;
    let mut sum_kicad_x = 0.0;
    let mut sum_kicad_y = 0.0;
    let mut sum_cur_x = 0.0;
    let mut sum_cur_y = 0.0;

    for fp in kicad_fps {
        if let Some(&(cur_p, _, _)) = current_map.get(&fp.designator) {
            common_count += 1;
            sum_kicad_x += fp.x;
            sum_kicad_y += fp.y;
            sum_cur_x += cur_p[0];
            sum_cur_y += cur_p[1];
        }
    }

    let (offset_x, offset_y) = if common_count > 0 {
        let avg_k_x = sum_kicad_x / common_count as f64;
        let avg_k_y = sum_kicad_y / common_count as f64;
        let avg_c_x = sum_cur_x / common_count as f64;
        let avg_c_y = sum_cur_y / common_count as f64;
        (avg_k_x - avg_c_x, avg_k_y + avg_c_y)
    } else {
        (0.0, 0.0)
    };

    let mut diffs = Vec::new();
    let mut seen_designators = std::collections::HashSet::new();

    let mut modified_count = 0;
    let mut unmatched_count = 0;
    let mut unchanged_count = 0;

    for fp in kicad_fps {
        seen_designators.insert(fp.designator.clone());
        let norm_x = fp.x - offset_x;
        let norm_y = -(fp.y - offset_y);

        if let Some(&(cur_p, cur_rot, ref cur_layer)) = current_map.get(&fp.designator) {
            let dx = norm_x - cur_p[0];
            let dy = norm_y - cur_p[1];
            let d_rot = (fp.rotation - cur_rot).abs();

            let is_moved = dx.abs() > 0.1 || dy.abs() > 0.1;
            let is_rotated = d_rot > 0.5 && (d_rot - 360.0).abs() > 0.5;
            let is_flipped = !fp.layer.eq_ignore_ascii_case(cur_layer);

            let kind = if is_flipped {
                DiffKind::Flipped {
                    from: cur_layer.clone(),
                    to: fp.layer.clone(),
                }
            } else if is_moved {
                DiffKind::Moved { dx, dy }
            } else if is_rotated {
                DiffKind::Rotated { d_rot }
            } else {
                DiffKind::Unchanged
            };

            if kind == DiffKind::Unchanged {
                unchanged_count += 1;
            } else {
                modified_count += 1;
            }

            diffs.push(PlacementDiff {
                designator: fp.designator.clone(),
                current_pos: Some(cur_p),
                current_rot: cur_rot,
                current_layer: cur_layer.clone(),
                kicad_pos: Some([norm_x, norm_y]),
                kicad_rot: fp.rotation,
                kicad_layer: fp.layer.clone(),
                kind,
            });
        } else {
            unmatched_count += 1;
            diffs.push(PlacementDiff {
                designator: fp.designator.clone(),
                current_pos: None,
                current_rot: 0.0,
                current_layer: String::new(),
                kicad_pos: Some([norm_x, norm_y]),
                kicad_rot: fp.rotation,
                kicad_layer: fp.layer.clone(),
                kind: DiffKind::AddedInKicad,
            });
        }
    }

    for (des, (cur_p, cur_rot, cur_layer)) in &current_map {
        if !seen_designators.contains(des) {
            unmatched_count += 1;
            diffs.push(PlacementDiff {
                designator: des.clone(),
                current_pos: Some(*cur_p),
                current_rot: *cur_rot,
                current_layer: cur_layer.clone(),
                kicad_pos: None,
                kicad_rot: 0.0,
                kicad_layer: String::new(),
                kind: DiffKind::MissingInKicad,
            });
        }
    }

    let has_drift = modified_count > 0 || unmatched_count > 0;

    DiffReport {
        diffs,
        modified_count,
        unmatched_count,
        unchanged_count,
        has_drift,
    }
}

pub fn format_diff_report(report: &DiffReport) -> String {
    let mut out = String::new();
    out.push_str("Component Layout Diff (KiCad <-> tscircuit):\n");

    for d in &report.diffs {
        match &d.kind {
            DiffKind::Moved { dx, dy } => {
                let cur = d.current_pos.unwrap_or([0.0, 0.0]);
                let k = d.kicad_pos.unwrap_or([0.0, 0.0]);
                out.push_str(&format!(
                    "  {} (Moved):\n    x: {:>7.2} mm -> {:>7.2} mm ({:+.2} mm)\n    y: {:>7.2} mm -> {:>7.2} mm ({:+.2} mm)\n",
                    d.designator, cur[0], k[0], dx, cur[1], k[1], dy
                ));
            }
            DiffKind::Rotated { d_rot } => {
                out.push_str(&format!(
                    "  {} (Rotated):\n    rot: {:>5.1}° -> {:>5.1}° ({:+.1}°)\n",
                    d.designator, d.current_rot, d.kicad_rot, d_rot
                ));
            }
            DiffKind::Flipped { from, to } => {
                out.push_str(&format!(
                    "  {} (Flipped):\n    layer: {} -> {}\n",
                    d.designator, from, to
                ));
            }
            DiffKind::AddedInKicad => {
                let k = d.kicad_pos.unwrap_or([0.0, 0.0]);
                out.push_str(&format!(
                    "  {} (Added in KiCad at ({:.2}, {:.2}))\n",
                    d.designator, k[0], k[1]
                ));
            }
            DiffKind::MissingInKicad => {
                out.push_str(&format!("  {} (Missing in KiCad)\n", d.designator));
            }
            DiffKind::Unchanged => {}
        }
    }

    out.push_str(&format!(
        "\nSummary: {} modified, {} unmatched, {} unchanged.\n",
        report.modified_count, report.unmatched_count, report.unchanged_count
    ));

    out
}
