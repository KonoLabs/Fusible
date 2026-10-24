//! Net connectivity compared between a KiCad board and the Circuit JSON.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::sexpr::KicadFootprint;

type Groups = BTreeSet<BTreeSet<String>>;

const LISTED_PINS: usize = 6;

/// Lists the nets that connect different pins in the design and in KiCad.
///
/// Only parts present on both sides are compared, so an added or renamed part is reported
/// once, by name, and not again as a net change.
pub(super) fn net_differences(
    kicad_fps: &[KicadFootprint],
    elements: &[Value],
    shared: &BTreeSet<String>,
) -> Vec<String> {
    let design = circuit_groups(elements, shared);
    let kicad = kicad_groups(kicad_fps, shared);
    let describe = |who: &str, groups: &Groups, other: &Groups| -> Vec<String> {
        groups
            .difference(other)
            .map(|pins| format!("net differs, {who} connects {}", pin_list(pins)))
            .collect()
    };
    let mut lines = describe("the design", &design, &kicad);
    lines.extend(describe("KiCad", &kicad, &design));
    lines
}

/// Formats a pin group, cut after a few pins.
fn pin_list(pins: &BTreeSet<String>) -> String {
    let shown: Vec<&str> = pins.iter().take(LISTED_PINS).map(String::as_str).collect();
    let more = pins.len().saturating_sub(LISTED_PINS);
    match more {
        0 => shown.join(" "),
        n => format!("{} and {n} more", shown.join(" ")),
    }
}

/// Groups the pads of shared parts by net name, keeping groups of two pads or more.
fn kicad_groups(kicad_fps: &[KicadFootprint], shared: &BTreeSet<String>) -> Groups {
    let mut nets: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for fp in kicad_fps
        .iter()
        .filter(|fp| shared.contains(&fp.designator))
    {
        for pad in fp.pads.iter().filter(|pad| !pad.net.is_empty()) {
            nets.entry(&pad.net)
                .or_default()
                .insert(format!("{}.{}", fp.designator, pad.number));
        }
    }
    nets.into_values().filter(|pins| pins.len() > 1).collect()
}

/// Groups the ports of shared parts by connectivity key, keeping groups of two pins or more.
fn circuit_groups(elements: &[Value], shared: &BTreeSet<String>) -> Groups {
    let names: BTreeMap<&str, &str> = elements
        .iter()
        .filter(|el| el["type"] == "source_component")
        .filter_map(|el| Some((el["source_component_id"].as_str()?, el["name"].as_str()?)))
        .filter(|(_, name)| shared.contains(*name))
        .collect();
    let mut nets: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for port in elements.iter().filter(|el| el["type"] == "source_port") {
        let owner = port["source_component_id"]
            .as_str()
            .and_then(|id| names.get(id));
        let key = port["subcircuit_connectivity_map_key"].as_str();
        let pin = port["pin_number"].as_u64();
        if let (Some(owner), Some(key), Some(pin)) = (owner, key, pin) {
            nets.entry(key)
                .or_default()
                .insert(format!("{owner}.{pin}"));
        }
    }
    nets.into_values().filter(|pins| pins.len() > 1).collect()
}
