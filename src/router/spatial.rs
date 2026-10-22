use std::collections::BTreeMap;

use rstar::{RTree, RTreeObject, AABB};

/// Copper geometry an obstacle occupies on one layer.
#[derive(Clone, Debug, PartialEq)]
pub enum ObstacleKind {
    Pad,
    Via {
        center: [f64; 2],
        diameter: f64,
    },
    TraceSegment {
        from: [f64; 2],
        to: [f64; 2],
        width: f64,
    },
}

/// An obstacle on one layer; `net_id` is `None` when it must be avoided by every net.
#[derive(Clone, Debug)]
pub struct SpatialObstacle {
    pub net_id: Option<String>,
    pub kind: ObstacleKind,
    pub aabb: AABB<[f64; 2]>,
}

impl SpatialObstacle {
    /// Builds an axis-aligned rectangular pad obstacle.
    pub fn pad(net_id: Option<String>, lower: [f64; 2], upper: [f64; 2]) -> Self {
        Self {
            net_id,
            kind: ObstacleKind::Pad,
            aabb: AABB::from_corners(lower, upper),
        }
    }

    /// Builds a round via or drilled hole obstacle.
    pub fn via(net_id: Option<String>, center: [f64; 2], diameter: f64) -> Self {
        let r = diameter / 2.0;
        Self {
            net_id,
            kind: ObstacleKind::Via { center, diameter },
            aabb: AABB::from_corners(
                [center[0] - r, center[1] - r],
                [center[0] + r, center[1] + r],
            ),
        }
    }

    /// Builds a round-capped trace segment obstacle.
    pub fn trace(net_id: Option<String>, from: [f64; 2], to: [f64; 2], width: f64) -> Self {
        let r = width / 2.0;
        Self {
            net_id,
            kind: ObstacleKind::TraceSegment { from, to, width },
            aabb: AABB::from_corners(
                [from[0].min(to[0]) - r, from[1].min(to[1]) - r],
                [from[0].max(to[0]) + r, from[1].max(to[1]) + r],
            ),
        }
    }

    fn distance_to_segment(&self, p1: [f64; 2], p2: [f64; 2]) -> f64 {
        match &self.kind {
            ObstacleKind::Pad => segment_rect_distance(p1, p2, &self.aabb),
            ObstacleKind::Via { center, diameter } => {
                point_segment_distance(*center, p1, p2) - diameter / 2.0
            }
            ObstacleKind::TraceSegment { from, to, width } => {
                segment_distance(p1, p2, *from, *to) - width / 2.0
            }
        }
    }
}

impl RTreeObject for SpatialObstacle {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.aabb
    }
}

/// Per-layer obstacle trees plus an optional board outline every route must stay inside.
#[derive(Clone, Default)]
pub struct SpatialRoutingIndex {
    layer_trees: BTreeMap<String, RTree<SpatialObstacle>>,
    boundary: Option<Vec<[f64; 2]>>,
}

impl SpatialRoutingIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Restricts routing to the inside of `polygon`, keeping the clearance from its edges.
    pub fn set_boundary(&mut self, polygon: Vec<[f64; 2]>) {
        self.boundary = Some(polygon);
    }

    pub fn insert_obstacle(&mut self, layer: &str, obstacle: SpatialObstacle) {
        self.layer_trees
            .entry(layer.to_string())
            .or_default()
            .insert(obstacle);
    }

    /// Returns true when a round-capped stroke of `width` from `p1` to `p2` keeps `clearance` from every foreign obstacle and the board edge. A zero-length stroke checks a round pad of diameter `width`.
    pub fn is_segment_clear(
        &self,
        layer: &str,
        p1: [f64; 2],
        p2: [f64; 2],
        width: f64,
        clearance: f64,
        net_id: &str,
    ) -> bool {
        let margin = width / 2.0 + clearance;

        if let Some(outline) = &self.boundary {
            if !point_in_polygon(p1, outline) || !point_in_polygon(p2, outline) {
                return false;
            }
            let n = outline.len();
            if (0..n).any(|i| segment_distance(p1, p2, outline[i], outline[(i + 1) % n]) < margin) {
                return false;
            }
        }

        let Some(tree) = self.layer_trees.get(layer) else {
            return true;
        };
        let query_env = AABB::from_corners(
            [p1[0].min(p2[0]) - margin, p1[1].min(p2[1]) - margin],
            [p1[0].max(p2[0]) + margin, p1[1].max(p2[1]) + margin],
        );
        tree.locate_in_envelope_intersecting(query_env)
            .filter(|obs| obs.net_id.as_deref() != Some(net_id))
            .all(|obs| obs.distance_to_segment(p1, p2) >= margin)
    }
}

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

fn point_segment_distance(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
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

fn segment_rect_distance(p1: [f64; 2], p2: [f64; 2], rect: &AABB<[f64; 2]>) -> f64 {
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
    let mut tmin = 0.0f64;
    let mut tmax = 1.0f64;

    for i in 0..2 {
        let d = p2[i] - p1[i];
        if d.abs() < 1e-12 {
            if p1[i] < lower[i] || p1[i] > upper[i] {
                return false;
            }
        } else {
            let mut t1 = (lower[i] - p1[i]) / d;
            let mut t2 = (upper[i] - p1[i]) / d;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return false;
            }
        }
    }

    true
}

fn point_in_polygon(p: [f64; 2], polygon: &[[f64; 2]]) -> bool {
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
