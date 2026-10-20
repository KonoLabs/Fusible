//! Data structures representing microcontrollers, pin capabilities, and alternate functions.

use std::fmt;

/// Strongly typed GPIO pin index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpioNum(pub u8);

impl fmt::Display for GpioNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GPIO{}", self.0)
    }
}

/// Known microcontroller hardware peripheral signals.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PeripheralSignal {
    UartTx(u8),
    UartRx(u8),
    UartCts(u8),
    UartRts(u8),
    SpiSck(u8),
    SpiMosi(u8),
    SpiMiso(u8),
    SpiCs(u8, u8),
    I2cScl(u8),
    I2cSda(u8),
    Pwm(u8, char),
    Adc(u8),
    Dac(u8),
    UsbDp,
    UsbDm,
    SwdClk,
    SwdDio,
    Other(String),
}

impl PeripheralSignal {
    /// Canonical display string for the peripheral signal.
    pub fn name(&self) -> String {
        match self {
            Self::UartTx(i) => format!("UART{i}_TX"),
            Self::UartRx(i) => format!("UART{i}_RX"),
            Self::UartCts(i) => format!("UART{i}_CTS"),
            Self::UartRts(i) => format!("UART{i}_RTS"),
            Self::SpiSck(i) => format!("SPI{i}_SCK"),
            Self::SpiMosi(i) => format!("SPI{i}_MOSI"),
            Self::SpiMiso(i) => format!("SPI{i}_MISO"),
            Self::SpiCs(i, cs) => format!("SPI{i}_CS{cs}"),
            Self::I2cScl(i) => format!("I2C{i}_SCL"),
            Self::I2cSda(i) => format!("I2C{i}_SDA"),
            Self::Pwm(s, c) => format!("PWM{s}_{c}"),
            Self::Adc(ch) => format!("ADC{ch}"),
            Self::Dac(ch) => format!("DAC{ch}"),
            Self::UsbDp => "USB_DP".to_string(),
            Self::UsbDm => "USB_DM".to_string(),
            Self::SwdClk => "SWD_CLK".to_string(),
            Self::SwdDio => "SWD_DIO".to_string(),
            Self::Other(s) => s.clone(),
        }
    }

    /// Attempts to parse a peripheral signal from a net or pin function name.
    pub fn parse(name: &str) -> Option<Self> {
        let clean = name.trim().to_ascii_uppercase();

        // UART / USART
        if let Some(rest) = clean
            .strip_prefix("UART")
            .or_else(|| clean.strip_prefix("USART"))
            .or_else(|| clean.strip_prefix("U"))
        {
            if let Some(inst_char) = rest.chars().next() {
                if let Some(digit) = inst_char.to_digit(10) {
                    let inst = digit as u8;
                    let tail = &rest[1..].trim_start_matches('_');
                    if tail.contains("TX") {
                        return Some(Self::UartTx(inst));
                    }
                    if tail.contains("RX") {
                        return Some(Self::UartRx(inst));
                    }
                    if tail.contains("CTS") {
                        return Some(Self::UartCts(inst));
                    }
                    if tail.contains("RTS") {
                        return Some(Self::UartRts(inst));
                    }
                }
            }
        }

        // SPI
        if let Some(rest) = clean.strip_prefix("SPI") {
            if let Some(inst_char) = rest.chars().next() {
                if let Some(digit) = inst_char.to_digit(10) {
                    let inst = digit as u8;
                    let tail = &rest[1..].trim_start_matches('_');
                    if tail.contains("SCK") || tail.contains("CLK") {
                        return Some(Self::SpiSck(inst));
                    }
                    if tail.contains("MOSI") || tail.contains("TX") {
                        return Some(Self::SpiMosi(inst));
                    }
                    if tail.contains("MISO") || tail.contains("RX") {
                        return Some(Self::SpiMiso(inst));
                    }
                    if tail.contains("CS") || tail.contains("SS") {
                        return Some(Self::SpiCs(inst, 0));
                    }
                }
            }
        }

        // I2C
        if let Some(rest) = clean.strip_prefix("I2C") {
            if let Some(inst_char) = rest.chars().next() {
                if let Some(digit) = inst_char.to_digit(10) {
                    let inst = digit as u8;
                    let tail = &rest[1..].trim_start_matches('_');
                    if tail.contains("SCL") || tail.contains("CLK") {
                        return Some(Self::I2cScl(inst));
                    }
                    if tail.contains("SDA") || tail.contains("DAT") {
                        return Some(Self::I2cSda(inst));
                    }
                }
            }
        }

        // ADC
        if let Some(rest) = clean.strip_prefix("ADC") {
            let tail = rest.trim_start_matches('_').trim_start_matches("IN");
            if let Ok(ch) = tail.parse::<u8>() {
                return Some(Self::Adc(ch));
            }
        }

        // USB
        if clean.contains("USB_DP") || clean == "USB_D+" || clean == "DP" {
            return Some(Self::UsbDp);
        }
        if clean.contains("USB_DM") || clean == "USB_D-" || clean == "DM" {
            return Some(Self::UsbDm);
        }

        None
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
        for pin in self.pins {
            if pin.name.eq_ignore_ascii_case(&clean) {
                return Some(pin);
            }
            if let Some(gpio) = pin.gpio {
                if format!("GPIO{}", gpio.0).eq_ignore_ascii_case(&clean)
                    || format!("GP{}", gpio.0).eq_ignore_ascii_case(&clean)
                    || format!("IO{}", gpio.0).eq_ignore_ascii_case(&clean)
                    || format!("{}", gpio.0) == clean
                {
                    return Some(pin);
                }
            }
        }
        None
    }

    pub fn find_valid_pins_for_signal(&self, signal: &PeripheralSignal) -> Vec<&'static PinSpec> {
        self.pins
            .iter()
            .filter(|p| p.supports_signal(signal))
            .collect()
    }
}
