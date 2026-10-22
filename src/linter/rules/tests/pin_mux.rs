//! `pin_mux` rule tests.

use super::{net, rule_violations};
use crate::linter::rules::PinMuxRule;
use crate::parser::{CircuitElement, Component, Port};

/// A remote end of an MCU net: `(component, pin, port name)`.
type Remote<'a> = (&'a str, &'a str, &'a str);

/// A chip `U1` named `chip` whose `pin` sits on net `(id, name)` with `remote`.
fn chip_to_remote(
    chip: &str,
    pin: &str,
    net_def: (&str, &str),
    remote: Remote,
) -> Vec<CircuitElement> {
    let (net_id, net_name) = net_def;
    let (remote_comp, remote_pin, remote_name) = remote;
    let wired = |port_id: String, component: &str, name: &str| {
        CircuitElement::Port(Port {
            port_id: Some(port_id),
            source_component_id: Some(component.to_string()),
            name: Some(name.to_string()),
            net_id: Some(net_id.to_string()),
            ..Default::default()
        })
    };
    vec![
        CircuitElement::Component(Component {
            component_id: Some("U1".to_string()),
            name: Some(chip.to_string()),
            kind: Some("chip".to_string()),
        }),
        wired(format!("U1.{pin}"), "U1", pin),
        net(Some(net_id), net_name),
        wired(
            format!("{remote_comp}.{remote_pin}"),
            remote_comp,
            remote_name,
        ),
    ]
}

#[test]
fn pin_mux_rule_detects_invalid_alternate_function() {
    let elements = chip_to_remote(
        "STM32G431",
        "PA5",
        ("net_tx", "UART1_TX"),
        ("J1", "1", "RX"),
    );

    let v = rule_violations(PinMuxRule, elements);

    assert_eq!(v.len(), 1);
    assert_eq!(v[0].rule, "pin_mux");
    assert!(v[0].message.contains("PA5"));
    assert!(v[0].message.contains("UART1_TX"));
    assert!(v[0].help.as_ref().unwrap().contains("PA9"));
}

#[test]
fn pin_mux_rule_detects_esp32_strapping_conflict() {
    let elements = chip_to_remote(
        "ESP32-S3",
        "GPIO0",
        ("net_led", "LED_STATUS"),
        ("D1", "K", "K"),
    );

    let v = rule_violations(PinMuxRule, elements);

    assert_eq!(v.len(), 1);
    assert_eq!(v[0].rule, "pin_mux");
    assert!(v[0].message.contains("GPIO0"));
    assert!(v[0].message.contains("strapping pin"));
}

#[test]
fn pin_mux_rule_on_example_file() {
    let source = include_str!("../../../../examples/circuit-pinmux-violation.json");
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
