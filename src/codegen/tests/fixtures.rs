//! Circuit builders shared by the pin bindings generator tests.

use crate::codegen::{extract_mcu_pins, generate_pins_rs, McuInfo};
use crate::netlist::Netlist;
use crate::parser::{Circuit, CircuitElement, Component, Net, Port};

/// A `chip` component.
pub fn chip(component_id: &str, name: &str) -> CircuitElement {
    CircuitElement::Component(Component {
        component_id: Some(component_id.to_string()),
        kind: Some("chip".to_string()),
        name: Some(name.to_string()),
    })
}

/// A declared net.
pub fn net(net_id: &str, name: &str) -> CircuitElement {
    CircuitElement::Net(Net {
        net_id: Some(net_id.to_string()),
        name: Some(name.to_string()),
    })
}

/// A port of `component` on `net_id`.
pub fn port(port_id: &str, name: &str, net_id: &str, component: &str, pin: u32) -> CircuitElement {
    CircuitElement::Port(Port {
        port_id: Some(port_id.to_string()),
        name: Some(name.to_string()),
        net_id: Some(net_id.to_string()),
        source_component_id: Some(component.to_string()),
        pin_number: Some(pin),
        ..Default::default()
    })
}

fn circuit(elements: Vec<CircuitElement>) -> Circuit {
    Circuit {
        elements,
        raw_elements: Vec::new(),
    }
}

/// An ESP32-S3 wired to a camera data line and a modem TX line.
pub fn mcu_circuit_fixture() -> Circuit {
    circuit(vec![
        chip("U1", "ESP32-S3"),
        chip("CAM1", "OV2640"),
        net("n_cam_d0", "CAM_D0"),
        port("U1.GPIO4", "GPIO4", "n_cam_d0", "U1", 4),
        port("CAM1.D0", "D0", "n_cam_d0", "CAM1", 1),
        net("n_modem_tx", "MODEM_TX"),
        port("U1.GPIO43", "GPIO43", "n_modem_tx", "U1", 43),
        port("MODEM1.RXD", "RXD", "n_modem_tx", "MODEM1", 2),
    ])
}

/// An MCU `U1` whose `(label, net_name, pin_number)` pins each reach a
/// connector `J<i>` over their own net.
pub fn mcu_with_nets(chip_name: &str, pins: &[(&str, &str, u32)]) -> Circuit {
    let mut elements = vec![chip("U1", chip_name)];
    for (i, (label, net_name, pin_number)) in pins.iter().enumerate() {
        let net_id = format!("n{i}");
        elements.push(net(&net_id, net_name));
        elements.push(port(
            &format!("U1.{label}"),
            label,
            &net_id,
            "U1",
            *pin_number,
        ));
        elements.push(port(&format!("J{i}.1"), "A", &net_id, &format!("J{i}"), 1));
    }
    circuit(elements)
}

/// Extracts the MCU pins of [`mcu_with_nets`] and renders them under a board
/// name that attempts comment injection.
pub fn generate(
    chip_name: &str,
    target: Option<&str>,
    pins: &[(&str, &str, u32)],
) -> (McuInfo, String) {
    let circuit = mcu_with_nets(chip_name, pins);
    let netlist = Netlist::from_circuit(&circuit);
    let info = extract_mcu_pins(&circuit, &netlist, target).expect("MCU must be found");
    let code = generate_pins_rs(&info, "Evil\nfn injected() {}");
    (info, code)
}
