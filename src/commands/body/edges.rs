//! Which outline edges a component footprint reaches, shared by notches and the reference kit.

use super::layout::{edge_notch, footprint, signed_area};
use crate::gerber::layers::{PcbComponentElm, PcbPoint};

/// An outline edge that a component footprint reaches.
///
/// `span` is the part of the edge the wall notch cuts: the footprint (with the notch margin)
/// clipped to the edge, as two points on the board edge.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct EdgeContact {
    pub edge: usize,
    pub normal: [f64; 2],
    pub span: [[f64; 2]; 2],
}

/// The outward unit normal of the edge `a` to `b`, or `None` for a zero-length edge.
pub(super) fn edge_normal(a: PcbPoint, b: PcbPoint, winding: f64) -> Option<(f64, f64)> {
    let len = (b.x - a.x).hypot(b.y - a.y);
    (len > 0.0).then(|| ((b.y - a.y) / len * winding, -(b.x - a.x) / len * winding))
}

/// Lists the outline edges `component` reaches, using the same test as the wall notches.
pub(super) fn edge_contacts(component: &PcbComponentElm, outline: &[PcbPoint]) -> Vec<EdgeContact> {
    let Some(corners) = footprint(component) else {
        return Vec::new();
    };
    let winding = signed_area(outline).signum();
    (0..outline.len())
        .filter_map(|edge| {
            let (a, b) = (outline[edge], outline[(edge + 1) % outline.len()]);
            let notch = edge_notch(&corners, a, b, winding, 1.0)?;
            let (nx, ny) = edge_normal(a, b, winding)?;
            Some(EdgeContact {
                edge,
                normal: [nx, ny],
                span: [notch.points[0], notch.points[1]],
            })
        })
        .collect()
}
