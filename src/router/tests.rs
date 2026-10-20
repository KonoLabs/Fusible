use rstar::AABB;

use super::astar::{find_path, AStarConfig};
use super::engine::{route_circuit, RouteOptions};
use super::spatial::{ObstacleKind, SpatialObstacle, SpatialRoutingIndex};

#[test]
fn spatial_index_detects_obstacle_collisions() {
    let mut index = SpatialRoutingIndex::new();
    index.insert_obstacle(
        "top",
        SpatialObstacle {
            id: 1,
            net_id: Some("n1".to_string()),
            layer: "top".to_string(),
            kind: ObstacleKind::Pad {
                net_id: "n1".to_string(),
            },
            aabb: AABB::from_corners([5.0, 5.0], [7.0, 7.0]),
        },
    );

    assert!(!index.is_segment_clear("top", [0.0, 6.0], [10.0, 6.0], 0.2, 0.15, "n2"));

    assert!(index.is_segment_clear("top", [0.0, 6.0], [10.0, 6.0], 0.2, 0.15, "n1"));

    assert!(index.is_segment_clear("top", [0.0, 0.0], [10.0, 0.0], 0.2, 0.15, "n2"));
}

#[test]
fn astar_routes_between_two_points() {
    let index = SpatialRoutingIndex::new();
    let config = AStarConfig::default();

    let path = find_path([0.0, 0.0], [5.0, 0.0], "net1", &index, &config);
    assert!(path.is_some());
    let steps = path.unwrap().steps;
    assert!(steps.len() >= 2);
}

#[test]
fn astar_navigates_around_blocking_obstacle() {
    let mut index = SpatialRoutingIndex::new();
    index.insert_obstacle(
        "top",
        SpatialObstacle {
            id: 1,
            net_id: Some("other_net".to_string()),
            layer: "top".to_string(),
            kind: ObstacleKind::Pad {
                net_id: "other_net".to_string(),
            },
            aabb: AABB::from_corners([2.0, -1.0], [3.0, 1.0]),
        },
    );

    let config = AStarConfig {
        layers: vec!["top".to_string()],
        ..Default::default()
    };

    let path = find_path([0.0, 0.0], [5.0, 0.0], "my_net", &index, &config);
    assert!(path.is_some());
}

#[test]
fn engine_routes_circuit_json_elements() {
    let json = r#"[
        { "type": "pcb_smtpad", "x": 0.0, "y": 0.0, "width": 1.0, "height": 1.0, "layer": "top", "net_id": "GND" },
        { "type": "pcb_smtpad", "x": 10.0, "y": 0.0, "width": 1.0, "height": 1.0, "layer": "top", "net_id": "GND" },
        { "type": "pcb_smtpad", "x": 0.0, "y": 5.0, "width": 1.0, "height": 1.0, "layer": "top", "net_id": "VCC" },
        { "type": "pcb_smtpad", "x": 10.0, "y": 5.0, "width": 1.0, "height": 1.0, "layer": "top", "net_id": "VCC" }
    ]"#;

    let options = RouteOptions::default();
    let res = route_circuit(json, &options).expect("routing should succeed");

    assert_eq!(res.total_nets, 2);
    assert_eq!(res.routed_nets, 2);
    assert!(res.unrouted_nets.is_empty());
    assert!(res.generated_traces >= 2);
    assert!(res.output_json.contains("pcb_trace"));
}
