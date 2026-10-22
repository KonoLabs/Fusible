//! Silicon pin-muxing, Alternate Function (AF) matrices, and hardware validation.

pub mod database;
pub mod model;
pub mod mux;

pub use database::find_target;
pub use model::{
    GpioNum, MuxFunction, PeripheralSignal, PinCapabilities, PinSpec, SiliconTarget,
    StrappingRequirement,
};
pub use mux::{validate_pin_assignments, AssignedPin, PinConflict};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_stm32g431_af_tables() {
        let chip = find_target("stm32g431").expect("stm32g431 not found");
        assert_eq!(chip.family, "STM32");

        let pa9 = chip.find_pin("PA9").expect("PA9 not found");
        let sig = PeripheralSignal::UartTx(1);
        assert!(pa9.supports_signal(&sig));
        let af = pa9.get_af(&sig).expect("AF for UART1_TX on PA9");
        assert_eq!(af.af_number, 7);
        assert_eq!(af.af_name, "AF7");

        // PA5 does NOT support USART1_TX (it is SPI1_SCK)
        let pa5 = chip.find_pin("PA5").expect("PA5 not found");
        assert!(!pa5.supports_signal(&sig));
    }

    #[test]
    fn resolves_rp2040_function_select() {
        let chip = find_target("rp2040").expect("rp2040 not found");
        assert_eq!(chip.family, "Raspberry Pi");

        let gp0 = chip.find_pin("GPIO0").expect("GPIO0 not found");
        assert!(gp0.supports_signal(&PeripheralSignal::UartTx(0)));
        assert!(gp0.supports_signal(&PeripheralSignal::SpiMiso(0)));

        // ADC is on GPIO26, not GPIO0
        assert!(!gp0.supports_signal(&PeripheralSignal::Adc(0)));
        let gp26 = chip.find_pin("GPIO26").expect("GPIO26 not found");
        assert!(gp26.supports_signal(&PeripheralSignal::Adc(0)));
    }

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

        // PA9, PB6, PC4 should be valid alternatives for UART1_TX
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

    fn assigned(pin: &str, net: &str, pulled_high: bool) -> AssignedPin {
        AssignedPin {
            pin_name: pin.to_string(),
            net_name: net.to_string(),
            signal_name: net.to_string(),
            is_pulled_to_gnd: false,
            is_pulled_high: pulled_high,
        }
    }

    #[test]
    fn find_target_matches_exact_chip_ids_only() {
        for known in [
            "esp32-s3",
            "ESP32_S3",
            "esp32s3",
            "rp2040",
            "stm32g431",
            "STM32G431CB",
        ] {
            assert!(find_target(known).is_some(), "{known} must resolve");
        }
        for unknown in [
            "esp32",
            "esp32-c3",
            "stm32f103",
            "stm32g431xyz",
            "pico",
            "pico2",
        ] {
            assert!(find_target(unknown).is_none(), "{unknown} must not resolve");
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
    fn esp32s3_database_covers_every_gpio() {
        let chip = find_target("esp32-s3").unwrap();
        for n in (0..=21).chain(26..=48) {
            assert!(chip.find_pin(&format!("GPIO{n}")).is_some(), "GPIO{n}");
        }
    }

    #[test]
    fn parses_stm32_adc_channel_naming() {
        assert_eq!(
            PeripheralSignal::parse("ADC1_IN3"),
            Some(PeripheralSignal::AdcChannel(1, 3))
        );
        let chip = find_target("stm32g431").unwrap();
        let conflicts = validate_pin_assignments(chip, &[assigned("PA2", "ADC1_IN3", false)]);
        assert!(conflicts.is_empty(), "{conflicts:?}");
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
}
