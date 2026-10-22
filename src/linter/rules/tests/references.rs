//! `malformed_element` and `dangling_reference` tests on circuit-json traces.

use super::lint;
use crate::linter::rules::{DanglingReferenceRule, MalformedElementRule, ShortedNetsRule};

#[test]
fn trace_connecting_nothing_is_malformed() {
    let v = lint(
        MalformedElementRule,
        r#"[{"type": "source_trace", "source_trace_id": "t0"}]"#,
    );
    assert_eq!(v.len(), 1);
    assert!(v[0].message.contains("connects no port and no net"));
}

#[test]
fn trace_to_a_missing_port_is_a_dangling_reference() {
    let v = lint(
        DanglingReferenceRule,
        r#"[
        {"type": "source_net", "source_net_id": "net_0", "name": "SDA"},
        {"type": "source_port", "source_port_id": "sp_0"},
        {"type": "source_trace", "source_trace_id": "t0",
         "connected_source_port_ids": ["sp_0", "sp_missing"], "connected_source_net_ids": ["net_0"]}
    ]"#,
    );
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].rule, "dangling_reference");
    assert!(v[0].message.contains("sp_missing"));
}

#[test]
fn trace_joining_two_declared_nets_is_a_short() {
    let v = lint(
        ShortedNetsRule,
        r#"[
        {"type": "source_net", "source_net_id": "n_gnd", "name": "GND"},
        {"type": "source_net", "source_net_id": "n_3v3", "name": "3V3"},
        {"type": "source_port", "source_port_id": "a", "net_id": "n_gnd"},
        {"type": "source_port", "source_port_id": "b", "net_id": "n_3v3"},
        {"type": "source_trace", "source_trace_id": "t0", "connected_source_port_ids": ["a", "b"]}
    ]"#,
    );
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].rule, "shorted_nets");
    assert!(v[0].message.contains("3V3, GND"));
    assert!(v[0].message.contains("t0"));
}

#[test]
fn trace_joining_a_declared_net_to_a_bare_port_is_not_a_short() {
    let v = lint(
        ShortedNetsRule,
        r#"[
        {"type": "source_net", "source_net_id": "n_gnd", "name": "GND"},
        {"type": "source_port", "source_port_id": "a", "net_id": "n_gnd"},
        {"type": "source_port", "source_port_id": "b"},
        {"type": "source_trace", "source_trace_id": "t0", "connected_source_port_ids": ["a", "b"]}
    ]"#,
    );
    assert!(v.is_empty());
}
