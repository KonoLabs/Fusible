//! Planar distance and containment primitives for clearance checks.

use rstar::AABB;

/// Shortest distance between segments `a1-a2` and `b1-b2` (zero when they cross).
pub fn segment_distance(a1: [f64; 2], a2: [f64; 2], b1: [f64; 2], b2: [f64; 2]) -> f64 {
    let d1 = cross(sub(b2, b1), sub(a1, b1));
    let d2 = cross(sub(b2, b1), sub(a2, b1));
    let d3 = cross(sub(a2, a1), sub(b1, a1));
    let d4 = cross(sub(a2, a1), sub(b2, a1));
    if d1 * d2 < 0.0 && d3 * d4 < 0.0 {
        return 0.0;
    }
    point_segment_distance(a1, b1, b2)
        .min(point_segment_distance(a2, b1, b2))
        .min(point_segment_distance(b1, a1, a2))
        .min(point_segment_distance(b2, a1, a2))
}

pub(super) fn point_segment_distance(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = sub(b, a);
    let len2 = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if len2 == 0.0 {
        0.0
    } else {
        ((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / len2
    }
    .clamp(0.0, 1.0);
    let q = [a[0] + t * ab[0], a[1] + t * ab[1]];
    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt()
}

pub(super) fn segment_rect_distance(p1: [f64; 2], p2: [f64; 2], rect: &AABB<[f64; 2]>) -> f64 {
    if segment_intersects_aabb(p1, p2, rect) {
        return 0.0;
    }
    let [lx, ly] = rect.lower();
    let [ux, uy] = rect.upper();
    let corners = [[lx, ly], [ux, ly], [ux, uy], [lx, uy]];
    (0..4)
        .map(|i| segment_distance(p1, p2, corners[i], corners[(i + 1) % 4]))
        .fold(f64::INFINITY, f64::min)
}

fn segment_intersects_aabb(p1: [f64; 2], p2: [f64; 2], rect: &AABB<[f64; 2]>) -> bool {
    let lower = rect.lower();
    let upper = rect.upper();
    let mut range = (0.0f64, 1.0f64);
    for i in 0..2 {
        match clip_slab(p1[i], p2[i] - p1[i], [lower[i], upper[i]], range) {
            Some(clipped) => range = clipped,
            None => return false,
        }
    }
    true
}

/// Clips the segment parameter range `(tmin, tmax)` to the slab
/// `bounds[0]..=bounds[1]` along one axis, or `None` when it misses the slab.
fn clip_slab(start: f64, d: f64, bounds: [f64; 2], range: (f64, f64)) -> Option<(f64, f64)> {
    let [lower, upper] = bounds;
    if d.abs() < 1e-12 {
        if start < lower || start > upper {
            return None;
        }
        return Some(range);
    }
    let mut t1 = (lower - start) / d;
    let mut t2 = (upper - start) / d;
    if t1 > t2 {
        std::mem::swap(&mut t1, &mut t2);
    }
    let tmin = range.0.max(t1);
    let tmax = range.1.min(t2);
    if tmin > tmax {
        None
    } else {
        Some((tmin, tmax))
    }
}

pub(super) fn point_in_polygon(p: [f64; 2], polygon: &[[f64; 2]]) -> bool {
    let n = polygon.len();
    let mut inside = false;
    for i in 0..n {
        let a = polygon[i];
        let b = polygon[(i + n - 1) % n];
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
    }
    inside
}

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn cross(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
