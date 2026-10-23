//! MCU component selection and pin number resolution.

use crate::parser::{Circuit, Component};
mod fields;

use crate::silicon::model::{parse_gpio_index, parse_port_pin};
use fields::{descriptors, manufacturer_part_number, matches_target, names_database_part};

/// Picks the MCU component among the board's chips, never a passive.
///
/// Order: the chip matching `target_chip` by part number, kind, name or id;
/// else the only chip naming a part of the silicon database; else the only
/// chip. With several candidates and no way to choose, returns `None` and
/// prints a hint naming them.
pub(crate) fn select_mcu_component<'a>(
    circuit: &'a Circuit,
    target_chip: Option<&str>,
) -> Option<&'a Component> {
    let chips: Vec<&Component> = circuit.components().filter(|c| is_chip(c)).collect();
    let fields = |c: &Component| descriptors(circuit, c);
    let picked = target_chip
        .and_then(|target| {
            chips
                .iter()
                .copied()
                .find(|c| fields(c).iter().any(|f| matches_target(f, target)))
        })
        .or_else(|| {
            only(
                chips
                    .iter()
                    .copied()
                    .filter(|c| fields(c).iter().any(|f| names_database_part(f))),
            )
        })
        .or_else(|| only(chips.iter().copied()));
    if picked.is_none() && chips.len() > 1 {
        eprintln!("{}", hint_message(circuit, &chips));
    }
    picked
}

fn is_chip(component: &Component) -> bool {
    matches!(
        component.kind.as_deref(),
        Some("chip") | Some("simple_chip")
    )
}

fn only<'a>(mut candidates: impl Iterator<Item = &'a Component>) -> Option<&'a Component> {
    let first = candidates.next()?;
    candidates.next().is_none().then_some(first)
}

pub(crate) fn hint_message(circuit: &Circuit, chips: &[&Component]) -> String {
    let labels: Vec<String> = chips.iter().map(|c| chip_label(circuit, c)).collect();
    format!(
        "hint: several chips on the board ({}), none picked as the MCU; \
         set `mcu.chip` in fusible.yml or pass --mcu",
        labels.join(", ")
    )
}

fn chip_label(circuit: &Circuit, chip: &Component) -> String {
    let name = chip
        .name
        .as_deref()
        .or(chip.component_id.as_deref())
        .unwrap_or("?");
    match manufacturer_part_number(circuit, chip) {
        Some(part) => format!("{name} [{part}]"),
        None => name.to_string(),
    }
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
