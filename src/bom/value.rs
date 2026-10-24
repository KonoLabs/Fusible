//! Human values of a `source_component`: display strings, or SI-formatted numbers.

use serde_json::{Map, Value};

const DISPLAY_KEYS: &[&str] = &[
    "display_value",
    "display_resistance",
    "display_capacitance",
    "display_inductance",
];
const UNITS: &[(&str, &str)] = &[
    ("resistance", "Ω"),
    ("capacitance", "F"),
    ("inductance", "H"),
];
const PREFIXES: &[(f64, &str)] = &[
    (1e9, "G"),
    (1e6, "M"),
    (1e3, "k"),
    (1.0, ""),
    (1e-3, "m"),
    (1e-6, "µ"),
    (1e-9, "n"),
    (1e-12, "p"),
];

fn text<'a>(object: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    object
        .get(key)?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// Formats `value` with an SI prefix, e.g. `4700.0` and `Ω` give `4.7kΩ`.
pub(super) fn si(value: f64, unit: &str) -> String {
    let (scale, prefix) = PREFIXES
        .iter()
        .find(|(scale, _)| value.abs() >= *scale)
        .copied()
        .unwrap_or(PREFIXES[PREFIXES.len() - 1]);
    let scaled = (value / scale * 1000.0).round() / 1000.0;
    format!("{scaled}{prefix}{unit}")
}

/// The value shown for a component, from a display string or a raw number.
pub(super) fn value_of(object: &Map<String, Value>) -> Option<String> {
    if let Some(shown) = DISPLAY_KEYS.iter().find_map(|key| text(object, key)) {
        return Some(shown.to_string());
    }
    UNITS.iter().find_map(|(key, unit)| {
        let number = object.get(*key)?.as_f64().filter(|n| *n != 0.0)?;
        Some(si(number, unit))
    })
}

/// The manufacturer part number, when the component carries one.
pub(super) fn mpn_of(object: &Map<String, Value>) -> Option<String> {
    text(object, "manufacturer_part_number")
        .or_else(|| text(object, "mpn"))
        .map(str::to_string)
}
