use super::harness::link_violations;
use crate::config::FusibleConfig;
use crate::parser::{Circuit, CircuitElement, Component, Net, Port};

fn board(refdes: &str, pins: &[(u32, Option<&str>)]) -> Circuit {
    let mut elements = vec![CircuitElement::Component(Component {
        component_id: Some(format!("c_{refdes}")),
        name: Some(refdes.into()),
        ..Default::default()
    })];
    for (number, net) in pins {
        let net_id = net.map(|n| format!("net_{n}"));
        if let Some(name) = net {
            elements.push(CircuitElement::Net(Net {
                net_id: net_id.clone(),
                name: Some((*name).into()),
            }));
        }
        elements.push(CircuitElement::Port(Port {
            port_id: Some(format!("{refdes}.{number}")),
            source_component_id: Some(format!("c_{refdes}")),
            pin_number: Some(*number),
            net_id,
            ..Default::default()
        }));
    }
    Circuit {
        elements,
        raw_elements: Vec::new(),
    }
}

fn violations(yaml_wires: &str, from: &Circuit, to: &Circuit) -> Vec<String> {
    let manifest = format!(
        "boards: {{main: {{}}, sensor: {{}}}}\nlinks:\n  uart:\n    from: main.J2\n    to: sensor.J1\n\
         \x20   wires: {yaml_wires}"
    );
    let cfg = FusibleConfig::from_str(&manifest).expect("manifest");
    link_violations("uart", &cfg.links["uart"], from, to)
        .into_iter()
        .map(|d| d.message)
        .collect()
}

#[test]
fn matching_nets_pass() {
    let a = board("J2", &[(1, Some("GND")), (2, Some("TX"))]);
    let b = board("J1", &[(1, Some("GND")), (3, Some("RX"))]);
    let wires = "[{a: 1, b: 1}, {a: 2, b: 3, nets: [TX, RX]}]";
    assert!(violations(wires, &a, &b).is_empty());
}

#[test]
fn different_net_names_blow_the_fuse() {
    let a = board("J2", &[(1, Some("GND"))]);
    let b = board("J1", &[(1, Some("3V3"))]);
    let found = violations("[{a: 1, b: 1}]", &a, &b);
    assert_eq!(found.len(), 1);
    assert!(found[0].contains("carries GND") && found[0].contains("carries 3V3"));
}

#[test]
fn a_declared_net_must_match_each_end() {
    let a = board("J2", &[(2, Some("TX"))]);
    let b = board("J1", &[(3, Some("TX"))]);
    assert_eq!(
        violations("[{a: 2, b: 3, nets: [TX, RX]}]", &a, &b).len(),
        1
    );
}

#[test]
fn missing_part_missing_pin_and_floating_pin_fail_closed() {
    let a = board("J2", &[(1, Some("GND")), (4, None)]);
    let b = board("J1", &[(1, Some("GND"))]);
    assert!(violations("[{a: 9, b: 1}]", &a, &b)[0].contains("does not exist"));
    assert!(violations("[{a: 4, b: 1}]", &a, &b)[0].contains("not connected"));
    let other = board("J9", &[(1, Some("GND"))]);
    assert!(violations("[{a: 1, b: 1}]", &a, &other)[0].contains("has no part J1"));
}
