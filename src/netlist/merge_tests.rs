//! Tests for nets merged by traces.

use super::tests::{net, netlist};

#[test]
fn trace_between_two_declared_nets_merges_them_and_records_a_short() {
    let n = netlist(
        r#"[
        {"type": "source_net", "source_net_id": "n_gnd", "name": "GND"},
        {"type": "source_net", "source_net_id": "n_3v3", "name": "3V3"},
        {"type": "source_port", "source_port_id": "a", "net_id": "n_gnd"},
        {"type": "source_port", "source_port_id": "b", "net_id": "n_3v3"},
        {"type": "source_trace", "source_trace_id": "t0", "connected_source_port_ids": ["a", "b"]}
    ]"#,
    );
    assert_eq!(n.nets.len(), 1);
    assert_eq!(n.by_id["n_gnd"], n.by_id["n_3v3"]);
    assert_eq!(n.port_to_net["a"], n.port_to_net["b"]);
    assert_eq!(net(&n, "n_3v3").endpoints.len(), 2);
    assert_eq!(n.shorts().len(), 1);
    assert_eq!(n.shorts()[0].nets, ["3V3", "GND"]);
}

#[test]
fn trace_to_a_port_without_net_joins_the_declared_net() {
    let n = netlist(
        r#"[
        {"type": "source_net", "source_net_id": "n_gnd", "name": "GND"},
        {"type": "source_port", "source_port_id": "a", "net_id": "n_gnd"},
        {"type": "source_port", "source_port_id": "b"},
        {"type": "source_trace", "source_trace_id": "t0", "connected_source_port_ids": ["a", "b"]}
    ]"#,
    );
    assert_eq!(n.nets.len(), 1);
    assert_eq!(n.port_to_net["b"], n.by_id["n_gnd"]);
    assert!(n.shorts().is_empty());
}

#[test]
fn port_only_traces_sharing_a_port_form_one_synthetic_net() {
    let n = netlist(
        r#"[
        {"type": "source_port", "source_port_id": "a"},
        {"type": "source_port", "source_port_id": "b"},
        {"type": "source_port", "source_port_id": "c"},
        {"type": "source_trace", "source_trace_id": "t0", "connected_source_port_ids": ["a", "b"]},
        {"type": "source_trace", "source_trace_id": "t1", "connected_source_port_ids": ["b", "c"]}
    ]"#,
    );
    assert_eq!(n.nets.len(), 1);
    assert_eq!(n.nets[0].endpoints.len(), 3);
    assert!(n.nets[0].is_defined);
}

#[test]
fn ghost_nets_survive_a_merge_into_a_declared_net() {
    let n = netlist(
        r#"[
        {"type": "source_net", "source_net_id": "n_gnd", "name": "GND"},
        {"type": "source_port", "source_port_id": "a", "net_id": "n_gnd"},
        {"type": "source_port", "source_port_id": "b", "net_id": "missing"},
        {"type": "source_trace", "source_trace_id": "t0", "connected_source_port_ids": ["a", "b"]}
    ]"#,
    );
    assert_eq!(n.nets.len(), 1);
    assert_eq!(n.ghost_nets().count(), 1);
    assert_eq!(n.by_id["missing"], n.by_id["n_gnd"]);
}

#[test]
fn short_names_the_trace_that_joined_the_declared_nets() {
    let n = netlist(
        r#"[
        {"type": "source_net", "source_net_id": "n_gnd", "name": "GND"},
        {"type": "source_net", "source_net_id": "n_3v3", "name": "3V3"},
        {"type": "source_port", "source_port_id": "a"},
        {"type": "source_port", "source_port_id": "b"},
        {"type": "source_trace", "source_trace_id": "t_gnd",
         "connected_source_port_ids": ["a"], "connected_source_net_ids": ["n_gnd"]},
        {"type": "source_trace", "source_trace_id": "t_3v3",
         "connected_source_port_ids": ["b"], "connected_source_net_ids": ["n_3v3"]},
        {"type": "source_trace", "source_trace_id": "t_bad", "connected_source_port_ids": ["a", "b"]}
    ]"#,
    );
    assert_eq!(n.shorts().len(), 1);
    assert_eq!(n.shorts()[0].trace, r#"source_trace "t_bad""#);
}

#[test]
fn pad_on_another_net_than_its_port_is_a_short() {
    let n = netlist(
        r#"[
        {"type": "source_net", "source_net_id": "n_gnd", "name": "GND"},
        {"type": "source_net", "source_net_id": "n_3v3", "name": "3V3"},
        {"type": "source_port", "source_port_id": "a", "net_id": "n_3v3"},
        {"type": "pcb_port", "pcb_port_id": "pp_a", "source_port_id": "a"},
        {"type": "pcb_smtpad", "pcb_smtpad_id": "pad_a", "pcb_port_id": "pp_a", "net_id": "n_gnd"}
    ]"#,
    );
    assert_eq!(n.shorts().len(), 1);
    assert_eq!(n.shorts()[0].trace, r#"pad "pad_a""#);
}
