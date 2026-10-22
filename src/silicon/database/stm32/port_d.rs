//! STM32G431 port D alternate function tables (DS12589 Rev 6, Table 13).

use crate::silicon::model::{MuxFunction, PeripheralSignal};

pub(super) static PD0_FUNCS: &[MuxFunction] = &[];
pub(super) static PD1_FUNCS: &[MuxFunction] = &[];
pub(super) static PD2_FUNCS: &[MuxFunction] = &[];
pub(super) static PD3_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartCts(2), 7, "AF7")];
pub(super) static PD4_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartRts(2), 7, "AF7")];
pub(super) static PD5_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartTx(2), 7, "AF7")];
pub(super) static PD6_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartRx(2), 7, "AF7")];
pub(super) static PD7_FUNCS: &[MuxFunction] = &[];
pub(super) static PD8_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartTx(3), 7, "AF7")];
pub(super) static PD9_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartRx(3), 7, "AF7")];
pub(super) static PD10_FUNCS: &[MuxFunction] = &[];
pub(super) static PD11_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartCts(3), 7, "AF7")];
pub(super) static PD12_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartRts(3), 7, "AF7")];
pub(super) static PD13_FUNCS: &[MuxFunction] = &[];
pub(super) static PD14_FUNCS: &[MuxFunction] = &[];
pub(super) static PD15_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::SpiCs(2, 0), 6, "AF6")];
