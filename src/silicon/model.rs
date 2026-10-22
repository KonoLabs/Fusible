//! Data structures representing microcontrollers, pin capabilities, and alternate functions.

use std::fmt;

pub use super::designator::{is_gpio_designator, parse_gpio_index, parse_port_pin};
pub use super::signal::PeripheralSignal;

/// Strongly typed GPIO pin index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpioNum(pub u8);

impl fmt::Display for GpioNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GPIO{}", self.0)
    }
}

/// Reset logic requirements for boot strapping pins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrappingRequirement {
    /// Pin must be pulled logic HIGH at reset.
    MustBeHigh {
        purpose: &'static str,
        violation_consequence: &'static str,
    },
    /// Pin must be pulled logic LOW at reset.
    MustBeLow {
        purpose: &'static str,
        violation_consequence: &'static str,
    },
}

/// Silicon electrical capabilities for a physical pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PinCapabilities {
    pub is_gpio: bool,
    pub is_analog: bool,
    pub is_input_only: bool,
    pub is_reserved_flash: bool,
}

/// A multiplexed alternate function binding for a pin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuxFunction {
    pub signal: PeripheralSignal,
    pub af_number: u8,
    pub af_name: &'static str,
}

/// Static description of a physical pin on a microcontroller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinSpec {
    pub gpio: Option<GpioNum>,
    pub name: &'static str,
    pub capabilities: PinCapabilities,
    pub strapping: Option<StrappingRequirement>,
    pub functions: &'static [MuxFunction],
}

impl PinSpec {
    pub fn supports_signal(&self, signal: &PeripheralSignal) -> bool {
        self.functions.iter().any(|f| &f.signal == signal)
    }

    pub fn get_af(&self, signal: &PeripheralSignal) -> Option<&'static MuxFunction> {
        self.functions.iter().find(|f| &f.signal == signal)
    }
}

/// Silicon microcontroller definition with static pinout tables.
#[derive(Debug, Clone)]
pub struct SiliconTarget {
    pub name: &'static str,
    pub family: &'static str,
    pub pins: &'static [PinSpec],
}

impl SiliconTarget {
    pub fn find_pin(&self, name_or_gpio: &str) -> Option<&'static PinSpec> {
        let clean = name_or_gpio.trim().to_ascii_uppercase();
        self.pins.iter().find(|pin| {
            pin.name.eq_ignore_ascii_case(&clean)
                || pin.gpio.is_some_and(|g| gpio_matches(g, &clean))
        })
    }

    pub fn find_valid_pins_for_signal(&self, signal: &PeripheralSignal) -> Vec<&'static PinSpec> {
        self.pins
            .iter()
            .filter(|p| p.supports_signal(signal))
            .collect()
    }
}

/// True when `clean` names `gpio` as `GPIOn`, `GPn`, `IOn` or bare `n`.
fn gpio_matches(gpio: GpioNum, clean: &str) -> bool {
    format!("GPIO{}", gpio.0).eq_ignore_ascii_case(clean)
        || format!("GP{}", gpio.0).eq_ignore_ascii_case(clean)
        || format!("IO{}", gpio.0).eq_ignore_ascii_case(clean)
        || format!("{}", gpio.0) == clean
}
