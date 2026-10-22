use serde_json::Value;

use super::astar::{find_path, AStarConfig, RouteStepKind, Terminal};
use super::engine::{route_circuit, RouteOptions};
use super::spatial::{segment_distance, SpatialObstacle, SpatialRoutingIndex};

fn terminal(x: f64, y: f64, layers: &[&str]) -> Terminal {
    Terminal {
        point: [x, y],
        layers: layers.iter().map(|l| l.to_string()).collect(),
    }
}

fn pad(id: &str, x: f64, y: f64, layer: &str) -> String {
    format!(
        r#"{{ "type": "source_port", "source_port_id": "sp_{id}", "name": "{id}" }},
        {{ "type": "pcb_port", "pcb_port_id": "pp_{id}", "source_port_id": "sp_{id}", "x": {x}, "y": {y}, "layers": ["{layer}"] }},
        {{ "type": "pcb_smtpad", "pcb_smtpad_id": "pad_{id}", "pcb_port_id": "pp_{id}", "shape": "rect", "x": {x}, "y": {y}, "width": 1.0, "height": 1.0, "layer": "{layer}" }}"#
    )
}

fn connect(id: &str, a: &str, b: &str) -> String {
    format!(
        r#"{{ "type": "source_trace", "source_trace_id": "st_{id}", "connected_source_port_ids": ["sp_{a}", "sp_{b}"], "connected_source_net_ids": [] }}"#
    )
}

fn circuit(board: (f64, f64), parts: &[String]) -> String {
    format!(
        r#"[{{ "type": "pcb_board", "width": {}, "height": {}, "center": {{ "x": 0, "y": 0 }} }}, {}]"#,
        board.0,
        board.1,
        parts.join(",")
    )
}

fn crossing_nets() -> String {
    circuit(
        (30.0, 30.0),
        &[
            pad("a1", -5.0, 0.0, "top"),
            pad("a2", 5.0, 0.0, "top"),
            pad("b1", 0.0, -5.0, "top"),
            pad("b2", 0.0, 5.0, "top"),
            connect("a", "a1", "a2"),
            connect("b", "b1", "b2"),
        ],
    )
}

#[test]
fn spatial_index_detects_obstacle_collisions() {
    let mut index = SpatialRoutingIndex::new();
    index.insert_obstacle(
        "top",
        SpatialObstacle::pad(Some("n1".to_string()), [5.0, 5.0], [7.0, 7.0]),
    );

    assert!(!index.is_segment_clear("top", [0.0, 6.0], [10.0, 6.0], 0.2, 0.15, "n2"));
    assert!(index.is_segment_clear("top", [0.0, 6.0], [10.0, 6.0], 0.2, 0.15, "n1"));
    assert!(index.is_segment_clear("top", [0.0, 0.0], [10.0, 0.0], 0.2, 0.15, "n2"));
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

#[test]
fn engine_resolves_nets_through_pcb_port_and_source_trace() {
    let json = circuit(
        (40.0, 20.0),
        &[
            pad("g1", 0.0, 0.0, "top"),
            pad("g2", 10.0, 0.0, "top"),
            pad("v1", 0.0, 5.0, "top"),
            pad("v2", 10.0, 5.0, "top"),
            connect("gnd", "g1", "g2"),
            connect("vcc", "v1", "v2"),
        ],
    );

    let res = route_circuit(&json, &RouteOptions::default()).expect("routing should succeed");
    assert_eq!(res.total_nets, 2);
    assert_eq!(res.routed_nets, 2);
    assert!(res.unrouted_nets.is_empty());
    assert!(res.output_json.contains("pcb_trace_auto_2"));
}

#[test]
fn routed_nets_never_violate_clearance_to_each_other() {
    let options = RouteOptions {
        layers: 1,
        ..Default::default()
    };
    let res = route_circuit(&crossing_nets(), &options).expect("routing should run");
    assert_eq!(res.routed_nets, 2);
    let elements: Vec<Value> = serde_json::from_str(&res.output_json).unwrap();

    let mut segments = Vec::new();
    for el in elements.iter().filter(|e| e["type"] == "pcb_trace") {
        let net = el["net_id"].as_str().unwrap().to_string();
        let route = el["route"].as_array().unwrap();
        for pair in route.windows(2) {
            let p = |v: &Value| [v["x"].as_f64().unwrap(), v["y"].as_f64().unwrap()];
            segments.push((net.clone(), p(&pair[0]), p(&pair[1])));
        }
    }
    assert!(!segments.is_empty());
    for (na, a1, a2) in &segments {
        for (nb, b1, b2) in &segments {
            if na != nb {
                let gap = segment_distance(*a1, *a2, *b1, *b2);
                assert!(
                    gap >= options.trace_width + options.clearance - 1e-9,
                    "{na} and {nb} are {gap} mm apart"
                );
            }
        }
    }
}

#[test]
fn routing_is_deterministic() {
    let first = route_circuit(&crossing_nets(), &RouteOptions::default()).unwrap();
    let second = route_circuit(&crossing_nets(), &RouteOptions::default()).unwrap();
    assert_eq!(first.output_json, second.output_json);
}

#[test]
fn board_outline_blocks_routes_that_would_leave_the_board() {
    let json = circuit(
        (20.0, 2.0),
        &[
            pad("a1", -8.0, 0.0, "top"),
            pad("a2", 8.0, 0.0, "top"),
            connect("a", "a1", "a2"),
            r#"{ "type": "pcb_smtpad", "shape": "rect", "x": 0, "y": 0, "width": 1, "height": 1.6, "layer": "top" }"#.to_string(),
        ],
    );
    let options = RouteOptions {
        layers: 1,
        max_iterations: 1,
        ..Default::default()
    };
    let res = route_circuit(&json, &options).unwrap();
    assert_eq!(res.routed_nets, 0);
}

#[test]
fn routing_without_a_board_fails_closed() {
    let json = format!(
        "[{}, {}, {}]",
        pad("a1", 0.0, 0.0, "top"),
        pad("a2", 5.0, 0.0, "top"),
        connect("a", "a1", "a2")
    );
    assert!(route_circuit(&json, &RouteOptions::default()).is_err());
}

#[test]
fn routed_output_feeds_the_strict_gerber_generator() {
    let res = route_circuit(&crossing_nets(), &RouteOptions::default()).unwrap();
    assert_eq!(res.routed_nets, 2);
    crate::gerber::generate_gerber_files(&res.output_json, "t").expect("routed JSON is fabricable");
}
