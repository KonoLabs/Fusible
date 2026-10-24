//! Price estimates for lines the user has not priced, so a BOM never reads as free.

/// A printed body, with the weight measured from its model when one exists.
#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub name: String,
    pub grams: Option<f64>,
}

/// Typical hobby-quantity unit price in euros by component kind, value, and part number.
pub fn unit_price(kind: &str, value: Option<&str>, mpn: Option<&str>) -> Option<f64> {
    let kind = kind.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| kind.contains(w));
    match () {
        _ if has(&["chip", "mcu", "ic"]) => Some(chip_price(mpn.or(value))),
        _ if has(&["resistor"]) => Some(0.01),
        _ if has(&["capacitor"]) => Some(capacitor_price(value)),
        _ if has(&["inductor"]) => Some(0.15),
        _ if has(&["led"]) => Some(0.05),
        _ if has(&["diode"]) => Some(0.05),
        _ if has(&["transistor", "mosfet"]) => Some(0.10),
        _ if has(&["crystal", "resonator"]) => Some(0.30),
        _ if has(&["pushbutton", "switch", "button"]) => Some(0.15),
        _ if has(&["header", "pin"]) => Some(0.20),
        _ if has(&["connector", "jack", "usb"]) => Some(0.40),
        _ if has(&["battery", "buzzer", "fuse"]) => Some(0.50),
        _ => None,
    }
}

fn chip_price(name: Option<&str>) -> f64 {
    let name = name.unwrap_or_default().to_lowercase();
    let families: &[(&str, f64)] = &[
        ("esp32", 3.0),
        ("rp2040", 1.0),
        ("stm32", 2.5),
        ("atmega", 2.0),
        ("attiny", 1.0),
        ("nrf5", 3.5),
        ("lm", 0.5),
        ("ne555", 0.3),
    ];
    families
        .iter()
        .find(|(prefix, _)| name.contains(prefix))
        .map_or(1.5, |(_, price)| *price)
}

fn capacitor_price(value: Option<&str>) -> f64 {
    let microfarads = value.and_then(microfarads).unwrap_or(0.1);
    match microfarads {
        v if v < 1.0 => 0.02,
        v if v < 22.0 => 0.05,
        _ => 0.15,
    }
}

fn microfarads(text: &str) -> Option<f64> {
    let digits = text.trim().trim_end_matches(['F', 'f']);
    let (number, scale) = match digits.chars().last()? {
        'p' => (&digits[..digits.len() - 1], 1e-6),
        'n' => (&digits[..digits.len() - 1], 1e-3),
        'm' => (&digits[..digits.len() - 1], 1e3),
        'u' | 'µ' | 'μ' => (
            &digits[..digits.len() - digits.chars().last()?.len_utf8()],
            1.0,
        ),
        _ => (digits, 1.0),
    };
    number.trim().parse::<f64>().ok().map(|n| n * scale)
}
