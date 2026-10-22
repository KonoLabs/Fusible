//! STM32G431 pin table checks against DS12589 Rev 6, Table 13.

use crate::silicon::*;

fn af(pin: &str, signal: PeripheralSignal) -> Option<u8> {
    let chip = find_target("stm32g431").unwrap();
    chip.find_pin(pin)?.get_af(&signal).map(|f| f.af_number)
}

#[test]
fn covers_every_gpio_of_the_largest_package() {
    let chip = find_target("stm32g431vb").unwrap();
    let expected = (0..16)
        .flat_map(|n| ["PA", "PB", "PC", "PD", "PE"].map(|p| format!("{p}{n}")))
        .chain(["PF0", "PF1", "PF2", "PF9", "PF10", "PG10"].map(String::from));
    for pin in expected {
        assert!(chip.find_pin(&pin).is_some(), "{pin} missing");
    }
    assert_eq!(chip.pins.len(), 86);
}

#[test]
fn rejects_pins_that_do_not_exist() {
    let chip = find_target("stm32g431").unwrap();
    for pin in ["PA16", "PB16", "PC16", "PF3", "PG0", "PG9"] {
        assert!(chip.find_pin(pin).is_none(), "{pin} must not exist");
    }
}

#[test]
fn matches_datasheet_alternate_functions() {
    assert_eq!(af("PA9", PeripheralSignal::UartTx(1)), Some(7));
    assert_eq!(af("PA9", PeripheralSignal::I2cScl(2)), Some(4));
    assert_eq!(af("PB8", PeripheralSignal::I2cScl(1)), Some(4));
    assert_eq!(af("PB3", PeripheralSignal::SpiSck(1)), Some(5));
    assert_eq!(af("PB3", PeripheralSignal::SpiSck(3)), Some(6));
    assert_eq!(af("PC10", PeripheralSignal::UartTx(4)), Some(5));
    assert_eq!(af("PC10", PeripheralSignal::UartTx(3)), Some(7));
    assert_eq!(af("PA0", PeripheralSignal::UartCts(2)), Some(7));
    assert_eq!(af("PA13", PeripheralSignal::SwdDio), Some(0));
    assert_eq!(af("PA14", PeripheralSignal::SwdClk), Some(0));
}

#[test]
fn omits_signals_the_datasheet_does_not_list() {
    let chip = find_target("stm32g431").unwrap();
    let pc13 = chip.find_pin("PC13").unwrap();
    assert!(pc13.functions.iter().all(|f| f.af_number != 7));
    assert_eq!(af("PB6", PeripheralSignal::I2cScl(1)), None);
    assert_eq!(af("PA0", PeripheralSignal::UartTx(2)), None);
    assert_eq!(af("PA11", PeripheralSignal::UsbDm), None);
}

#[test]
fn marks_adc_pins_analog() {
    let chip = find_target("stm32g431").unwrap();
    assert!(chip.find_pin("PC4").unwrap().capabilities.is_analog);
    assert!(!chip.find_pin("PA9").unwrap().capabilities.is_analog);
    assert_eq!(af("PC4", PeripheralSignal::AdcChannel(2, 5)), Some(0));
    assert_eq!(af("PC0", PeripheralSignal::AdcChannel(1, 6)), Some(0));
    assert_eq!(af("PC0", PeripheralSignal::AdcChannel(2, 6)), Some(0));
}
