//! STM32G431 port B alternate function tables (DS12589 Rev 6, Table 13).

use crate::silicon::model::{MuxFunction, PeripheralSignal};

pub(super) static PB0_FUNCS: &[MuxFunction] =
    &[mux!(PeripheralSignal::AdcChannel(1, 15), 0, "ADC")];
pub(super) static PB1_FUNCS: &[MuxFunction] =
    &[mux!(PeripheralSignal::AdcChannel(1, 12), 0, "ADC")];
pub(super) static PB2_FUNCS: &[MuxFunction] =
    &[mux!(PeripheralSignal::AdcChannel(2, 12), 0, "ADC")];
pub(super) static PB3_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiSck(1), 5, "AF5"),
    mux!(PeripheralSignal::SpiSck(3), 6, "AF6"),
    mux!(PeripheralSignal::UartTx(2), 7, "AF7"),
];
pub(super) static PB4_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMiso(1), 5, "AF5"),
    mux!(PeripheralSignal::SpiMiso(3), 6, "AF6"),
    mux!(PeripheralSignal::UartRx(2), 7, "AF7"),
];
pub(super) static PB5_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMosi(1), 5, "AF5"),
    mux!(PeripheralSignal::SpiMosi(3), 6, "AF6"),
    mux!(PeripheralSignal::I2cSda(3), 8, "AF8"),
];
pub(super) static PB6_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartTx(1), 7, "AF7")];
pub(super) static PB7_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::I2cSda(1), 4, "AF4"),
    mux!(PeripheralSignal::UartRx(1), 7, "AF7"),
    mux!(PeripheralSignal::UartCts(4), 14, "AF14"),
];
pub(super) static PB8_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::I2cScl(1), 4, "AF4"),
    mux!(PeripheralSignal::UartRx(3), 7, "AF7"),
];
pub(super) static PB9_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::I2cSda(1), 4, "AF4"),
    mux!(PeripheralSignal::UartTx(3), 7, "AF7"),
];
pub(super) static PB10_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartTx(3), 7, "AF7")];
pub(super) static PB11_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartRx(3), 7, "AF7"),
    mux!(PeripheralSignal::AdcChannel(1, 14), 0, "ADC"),
    mux!(PeripheralSignal::AdcChannel(2, 14), 0, "ADC"),
];
pub(super) static PB12_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiCs(2, 0), 5, "AF5"),
    mux!(PeripheralSignal::AdcChannel(1, 11), 0, "ADC"),
];
pub(super) static PB13_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiSck(2), 5, "AF5"),
    mux!(PeripheralSignal::UartCts(3), 7, "AF7"),
];
pub(super) static PB14_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMiso(2), 5, "AF5"),
    mux!(PeripheralSignal::UartRts(3), 7, "AF7"),
    mux!(PeripheralSignal::AdcChannel(1, 5), 0, "ADC"),
];
pub(super) static PB15_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMosi(2), 5, "AF5"),
    mux!(PeripheralSignal::AdcChannel(2, 15), 0, "ADC"),
];
