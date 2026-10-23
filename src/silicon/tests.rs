//! Silicon database, signal parsing, and pin-mux validation tests.

mod mux;
mod packages;
mod stm32;

use mux::assigned;

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

    let pa5 = chip.find_pin("PA5").expect("PA5 not found");
    assert!(!pa5.supports_signal(&sig), "PA5 is SPI1_SCK, not USART1_TX");
}

#[test]
fn resolves_rp2040_function_select() {
    let chip = find_target("rp2040").expect("rp2040 not found");
    assert_eq!(chip.family, "Raspberry Pi");

    let gp0 = chip.find_pin("GPIO0").expect("GPIO0 not found");
    assert!(gp0.supports_signal(&PeripheralSignal::UartTx(0)));
    assert!(gp0.supports_signal(&PeripheralSignal::SpiMiso(0)));

    assert!(
        !gp0.supports_signal(&PeripheralSignal::Adc(0)),
        "ADC is on GPIO26, not GPIO0"
    );
    let gp26 = chip.find_pin("GPIO26").expect("GPIO26 not found");
    assert!(gp26.supports_signal(&PeripheralSignal::Adc(0)));
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
