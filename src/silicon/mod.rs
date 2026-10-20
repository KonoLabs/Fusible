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
            },
            AssignedPin {
                pin_name: "GPIO0".to_string(),
                net_name: "SPI_MISO".to_string(),
                signal_name: "SPI0_MISO".to_string(),
                is_pulled_to_gnd: false,
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
        }];

        let conflicts = validate_pin_assignments(chip, &assignments);
        assert!(conflicts
            .iter()
            .any(|c| matches!(c, PinConflict::StrappingConflict { .. })));
    }

    #[test]
    fn flags_esp32_reserved_flash_pins() {
        let chip = find_target("esp32-s3").unwrap();
        let assignments = vec![AssignedPin {
            pin_name: "GPIO26".to_string(),
            net_name: "EXTRA_IO".to_string(),
            signal_name: "EXTRA_IO".to_string(),
            is_pulled_to_gnd: false,
        }];

        let conflicts = validate_pin_assignments(chip, &assignments);
        assert!(conflicts
            .iter()
            .any(|c| matches!(c, PinConflict::ReservedPinUsed { .. })));
    }
}
