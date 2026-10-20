use std::collections::HashMap;

use rstar::{RTree, RTreeObject, AABB};

#[derive(Clone, Debug, PartialEq)]
pub enum ObstacleKind {
    Pad { net_id: String },
    Via { net_id: String, diameter: f64 },
    TraceSegment { net_id: String, width: f64 },
    Keepout,
}

#[derive(Clone, Debug)]
pub struct SpatialObstacle {
    pub id: usize,
    pub net_id: Option<String>,
    pub layer: String,
    pub kind: ObstacleKind,
    pub aabb: AABB<[f64; 2]>,
}

impl RTreeObject for SpatialObstacle {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.aabb
    }
}

#[derive(Default)]
pub struct SpatialRoutingIndex {
    pub layer_trees: HashMap<String, RTree<SpatialObstacle>>,
}

impl SpatialRoutingIndex {
    pub fn new() -> Self {
        Self {
            layer_trees: HashMap::new(),
        }
    }

    pub fn insert_obstacle(&mut self, layer: &str, obstacle: SpatialObstacle) {
        let tree = self.layer_trees.entry(layer.to_string()).or_default();
        tree.insert(obstacle);
    }

    pub fn is_point_clear(
        &self,
        layer: &str,
        p: [f64; 2],
        radius: f64,
        clearance: f64,
        net_id: &str,
    ) -> bool {
        let tree = match self.layer_trees.get(layer) {
            Some(t) => t,
            None => return true,
        };

        let total_margin = radius + clearance;
        let query_env = AABB::from_corners(
            [p[0] - total_margin, p[1] - total_margin],
            [p[0] + total_margin, p[1] + total_margin],
        );

        for obs in tree.locate_in_envelope_intersecting(query_env) {
            if let Some(ref obs_net) = obs.net_id {
                if obs_net == net_id {
                    continue;
                }
            }

            let box_lower = obs.aabb.lower();
            let box_upper = obs.aabb.upper();
            let clamped_x = p[0].clamp(box_lower[0], box_upper[0]);
            let clamped_y = p[1].clamp(box_lower[1], box_upper[1]);
            let dist_sq = (p[0] - clamped_x).powi(2) + (p[1] - clamped_y).powi(2);
            if dist_sq < total_margin.powi(2) {
                return false;
            }
        }

        true
    }

    pub fn is_segment_clear(
        &self,
        layer: &str,
        p1: [f64; 2],
        p2: [f64; 2],
        width: f64,
        clearance: f64,
        net_id: &str,
    ) -> bool {
        let tree = match self.layer_trees.get(layer) {
            Some(t) => t,
            None => return true,
        };

        let total_margin = (width / 2.0) + clearance;
        let min_x = p1[0].min(p2[0]) - total_margin;
        let min_y = p1[1].min(p2[1]) - total_margin;
        let max_x = p1[0].max(p2[0]) + total_margin;
        let max_y = p1[1].max(p2[1]) + total_margin;

        let query_env = AABB::from_corners([min_x, min_y], [max_x, max_y]);

        for obs in tree.locate_in_envelope_intersecting(query_env) {
            if let Some(ref obs_net) = obs.net_id {
                if obs_net == net_id {
                    continue;
                }
            }

            if segment_intersects_aabb(p1, p2, &obs.aabb, total_margin) {
                return false;
            }
        }

        true
    }
}

fn segment_intersects_aabb(
    p1: [f64; 2],
    p2: [f64; 2],
    box_aabb: &AABB<[f64; 2]>,
    margin: f64,
) -> bool {
    let lower = [box_aabb.lower()[0] - margin, box_aabb.lower()[1] - margin];
    let upper = [box_aabb.upper()[0] + margin, box_aabb.upper()[1] + margin];

    let mut tmin = 0.0f64;
    let mut tmax = 1.0f64;

    for i in 0..2 {
        let d = p2[i] - p1[i];
        if d.abs() < 1e-9 {
            if p1[i] < lower[i] || p1[i] > upper[i] {
                return false;
            }
        } else {
            let inv_d = 1.0 / d;
            let mut t1 = (lower[i] - p1[i]) * inv_d;
            let mut t2 = (upper[i] - p1[i]) * inv_d;
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
