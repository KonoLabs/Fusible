//! STM32G431 port A alternate function tables (DS12589 Rev 6, Table 13).

use crate::silicon::model::{MuxFunction, PeripheralSignal};

pub(super) static PA0_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartCts(2), 7, "AF7"),
    mux!(PeripheralSignal::AdcChannel(1, 1), 0, "ADC"),
    mux!(PeripheralSignal::AdcChannel(2, 1), 0, "ADC"),
];
pub(super) static PA1_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartRts(2), 7, "AF7"),
    mux!(PeripheralSignal::AdcChannel(1, 2), 0, "ADC"),
    mux!(PeripheralSignal::AdcChannel(2, 2), 0, "ADC"),
];
pub(super) static PA2_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartTx(2), 7, "AF7"),
    mux!(PeripheralSignal::AdcChannel(1, 3), 0, "ADC"),
];
pub(super) static PA3_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartRx(2), 7, "AF7"),
    mux!(PeripheralSignal::AdcChannel(1, 4), 0, "ADC"),
];
pub(super) static PA4_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiCs(1, 0), 5, "AF5"),
    mux!(PeripheralSignal::SpiCs(3, 0), 6, "AF6"),
    mux!(PeripheralSignal::AdcChannel(2, 17), 0, "ADC"),
];
pub(super) static PA5_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiSck(1), 5, "AF5"),
    mux!(PeripheralSignal::AdcChannel(2, 13), 0, "ADC"),
];
pub(super) static PA6_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMiso(1), 5, "AF5"),
    mux!(PeripheralSignal::AdcChannel(2, 3), 0, "ADC"),
];
pub(super) static PA7_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMosi(1), 5, "AF5"),
    mux!(PeripheralSignal::AdcChannel(2, 4), 0, "ADC"),
];
pub(super) static PA8_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::I2cScl(3), 2, "AF2"),
    mux!(PeripheralSignal::I2cSda(2), 4, "AF4"),
];
pub(super) static PA9_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::I2cScl(2), 4, "AF4"),
    mux!(PeripheralSignal::UartTx(1), 7, "AF7"),
];
pub(super) static PA10_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMiso(2), 5, "AF5"),
    mux!(PeripheralSignal::UartRx(1), 7, "AF7"),
];
pub(super) static PA11_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMosi(2), 5, "AF5"),
    mux!(PeripheralSignal::UartCts(1), 7, "AF7"),
];
pub(super) static PA12_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartRts(1), 7, "AF7")];
pub(super) static PA13_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SwdDio, 0, "AF0"),
    mux!(PeripheralSignal::I2cScl(1), 4, "AF4"),
    mux!(PeripheralSignal::UartCts(3), 7, "AF7"),
];
pub(super) static PA14_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SwdClk, 0, "AF0"),
    mux!(PeripheralSignal::I2cSda(1), 4, "AF4"),
    mux!(PeripheralSignal::UartTx(2), 7, "AF7"),
];
pub(super) static PA15_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::I2cScl(1), 4, "AF4"),
    mux!(PeripheralSignal::SpiCs(1, 0), 5, "AF5"),
    mux!(PeripheralSignal::SpiCs(3, 0), 6, "AF6"),
    mux!(PeripheralSignal::UartRx(2), 7, "AF7"),
    mux!(PeripheralSignal::UartRts(4), 8, "AF8"),
];
