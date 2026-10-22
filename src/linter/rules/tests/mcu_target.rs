//! `pin_mux` target resolution: unknown chips and database coverage.

use crate::linter::rules::PinMuxRule;
use crate::linter::{Diagnostic, LintContext, Rule};
use crate::netlist::Netlist;
use crate::parser::{Circuit, CircuitElement, Component, Port};

fn esp_circuit(pin: &str) -> Circuit {
    Circuit {
        elements: vec![
            CircuitElement::Component(Component {
                component_id: Some("U1".to_string()),
                kind: Some("chip".to_string()),
                name: Some("ESP32-S3".to_string()),
            }),
            CircuitElement::Port(Port {
                port_id: Some(format!("U1.{pin}")),
                source_component_id: Some("U1".to_string()),
                name: Some(pin.to_string()),
                ..Default::default()
            }),
        ],
        raw_elements: Vec::new(),
    }
}

fn violations(circuit: &Circuit, mcu: Option<&str>) -> Vec<Diagnostic> {
    let netlist = Netlist::from_circuit(circuit);
    let ctx = LintContext::new(circuit, &netlist).with_mcu(mcu);
    PinMuxRule.check(&ctx)
}

#[test]
fn explicit_unknown_mcu_blows_the_fuse() {
    let v = violations(&esp_circuit("GPIO4"), Some("stm32f103"));
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].rule, "unknown_mcu");
    assert!(v[0].message.contains("stm32f103"));
}

#[test]
fn gpio_designator_missing_from_database_is_reported() {
    let v = violations(&esp_circuit("GPIO22"), Some("esp32-s3"));
    assert_eq!(v.len(), 1);
    assert!(v[0].message.contains("GPIO22"));
}

#[test]
fn non_gpio_package_pins_are_ignored() {
    assert!(violations(&esp_circuit("VDD3V3"), Some("esp32-s3")).is_empty());
}
