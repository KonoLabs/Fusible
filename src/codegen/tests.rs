//! Tests for firmware pin bindings generator.

use super::*;
use crate::netlist::Netlist;
use crate::parser::{Circuit, CircuitElement, Component, Net, Port};

fn mcu_circuit_fixture() -> Circuit {
    Circuit {
        elements: vec![
            CircuitElement::Component(Component {
                component_id: Some("U1".to_string()),
                kind: Some("chip".to_string()),
                name: Some("ESP32-S3".to_string()),
            }),
            CircuitElement::Component(Component {
                component_id: Some("CAM1".to_string()),
                kind: Some("chip".to_string()),
                name: Some("OV2640".to_string()),
            }),
            CircuitElement::Net(Net {
                net_id: Some("n_cam_d0".to_string()),
                name: Some("CAM_D0".to_string()),
            }),
            CircuitElement::Port(Port {
                port_id: Some("U1.GPIO4".to_string()),
                name: Some("GPIO4".to_string()),
                net_id: Some("n_cam_d0".to_string()),
                source_component_id: Some("U1".to_string()),
                pin_number: Some(4),
                ..Default::default()
            }),
            CircuitElement::Port(Port {
                port_id: Some("CAM1.D0".to_string()),
                name: Some("D0".to_string()),
                net_id: Some("n_cam_d0".to_string()),
                source_component_id: Some("CAM1".to_string()),
                pin_number: Some(1),
                ..Default::default()
            }),
            CircuitElement::Net(Net {
                net_id: Some("n_modem_tx".to_string()),
                name: Some("MODEM_TX".to_string()),
            }),
            CircuitElement::Port(Port {
                port_id: Some("U1.GPIO43".to_string()),
                name: Some("GPIO43".to_string()),
                net_id: Some("n_modem_tx".to_string()),
                source_component_id: Some("U1".to_string()),
                pin_number: Some(43),
                ..Default::default()
            }),
            CircuitElement::Port(Port {
                port_id: Some("MODEM1.RXD".to_string()),
                name: Some("RXD".to_string()),
                net_id: Some("n_modem_tx".to_string()),
                source_component_id: Some("MODEM1".to_string()),
                pin_number: Some(2),
                ..Default::default()
            }),
        ],
        raw_elements: Vec::new(),
    }
}

#[test]
fn extracts_mcu_pins_and_generates_rust_code() {
    let circuit = mcu_circuit_fixture();
    let netlist = Netlist::from_circuit(&circuit);

    let mcu_info =
        extract_mcu_pins(&circuit, &netlist, Some("esp32-s3")).expect("MCU pins must be extracted");

    assert_eq!(mcu_info.name, "ESP32-S3");
    assert_eq!(mcu_info.pins.len(), 2);

    let cam_pin = mcu_info
        .pins
        .iter()
        .find(|p| p.signal_name == "CAM_D0")
        .expect("CAM_D0 pin must exist");
    assert_eq!(cam_pin.pin_number, 4);

    let modem_pin = mcu_info
        .pins
        .iter()
        .find(|p| p.signal_name == "MODEM_TX")
        .expect("MODEM_TX pin must exist");
    assert_eq!(modem_pin.pin_number, 43);

    let rust_code = generate_pins_rs(&mcu_info, "FieldCam Controller");
    assert!(rust_code.contains("pub mod gpio"));
    assert!(rust_code.contains("pub const CAM_D0: u8 = 4;"));
    assert!(rust_code.contains("pub const MODEM_TX: u8 = 43;"));
    assert!(rust_code.contains("pub struct CameraPins"));
    assert!(rust_code.contains("pub struct ModemPins"));
    assert!(rust_code.contains("pub const BOARD_PINS: BoardPins"));
}
