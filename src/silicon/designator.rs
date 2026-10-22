//! GPIO and port/pin designator parsing (`GPIO4`, `GP0`, `IO12`, `PA12`).

/// Parses an STM32-style port/pin designator such as `PA12` into
/// `(port_letter, pin_index)`. The port letter is uppercase `A`..=`K` and
/// the pin index is `0..=15`.
pub fn parse_port_pin(name: &str) -> Option<(char, u8)> {
    split_port_pin(name)
        .filter(|(_, pin)| *pin < 16)
        .map(|(port, pin)| (port, pin as u8))
}

fn split_port_pin(name: &str) -> Option<(char, u32)> {
    let clean = name.trim().to_ascii_uppercase();
    let rest = clean.strip_prefix('P')?;
    let port = rest.chars().next().filter(|c| ('A'..='K').contains(c))?;
    let digits = &rest[1..];
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse::<u32>().ok().map(|pin| (port, pin))
}

/// Returns the numeric GPIO index from `GPIOn`, `GPn` or `IOn` designators.
pub fn parse_gpio_index(name: &str) -> Option<u32> {
    let clean = name.trim().to_ascii_uppercase();
    ["GPIO", "GP", "IO"].iter().find_map(|prefix| {
        let digits = clean.strip_prefix(prefix)?;
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        digits.parse::<u32>().ok()
    })
}

/// True when `name` reads as a silicon GPIO designator (`GPIO4`, `GP0`,
/// `IO12`, `PA12`) rather than a power, reset, or package pin label.
pub fn is_gpio_designator(name: &str) -> bool {
    parse_gpio_index(name).is_some() || split_port_pin(name).is_some()
}
