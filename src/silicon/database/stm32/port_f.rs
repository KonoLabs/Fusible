//! STM32G431 port F alternate function tables (DS12589 Rev 6, Table 13).

use crate::silicon::model::{MuxFunction, PeripheralSignal};

pub(super) static PF0_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::I2cSda(2), 4, "AF4"),
    mux!(PeripheralSignal::SpiCs(2, 0), 5, "AF5"),
    mux!(PeripheralSignal::AdcChannel(1, 10), 0, "ADC"),
];
pub(super) static PF1_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiSck(2), 5, "AF5"),
    mux!(PeripheralSignal::AdcChannel(2, 10), 0, "ADC"),
];
pub(super) static PF2_FUNCS: &[MuxFunction] = &[];
pub(super) static PF9_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::SpiSck(2), 5, "AF5")];
pub(super) static PF10_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::SpiSck(2), 5, "AF5")];
