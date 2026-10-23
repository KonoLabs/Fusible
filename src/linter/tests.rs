//! Engine tests: construction, registration order, and the miette rendering
//! contract of `Diagnostic` / `FuseBlown`.
//!
//! Sibling file of `linter.rs` (declared `#[cfg(test)] mod tests`): keeping
//! tests out of the module file is what holds `linter.rs` under filet's
//! functions-per-file budget. Rule behavior lives in `rules/tests.rs`.

use miette::Diagnostic as _;

use super::*;
use crate::parser::{Circuit, CircuitElement, Net, Port};

/// One net wired to a single port: `unconnected_net` must fire on it.
fn floating_net() -> Circuit {
    Circuit {
        elements: vec![
            CircuitElement::Net(Net {
                net_id: Some("n1".to_string()),
                name: Some("GND".to_string()),
            }),
            CircuitElement::Port(Port {
                port_id: Some("U1.1".to_string()),
                name: None,
                net_id: Some("n1".to_string()),
                ..Default::default()
            }),
        ],
        raw_elements: Vec::new(),
    }
}

/// `new()` is the isolation hatch: nothing runs unless it was added.
#[test]
fn new_starts_with_no_rules() {
    let violations = Linter::new().run(&floating_net());
    assert!(
        violations.is_empty(),
        "expected no rules, got {violations:?}"
    );
}

#[test]
fn default_registers_every_phase_1_rule() {
    let violations = Linter::default().run(&floating_net());
    assert!(
        violations.iter().any(|v| v.rule == "unconnected_net"),
        "default must register UnconnectedNetRule, got {violations:?}"
    );
}

#[test]
fn with_rule_appends_to_an_empty_linter() {
    let violations = Linter::new()
        .with_rule(UnconnectedNetRule)
        .run(&floating_net());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "unconnected_net");
}

/// One violation per rule, in the order `default()` registers them:
/// unconnected (identified floating net), malformed (net with no id),
/// ghost (port pointing at an undefined net).
#[test]
fn rules_run_in_registration_order() {
    let circuit = Circuit {
        elements: vec![
            CircuitElement::Net(Net {
                net_id: Some("n1".to_string()),
                name: Some("GND".to_string()),
            }),
            CircuitElement::Port(Port {
                port_id: Some("U1.1".to_string()),
                name: None,
                net_id: Some("n1".to_string()),
                ..Default::default()
            }),
            CircuitElement::Net(Net {
                net_id: None,
                name: Some("nameless".to_string()),
            }),
            CircuitElement::Port(Port {
                port_id: Some("U1.2".to_string()),
                name: None,
                net_id: Some("undefined".to_string()),
                ..Default::default()
            }),
        ],
        raw_elements: Vec::new(),
    };

    let violations = Linter::default().run(&circuit);
    let rules: Vec<&str> = violations.iter().map(|v| v.rule.as_str()).collect();

    assert_eq!(rules, ["unconnected_net", "malformed_element", "ghost_net"]);
}

#[test]
fn diagnostic_exposes_code_severity_and_help() {
    let diagnostic = Diagnostic {
        rule: "unconnected_net".to_string(),
        message: "net is floating".to_string(),
        help: Some("route it".to_string()),
        ..Default::default()
    };

    assert_eq!(
        diagnostic.code().expect("code").to_string(),
        "unconnected_net"
    );
    assert_eq!(diagnostic.severity(), Some(miette::Severity::Error));
    assert_eq!(diagnostic.help().expect("help").to_string(), "route it");

    let silent = Diagnostic {
        rule: "r".to_string(),
        message: "m".to_string(),
        help: None,
        ..Default::default()
    };
    assert!(silent.help().is_none());
}

#[test]
fn fuse_blown_renders_its_violations_as_related_errors() {
    let report = miette::Report::new(FuseBlown::new(vec![Diagnostic {
        rule: "unconnected_net".to_string(),
        message: "net \"I2C_SDA\" is floating".to_string(),
        help: None,
        ..Default::default()
    }]));
    let rendered = format!("{report:?}");

    assert!(rendered.contains("the fuse blew"), "got:\n{rendered}");
    assert!(rendered.contains("unconnected_net"), "got:\n{rendered}");
    assert!(rendered.contains("I2C_SDA"), "got:\n{rendered}");
}
