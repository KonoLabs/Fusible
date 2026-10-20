//! Firmware pin bindings generator (`pins.rs`).

use std::collections::{BTreeMap, HashMap};

use crate::netlist::Netlist;
use crate::parser::Circuit;

/// A mapped electrical pin from MCU to a peripheral or net.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinMapping {
    /// Pin / GPIO number.
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
    /// List of resolved pin mappings.
    pub pins: Vec<PinMapping>,
}

/// Extracts MCU pin assignments and peripheral connections from the circuit netlist.
pub fn extract_mcu_pins(
    circuit: &Circuit,
    netlist: &Netlist,
    target_chip: Option<&str>,
) -> Option<McuInfo> {
    let comp_map: HashMap<_, _> = circuit
        .components()
        .filter_map(|c| {
            c.component_id
                .as_deref()
                .map(|id| (id, c.name.as_deref().unwrap_or(id)))
        })
        .collect();

    // 1. Identify the MCU component
    let mcu_comp = select_mcu_component(circuit, target_chip)?;
    let mcu_id = mcu_comp.component_id.as_deref().unwrap_or("U1");
    let mcu_name = mcu_comp.name.as_deref().unwrap_or(mcu_id);
    let chip_name = target_chip
        .map(str::to_string)
        .or_else(|| mcu_comp.kind.clone())
        .unwrap_or_else(|| mcu_name.to_string());

    // 2. Collect all ports belonging to this MCU
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

        // Determine pin number
        let pin_label = port
            .name
            .as_deref()
            .or_else(|| {
                port.port_hints
                    .as_ref()
                    .and_then(|h| h.first().map(String::as_str))
            })
            .unwrap_or(port_id);

        let pin_number = match parse_gpio_number(pin_label, port.pin_number) {
            Some(n) => n,
            None => continue,
        };

        // Find connected net
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

                let r_comp = remote.and_then(|ep| ep.component_id.clone());
                let r_port = remote.and_then(|ep| ep.name.clone());

                let signal = derive_signal_name(
                    &n_name,
                    pin_label,
                    r_comp.as_deref(),
                    r_port.as_deref(),
                    &comp_map,
                );

                (n_name, r_comp, r_port, signal)
            }
            None => (
                format!("unconnected_{pin_label}"),
                None,
                None,
                sanitize_ident(pin_label),
            ),
        };

        // Skip power rails and unconnected NC pins from active GPIO signal lists
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

    pin_mappings.sort_by(|a, b| {
        a.signal_name
            .cmp(&b.signal_name)
            .then_with(|| a.pin_number.cmp(&b.pin_number))
    });

    Some(McuInfo {
        component_id: mcu_id.to_string(),
        name: mcu_name.to_string(),
        chip: chip_name,
        pins: pin_mappings,
    })
}

fn select_mcu_component<'a>(
    circuit: &'a Circuit,
    target_chip: Option<&str>,
) -> Option<&'a crate::parser::Component> {
    let comps: Vec<_> = circuit.components().collect();
    if comps.is_empty() {
        return None;
    }

    // 1. Target chip name match
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

    // 2. Common MCU indicators: "U1", "ESP", "STM", "RP2040", "MCU", "chip"
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

    // 3. Fallback: first chip
    comps
        .into_iter()
        .find(|c| c.kind.as_deref() == Some("chip") || c.kind.as_deref() == Some("simple_chip"))
}

