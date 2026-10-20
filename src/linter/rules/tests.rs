//! Tests for the three phase-1 rules, each exercised on an isolated
//! `Linter::new().with_rule(...)` so a failing assertion names the rule,
//! not the default registry.
//!
//! Sibling file of `rules.rs` (declared `#[cfg(test)] mod tests`), kept
//! under filet's functions-per-file budget.

use miette::Diagnostic as _;

use super::{GhostNetRule, MalformedElementRule, SchematicScaleRule, UnconnectedNetRule};
use crate::linter::{Diagnostic, Linter, Rule};
use crate::parser::{Circuit, CircuitElement, Net, Pad, Port};

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

#[test]
fn decoupling_capacitor_rule_flags_missing_bypass_caps() {
    use super::BypassCapRule;
    use crate::parser::Component;

    let chip = CircuitElement::Component(Component {
        component_id: Some("U1".to_string()),
        name: Some("STM32G431".to_string()),
        kind: Some("chip".to_string()),
    });
    let vdd_port = CircuitElement::Port(Port {
        port_id: Some("U1.VDD".to_string()),
        source_component_id: Some("U1".to_string()),
        name: Some("VDD".to_string()),
        net_id: Some("n_3v3".to_string()),
        ..Default::default()
    });
    let power_in = CircuitElement::Port(Port {
        port_id: Some("J1.VOUT".to_string()),
        source_component_id: Some("J1".to_string()),
        name: Some("VOUT".to_string()),
        net_id: Some("n_3v3".to_string()),
        ..Default::default()
    });
    let net_3v3 = net(Some("n_3v3"), "+3V3");

    // Missing capacitor: fails
    let v = rule_violations(
        BypassCapRule,
        vec![
            chip.clone(),
            vdd_port.clone(),
            power_in.clone(),
            net_3v3.clone(),
        ],
    );
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].rule, "decoupling_capacitor");
    assert!(v[0].message.contains("VDD"));
    assert!(v[0].message.contains("STM32G431"));

    // Add bypass capacitor C1
    let cap = CircuitElement::Component(Component {
        component_id: Some("C1".to_string()),
        name: Some("C1".to_string()),
        kind: Some("capacitor".to_string()),
    });
    let cap_port = CircuitElement::Port(Port {
        port_id: Some("C1.1".to_string()),
        source_component_id: Some("C1".to_string()),
        name: Some("1".to_string()),
        net_id: Some("n_3v3".to_string()),
        ..Default::default()
    });

    let v_clean = rule_violations(
        BypassCapRule,
        vec![chip, vdd_port, power_in, net_3v3, cap, cap_port],
    );
    assert!(v_clean.is_empty());
}

#[test]
fn trace_clearance_rule_detects_copper_overlap() {
    use super::TraceClearanceRule;

    let pad1 = CircuitElement::Pad(Pad {
        pad_id: Some("P1".to_string()),
        net_id: Some("net_a".to_string()),
        x: Some(10.0),
        y: Some(10.0),
        width: Some(1.0),
        height: Some(1.0),
        layer: Some("top".to_string()),
        ..Default::default()
    });
    let pad2 = CircuitElement::Pad(Pad {
        pad_id: Some("P2".to_string()),
        net_id: Some("net_b".to_string()),
        x: Some(10.05), // Overlapping!
        y: Some(10.05),
        width: Some(1.0),
        height: Some(1.0),
        layer: Some("top".to_string()),
        ..Default::default()
    });

    let net_a = net(Some("net_a"), "SIG_A");
    let net_b = net(Some("net_b"), "SIG_B");

    let v = rule_violations(
        TraceClearanceRule,
        vec![pad1.clone(), pad2.clone(), net_a.clone(), net_b.clone()],
    );
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].rule, "trace_clearance");
    assert!(v[0].message.contains("short-circuit"));

    // Move pad2 far away: clearance passes
    let pad2_far = CircuitElement::Pad(Pad {
        pad_id: Some("P2".to_string()),
        net_id: Some("net_b".to_string()),
        x: Some(25.0),
        y: Some(25.0),
        width: Some(1.0),
        height: Some(1.0),
        layer: Some("top".to_string()),
        ..Default::default()
    });

    let v_clean = rule_violations(TraceClearanceRule, vec![pad1, pad2_far, net_a, net_b]);
    assert!(v_clean.is_empty());
}

#[test]
fn schematic_scale_rule_detects_out_of_bounds_components() {
    let circuit = Circuit {
        elements: vec![],
        raw_elements: vec![serde_json::json!({
            "type": "schematic_component",
            "schematic_component_id": "schematic_component_0",
            "source_component_id": "source_component_0",
            "refdes": "U1",
            "center": { "x": 700.0, "y": -1000.0 }
        })],
    };
    let violations = Linter::new().with_rule(SchematicScaleRule).run(&circuit);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "schematic_scale");
    assert_eq!(
        violations[0].code().unwrap().to_string(),
        "E_SCHEMATIC_SCALE_EXCESSIVE"
    );
    assert_eq!(
        violations[0].message,
        "Component 'U1' placed at (700.0, -1000.0) exceeds standard schematic sheet bounds"
    );
    assert!(violations[0]
        .help()
        .unwrap()
        .to_string()
        .contains("tscircuit schematic coordinates"));
}

#[test]
fn schematic_scale_rule_passes_normal_coordinates() {
    let circuit = Circuit {
        elements: vec![],
        raw_elements: vec![serde_json::json!({
            "type": "schematic_component",
            "schematic_component_id": "schematic_component_0",
            "source_component_id": "source_component_0",
            "refdes": "R1",
            "center": { "x": 0.0, "y": 5.0 }
        })],
    };
    let violations = Linter::new().with_rule(SchematicScaleRule).run(&circuit);
    assert!(violations.is_empty());
}
