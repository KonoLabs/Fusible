//! `trace_clearance` rule tests on canonical `pcb_smtpad` fixtures.

use super::lint;
use crate::linter::rules::TraceClearanceRule;

/// Two nets, each reaching one canonical `pcb_smtpad` through
/// `source_trace` -> `source_port` <- `pcb_port` <- `pcb_port_id`.
fn two_pads(pad_a: &str, pad_b: &str) -> String {
    format!(
        r#"[
        {{"type": "source_net", "source_net_id": "net_a", "name": "SIG_A"}},
        {{"type": "source_net", "source_net_id": "net_b", "name": "SIG_B"}},
        {{"type": "source_port", "source_port_id": "sp_a"}},
        {{"type": "source_port", "source_port_id": "sp_b"}},
        {{"type": "source_trace", "source_trace_id": "st_a",
          "connected_source_port_ids": ["sp_a"], "connected_source_net_ids": ["net_a"]}},
        {{"type": "source_trace", "source_trace_id": "st_b",
          "connected_source_port_ids": ["sp_b"], "connected_source_net_ids": ["net_b"]}},
        {{"type": "pcb_port", "pcb_port_id": "pp_a", "source_port_id": "sp_a"}},
        {{"type": "pcb_port", "pcb_port_id": "pp_b", "source_port_id": "sp_b"}},
        {{"type": "pcb_smtpad", "pcb_smtpad_id": "pad_a", "layer": "top", {pad_a}}},
        {{"type": "pcb_smtpad", "pcb_smtpad_id": "pad_b", "layer": "top", {pad_b}}}
    ]"#
    )
}

#[test]
fn trace_clearance_resolves_canonical_pad_nets() {
    let a = r#""pcb_port_id": "pp_a", "shape": "rect", "x": 0, "y": 0, "width": 1, "height": 1"#;
    let b = r#""pcb_port_id": "pp_b", "shape": "rect", "x": 0.5, "y": 0, "width": 1, "height": 1"#;
    let v = lint(TraceClearanceRule, &two_pads(a, b));
    assert_eq!(v.len(), 1);
    assert!(v[0].message.contains("short-circuit"));
    assert!(v[0].message.contains("SIG_A"));

    let same_net = b.replace("pp_b", "pp_a");
    assert!(lint(TraceClearanceRule, &two_pads(a, &same_net)).is_empty());
}

#[test]
fn trace_clearance_reads_circle_pad_radius() {
    let a = r#""pcb_port_id": "pp_a", "shape": "circle", "x": 0, "y": 0, "radius": 1"#;
    let b = r#""pcb_port_id": "pp_b", "shape": "circle", "x": 1.5, "y": 0, "radius": 1"#;
    assert_eq!(lint(TraceClearanceRule, &two_pads(a, b)).len(), 1);
}

#[test]
fn trace_clearance_treats_an_unresolved_pad_as_its_own_net() {
    let a = r#""pcb_port_id": "pp_a", "shape": "rect", "x": 0, "y": 0, "width": 1, "height": 1"#;
    let b = r#""shape": "rect", "x": 0.5, "y": 0, "width": 1, "height": 1"#;
    let v = lint(TraceClearanceRule, &two_pads(a, b));
    assert_eq!(v.len(), 1);
    assert!(v[0].message.contains("unresolved pad pad_b"));
}

#[test]
fn trace_clearance_measures_diagonal_gaps_corner_to_corner() {
    let a = r#""pcb_port_id": "pp_a", "shape": "rect", "x": 0, "y": 0, "width": 1, "height": 1"#;
    let diagonal =
        r#""pcb_port_id": "pp_b", "shape": "rect", "x": 1.15, "y": 1.15, "width": 1, "height": 1"#;
    assert!(lint(TraceClearanceRule, &two_pads(a, diagonal)).is_empty());

    let close =
        r#""pcb_port_id": "pp_b", "shape": "rect", "x": 1.1, "y": 0, "width": 1, "height": 1"#;
    assert_eq!(lint(TraceClearanceRule, &two_pads(a, close)).len(), 1);
}
