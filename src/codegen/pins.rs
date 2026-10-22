//! Firmware pin bindings generator (`pins.rs`).

use std::collections::{BTreeMap, BTreeSet};

use crate::netlist::Netlist;
use crate::parser::Circuit;
use crate::silicon::model::{parse_gpio_index, parse_port_pin};

/// A mapped electrical pin from MCU to a peripheral or net.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinMapping {
    /// Pin number. `GPIOn`/`GPn`/`IOn` labels emit `n`; STM32 port/pin labels
    /// (`PA12`) emit `port_index * 16 + pin` (`PA0` = 0, `PB3` = 19), the
    /// encoding Embassy uses for `AnyPin`.
    pub pin_number: u32,
    /// MCU pin designator (e.g. "GPIO4", "pin3").
    pub pin_label: String,
    /// Rust identifier for the signal (e.g. "CAM_D0").
    pub signal_name: String,
    /// Connected net name.
    pub net_name: String,
    /// Remote connected component designator if resolved (e.g. "CAM1").
    pub remote_component: Option<String>,
    /// Remote connected pin name if resolved (e.g. "D0").
    pub remote_port: Option<String>,
}

/// Metadata and resolved pin mappings for a target microcontroller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McuInfo {
    /// Component ID in the circuit model (e.g. "source_component_0" or "U1").
    pub component_id: String,
    /// Human-readable designator (e.g. "U1" or "ESP32").
    pub name: String,
    /// Silicon chip name (e.g. "esp32-s3" or "stm32g431").
    pub chip: String,
    /// List of resolved pin mappings, sorted by signal name with unique
    /// signal names.
    pub pins: Vec<PinMapping>,
}

/// Extracts MCU pin assignments and peripheral connections from the circuit netlist.
///
/// When the chip is in the silicon database, only ports that resolve to a
/// database pin are emitted, so power and package pins never masquerade as
/// GPIO numbers.
pub fn extract_mcu_pins(
    circuit: &Circuit,
    netlist: &Netlist,
    target_chip: Option<&str>,
) -> Option<McuInfo> {
    let mcu_comp = select_mcu_component(circuit, target_chip)?;
    let mcu_id = mcu_comp.component_id.as_deref().unwrap_or("U1");
    let mcu_name = mcu_comp.name.as_deref().unwrap_or(mcu_id);
    let chip_name = target_chip
        .map(str::to_string)
        .or_else(|| mcu_comp.kind.clone())
        .unwrap_or_else(|| mcu_name.to_string());
    let silicon = crate::silicon::find_target(&chip_name);

    let mut pin_mappings = Vec::new();

    for port in circuit.ports() {
        let is_mcu_port = port.source_component_id.as_deref() == Some(mcu_id)
            || port.source_component_id.as_deref() == Some(mcu_name)
            || port
                .port_id
                .as_deref()
                .map(|pid| pid.starts_with(mcu_id) || pid.starts_with(mcu_name))
                .unwrap_or(false);

        if !is_mcu_port {
            continue;
        }

        let port_id = match port.port_id.as_deref() {
            Some(pid) => pid,
            None => continue,
        };

        let pin_label = port
            .name
            .as_deref()
            .or_else(|| {
                port.port_hints
                    .as_ref()
                    .and_then(|h| h.first().map(String::as_str))
            })
            .unwrap_or(port_id);

        if silicon.is_some_and(|t| t.find_pin(pin_label).is_none()) {
            continue;
        }

        let pin_number = match parse_gpio_number(pin_label, port.pin_number) {
            Some(n) => n,
            None => continue,
        };

        let net_node = netlist
            .port_to_net
            .get(port_id)
            .and_then(|&idx| netlist.nets.get(idx));

        let (net_name, remote_comp, remote_port, derived_signal) = match net_node {
            Some(node) => {
                let n_name = node.display_name().to_string();
                let remote = node.endpoints.iter().find(|ep| {
                    ep.component_id.as_deref() != Some(mcu_id)
                        && ep.component_id.as_deref() != Some(mcu_name)
                });

                let r_comp = remote.and_then(|ep| ep.component_id.as_deref()).map(|id| {
                    circuit
                        .components()
                        .find(|c| c.component_id.as_deref() == Some(id))
                        .and_then(|c| c.name.clone())
                        .unwrap_or_else(|| id.to_string())
                });
                let r_port = remote.and_then(|ep| ep.name.clone());

                let signal =
                    derive_signal_name(&n_name, pin_label, r_comp.as_deref(), r_port.as_deref());

                (n_name, r_comp, r_port, signal)
            }
            None => (
                format!("unconnected_{pin_label}"),
                None,
                None,
                sanitize_ident(pin_label),
            ),
        };

        if is_power_rail(&net_name) || is_power_rail(pin_label) || pin_label.starts_with("NC") {
            continue;
        }

        pin_mappings.push(PinMapping {
            pin_number,
            pin_label: pin_label.to_string(),
            signal_name: derived_signal,
            net_name,
            remote_component: remote_comp,
            remote_port,
        });
    }

    dedupe_signal_names(&mut pin_mappings);

    Some(McuInfo {
        component_id: mcu_id.to_string(),
        name: mcu_name.to_string(),
        chip: chip_name,
        pins: pin_mappings,
    })
}

