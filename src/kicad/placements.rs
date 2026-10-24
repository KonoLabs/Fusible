//! Current component placements read from Circuit JSON.

use std::collections::BTreeMap;

use serde_json::Value;

use super::sexpr::KicadFootprint;

/// A component's center, rotation in degrees and layer.
pub(super) type Placement = ([f64; 2], f64, String);

/// Collects each placed component's designator, center, rotation and layer by joining
/// `pcb_component` to its `source_component`.
pub(super) fn current_placements(elements: &[Value]) -> BTreeMap<String, Placement> {
    let names = source_names(elements);
    let mut placements = BTreeMap::new();
    for el in elements {
        if el.get("type").and_then(Value::as_str) != Some("pcb_component") {
            continue;
        }
        let Some(name) = str_field(el, "source_component_id").and_then(|id| names.get(&id)) else {
            continue;
        };
        placements
            .entry(name.clone())
            .or_insert_with(|| placement(el));
    }
    placements
}

/// Maps each `source_component_id` to its component name.
fn source_names(elements: &[Value]) -> BTreeMap<String, String> {
    elements
        .iter()
        .filter(|el| el.get("type").and_then(Value::as_str) == Some("source_component"))
        .filter_map(|el| {
            Some((
                str_field(el, "source_component_id")?,
                str_field(el, "name")?,
            ))
        })
        .collect()
}

/// Reads a `pcb_component`'s placement, defaulting to the origin, no rotation and the top.
fn placement(el: &Value) -> Placement {
    let center = el.get("center");
    let coord = |axis: &str| {
        center
            .and_then(|c| c.get(axis))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    };
    let rot = el.get("rotation").and_then(Value::as_f64).unwrap_or(0.0);
    let layer = str_field(el, "layer").unwrap_or_else(|| "top".to_string());
    ([coord("x"), coord("y")], rot, layer)
}

fn str_field(el: &Value, key: &str) -> Option<String> {
    el.get(key).and_then(Value::as_str).map(str::to_owned)
}

/// Returns the signed rotation difference `to - from` normalized into (-180, 180] degrees.
pub(super) fn rotation_delta(to: f64, from: f64) -> f64 {
    let d = (to - from).rem_euclid(360.0);
    if d > 180.0 {
        d - 360.0
    } else {
        d
    }
}

/// Returns the offset aligning the KiCad frame with the Circuit JSON frame, with Y flipped
/// between the two.
///
/// It is the median over the footprints present in both, so a few moved parts do not shift
/// the ones that stayed.
pub(super) fn centroid_offset(
    kicad_fps: &[KicadFootprint],
    current_map: &BTreeMap<String, Placement>,
) -> (f64, f64) {
    let (mut xs, mut ys) = (Vec::new(), Vec::new());
    for fp in kicad_fps {
        if let Some(&(cur_p, _, _)) = current_map.get(&fp.designator) {
            xs.push(fp.x - cur_p[0]);
            ys.push(fp.y + cur_p[1]);
        }
    }
    (median(&mut xs), median(&mut ys))
}

/// Returns the median of `values`, or zero when empty.
fn median(values: &mut [f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(f64::total_cmp);
    let mid = values.len() / 2;
    match values.len() % 2 {
        0 => (values[mid - 1] + values[mid]) / 2.0,
        _ => values[mid],
    }
}
