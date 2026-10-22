//! Router tests: spatial queries and A* search here, end-to-end engine runs
//! in `tests/engine.rs`.

mod engine;
mod fixtures;

use super::astar::{find_path, AStarConfig, RouteStepKind, Terminal};
use super::spatial::{Probe, SpatialObstacle, SpatialRoutingIndex};

fn terminal(x: f64, y: f64, layers: &[&str]) -> Terminal {
    Terminal {
        point: [x, y],
        layers: layers.iter().map(|l| l.to_string()).collect(),
    }
}

#[test]
fn spatial_index_detects_obstacle_collisions() {
    let mut index = SpatialRoutingIndex::new();
    index.insert_obstacle(
        "top",
        SpatialObstacle::pad(Some("n1".to_string()), [5.0, 5.0], [7.0, 7.0]),
    );
    let probe = |net_id| Probe {
        width: 0.2,
        clearance: 0.15,
        net_id,
    };

    assert!(!index.is_segment_clear("top", [0.0, 6.0], [10.0, 6.0], probe("n2")));
    assert!(index.is_segment_clear("top", [0.0, 6.0], [10.0, 6.0], probe("n1")));
    assert!(index.is_segment_clear("top", [0.0, 0.0], [10.0, 0.0], probe("n2")));
}

#[test]
fn astar_routes_between_two_points() {
    let index = SpatialRoutingIndex::new();
    let config = AStarConfig::default();

    let path = find_path(
        &terminal(0.0, 0.0, &["top"]),
        &terminal(5.0, 0.0, &["top"]),
        "net1",
        &index,
        &config,
    )
    .expect("open board routes");
    assert!(path.steps.len() >= 2);
}

#[test]
fn astar_navigates_around_blocking_obstacle() {
    let mut index = SpatialRoutingIndex::new();
    index.insert_obstacle(
        "top",
        SpatialObstacle::pad(Some("other_net".to_string()), [2.0, -1.0], [3.0, 1.0]),
    );

    let config = AStarConfig {
        layers: vec!["top".to_string()],
        ..Default::default()
    };

    let path = find_path(
        &terminal(0.0, 0.0, &["top"]),
        &terminal(5.0, 0.0, &["top"]),
        "my_net",
        &index,
        &config,
    );
    assert!(path.is_some());
}

#[test]
fn astar_ends_on_the_target_pad_layer_and_exact_center() {
    let index = SpatialRoutingIndex::new();
    let config = AStarConfig::default();

    let path = find_path(
        &terminal(0.03, 0.0, &["top"]),
        &terminal(5.11, 0.0, &["bottom"]),
        "n",
        &index,
        &config,
    )
    .expect("routes with a via");
    assert!(matches!(
        path.steps.first(),
        Some(RouteStepKind::Wire { layer, x, .. }) if layer == "top" && *x == 0.03
    ));
    assert!(matches!(
        path.steps.last(),
        Some(RouteStepKind::Wire { layer, x, .. }) if layer == "bottom" && *x == 5.11
    ));
    assert!(path
        .steps
        .iter()
        .any(|s| matches!(s, RouteStepKind::Via { .. })));
}
