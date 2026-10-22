//! Obstacle shapes stored in the per-layer R-trees.

use rstar::{RTreeObject, AABB};

use super::geometry::{point_segment_distance, segment_distance, segment_rect_distance};

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

    pub(super) fn distance_to_segment(&self, p1: [f64; 2], p2: [f64; 2]) -> f64 {
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
