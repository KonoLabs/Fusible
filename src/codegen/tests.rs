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

fn mcu_with_nets(chip_name: &str, pins: &[(&str, &str, u32)]) -> Circuit {
    let mut elements = vec![CircuitElement::Component(Component {
        component_id: Some("U1".to_string()),
        kind: Some("chip".to_string()),
        name: Some(chip_name.to_string()),
    })];
    for (i, (label, net_name, pin_number)) in pins.iter().enumerate() {
        let net_id = format!("n{i}");
        elements.push(CircuitElement::Net(Net {
            net_id: Some(net_id.clone()),
            name: Some(net_name.to_string()),
        }));
        elements.push(CircuitElement::Port(Port {
            port_id: Some(format!("U1.{label}")),
            name: Some(label.to_string()),
            net_id: Some(net_id.clone()),
            source_component_id: Some("U1".to_string()),
            pin_number: Some(*pin_number),
            ..Default::default()
        }));
        elements.push(CircuitElement::Port(Port {
            port_id: Some(format!("J{i}.1")),
            name: Some("A".to_string()),
            net_id: Some(net_id),
            source_component_id: Some(format!("J{i}")),
            pin_number: Some(1),
            ..Default::default()
        }));
    }
    Circuit {
        elements,
        raw_elements: Vec::new(),
    }
}

fn generate(
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

#[test]
fn tricky_net_names_produce_valid_unique_identifiers() {
    let (info, code) = generate(
        "ESP32-S3",
        Some("esp32-s3"),
        &[
            ("GPIO1", "3V3_EN", 1),
            ("GPIO2", "IN", 2),
            ("GPIO48", "LED_R", 48),
            ("GPIO8", "LED_G", 8),
            ("GPIO9", "BATT\nfn evil() {}", 9),
            ("GPIO10", "SELF", 10),
            ("GPIO11", "DUP", 11),
            ("GPIO12", "DUP", 12),
        ],
    );

    let names: Vec<_> = info.pins.iter().map(|p| p.signal_name.as_str()).collect();
    assert_eq!(
        names,
        [
            "BATT_FN_EVIL",
            "DUP_11",
            "DUP_12",
            "IN",
            "LED_G",
            "LED_R",
            "PIN_3V3_EN",
            "SELF"
        ]
    );

    assert!(code.contains("pub const PIN_3V3_EN: u8 = 1;"));
    assert!(code.contains("pub in_: u8,"));
    assert!(code.contains("pub self_: u8,"));
    assert!(code.contains("pub pin_3v3_en: u8,"));
    assert!(code.contains("pub r: u8,"));
    assert!(code.contains("pub g: u8,"));
    assert!(code.contains("hardware net `BATT\\nfn evil() {}`"));
    assert!(code.contains("//! Board: Evil\\nfn injected() {}"));
    for line in code.lines() {
        assert!(!line.starts_with("fn "), "comment injection: {line}");
    }
}

#[test]
fn stm32_port_pins_use_port_times_sixteen_encoding() {
    let (info, code) = generate(
        "STM32G431",
        Some("stm32g431"),
        &[("PA9", "DEBUG_TX", 30), ("PB3", "SPI_SCK", 39)],
    );

    let by_label = |l: &str| {
        info.pins
            .iter()
            .find(|p| p.pin_label == l)
            .unwrap()
            .pin_number
    };
    assert_eq!(by_label("PA9"), 9);
    assert_eq!(by_label("PB3"), 19);
    assert!(code.contains("/// Pin 19 = `PB3` (port * 16 + pin)"));
}

#[test]
fn pin_numbers_above_u8_widen_the_constant_type() {
    let (_, code) = generate("BGA_SOC", None, &[("BALL_A", "SIG_HIGH", 300)]);
    assert!(code.contains("pub const SIG_HIGH: u16 = 300;"));
}

#[test]
fn known_silicon_skips_package_only_pins() {
    let (info, _) = generate(
        "ESP32-S3",
        Some("esp32-s3"),
        &[("GPIO2", "CAM_SIOD", 4), ("VDD3V3", "VDD3V3", 2)],
    );
    let labels: Vec<_> = info.pins.iter().map(|p| p.pin_label.as_str()).collect();
    assert_eq!(labels, ["GPIO2"]);
}
