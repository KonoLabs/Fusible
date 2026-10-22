//! STM32G431 port C alternate function tables (DS12589 Rev 6, Table 13).

use crate::silicon::model::{MuxFunction, PeripheralSignal};

pub(super) static PC0_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::AdcChannel(1, 6), 0, "ADC"),
    mux!(PeripheralSignal::AdcChannel(2, 6), 0, "ADC"),
];
pub(super) static PC1_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::AdcChannel(1, 7), 0, "ADC"),
    mux!(PeripheralSignal::AdcChannel(2, 7), 0, "ADC"),
];
pub(super) static PC2_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::AdcChannel(1, 8), 0, "ADC"),
    mux!(PeripheralSignal::AdcChannel(2, 8), 0, "ADC"),
];
pub(super) static PC3_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::AdcChannel(1, 9), 0, "ADC"),
    mux!(PeripheralSignal::AdcChannel(2, 9), 0, "ADC"),
];
pub(super) static PC4_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::I2cScl(2), 4, "AF4"),
    mux!(PeripheralSignal::UartTx(1), 7, "AF7"),
    mux!(PeripheralSignal::AdcChannel(2, 5), 0, "ADC"),
];
pub(super) static PC5_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartRx(1), 7, "AF7"),
    mux!(PeripheralSignal::AdcChannel(2, 11), 0, "ADC"),
];
pub(super) static PC6_FUNCS: &[MuxFunction] = &[];
pub(super) static PC7_FUNCS: &[MuxFunction] = &[];
pub(super) static PC8_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::I2cScl(3), 8, "AF8")];
pub(super) static PC9_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::I2cSda(3), 8, "AF8")];
pub(super) static PC10_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartTx(4), 5, "AF5"),
    mux!(PeripheralSignal::SpiSck(3), 6, "AF6"),
    mux!(PeripheralSignal::UartTx(3), 7, "AF7"),
];
pub(super) static PC11_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartRx(4), 5, "AF5"),
    mux!(PeripheralSignal::SpiMiso(3), 6, "AF6"),
    mux!(PeripheralSignal::UartRx(3), 7, "AF7"),
    mux!(PeripheralSignal::I2cSda(3), 8, "AF8"),
];
pub(super) static PC12_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::SpiMosi(3), 6, "AF6")];
pub(super) static PC13_FUNCS: &[MuxFunction] = &[];
pub(super) static PC14_FUNCS: &[MuxFunction] = &[];
pub(super) static PC15_FUNCS: &[MuxFunction] = &[];
