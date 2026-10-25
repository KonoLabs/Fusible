//! Shrinking probes by [`INSET`], so parts that only touch never overlap.

use super::rules::INSET;

fn signed_area(points: &[[f64; 2]]) -> f64 {
    let n = points.len();
    (0..n)
        .map(|i| {
            let ([x0, y0], [x1, y1]) = (points[i], points[(i + 1) % n]);
            x0 * y1 - x1 * y0
        })
        .sum::<f64>()
        / 2.0
}

fn unit_normal(from: [f64; 2], to: [f64; 2], ccw: bool) -> Option<[f64; 2]> {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = dx.hypot(dy);
    let sign = if ccw { 1.0 } else { -1.0 };
    (len > 1e-9).then(|| [-dy / len * sign, dx / len * sign])
}

/// Moves every edge of `points` inward by `distance`, or `None` when the polygon would collapse.
fn inset_polygon(points: &[[f64; 2]], distance: f64) -> Option<Vec<[f64; 2]>> {
    let n = points.len();
    let area = signed_area(points);
    if n < 3 || area == 0.0 {
        return None;
    }
    let ccw = area > 0.0;
    let mut moved = Vec::with_capacity(n);
    for i in 0..n {
        let (prev, here, next) = (points[(i + n - 1) % n], points[i], points[(i + 1) % n]);
        let (a, b) = (unit_normal(prev, here, ccw)?, unit_normal(here, next, ccw)?);
        let bend = 1.0 + a[0] * b[0] + a[1] * b[1];
        if bend < 1e-6 {
            return None;
        }
        let scale = distance / bend;
        moved.push([
            here[0] + (a[0] + b[0]) * scale,
            here[1] + (a[1] + b[1]) * scale,
        ]);
    }
    let shrunk = signed_area(&moved);
    (shrunk * area > 0.0 && shrunk.abs() < area.abs()).then_some(moved)
}

/// The polygon moved inward by [`INSET`]; a polygon too thin to shrink is returned as it is.
pub(super) fn shrunk_polygon(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    inset_polygon(points, INSET).unwrap_or_else(|| points.to_vec())
}

/// The span `z0..z1` shrunk by [`INSET`] at both ends, at most a quarter of its length each.
pub(super) fn shrunk_span(z0: f64, z1: f64) -> (f64, f64) {
    let inset = INSET.min((z1 - z0) / 4.0);
    (z0 + inset, z1 - inset)
}

/// The corners of a box shrunk by [`INSET`] on every face, at most a quarter of each size.
pub(super) fn shrunk_corners(min: [f64; 3], max: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let inset = |i: usize| INSET.min((max[i] - min[i]) / 4.0);
    (
        [0, 1, 2].map(|i| min[i] + inset(i)),
        [0, 1, 2].map(|i| max[i] - inset(i)),
    )
}
