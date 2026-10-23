//! Wall notches and lid vents derived from the board outline.

use super::edges::edge_normal;
use super::settings::Ventilation;
use super::spec::{Bounds, Notch, Vent};
use super::{CLEARANCE, EDGE_TOLERANCE, NOTCH_MARGIN, VENT_INSET};
use crate::gerber::layers::{PcbComponentElm, PcbPoint};

type Corners = [(f64, f64); 4];

/// Notches the wall wherever a component footprint reaches an outline edge, aligned to that edge.
pub(super) fn notches(
    components: &[PcbComponentElm],
    outline: &[PcbPoint],
    wall: f64,
) -> Vec<Notch> {
    let reach = CLEARANCE + wall + 1.0;
    let winding = signed_area(outline).signum();
    let edges: Vec<(PcbPoint, PcbPoint)> = (0..outline.len())
        .map(|i| (outline[i], outline[(i + 1) % outline.len()]))
        .collect();
    components
        .iter()
        .filter_map(footprint)
        .flat_map(|corners| {
            edges
                .iter()
                .filter_map(move |&(a, b)| edge_notch(&corners, a, b, winding, reach))
        })
        .collect()
}

pub(super) fn footprint(c: &PcbComponentElm) -> Option<Corners> {
    let (center, w, h) = (c.center?, c.width?, c.height?);
    let (hw, hh) = (w / 2.0 + NOTCH_MARGIN, h / 2.0 + NOTCH_MARGIN);
    Some(
        [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)].map(|(dx, dy)| (center.x + dx, center.y + dy)),
    )
}

pub(super) fn edge_notch(
    corners: &Corners,
    a: PcbPoint,
    b: PcbPoint,
    winding: f64,
    reach: f64,
) -> Option<Notch> {
    let len = (b.x - a.x).hypot(b.y - a.y);
    if len == 0.0 {
        return None;
    }
    let (ux, uy) = ((b.x - a.x) / len, (b.y - a.y) / len);
    let (nx, ny) = edge_normal(a, b, winding)?;
    let along = corners.map(|(x, y)| (x - a.x) * ux + (y - a.y) * uy);
    let outward = corners.map(|(x, y)| (x - a.x) * nx + (y - a.y) * ny);
    let t0 = along.into_iter().fold(f64::INFINITY, f64::min).max(0.0);
    let t1 = along.into_iter().fold(f64::NEG_INFINITY, f64::max).min(len);
    let reaches = outward.into_iter().fold(f64::NEG_INFINITY, f64::max) >= -EDGE_TOLERANCE;
    let on_board = outward.into_iter().fold(f64::INFINITY, f64::min) < 0.0;
    let at = |t: f64, s: f64| [a.x + ux * t + nx * s, a.y + uy * t + ny * s];
    (t1 > t0 && reaches && on_board).then(|| Notch {
        points: [at(t0, 0.0), at(t1, 0.0), at(t1, reach), at(t0, reach)],
    })
}

pub(super) fn signed_area(polygon: &[PcbPoint]) -> f64 {
    polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .map(|(a, b)| a.x * b.y - b.x * a.y)
        .sum::<f64>()
        / 2.0
}

/// Lays out lid vents in `pattern`, keeping only those fully inside the outline.
pub(super) fn vents(pattern: Ventilation, outline: &[PcbPoint], board: Bounds) -> Vec<Vent> {
    let (w, h, pitch_x, pitch_y, round) = match pattern {
        Ventilation::None => return Vec::new(),
        Ventilation::Grid => (3.0, 3.0, 6.0, 6.0, true),
        Ventilation::Slots => (2.0, 10.0, 5.0, 14.0, false),
    };
    let xs = lattice(
        board.min_x + VENT_INSET,
        board.max_x - VENT_INSET,
        w,
        pitch_x,
    );
    let ys = lattice(
        board.min_y + VENT_INSET,
        board.max_y - VENT_INSET,
        h,
        pitch_y,
    );
    ys.iter()
        .flat_map(|&y| xs.iter().map(move |&x| Vent { x, y, w, h, round }))
        .filter(|v| fits(outline, v))
        .collect()
}

fn fits(outline: &[PcbPoint], v: &Vent) -> bool {
    let (hw, hh) = (v.w / 2.0, v.h / 2.0);
    [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)]
        .iter()
        .all(|&(dx, dy)| contains(outline, v.x + dx, v.y + dy))
}

/// Returns evenly spaced centers for features of `size` that fit inside `[lo, hi]`, centered on the span.
pub(super) fn lattice(lo: f64, hi: f64, size: f64, pitch: f64) -> Vec<f64> {
    let span = hi - lo - size;
    if span < 0.0 {
        return Vec::new();
    }
    let count = (span / pitch).floor() as usize + 1;
    let start = lo + size / 2.0 + (span - (count - 1) as f64 * pitch) / 2.0;
    (0..count).map(|i| start + i as f64 * pitch).collect()
}

fn contains(polygon: &[PcbPoint], x: f64, y: f64) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let (a, b) = (polygon[i], polygon[j]);
        if (a.y > y) != (b.y > y) && x < (b.x - a.x) * (y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
        j = i;
    }
    inside
}
