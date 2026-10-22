use std::collections::{BTreeMap, BTreeSet};

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
    let current_map = current_placements(&elements);

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
    let mut seen_designators = BTreeSet::new();

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
            let d_rot = rotation_delta(fp.rotation, cur_rot);

            let is_moved = dx.abs() > 0.1 || dy.abs() > 0.1;
            let is_rotated = d_rot.abs() > 0.5;
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

    diffs.sort_by(|a, b| a.designator.cmp(&b.designator));
    let has_drift = modified_count > 0 || unmatched_count > 0;

    DiffReport {
        diffs,
        modified_count,
        unmatched_count,
        unchanged_count,
        has_drift,
    }
}

/// Collects each placed component's designator, center, rotation and layer by joining `pcb_component` to its `source_component`.
fn current_placements(elements: &[Value]) -> BTreeMap<String, ([f64; 2], f64, String)> {
    let str_field = |el: &Value, key: &str| el.get(key).and_then(Value::as_str).map(str::to_owned);
    let names: BTreeMap<String, String> = elements
        .iter()
        .filter(|el| el.get("type").and_then(Value::as_str) == Some("source_component"))
        .filter_map(|el| {
            Some((
                str_field(el, "source_component_id")?,
                str_field(el, "name")?,
            ))
        })
        .collect();

    let mut placements = BTreeMap::new();
    for el in elements {
        if el.get("type").and_then(Value::as_str) != Some("pcb_component") {
            continue;
        }
        let Some(name) = str_field(el, "source_component_id").and_then(|id| names.get(&id)) else {
            continue;
        };
        let center = el.get("center");
        let coord = |axis: &str| {
            center
                .and_then(|c| c.get(axis))
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
        };
        let rot = el.get("rotation").and_then(Value::as_f64).unwrap_or(0.0);
        let layer = str_field(el, "layer").unwrap_or_else(|| "top".to_string());
        placements
            .entry(name.clone())
            .or_insert(([coord("x"), coord("y")], rot, layer));
    }
    placements
}

/// Returns the signed rotation difference `to - from` normalized into (-180, 180] degrees.
fn rotation_delta(to: f64, from: f64) -> f64 {
    let d = (to - from).rem_euclid(360.0);
    if d > 180.0 {
        d - 360.0
    } else {
        d
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
