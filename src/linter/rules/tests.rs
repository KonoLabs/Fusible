//! Tests for the three phase-1 rules, each exercised on an isolated
//! `Linter::new().with_rule(...)` so a failing assertion names the rule,
//! not the default registry.
//!
//! Sibling file of `rules.rs` (declared `#[cfg(test)] mod tests`), kept
//! under filet's functions-per-file budget.

use miette::Diagnostic as _;

use super::{GhostNetRule, MalformedElementRule, UnconnectedNetRule};
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
    })
}

fn rule_violations(rule: impl Rule + 'static, elements: Vec<CircuitElement>) -> Vec<Diagnostic> {
    let circuit = Circuit { elements };
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
    });
    let one_pad = vec![net(Some("n1"), "+3V3"), pad.clone()];
    assert_eq!(rule_violations(UnconnectedNetRule, one_pad).len(), 1);

    let clean = vec![net(Some("n1"), "+3V3"), port("U1.1", Some("n1")), pad];
    assert!(rule_violations(UnconnectedNetRule, clean).is_empty());

    let trace = CircuitElement::Trace(crate::parser::Trace {
        trace_id: Some("t1".to_string()),
        connected_source_port_ids: vec!["U1.1".to_string(), "U1.2".to_string()],
        connected_source_net_ids: vec!["n1".to_string()],
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
    });
    let trace = CircuitElement::Trace(crate::parser::Trace {
        trace_id: Some("t1".to_string()),
        connected_source_port_ids: vec!["U1.4".to_string()],
        connected_source_net_ids: vec!["missing".to_string()],
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