fn parse_gpio_number(label: &str, explicit_pin: Option<u32>) -> Option<u32> {
    let clean = label.trim();

    // Check for "GPIO12", "IO12", "pin12", "P12"
    let prefixes = ["GPIO", "gpio", "IO", "io", "pin", "PIN", "P", "p"];
    for prefix in prefixes {
        if let Some(rest) = clean.strip_prefix(prefix) {
            if let Ok(num) = rest.parse::<u32>() {
                return Some(num);
            }
        }
    }

    if let Ok(num) = clean.parse::<u32>() {
        return Some(num);
    }

    explicit_pin
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
    _comp_map: &HashMap<&str, &str>,
) -> String {
    // Battery and LED conventions
    if net_name.contains("BATT") || pin_label == "GPIO8" {
        return "BATT_SENSE".to_string();
    }
    if net_name.contains("LED") || pin_label == "GPIO48" {
        return "LED_STATUS".to_string();
    }

    // Explicit net name (e.g. "CAM_D0", "MODEM_TX")
    let net_clean = sanitize_ident(net_name);
    if !net_clean.is_empty()
        && !net_clean.starts_with("TRACE_NET")
        && !net_clean.starts_with("UNNAMED")
        && !net_clean.starts_with("SOURCE_NET")
        && !net_clean.starts_with("NET")
        && !net_name.contains('>')
    {
        return net_clean;
    }

    // If connected to remote component port (e.g. CAM1 -> D0)
    if let (Some(comp), Some(port)) = (remote_comp, remote_port) {
        let comp_clean = sanitize_ident(comp);
        let comp_prefix = comp_clean.trim_end_matches(char::is_numeric);
        let port_clean = sanitize_ident(port);
        return format!("{comp_prefix}_{port_clean}");
    }

    // If net name is in display trace format like ".U1 > .GPIO4 to .CAM1 > .D0"
    if let Some((_, dest)) = net_name.split_once(" to ") {
        let trimmed = dest.trim().trim_start_matches('.');
        if let Some((comp, port)) = trimmed.split_once(" > .") {
            let comp_clean = sanitize_ident(comp);
            let comp_prefix = comp_clean.trim_end_matches(char::is_numeric);
            let port_clean = sanitize_ident(port);
            return format!("{comp_prefix}_{port_clean}");
        }
    }

    sanitize_ident(pin_label)
}

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
    } else {
        out
    }
}

/// Generates typed Rust source code (`pins.rs`) from resolved MCU info.
pub fn generate_pins_rs(mcu: &McuInfo, board_name: &str) -> String {
    let mut buf = String::new();

    buf.push_str("//! Generated by Fusible — Electronic Linter & Hardware Bridge.\n");
    buf.push_str("//! DO NOT EDIT MANUALLY. Changes will be overwritten on next build.\n");
    buf.push_str("//!\n");
    buf.push_str(&format!("//! Board: {}\n", board_name));
    buf.push_str(&format!(
        "//! Microcontroller: {} ({})\n",
        mcu.chip, mcu.name
    ));
    buf.push_str("\n#![allow(dead_code)]\n\n");

    // Module with all individual GPIO constants
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

        buf.push_str(&format!(
            "    /// Pin {} (hardware net `{}`){}.\n",
            pin.pin_number, pin.net_name, af_doc
        ));
        buf.push_str(&format!(
            "    pub const {}: u8 = {};\n",
            pin.signal_name, pin.pin_number
        ));
    }
    buf.push_str("}\n\n");

    // Peripheral groupings
    let groups = categorize_pins(&mcu.pins);

    for (group_name, pins) in &groups {
        let struct_name = format!("{group_name}Pins");
        let const_name = format!("{}_PINS", group_name.to_ascii_uppercase());

        buf.push_str(&format!("/// Peripheral pin mapping for `{group_name}`.\n"));
        buf.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
        buf.push_str(&format!("pub struct {struct_name} {{\n"));
        for pin in pins {
            let field_name = to_field_name(&pin.signal_name, group_name);
            buf.push_str(&format!("    pub {field_name}: u8,\n"));
        }
        buf.push_str("}\n\n");

        buf.push_str(&format!(
            "/// Default `{group_name}` hardware configuration.\n"
        ));
        buf.push_str(&format!(
            "pub const {const_name}: {struct_name} = {struct_name} {{\n"
        ));
        for pin in pins {
            let field_name = to_field_name(&pin.signal_name, group_name);
            buf.push_str(&format!("    {field_name}: gpio::{},\n", pin.signal_name));
        }
        buf.push_str("};\n\n");
    }

    // Top-level unified container
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

fn to_field_name(signal_name: &str, group_name: &str) -> String {
    let group_upper = group_name.to_ascii_uppercase();
    let mut clean = signal_name.to_ascii_lowercase();

    // Strip group prefix (e.g. "cam_d0" in Camera group -> "d0")
    let prefix = format!("{}_", group_upper.to_ascii_lowercase());
    if let Some(stripped) = clean.strip_prefix(&prefix) {
        clean = stripped.to_string();
    }

    if clean
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("pin_{clean}")
    } else {
        clean
    }
}
