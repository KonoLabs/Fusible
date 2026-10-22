//! Per-layer R-tree obstacle index answering stroke clearance queries.

mod geometry;
mod obstacle;

use std::collections::BTreeMap;

use rstar::{RTree, AABB};

use geometry::point_in_polygon;
pub use geometry::segment_distance;
pub use obstacle::{ObstacleKind, SpatialObstacle};

/// A round-capped stroke of `width` that must keep `clearance` from every
/// obstacle not owned by `net_id`.
#[derive(Copy, Clone, Debug)]
pub struct Probe<'a> {
    pub width: f64,
    pub clearance: f64,
    pub net_id: &'a str,
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

    /// Returns true when the `probe` stroke from `p1` to `p2` keeps its
    /// clearance from every foreign obstacle and the board edge. A
    /// zero-length stroke checks a round pad of diameter `probe.width`.
    pub fn is_segment_clear(
        &self,
        layer: &str,
        p1: [f64; 2],
        p2: [f64; 2],
        probe: Probe<'_>,
    ) -> bool {
        let margin = probe.width / 2.0 + probe.clearance;
        if !self.inside_boundary(p1, p2, margin) {
            return false;
        }
        let Some(tree) = self.layer_trees.get(layer) else {
            return true;
        };
        let query_env = AABB::from_corners(
            [p1[0].min(p2[0]) - margin, p1[1].min(p2[1]) - margin],
            [p1[0].max(p2[0]) + margin, p1[1].max(p2[1]) + margin],
        );
        tree.locate_in_envelope_intersecting(query_env)
            .filter(|obs| obs.net_id.as_deref() != Some(probe.net_id))
            .all(|obs| obs.distance_to_segment(p1, p2) >= margin)
    }

    /// True without a board outline, or when the stroke stays inside it at
    /// least `margin` from every edge.
    fn inside_boundary(&self, p1: [f64; 2], p2: [f64; 2], margin: f64) -> bool {
        let Some(outline) = &self.boundary else {
            return true;
        };
        if !point_in_polygon(p1, outline) || !point_in_polygon(p2, outline) {
            return false;
        }
        let n = outline.len();
        !(0..n).any(|i| segment_distance(p1, p2, outline[i], outline[(i + 1) % n]) < margin)
    }
}