/// Sorts mappings by `(signal_name, pin_number, pin_label)` and renames
/// colliding signals to `NAME_<pin_number>`, adding `_<n>` if that still collides.
fn dedupe_signal_names(pins: &mut [PinMapping]) {
    pins.sort_by(|a, b| {
        a.signal_name
            .cmp(&b.signal_name)
            .then_with(|| a.pin_number.cmp(&b.pin_number))
            .then_with(|| a.pin_label.cmp(&b.pin_label))
    });

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for pin in pins.iter() {
        *counts.entry(pin.signal_name.clone()).or_default() += 1;
    }

    let mut used = BTreeSet::new();
    for pin in pins.iter_mut() {
        let base = if counts[&pin.signal_name] > 1 {
            format!("{}_{}", pin.signal_name, pin.pin_number)
        } else {
            pin.signal_name.clone()
        };
        pin.signal_name = unique_name(base, &mut used);
    }

    pins.sort_by(|a, b| a.signal_name.cmp(&b.signal_name));
}

/// Returns `base`, or `base_<n>` for the smallest `n >= 2` not yet in `used`,
/// and records the result.
fn unique_name(base: String, used: &mut BTreeSet<String>) -> String {
    let mut candidate = base.clone();
    let mut n = 2;
    while used.contains(&candidate) {
        candidate = format!("{base}_{n}");
        n += 1;
    }
    used.insert(candidate.clone());
    candidate
}

pub(crate) fn select_mcu_component<'a>(
    circuit: &'a Circuit,
    target_chip: Option<&str>,
) -> Option<&'a crate::parser::Component> {
    let comps: Vec<_> = circuit.components().collect();
    if comps.is_empty() {
        return None;
    }

    if let Some(target) = target_chip {
        let target_lower = target.to_ascii_lowercase();
        if let Some(c) = comps.iter().copied().find(|c| {
            c.name
                .as_deref()
                .map(|n| n.to_ascii_lowercase().contains(&target_lower))
                .unwrap_or(false)
                || c.kind
                    .as_deref()
                    .map(|k| k.to_ascii_lowercase().contains(&target_lower))
                    .unwrap_or(false)
                || c.component_id
                    .as_deref()
                    .map(|id| id.to_ascii_lowercase().contains(&target_lower))
                    .unwrap_or(false)
        }) {
            return Some(c);
        }
    }

    let mcu_keywords = ["esp", "stm", "mcu", "soc", "rp2040", "u1"];
    for kw in mcu_keywords {
        if let Some(c) = comps.iter().copied().find(|c| {
            c.name
                .as_deref()
                .map(|n| n.to_ascii_lowercase().contains(kw))
                .unwrap_or(false)
                || c.component_id
                    .as_deref()
                    .map(|id| id.to_ascii_lowercase().contains(kw))
                    .unwrap_or(false)
        }) {
            return Some(c);
        }
    }

    comps
        .into_iter()
        .find(|c| c.kind.as_deref() == Some("chip") || c.kind.as_deref() == Some("simple_chip"))
}

