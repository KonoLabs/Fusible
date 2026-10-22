//! Tests for firmware pin bindings generator.

mod fixtures;

use super::*;
use crate::netlist::Netlist;
use fixtures::{generate, mcu_circuit_fixture};

const TRICKY_PINS: &[(&str, &str, u32)] = &[
    ("GPIO1", "3V3_EN", 1),
    ("GPIO2", "IN", 2),
    ("GPIO48", "LED_R", 48),
    ("GPIO8", "LED_G", 8),
    ("GPIO9", "BATT\nfn evil() {}", 9),
    ("GPIO10", "SELF", 10),
    ("GPIO11", "DUP", 11),
    ("GPIO12", "DUP", 12),
];

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

#[test]
fn tricky_net_names_produce_valid_unique_identifiers() {
    let (info, code) = generate("ESP32-S3", Some("esp32-s3"), TRICKY_PINS);

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
    assert_tricky_code_is_safe(&code);
}

fn assert_tricky_code_is_safe(code: &str) {
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
