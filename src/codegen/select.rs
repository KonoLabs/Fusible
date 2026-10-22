//! MCU component selection and pin number resolution.

use crate::parser::{Circuit, Component};
use crate::silicon::model::{parse_gpio_index, parse_port_pin};

const MCU_KEYWORDS: [&str; 6] = ["esp", "stm", "mcu", "soc", "rp2040", "u1"];

/// Picks the MCU component: the one matching `target_chip`, else the first
/// whose name or id holds an MCU keyword, else the first chip.
pub(crate) fn select_mcu_component<'a>(
    circuit: &'a Circuit,
    target_chip: Option<&str>,
) -> Option<&'a Component> {
    let comps: Vec<_> = circuit.components().collect();
    if comps.is_empty() {
        return None;
    }
    target_chip
        .and_then(|target| find_by_target(&comps, target))
        .or_else(|| find_by_keyword(&comps))
        .or_else(|| {
            comps.into_iter().find(|c| {
                c.kind.as_deref() == Some("chip") || c.kind.as_deref() == Some("simple_chip")
            })
        })
}

fn contains_lower(field: Option<&str>, needle: &str) -> bool {
    field
        .map(|v| v.to_ascii_lowercase().contains(needle))
        .unwrap_or(false)
}

fn find_by_target<'a>(comps: &[&'a Component], target: &str) -> Option<&'a Component> {
    let target_lower = target.to_ascii_lowercase();
    comps.iter().copied().find(|c| {
        contains_lower(c.name.as_deref(), &target_lower)
            || contains_lower(c.kind.as_deref(), &target_lower)
            || contains_lower(c.component_id.as_deref(), &target_lower)
    })
}

fn find_by_keyword<'a>(comps: &[&'a Component]) -> Option<&'a Component> {
    MCU_KEYWORDS.iter().find_map(|kw| {
        comps.iter().copied().find(|c| {
            contains_lower(c.name.as_deref(), kw) || contains_lower(c.component_id.as_deref(), kw)
        })
    })
}

/// Resolves the emitted pin number for a port label. See
/// [`PinMapping::pin_number`] for the encoding.
pub(super) fn parse_gpio_number(label: &str, explicit_pin: Option<u32>) -> Option<u32> {
    let clean = label.trim();

    if let Some((port, pin)) = parse_port_pin(clean) {
        return Some((port as u32 - 'A' as u32) * 16 + u32::from(pin));
    }

    if let Some(num) = parse_gpio_index(clean) {
        return Some(num);
    }

    let upper = clean.to_ascii_uppercase();
    for prefix in ["PIN", "P"] {
        if let Some(num) = upper
            .strip_prefix(prefix)
            .and_then(|rest| rest.parse::<u32>().ok())
        {
            return Some(num);
        }
    }

    clean.parse::<u32>().ok().or(explicit_pin)
}

pub(super) fn is_power_rail(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    upper == "GND"
        || upper == "VSS"
        || upper == "VCC"
        || upper == "VDD"
        || upper == "+3V3"
        || upper == "3V3"
        || upper == "+5V"
        || upper == "5V"
        || upper == "VBUS"
}