/// Resolves the emitted pin number for a port label. See
/// [`PinMapping::pin_number`] for the encoding.
fn parse_gpio_number(label: &str, explicit_pin: Option<u32>) -> Option<u32> {
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

fn is_power_rail(name: &str) -> bool {
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

fn derive_signal_name(
    net_name: &str,
    pin_label: &str,
    remote_comp: Option<&str>,
    remote_port: Option<&str>,
) -> String {
    let net_clean = sanitize_ident(net_name);
    if !net_clean.starts_with("TRACE_NET")
        && !net_clean.starts_with("UNNAMED")
        && !net_clean.starts_with("SOURCE_NET")
        && !net_clean.starts_with("NET")
        && !net_name.contains('>')
    {
        return net_clean;
    }

    if let (Some(comp), Some(port)) = (remote_comp, remote_port) {
        return remote_signal_name(comp, port);
    }

    if let Some((_, dest)) = net_name.split_once(" to ") {
        let trimmed = dest.trim().trim_start_matches('.');
        if let Some((comp, port)) = trimmed.split_once(" > .") {
            return remote_signal_name(comp, port);
        }
    }

    sanitize_ident(pin_label)
}

/// Builds `COMP_PORT` from a remote designator with its trailing index
/// stripped (`CAM1`, `D0` -> `CAM_D0`).
fn remote_signal_name(comp: &str, port: &str) -> String {
    let comp_clean = sanitize_ident(comp);
    let comp_prefix = comp_clean.trim_end_matches(|c: char| c.is_ascii_digit());
    sanitize_ident(&format!("{comp_prefix}_{port}"))
}

/// Converts arbitrary text to an `UPPER_SNAKE` Rust identifier. Names that
/// would start with a digit get a `PIN_` prefix; empty input becomes `PIN`.
fn sanitize_ident(s: &str) -> String {
    let mut out = String::new();
    let mut prev_underscore = false;

    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
            prev_underscore = false;
        } else if !prev_underscore && !out.is_empty() {
            out.push('_');
            prev_underscore = true;
        }
    }

    while out.ends_with('_') {
        out.pop();
    }

    if out.is_empty() {
        "PIN".to_string()
    } else if out.starts_with(|c: char| c.is_ascii_digit()) {
        format!("PIN_{out}")
    } else {
        out
    }
}

/// Renders text for a `//` or `///` comment: control characters (newlines,
/// carriage returns, tabs) are escaped so the text stays on one line.
fn comment_text(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_control() {
                c.escape_default().to_string()
            } else {
                c.to_string()
            }
        })
        .collect()
}

/// Smallest unsigned integer type holding every pin number.
fn pin_type(pins: &[PinMapping]) -> &'static str {
    match pins.iter().map(|p| p.pin_number).max().unwrap_or(0) {
        0..=0xFF => "u8",
        0x100..=0xFFFF => "u16",
        _ => "u32",
    }
}

