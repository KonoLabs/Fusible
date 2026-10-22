//! Peripheral signal identities and their parsing from net or pin names.

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
    AdcChannel(u8, u8),
    Dac(u8),
    UsbDp,
    UsbDm,
    SwdClk,
    SwdDio,
    Other(String),
}

/// Tail keywords mapped to the signal constructor they select, tried in order.
type TailTable = [(&'static [&'static str], fn(u8) -> PeripheralSignal)];

const UART_TAILS: &TailTable = &[
    (&["TX"], PeripheralSignal::UartTx),
    (&["RX"], PeripheralSignal::UartRx),
    (&["CTS"], PeripheralSignal::UartCts),
    (&["RTS"], PeripheralSignal::UartRts),
];

const SPI_TAILS: &TailTable = &[
    (&["SCK", "CLK"], PeripheralSignal::SpiSck),
    (&["MOSI", "TX"], PeripheralSignal::SpiMosi),
    (&["MISO", "RX"], PeripheralSignal::SpiMiso),
    (&["CS", "SS"], |inst| PeripheralSignal::SpiCs(inst, 0)),
];

const I2C_TAILS: &TailTable = &[
    (&["SCL", "CLK"], PeripheralSignal::I2cScl),
    (&["SDA", "DAT"], PeripheralSignal::I2cSda),
];

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
            Self::AdcChannel(inst, ch) => format!("ADC{inst}_IN{ch}"),
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
        let uart = clean
            .strip_prefix("UART")
            .or_else(|| clean.strip_prefix("USART"))
            .or_else(|| clean.strip_prefix("U"));
        uart.and_then(|rest| parse_instance(rest, UART_TAILS))
            .or_else(|| parse_instance(clean.strip_prefix("SPI")?, SPI_TAILS))
            .or_else(|| parse_instance(clean.strip_prefix("I2C")?, I2C_TAILS))
            .or_else(|| parse_adc(clean.strip_prefix("ADC")?))
            .or_else(|| parse_usb(&clean))
    }
}

/// Parses `<digit>[_]<tail>` after a peripheral prefix, picking the first
/// table entry whose keyword appears in the tail.
fn parse_instance(rest: &str, table: &TailTable) -> Option<PeripheralSignal> {
    let inst = rest.chars().next()?.to_digit(10)? as u8;
    let tail = rest[1..].trim_start_matches('_');
    table
        .iter()
        .find(|(keys, _)| keys.iter().any(|k| tail.contains(k)))
        .map(|(_, signal)| signal(inst))
}

/// Parses `ADC<inst>_IN<ch>`, else `ADC[_][IN]<ch>`, after the `ADC` prefix.
fn parse_adc(rest: &str) -> Option<PeripheralSignal> {
    let channel = rest.split_once("_IN").and_then(|(inst, ch)| {
        Some(PeripheralSignal::AdcChannel(
            inst.parse().ok()?,
            ch.parse().ok()?,
        ))
    });
    channel.or_else(|| {
        let tail = rest.trim_start_matches('_').trim_start_matches("IN");
        tail.parse::<u8>().ok().map(PeripheralSignal::Adc)
    })
}

fn parse_usb(clean: &str) -> Option<PeripheralSignal> {
    if clean.contains("USB_DP") || clean == "USB_D+" || clean == "DP" {
        return Some(PeripheralSignal::UsbDp);
    }
    if clean.contains("USB_DM") || clean == "USB_D-" || clean == "DM" {
        return Some(PeripheralSignal::UsbDm);
    }
    None
}
