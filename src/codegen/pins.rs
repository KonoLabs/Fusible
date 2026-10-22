//! Firmware pin bindings generator (`pins.rs`).

use crate::netlist::{NetNode, Netlist};
use crate::parser::{Circuit, Port};
use crate::silicon::model::SiliconTarget;

use super::naming::{dedupe_signal_names, derive_signal_name, sanitize_ident};
pub use super::render::generate_pins_rs;
pub(crate) use super::select::select_mcu_component;
use super::select::{is_power_rail, parse_gpio_number};

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

/// The selected MCU and the context needed to map its ports.
struct McuRef<'a> {
    circuit: &'a Circuit,
    netlist: &'a Netlist,
    id: &'a str,
    name: &'a str,
    silicon: Option<&'static SiliconTarget>,
}

/// Where an MCU port's net leads and the signal name derived from it.
struct NetLink {
    net_name: String,
    remote_component: Option<String>,
    remote_port: Option<String>,
    signal_name: String,
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
    let mcu = McuRef {
        circuit,
        netlist,
        id: mcu_id,
        name: mcu_name,
        silicon: crate::silicon::find_target(&chip_name),
    };
    let mut pin_mappings: Vec<PinMapping> = circuit
        .ports()
        .filter_map(|port| mcu.map_port(port))
        .collect();
    dedupe_signal_names(&mut pin_mappings);
    Some(McuInfo {
        component_id: mcu_id.to_string(),
        name: mcu_name.to_string(),
        chip: chip_name,
        pins: pin_mappings,
    })
}

/// The port's display label: its name, first hint, or id.
fn port_label<'a>(port: &'a Port, port_id: &'a str) -> &'a str {
    port.name
        .as_deref()
        .or_else(|| {
            port.port_hints
                .as_ref()
                .and_then(|h| h.first().map(String::as_str))
        })
        .unwrap_or(port_id)
}

impl McuRef<'_> {
    fn is_mcu(&self, component_id: Option<&str>) -> bool {
        component_id == Some(self.id) || component_id == Some(self.name)
    }

    fn owns(&self, port: &Port) -> bool {
        self.is_mcu(port.source_component_id.as_deref())
            || port
                .port_id
                .as_deref()
                .map(|pid| pid.starts_with(self.id) || pid.starts_with(self.name))
                .unwrap_or(false)
    }

    fn map_port(&self, port: &Port) -> Option<PinMapping> {
        if !self.owns(port) {
            return None;
        }
        let port_id = port.port_id.as_deref()?;
        let pin_label = port_label(port, port_id);
        if self
            .silicon
            .is_some_and(|t| t.find_pin(pin_label).is_none())
        {
            return None;
        }
        let pin_number = parse_gpio_number(pin_label, port.pin_number)?;
        let link = self.net_link(port_id, pin_label);
        let skip = is_power_rail(&link.net_name)
            || is_power_rail(pin_label)
            || pin_label.starts_with("NC");
        (!skip).then(|| PinMapping {
            pin_number,
            pin_label: pin_label.to_string(),
            signal_name: link.signal_name,
            net_name: link.net_name,
            remote_component: link.remote_component,
            remote_port: link.remote_port,
        })
    }

    fn net_link(&self, port_id: &str, pin_label: &str) -> NetLink {
        let node = self
            .netlist
            .port_to_net
            .get(port_id)
            .and_then(|&idx| self.netlist.nets.get(idx));
        match node {
            Some(node) => self.connected_link(node, pin_label),
            None => NetLink {
                net_name: format!("unconnected_{pin_label}"),
                remote_component: None,
                remote_port: None,
                signal_name: sanitize_ident(pin_label),
            },
        }
    }

    fn connected_link(&self, node: &NetNode, pin_label: &str) -> NetLink {
        let net_name = node.display_name().to_string();
        let remote = node
            .endpoints
            .iter()
            .find(|ep| !self.is_mcu(ep.component_id.as_deref()));
        let remote_component = remote
            .and_then(|ep| ep.component_id.as_deref())
            .map(|id| self.component_name(id));
        let remote_port = remote.and_then(|ep| ep.name.clone());
        let signal_name = derive_signal_name(
            &net_name,
            pin_label,
            remote_component.as_deref(),
            remote_port.as_deref(),
        );
        NetLink {
            net_name,
            remote_component,
            remote_port,
            signal_name,
        }
    }

    fn component_name(&self, id: &str) -> String {
        self.circuit
            .components()
            .find(|c| c.component_id.as_deref() == Some(id))
            .and_then(|c| c.name.clone())
            .unwrap_or_else(|| id.to_string())
    }
}
