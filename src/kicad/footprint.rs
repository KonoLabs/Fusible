//! Footprint placement extraction from a parsed KiCad board S-expression.

use super::pads::{parse_pad, KicadPad};
use super::sexpr::Sexpr;

#[derive(Debug, Clone, PartialEq)]
pub struct KicadFootprint {
    pub designator: String,
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
    pub layer: String,
    pub pads: Vec<KicadPad>,
}

pub fn extract_footprints(sexpr: &Sexpr) -> Vec<KicadFootprint> {
    let mut footprints = Vec::new();
    walk_sexpr_for_footprints(sexpr, &mut footprints);
    footprints
}

fn walk_sexpr_for_footprints(node: &Sexpr, out: &mut Vec<KicadFootprint>) {
    let Sexpr::List(list) = node else {
        return;
    };
    if matches!(
        list.first().and_then(Sexpr::as_atom),
        Some("footprint" | "module")
    ) {
        out.extend(parse_footprint_node(list));
    }
    for item in list {
        walk_sexpr_for_footprints(item, out);
    }
}

/// Reads a footprint's position, rotation, side and reference; `None` without a reference.
fn parse_footprint_node(list: &[Sexpr]) -> Option<KicadFootprint> {
    let mut designator = None;
    let mut fp = KicadFootprint {
        designator: String::new(),
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        layer: "top".to_string(),
        pads: Vec::new(),
    };

    for sub in list.iter().filter_map(Sexpr::as_list) {
        let Some(tag) = sub.first().and_then(Sexpr::as_atom) else {
            continue;
        };
        match tag {
            "at" => apply_at(&mut fp, sub),
            "layer" => apply_layer(&mut fp, sub),
            "pad" => fp.pads.extend(parse_pad(sub)),
            "property" | "fp_text" => {
                if let Some(reference) = reference_field(tag, sub) {
                    designator = reference;
                }
            }
            _ => {}
        }
    }

    fp.designator = designator?;
    Some(fp)
}

/// Applies an `(at x y [rotation])` node; unparsable numbers read as zero.
fn apply_at(fp: &mut KicadFootprint, sub: &[Sexpr]) {
    let num = |i: usize| sub[i].as_atom().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    if sub.len() >= 3 {
        fp.x = num(1);
        fp.y = num(2);
    }
    if sub.len() >= 4 {
        fp.rotation = num(3);
    }
}

/// Applies a `(layer name)` node: `B.Cu` layers are the bottom, anything else the top.
fn apply_layer(fp: &mut KicadFootprint, sub: &[Sexpr]) {
    if let Some(l) = sub.get(1).and_then(Sexpr::as_atom) {
        fp.layer = if l.contains("B.Cu") { "bottom" } else { "top" }.to_string();
    }
}

/// Returns the reference value of a KiCad 6+ `(property "Reference" ..)` or KiCad 5
/// `(fp_text reference ..)` node, or `None` when the node is some other property.
fn reference_field(tag: &str, sub: &[Sexpr]) -> Option<Option<String>> {
    let key = if tag == "property" {
        "Reference"
    } else {
        "reference"
    };
    (sub.len() >= 3 && sub[1].as_atom() == Some(key)).then(|| sub[2].as_atom().map(str::to_string))
}
