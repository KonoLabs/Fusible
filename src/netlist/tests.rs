//! Tests for the Netlist connectivity graph, on canonical circuit-json
//! shapes: ports without nets, connectivity carried by `source_trace`, and
//! component ids distinct from their names.

use std::path::Path;

use super::*;
use crate::parser::parse;

fn netlist(json: &str) -> Netlist {
    let circuit = parse(json, Path::new("circuit.json")).expect("fixture must parse");
    Netlist::from_circuit(&circuit)
}

fn net<'a>(netlist: &'a Netlist, id: &str) -> &'a NetNode {
    &netlist.nets[netlist.by_id[id]]
}

#[test]
fn endpoints_carry_the_component_id_not_its_name() {
    let n = netlist(
        r#"[
        {"type": "source_component", "source_component_id": "source_component_0", "name": "U1"},
        {"type": "source_net", "source_net_id": "net_0", "name": "GND"},
        {"type": "source_port", "source_port_id": "source_port_0", "source_component_id": "source_component_0"},
        {"type": "source_trace", "source_trace_id": "t0",
         "connected_source_port_ids": ["source_port_0"], "connected_source_net_ids": ["net_0"]}
    ]"#,
    );
    let endpoints = &net(&n, "net_0").endpoints;
    assert_eq!(endpoints.len(), 1);
    assert_eq!(
        endpoints[0].component_id.as_deref(),
        Some("source_component_0")
    );
}

#[test]
fn trace_to_a_missing_port_adds_no_endpoint_and_is_recorded() {
    let n = netlist(
        r#"[
        {"type": "source_net", "source_net_id": "net_0", "name": "SDA"},
        {"type": "source_port", "source_port_id": "sp_0"},
        {"type": "source_trace", "source_trace_id": "t0",
         "connected_source_port_ids": ["sp_0", "sp_missing"], "connected_source_net_ids": ["net_0"]}
    ]"#,
    );
    assert_eq!(net(&n, "net_0").endpoints.len(), 1);
    assert_eq!(n.dangling_refs().len(), 1);
    assert!(n.dangling_refs()[0].target.contains("sp_missing"));
}

#[test]
fn a_pad_and_its_own_port_count_once() {
    let n = netlist(
        r#"[
        {"type": "source_net", "source_net_id": "net_0", "name": "VCC"},
        {"type": "source_port", "source_port_id": "sp_0", "net_id": "net_0"},
        {"type": "source_pad", "source_pad_id": "pad_0", "source_port_id": "sp_0", "net_id": "net_0"}
    ]"#,
    );
    assert_eq!(net(&n, "net_0").endpoints.len(), 1);
}

#[test]
fn canonical_smtpad_resolves_its_net_through_pcb_port() {
    let n = netlist(
        r#"[
        {"type": "source_net", "source_net_id": "net_0", "name": "VCC"},
        {"type": "source_port", "source_port_id": "sp_0"},
        {"type": "source_trace", "source_trace_id": "t0",
         "connected_source_port_ids": ["sp_0"], "connected_source_net_ids": ["net_0"]},
        {"type": "pcb_port", "pcb_port_id": "pp_0", "source_port_id": "sp_0"},
        {"type": "pcb_smtpad", "pcb_smtpad_id": "pad_0", "pcb_port_id": "pp_0"},
        {"type": "pcb_smtpad", "pcb_smtpad_id": "pad_1", "pcb_port_id": "pp_missing"}
    ]"#,
    );
    assert_eq!(n.pad_to_net.get("pad_0"), Some(&n.by_id["net_0"]));
    assert_eq!(net(&n, "net_0").endpoints.len(), 1);
    assert_eq!(n.dangling_refs().len(), 1);
    assert!(n.dangling_refs()[0].target.contains("pp_missing"));
}

#[test]
fn traces_without_id_do_not_merge_into_one_net() {
    let n = netlist(
        r#"[
        {"type": "source_port", "source_port_id": "a1"},
        {"type": "source_port", "source_port_id": "a2"},
        {"type": "source_port", "source_port_id": "b1"},
        {"type": "source_port", "source_port_id": "b2"},
        {"type": "source_trace", "connected_source_port_ids": ["a1", "a2"]},
        {"type": "source_trace", "connected_source_port_ids": ["b1", "b2"]}
    ]"#,
    );
    assert_eq!(n.nets.len(), 2);
    assert_ne!(n.port_to_net["a1"], n.port_to_net["b1"]);
}

#[test]
fn undefined_nets_are_ghosts_with_their_reference_count() {
    let n = netlist(
        r#"[
        {"type": "source_net", "source_net_id": "net_0", "name": "GND"},
        {"type": "source_pad", "source_pad_id": "P1", "net_id": "missing"}
    ]"#,
    );
    let ghosts: Vec<&NetNode> = n.ghost_nets().collect();
    assert_eq!(ghosts.len(), 1);
    assert_eq!(ghosts[0].id, "missing");
    assert_eq!(ghosts[0].reference_count, 1);
}
