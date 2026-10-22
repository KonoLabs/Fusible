//! Peripheral grouping and struct field naming for generated bindings.

use std::collections::{BTreeMap, BTreeSet};

use super::naming::unique_name;
use super::pins::PinMapping;

/// Groups pins into peripheral buckets keyed by group name.
pub(super) fn categorize_pins(pins: &[PinMapping]) -> BTreeMap<String, Vec<&PinMapping>> {
    let mut groups: BTreeMap<String, Vec<&PinMapping>> = BTreeMap::new();
    for pin in pins {
        groups
            .entry(group_of(&pin.signal_name))
            .or_default()
            .push(pin);
    }
    groups
}

fn group_of(signal_name: &str) -> String {
    let upper = signal_name.to_ascii_uppercase();
    let has_any = |keys: &[&str]| keys.iter().any(|k| upper.contains(k));
    let group = if upper.contains("CAM") {
        "Camera"
    } else if has_any(&["MODEM", "SIM", "LTE"]) {
        "Modem"
    } else if has_any(&["RADIO", "RF", "NRF"]) {
        "Radio"
    } else if has_any(&["BATT", "POWER", "ADC"]) {
        "Power"
    } else if upper.contains("LED") {
        "Led"
    } else {
        "General"
    };
    group.to_string()
}

/// Unique struct field names for one peripheral group, in pin order.
pub(super) fn field_names(pins: &[&PinMapping], group_name: &str) -> Vec<String> {
    let mut used = BTreeSet::new();
    pins.iter()
        .map(|pin| unique_name(to_field_name(&pin.signal_name, group_name), &mut used))
        .collect()
}

/// Lowercase field name with the group prefix stripped (`MODEM_TX` in
/// `Modem` -> `tx`). Leading digits get a `pin_` prefix and Rust keywords
/// get a `_` suffix (`in` -> `in_`), which also covers `self`/`crate`/`super`
/// that cannot be raw identifiers.
fn to_field_name(signal_name: &str, group_name: &str) -> String {
    let mut clean = signal_name.to_ascii_lowercase();

    let prefix = format!("{}_", group_name.to_ascii_lowercase());
    if let Some(stripped) = clean.strip_prefix(&prefix) {
        clean = stripped.to_string();
    }

    if clean.starts_with(|c: char| c.is_ascii_digit()) {
        format!("pin_{clean}")
    } else if is_rust_keyword(&clean) {
        format!("{clean}_")
    } else {
        clean
    }
}

fn is_rust_keyword(s: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
        "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl",
        "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub",
        "ref", "return", "self", "static", "struct", "super", "trait", "true", "try", "type",
        "typeof", "union", "unsafe", "unsized", "use", "virtual", "where", "while", "yield", "_",
    ];
    KEYWORDS.contains(&s)
}
