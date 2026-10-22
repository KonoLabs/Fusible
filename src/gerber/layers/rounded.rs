//! Rounded rectangle and polygon pad geometry.

use super::geometry::{axis_aligned, pill_feature, rect_feature, Feature};
use super::math::{add, rotate};
use crate::gerber::apertures::Aperture;

const ARC_STEPS: usize = 8;
const MITER_LIMIT: f64 = 4.0;

/// Flashes an axis-aligned rounded rectangle macro, or fills a rotated one as a region.
pub(super) fn round_rect_feature(
    w: f64,
    h: f64,
    r: f64,
    rot: f64,
    at: [f64; 2],
) -> Result<Feature, String> {
    if r <= 0.0 {
        return rect_feature(w, h, rot, at);
    }
    if 2.0 * r >= w.min(h) {
        return pill_feature(w, h, rot, at);
    }
    if let Some((w, h)) = axis_aligned(w, h, rot) {
        return Ok(Feature::Flash(Aperture::round_rect(w, h, r)?, at));
    }
    Aperture::round_rect(w, h, r)?;
    Ok(Feature::Region(rounded_outline(w, h, r, rot, at)))
}

/// Returns the outline of a rounded rectangle, approximating each corner arc with line segments.
fn rounded_outline(w: f64, h: f64, r: f64, rot: f64, at: [f64; 2]) -> Vec<[f64; 2]> {
    let (cx, cy) = (w / 2.0 - r, h / 2.0 - r);
    let corners = [
        (cx, cy, 0.0),
        (-cx, cy, 90.0),
        (-cx, -cy, 180.0),
        (cx, -cy, 270.0),
    ];
    let mut points = Vec::new();
    for (x, y, start) in corners {
        for step in 0..=ARC_STEPS {
            let angle = start + 90.0 * step as f64 / ARC_STEPS as f64;
            let arc = rotate([r, 0.0], angle);
            points.push(add(at, rotate([x + arc[0], y + arc[1]], rot)));
        }
    }
    points
}

/// Grows a polygon outward by `margin` with mitered corners, rejecting a degenerate result.
pub(super) fn offset_polygon(points: &[[f64; 2]], margin: f64) -> Result<Vec<[f64; 2]>, String> {
    if margin == 0.0 {
        return Ok(points.to_vec());
    }
    let sign = if signed_area(points) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let n = points.len();
    let grown: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            miter(
                points[(i + n - 1) % n],
                points[i],
                points[(i + 1) % n],
                margin * sign,
            )
        })
        .collect();
    if grown.iter().flatten().all(|v| v.is_finite()) {
        Ok(grown)
    } else {
        Err("polygon pad has coincident vertices".to_string())
    }
}

/// Returns the signed area of a polygon; positive when the vertices run counter-clockwise.
fn signed_area(points: &[[f64; 2]]) -> f64 {
    let n = points.len();
    (0..n)
        .map(|i| points[i][0] * points[(i + 1) % n][1] - points[(i + 1) % n][0] * points[i][1])
        .sum::<f64>()
        / 2.0
}

/// Moves vertex `p` outward by `d` along the bisector of its two edges.
fn miter(prev: [f64; 2], p: [f64; 2], next: [f64; 2], d: f64) -> [f64; 2] {
    let n1 = edge_normal(prev, p);
    let n2 = edge_normal(p, next);
    let sum = [n1[0] + n2[0], n1[1] + n2[1]];
    let len_sq = sum[0] * sum[0] + sum[1] * sum[1];
    let scale = (2.0 / len_sq).min(MITER_LIMIT * MITER_LIMIT);
    [p[0] + d * sum[0] * scale, p[1] + d * sum[1] * scale]
}

/// Returns the unit normal pointing to the right of the edge `a` to `b`.
fn edge_normal(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = dx.hypot(dy);
    [dy / len, -dx / len]
}
