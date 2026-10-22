//! A* grid search over the spatial obstacle index.

mod node;
mod path;
mod search;

use super::spatial::SpatialRoutingIndex;
use search::Search;

#[derive(Clone, Debug, PartialEq)]
pub enum RouteStepKind {
    Wire {
        layer: String,
        x: f64,
        y: f64,
    },
    Via {
        x: f64,
        y: f64,
        from_layer: String,
        to_layer: String,
    },
}

#[derive(Clone, Debug)]
pub struct RoutePath {
    pub steps: Vec<RouteStepKind>,
}

/// A connection endpoint: the exact pad center and the copper layers the pad exists on.
#[derive(Clone, Debug)]
pub struct Terminal {
    pub point: [f64; 2],
    pub layers: Vec<String>,
}

pub struct AStarConfig {
    pub grid_step: f64,
    pub trace_width: f64,
    pub clearance: f64,
    pub via_diameter: f64,
    pub via_hole_diameter: f64,
    pub via_cost: f64,
    pub layers: Vec<String>,
}

impl Default for AStarConfig {
    fn default() -> Self {
        Self {
            grid_step: 0.25,
            trace_width: 0.2,
            clearance: 0.15,
            via_diameter: 0.6,
            via_hole_diameter: 0.3,
            via_cost: 15.0,
            layers: vec!["top".to_string(), "bottom".to_string()],
        }
    }
}

/// Finds a clear path from `start` to `target`, leaving and entering only on
/// layers the terminals exist on.
pub fn find_path(
    start: &Terminal,
    target: &Terminal,
    net_id: &str,
    spatial: &SpatialRoutingIndex,
    config: &AStarConfig,
) -> Option<RoutePath> {
    let mut search = Search::new(start, target, net_id, spatial, config)?;
    search.seed();
    search.run()
}
