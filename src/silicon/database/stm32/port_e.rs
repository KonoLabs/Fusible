//! STM32G431 port E alternate function tables (DS12589 Rev 6, Table 13).

use crate::silicon::model::{MuxFunction, PeripheralSignal};

pub(super) static PE0_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartTx(1), 7, "AF7")];
pub(super) static PE1_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartRx(1), 7, "AF7")];
pub(super) static PE2_FUNCS: &[MuxFunction] = &[];
pub(super) static PE3_FUNCS: &[MuxFunction] = &[];
pub(super) static PE4_FUNCS: &[MuxFunction] = &[];
pub(super) static PE5_FUNCS: &[MuxFunction] = &[];
pub(super) static PE6_FUNCS: &[MuxFunction] = &[];
pub(super) static PE7_FUNCS: &[MuxFunction] = &[];
pub(super) static PE8_FUNCS: &[MuxFunction] = &[];
pub(super) static PE9_FUNCS: &[MuxFunction] = &[];
pub(super) static PE10_FUNCS: &[MuxFunction] = &[];
pub(super) static PE11_FUNCS: &[MuxFunction] = &[];
pub(super) static PE12_FUNCS: &[MuxFunction] = &[];
pub(super) static PE13_FUNCS: &[MuxFunction] = &[];
pub(super) static PE14_FUNCS: &[MuxFunction] = &[];
pub(super) static PE15_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartRx(3), 7, "AF7")];
