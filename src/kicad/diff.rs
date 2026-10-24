//! Placement drift between a KiCad board and the tscircuit Circuit JSON.

use std::collections::BTreeSet;

use serde_json::Value;

pub use super::report::format_diff_report;

use super::placements::{centroid_offset, current_placements, rotation_delta, Placement};
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

impl PlacementDiff {
    /// Whether the part exists on both sides and its placement differs.
    pub fn is_placement_change(&self) -> bool {
        matches!(
            self.kind,
            DiffKind::Moved { .. } | DiffKind::Rotated { .. } | DiffKind::Flipped { .. }
        )
    }
}

#[derive(Debug, Default)]
pub struct DiffReport {
    pub diffs: Vec<PlacementDiff>,
    pub modified_count: usize,
    pub unmatched_count: usize,
    pub unchanged_count: usize,
    pub has_drift: bool,
}

impl DiffReport {
    /// Adds a diff and bumps the counter matching its kind.
    fn record(&mut self, diff: PlacementDiff) {
        match diff.kind {
            DiffKind::Unchanged => self.unchanged_count += 1,
            DiffKind::AddedInKicad | DiffKind::MissingInKicad => self.unmatched_count += 1,
            _ => self.modified_count += 1,
        }
        self.diffs.push(diff);
    }
}

/// Parses Circuit JSON text into its element list.
pub fn parse_circuit_elements(circuit_json: &str) -> Result<Vec<Value>, serde_json::Error> {
    serde_json::from_str(circuit_json)
}

pub fn compare_kicad_with_circuit(kicad_fps: &[KicadFootprint], elements: &[Value]) -> DiffReport {
    let current_map = current_placements(elements);
    let offset = centroid_offset(kicad_fps, &current_map);

    let mut report = DiffReport::default();
    let mut seen_designators = BTreeSet::new();
    for fp in kicad_fps {
        seen_designators.insert(fp.designator.clone());
        report.record(kicad_diff(fp, offset, current_map.get(&fp.designator)));
    }
    for (des, current) in &current_map {
        if !seen_designators.contains(des) {
            report.record(missing_diff(des, current));
        }
    }

    report.diffs.sort_by(|a, b| a.designator.cmp(&b.designator));
    report.has_drift = report.modified_count > 0 || report.unmatched_count > 0;
    report
}

/// Compares one KiCad footprint, normalized into the Circuit JSON frame, with its placement.
fn kicad_diff(
    fp: &KicadFootprint,
    offset: (f64, f64),
    current: Option<&Placement>,
) -> PlacementDiff {
    let norm = [fp.x - offset.0, -(fp.y - offset.1)];
    let mut diff = PlacementDiff {
        designator: fp.designator.clone(),
        current_pos: None,
        current_rot: 0.0,
        current_layer: String::new(),
        kicad_pos: Some(norm),
        kicad_rot: fp.rotation,
        kicad_layer: fp.layer.clone(),
        kind: DiffKind::AddedInKicad,
    };
    if let Some((cur_p, cur_rot, cur_layer)) = current {
        diff.kind = classify(
            fp,
            [norm[0] - cur_p[0], norm[1] - cur_p[1]],
            *cur_rot,
            cur_layer,
        );
        diff.current_pos = Some(*cur_p);
        diff.current_rot = *cur_rot;
        diff.current_layer = cur_layer.clone();
    }
    diff
}

/// Classifies a matched footprint: a side flip wins over a move, which wins over a rotation.
fn classify(fp: &KicadFootprint, delta: [f64; 2], cur_rot: f64, cur_layer: &str) -> DiffKind {
    let [dx, dy] = delta;
    let d_rot = rotation_delta(fp.rotation, cur_rot);
    if !fp.layer.eq_ignore_ascii_case(cur_layer) {
        return DiffKind::Flipped {
            from: cur_layer.to_string(),
            to: fp.layer.clone(),
        };
    }
    if dx.abs() > 0.1 || dy.abs() > 0.1 {
        return DiffKind::Moved { dx, dy };
    }
    if d_rot.abs() > 0.5 {
        return DiffKind::Rotated { d_rot };
    }
    DiffKind::Unchanged
}

/// Returns the diff of a Circuit JSON component with no KiCad footprint.
fn missing_diff(designator: &str, current: &Placement) -> PlacementDiff {
    let (cur_p, cur_rot, cur_layer) = current;
    PlacementDiff {
        designator: designator.to_string(),
        current_pos: Some(*cur_p),
        current_rot: *cur_rot,
        current_layer: cur_layer.clone(),
        kicad_pos: None,
        kicad_rot: 0.0,
        kicad_layer: String::new(),
        kind: DiffKind::MissingInKicad,
    }
}
