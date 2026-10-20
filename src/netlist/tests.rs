//! Tests for the Netlist connectivity graph.

use super::*;
use crate::parser::{Circuit, CircuitElement, Net, Pad, Port, Trace};

fn net(id: &str, name: &str) -> CircuitElement {
    CircuitElement::Net(Net {
        net_id: Some(id.to_string()),
        name: Some(name.to_string()),
    })
}

fn port(id: &str, name: Option<&str>, net: &str) -> CircuitElement {
    CircuitElement::Port(Port {
        port_id: Some(id.to_string()),
        name: name.map(str::to_string),
        net_id: Some(net.to_string()),
        ..Default::default()
    })
}

fn circuit_fixture() -> Circuit {
    Circuit {
        elements: vec![
            net("n1", "GND"),
            net("n2", "+3V3"),
            port("U1.1", Some("VSS"), "n1"),
            port("C1.1", Some("1"), "n1"),
            port("U1.2", Some("VDD"), "n2"),
            CircuitElement::Pad(Pad {
                pad_id: Some("P1".to_string()),
                net_id: Some("ghost_net".to_string()),
            }),
        ],
    }
}

#[test]
fn queries_endpoints_by_name_and_id_with_zero_boilerplate() {
    let circuit = circuit_fixture();
    let netlist = Netlist::from_circuit(&circuit);

    // Query by human name: returns &[Endpoint]
    let gnd_endpoints = netlist.endpoints("GND");
    assert_eq!(gnd_endpoints.len(), 2);
    assert_eq!(gnd_endpoints[0].id(), Some("U1.1"));
    assert_eq!(gnd_endpoints[0].component_id(), Some("U1"));
    assert_eq!(gnd_endpoints[0].name(), Some("VSS"));
    assert!(gnd_endpoints[0].is_port());

    // Query by net_id
    let n1_endpoints = netlist.endpoints("n1");
    assert_eq!(n1_endpoints.len(), 2);

    // Non-existent net returns empty slice directly
    assert_eq!(netlist.endpoints("DOES_NOT_EXIST"), &[]);
}

#[test]
fn checks_floating_status_trivially() {
    let circuit = circuit_fixture();
    let netlist = Netlist::from_circuit(&circuit);

    // GND has 2 endpoints: not floating
    assert!(!netlist.is_floating("GND"));

    // +3V3 has 1 endpoint: floating
    assert!(netlist.is_floating("+3V3"));
    assert!(netlist.is_floating("n2"));

    // Non-existent net is floating by definition
    assert!(netlist.is_floating("UNKNOWN"));

    // Check directly on NetNode
    let n2 = netlist.net("+3V3").expect("+3V3 must exist");
    assert!(netlist.is_floating(n2));
    assert!(n2.is_floating());
}

#[test]
fn tracks_ghost_nets_and_reference_counts() {
    let circuit = circuit_fixture();
    let netlist = Netlist::from_circuit(&circuit);

    let ghost_nets: Vec<&NetNode> = netlist.ghost_nets().collect();
    assert_eq!(ghost_nets.len(), 1);
    assert_eq!(ghost_nets[0].id(), "ghost_net");
    assert!(ghost_nets[0].is_ghost());
    assert_eq!(ghost_nets[0].reference_count(), 1);
}

#[test]
fn traces_propagate_endpoints_and_bridge_ports() {
    let circuit = Circuit {
        elements: vec![
            CircuitElement::Net(Net {
                net_id: Some("n1".to_string()),
                name: Some("BUS".to_string()),
            }),
            CircuitElement::Port(Port {
                port_id: Some("U1.3".to_string()),
                name: None,
                net_id: None,
                ..Default::default()
            }),
            CircuitElement::Port(Port {
                port_id: Some("U2.3".to_string()),
                name: None,
                net_id: None,
                ..Default::default()
            }),
            CircuitElement::Trace(Trace {
                trace_id: Some("t1".to_string()),
                connected_source_port_ids: vec!["U1.3".to_string(), "U2.3".to_string()],
                connected_source_net_ids: vec!["n1".to_string()],
                ..Default::default()
            }),
        ],
    };

    let netlist = Netlist::from_circuit(&circuit);
    assert_eq!(netlist.endpoints("BUS").len(), 2);
    assert!(!netlist.is_floating("BUS"));
    assert!(netlist.are_connected("U1.3", "U2.3"));
}

#[test]
fn anonymous_endpoints_are_preserved_for_electrical_soundness() {
    let circuit = Circuit {
        elements: vec![
            CircuitElement::Net(Net {
                net_id: Some("n1".to_string()),
                name: Some("NET".to_string()),
            }),
            // Malformed port: no port_id
            CircuitElement::Port(Port {
                port_id: None,
                name: None,
                net_id: Some("n1".to_string()),
                ..Default::default()
            }),
            CircuitElement::Port(Port {
                port_id: Some("U1.1".to_string()),
                name: None,
                net_id: Some("n1".to_string()),
                ..Default::default()
            }),
        ],
    };

    let netlist = Netlist::from_circuit(&circuit);
    let net = netlist.net("NET").expect("net exists");
    // Both endpoints counted (one anonymous, one identified)
    assert_eq!(net.endpoint_count(), 2);
    assert!(!net.is_floating());
}
