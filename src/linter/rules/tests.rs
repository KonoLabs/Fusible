//! Rule tests, each exercised on an isolated `Linter::new().with_rule(...)`
//! so a failing assertion names the rule, not the default registry.
//! Fixtures for the physical and power rules use canonical circuit-json:
//! ports without nets, `pcb_smtpad` linked through `pcb_port`, and
//! component ids distinct from their names.
//!
//! Sibling file of `rules.rs` (declared `#[cfg(test)] mod tests`), kept
//! under filet's functions-per-file budget.

use std::path::Path;

use miette::Diagnostic as _;

use super::{
    BypassCapRule, DanglingReferenceRule, GhostNetRule, MalformedElementRule, SchematicScaleRule,
    TraceClearanceRule, UnconnectedNetRule,
};
use crate::linter::{Diagnostic, Linter, Rule};
use crate::parser::{parse, Circuit, CircuitElement, Net, Pad, Port};

fn net(id: Option<&str>, name: &str) -> CircuitElement {
    CircuitElement::Net(Net {
        net_id: id.map(str::to_string),
        name: Some(name.to_string()),
    })
}

fn port(id: &str, net_id: Option<&str>) -> CircuitElement {
    CircuitElement::Port(Port {
        port_id: Some(id.to_string()),
        name: None,
        net_id: net_id.map(str::to_string),
        ..Default::default()
    })
}

fn rule_violations(rule: impl Rule + 'static, elements: Vec<CircuitElement>) -> Vec<Diagnostic> {
    let circuit = Circuit {
        elements,
        raw_elements: Vec::new(),
    };
    Linter::new().with_rule(rule).run(&circuit)
}

#[test]
fn floating_net_reported_with_help() {
    let violations = rule_violations(
        UnconnectedNetRule,
        vec![net(Some("n1"), "GND"), port("U1.1", Some("n1"))],
    );
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "unconnected_net");
    assert!(violations[0].message.contains("GND"));
    assert!(violations[0].message.contains("floating"));
    let help = violations[0].help().expect("the rule ships a help line");
    assert!(help.to_string().contains("tscircuit"));

    let v = rule_violations(UnconnectedNetRule, vec![net(Some("n1"), "+3V3")]);
    assert_eq!(v.len(), 1);
}

#[test]
fn endpoints_counted_across_ports_pads_and_traces() {
    let pad = CircuitElement::Pad(Pad {
        pad_id: Some("P1".to_string()),
        net_id: Some("n1".to_string()),
        ..Default::default()
    });
    let one_pad = vec![net(Some("n1"), "+3V3"), pad.clone()];
    assert_eq!(rule_violations(UnconnectedNetRule, one_pad).len(), 1);

    let clean = vec![net(Some("n1"), "+3V3"), port("U1.1", Some("n1")), pad];
    assert!(rule_violations(UnconnectedNetRule, clean).is_empty());

    let trace = CircuitElement::Trace(crate::parser::Trace {
        trace_id: Some("t1".to_string()),
        connected_source_port_ids: vec!["U1.1".to_string(), "U1.2".to_string()],
        connected_source_net_ids: vec!["n1".to_string()],
        ..Default::default()
    });
    let traced = vec![
        net(Some("n1"), "GND"),
        port("U1.1", None),
        port("U1.2", None),
        trace,
    ];
    assert!(rule_violations(UnconnectedNetRule, traced).is_empty());
}

#[test]
fn elements_without_identifiers_are_reported_as_malformed() {
    // The fail-closed core: input the parser tolerates but the linter
    // refuses to wave through.
    let nameless_port = CircuitElement::Port(Port {
        port_id: None,
        name: Some("SDA".to_string()),
        net_id: None,
        ..Default::default()
    });
    let v = rule_violations(MalformedElementRule, vec![net(None, "GND"), nameless_port]);

    assert_eq!(v.len(), 2);
    assert_eq!(v[0].rule, "malformed_element");
    assert!(v[0].message.contains("source_net"));
    assert!(v[0].message.contains("cannot be verified"));
    assert!(v[1].message.contains("source_port"));

    // Identified elements are clean.
    let v = rule_violations(
        MalformedElementRule,
        vec![net(Some("n1"), "GND"), port("U1.1", Some("n1"))],
    );
    assert!(v.is_empty());
}

#[test]
fn referenced_but_undefined_nets_are_reported_once() {
    let pad = CircuitElement::Pad(Pad {
        pad_id: Some("P1".to_string()),
        net_id: Some("missing".to_string()),
        ..Default::default()
    });
    let trace = CircuitElement::Trace(crate::parser::Trace {
        trace_id: Some("t1".to_string()),
        connected_source_port_ids: vec!["U1.4".to_string()],
        connected_source_net_ids: vec!["missing".to_string()],
        ..Default::default()
    });
    let v = rule_violations(
        GhostNetRule,
        vec![
            net(Some("n1"), "GND"),
            // Two references to the defined net: no violation for it...
            port("U1.1", Some("n1")),
            port("U1.2", Some("n1")),
            // ...and four references to a net nobody defines (ports, pad, trace):
            // one violation, deduplicated, with the reference count.
            port("U1.3", Some("missing")),
            pad,
            port("C1.1", Some("missing")),
            trace,
        ],
    );

    assert_eq!(v.len(), 1);
    assert_eq!(v[0].rule, "ghost_net");
    assert!(v[0].message.contains("missing"));
    assert!(v[0].message.contains("4 connection(s)"));
}

