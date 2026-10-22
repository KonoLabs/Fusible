//! Pin-mux conflict detection tests.

use crate::silicon::*;

#[test]
fn flags_pin_collision_when_two_signals_share_same_pin() {
    let chip = find_target("rp2040").unwrap();
    let assignments = vec![
        AssignedPin {
            pin_name: "GPIO0".to_string(),
            net_name: "UART_TX".to_string(),
            signal_name: "UART0_TX".to_string(),
            is_pulled_to_gnd: false,
            is_pulled_high: false,
        },
        AssignedPin {
            pin_name: "GPIO0".to_string(),
            net_name: "SPI_MISO".to_string(),
            signal_name: "SPI0_MISO".to_string(),
            is_pulled_to_gnd: false,
            is_pulled_high: false,
        },
    ];

    let conflicts = validate_pin_assignments(chip, &assignments);
    assert!(conflicts
        .iter()
        .any(|c| matches!(c, PinConflict::PinCollision { .. })));
}

#[test]
fn flags_unsupported_af_function_with_alternatives() {
    let chip = find_target("stm32g431").unwrap();
    let assignments = vec![AssignedPin {
        pin_name: "PA5".to_string(),
        net_name: "DEBUG_TX".to_string(),
        signal_name: "UART1_TX".to_string(),
        is_pulled_to_gnd: false,
        is_pulled_high: false,
    }];

    let conflicts = validate_pin_assignments(chip, &assignments);
    let unsupported = conflicts
        .iter()
        .find_map(|c| match c {
            PinConflict::UnsupportedFunction {
                valid_alternatives, ..
            } => Some(valid_alternatives),
            _ => None,
        })
        .expect("expected unsupported function violation");

    let alt_names: Vec<_> = unsupported.iter().map(|p| p.name).collect();
    assert!(alt_names.contains(&"PA9"));
    assert!(alt_names.contains(&"PB6"));
}

#[test]
fn flags_esp32_strapping_violation_on_gpio0() {
    let chip = find_target("esp32-s3").unwrap();
    let assignments = vec![AssignedPin {
        pin_name: "GPIO0".to_string(),
        net_name: "LED_STATUS".to_string(),
        signal_name: "LED_STATUS".to_string(),
        is_pulled_to_gnd: true,
        is_pulled_high: false,
    }];

    let conflicts = validate_pin_assignments(chip, &assignments);
    assert!(conflicts
        .iter()
        .any(|c| matches!(c, PinConflict::StrappingConflict { .. })));
}

pub(super) fn assigned(pin: &str, net: &str, pulled_high: bool) -> AssignedPin {
    AssignedPin {
        pin_name: pin.to_string(),
        net_name: net.to_string(),
        signal_name: net.to_string(),
        is_pulled_to_gnd: false,
        is_pulled_high: pulled_high,
    }
}

#[test]
fn flags_must_be_low_strapping_pin_pulled_high() {
    let chip = find_target("esp32-s3").unwrap();
    let conflicts = validate_pin_assignments(chip, &[assigned("GPIO45", "VDD3V3", true)]);
    assert!(conflicts
        .iter()
        .any(|c| matches!(c, PinConflict::StrappingConflict { .. })));
}

#[test]
fn reports_pin_missing_from_database() {
    let chip = find_target("esp32-s3").unwrap();
    let conflicts = validate_pin_assignments(chip, &[assigned("GPIO22", "SIG", false)]);
    assert_eq!(
        conflicts,
        vec![PinConflict::UnknownPin {
            pin_name: "GPIO22".to_string(),
            chip_name: "esp32-s3",
        }]
    );
}

#[test]
fn collisions_are_reported_in_pin_order() {
    let chip = find_target("rp2040").unwrap();
    let pins: Vec<_> = ["GPIO9", "GPIO1", "GPIO5"]
        .iter()
        .flat_map(|p| [assigned(p, "A", false), assigned(p, "B", false)])
        .collect();
    let order: Vec<_> = validate_pin_assignments(chip, &pins)
        .into_iter()
        .filter_map(|c| match c {
            PinConflict::PinCollision { pin_name, .. } => Some(pin_name),
            _ => None,
        })
        .collect();
    assert_eq!(order, ["GPIO1", "GPIO5", "GPIO9"]);
}

#[test]
fn flags_esp32_reserved_flash_pins() {
    let chip = find_target("esp32-s3").unwrap();
    let assignments = vec![AssignedPin {
        pin_name: "GPIO26".to_string(),
        net_name: "EXTRA_IO".to_string(),
        signal_name: "EXTRA_IO".to_string(),
        is_pulled_to_gnd: false,
        is_pulled_high: false,
    }];

    let conflicts = validate_pin_assignments(chip, &assignments);
    assert!(conflicts
        .iter()
        .any(|c| matches!(c, PinConflict::ReservedPinUsed { .. })));
}