/// Generates typed Rust source code (`pins.rs`) from resolved MCU info.
pub fn generate_pins_rs(mcu: &McuInfo, board_name: &str) -> String {
    let mut buf = String::new();
    let ty = pin_type(&mcu.pins);

    buf.push_str("//! Generated by Fusible — Electronic Linter & Hardware Bridge.\n");
    buf.push_str("//! DO NOT EDIT MANUALLY. Changes will be overwritten on next build.\n");
    buf.push_str("//!\n");
    buf.push_str(&format!("//! Board: {}\n", comment_text(board_name)));
    buf.push_str(&format!(
        "//! Microcontroller: {} ({})\n",
        comment_text(&mcu.chip),
        comment_text(&mcu.name)
    ));
    buf.push_str("\n#![allow(dead_code)]\n\n");

    buf.push_str("/// Hardware GPIO pin constants extracted from the circuit netlist.\n");
    buf.push_str("pub mod gpio {\n");
    let target = crate::silicon::find_target(&mcu.chip);
    for pin in &mcu.pins {
        let af_doc = target
            .and_then(|t| {
                let spec = t.find_pin(&pin.pin_label)?;
                let sig = crate::silicon::PeripheralSignal::parse(&pin.signal_name)
                    .or_else(|| crate::silicon::PeripheralSignal::parse(&pin.net_name))?;
                let af = spec.get_af(&sig)?;
                Some(format!(" [{}: AF{}]", af.af_name, af.af_number))
            })
            .unwrap_or_default();
        let port_doc = if parse_port_pin(&pin.pin_label).is_some() {
            format!(" = `{}` (port * 16 + pin)", comment_text(&pin.pin_label))
        } else {
            String::new()
        };

        buf.push_str(&format!(
            "    /// Pin {}{} (hardware net `{}`){}.\n",
            pin.pin_number,
            port_doc,
            comment_text(&pin.net_name),
            af_doc
        ));
        buf.push_str(&format!(
            "    pub const {}: {ty} = {};\n",
            pin.signal_name, pin.pin_number
        ));
    }
    buf.push_str("}\n\n");

    let groups = categorize_pins(&mcu.pins);

    for (group_name, pins) in &groups {
        let struct_name = format!("{group_name}Pins");
        let const_name = format!("{}_PINS", group_name.to_ascii_uppercase());
        let fields = field_names(pins, group_name);

        buf.push_str(&format!("/// Peripheral pin mapping for `{group_name}`.\n"));
        buf.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
        buf.push_str(&format!("pub struct {struct_name} {{\n"));
        for field_name in &fields {
            buf.push_str(&format!("    pub {field_name}: {ty},\n"));
        }
        buf.push_str("}\n\n");

        buf.push_str(&format!(
            "/// Default `{group_name}` hardware configuration.\n"
        ));
        buf.push_str(&format!(
            "pub const {const_name}: {struct_name} = {struct_name} {{\n"
        ));
        for (pin, field_name) in pins.iter().zip(&fields) {
            buf.push_str(&format!("    {field_name}: gpio::{},\n", pin.signal_name));
        }
        buf.push_str("};\n\n");
    }

    buf.push_str("/// Complete board pin assignment container.\n");
    buf.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
    buf.push_str("pub struct BoardPins {\n");
    for group_name in groups.keys() {
        let field_name = group_name.to_ascii_lowercase();
        let struct_name = format!("{group_name}Pins");
        buf.push_str(&format!("    pub {field_name}: {struct_name},\n"));
    }
    buf.push_str("}\n\n");

    buf.push_str("/// Verified hardware board pin assignments.\n");
    buf.push_str("pub const BOARD_PINS: BoardPins = BoardPins {\n");
    for group_name in groups.keys() {
        let field_name = group_name.to_ascii_lowercase();
        let const_name = format!("{}_PINS", group_name.to_ascii_uppercase());
        buf.push_str(&format!("    {field_name}: {const_name},\n"));
    }
    buf.push_str("};\n");

    buf
}

fn categorize_pins(pins: &[PinMapping]) -> BTreeMap<String, Vec<&PinMapping>> {
    let mut groups: BTreeMap<String, Vec<&PinMapping>> = BTreeMap::new();

    for pin in pins {
        let upper = pin.signal_name.to_ascii_uppercase();
        let group = if upper.contains("CAM") {
            "Camera".to_string()
        } else if upper.contains("MODEM") || upper.contains("SIM") || upper.contains("LTE") {
            "Modem".to_string()
        } else if upper.contains("RADIO") || upper.contains("RF") || upper.contains("NRF") {
            "Radio".to_string()
        } else if upper.contains("BATT") || upper.contains("POWER") || upper.contains("ADC") {
            "Power".to_string()
        } else if upper.contains("LED") {
            "Led".to_string()
        } else {
            "General".to_string()
        };

        groups.entry(group).or_default().push(pin);
    }

    groups
}

/// Unique struct field names for one peripheral group, in pin order.
fn field_names(pins: &[&PinMapping], group_name: &str) -> Vec<String> {
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