#[test]
fn pin_mux_rule_detects_invalid_alternate_function() {
    use super::PinMuxRule;
    use crate::parser::Component;

    let chip = CircuitElement::Component(Component {
        component_id: Some("U1".to_string()),
        name: Some("STM32G431".to_string()),
        kind: Some("chip".to_string()),
    });
    let port_pa5 = CircuitElement::Port(Port {
        port_id: Some("U1.PA5".to_string()),
        source_component_id: Some("U1".to_string()),
        name: Some("PA5".to_string()),
        net_id: Some("net_tx".to_string()),
        ..Default::default()
    });
    let net_tx = net(Some("net_tx"), "UART1_TX");
    let remote_port = CircuitElement::Port(Port {
        port_id: Some("J1.1".to_string()),
        source_component_id: Some("J1".to_string()),
        name: Some("RX".to_string()),
        net_id: Some("net_tx".to_string()),
        ..Default::default()
    });

    let v = rule_violations(PinMuxRule, vec![chip, port_pa5, net_tx, remote_port]);

    assert_eq!(v.len(), 1);
    assert_eq!(v[0].rule, "pin_mux");
    assert!(v[0].message.contains("PA5"));
    assert!(v[0].message.contains("UART1_TX"));
    assert!(v[0].help.as_ref().unwrap().contains("PA9"));
}

#[test]
fn pin_mux_rule_detects_esp32_strapping_conflict() {
    use super::PinMuxRule;
    use crate::parser::Component;

    let chip = CircuitElement::Component(Component {
        component_id: Some("U1".to_string()),
        name: Some("ESP32-S3".to_string()),
        kind: Some("chip".to_string()),
    });
    let port_gpio0 = CircuitElement::Port(Port {
        port_id: Some("U1.GPIO0".to_string()),
        source_component_id: Some("U1".to_string()),
        name: Some("GPIO0".to_string()),
        net_id: Some("net_led".to_string()),
        ..Default::default()
    });
    let net_led = net(Some("net_led"), "LED_STATUS");
    let remote_port = CircuitElement::Port(Port {
        port_id: Some("D1.K".to_string()),
        source_component_id: Some("D1".to_string()),
        name: Some("K".to_string()),
        net_id: Some("net_led".to_string()),
        ..Default::default()
    });

    let v = rule_violations(PinMuxRule, vec![chip, port_gpio0, net_led, remote_port]);

    assert_eq!(v.len(), 1);
    assert_eq!(v[0].rule, "pin_mux");
    assert!(v[0].message.contains("GPIO0"));
    assert!(v[0].message.contains("strapping pin"));
}

#[test]
fn pin_mux_rule_on_example_file() {
    let source = include_str!("../../../examples/circuit-pinmux-violation.json");
    let circuit = crate::parser::parse(
        source,
        std::path::Path::new("examples/circuit-pinmux-violation.json"),
    )
    .unwrap();
    let netlist = crate::netlist::Netlist::from_circuit(&circuit);
    let ctx = crate::linter::LintContext::new(&circuit, &netlist).with_mcu(Some("stm32g431"));
    let v = crate::linter::Linter::default().run_with_context(&ctx);
    println!("Violations on example file: {:?}", v);
    assert_eq!(v.len(), 1);
}

fn lint(rule: impl Rule + 'static, json: &str) -> Vec<Diagnostic> {
    let circuit = parse(json, Path::new("circuit.json")).expect("fixture must parse");
    Linter::new().with_rule(rule).run(&circuit)
}

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

/// A chip whose VDD pin reaches rail `V3_3` only through a `source_trace`,
/// next to a second component of the given name and `ftype` on that rail.
fn rail_with(name: &str, ftype: &str) -> String {
    format!(
        r#"[
        {{"type": "source_component", "source_component_id": "source_component_0", "name": "MCU", "ftype": "simple_chip"}},
        {{"type": "source_component", "source_component_id": "source_component_1", "name": "{name}", "ftype": "{ftype}"}},
        {{"type": "source_net", "source_net_id": "source_net_0", "name": "V3_3"}},
        {{"type": "source_port", "source_port_id": "source_port_0", "name": "VDD", "source_component_id": "source_component_0"}},
        {{"type": "source_port", "source_port_id": "source_port_1", "name": "pin1", "source_component_id": "source_component_1"}},
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

fn schematic_component(center: serde_json::Value) -> Circuit {
    Circuit {
        elements: vec![],
        raw_elements: vec![serde_json::json!({
            "type": "schematic_component",
            "schematic_component_id": "schematic_component_0",
            "source_component_id": "source_component_0",
            "refdes": "U1",
            "center": center
        })],
    }
}

#[test]
fn schematic_scale_reports_its_code_as_rule_name() {
    let circuit = schematic_component(serde_json::json!({ "x": 700.0, "y": -1000.0 }));
    let violations = Linter::new().with_rule(SchematicScaleRule).run(&circuit);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "E_SCHEMATIC_SCALE_EXCESSIVE");
    assert_eq!(
        violations[0].code().unwrap().to_string(),
        "E_SCHEMATIC_SCALE_EXCESSIVE"
    );
    assert_eq!(
        violations[0].message,
        "Component 'U1' placed at (700.0, -1000.0) exceeds standard schematic sheet bounds"
    );
}

#[test]
fn schematic_scale_fails_closed_on_nan_coordinates() {
    let circuit = schematic_component(serde_json::json!({ "x": "NaN", "y": 0.0 }));
    let violations = Linter::new().with_rule(SchematicScaleRule).run(&circuit);
    assert_eq!(violations.len(), 1);
}

#[test]
fn schematic_scale_passes_normal_coordinates() {
    let circuit = schematic_component(serde_json::json!({ "x": 0.0, "y": 5.0 }));
    let violations = Linter::new().with_rule(SchematicScaleRule).run(&circuit);
    assert!(violations.is_empty());
}
