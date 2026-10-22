//! `decoupling_capacitor` rule tests.

use super::lint;
use crate::linter::rules::BypassCapRule;

/// A chip whose VDD pin reaches rail `V3_3` only through a `source_trace`,
/// next to a second component of the given name and `ftype` on that rail.
fn rail_with(name: &str, ftype: &str) -> String {
    format!(
        r#"[
        {{"type": "source_component", "source_component_id": "source_component_0",
          "name": "MCU", "ftype": "simple_chip"}},
        {{"type": "source_component", "source_component_id": "source_component_1",
          "name": "{name}", "ftype": "{ftype}"}},
        {{"type": "source_net", "source_net_id": "source_net_0", "name": "V3_3"}},
        {{"type": "source_port", "source_port_id": "source_port_0", "name": "VDD",
          "source_component_id": "source_component_0"}},
        {{"type": "source_port", "source_port_id": "source_port_1", "name": "pin1",
          "source_component_id": "source_component_1"}},
        {{"type": "source_trace", "source_trace_id": "source_trace_0",
          "connected_source_port_ids": ["source_port_0", "source_port_1"],
          "connected_source_net_ids": ["source_net_0"]}}
    ]"#
    )
}

#[test]
fn bypass_cap_flags_a_traced_rail_without_capacitor() {
    let v = lint(BypassCapRule, &rail_with("C9", "simple_resistor"));
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].rule, "decoupling_capacitor");
    assert!(v[0].message.contains("MCU"));
    assert!(v[0].message.contains("V3_3"));
}

#[test]
fn bypass_cap_accepts_a_capacitor_named_unlike_c() {
    assert!(lint(BypassCapRule, &rail_with("BYP", "simple_capacitor")).is_empty());
}
