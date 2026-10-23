//! End-to-end `route_circuit` tests.

use serde_json::Value;

use super::fixtures::{circuit, connect, crossing_nets, pad};
use crate::router::astar::{AStarConfig, RoutePath, RouteStepKind, Terminal};
use crate::router::engine::ripup::rip_up_and_reroute;
use crate::router::engine::{route_circuit, ConnectionJob, RouteOptions};
use crate::router::spatial::segment_distance;
use crate::router::spatial::SpatialRoutingIndex;

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
            r#"{ "type": "pcb_smtpad", "shape": "rect", "x": 0, "y": 0,
                 "width": 1, "height": 1.6, "layer": "top" }"#
                .to_string(),
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

fn job(net: &str, from: [f64; 2], to: [f64; 2]) -> ConnectionJob {
    let terminal = |point: [f64; 2]| Terminal {
        point,
        layers: vec!["top".to_string()],
    };
    ConnectionJob {
        net_id: net.to_string(),
        start: terminal(from),
        target: terminal(to),
    }
}

#[test]
fn rip_up_moves_a_blocking_route_to_free_a_blocked_one() {
    let config = AStarConfig {
        layers: vec!["top".to_string()],
        ..Default::default()
    };
    let jobs = [
        job("a", [-8.0, 0.0], [8.0, 0.0]),
        job("b", [0.0, -3.0], [0.0, 3.0]),
    ];
    let base = SpatialRoutingIndex::new();
    let wire = |x, y| RouteStepKind::Wire {
        layer: "top".to_string(),
        x,
        y,
    };
    let mut paths = vec![
        Some(RoutePath {
            steps: vec![wire(-8.0, 0.0), wire(8.0, 0.0)],
        }),
        None,
    ];

    rip_up_and_reroute(&jobs, &base, &config, &mut paths);

    assert!(paths.iter().all(Option::is_some));
}
